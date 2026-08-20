//! T8 — list_applications(job) read-through + cursor + timeouts.
//!
//! Valida FR-7 (applications derivadas por job via get_program_accounts)
//! y Security B3 (timeouts, cursor opaco, loops acotados).
//! Parte unit corre sin validator; integración requiere local validator 7a2Y.

#![cfg(feature = "solana")]

use std::collections::HashSet;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anchor_client::Cluster;
use anchor_lang::AnchorSerialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, hash::hash, pubkey::Pubkey, signature::Keypair,
    signer::Signer,
};
use trust_escrow_sdk::{
    client::{deserialize_account, TrustEscrowClient},
    pda,
    types::*,
    utils::{
        decode_cursor, encode_cursor, validate_limit, with_timeout, Page, DEFAULT_PAGE_LIMIT,
        MAX_PAGE_LIMIT,
    },
    PROGRAM_ID_STR,
};

// ---- Unit / offline ----

#[test]
fn cursor_is_opaque_and_roundtrips_applications() {
    for off in [0usize, 1, 19, 20, 21, 99, 10_000] {
        let c = encode_cursor(off);
        assert_ne!(c, off.to_string(), "cursor must be opaque");
        assert_eq!(decode_cursor(Some(&c)).unwrap(), off);
    }
    assert_eq!(decode_cursor(None).unwrap(), 0);
    assert_eq!(decode_cursor(Some("")).unwrap(), 0);
    assert!(decode_cursor(Some("not-base64!!!")).is_err());
}

#[test]
fn page_sorts_by_index_and_cursor_advances() {
    // Simulate Page logic sorted by index.
    let job = Pubkey::new_unique();
    let mk = |idx: u8| Application {
        job,
        index: idx,
        applicant: Pubkey::new_unique(),
        proposal_hash: [0u8; 32],
        status: ApplicationStatus::Pending,
        bump: 255,
    };
    let apps: Vec<(Pubkey, Application)> = vec![
        (Pubkey::new_unique(), mk(2)),
        (Pubkey::new_unique(), mk(0)),
        (Pubkey::new_unique(), mk(1)),
    ];
    let mut sorted = apps.clone();
    sorted.sort_by(|a, b| a.1.index.cmp(&b.1.index).then_with(|| a.0.cmp(&b.0)));
    assert_eq!(sorted[0].1.index, 0);
    assert_eq!(sorted[1].1.index, 1);
    assert_eq!(sorted[2].1.index, 2);

    // Page pagination with opaque cursor
    let all: Vec<u32> = (0..25).collect();
    let p1 = Page::from_slice(all.clone(), 0, 10);
    let off1 = decode_cursor(p1.next_cursor.as_deref()).unwrap();
    assert_eq!(off1, 10);
    assert!(p1.has_more);
    let p2 = Page::from_slice(all.clone(), off1, 10);
    assert_eq!(decode_cursor(p2.next_cursor.as_deref()).unwrap(), 20);
    assert!(p2.has_more);
    let p3 = Page::from_slice(all.clone(), 20, 10);
    assert!(!p3.has_more);
    assert!(p3.next_cursor.is_none());
    let mut seen = HashSet::new();
    for v in p1
        .items
        .iter()
        .chain(p2.items.iter())
        .chain(p3.items.iter())
    {
        assert!(seen.insert(*v));
    }
    assert_eq!(seen.len(), 25);
}

#[test]
fn deserialize_application_roundtrip_and_filter() {
    let job_a = Pubkey::new_unique();
    let job_b = Pubkey::new_unique();
    let mk = |job: Pubkey, idx: u8| {
        let app = Application {
            job,
            index: idx,
            applicant: Pubkey::new_unique(),
            proposal_hash: hash(format!("proposal-{}", idx).as_bytes()).to_bytes(),
            status: ApplicationStatus::Pending,
            bump: 254,
        };
        let mut buf = account_discriminator("Application").to_vec();
        app.serialize(&mut buf).unwrap();
        let got = deserialize_account::<Application>(&buf).unwrap();
        assert_eq!(got.job, job);
        assert_eq!(got.index, idx);
        got
    };
    let a0 = mk(job_a, 0);
    let a1 = mk(job_a, 1);
    let b0 = mk(job_b, 0);
    let all = vec![a0.clone(), a1.clone(), b0.clone()];
    let filtered: Vec<_> = all.into_iter().filter(|app| app.job == job_a).collect();
    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().all(|a| a.job == job_a));
}

