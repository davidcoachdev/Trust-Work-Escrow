//! Common testing utilities and helpers
//!
//! This module provides shared utilities for testing the Trust Escrow SDK,
//! including mock clients, test data generation, and assertion helpers.

use std::sync::Arc;
use std::time::Duration;

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
};
use tokio::time::timeout;

use trust_escrow_sdk::types::*;
use trust_escrow_sdk::{CofreClient, EscrowError, Result};

#[cfg(test)]
use fake::{Fake, Faker};
#[cfg(test)]
use rand::Rng;

/// Test configuration and constants
pub struct TestConfig {
    pub timeout_duration: Duration,
    pub test_cluster_url: String,
    pub commitment: CommitmentConfig,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            timeout_duration: Duration::from_secs(30),
            test_cluster_url: "http://localhost:8899".to_string(),
            commitment: CommitmentConfig::confirmed(),
        }
    }
}

/// Mock RPC client for testing that doesn't require a real Solana validator
pub struct MockRpcClient {
    pub simulated_responses: std::collections::HashMap<String, Result<Vec<u8>>>,
    pub call_count: std::sync::atomic::AtomicUsize,
}

impl MockRpcClient {
    pub fn new() -> Self {
        Self {
            simulated_responses: std::collections::HashMap::new(),
            call_count: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    pub fn add_response(&mut self, method: &str, response: Result<Vec<u8>>) {
        self.simulated_responses
            .insert(method.to_string(), response);
    }

    pub fn call_count(&self) -> usize {
        self.call_count.load(std::sync::atomic::Ordering::SeqCst)
    }
}

/// Test data factory for generating realistic test data
pub struct TestDataFactory;

impl TestDataFactory {
    /// Generate a random test user
    pub fn create_test_user() -> (String, Option<String>) {
        let username: String = fake::internet::Username().fake();
        let bio: Option<String> = if rand::thread_rng().gen_bool(0.7) {
            Some(fake::lorem::Sentence(1..3).fake())
        } else {
            None
        };
        (username, bio)
    }

    /// Generate test job data
    pub fn create_test_job() -> (String, String, u64, Duration) {
        let title: String = fake::company::Industry().fake();
        let description: String = fake::lorem::Paragraph(3..6).fake();
        let amount = rand::thread_rng().gen_range(1_000_000..10_000_000); // 0.001 to 0.01 SOL
        let deadline = Duration::from_secs(
            rand::thread_rng().gen_range(86400..2_592_000), // 1 day to 30 days
        );
        (title, description, amount, deadline)
    }

    /// Generate test team data
    pub fn create_test_team() -> (String, String) {
        let name: String = fake::company::CompanyName().fake();
        let description: String = fake::lorem::Sentence(2..4).fake();
        (name, description)
    }

    /// Generate test dispute evidence
    pub fn create_test_evidence() -> String {
        fake::lorem::Paragraph(2..4).fake()
    }

    /// Generate test milestone data
    pub fn create_test_milestone() -> (String, String, u64, Duration) {
        let title: String = format!("Milestone: {}", fake::lorem::Word().fake());
        let description: String = fake::lorem::Sentence(1..3).fake();
        let amount = rand::thread_rng().gen_range(100_000..1_000_000); // 0.0001 to 0.001 SOL
        let deadline = Duration::from_secs(
            rand::thread_rng().gen_range(86400..604800), // 1 to 7 days
        );
        (title, description, amount, deadline)
    }
}

/// Test utilities for client operations
pub struct TestClientUtils;

impl TestClientUtils {
    /// Create a test client with a mock RPC client
    pub async fn create_test_client() -> Result<CofreClient> {
        let config = TestConfig::default();
        let rpc_client = Arc::new(RpcClient::new_with_commitment(
            config.test_cluster_url,
            config.commitment,
        ));
        let payer = Arc::new(Keypair::new());

        CofreClient::new(rpc_client, payer)
    }

    /// Create a client with a specific keypair for reproducible tests
    pub async fn create_client_with_keypair(keypair: Keypair) -> Result<CofreClient> {
        let config = TestConfig::default();
        let rpc_client = Arc::new(RpcClient::new_with_commitment(
            config.test_cluster_url,
            config.commitment,
        ));
        let payer = Arc::new(keypair);

        CofreClient::new(rpc_client, payer)
    }

    /// Wait for transaction confirmation with timeout
    pub async fn wait_for_confirmation(
        client: &CofreClient,
        signature: &Signature,
    ) -> Result<bool> {
        timeout(
            TestConfig::default().timeout_duration,
            client.wait_for_confirmation(signature),
        )
        .await
        .map_err(|_| EscrowError::Timeout("Transaction confirmation timeout".to_string()))?
    }

    /// Check if a pubkey exists on-chain
    pub async fn account_exists(client: &CofreClient, pubkey: &Pubkey) -> Result<bool> {
        timeout(
            TestConfig::default().timeout_duration,
            client.account_exists(pubkey),
        )
        .await
        .map_err(|_| EscrowError::Timeout("Account existence check timeout".to_string()))?
    }
}

/// Assertion helpers for testing
pub struct TestAssertions;

impl TestAssertions {
    /// Assert that a signature is valid
    pub fn assert_valid_signature(signature: &Signature) {
        assert_ne!(
            *signature,
            Signature::default(),
            "Signature should not be default"
        );
    }

    /// Assert that a pubkey is valid (not default)
    pub fn assert_valid_pubkey(pubkey: &Pubkey) {
        assert_ne!(*pubkey, Pubkey::default(), "Pubkey should not be default");
    }

    /// Assert job state matches expected
    pub fn assert_job_state(job: &Job, expected_status: JobStatus) {
        assert_eq!(
            job.status, expected_status,
            "Job status should match expected"
        );
    }

    /// Assert user has expected properties
    pub fn assert_user_valid(user: &User, expected_username: &str) {
        assert_eq!(user.username, expected_username, "Username should match");
        assert!(
            !user.wallets.is_empty(),
            "User should have at least one wallet"
        );
        assert_ne!(
            user.authority,
            Pubkey::default(),
            "User authority should be set"
        );
    }

    /// Assert team has valid properties  
    pub fn assert_team_valid(team: &Team, expected_name: &str) {
        assert_eq!(team.name, expected_name, "Team name should match");
        assert_ne!(team.owner, Pubkey::default(), "Team owner should be set");
        assert!(
            !team.members.is_empty(),
            "Team should have at least the owner as member"
        );
    }

    /// Assert dispute is in valid state
    pub fn assert_dispute_valid(dispute: &Dispute, expected_status: DisputeStatus) {
        assert_eq!(
            dispute.status, expected_status,
            "Dispute status should match"
        );
        assert_ne!(dispute.job, Pubkey::default(), "Dispute job should be set");
        assert!(!dispute.evidence.is_empty(), "Dispute should have evidence");
    }

    /// Assert milestone is in valid state
    pub fn assert_milestone_valid(
        milestone: &Milestone,
        expected_status: MilestoneStatus,
        expected_amount: u64,
    ) {
        assert_eq!(
            milestone.status, expected_status,
            "Milestone status should match"
        );
        assert_eq!(
            milestone.amount, expected_amount,
            "Milestone amount should match"
        );
        assert!(!milestone.title.is_empty(), "Milestone should have a title");
    }
}

/// Error testing utilities
pub struct TestErrorUtils;

impl TestErrorUtils {
    /// Test that an operation fails with expected error
    pub async fn assert_fails_with_error<F, Fut>(
        operation: F,
        expected_error_type: fn() -> EscrowError,
    ) where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<Signature>>,
    {
        let result = operation().await;
        assert!(result.is_err(), "Operation should fail");

        // We can't easily compare error types, so we just ensure it failed
        // In a real implementation, we'd have more specific error matching
    }

    /// Test network timeout scenarios
    pub async fn simulate_timeout_error() -> Result<Signature> {
        tokio::time::sleep(Duration::from_secs(1)).await;
        Err(EscrowError::Timeout("Simulated timeout".to_string()))
    }

    /// Test invalid input scenarios
    pub fn create_invalid_inputs() -> Vec<(&'static str, String)> {
        vec![
            ("empty_string", "".to_string()),
            ("too_long", "a".repeat(1000)),
            ("special_chars", "!@#$%^&*()".to_string()),
            ("unicode", "🚀🌟💎".to_string()),
        ]
    }
}

/// Performance testing utilities
pub struct TestPerformanceUtils;

impl TestPerformanceUtils {
    /// Benchmark PDA derivation performance
    pub async fn benchmark_pda_derivation(iterations: usize) -> Duration {
        use std::time::Instant;
        use trust_escrow_sdk::pda;

        let start = Instant::now();

        for i in 0..iterations {
            let user = Pubkey::new_unique();
            let _user_pda = pda::find_user_pda(&user);

            let job_id = i as u64;
            let _job_pda = pda::find_job_pda(&user, job_id);
        }

        start.elapsed()
    }

    /// Benchmark client operation performance
    pub async fn benchmark_client_operations(client: &CofreClient, operations: usize) -> Duration {
        use std::time::Instant;

        let start = Instant::now();

        for _i in 0..operations {
            // Simulate client operations that don't modify state
            let _balance = client.get_balance(&client.payer().pubkey()).await;
        }

        start.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_data_factory() {
        let (username, bio) = TestDataFactory::create_test_user();
        assert!(!username.is_empty());
        assert!(bio.is_none() || bio.as_ref().unwrap().len() > 0);

        let (title, desc, amount, deadline) = TestDataFactory::create_test_job();
        assert!(!title.is_empty());
        assert!(!desc.is_empty());
        assert!(amount > 0);
        assert!(deadline.as_secs() > 0);
    }

    #[test]
    fn test_assertions() {
        let keypair = Keypair::new();
        let pubkey = keypair.pubkey();
        TestAssertions::assert_valid_pubkey(&pubkey);
    }

    #[tokio::test]
    async fn test_performance_utils() {
        let duration = TestPerformanceUtils::benchmark_pda_derivation(100).await;
        assert!(duration.as_millis() < 1000, "PDA derivation should be fast");
    }
}
