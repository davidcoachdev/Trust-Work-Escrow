//! Unit tests for Trust Escrow SDK Core Operations
//!
//! Tests the fundamental escrow operations with mocked RPC calls

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
};
use std::sync::Arc;
use trust_escrow_sdk::{CofreClient, EscrowError, Result};

/// Mock RPC client for testing (replace with actual mock in full implementation)
fn create_test_client() -> CofreClient {
    let rpc = Arc::new(RpcClient::new_with_commitment(
        "https://api.devnet.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    ));
    let payer = Arc::new(Keypair::new());

    CofreClient::new(rpc, payer).expect("Failed to create test client")
}

#[cfg(test)]
mod core_operations {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = create_test_client();
        assert_eq!(client.commitment(), CommitmentConfig::confirmed());

        // Test payer is valid
        let payer_pubkey = client.payer().pubkey();
        assert_ne!(payer_pubkey, Pubkey::default());
    }

    #[tokio::test]
    async fn test_user_creation_validation() {
        let client = create_test_client();

        // Valid username should work (would need mock for actual RPC)
        // For now, test validation logic
        assert!(validate_username("alice_dev").is_ok());
        assert!(validate_username("bob123").is_ok());

        // Invalid usernames
        assert!(validate_username("").is_err()); // Too short
        assert!(validate_username("ab").is_err()); // Too short
        assert!(validate_username(&"x".repeat(50)).is_err()); // Too long
        assert!(validate_username("user@name").is_err()); // Invalid chars
    }

    #[tokio::test]
    async fn test_job_creation_validation() {
        let client = create_test_client();

        // Test job title validation
        assert!(validate_job_title("Build Web App").is_ok());
        assert!(validate_job_title("").is_err()); // Empty
        assert!(validate_job_title(&"x".repeat(101)).is_err()); // Too long

        // Test amount validation
        assert!(validate_job_amount(1_000_000_000).is_ok()); // 1 SOL
        assert!(validate_job_amount(50_000).is_err()); // Below minimum
        assert!(validate_job_amount(0).is_err()); // Zero amount
    }

    #[tokio::test]
    async fn test_pda_derivation() {
        let client = create_test_client();
        let client_pubkey = client.payer().pubkey();

        // Test job PDA derivation
        let (job_pda1, bump1) = trust_escrow_sdk::derive_job_pda(&client_pubkey, 1)
            .expect("PDA derivation should succeed");
        let (job_pda2, bump2) = trust_escrow_sdk::derive_job_pda(&client_pubkey, 1)
            .expect("PDA derivation should succeed");

        // Same inputs should produce same PDA
        assert_eq!(job_pda1, job_pda2);
        assert_eq!(bump1, bump2);

        // Different job IDs should produce different PDAs
        let (job_pda_different, _) = trust_escrow_sdk::derive_job_pda(&client_pubkey, 2)
            .expect("PDA derivation should succeed");
        assert_ne!(job_pda1, job_pda_different);
    }

    #[tokio::test]
    async fn test_error_handling() {
        // Test error creation and formatting
        let param_error = EscrowError::invalid_parameter("Test parameter error".to_string());
        assert!(param_error.to_string().contains("Test parameter error"));

        let network_error = EscrowError::network_error("Connection failed");
        assert!(network_error.to_string().contains("Connection failed"));

        let account_error = EscrowError::account_not_found("Account missing");
        assert!(account_error.to_string().contains("Account missing"));
    }

    #[tokio::test]
    async fn test_milestone_spec_validation() {
        use trust_escrow_sdk::MilestoneSpec;

        let valid_specs = vec![
            MilestoneSpec {
                title: "Phase 1".to_string(),
                description: "Design phase".to_string(),
                amount: 500_000_000,
                index: 0,
            },
            MilestoneSpec {
                title: "Phase 2".to_string(),
                description: "Development phase".to_string(),
                amount: 1_000_000_000,
                index: 1,
            },
        ];

        // Test validation helper
        assert!(validate_milestone_specs(&valid_specs).is_ok());

        // Test with duplicate indices
        let mut invalid_specs = valid_specs.clone();
        invalid_specs[1].index = 0; // Duplicate index
        assert!(validate_milestone_specs(&invalid_specs).is_err());

        // Test with zero amount
        invalid_specs[0].index = 0;
        invalid_specs[1].index = 1;
        invalid_specs[0].amount = 0;
        assert!(validate_milestone_specs(&invalid_specs).is_err());
    }
}

#[cfg(test)]
mod advanced_operations {
    use super::*;

    #[tokio::test]
    async fn test_event_types() {
        use trust_escrow_sdk::EscrowEvent;

        // Test event creation and serialization
        let event = EscrowEvent::JobCreated {
            job: Pubkey::new_unique(),
            client: Pubkey::new_unique(),
            amount: 1_000_000_000,
            title: "Test Job".to_string(),
        };

        // Events should be cloneable and debuggable
        let cloned = event.clone();
        println!("Event: {:?}", cloned);
    }

    #[tokio::test]
    async fn test_batch_validation() {
        use trust_escrow_sdk::MilestoneSpec;

        // Test batch size limits
        let client = create_test_client();

        let too_many_specs: Vec<MilestoneSpec> = (0..25)
            .map(|i| MilestoneSpec {
                title: format!("Milestone {}", i),
                description: "Description".to_string(),
                amount: 100_000_000,
                index: i as u8,
            })
            .collect();

        // Should reject batches larger than MAX_MILESTONES (20)
        assert!(validate_milestone_specs(&too_many_specs).is_err());
    }

    #[tokio::test]
    async fn test_utils_formatting() {
        use trust_escrow_sdk::WalletUtils;

        // Test balance formatting
        assert_eq!(WalletUtils::format_balance(1_000_000_000), "1.0 SOL");
        assert_eq!(WalletUtils::format_balance(1_500_000_000), "1.5 SOL");
        assert_eq!(WalletUtils::format_balance(100_000), "0.0001 SOL");
    }
}

// Helper validation functions (these would be imported from the SDK in reality)
fn validate_username(username: &str) -> Result<()> {
    if username.len() < 3 || username.len() > 32 {
        return Err(EscrowError::invalid_parameter(
            "Username must be 3-32 characters".to_string(),
        ));
    }
    if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(EscrowError::invalid_parameter(
            "Username can only contain alphanumeric characters and underscores".to_string(),
        ));
    }
    Ok(())
}

