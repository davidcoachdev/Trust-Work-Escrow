//! PDA derivation and cache tests for `trust-escrow-v3` (T2).
//!
//! These run without any RPC: they compare our `derive_*_pda` output against
//! `Pubkey::find_program_address` using the same program id, and benchmark the
//! cached lookup path.

#![cfg(feature = "solana")]

use solana_sdk::pubkey::Pubkey;
use trust_escrow_sdk::pda::*;
use trust_escrow_sdk::PROGRAM_ID_STR;

#[test]
fn test_pda_vectors_match_find_program_address() {
    let pid: Pubkey = PROGRAM_ID_STR.parse().unwrap();
    let client = Pubkey::new_unique();
    let job = Pubkey::new_unique();
    let applicant = Pubkey::new_unique();
    let dispute = Pubkey::new_unique();

    // config: ["config"]
    assert_eq!(
        derive_config_pda().unwrap(),
        Pubkey::find_program_address(&[b"config"], &pid)
    );

    // job: ["job", client, job_id.le_bytes()]
    assert_eq!(
        derive_job_pda(&client, 7).unwrap(),
        Pubkey::find_program_address(&[b"job", client.as_ref(), &7u64.to_le_bytes()], &pid)
    );

    // application: ["application", job, index, applicant]
    assert_eq!(
        derive_application_pda(&job, 3, &applicant).unwrap(),
        Pubkey::find_program_address(
            &[b"application", job.as_ref(), &[3u8], applicant.as_ref()],
            &pid
        )
    );

    // arbiter_pool: ["arbiter_pool"]
    assert_eq!(
        derive_arbiter_pool_pda().unwrap(),
        Pubkey::find_program_address(&[b"arbiter_pool"], &pid)
    );

    // dispute: ["dispute", job]
    assert_eq!(
        derive_dispute_pda(&job).unwrap(),
        Pubkey::find_program_address(&[b"dispute", job.as_ref()], &pid)
    );

    // arb_fee: ["arb_fee", job]
    assert_eq!(
        derive_arb_fee_pda(&job).unwrap(),
        Pubkey::find_program_address(&[b"arb_fee", job.as_ref()], &pid)
    );

    // milestone: ["milestone", job, index]
    assert_eq!(
        derive_milestone_pda(&job, 2).unwrap(),
        Pubkey::find_program_address(&[b"milestone", job.as_ref(), &[2u8]], &pid)
    );

    // evidence: ["evidence", dispute, index]
    assert_eq!(
        derive_evidence_pda(&dispute, 1).unwrap(),
        Pubkey::find_program_address(&[b"evidence", dispute.as_ref(), &[1u8]], &pid)
    );

    // support: ["support", job]
    assert_eq!(
        derive_support_pda(&job).unwrap(),
        Pubkey::find_program_address(&[b"support", job.as_ref()], &pid)
    );
}

#[test]
fn test_pda_determinism() {
    let client = Pubkey::new_unique();
    let (a, ba) = derive_job_pda(&client, 42).unwrap();
    let (b, bb) = derive_job_pda(&client, 42).unwrap();
    assert_eq!(a, b);
    assert_eq!(ba, bb);
}

#[test]
fn test_pda_cache_returns_same_address() {
    clear_pda_cache();
    let client = Pubkey::new_unique();

    let (a, ba) = get_job_pda(&client, 1).unwrap();
    let (b, bb) = get_job_pda(&client, 1).unwrap();
    assert_eq!(a, b);
    assert_eq!(ba, bb);

    // A different id must not collide in the cache.
    let (c, _) = get_job_pda(&client, 2).unwrap();
    assert_ne!(a, c);
}

#[test]
fn test_pda_cache_hit_under_1ms() {
    clear_pda_cache();
    let client = Pubkey::new_unique();

    // Warm the cache.
    let _ = get_job_pda(&client, 1).unwrap();

    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = get_job_pda(&client, 1).unwrap();
    }
    let elapsed_ns = start.elapsed().as_nanos();
    let per_ms = elapsed_ns as f64 / 1000.0 / 1000.0;
    assert!(per_ms < 1.0, "cache hit avg {:.4} ms (>1ms)", per_ms);
}
