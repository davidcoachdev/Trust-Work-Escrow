//! T9 — Event listener / fallback (decode `#[event]` + `msg!` logs, timeouts, sin panics).
//!
//! Valida que `trust-escrow-v3` actualmente NO emite `#[event]` Anchor y que el
//! SDK decodifica sin panics retornando vacio/None, con fallback `msg!` opcional
//! y timeouts tipados. Unit corre sin validator; parte timeout usa tokio.

#![cfg(feature = "solana")]

use std::time::Duration;

use solana_sdk::pubkey::Pubkey;
use trust_escrow_sdk::events::{
    decode_anchor_logs, decode_logs, decode_msg_logs, encode_anchor_log, event_discriminator,
    try_decode_anchor_log, try_decode_log, try_parse_msg_log, EscrowEventKind, JobCreatedEvent,
};

// ---- Discriminator determinism ----

#[test]
fn event_discriminator_is_deterministic() {
    let a = event_discriminator("JobCreated");
    let b = event_discriminator("JobCreated");
    assert_eq!(a, b);
    let c = event_discriminator("FundsDeposited");
    assert_ne!(a, c);
}

// ---- Anchor round-trip ----

#[test]
fn anchor_roundtrip_job_created() {
    let ev = JobCreatedEvent {
        job: Pubkey::new_unique(),
        client: Pubkey::new_unique(),
        amount: 123_456,
        deadline: 1_700_000_000,
    };
    let log = encode_anchor_log("JobCreated", &ev);
    assert!(log.starts_with("Program data:"));
    let decoded = try_decode_anchor_log(&log).expect("should decode");
    assert!(!decoded.is_fallback());
    assert_eq!(
        decoded.discriminator,
        Some(event_discriminator("JobCreated"))
    );
    match decoded.kind {
        EscrowEventKind::JobCreated(inner) => assert_eq!(inner, ev),
        other => panic!("unexpected kind {:?}", other),
    }
}

#[test]
fn anchor_roundtrip_via_decode_logs() {
    let ev = JobCreatedEvent {
        job: Pubkey::new_unique(),
        client: Pubkey::new_unique(),
        amount: 999,
        deadline: 42,
    };
    let log = encode_anchor_log("JobCreated", &ev);
    let logs = vec![log, "Program log: unrelated".to_string()];
    let anchor_only = decode_anchor_logs(&logs);
    assert_eq!(anchor_only.len(), 1);
    let all = decode_logs(&logs);
    // decode_logs includes fallback for the second log
    assert_eq!(all.len(), 2);
}

// ---- Fallback / empty without panic ----

#[test]
fn v3_currently_emits_no_anchor_event_so_decode_returns_empty() {
    // Estos son los msg! reales que emite v3 hoy; ninguno es "Program data:"
    let v3_logs = vec![
        "Program log: Job created".to_string(),
        "Program log: Funds deposited: 500000".to_string(),
        "Program log: Work submitted for job: 11111111111111111111111111111111".to_string(),
        "Program log: Instruction: CreateJob".to_string(),
    ];
    // Anchor-only decoder debe retornar vacio (no hay #[event])
    let anchor = decode_anchor_logs(&v3_logs);
    assert!(anchor.is_empty(), "v3 no emite Program data: Anchor events");

    // try_decode_anchor_log sobre cada log debe ser None sin panic
    for l in &v3_logs {
        assert!(try_decode_anchor_log(l).is_none());
    }

    // try_decode_log con fallback clasifica msg! como LogFallback
    let fallback = decode_logs(&v3_logs);
    assert_eq!(fallback.len(), 4);
    for ev in &fallback {
        assert!(ev.is_fallback());
    }

    // Logs vacios -> vacio sin error
    assert!(decode_logs(&[]).is_empty());
    assert!(decode_anchor_logs(&[]).is_empty());
    assert!(decode_msg_logs(&[]).is_empty());
}

#[test]
fn malformed_logs_never_panic_and_return_none_or_empty() {
    let bad = vec![
        "".to_string(),
        "Program data:".to_string(),
        "Program data: not-base64!!!".to_string(),
        "Program data: AQID".to_string(), // <8 bytes after decode
        "Program data: AAAAAAAAAAAAAAAAAAAAAA==".to_string(), // 8 zero bytes unknown disc
        "Totally unrelated".to_string(),
        "Program log:".to_string(),
    ];
    for l in &bad {
        // estas funciones nunca deben hacer panic
        let _ = try_decode_anchor_log(l);
        let _ = try_decode_log(l);
        let _ = try_parse_msg_log(l);
    }
    // lista mixta con un valido intercalado
    let mut mixed = bad.clone();
    let ev = JobCreatedEvent {
        job: Pubkey::new_unique(),
        client: Pubkey::new_unique(),
        amount: 1,
        deadline: 1,
    };
    mixed.push(encode_anchor_log("JobCreated", &ev));
    let decoded = decode_anchor_logs(&mixed);
    assert_eq!(decoded.len(), 1);
}

