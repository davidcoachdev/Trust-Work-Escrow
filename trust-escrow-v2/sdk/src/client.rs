//! High-level client for Trust Escrow operations
//!
//! The CofreClient provides a type-safe, high-level interface for all Trust Escrow v2
//! operations. It wraps the lower-level Anchor client and provides convenient methods
//! for managing users, jobs, teams, disputes, and milestones.

//! High-level client for Trust Escrow operations
//!
//! The CofreClient provides a type-safe, high-level interface for all Trust Escrow v2
//! operations. It wraps the lower-level Anchor client and provides convenient methods
//! for managing users, jobs, teams, disputes, and milestones.

use anchor_client::{Client, Program};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature, signer::Signer,
    system_program,
};
use std::sync::Arc;

use crate::error::{EscrowError, Result};
use crate::pda;
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
    pub async fn add_wallet(&self, _wallet: &Pubkey) -> Result<Signature> {
        // TODO: Implement using Anchor client
        Err(EscrowError::sdk_error("add_wallet not yet implemented"))
    }

    /// Set active wallet for user
    pub async fn set_active_wallet(&self, _wallet: &Pubkey) -> Result<Signature> {
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

    // ===== CORE ESCROW OPERATIONS (Phase 2) =====

    /// 1. Create a new escrow (job posting)
    ///
    /// # Arguments
    /// * `job_id` - Unique job ID for this client
    /// * `title` - Job title
    /// * `description` - Job description
    /// * `amount` - Job amount in lamports
    /// * `deadline` - Job deadline as Unix timestamp
    ///
    /// # Returns
    /// (Job PDA, Transaction signature)
    ///
    /// # Example
    /// ```rust,no_run
    /// # use trust_escrow_sdk::CofreClient;
    /// # async fn example(client: CofreClient) -> trust_escrow_sdk::Result<()> {
    /// let deadline = chrono::Utc::now().timestamp() + 7 * 24 * 60 * 60; // 7 days
    /// let (job_pda, signature) = client.create_escrow(
    ///     1,
    ///     "Smart Contract Development",
    ///     "Build a Solana escrow contract",
    ///     1_000_000_000, // 1 SOL
    ///     deadline
    /// ).await?;
    /// println!("Escrow created: {} at {}", signature, job_pda);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_escrow(
        &self,
        job_id: u64,
        title: &str,
        description: &str,
        amount: u64,
        deadline: i64,
    ) -> Result<(Pubkey, Signature)> {
        // Validate inputs
        ValidationUtils::validate_job_title(title)?;
        ValidationUtils::validate_job_description(description)?;
        ValidationUtils::validate_amount(amount, crate::MIN_JOB_AMOUNT)?;

        // Derive PDAs
        let (job_pda, _job_bump) = pda::derive_job_pda(&self.payer().pubkey(), job_id)?;
        let (_config_pda, _config_bump) = pda::derive_config_pda()?;

        // TODO: Implement actual Anchor instruction call
        // For now return placeholder values to demonstrate API
        let placeholder_signature = Signature::default();

        Err(EscrowError::sdk_error(
            "create_escrow not yet implemented - needs Anchor client integration",
        ))
    }

    /// 2. Fund escrow by depositing required funds
    ///
    /// # Arguments
    /// * `job_id` - Job ID to fund
    ///
    /// # Returns
    /// Transaction signature
    ///
    /// # Example
    /// ```rust,no_run
    /// # use trust_escrow_sdk::CofreClient;
    /// # async fn example(client: CofreClient) -> trust_escrow_sdk::Result<()> {
    /// let signature = client.fund_escrow(1).await?;
    /// println!("Escrow funded: {}", signature);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fund_escrow(&self, job_id: u64) -> Result<Signature> {
        // Derive PDAs
        let (job_pda, _job_bump) = pda::derive_job_pda(&self.payer().pubkey(), job_id)?;
        let (config_pda, _config_bump) = pda::derive_config_pda()?;

        // Build and send transaction
        // TODO: Implement manual transaction building without Anchor client
        // For now return placeholder to make compilation work
        Err(EscrowError::sdk_error(
            "fund_escrow implementation pending - manual transaction building required",
        ))
    }

    /// 3. Release payment to freelancer (approve work)
    ///
    /// # Arguments
    /// * `job_id` - Job ID to release payment for
    /// * `freelancer` - Freelancer pubkey to receive payment
    ///
    /// # Returns
    /// Transaction signature
    ///
    /// # Example
    /// ```rust,no_run
    /// # use trust_escrow_sdk::CofreClient;
    /// # use solana_sdk::pubkey::Pubkey;
    /// # async fn example(client: CofreClient, freelancer: Pubkey) -> trust_escrow_sdk::Result<()> {
    /// let signature = client.release_payment(1, freelancer).await?;
    /// println!("Payment released: {}", signature);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn release_payment(&self, job_id: u64, freelancer: Pubkey) -> Result<Signature> {
        // Derive PDAs
        let (job_pda, _job_bump) = pda::derive_job_pda(&self.payer().pubkey(), job_id)?;
        let (config_pda, _config_bump) = pda::derive_config_pda()?;

        // Build and send transaction
        // TODO: Implement manual transaction building without Anchor client
        Err(EscrowError::sdk_error(
            "release_payment implementation pending - manual transaction building required",
        ))
    }

    /// 4. Refund escrow back to client (cancel job)
    ///
    /// # Arguments
    /// * `job_id` - Job ID to refund
    ///
    /// # Returns
    /// Transaction signature
    ///
    /// # Example
    /// ```rust,no_run
    /// # use trust_escrow_sdk::CofreClient;
    /// # async fn example(client: CofreClient) -> trust_escrow_sdk::Result<()> {
    /// let signature = client.refund_escrow(1).await?;
    /// println!("Escrow refunded: {}", signature);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn refund_escrow(&self, job_id: u64) -> Result<Signature> {
        // Derive PDAs
        let (job_pda, _job_bump) = pda::derive_job_pda(&self.payer().pubkey(), job_id)?;
        let (config_pda, _config_bump) = pda::derive_config_pda()?;

        // Build and send transaction
        // TODO: Implement manual transaction building without Anchor client
        Err(EscrowError::sdk_error(
            "refund_escrow implementation pending - manual transaction building required",
        ))
    }

    /// 5. Update escrow details (placeholder - not supported by v2 contract)
    ///
    /// # Arguments
    /// * `job_id` - Job ID to update
    /// * `new_title` - New job title (optional)
    /// * `new_description` - New job description (optional)
    ///
    /// # Returns
    /// Error indicating operation not supported
    ///
    /// # Note
    /// The Trust Escrow v2 contract doesn't support job updates after creation.
    /// This method is provided for API completeness but will always return an error.
    pub async fn update_escrow(
        &self,
        _job_id: u64,
        _new_title: Option<&str>,
        _new_description: Option<&str>,
    ) -> Result<Signature> {
        Err(EscrowError::not_permitted(
            "Trust Escrow v2 contract does not support job updates after creation. Cancel and create a new job instead.",
        ))
    }

    /// 6. Cancel escrow (same as refund_escrow for API consistency)
    ///
    /// # Arguments
    /// * `job_id` - Job ID to cancel
    ///
    /// # Returns
    /// Transaction signature
    ///
    /// # Example
    /// ```rust,no_run
    /// # use trust_escrow_sdk::CofreClient;
    /// # async fn example(client: CofreClient) -> trust_escrow_sdk::Result<()> {
    /// let signature = client.cancel_escrow(1).await?;
    /// println!("Escrow cancelled: {}", signature);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn cancel_escrow(&self, job_id: u64) -> Result<Signature> {
        // Delegate to refund_escrow for consistency
        self.refund_escrow(job_id).await
    }

    /// 7. Get escrow account data
    ///
    /// # Arguments
    /// * `job_id` - Job ID to fetch
    ///
    /// # Returns
    /// Job account data
    ///
    /// # Example
    /// ```rust,no_run
    /// # use trust_escrow_sdk::CofreClient;
    /// # async fn example(client: CofreClient) -> trust_escrow_sdk::Result<()> {
    /// let job = client.get_escrow(1).await?;
    /// println!("Job status: {:?}", job.status);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_escrow(&self, job_id: u64) -> Result<Job> {
        // Derive job PDA
        let (job_pda, _job_bump) = pda::derive_job_pda(&self.payer().pubkey(), job_id)?;

        // Fetch account data using RPC client directly
        // TODO: Implement manual account fetching and deserialization without Anchor client
        Err(EscrowError::sdk_error(
            "get_escrow implementation pending - manual account fetching required",
        ))
    }

    /// 8. List multiple escrows for the current payer
    ///
    /// # Arguments
    /// * `limit` - Maximum number of escrows to return (optional, default 10)
    ///
    /// # Returns
    /// Vector of (Pubkey, Job) tuples
    ///
    /// # Example
    /// ```rust,no_run
    /// # use trust_escrow_sdk::CofreClient;
    /// # async fn example(client: CofreClient) -> trust_escrow_sdk::Result<()> {
    /// let escrows = client.list_escrows(Some(20)).await?;
    /// for (pubkey, job) in escrows {
    ///     println!("Job {}: {} - {:?}", job.job_id, job.title, job.status);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_escrows(&self, limit: Option<usize>) -> Result<Vec<(Pubkey, Job)>> {
        let _limit = limit.unwrap_or(10);

        // TODO: Implement using getProgramAccounts with proper filters and manual deserialization
        Err(EscrowError::sdk_error(
            "list_escrows implementation pending - requires manual account deserialization",
        ))
    }

    // ===== ACCOUNT FETCHING =====

    /// Fetch user account data
    pub async fn get_user(&self, _user_pda: &Pubkey) -> Result<User> {
        // TODO: Implement using Anchor client
        Err(EscrowError::sdk_error("get_user not yet implemented"))
    }

    /// Fetch job account data
    pub async fn get_job(&self, _job_pda: &Pubkey) -> Result<Job> {
        // TODO: Implement using Anchor client
        Err(EscrowError::sdk_error("get_job not yet implemented"))
    }

    /// Fetch team account data
    pub async fn get_team(&self, _team_pda: &Pubkey) -> Result<Team> {
        // TODO: Implement using Anchor client
        Err(EscrowError::sdk_error("get_team not yet implemented"))
    }

    /// Fetch dispute account data
    pub async fn get_dispute(&self, _dispute_pda: &Pubkey) -> Result<Dispute> {
        // TODO: Implement using Anchor client
        Err(EscrowError::sdk_error("get_dispute not yet implemented"))
    }

    /// Fetch milestone account data
    pub async fn get_milestone(&self, _milestone_pda: &Pubkey) -> Result<Milestone> {
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