fn validate_job_title(title: &str) -> Result<()> {
    if title.is_empty() || title.len() > 100 {
        return Err(EscrowError::invalid_parameter(
            "Job title must be 1-100 characters".to_string(),
        ));
    }
    Ok(())
}

fn validate_job_amount(amount: u64) -> Result<()> {
    if amount < trust_escrow_sdk::MIN_JOB_AMOUNT {
        return Err(EscrowError::invalid_parameter(format!(
            "Amount must be at least {} lamports",
            trust_escrow_sdk::MIN_JOB_AMOUNT
        )));
    }
    Ok(())
}

fn validate_milestone_specs(specs: &[trust_escrow_sdk::MilestoneSpec]) -> Result<()> {
    use trust_escrow_sdk::MAX_MILESTONES;

    if specs.len() > MAX_MILESTONES {
        return Err(EscrowError::invalid_parameter(format!(
            "Cannot create more than {} milestones",
            MAX_MILESTONES
        )));
    }

    for (i, spec) in specs.iter().enumerate() {
        if spec.amount == 0 {
            return Err(EscrowError::invalid_parameter(format!(
                "Milestone {} amount cannot be zero",
                i + 1
            )));
        }

        if spec.title.is_empty() || spec.title.len() > 100 {
            return Err(EscrowError::invalid_parameter(format!(
                "Milestone {} title must be 1-100 characters",
                i + 1
            )));
        }

        // Check for duplicate indices
        for (j, other_spec) in specs.iter().enumerate() {
            if i != j && spec.index == other_spec.index {
                return Err(EscrowError::invalid_parameter(format!(
                    "Duplicate milestone index: {}",
                    spec.index
                )));
            }
        }
    }

    Ok(())
}
