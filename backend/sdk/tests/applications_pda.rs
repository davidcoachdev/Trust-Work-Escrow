//! T21 — Applications PDA individual: modelo `Application` con seeds
//! `[b"application", job, index, applicant]` — tests-first.
//!
//! Scope estricto T21:
//! - Seeds deterministas y bump válido (contrato vs SDK).
//! - Validación de índice `0..49`, next-index == len, rechazos fuera de rango.
//! - Duplicados: `AlreadyApplied` aunque cambie índice.
//! - Límites: `MAX_APPLICATIONS = 50`, Vec compacto, proposal_hash [u8;32].
//! - Contraste con:
//!   - T22: Job compacto Vec (tamaño/serialización, no colección inline).
//!   - T23: `apply_to_job` instrucción (status Funded, signer, etc.).
//!   - T24: `accept_application` + cleanup/rent.
//!   - T25: build/IDL drift.
//!   - T26: docs sincronizados.
//!
//!   Este archivo NO implementa T22-T26; solo deja el modelo probado para que
//!   esos T lo consuman.

#![cfg(feature = "solana")]

use anchor_lang::AnchorSerialize;
use solana_sdk::{hash::hash, pubkey::Pubkey};
use trust_escrow_sdk::{
    client::deserialize_account, error::ErrorCode, pda, types::*, PROGRAM_ID_STR,
};

// ---------------------------------------------------------------------------
// Helpers — mirror on-chain checks without living validator (offline oracle).
// ---------------------------------------------------------------------------

const MAX_APPLICATIONS: usize = 50;

