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

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Signature,
    signer::Signer,
    system_program,
    transaction::Transaction,
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
        let (config_pda, _config_bump) = pda::derive_config_pda()?;

        // Get recent blockhash
        let recent_blockhash = TransactionUtils::get_recent_blockhash(&self.rpc).await?;

        // Build instruction data manually
        // create_job discriminator from IDL: [178, 130, 217, 110, 100, 27, 82, 119]
        let mut instruction_data = vec![178, 130, 217, 110, 100, 27, 82, 119];

        // Add instruction arguments
        instruction_data.extend_from_slice(&job_id.to_le_bytes());

        // Add title (string - length + data)
        let title_bytes = title.as_bytes();
        instruction_data.extend_from_slice(&(title_bytes.len() as u32).to_le_bytes());
        instruction_data.extend_from_slice(title_bytes);

        // Add description (string - length + data)
        let description_bytes = description.as_bytes();
        instruction_data.extend_from_slice(&(description_bytes.len() as u32).to_le_bytes());
        instruction_data.extend_from_slice(description_bytes);

        // Add amount and deadline
        instruction_data.extend_from_slice(&amount.to_le_bytes());
        instruction_data.extend_from_slice(&deadline.to_le_bytes());

        // Build instruction with accounts from IDL
        let instruction = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(self.payer().pubkey(), true), // client
                AccountMeta::new(job_pda, false),              // job
                AccountMeta::new_readonly(config_pda, false),  // config
                AccountMeta::new_readonly(system_program::ID, false), // system_program
            ],
            data: instruction_data,
        };

        // Build and sign transaction
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.payer().pubkey()),
            &[&*self.payer()],
            recent_blockhash,
        );

        // Send transaction
        let signature = TransactionUtils::send_and_confirm_transaction(
            &self.rpc,
            &transaction,
            DEFAULT_COMMITMENT,
        )
        .await?;

        Ok((job_pda, signature))
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

        // Get recent blockhash
        let recent_blockhash = TransactionUtils::get_recent_blockhash(&self.rpc).await?;

        // Build instruction data manually
        // deposit_funds discriminator from IDL: [202, 39, 52, 211, 53, 20, 250, 88]
        let mut instruction_data = vec![202, 39, 52, 211, 53, 20, 250, 88];

        // Add job_id argument
        instruction_data.extend_from_slice(&job_id.to_le_bytes());

        // Build instruction with accounts from IDL
        let instruction = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(self.payer().pubkey(), true), // client
                AccountMeta::new(job_pda, false),              // job
                AccountMeta::new_readonly(config_pda, false),  // config
                AccountMeta::new_readonly(system_program::ID, false), // system_program
            ],
            data: instruction_data,
        };

        // Build and sign transaction
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.payer().pubkey()),
            &[&*self.payer()],
            recent_blockhash,
        );

        // Send transaction
        TransactionUtils::send_and_confirm_transaction(&self.rpc, &transaction, DEFAULT_COMMITMENT)
            .await
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

        // Get recent blockhash
        let recent_blockhash = TransactionUtils::get_recent_blockhash(&self.rpc).await?;

        // Build instruction data manually
        // approve_work discriminator from IDL: [181, 118, 45, 143, 204, 88, 237, 109]
        let mut instruction_data = vec![181, 118, 45, 143, 204, 88, 237, 109];

        // Add job_id argument
        instruction_data.extend_from_slice(&job_id.to_le_bytes());

        // Build instruction with accounts from IDL
        let instruction = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new_readonly(self.payer().pubkey(), true), // client
                AccountMeta::new(job_pda, false),                       // job
                AccountMeta::new(config_pda, false),                    // config
                AccountMeta::new(freelancer, false),                    // freelancer
                AccountMeta::new_readonly(system_program::ID, false),   // system_program
            ],
            data: instruction_data,
        };

        // Build and sign transaction
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.payer().pubkey()),
            &[&*self.payer()],
            recent_blockhash,
        );

        // Send transaction
        TransactionUtils::send_and_confirm_transaction(&self.rpc, &transaction, DEFAULT_COMMITMENT)
            .await
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

        // Get recent blockhash
        let recent_blockhash = TransactionUtils::get_recent_blockhash(&self.rpc).await?;

        // Build instruction data manually
        // cancel_job discriminator from IDL: [126, 241, 155, 241, 50, 236, 83, 118]
        let mut instruction_data = vec![126, 241, 155, 241, 50, 236, 83, 118];

        // Add job_id argument
        instruction_data.extend_from_slice(&job_id.to_le_bytes());

        // Build instruction with accounts from IDL
        let instruction = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new_readonly(self.payer().pubkey(), true), // client
                AccountMeta::new(job_pda, false),                       // job
                AccountMeta::new_readonly(config_pda, false),           // config
                AccountMeta::new_readonly(system_program::ID, false),   // system_program
            ],
            data: instruction_data,
        };

        // Build and sign transaction
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.payer().pubkey()),
            &[&*self.payer()],
            recent_blockhash,
        );

        // Send transaction
        TransactionUtils::send_and_confirm_transaction(&self.rpc, &transaction, DEFAULT_COMMITMENT)
            .await
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
        let account = self.rpc.get_account(&job_pda).map_err(|e| {
            EscrowError::network_error(format!("Failed to fetch job account: {}", e))
        })?;

        // Verify account is owned by our program
        if account.owner != PROGRAM_ID {
            return Err(EscrowError::invalid_account(
                "Account is not owned by Trust Escrow program",
            ));
        }

        // Deserialize account data manually
        // Skip the 8-byte discriminator at the beginning
        if account.data.len() < 8 {
            return Err(EscrowError::invalid_account("Account data too short"));
        }

        let data_slice = &account.data[8..];

        // Use borsh to deserialize the Job struct
        use borsh::BorshDeserialize;
        let job = Job::try_from_slice(data_slice).map_err(|e| {
            EscrowError::deserialization_error(format!("Failed to deserialize Job: {}", e))
        })?;

        Ok(job)
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
        let limit = limit.unwrap_or(10);

        use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};

        // Get all program accounts for jobs created by this client
        let payer_pubkey = self.payer().pubkey();

        // Create filter to get only job accounts for this client
        // Job accounts have seeds: [b"job", client_pubkey, job_id]
        let mut memcmp_data = vec![0u8; 8]; // Skip discriminator (8 bytes)
        memcmp_data.extend_from_slice(&[0u8; 8]); // Skip job_id field position (8 bytes for u64)
        memcmp_data.extend_from_slice(payer_pubkey.as_ref()); // Client pubkey (32 bytes)

        let config = RpcProgramAccountsConfig {
            filters: Some(vec![
                // Filter by account size (approximate Job struct size)
                solana_client::rpc_filter::RpcFilterType::DataSize(200), // Approximate size
                // Filter by client pubkey at the correct offset in job data
                solana_client::rpc_filter::RpcFilterType::Memcmp(
                    solana_client::rpc_filter::Memcmp::new_base58_encoded(
                        16, // Offset: skip discriminator (8) + job_id (8) = 16 bytes
                        payer_pubkey.as_ref(),
                    ),
                ),
            ]),
            account_config: RpcAccountInfoConfig {
                encoding: Some(solana_account_decoder::UiAccountEncoding::Base64),
                data_slice: None,
                commitment: Some(DEFAULT_COMMITMENT),
                min_context_slot: None,
            },
            with_context: Some(false),
        };

        let accounts = self
            .rpc
            .get_program_accounts_with_config(&PROGRAM_ID, config)
            .map_err(|e| {
                EscrowError::network_error(format!("Failed to get program accounts: {}", e))
            })?;

        let mut jobs = Vec::new();

        for (pubkey, account) in accounts {
            // Verify account is owned by our program
            if account.owner != PROGRAM_ID {
                continue;
            }

            // Skip if account data is too short
            if account.data.len() < 8 {
                continue;
            }

            // Deserialize job data
            let data_slice = &account.data[8..]; // Skip discriminator

            use borsh::BorshDeserialize;
            match Job::try_from_slice(data_slice) {
                Ok(job) => {
                    // Verify the job belongs to this client
                    if job.client == payer_pubkey {
                        jobs.push((pubkey, job));
                    }
                }
                Err(_) => continue, // Skip accounts that can't be deserialized as Job
            }

            // Apply limit
            if jobs.len() >= limit {
                break;
            }
        }

        // Sort by creation time (most recent first)
        jobs.sort_by(|(_, a), (_, b)| b.created_at.cmp(&a.created_at));

        Ok(jobs)
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
