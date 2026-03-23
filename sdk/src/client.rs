//! High-level client for Trust Escrow operations
//!
//! The CofreClient provides a type-safe, high-level interface for all Trust Escrow v2
//! operations. It wraps the lower-level Anchor client and provides convenient methods
//! for managing users, jobs, teams, disputes, and milestones.

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature, signer::Signer,
};
use std::sync::Arc;

use crate::error::{EscrowError, Result};
use crate::types::*;
use crate::utils::{TransactionUtils, ValidationUtils, DEFAULT_COMMITMENT};
use crate::PROGRAM_ID;

/// High-level client for Trust Escrow v2 operations
pub struct CofreClient {
    /// RPC client for Solana network communication
    rpc: Arc<RpcClient>,
    /// Default payer for transactions
    payer: Arc<dyn Signer>,
    /// Commitment level for transactions
    commitment: CommitmentConfig,
}

impl CofreClient {
    /// Create a new CofreClient instance
    ///
    /// # Arguments
    /// * `rpc` - RPC client for network communication
    /// * `payer` - Default keypair for signing transactions
    ///
    /// # Example
    /// ```rust,no_run
    /// use trust_escrow_sdk::CofreClient;
    /// use solana_client::rpc_client::RpcClient;
    /// use solana_sdk::signature::Keypair;
    /// use std::sync::Arc;
    ///
    /// let rpc = Arc::new(RpcClient::new("https://api.devnet.solana.com"));
    /// let payer = Arc::new(Keypair::new());
    /// let client = CofreClient::new(rpc, payer).unwrap();
    /// ```
    pub fn new(rpc: Arc<RpcClient>, payer: Arc<dyn Signer>) -> Result<Self> {
        let commitment = DEFAULT_COMMITMENT;

        Ok(CofreClient {
            rpc,
            payer,
            commitment,
        })
    }

    /// Create a new CofreClient with custom commitment
    pub fn new_with_commitment(
        rpc: Arc<RpcClient>,
        payer: Arc<dyn Signer>,
        commitment: CommitmentConfig,
    ) -> Result<Self> {
        Ok(CofreClient {
            rpc,
            payer,
            commitment,
        })
    }

    /// Get the RPC client
    pub fn rpc(&self) -> &RpcClient {
        &self.rpc
    }

    /// Get the default payer
    pub fn payer(&self) -> &dyn Signer {
        self.payer.as_ref()
    }

    /// Get the commitment config
    pub fn commitment(&self) -> CommitmentConfig {
        self.commitment
    }

    /// Get the program ID
    pub fn program_id(&self) -> Pubkey {
        PROGRAM_ID
    }

    /// Check if an account exists
    pub async fn account_exists(&self, pubkey: &Pubkey) -> Result<bool> {
        TransactionUtils::account_exists(&self.rpc, pubkey).await
    }

    /// Get account balance in lamports
    pub async fn get_balance(&self, pubkey: &Pubkey) -> Result<u64> {
        TransactionUtils::get_balance(&self.rpc, pubkey).await
    }

    /// Wait for transaction confirmation
    pub async fn wait_for_confirmation(&self, signature: &Signature) -> Result<bool> {
        TransactionUtils::wait_for_confirmation(&self.rpc, signature, self.commitment).await
    }

    // ===== USER OPERATIONS =====

    /// Create a new user account
    ///
    /// # Arguments
    /// * `username` - Unique username (max 32 characters)
    /// * `bio` - Optional bio (max 500 characters)
    ///
    /// # Returns
    /// Transaction signature
    ///
    /// # Example
    /// ```rust,no_run
    /// # use trust_escrow_sdk::CofreClient;
    /// # async fn example(client: CofreClient) -> trust_escrow_sdk::Result<()> {
    /// let signature = client.create_user("alice", Some("Freelance developer")).await?;
    /// println!("User created: {}", signature);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_user(&self, username: &str, bio: Option<&str>) -> Result<Signature> {
        // Validate inputs
        ValidationUtils::validate_username(username)?;
        if let Some(bio_text) = bio {
            ValidationUtils::validate_bio(bio_text)?;
        }

