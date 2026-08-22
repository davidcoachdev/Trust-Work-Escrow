//! T7 — Listings read-through + cursor + timeouts.
//!
//! Validates FR-6 (list_jobs_by_client/status) and Security B3 (bounded loops,
//! typed timeout). Unit part runs without a validator (cursor/pagination/timeouts);
//! integration part runs against a local `solana-test-validator` with
//! `trust_escrow_v3` deployed (program id 7a2Y).

#![cfg(feature = "solana")]

use std::collections::HashSet;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anchor_client::Cluster;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Keypair, signer::Signer,
};
use trust_escrow_sdk::{
    client::TrustEscrowClient,
    types::JobStatus,
    utils::{
        decode_cursor, encode_cursor, validate_limit, with_timeout, DEFAULT_PAGE_LIMIT,
        MAX_PAGE_LIMIT,
    },
    PROGRAM_ID_STR,
};

// ---- Unit / offline ----

#[test]
fn cursor_is_opaque_and_roundtrips() {
    for off in [0usize, 1, 19, 20, 21, 99, 10_000] {
        let c = encode_cursor(off);
        // opaque: not a plain decimal
        assert_ne!(c, off.to_string());
        assert_eq!(decode_cursor(Some(&c)).unwrap(), off);
    }
    assert_eq!(decode_cursor(None).unwrap(), 0);
    assert_eq!(decode_cursor(Some("")).unwrap(), 0);
    assert!(decode_cursor(Some("not-base64!!!")).is_err());
}

#[test]
fn limit_is_clamped_and_validated() {
    assert_eq!(validate_limit(None).unwrap(), DEFAULT_PAGE_LIMIT);
    assert_eq!(validate_limit(Some(10)).unwrap(), 10);
    assert_eq!(validate_limit(Some(10_000)).unwrap(), MAX_PAGE_LIMIT);
    assert!(validate_limit(Some(0)).is_err());
}

#[tokio::test]
async fn pagination_has_no_duplicates_and_cursor_advances() {
    // Simulate the Page logic used by the client (no RPC).
    use trust_escrow_sdk::utils::Page;
    let all: Vec<u32> = (0..25).collect();
    let p1 = Page::from_slice(all.clone(), 0, 10);
    let off1 = decode_cursor(p1.next_cursor.as_deref()).unwrap();
    assert_eq!(off1, 10);
    assert!(p1.has_more);
    let p2 = Page::from_slice(all.clone(), off1, 10);
    let off2 = decode_cursor(p2.next_cursor.as_deref()).unwrap();
    assert_eq!(off2, 20);
    assert!(p2.has_more);
    let p3 = Page::from_slice(all.clone(), off2, 10);
    assert!(!p3.has_more);
    assert!(p3.next_cursor.is_none());
    // no duplicates across pages
    let mut seen = HashSet::new();
    for v in p1
        .items
        .iter()
        .chain(p2.items.iter())
        .chain(p3.items.iter())
    {
        assert!(seen.insert(*v), "duplicate across pages: {}", v);
    }
    assert_eq!(seen.len(), 25);
}

#[tokio::test]
async fn timeout_yields_typed_error() {
    let err = with_timeout(Duration::from_millis(10), async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok::<(), _>(())
    })
    .await
    .unwrap_err();
    assert!(err.is_timeout(), "expected Timeout, got {:?}", err);
    assert!(format!("{}", err).to_lowercase().contains("timed out"));
}

#[tokio::test]
async fn no_infinite_loop_on_empty_and_oob_cursor() {
    use trust_escrow_sdk::utils::Page;
    let all: Vec<u32> = vec![];
    let p = Page::from_slice(all, 999, 10);
    assert!(p.items.is_empty());
    assert!(!p.has_more);
    assert!(p.next_cursor.is_none());

    let all2: Vec<u32> = (0..5).collect();
    let p2 = Page::from_slice(all2, 10, 10);
    assert!(p2.items.is_empty());
    assert!(!p2.has_more);
}

// ---- Integration (requires local validator) ----

const RPC_URL: &str = "http://127.0.0.1:8899";

fn rpc() -> RpcClient {
    RpcClient::new_with_commitment(RPC_URL.to_string(), CommitmentConfig::confirmed())
}

