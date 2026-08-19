//! Integration happy-path tests for the T4 wrapper group (config / jobs /
//! applications / work lifecycle) against a local `solana-test-validator`
//! with the `trust_escrow_v3` program deployed.
//!
//! Run with: `cargo test -p trust-escrow-sdk --features solana --test instructions_jobs`

#![cfg(feature = "solana")]

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anchor_client::Cluster;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    hash::hash,
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signer},
};
use trust_escrow_sdk::{client::TrustEscrowClient, pda, types::*, PROGRAM_ID_STR};

const RPC_URL: &str = "http://127.0.0.1:8899";
const KEYPAIR_PATH: &str = "~/.config/solana/id.json";
const AMOUNT: u64 = 500_000; // >= MIN_JOB_AMOUNT (100_000)
const FEE_BPS: u16 = 100; // 1%

fn rpc() -> RpcClient {
    RpcClient::new_with_commitment(RPC_URL.to_string(), CommitmentConfig::confirmed())
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// Mirrors the contract's `compute_fee(amount, fee_bps) = amount * fee / BASIS_POINTS`.
fn compute_fee(amount: u64, fee_bps: u16) -> u64 {
    amount * fee_bps as u64 / 10_000
}

/// Convert a proposal string into the `[u8; 32]` hash `apply_to_job` expects.
fn proposal_hash(s: &str) -> [u8; 32] {
    hash(s.as_bytes()).to_bytes()
}

fn airdrop(pubkey: &Pubkey, lamports: u64) {
    let before = rpc().get_balance(pubkey).unwrap_or(0);
    let _ = rpc()
        .request_airdrop(pubkey, lamports)
        .expect("airdrop request");
    for _ in 0..100 {
        let after = rpc().get_balance(pubkey).unwrap_or(0);
        if after > before {
            return;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    panic!("airdrop not confirmed for {pubkey}");
}

fn expand_path(p: &str) -> String {
    if let Some(rest) = p.strip_prefix("~/") {
        let home = std::env::var("HOME").expect("HOME set");
        format!("{}/{}", home, rest)
    } else {
        p.to_string()
    }
}

/// The local wallet (`~/.config/solana/id.json`) — its pubkey matches the
/// contract's `INITIAL_AUTHORITY`, so it is the only signer allowed to
/// initialize the protocol config.
fn authority_keypair() -> Keypair {
    read_keypair_file(expand_path(KEYPAIR_PATH)).expect("read local wallet")
}

fn authority_client() -> TrustEscrowClient {
    TrustEscrowClient::new(Cluster::Localnet, authority_keypair()).expect("build authority client")
}

/// Assert the program is deployed and executable on the local validator.
fn assert_program_available() {
    let pid: Pubkey = PROGRAM_ID_STR.parse().unwrap();
    let acc = rpc().get_account(&pid).expect("program account exists");
    assert!(acc.executable, "program must be executable");
}

async fn initialize_config(client: &TrustEscrowClient) -> (Pubkey, Pubkey) {
    if let Ok(Some(cfg)) = client.get_config() {
        // Config already initialized on a reused validator — reuse it.
        return (cfg.treasury, cfg.arbitration_treasury);
    }
    let advisor = Keypair::new();
    let treasury = Keypair::new();
    let arb_treasury = Keypair::new();
    airdrop(&treasury.pubkey(), 1_000_000_000);
    airdrop(&arb_treasury.pubkey(), 1_000_000_000);
    client
        .initialize_config(
            &advisor.pubkey(),
            &treasury.pubkey(),
            &arb_treasury.pubkey(),
            FEE_BPS,
        )
        .await
        .expect("initialize_config");
    let cfg = client.get_config().unwrap().expect("config present");
    assert_eq!(cfg.fee_bps, FEE_BPS);
    assert!(!cfg.paused);
    (treasury.pubkey(), arb_treasury.pubkey())
}

#[test]
fn group_config_jobs_applications_work_happy_paths() {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(async {
        group_config_jobs_applications_work_happy_paths_inner().await;
    });
}

async fn group_config_jobs_applications_work_happy_paths_inner() {
    assert_program_available();
    let client = authority_client();
    let authority = authority_keypair();
    let client_pk = authority.pubkey();
    // Unique job ids per run so a reused validator does not collide.
    let mut job_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        % 1_000_000
        + 1;

    // ===== Grupo config: initialize + pause/unpause =====
    let (treasury, arb_treasury) = initialize_config(&client).await;
    let cfg = client.get_config().unwrap().expect("config present");
    assert_eq!(cfg.authority, client_pk);
    assert_eq!(cfg.treasury, treasury);
    assert_eq!(cfg.arbitration_treasury, arb_treasury);

    client.pause().await.expect("pause succeeds");
    assert!(client.get_config().unwrap().unwrap().paused);
    client.unpause().await.expect("unpause succeeds");
    assert!(!client.get_config().unwrap().unwrap().paused);

    // ===== Grupo jobs: create + deposit + cancel =====
    let j = job_id;
    job_id += 1;
    client
        .create_job(j, AMOUNT, now_ts() + 3600)
        .await
        .expect("create_job");
    let job = client.get_job(&client_pk, j).unwrap().expect("job");
    assert_eq!(job.status, JobStatus::Created);
    assert_eq!(job.amount, AMOUNT);
    assert_eq!(job.client, client_pk);
    assert_eq!(job.fee_amount, compute_fee(AMOUNT, FEE_BPS));

    client.deposit_funds(j).await.expect("deposit_funds");
    let job = client.get_job(&client_pk, j).unwrap().unwrap();
    assert_eq!(job.status, JobStatus::Funded);

    // Cancelling a funded job returns the escrowed funds and closes the job.
    client.cancel_job(j).await.expect("cancel_job");
    let closed = client.get_job(&client_pk, j).unwrap();
    assert!(closed.is_none(), "cancelled job account is closed");

    // ===== Grupo applications: apply + accept + cleanup =====
    let j = job_id;
    job_id += 1;
    client
        .create_job(j, AMOUNT, now_ts() + 3600)
        .await
        .unwrap();
    client.deposit_funds(j).await.unwrap();

    let fl_a = Keypair::new();
    let fl_b = Keypair::new();
    airdrop(&fl_a.pubkey(), 2_000_000_000);
    airdrop(&fl_b.pubkey(), 2_000_000_000);
    let fl_a_pk = fl_a.pubkey();
    let fl_b_pk = fl_b.pubkey();
    let a_client = TrustEscrowClient::new(Cluster::Localnet, fl_a).unwrap();
    let b_client = TrustEscrowClient::new(Cluster::Localnet, fl_b).unwrap();

    a_client
        .apply_to_job(&client_pk, j, 0, proposal_hash("First"))
        .await
        .expect("apply_to_job (a)");
    b_client
        .apply_to_job(&client_pk, j, 1, proposal_hash("Second"))
        .await
        .expect("apply_to_job (b)");
    let job_pk = pda::get_job_pda(&client_pk, j).unwrap().0;
    let app_a = client
        .get_application(&job_pk, 0, &fl_a_pk)
        .unwrap()
        .expect("app a");
    assert_eq!(app_a.status, ApplicationStatus::Pending);
    let app_b = client
        .get_application(&job_pk, 1, &fl_b_pk)
        .unwrap()
        .expect("app b");
    assert_eq!(app_b.status, ApplicationStatus::Pending);

    client
        .accept_application(&client_pk, j, 0, &fl_a_pk)
        .await
        .expect("accept_application");
    let job = client.get_job(&client_pk, j).unwrap().unwrap();
    assert_eq!(job.status, JobStatus::InProgress);
    assert_eq!(job.freelancer, Some(fl_a_pk));
    assert_eq!(job.applicants_len, 2);
    let app_a = client
        .get_application(&job_pk, 0, &fl_a_pk)
        .unwrap()
        .unwrap();
    assert_eq!(app_a.status, ApplicationStatus::Accepted);

    // Cleanup from index 1 closes freelancer B's still-Pending application.
    client
        .cleanup_applications(j, 1)
        .await
        .expect("cleanup_applications");
    let app_b = client.get_application(&job_pk, 1, &fl_b_pk).unwrap();
    assert!(app_b.is_none(), "second application must be closed");

    // ===== Grupo work: submit + approve (pays) + reject (returns) =====
    let j = job_id;
    job_id += 1;
    let freelancer = Keypair::new();
    airdrop(&freelancer.pubkey(), 2_000_000_000);
    let fl_pk = freelancer.pubkey();
    let fl_client = TrustEscrowClient::new(Cluster::Localnet, freelancer).unwrap();

    client
        .create_job(j, AMOUNT, now_ts() + 3600)
        .await
        .unwrap();
    client.deposit_funds(j).await.unwrap();
    fl_client
        .apply_to_job(&client_pk, j, 0, proposal_hash("Happy to cover it"))
        .await
        .unwrap();
    client
        .accept_application(&client_pk, j, 0, &fl_pk)
        .await
        .unwrap();

    fl_client
        .submit_work(&client_pk, j)
        .await
        .expect("submit_work");
    let job = client.get_job(&client_pk, j).unwrap().unwrap();
    assert_eq!(job.status, JobStatus::Submitted);

    let fl_balance_before = rpc().get_balance(&fl_pk).unwrap();
    client.approve_work(j, &fl_pk).await.expect("approve_work");
    let job_after = client.get_job(&client_pk, j).unwrap();
    assert!(job_after.is_none(), "approved job account is closed");
    let fl_balance_after = rpc().get_balance(&fl_pk).unwrap();
    assert!(
        fl_balance_after > fl_balance_before,
        "freelancer must be paid on approve"
    );
    let treasury_balance = rpc().get_balance(&treasury).unwrap();
    assert!(treasury_balance > 0, "fee must land in treasury");

    let j = job_id;
    job_id += 1;
    client
        .create_job(j, AMOUNT, now_ts() + 3600)
        .await
        .unwrap();
    client.deposit_funds(j).await.unwrap();
    fl_client
        .apply_to_job(&client_pk, j, 0, proposal_hash("Accept me?"))
        .await
        .unwrap();
    client
        .accept_application(&client_pk, j, 0, &fl_pk)
        .await
        .unwrap();
    fl_client.submit_work(&client_pk, j).await.unwrap();
    client
        .reject_work(j)
        .await
        .expect("reject_work");
    let job = client.get_job(&client_pk, j).unwrap().unwrap();
    assert_eq!(job.status, JobStatus::InProgress);

    // ===== Grupo work: pause_job / unpause_job =====
    let j = job_id;
    client
        .create_job(j, AMOUNT, now_ts() + 3600)
        .await
        .unwrap();
    client.pause_job(j).await.expect("pause_job");
    let job = client.get_job(&client_pk, j).unwrap().unwrap();
    assert!(job.paused);
    client.unpause_job(j).await.expect("unpause_job");
    let job = client.get_job(&client_pk, j).unwrap().unwrap();
    assert!(!job.paused);

    println!("T4 happy-path groups OK (config, jobs, applications, work)");

    // Drop RpcClient-backed clients inside a blocking region: solana-client's
    // RpcClient owns a tokio Runtime whose drop panics inside an async context.
    tokio::task::block_in_place(|| {
        drop(client);
        drop(a_client);
        drop(b_client);
        drop(fl_client);
    });
}
