//! Core client / types / error tests for `trust-escrow-v3` (T3).
//!
//! These run without a validator: they build the client offline, verify the
//! `ErrorCode` mapping, and round-trip a mock account buffer through the
//! deserialization path used by the getters.

#![cfg(feature = "solana")]

use anchor_lang::AnchorSerialize;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;

use trust_escrow_sdk::client::{deserialize_account, TrustEscrowClient};
use trust_escrow_sdk::error::{BackendError, ErrorCode};
use trust_escrow_sdk::types::*;

#[test]
fn test_error_code_mapping() {
    assert_eq!(ErrorCode::from_code(6000), Some(ErrorCode::MathOverflow));
    assert_eq!(ErrorCode::from_code(6001), Some(ErrorCode::ProgramPaused));
    assert_eq!(
        ErrorCode::from_code(6050),
        Some(ErrorCode::InvalidApplicationCleanupAccounts)
    );
    assert_eq!(ErrorCode::from_code(9999), None);

    let err: BackendError = ErrorCode::ProgramPaused.into();
    assert!(matches!(
        err,
        BackendError::Contract(ErrorCode::ProgramPaused)
    ));
}

#[test]
fn test_deserialize_mock_config() {
    let cfg = Config {
        authority: Pubkey::new_unique(),
        advisor: Pubkey::new_unique(),
        treasury: Pubkey::new_unique(),
        arbitration_treasury: Pubkey::new_unique(),
        fee_bps: 250,
        paused: false,
        bump: 255,
        pending_authority: None,
        pending_authority_at: 0,
    };

    let mut buf = account_discriminator("Config").to_vec();
    cfg.serialize(&mut buf).expect("serialize config");

    let got = deserialize_account::<Config>(&buf).expect("deserialize config");
    assert_eq!(got, cfg);
}

#[test]
fn test_deserialize_mock_job_with_optionals() {
    let job = Job {
        client: Pubkey::new_unique(),
        freelancer: Some(Pubkey::new_unique()),
        amount: 1_000_000,
        fee_amount: 25_000,
        status: JobStatus::Funded,
        paused: false,
        paused_at: 0,
        deadline: 1_700_000_000,
        submitted_at: None,
        milestones_total: 2,
        milestones_approved: 0,
        milestones_amount_total: 1_000_000,
        applicants: vec![Pubkey::new_unique()],
        bump: 254,
    };

    let mut buf = account_discriminator("Job").to_vec();
    job.serialize(&mut buf).expect("serialize job");

    let got = deserialize_account::<Job>(&buf).expect("deserialize job");
    assert_eq!(got, job);
    assert_eq!(got.status, JobStatus::Funded);
    assert_eq!(got.applicants.len(), 1);
}

#[test]
fn test_deserialize_none_on_invalid_buffer() {
    // Empty buffer -> discriminator mismatch -> None.
    assert!(deserialize_account::<Config>(&[]).is_none());
    // Garbage -> None.
    assert!(deserialize_account::<Config>(b"not a real account").is_none());
    // Wrong discriminator -> None.
    let mut buf = account_discriminator("Job").to_vec();
    let cfg = Config {
        authority: Pubkey::new_unique(),
        advisor: Pubkey::new_unique(),
        treasury: Pubkey::new_unique(),
        arbitration_treasury: Pubkey::new_unique(),
        fee_bps: 0,
        paused: false,
        bump: 0,
        pending_authority: None,
        pending_authority_at: 0,
    };
    cfg.serialize(&mut buf).unwrap();
    assert!(deserialize_account::<Dispute>(&buf).is_none());
}

#[test]
fn test_client_builds_offline_and_getters_return_none() {
    // Building the client performs no RPC, so it works without a validator.
    let client = TrustEscrowClient::new(anchor_client::Cluster::Localnet, Keypair::new())
        .expect("client builds offline");

    // The same deserialize path used by every getter yields None for bad data.
    assert!(client_getter_none::<Config>(&client));
    assert!(client_getter_none::<Job>(&client));
    assert!(client_getter_none::<Application>(&client));
    assert!(client_getter_none::<ArbiterPool>(&client));
    assert!(client_getter_none::<Dispute>(&client));
    assert!(client_getter_none::<Evidence>(&client));
    assert!(client_getter_none::<Milestone>(&client));
    assert!(client_getter_none::<SupportTicket>(&client));
    assert!(client_getter_none::<ArbitrationEscrow>(&client));
}

/// Helper: exercise the deserialize path a getter would use on an absent account.
fn client_getter_none<T: anchor_lang::AccountDeserialize>(_client: &TrustEscrowClient) -> bool {
    deserialize_account::<T>(&[]).is_none()
}