        // TODO: Implement using Anchor client
        // This is a placeholder for the actual implementation
        Err(EscrowError::sdk_error("create_user not yet implemented"))
    }

    /// Add a wallet to user account
    pub async fn add_wallet(&self, wallet: &Pubkey) -> Result<Signature> {
        // TODO: Implement using Anchor client
        Err(EscrowError::sdk_error("add_wallet not yet implemented"))
    }

    /// Set active wallet for user
    pub async fn set_active_wallet(&self, wallet: &Pubkey) -> Result<Signature> {
        // TODO: Implement using Anchor client
        Err(EscrowError::sdk_error(
            "set_active_wallet not yet implemented",
        ))
    }

    /// Update user bio
    pub async fn update_user(&self, bio: &str) -> Result<Signature> {
        ValidationUtils::validate_bio(bio)?;
        // TODO: Implement using Anchor client
        Err(EscrowError::sdk_error("update_user not yet implemented"))
    }

    // ===== JOB OPERATIONS =====

    /// Create a new job posting
    ///
    /// # Arguments
    /// * `job_id` - Unique job ID for this client
    /// * `title` - Job title
    /// * `description` - Job description
    /// * `amount` - Job amount in lamports
    ///
    /// # Returns
    /// (Job PDA, Transaction signature)
    pub async fn create_job(
        &self,
        job_id: u64,
        title: &str,
        description: &str,
        amount: u64,
    ) -> Result<(Pubkey, Signature)> {
        // Validate inputs
        ValidationUtils::validate_job_title(title)?;
        ValidationUtils::validate_job_description(description)?;
        ValidationUtils::validate_amount(amount, crate::MIN_JOB_AMOUNT)?;

        // TODO: Implement using Anchor client
        Err(EscrowError::sdk_error("create_job not yet implemented"))
    }

    /// Deposit funds to a job
    pub async fn deposit_funds(&self, job: &Pubkey) -> Result<Signature> {
        // TODO: Implement using Anchor client
        Err(EscrowError::sdk_error("deposit_funds not yet implemented"))
    }

    /// Apply to a job
    pub async fn apply_to_job(&self, job: &Pubkey, proposal: &str) -> Result<Signature> {
        if proposal.len() > 1000 {
            return Err(EscrowError::invalid_parameter(
                "Proposal cannot exceed 1000 characters",
            ));
        }

        // TODO: Implement using Anchor client
        Err(EscrowError::sdk_error("apply_to_job not yet implemented"))
    }

    /// Accept an application (client only)
    pub async fn accept_application(&self, job: &Pubkey, freelancer: &Pubkey) -> Result<Signature> {
        // TODO: Implement using Anchor client
        Err(EscrowError::sdk_error(
            "accept_application not yet implemented",
        ))
    }

    /// Submit work (freelancer only)
    pub async fn submit_work(&self, job: &Pubkey, work_url: &str) -> Result<Signature> {
        if work_url.len() > 500 {
            return Err(EscrowError::invalid_parameter(
                "Work URL cannot exceed 500 characters",
            ));
        }

        // TODO: Implement using Anchor client
        Err(EscrowError::sdk_error("submit_work not yet implemented"))
    }

    /// Approve work and release payment (client only)
    pub async fn approve_work(&self, job: &Pubkey) -> Result<Signature> {
        // TODO: Implement using Anchor client
        Err(EscrowError::sdk_error("approve_work not yet implemented"))
    }

    /// Reject work (client only)
    pub async fn reject_work(&self, job: &Pubkey, reason: &str) -> Result<Signature> {
        if reason.len() > 500 {
            return Err(EscrowError::invalid_parameter(
                "Rejection reason cannot exceed 500 characters",
            ));
        }

        // TODO: Implement using Anchor client
        Err(EscrowError::sdk_error("reject_work not yet implemented"))
    }

    /// Cancel job (client only, before work starts)
    pub async fn cancel_job(&self, job: &Pubkey) -> Result<Signature> {
        // TODO: Implement using Anchor client
        Err(EscrowError::sdk_error("cancel_job not yet implemented"))
    }

    // ===== TEAM OPERATIONS =====

    /// Create a new team
    pub async fn create_team(&self, name: &str, description: &str) -> Result<(Pubkey, Signature)> {
        ValidationUtils::validate_team_name(name)?;
        if description.len() > 500 {
            return Err(EscrowError::invalid_parameter(
                "Team description cannot exceed 500 characters",
            ));
        }

        // TODO: Implement using Anchor client
        Err(EscrowError::sdk_error("create_team not yet implemented"))
    }

    /// Add member to team
    pub async fn add_team_member(
        &self,
        team: &Pubkey,
        member: &Pubkey,
        role: MemberRole,
    ) -> Result<Signature> {
        // TODO: Implement using Anchor client
        Err(EscrowError::sdk_error(
            "add_team_member not yet implemented",
        ))
    }

    // ===== DISPUTE OPERATIONS =====

    /// Raise a dispute for a job
    pub async fn raise_dispute(&self, job: &Pubkey, evidence: &str) -> Result<(Pubkey, Signature)> {
        if evidence.len() > 2048 {
            return Err(EscrowError::invalid_parameter(
                "Dispute evidence cannot exceed 2048 characters",
            ));
        }

        // TODO: Implement using Anchor client
        Err(EscrowError::sdk_error("raise_dispute not yet implemented"))
    }

    /// Submit additional evidence to dispute
    pub async fn submit_evidence(&self, dispute: &Pubkey, evidence: &str) -> Result<Signature> {
        if evidence.len() > 2048 {
            return Err(EscrowError::invalid_parameter(
                "Evidence cannot exceed 2048 characters",
            ));
        }

        // TODO: Implement using Anchor client
        Err(EscrowError::sdk_error(
            "submit_evidence not yet implemented",
        ))
    }

    /// Resolve dispute (arbiter only)
    pub async fn resolve_dispute(
        &self,
        dispute: &Pubkey,
        client_percentage: u8,
    ) -> Result<Signature> {
        ValidationUtils::validate_percentage(client_percentage)?;

        // TODO: Implement using Anchor client
        Err(EscrowError::sdk_error(
            "resolve_dispute not yet implemented",
        ))
    }

    // ===== MILESTONE OPERATIONS =====

    /// Create a milestone for a job
    pub async fn create_milestone(
        &self,
        job: &Pubkey,
        title: &str,
        description: &str,
        amount: u64,
    ) -> Result<(Pubkey, Signature)> {
        ValidationUtils::validate_job_title(title)?; // Reuse job title validation
        if description.len() > 1000 {
            return Err(EscrowError::invalid_parameter(
                "Milestone description cannot exceed 1000 characters",
            ));
        }

        // TODO: Implement using Anchor client
        Err(EscrowError::sdk_error(
            "create_milestone not yet implemented",
        ))
    }

    /// Submit milestone work
    pub async fn submit_milestone(&self, milestone: &Pubkey, work_url: &str) -> Result<Signature> {
        if work_url.len() > 500 {
            return Err(EscrowError::invalid_parameter(
                "Work URL cannot exceed 500 characters",
            ));
        }

        // TODO: Implement using Anchor client
        Err(EscrowError::sdk_error(
            "submit_milestone not yet implemented",
        ))
    }

    /// Approve milestone and release payment
    pub async fn approve_milestone(&self, milestone: &Pubkey) -> Result<Signature> {
        // TODO: Implement using Anchor client
        Err(EscrowError::sdk_error(
            "approve_milestone not yet implemented",
        ))
    }

    /// Reject milestone
    pub async fn reject_milestone(&self, milestone: &Pubkey, reason: &str) -> Result<Signature> {
        if reason.len() > 500 {
            return Err(EscrowError::invalid_parameter(
                "Rejection reason cannot exceed 500 characters",
            ));
        }

        // TODO: Implement using Anchor client
        Err(EscrowError::sdk_error(
            "reject_milestone not yet implemented",
        ))
    }

    // ===== ACCOUNT FETCHING =====

    /// Fetch user account data
    pub async fn get_user(&self, user_pda: &Pubkey) -> Result<User> {
        // TODO: Implement using Anchor client
        Err(EscrowError::sdk_error("get_user not yet implemented"))
    }

    /// Fetch job account data
    pub async fn get_job(&self, job_pda: &Pubkey) -> Result<Job> {
        // TODO: Implement using Anchor client
        Err(EscrowError::sdk_error("get_job not yet implemented"))
    }

    /// Fetch team account data
    pub async fn get_team(&self, team_pda: &Pubkey) -> Result<Team> {
        // TODO: Implement using Anchor client
        Err(EscrowError::sdk_error("get_team not yet implemented"))
    }

    /// Fetch dispute account data
    pub async fn get_dispute(&self, dispute_pda: &Pubkey) -> Result<Dispute> {
        // TODO: Implement using Anchor client
        Err(EscrowError::sdk_error("get_dispute not yet implemented"))
    }

    /// Fetch milestone account data
    pub async fn get_milestone(&self, milestone_pda: &Pubkey) -> Result<Milestone> {
        // TODO: Implement using Anchor client
        Err(EscrowError::sdk_error("get_milestone not yet implemented"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{DevUtils, WalletUtils};

    #[test]
    fn test_client_creation() {
        let rpc = Arc::new(DevUtils::test_rpc_client());
        let payer = Arc::new(WalletUtils::generate_keypair());

        // This will fail without actual program deployment, but tests the interface
        let result = CofreClient::new(rpc, payer);
        assert!(result.is_err()); // Expected since program isn't deployed in test
    }

    #[test]
    fn test_input_validation() {
        // These test the validation without requiring network calls

        // Username validation
        assert!(ValidationUtils::validate_username("valid_user").is_ok());
        assert!(ValidationUtils::validate_username("").is_err());
        assert!(ValidationUtils::validate_username(&"x".repeat(33)).is_err());

        // Job title validation
        assert!(ValidationUtils::validate_job_title("Valid Job").is_ok());
        assert!(ValidationUtils::validate_job_title("").is_err());
        assert!(ValidationUtils::validate_job_title(&"x".repeat(101)).is_err());

        // Amount validation
        assert!(ValidationUtils::validate_amount(100_000, crate::MIN_JOB_AMOUNT).is_ok());
        assert!(ValidationUtils::validate_amount(50_000, crate::MIN_JOB_AMOUNT).is_err());
    }
}