fn assert_program_available() {
    let pid: Pubkey = PROGRAM_ID_STR.parse().unwrap();
    let acc = rpc().get_account(&pid).expect("program account exists");
    assert!(acc.executable, "program must be executable");
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn airdrop(pubkey: &Pubkey) {
    let before = rpc().get_balance(pubkey).unwrap_or(0);
    let _ = rpc().request_airdrop(pubkey, 2_000_000_000).unwrap();
    for _ in 0..80 {
        let after = rpc().get_balance(pubkey).unwrap_or(0);
        if after > before {
            return;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    panic!("airdrop not confirmed");
}

#[test]
fn queries_read_through_cursor_and_status() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async { queries_read_through_cursor_and_status_inner().await });
}

async fn queries_read_through_cursor_and_status_inner() {
    assert_program_available();

    // Two distinct payers so `by_client` actually discriminates.
    let payer_a = Keypair::new();
    let payer_b = Keypair::new();
    airdrop(&payer_a.pubkey());
    airdrop(&payer_b.pubkey());
    // fund a bit more for rent-exempt job accounts (multiple)
    for kp in [&payer_a, &payer_b] {
        airdrop(&kp.pubkey());
    }

    let client_a = TrustEscrowClient::new(Cluster::Localnet, payer_a.insecure_clone()).unwrap();
    let client_b = TrustEscrowClient::new(Cluster::Localnet, payer_b.insecure_clone()).unwrap();

    // Create ~22 jobs for A so we can exercise cursor >20 across pages.
    // Use unique job_ids per run to avoid collisions on a reused validator.
    let base = (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        % 900_000)
        + 5000;
    let n = 22usize;
    for i in 0..n {
        let job_id = base + i as u64;
        client_a
            .create_job(job_id, 200_000, now_ts() + 3600)
            .await
            .expect("create_job");
    }
    // A few jobs for B to ensure by_client does not leak.
    for i in 0..3 {
        let job_id = base + 1000 + i as u64;
        client_b
            .create_job(job_id, 200_000, now_ts() + 3600)
            .await
            .expect("create_job b");
    }
    // fund one of A's jobs so we have at least one Funded status for by_status check
    client_a.deposit_funds(base).await.expect("deposit_funds");

    // ---- cursor pagination: 10 + 10 + remainder, no duplicates ----
    let mut all_pks: Vec<Pubkey> = Vec::new();
    let mut cursor: Option<String> = None;
    let mut pages = 0usize;
    loop {
        let page = client_a
            .list_jobs_by_client(&payer_a.pubkey(), cursor.clone(), Some(10))
            .await
            .expect("list_jobs_by_client page");
        pages += 1;
        assert!(page.jobs.len() <= 10);
        // ensure alignment with offset semantics: after last page, has_more false and no cursor
        if page.has_more {
            assert!(page.next_cursor.is_some());
        } else {
            assert!(page.next_cursor.is_none());
        }
        for (pk, _) in &page.jobs {
            assert!(!all_pks.contains(pk), "duplicate job across pages: {}", pk);
            all_pks.push(*pk);
        }
        if !page.has_more {
            break;
        }
        cursor = page.next_cursor.clone();
        // bounded loop: should finish in ceil(n/10) pages, never infinite
        assert!(pages < 10, "pagination loop appears unbounded");
        // next call must advance
        assert!(cursor.is_some());
    }
    assert!(
        all_pks.len() >= 22,
        "expected at least {} jobs, got {}",
        n,
        all_pks.len()
    );
    assert!(
        pages >= 3,
        "expected at least 3 pages for {} jobs with limit 10",
        n
    );

    // ---- by_client discriminates: B's listing does not contain A's jobs ----
    let page_b = client_b
        .list_jobs_by_client(&payer_b.pubkey(), None, Some(20))
        .await
        .expect("list_jobs_by_client b");
    assert!(page_b.jobs.len() >= 3);
    for pk in &all_pks {
        assert!(
            !page_b.jobs.iter().any(|(p, _)| p == pk),
            "leak across clients"
        );
    }

    // ---- by_status: only Created vs Funded discrimination ----
    let created_page = client_a
        .list_jobs_by_status(vec![JobStatus::Created], None, Some(100))
        .await
        .expect("list_by_status Created");
    for (_, job) in &created_page.jobs {
        assert_eq!(job.status, JobStatus::Created);
    }
    let funded_page = client_a
        .list_jobs_by_status(vec![JobStatus::Funded], None, Some(100))
        .await
        .expect("list_by_status Funded");
    for (_, job) in &funded_page.jobs {
        assert_eq!(job.status, JobStatus::Funded);
    }
    // At least one funded (the deposited one) must appear in Funded.
    assert!(!funded_page.jobs.is_empty());

    // ---- timeout: a 1ms timeout on a real RPC should be a typed Timeout ----
    let short = Duration::from_millis(1);
    // We call the with_timeout variant directly with an unrealistically short deadline.
    // The underlying RPC (get_program_accounts) takes >>1ms even on localnet, so it
    // should hit the deadline and surface BackendError::Timeout.
    let timed = client_a
        .list_jobs_by_client_with_timeout(&payer_a.pubkey(), None, Some(10), short)
        .await;
    // The validator is extremely fast on loopback, so we cannot *guarantee* a 1ms
    // hit; treat either Timeout or success as non-failure, but if it errors it must
    // be a typed Timeout and not Sdk/Account. We assert the error kind when it fails.
    if let Err(e) = timed {
        assert!(
            e.is_timeout(),
            "short timeout must be typed Timeout, got {:?}",
            e
        );
    }

    // ---- invalid cursor returns InvalidParameter, not panic ----
    let bad = client_a
        .list_jobs_by_client(
            &payer_a.pubkey(),
            Some("!!!bad-cursor".to_string()),
            Some(10),
        )
        .await;
    assert!(bad.is_err());

    // Drop RPC-holding clients outside async (RpcClient drop panics inside async).
    tokio::task::block_in_place(|| {
        drop(client_a);
        drop(client_b);
    });

    println!("T7 queries OK (cursor paginates >20 without dup, by_client/by_status discriminate, timeout typed)");
}