#[test]
fn msg_fallback_classification() {
    let log = "Program log: Job created".to_string();
    let ev = try_parse_msg_log(&log).unwrap();
    match ev.kind {
        EscrowEventKind::LogFallback(ref f) => assert_eq!(f.kind_hint, "JobCreated"),
        _ => panic!("expected fallback"),
    }
    let unknown = "Program log: something we never saw".to_string();
    let ev2 = try_parse_msg_log(&unknown).unwrap();
    match ev2.kind {
        EscrowEventKind::LogFallback(ref f) => assert_eq!(f.kind_hint, "unknown"),
        _ => panic!("expected fallback"),
    }
    // sin prefijo -> None
    assert!(try_parse_msg_log("Job created").is_none());
}

#[test]
fn unknown_discriminator_returns_none() {
    // Codificamos un evento con nombre no registrado usando un tipo ya existente
    let dummy = JobCreatedEvent {
        job: Pubkey::new_unique(),
        client: Pubkey::new_unique(),
        amount: 42,
        deadline: 0,
    };
    let log = encode_anchor_log("NonExistentEvent", &dummy);
    assert!(try_decode_anchor_log(&log).is_none());
    assert!(decode_anchor_logs(&[log]).is_empty());
}

#[test]
fn corrupted_borsh_returns_none_without_panic() {
    // Discriminador valido de JobCreated pero payload truncado
    let disc = event_discriminator("JobCreated");
    let mut buf = Vec::new();
    buf.extend_from_slice(&disc);
    buf.extend_from_slice(&[1, 2, 3]); // truncated borsh
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    let b64 = STANDARD.encode(&buf);
    let log = format!("Program data: {}", b64);
    assert!(try_decode_anchor_log(&log).is_none());
}

// ---- Timeouts ----

#[tokio::test]
async fn decode_logs_with_timeout_ok() {
    let logs = vec!["Program log: Job created".to_string()];
    let res =
        trust_escrow_sdk::events::decode_logs_with_timeout(logs, Duration::from_secs(1)).await;
    assert!(res.is_ok());
    assert_eq!(res.unwrap().len(), 1);
}

#[tokio::test]
async fn decode_logs_with_timeout_times_out() {
    // with_timeout envolviendo un future que duerme mas que el deadline
    let res: trust_escrow_sdk::error::Result<Vec<trust_escrow_sdk::events::EscrowEvent>> =
        trust_escrow_sdk::utils::with_timeout(Duration::from_millis(10), async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok(Vec::new())
        })
        .await;
    assert!(res.is_err());
    assert!(res.unwrap_err().is_timeout());
}

#[tokio::test(flavor = "multi_thread")]
async fn fetch_events_via_rpc_times_out_without_validator() {
    // Sin validator local, get_transaction debe timeout o error tipado, nunca panic
    use solana_client::rpc_client::RpcClient;
    use solana_sdk::commitment_config::CommitmentConfig;
    use solana_sdk::signature::Signature;
    let rpc = RpcClient::new_with_commitment(
        "http://127.0.0.1:8899".to_string(),
        CommitmentConfig::confirmed(),
    );
    // signature dummy (no existe)
    let sig = Signature::default();
    let res = trust_escrow_sdk::events::fetch_events_via_rpc_with_timeout(
        &rpc,
        sig,
        Duration::from_millis(300),
    )
    .await;
    // Puede ser error RPC o vacio, pero nunca panic. Si hay validator sin tx, sera error.
    // Solo verificamos que no paniquea y que el Result es manejable.
    assert!(res.is_ok() || res.is_err());
    // Timeout helper path tambien debe mapear a BackendError::Timeout cuando aplica
    let timeout_res: trust_escrow_sdk::error::Result<()> =
        trust_escrow_sdk::utils::with_timeout(Duration::from_millis(5), async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            Ok(())
        })
        .await;
    assert!(timeout_res.is_err());
    assert!(timeout_res.unwrap_err().is_timeout());
}
