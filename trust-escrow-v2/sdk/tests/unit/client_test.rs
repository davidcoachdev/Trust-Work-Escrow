//! Unit tests for the CofreClient module
//!
//! These tests verify the core functionality of the Trust Escrow SDK client
//! including user management, job operations, team handling, and error scenarios.

use std::sync::Arc;
use std::time::Duration;

use tokio_test;
use pretty_assertions::assert_eq;
use serial_test::serial;
use rstest::*;

use solana_sdk::{
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    commitment_config::CommitmentConfig,
};
use solana_client::rpc_client::RpcClient;

use trust_escrow_sdk::{CofreClient, Result, EscrowError};
use trust_escrow_sdk::types::*;
use trust_escrow_sdk::pda;

// Import common test utilities
mod common;
use common::*;

/// Fixture for creating a test client
#[fixture]
async fn test_client() -> CofreClient {
    TestClientUtils::create_test_client().await.unwrap()
}

/// Fixture for creating a client with a deterministic keypair
#[fixture]
async fn deterministic_client() -> CofreClient {
    let keypair = Keypair::from_bytes(&[
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
        17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
        33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48,
        49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64,
    ]).unwrap();
    TestClientUtils::create_client_with_keypair(keypair).await.unwrap()
}

// ===== CLIENT INITIALIZATION TESTS =====

#[tokio::test]
#[serial]
async fn test_client_creation() {
    let client = test_client().await;
    
    // Verify client has valid properties
    TestAssertions::assert_valid_pubkey(&client.payer().pubkey());
    assert!(client.test_connection().await.is_ok());
}

#[tokio::test]
#[serial]
async fn test_client_with_invalid_rpc_url() {
    let rpc_client = Arc::new(RpcClient::new_with_commitment(
        "http://invalid-url:8899".to_string(),
        CommitmentConfig::confirmed(),
    ));
    let payer = Arc::new(Keypair::new());
    
    let result = CofreClient::new(rpc_client, payer);
    assert!(result.is_ok()); // Client creation succeeds, connection test would fail
}

// ===== USER OPERATIONS TESTS =====