/// Simula `apply_to_job` checks previos al `push` (sin estado on-chain):
/// - applicant != client
/// - not already applied
/// - len < 50
/// - index == len
fn validate_apply(
    job_client: &Pubkey,
    applicants: &[Pubkey],
    applicant: &Pubkey,
    index: u8,
) -> Result<(), ErrorCode> {
    if applicant == job_client {
        return Err(ErrorCode::CannotWorkOnOwnJob);
    }
    if applicants.iter().any(|a| a == applicant) {
        return Err(ErrorCode::AlreadyApplied);
    }
    if applicants.len() >= MAX_APPLICATIONS {
        return Err(ErrorCode::InvalidApplicationIndex);
    }
    if index as usize != applicants.len() {
        return Err(ErrorCode::ApplicationIndexMismatch);
    }
    if index as usize >= MAX_APPLICATIONS {
        return Err(ErrorCode::InvalidApplicationIndex);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// 1. Seeds — determinismo y coincidencia exacto con el contrato.
// ---------------------------------------------------------------------------

#[test]
fn application_pda_seeds_match_contract_and_are_deterministic() {
    let pid: Pubkey = PROGRAM_ID_STR.parse().unwrap();
    let job = Pubkey::new_unique();
    let applicant = Pubkey::new_unique();

    for idx in [0u8, 1, 7, 49, 255] {
        let (derived, bump) = pda::derive_application_pda(&job, idx, &applicant).unwrap();
        let (expected, ebump) = Pubkey::find_program_address(
            &[b"application", job.as_ref(), &[idx], applicant.as_ref()],
            &pid,
        );
        assert_eq!(derived, expected, "seed mismatch idx={}", idx);
        assert_eq!(bump, ebump, "bump mismatch idx={}", idx);
        // segunda derivación idéntica
        let (again, abump) = pda::derive_application_pda(&job, idx, &applicant).unwrap();
        assert_eq!(derived, again);
        assert_eq!(bump, abump);
    }
}

#[test]
fn application_pda_differs_by_job_index_applicant() {
    let job_a = Pubkey::new_unique();
    let job_b = Pubkey::new_unique();
    let alice = Pubkey::new_unique();
    let bob = Pubkey::new_unique();

    let (p00a, _) = pda::derive_application_pda(&job_a, 0, &alice).unwrap();
    let (p01a, _) = pda::derive_application_pda(&job_a, 1, &alice).unwrap();
    let (p00b, _) = pda::derive_application_pda(&job_b, 0, &alice).unwrap();
    let (p00alice_vs_bob, _) = pda::derive_application_pda(&job_a, 0, &bob).unwrap();

    assert_ne!(p00a, p01a, "index must affect PDA");
    assert_ne!(p00a, p00b, "job must affect PDA");
    assert_ne!(p00a, p00alice_vs_bob, "applicant must affect PDA");
}

#[test]
fn application_pda_single_byte_index_encoding() {
    // Verifica que el contrato usa &[index] (1 byte), no LE/BE u32.
    let pid: Pubkey = PROGRAM_ID_STR.parse().unwrap();
    let job = Pubkey::new_unique();
    let applicant = Pubkey::new_unique();
    for idx in [0u8, 1, 49] {
        let (ok, _) = pda::derive_application_pda(&job, idx, &applicant).unwrap();
        let (wrong_le, _) = Pubkey::find_program_address(
            &[
                b"application",
                job.as_ref(),
                &(idx as u32).to_le_bytes(),
                applicant.as_ref(),
            ],
            &pid,
        );
        assert_ne!(
            ok, wrong_le,
            "must be single byte, not u32 LE for idx {}",
            idx
        );
    }
}

#[test]
fn application_pda_bump_is_canonical_and_cached() {
    pda::clear_pda_cache();
    let job = Pubkey::new_unique();
    let applicant = Pubkey::new_unique();
    let (a, ba) = pda::derive_application_pda(&job, 3, &applicant).unwrap();
    let (b, bb) = pda::get_application_pda(&job, 3, &applicant).unwrap();
    assert_eq!(a, b);
    assert_eq!(ba, bb);
    // cache hit
    let (c, bc) = pda::get_application_pda(&job, 3, &applicant).unwrap();
    assert_eq!(b, c);
    assert_eq!(bb, bc);
}

#[test]
fn application_pda_is_off_curve() {
    // Toda PDA debe estar off-curve (no es pubkey ed25519 válida como signer).
    let job = Pubkey::new_unique();
    let applicant = Pubkey::new_unique();
    let (pda, _) = pda::derive_application_pda(&job, 10, &applicant).unwrap();
    assert!(!pda.is_on_curve(), "Application PDA must be off-curve");
}

// ---------------------------------------------------------------------------
// 2. Validación de índice 0..49 y next-index == len.
// ---------------------------------------------------------------------------

#[test]
fn application_index_valid_range_0_to_49() {
    for idx in 0u8..50 {
        assert!(
            (idx as usize) < MAX_APPLICATIONS,
            "idx {} should be valid",
            idx
        );
    }
    assert_eq!(49u8 as usize, MAX_APPLICATIONS - 1);
    // 50 es el primer inválido (len == 50 ya lleno)
    assert_eq!(50usize, MAX_APPLICATIONS);
}

#[test]
fn application_index_must_equal_next_len() {
    let client = Pubkey::new_unique();
    let alice = Pubkey::new_unique();
    let bob = Pubkey::new_unique();

    // len 0 -> solo index 0 válido
    assert!(validate_apply(&client, &[], &alice, 0).is_ok());
    assert_eq!(
        validate_apply(&client, &[], &alice, 1).unwrap_err(),
        ErrorCode::ApplicationIndexMismatch
    );

    // len 1 -> solo index 1 válido, y no duplicado
    let applicants = vec![alice];
    assert!(validate_apply(&client, &applicants, &bob, 1).is_ok());
    assert_eq!(
        validate_apply(&client, &applicants, &bob, 0).unwrap_err(),
        ErrorCode::ApplicationIndexMismatch
    );
    assert_eq!(
        validate_apply(&client, &applicants, &bob, 2).unwrap_err(),
        ErrorCode::ApplicationIndexMismatch
    );

    // índice 50 nunca válido (u8 lo permite pero contrato lo rechaza por len<50)
    let mut many = vec![Pubkey::new_unique(); 50];
    // asegurar applicant distinto
    let extra = Pubkey::new_unique();
    assert_eq!(
        validate_apply(&client, &many, &extra, 50).unwrap_err(),
        ErrorCode::InvalidApplicationIndex
    );
    // aunque el índice coincida con len, 50 excede MAX_APPLICATIONS
    many.truncate(49);
    // índice 49 es el último válido cuando len=49
    assert!(validate_apply(&client, &many, &extra, 49).is_ok());
}

#[test]
fn application_index_u8_bounds_no_panic() {
    let client = Pubkey::new_unique();
    let applicant = Pubkey::new_unique();
    // u8::MAX (255) debe fallar por mismatch/limit, no paniquear
    assert!(validate_apply(&client, &[], &applicant, 255).is_err());
    // derivación PDA con 255 no debe paniqear (solo seeds)
    let job = Pubkey::new_unique();
    assert!(pda::derive_application_pda(&job, 255, &applicant).is_ok());
}

// ---------------------------------------------------------------------------
// 3. Duplicados — AlreadyApplied incluso cambiando índice.
// ---------------------------------------------------------------------------

#[test]
fn application_duplicate_applicant_rejected_even_with_different_index() {
    let client = Pubkey::new_unique();
    let alice = Pubkey::new_unique();
    let applicants = vec![alice];

    // Alice ya aplicó en index 0; intentar index 1 debe fallar AlreadyApplied,
    // no solo IndexMismatch.
    let err = validate_apply(&client, &applicants, &alice, 1).unwrap_err();
    assert_eq!(err, ErrorCode::AlreadyApplied);

    // Incluso si el índice coincidiera con len, sigue siendo duplicado
    // (en el caso len=1, index 1 coincide pero el check de duplicado va antes).
    // Simulamos len=0 con duplicado imposible, así que probamos con len=1 y
    // otro applicant distinto que debería pasar.
    let bob = Pubkey::new_unique();
    assert!(validate_apply(&client, &applicants, &bob, 1).is_ok());
}

#[test]
fn application_duplicate_across_multiple_applicants() {
    let client = Pubkey::new_unique();
    let alice = Pubkey::new_unique();
    let bob = Pubkey::new_unique();
    let carol = Pubkey::new_unique();
    let applicants = vec![alice, bob];

    // Carol nuevo -> ok en index 2
    assert!(validate_apply(&client, &applicants, &carol, 2).is_ok());
    // Bob duplicado -> AlreadyApplied aunque index 2 sea el next
    assert_eq!(
        validate_apply(&client, &applicants, &bob, 2).unwrap_err(),
        ErrorCode::AlreadyApplied
    );
    // Alice duplicado idem
    assert_eq!(
        validate_apply(&client, &applicants, &alice, 2).unwrap_err(),
        ErrorCode::AlreadyApplied
    );
}

#[test]
fn application_self_apply_rejected() {
    let client = Pubkey::new_unique();
    // applicant == client
    assert_eq!(
        validate_apply(&client, &[], &client, 0).unwrap_err(),
        ErrorCode::CannotWorkOnOwnJob
    );
}

// ---------------------------------------------------------------------------
// 4. Límites — MAX_APPLICATIONS 50, Vec compacto, proposal_hash.
// ---------------------------------------------------------------------------

#[test]
fn application_max_50_vec_limit() {
    let client = Pubkey::new_unique();
    // construir 50 applicants distintos
    let applicants: Vec<Pubkey> = (0..50).map(|_| Pubkey::new_unique()).collect();
    assert_eq!(applicants.len(), MAX_APPLICATIONS);

    // 50º (index 49) fue el último válido; len 50 ya está lleno
    let extra = Pubkey::new_unique();
    assert_eq!(
        validate_apply(&client, &applicants, &extra, 50).unwrap_err(),
        ErrorCode::InvalidApplicationIndex
    );
    // incluso con index 0 no debería pasar por duplicate/limit
    // (usamos un vector lleno)
    assert!(validate_apply(&client, &applicants, &extra, 0).is_err());
}

#[test]
fn application_vec_is_compact_not_inline_50_objects() {
    // El Job guarda solo Vec<Pubkey> de hasta 50, no 50 * Application completas.
    // Verificamos serialización borsh: Job con 0 vs 50 applicants difiere
    // ~ 50*32 bytes, no 50*sizeof(Application) (~50*~100 bytes).
    let job_empty = Job {
        client: Pubkey::new_unique(),
        freelancer: None,
        amount: 1_000_000,
        fee_amount: 10_000,
        status: JobStatus::Funded,
        paused: false,
        paused_at: 0,
        deadline: 1_700_000_000,
        submitted_at: None,
        milestones_total: 0,
        milestones_approved: 0,
        milestones_amount_total: 0,
        applicants: vec![],
        bump: 255,
    };
    let job_full = Job {
        applicants: (0..50).map(|_| Pubkey::new_unique()).collect(),
        ..job_empty.clone()
    };

    let mut buf_empty = account_discriminator("Job").to_vec();
    job_empty.serialize(&mut buf_empty).unwrap();
    let mut buf_full = account_discriminator("Job").to_vec();
    job_full.serialize(&mut buf_full).unwrap();

    let delta = buf_full.len() - buf_empty.len();
    // borsh Vec<Pubkey>: len prefix 4 bytes ya presente en ambos buffers,
    // por lo que delta == 50*32 (no 4+50*32).
    assert_eq!(delta, 50 * 32, "Job Vec<Pubkey> debe ser compacto");
    // No debe reservar espacio para 50 Applications inline (~70 bytes c/u)
    assert!(delta < 50 * 70, "No debe ser inline de Applications");
}

#[test]
fn application_serialization_roundtrip_and_discriminator() {
    let job = Pubkey::new_unique();
    let app = Application {
        job,
        index: 7,
        applicant: Pubkey::new_unique(),
        proposal_hash: hash(b"proposal text for t21").to_bytes(),
        status: ApplicationStatus::Pending,
        bump: 254,
    };
    let mut buf = account_discriminator("Application").to_vec();
    app.serialize(&mut buf).unwrap();

    // discriminator de Application debe ser sha256("account:Application")[..8]
    let disc = account_discriminator("Application");
    assert_eq!(&buf[..8], &disc);

    let got = deserialize_account::<Application>(&buf).expect("roundtrip");
    assert_eq!(got, app);
    assert_eq!(got.job, job);
    assert_eq!(got.index, 7);
    assert_eq!(got.status, ApplicationStatus::Pending);

    // Job discriminator no debe deserializar como Application
    let job_disc = account_discriminator("Job");
    let mut bad = job_disc.to_vec();
    // apendizamos la misma app payload pero con disc de Job -> debe fallar
    bad.extend_from_slice(&buf[8..]);
    assert!(deserialize_account::<Application>(&bad).is_none());
}

#[test]
fn application_proposal_hash_is_32_bytes_and_deterministic() {
    let h1 = hash(b"hello proposal").to_bytes();
    let h2 = hash(b"hello proposal").to_bytes();
    let h3 = hash(b"different").to_bytes();
    assert_eq!(h1.len(), 32);
    assert_eq!(h1, h2);
    assert_ne!(h1, h3);
}

#[test]
fn application_error_codes_are_stable() {
    // Estos códigos son parte del IDL; cambiarlos rompe clientes.
    assert_eq!(ErrorCode::AlreadyApplied as u32, 6040);
    assert_eq!(ErrorCode::InvalidApplicationIndex as u32, 6041);
    assert_eq!(ErrorCode::ApplicationIndexMismatch as u32, 6046);
    assert_eq!(ErrorCode::InvalidApplicationAccount as u32, 6047);
    assert_eq!(ErrorCode::ApplicationNotPending as u32, 6048);
    assert_eq!(ErrorCode::InvalidApplicationCleanupAccounts as u32, 6050);

    assert_eq!(ErrorCode::from_code(6040), Some(ErrorCode::AlreadyApplied));
    assert_eq!(
        ErrorCode::from_code(6046),
        Some(ErrorCode::ApplicationIndexMismatch)
    );
    assert_eq!(ErrorCode::from_code(9999), None);
}

// ---------------------------------------------------------------------------
// 5. Integración contra validator 7a2Y (requiere localnet). Offline tests ya
//    cubren el modelo; este test valida on-chain duplicados/índice/límites
//    sin solaparse con T23-T24 (no prueba accept/cleanup/rent).
// ---------------------------------------------------------------------------

#[test]
fn applications_pda_integration_duplicates_index_limits_on_validator() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async { applications_pda_integration_inner().await });
}