#[test]
fn limit_validation_applications() {
    assert_eq!(validate_limit(None).unwrap(), DEFAULT_PAGE_LIMIT);
    assert_eq!(validate_limit(Some(10)).unwrap(), 10);
    assert_eq!(validate_limit(Some(10_000)).unwrap(), MAX_PAGE_LIMIT);
    assert!(validate_limit(Some(0)).is_err());
}

#[tokio::test]
async fn timeout_yields_typed_error_applications() {
    let err = with_timeout(Duration::from_millis(10), async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok::<(), _>(())
    })
    .await
    .unwrap_err();
    assert!(err.is_timeout());
}

// ---- Integration (requires local validator with 7a2Y program) ----

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

fn proposal_hash(s: &str) -> [u8; 32] {
    hash(s.as_bytes()).to_bytes()
}

#[test]
fn list_applications_per_job_with_cursor_and_filter() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async { list_applications_per_job_with_cursor_and_filter_inner().await });
}

async fn list_applications_per_job_with_cursor_and_filter_inner() {
    assert_program_available();

    let payer = Keypair::new();
    airdrop(&payer.pubkey());
    airdrop(&payer.pubkey());
    let client = TrustEscrowClient::new(Cluster::Localnet, payer.insecure_clone()).unwrap();

    // Two jobs for same client to test filtering by job
    let base = (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        % 900_000)
        + 8000;
    let job_id_a = base;
    let job_id_b = base + 1;

    // Ensure config exists (authority wallet may have initialized)
    // Create jobs (amount >= MIN_JOB_AMOUNT)
    client
        .create_job(job_id_a, 200_000, now_ts() + 3600)
        .await
        .expect("create_job a");
    client
        .create_job(job_id_b, 200_000, now_ts() + 3600)
        .await
        .expect("create_job b");
    client.deposit_funds(job_id_a).await.expect("deposit a");
    client.deposit_funds(job_id_b).await.expect("deposit b");

    let job_a = pda::get_job_pda(&payer.pubkey(), job_id_a).unwrap().0;
    let job_b = pda::get_job_pda(&payer.pubkey(), job_id_b).unwrap().0;

    // Create 3 freelancers for job A and 2 for job B
    let mut freelancers_a = Vec::new();
    for i in 0..3 {
        let kp = Keypair::new();
        airdrop(&kp.pubkey());
        let fk_client = TrustEscrowClient::new(Cluster::Localnet, kp.insecure_clone()).unwrap();
        fk_client
            .apply_to_job(
                &payer.pubkey(),
                job_id_a,
                i,
                proposal_hash(&format!("proposal-a-{}", i)),
            )
            .await
            .expect("apply a");
        freelancers_a.push(kp);
        // Drop RpcClient holder inside blocking
        tokio::task::block_in_place(|| drop(fk_client));
    }
    for i in 0..2 {
        let kp = Keypair::new();
        airdrop(&kp.pubkey());
        let fk_client = TrustEscrowClient::new(Cluster::Localnet, kp.insecure_clone()).unwrap();
        fk_client
            .apply_to_job(
                &payer.pubkey(),
                job_id_b,
                i,
                proposal_hash(&format!("proposal-b-{}", i)),
            )
            .await
            .expect("apply b");
        tokio::task::block_in_place(|| drop(fk_client));
    }

    // ---- basic listing: job A has 3 applications sorted by index ----
    let page = client
        .list_applications(&job_a, None, Some(10))
        .await
        .expect("list_applications a");
    assert_eq!(page.applications.len(), 3, "job A should have 3 apps");
    // Verify sorted by index
    for (i, (_, app)) in page.applications.iter().enumerate() {
        assert_eq!(app.index as usize, i, "index mismatch at {}", i);
        assert_eq!(app.job, job_a);
        assert!(app.status == ApplicationStatus::Pending);
    }
    assert!(!page.has_more);
    assert!(page.next_cursor.is_none());

    // ---- filtering: job B has 2 ----
    let page_b = client
        .list_applications(&job_b, None, Some(10))
        .await
        .expect("list_applications b");
    assert_eq!(page_b.applications.len(), 2);
    for (_, app) in &page_b.applications {
        assert_eq!(app.job, job_b);
    }
    // no leak across jobs
    let pks_a: HashSet<_> = page.applications.iter().map(|(pk, _)| *pk).collect();
    for (pk, _) in &page_b.applications {
        assert!(!pks_a.contains(pk), "leak across jobs: {}", pk);
    }

    // ---- cursor pagination: limit 2 on job A (3 items -> 2+1) ----
    let p1 = client
        .list_applications(&job_a, None, Some(2))
        .await
        .expect("page1");
    assert_eq!(p1.applications.len(), 2);
    assert!(p1.has_more);
    assert!(p1.next_cursor.is_some());
    let c1 = p1.next_cursor.clone().unwrap();
    // opaque: not plain "2"
    assert_ne!(c1, "2");
    let p2 = client
        .list_applications(&job_a, Some(c1.clone()), Some(2))
        .await
        .expect("page2");
    assert_eq!(p2.applications.len(), 1);
    assert!(!p2.has_more);
    assert!(p2.next_cursor.is_none());
    // no duplicates, no overlap
    let mut seen = HashSet::new();
    for (pk, _) in p1.applications.iter().chain(p2.applications.iter()) {
        assert!(seen.insert(*pk), "duplicate across pages");
    }
    assert_eq!(seen.len(), 3);
    // verify sorted order preserved across pages
    let combined: Vec<u8> = p1
        .applications
        .iter()
        .chain(p2.applications.iter())
        .map(|(_, a)| a.index)
        .collect();
    assert_eq!(combined, vec![0, 1, 2]);

    // ---- invalid cursor returns InvalidParameter ----
    let bad = client
        .list_applications(&job_a, Some("!!!bad-cursor".to_string()), Some(2))
        .await;
    assert!(bad.is_err());
    let msg = format!("{:?}", bad.as_ref().unwrap_err()).to_lowercase();
    assert!(msg.contains("invalid") || msg.contains("cursor"));

    // ---- gap handling: cleanup index 1 closes one app, then listing has 2 ----
    // Accept first application to transition job to InProgress so cleanup is allowed from any index
    // Use first freelancer from job A
    let fl_a0 = freelancers_a[0].pubkey();
    client
        .accept_application(&payer.pubkey(), job_id_a, 0, &fl_a0)
        .await
        .expect("accept");
    // Cleanup from index 1 should close applications 1 and 2 (still Pending)
    client
        .cleanup_applications(job_id_a, 1)
        .await
        .expect("cleanup");
    let after = client
        .list_applications(&job_a, None, Some(10))
        .await
        .expect("list after cleanup");
    // After cleanup, only index 0 (Accepted) remains; gap handling must not panic
    assert_eq!(after.applications.len(), 1);
    assert_eq!(after.applications[0].1.index, 0);
    assert_eq!(after.applications[0].1.status, ApplicationStatus::Accepted);

    // ---- timeout typed: short timeout should be BackendError::Timeout when it fails ----
    let short = Duration::from_millis(1);
    let timed = client
        .list_applications_with_timeout(&job_a, None, Some(10), short)
        .await;
    if let Err(e) = timed {
        assert!(
            e.is_timeout(),
            "short timeout must be typed Timeout, got {:?}",
            e
        );
    }

    tokio::task::block_in_place(|| drop(client));
    println!("T8 list_applications OK (filter by job, sort by index, cursor opaque, gap handling, timeout typed)");
}