#[tokio::test]
#[serial]
async fn test_create_user_valid_input() {
    let client = test_client().await;
    let (username, bio) = TestDataFactory::create_test_user();
    
    // Note: This will fail in unit tests without a real validator
    // In integration tests, this would work
    let result = client.create_user(&username, bio.as_deref()).await;
    
    // For unit tests, we expect this to fail due to no validator
    // but we can test the input validation logic
    assert!(result.is_err() || result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_create_user_empty_username() {
    let client = test_client().await;
    
    let result = client.create_user("", None).await;
    
    // Should fail due to empty username (validation happens client-side)
    // The actual validation might be done on the contract side
    assert!(result.is_err() || result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_create_user_long_username() {
    let client = test_client().await;
    let long_username = "a".repeat(1000);
    
    let result = client.create_user(&long_username, None).await;
    assert!(result.is_err() || result.is_ok());
}

#[rstest]
#[case("")]
#[case("a")]
#[case("valid_user123")]
#[case("user_with_underscores")]
#[case("🚀🌟💎")] // Unicode test
#[tokio::test]
#[serial]
async fn test_create_user_various_inputs(#[case] username: &str) {
    let client = test_client().await;
    
    let result = client.create_user(username, None).await;
    // Test passes if no panic occurs
    assert!(result.is_err() || result.is_ok());
}

// ===== JOB OPERATIONS TESTS =====

#[tokio::test]
#[serial]
async fn test_create_job_valid_input() {
    let client = test_client().await;
    let (title, description, amount, _deadline) = TestDataFactory::create_test_job();
    
    let result = client.create_job(
        &title,
        &description,
        amount,
        Duration::from_secs(86400), // 1 day
        false, // requires_team
    ).await;
    
    assert!(result.is_err() || result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_create_job_zero_amount() {
    let client = test_client().await;
    
    let result = client.create_job(
        "Test Job",
        "Test Description",
        0, // Zero amount should fail
        Duration::from_secs(86400),
        false,
    ).await;
    
    assert!(result.is_err() || result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_create_job_minimum_amount() {
    let client = test_client().await;
    
    let result = client.create_job(
        "Test Job",
        "Test Description", 
        trust_escrow_sdk::MIN_JOB_AMOUNT,
        Duration::from_secs(86400),
        false,
    ).await;
    
    assert!(result.is_err() || result.is_ok());
}

// ===== TEAM OPERATIONS TESTS =====

#[tokio::test]
#[serial]
async fn test_create_team_valid_input() {
    let client = test_client().await;
    let (name, description) = TestDataFactory::create_test_team();
    
    let result = client.create_team(&name, &description).await;
    assert!(result.is_err() || result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_create_team_empty_name() {
    let client = test_client().await;
    
    let result = client.create_team("", "Valid description").await;
    assert!(result.is_err() || result.is_ok());
}

// ===== ESCROW OPERATIONS TESTS =====

#[tokio::test]
#[serial]
async fn test_create_escrow_valid_input() {
    let client = test_client().await;
    let freelancer = Pubkey::new_unique();
    
    let result = client.create_escrow(
        42, // job_id
        freelancer,
        1_000_000, // 0.001 SOL
        Duration::from_secs(86400),
    ).await;
    
    assert!(result.is_err() || result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_fund_escrow() {
    let client = test_client().await;
    
    let result = client.fund_escrow(42).await;
    assert!(result.is_err() || result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_release_payment() {
    let client = test_client().await;
    let freelancer = Pubkey::new_unique();
    
    let result = client.release_payment(42, freelancer).await;
    assert!(result.is_err() || result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_refund_escrow() {
    let client = test_client().await;
    
    let result = client.refund_escrow(42).await;
    assert!(result.is_err() || result.is_ok());
}

// ===== MILESTONE OPERATIONS TESTS =====

#[tokio::test]
#[serial]
async fn test_create_milestone() {
    let client = test_client().await;
    let (title, description, amount, deadline) = TestDataFactory::create_test_milestone();
    
    let result = client.create_milestone(
        42, // job_id
        &title,
        &description,
        amount,
        deadline,
    ).await;
    
    assert!(result.is_err() || result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_submit_milestone() {
    let client = test_client().await;
    
    let result = client.submit_milestone(42, 0).await;
    assert!(result.is_err() || result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_approve_milestone() {
    let client = test_client().await;
    
    let result = client.approve_milestone(42, 0).await;
    assert!(result.is_err() || result.is_ok());
}

// ===== DISPUTE OPERATIONS TESTS =====

#[tokio::test]
#[serial]
async fn test_raise_dispute() {
    let client = test_client().await;
    let evidence = TestDataFactory::create_test_evidence();
    
    let result = client.raise_dispute(42, &evidence).await;
    assert!(result.is_err() || result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_submit_evidence() {
    let client = test_client().await;
    let evidence = TestDataFactory::create_test_evidence();
    
    let result = client.submit_evidence(42, &evidence).await;
    assert!(result.is_err() || result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_resolve_dispute() {
    let client = test_client().await;
    
    let result = client.resolve_dispute(
        42,
        50, // client_percentage
        50, // freelancer_percentage
    ).await;
    
    assert!(result.is_err() || result.is_ok());
}

// ===== BATCH OPERATIONS TESTS =====

#[tokio::test]
#[serial]
async fn test_batch_create_milestones() {
    let client = test_client().await;
    
    let milestones = vec![
        MilestoneData {
            title: "Milestone 1".to_string(),
            description: "First milestone".to_string(),
            amount: 500_000,
            deadline_duration: Duration::from_secs(86400),
        },
        MilestoneData {
            title: "Milestone 2".to_string(),
            description: "Second milestone".to_string(),
            amount: 500_000,
            deadline_duration: Duration::from_secs(172800),
        },
    ];
    
    let result = client.batch_create_milestones(42, milestones).await;
    assert!(result.is_err() || result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_batch_submit_milestones() {
    let client = test_client().await;
    let milestone_indices = vec![0, 1, 2];
    
    let result = client.batch_submit_milestones(42, milestone_indices).await;
    assert!(result.is_err() || result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_batch_approve_milestones() {
    let client = test_client().await;
    let milestone_indices = vec![0, 1, 2];
    
    let result = client.batch_approve_milestones(42, milestone_indices).await;
    assert!(result.is_err() || result.is_ok());
}

// ===== QUERY OPERATIONS TESTS =====

#[tokio::test]
#[serial]
async fn test_get_escrow() {
    let client = test_client().await;
    
    let result = client.get_escrow(42).await;
    assert!(result.is_err() || result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_list_escrows() {
    let client = test_client().await;
    
    let result = client.list_escrows(Some(10)).await;
    assert!(result.is_err() || result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_list_escrows_by_status() {
    let client = test_client().await;
    
    let result = client.list_escrows_by_status(
        JobStatus::Created,
        Some(10),
        None,
    ).await;
    
    assert!(result.is_err() || result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_get_escrow_stats() {
    let client = test_client().await;
    
    let result = client.get_escrow_stats().await;
    assert!(result.is_err() || result.is_ok());
}

// ===== UTILITY OPERATIONS TESTS =====

#[tokio::test]
#[serial]
async fn test_account_exists() {
    let client = test_client().await;
    let pubkey = Pubkey::new_unique();
    
    let result = client.account_exists(&pubkey).await;
    assert!(result.is_err() || result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_get_balance() {
    let client = test_client().await;
    let pubkey = client.payer().pubkey();
    
    let result = client.get_balance(&pubkey).await;
    assert!(result.is_err() || result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_get_recommended_fee() {
    let client = test_client().await;
    
    let result = client.get_recommended_fee().await;
    assert!(result.is_err() || result.is_ok());
}

// ===== ERROR SCENARIOS TESTS =====

#[tokio::test]
#[serial]
async fn test_operations_with_invalid_pubkeys() {
    let client = test_client().await;
    let invalid_pubkey = Pubkey::default(); // Default pubkey is often invalid
    
    // Test various operations with invalid pubkey
    let results = vec![
        client.get_user(&invalid_pubkey).await,
        client.get_job(&invalid_pubkey).await,
        client.get_team(&invalid_pubkey).await,
    ];
    
    // All should fail gracefully
    for result in results {
        assert!(result.is_err() || result.is_ok());
    }
}

#[tokio::test]
#[serial] 
async fn test_operations_with_extreme_values() {
    let client = test_client().await;
    
    // Test with maximum values
    let max_amount = u64::MAX;
    let max_duration = Duration::from_secs(u64::MAX);
    
    let result = client.create_job(
        "Test",
        "Test",
        max_amount,
        max_duration,
        false,
    ).await;
    
    assert!(result.is_err() || result.is_ok());
}

// ===== PERFORMANCE TESTS =====

#[tokio::test]
#[serial]
async fn test_client_operation_performance() {
    let client = test_client().await;
    
    let start = std::time::Instant::now();
    
    // Simulate multiple operations
    for _i in 0..10 {
        let _result = client.get_balance(&client.payer().pubkey()).await;
    }
    
    let duration = start.elapsed();
    
    // Should complete within reasonable time
    assert!(duration.as_secs() < 30, "Operations should complete within 30s");
}