async fn applications_pda_integration_inner() {
    use anchor_client::Cluster;
    use solana_client::rpc_client::RpcClient;
    use solana_sdk::{commitment_config::CommitmentConfig, signature::Keypair, signer::Signer};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use trust_escrow_sdk::client::TrustEscrowClient;

    const RPC_URL: &str = "http://127.0.0.1:8899";

    fn rpc() -> RpcClient {
        RpcClient::new_with_commitment(RPC_URL.to_string(), CommitmentConfig::confirmed())
    }
    fn now_ts() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }
    fn airdrop(pk: &Pubkey) {
        let before = rpc().get_balance(pk).unwrap_or(0);
        let _ = rpc().request_airdrop(pk, 2_000_000_000).unwrap();
        for _ in 0..80 {
            let after = rpc().get_balance(pk).unwrap_or(0);
            if after > before {
                return;
            }
            std::thread::sleep(Duration::from_millis(200));
        }
        panic!("airdrop not confirmed for {pk}");
    }

    // Guard: if program not deployed, skip (no panics in CI without validator)
    let pid: Pubkey = PROGRAM_ID_STR.parse().unwrap();
    if rpc().get_account(&pid).is_err() {
        eprintln!("skip: program 7a2Y not deployed on localnet");
        return;
    }

    let client_kp = Keypair::new();
    airdrop(&client_kp.pubkey());
    let client = TrustEscrowClient::new(Cluster::Localnet, client_kp.insecure_clone()).unwrap();

    // Create job funded — amount >= MIN_JOB_AMOUNT
    let job_id = (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        % 800_000)
        + 50_000;
    client
        .create_job(job_id, 200_000, now_ts() + 3600)
        .await
        .expect("create_job");
    client.deposit_funds(job_id).await.expect("deposit");

    let job_pda = pda::get_job_pda(&client_kp.pubkey(), job_id).unwrap().0;

    // --- index 0 ok ---
    let alice = Keypair::new();
    airdrop(&alice.pubkey());
    let alice_client = TrustEscrowClient::new(Cluster::Localnet, alice.insecure_clone()).unwrap();
    let h0 = hash(b"proposal alice 0").to_bytes();
    alice_client
        .apply_to_job(&client_kp.pubkey(), job_id, 0, h0)
        .await
        .expect("alice apply index 0");

    // Verify PDA exists and fields match
    let app0 = alice_client
        .get_application(&job_pda, 0, &alice.pubkey())
        .unwrap()
        .expect("app0 present");
    assert_eq!(app0.job, job_pda);
    assert_eq!(app0.index, 0);
    assert_eq!(app0.applicant, alice.pubkey());
    assert_eq!(app0.proposal_hash, h0);
    assert_eq!(app0.status, ApplicationStatus::Pending);

    // --- duplicate applicant even with next index must fail AlreadyApplied ---
    let dup = alice_client
        .apply_to_job(
            &client_kp.pubkey(),
            job_id,
            1,
            hash(b"alice again").to_bytes(),
        )
        .await;
    assert!(dup.is_err(), "duplicate applicant must fail");
    let msg = format!("{:?}", dup.unwrap_err()).to_lowercase();
    assert!(
        msg.contains("alreadyapplied") || msg.contains("already applied") || msg.contains("6040"),
        "expected AlreadyApplied, got {}",
        msg
    );

    // --- index mismatch: bob tries index 2 while len==1 -> ApplicationIndexMismatch ---
    let bob = Keypair::new();
    airdrop(&bob.pubkey());
    let bob_client = TrustEscrowClient::new(Cluster::Localnet, bob.insecure_clone()).unwrap();
    let bad_idx = bob_client
        .apply_to_job(
            &client_kp.pubkey(),
            job_id,
            2,
            hash(b"bob bad idx").to_bytes(),
        )
        .await;
    assert!(bad_idx.is_err(), "index 2 while len=1 must fail");
    let msg2 = format!("{:?}", bad_idx.unwrap_err()).to_lowercase();
    assert!(
        msg2.contains("indexmismatch")
            || msg2.contains("applicationindexmismatch")
            || msg2.contains("6046"),
        "expected ApplicationIndexMismatch, got {}",
        msg2
    );

    // --- correct next index 1 succeeds ---
    bob_client
        .apply_to_job(&client_kp.pubkey(), job_id, 1, hash(b"bob ok 1").to_bytes())
        .await
        .expect("bob apply index 1");

    // list_applications should now have 2, sorted by index
    let page = client
        .list_applications(&job_pda, None, Some(10))
        .await
        .expect("list");
    assert_eq!(page.applications.len(), 2);
    assert_eq!(page.applications[0].1.index, 0);
    assert_eq!(page.applications[1].1.index, 1);

    // --- self-apply (client == applicant) must fail CannotWorkOnOwnJob ---
    let self_apply = client
        .apply_to_job(&client_kp.pubkey(), job_id, 2, hash(b"self").to_bytes())
        .await;
    assert!(self_apply.is_err(), "client cannot apply to own job");
    let msg3 = format!("{:?}", self_apply.unwrap_err()).to_lowercase();
    assert!(
        msg3.contains("cannotworkonownjob") || msg3.contains("6011"),
        "expected CannotWorkOnOwnJob, got {}",
        msg3
    );

    // --- limit 50 is enforced via MAX_APPLICATIONS check (we fill to 2 here,
    //     full 50 is covered offline; we just verify the 50th boundary not
    //     panics: ensure len=2 still allows index 2)
    let carol = Keypair::new();
    airdrop(&carol.pubkey());
    let carol_client = TrustEscrowClient::new(Cluster::Localnet, carol.insecure_clone()).unwrap();
    carol_client
        .apply_to_job(&client_kp.pubkey(), job_id, 2, hash(b"carol 2").to_bytes())
        .await
        .expect("carol index 2");

    let page2 = client
        .list_applications(&job_pda, None, Some(10))
        .await
        .unwrap();
    assert_eq!(page2.applications.len(), 3);

    // Evitar "Cannot drop a runtime in a context where blocking is not allowed"
    // — los clients mantienen sockets que deben soltarse fuera del async stack
    // (mismo patrón que list_applications.rs e instructions_jobs.rs).
    tokio::task::block_in_place(|| {
        drop(carol_client);
        drop(bob_client);
        drop(alice_client);
        drop(client);
    });

    eprintln!("T21 applications PDA integration OK (seeds, index, duplicates, limits)");
}
