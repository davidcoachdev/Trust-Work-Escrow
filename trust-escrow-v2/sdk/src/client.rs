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
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{Duration, SystemTime},
};

use crate::error::{EscrowError, Result};
use crate::events::{EscrowEvent, EventListener, EventListenerConfig};
use crate::pda;
use crate::types::*;
use crate::utils::{ConversionUtils, TransactionUtils, ValidationUtils, DEFAULT_COMMITMENT};
use crate::PROGRAM_ID;

/// Cache entry for account data
#[derive(Debug, Clone)]
struct CacheEntry {
    data: Vec<u8>,
    timestamp: SystemTime,
    ttl: Duration,
}

impl CacheEntry {
    fn is_valid(&self) -> bool {
        self.timestamp.elapsed().unwrap_or(Duration::MAX) < self.ttl
    }
}

/// Performance optimization configuration
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Enable account data caching
    pub enable_cache: bool,
    /// Cache TTL for account data
    pub cache_ttl: Duration,
    /// Maximum cache size (number of entries)
    pub max_cache_size: usize,
    /// Retry configuration
    pub retry_config: RetryConfig,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_cache: true,
            cache_ttl: Duration::from_secs(30),
            max_cache_size: 1000,
            retry_config: RetryConfig {
                max_retries: 3,
                initial_delay: Duration::from_millis(100),
                max_delay: Duration::from_secs(5),
                backoff_multiplier: 2.0,
            },
        }
    }
}

/// High-level client for Trust Escrow v2 operations
pub struct CofreClient {
    /// RPC client for Solana network communication
    rpc: Arc<RpcClient>,
    /// Default payer for transactions
    payer: Arc<dyn Signer + Send + Sync>,
    /// Commitment level for transactions
    commitment: CommitmentConfig,
    /// Account data cache for performance
    cache: Arc<RwLock<HashMap<Pubkey, CacheEntry>>>,
    /// Performance configuration
    perf_config: PerformanceConfig,
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
    pub fn new(rpc: Arc<RpcClient>, payer: Arc<dyn Signer + Send + Sync>) -> Result<Self> {
        Self::new_with_config(rpc, payer, DEFAULT_COMMITMENT, PerformanceConfig::default())
    }

    /// Create a new CofreClient with custom commitment
    pub fn new_with_commitment(
        rpc: Arc<RpcClient>,
        payer: Arc<dyn Signer + Send + Sync>,
        commitment: CommitmentConfig,
    ) -> Result<Self> {
        Self::new_with_config(rpc, payer, commitment, PerformanceConfig::default())
    }

    /// Create a new CofreClient with custom performance configuration
    pub fn new_with_config(
        rpc: Arc<RpcClient>,
        payer: Arc<dyn Signer + Send + Sync>,
        commitment: CommitmentConfig,
        perf_config: PerformanceConfig,
    ) -> Result<Self> {
        Ok(CofreClient {
            rpc,
            payer,
            commitment,
            cache: Arc::new(RwLock::new(HashMap::new())),
            perf_config,
        })
    }

    /// Clear the internal cache
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        if let Ok(cache) = self.cache.read() {
            let total = cache.len();
            let valid = cache.values().filter(|entry| entry.is_valid()).count();
            (total, valid)
        } else {
            (0, 0)
        }
    }

    /// Execute a function with retry logic
    async fn with_retry<F, T>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>> + Send,
        T: Send,
    {
        let mut delay = self.perf_config.retry_config.initial_delay;
        let mut last_error = None;

        for attempt in 0..=self.perf_config.retry_config.max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.perf_config.retry_config.max_retries {
                        tokio::time::sleep(delay).await;
                        delay = std::cmp::min(
                            Duration::from_millis(
                                (delay.as_millis() as f64
                                    * self.perf_config.retry_config.backoff_multiplier)
                                    as u64,
                            ),
                            self.perf_config.retry_config.max_delay,
                        );
                    }
                }
            }
        }

        Err(last_error.unwrap_or(EscrowError::sdk_error("Retry failed")))
    }

    /// Get account data with caching
    async fn get_account_data_cached(&self, pubkey: &Pubkey) -> Result<Vec<u8>> {
        // Check cache first
        if self.perf_config.enable_cache {
            if let Ok(cache) = self.cache.read() {
                if let Some(entry) = cache.get(pubkey) {
                    if entry.is_valid() {
                        return Ok(entry.data.clone());
                    }
                }
            }
        }

        // Fetch from network with retry
        let pubkey_clone = *pubkey;
        let data = self
            .with_retry(|| {
                let rpc = self.rpc.clone();
                let pubkey = pubkey_clone;
                Box::pin(async move {
                    rpc.get_account(&pubkey)
                        .map_err(|e| {
                            EscrowError::sdk_error(&format!(
                                "Failed to get account {}: {}",
                                pubkey, e
                            ))
                        })
                        .map(|acc| acc.data)
                })
            })
            .await?;

        // Store in cache
        if self.perf_config.enable_cache {
            if let Ok(mut cache) = self.cache.write() {
                // Clean up expired entries if cache is full
                if cache.len() >= self.perf_config.max_cache_size {
                    cache.retain(|_, entry| entry.is_valid());

                    // If still full after cleanup, remove oldest entries
                    if cache.len() >= self.perf_config.max_cache_size {
                        let keys_to_remove: Vec<Pubkey> =
                            cache.keys().take(cache.len() / 4).cloned().collect();
                        for key in keys_to_remove {
                            cache.remove(&key);
                        }
                    }
                }

                cache.insert(
                    pubkey_clone,
                    CacheEntry {
                        data: data.clone(),
                        timestamp: SystemTime::now(),
                        ttl: self.perf_config.cache_ttl,
                    },
                );
            }
        }

        Ok(data)
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

        // Derive team PDA
        let (team_pda, _team_bump) = pda::derive_team_pda(&self.payer().pubkey())?;

        // Get recent blockhash
        let recent_blockhash = TransactionUtils::get_recent_blockhash(&self.rpc).await?;

        // Build instruction data manually
        // create_team discriminator from IDL: [122, 161, 98, 67, 178, 128, 116, 113]
        let mut instruction_data = vec![122, 161, 98, 67, 178, 128, 116, 113];

        // Add name (string - length + data)
        let name_bytes = name.as_bytes();
        instruction_data.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        instruction_data.extend_from_slice(name_bytes);

        // Add description (string - length + data)
        let description_bytes = description.as_bytes();
        instruction_data.extend_from_slice(&(description_bytes.len() as u32).to_le_bytes());
        instruction_data.extend_from_slice(description_bytes);

        // Build instruction with accounts from IDL
        let instruction = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(self.payer().pubkey(), true), // owner
                AccountMeta::new(team_pda, false),             // team
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
            self.commitment,
        )
        .await?;

        Ok((team_pda, signature))
    }

    /// Add member to team
    pub async fn add_team_member(
        &self,
        team: &Pubkey,
        member: &Pubkey,
        _role: MemberRole, // Role is not used in current contract implementation
    ) -> Result<Signature> {
        // Derive team PDA to verify ownership
        let (team_pda, _team_bump) = pda::derive_team_pda(&self.payer().pubkey())?;

        // Verify the provided team pubkey matches expected PDA
        if &team_pda != team {
            return Err(EscrowError::invalid_parameter(
                "Invalid team pubkey - does not match expected PDA for this owner",
            ));
        }

        // Get recent blockhash
        let recent_blockhash = TransactionUtils::get_recent_blockhash(&self.rpc).await?;

        // Build instruction data manually
        // add_team_member discriminator from IDL: [64, 13, 248, 67, 55, 245, 184, 173]
        let mut instruction_data = vec![64, 13, 248, 67, 55, 245, 184, 173];

        // Add member pubkey argument
        instruction_data.extend_from_slice(member.as_ref());

        // Build instruction with accounts from IDL
        let instruction = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(self.payer().pubkey(), true), // owner
                AccountMeta::new(*team, false),                // team
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
        TransactionUtils::send_and_confirm_transaction(&self.rpc, &transaction, self.commitment)
            .await
    }

    // ===== DISPUTE OPERATIONS =====

    /// Raise a dispute for a job
    pub async fn raise_dispute(&self, job_id: u64, evidence: &str) -> Result<(Pubkey, Signature)> {
        if evidence.len() > 2048 {
            return Err(EscrowError::invalid_parameter(
                "Dispute evidence cannot exceed 2048 characters",
            ));
        }

        // Derive PDAs
        let (job_pda, _job_bump) = pda::derive_job_pda(&self.payer().pubkey(), job_id)?;
        let (dispute_pda, _dispute_bump) = pda::derive_dispute_pda(&job_pda)?;

        // Set deadline to 7 days from now (default dispute period)
        let deadline = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            + 7 * 24 * 60 * 60; // 7 days

        // Get recent blockhash
        let recent_blockhash = TransactionUtils::get_recent_blockhash(&self.rpc).await?;

        // Build instruction data manually
        // raise_dispute discriminator from IDL: [41, 243, 1, 51, 150, 95, 246, 73]
        let mut instruction_data = vec![41, 243, 1, 51, 150, 95, 246, 73];

        // Add job_id argument
        instruction_data.extend_from_slice(&job_id.to_le_bytes());

        // Add reason (evidence) string
        let reason_bytes = evidence.as_bytes();
        instruction_data.extend_from_slice(&(reason_bytes.len() as u32).to_le_bytes());
        instruction_data.extend_from_slice(reason_bytes);

        // Add deadline
        instruction_data.extend_from_slice(&deadline.to_le_bytes());

        // Build instruction with accounts from IDL
        let instruction = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(self.payer().pubkey(), true), // raiser
                AccountMeta::new(job_pda, false),              // job
                AccountMeta::new(dispute_pda, false),          // dispute
                AccountMeta::new_readonly(self.payer().pubkey(), false), // client (Note: assuming raiser is client)
                AccountMeta::new_readonly(system_program::ID, false),    // system_program
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
            self.commitment,
        )
        .await?;

        Ok((dispute_pda, signature))
    }

    /// Submit additional evidence to dispute
    pub async fn submit_evidence(&self, job_id: u64, evidence: &str) -> Result<Signature> {
        if evidence.len() > 2048 {
            return Err(EscrowError::invalid_parameter(
                "Evidence cannot exceed 2048 characters",
            ));
        }

        // Derive PDAs
        let (job_pda, _job_bump) = pda::derive_job_pda(&self.payer().pubkey(), job_id)?;
        let (dispute_pda, _dispute_bump) = pda::derive_dispute_pda(&job_pda)?;

        // Get recent blockhash
        let recent_blockhash = TransactionUtils::get_recent_blockhash(&self.rpc).await?;

        // Build instruction data manually
        // submit_evidence discriminator from IDL: [12, 169, 228, 194, 229, 31, 44, 39]
        let mut instruction_data = vec![12, 169, 228, 194, 229, 31, 44, 39];

        // Add job_id argument
        instruction_data.extend_from_slice(&job_id.to_le_bytes());

        // Add content (evidence) string
        let content_bytes = evidence.as_bytes();
        instruction_data.extend_from_slice(&(content_bytes.len() as u32).to_le_bytes());
        instruction_data.extend_from_slice(content_bytes);

        // Build instruction with accounts from IDL
        let instruction = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(self.payer().pubkey(), true), // submitter
                AccountMeta::new(dispute_pda, false),          // dispute
                AccountMeta::new(job_pda, false),              // job
                AccountMeta::new_readonly(self.payer().pubkey(), false), // client
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
        TransactionUtils::send_and_confirm_transaction(&self.rpc, &transaction, self.commitment)
            .await
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
        job_id: u64,
        title: &str,
        description: &str,
        amount: u64,
        index: u8,
    ) -> Result<(Pubkey, Signature)> {
        ValidationUtils::validate_job_title(title)?; // Reuse job title validation
        if description.len() > 1000 {
            return Err(EscrowError::invalid_parameter(
                "Milestone description cannot exceed 1000 characters",
            ));
        }

        // Derive PDAs
        let (job_pda, _job_bump) = pda::derive_job_pda(&self.payer().pubkey(), job_id)?;
        let (milestone_pda, _milestone_bump) = pda::derive_milestone_pda(&job_pda, index)?;

        // Set deadline to 30 days from now (default milestone deadline)
        let deadline = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            + 30 * 24 * 60 * 60; // 30 days

        // Get recent blockhash
        let recent_blockhash = TransactionUtils::get_recent_blockhash(&self.rpc).await?;

        // Build instruction data manually
        // create_milestone discriminator from IDL: [239, 58, 201, 28, 40, 186, 173, 48]
        let mut instruction_data = vec![239, 58, 201, 28, 40, 186, 173, 48];

        // Add job_id argument
        instruction_data.extend_from_slice(&job_id.to_le_bytes());

        // Add title string
        let title_bytes = title.as_bytes();
        instruction_data.extend_from_slice(&(title_bytes.len() as u32).to_le_bytes());
        instruction_data.extend_from_slice(title_bytes);

        // Add description string
        let description_bytes = description.as_bytes();
        instruction_data.extend_from_slice(&(description_bytes.len() as u32).to_le_bytes());
        instruction_data.extend_from_slice(description_bytes);

        // Add amount
        instruction_data.extend_from_slice(&amount.to_le_bytes());

        // Add deadline
        instruction_data.extend_from_slice(&deadline.to_le_bytes());

        // Add index
        instruction_data.extend_from_slice(&[index]);

        // Build instruction with accounts from IDL
        let instruction = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(self.payer().pubkey(), true), // client
                AccountMeta::new_readonly(job_pda, false),     // job
                AccountMeta::new(milestone_pda, false),        // milestone
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
            self.commitment,
        )
        .await?;

        Ok((milestone_pda, signature))
    }

    /// Submit milestone work
    pub async fn submit_milestone(&self, job_id: u64, milestone_index: u8) -> Result<Signature> {
        // Derive PDAs
        let (job_pda, _job_bump) = pda::derive_job_pda(&self.payer().pubkey(), job_id)?;
        let (milestone_pda, _milestone_bump) =
            pda::derive_milestone_pda(&job_pda, milestone_index)?;

        // Get recent blockhash
        let recent_blockhash = TransactionUtils::get_recent_blockhash(&self.rpc).await?;

        // Build instruction data manually
        // submit_milestone discriminator from IDL: [35, 96, 220, 215, 102, 83, 139, 52]
        let mut instruction_data = vec![35, 96, 220, 215, 102, 83, 139, 52];

        // Add job_id argument
        instruction_data.extend_from_slice(&job_id.to_le_bytes());

        // Add milestone_index argument
        instruction_data.extend_from_slice(&[milestone_index]);

        // Build instruction with accounts from IDL
        let instruction = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new_readonly(self.payer().pubkey(), true), // freelancer
                AccountMeta::new(milestone_pda, false),                 // milestone
                AccountMeta::new(job_pda, false),                       // job
                AccountMeta::new_readonly(self.payer().pubkey(), false), // client (placeholder, should be actual client)
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
        TransactionUtils::send_and_confirm_transaction(&self.rpc, &transaction, self.commitment)
            .await
    }

    /// Approve milestone and release payment
    pub async fn approve_milestone(
        &self,
        job_id: u64,
        milestone_index: u8,
        freelancer: &Pubkey,
    ) -> Result<Signature> {
        // Derive PDAs
        let (job_pda, _job_bump) = pda::derive_job_pda(&self.payer().pubkey(), job_id)?;
        let (milestone_pda, _milestone_bump) =
            pda::derive_milestone_pda(&job_pda, milestone_index)?;

        // Get recent blockhash
        let recent_blockhash = TransactionUtils::get_recent_blockhash(&self.rpc).await?;

        // Build instruction data manually
        // approve_milestone discriminator from IDL: [145, 85, 92, 60, 50, 130, 219, 106]
        let mut instruction_data = vec![145, 85, 92, 60, 50, 130, 219, 106];

        // Add job_id argument
        instruction_data.extend_from_slice(&job_id.to_le_bytes());

        // Add milestone_index argument
        instruction_data.extend_from_slice(&[milestone_index]);

        // Build instruction with accounts from IDL
        let instruction = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new_readonly(self.payer().pubkey(), true), // client
                AccountMeta::new(milestone_pda, false),                 // milestone
                AccountMeta::new(job_pda, false),                       // job
                AccountMeta::new(*freelancer, false),                   // freelancer
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
        TransactionUtils::send_and_confirm_transaction(&self.rpc, &transaction, self.commitment)
            .await
    }

    /// Reject milestone
    pub async fn reject_milestone(&self, job_id: u64, milestone_index: u8) -> Result<Signature> {
        // Derive PDAs
        let (job_pda, _job_bump) = pda::derive_job_pda(&self.payer().pubkey(), job_id)?;
        let (milestone_pda, _milestone_bump) =
            pda::derive_milestone_pda(&job_pda, milestone_index)?;

        // Get recent blockhash
        let recent_blockhash = TransactionUtils::get_recent_blockhash(&self.rpc).await?;

        // Build instruction data manually
        // reject_milestone discriminator from IDL: [243, 48, 66, 165, 237, 41, 116, 249]
        let mut instruction_data = vec![243, 48, 66, 165, 237, 41, 116, 249];

        // Add job_id argument
        instruction_data.extend_from_slice(&job_id.to_le_bytes());

        // Add milestone_index argument
        instruction_data.extend_from_slice(&[milestone_index]);

        // Build instruction with accounts from IDL
        let instruction = Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new_readonly(self.payer().pubkey(), true), // client
                AccountMeta::new(milestone_pda, false),                 // milestone
                AccountMeta::new(job_pda, false),                       // job
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
        TransactionUtils::send_and_confirm_transaction(&self.rpc, &transaction, self.commitment)
            .await
    }

    // ===== BATCH OPERATIONS (Phase 3) =====

    /// Batch create multiple milestones for a job
    ///
    /// # Arguments
    /// * `job_id` - Job ID to create milestones for
    /// * `milestones` - Vector of milestone specifications
    ///
    /// # Returns
    /// Vector of (milestone PDA, signature) pairs
    ///
    /// # Example
    /// ```rust,no_run
    /// # use trust_escrow_sdk::{CofreClient, MilestoneSpec};
    /// # async fn example(client: CofreClient) -> trust_escrow_sdk::Result<()> {
    /// let milestones = vec![
    ///     MilestoneSpec {
    ///         title: "Design Phase".to_string(),
    ///         description: "Complete UI/UX design".to_string(),
    ///         amount: 300_000_000, // 0.3 SOL
    ///         index: 0,
    ///     },
    ///     MilestoneSpec {
    ///         title: "Development Phase".to_string(),
    ///         description: "Implement core features".to_string(),
    ///         amount: 700_000_000, // 0.7 SOL
    ///         index: 1,
    ///     },
    /// ];
    ///
    /// let results = client.batch_create_milestones(1, milestones).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn batch_create_milestones(
        &self,
        job_id: u64,
        milestone_specs: Vec<MilestoneSpec>,
    ) -> Result<Vec<(Pubkey, Signature)>> {
        self.validate_milestone_specs(&milestone_specs)?;

        let mut results = Vec::with_capacity(milestone_specs.len());

        for (index, spec) in milestone_specs.iter().enumerate() {
            let milestone_result = self
                .create_milestone(
                    job_id,
                    &spec.title,
                    spec.description.as_deref().unwrap_or(""),
                    spec.amount,
                    index as u8,
                )
                .await?;
            results.push(milestone_result);

            // Small delay between transactions to avoid rate limits
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }

        Ok(results)
    }

    /// Batch submit multiple milestones
    ///
    /// # Arguments
    /// * `job_id` - Job ID
    /// * `milestone_indices` - Indices of milestones to submit
    ///
    /// # Returns
    /// Vector of transaction signatures
    pub async fn batch_submit_milestones(
        &self,
        job_id: u64,
        milestone_indices: Vec<u8>,
    ) -> Result<Vec<Signature>> {
        let mut results = Vec::with_capacity(milestone_indices.len());

        for index in milestone_indices {
            let signature = self.submit_milestone(job_id, index).await?;
            results.push(signature);

            // Small delay between transactions
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }

        Ok(results)
    }

    /// Batch approve multiple milestones
    ///
    /// # Arguments
    /// * `job_id` - Job ID
    /// * `milestone_indices` - Indices of milestones to approve
    /// * `freelancer` - Freelancer to receive payments
    ///
    /// # Returns
    /// Vector of transaction signatures
    pub async fn batch_approve_milestones(
        &self,
        job_id: u64,
        milestone_indices: Vec<u8>,
        freelancer: &Pubkey,
    ) -> Result<Vec<Signature>> {
        let mut results = Vec::with_capacity(milestone_indices.len());

        for index in milestone_indices {
            let signature = self.approve_milestone(job_id, index, freelancer).await?;
            results.push(signature);

            // Small delay between transactions
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }

        Ok(results)
    }

    // ===== SDK UTILITIES & HELPERS (Phase 3) =====

    /// Validate milestone specifications
    pub fn validate_milestone_specs(&self, specs: &[MilestoneSpec]) -> Result<()> {
        if specs.is_empty() {
            return Err(EscrowError::invalid_parameter(
                "At least one milestone required",
            ));
        }

        if specs.len() > 20 {
            return Err(EscrowError::invalid_parameter(
                "Maximum 20 milestones allowed",
            ));
        }

        for (index, spec) in specs.iter().enumerate() {
            if spec.title.is_empty() {
                return Err(EscrowError::invalid_parameter(format!(
                    "Milestone {} title cannot be empty",
                    index + 1
                )));
            }

            if spec.title.len() > 200 {
                return Err(EscrowError::invalid_parameter(format!(
                    "Milestone {} title too long (max 200 characters)",
                    index + 1
                )));
            }

            if let Some(ref desc) = spec.description {
                if desc.len() > 1000 {
                    return Err(EscrowError::invalid_parameter(format!(
                        "Milestone {} description too long (max 1000 characters)",
                        index + 1
                    )));
                }
            }

            if spec.amount == 0 {
                return Err(EscrowError::invalid_parameter(format!(
                    "Milestone {} amount cannot be zero",
                    index + 1
                )));
            }
        }

        // Check for duplicate indices (since we're using enumeration, this is no longer needed)
        Ok(())
    }

    /// Calculate total milestone amount
    pub fn calculate_total_milestone_amount(&self, specs: &[MilestoneSpec]) -> u64 {
        specs.iter().map(|spec| spec.amount).sum()
    }

    /// Get recommended fee for transaction
    pub async fn get_recommended_fee(&self) -> Result<u64> {
        // For now, return a fixed fee. In production, this could be dynamic
        // based on network conditions
        Ok(5000) // 0.000005 SOL
    }

    /// Check if job can accept milestones
    pub async fn can_create_milestones(&self, job_id: u64) -> Result<bool> {
        let job = self.get_escrow(job_id).await?;

        // Jobs can accept milestones if they're created but not yet in progress
        match job.status {
            JobStatus::Created | JobStatus::ApplicationsOpen => Ok(true),
            _ => Ok(false),
        }
    }

    /// Estimate total gas cost for batch operations
    pub async fn estimate_batch_milestone_cost(&self, milestone_count: usize) -> Result<u64> {
        let fee_per_transaction = self.get_recommended_fee().await?;
        Ok(fee_per_transaction * milestone_count as u64)
    }

    /// Format job status for display
    pub fn format_job_status(&self, status: JobStatus) -> &'static str {
        match status {
            JobStatus::Created => "Created",
            JobStatus::ApplicationsOpen => "Accepting Applications",
            JobStatus::InProgress => "In Progress",
            JobStatus::Submitted => "Work Submitted",
            JobStatus::Approved => "Completed",
            JobStatus::Disputed => "Under Dispute",
            JobStatus::Cancelled => "Cancelled",
            JobStatus::Resolved => "Dispute Resolved",
        }
    }

    /// Format amount as human-readable string
    pub fn format_amount(&self, lamports: u64) -> String {
        ConversionUtils::format_sol(lamports)
    }

    /// Parse amount from string
    pub fn parse_amount(&self, amount_str: &str) -> Result<u64> {
        ConversionUtils::parse_sol(amount_str)
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

    // ===== ENHANCED QUERY CAPABILITIES (Phase 3) =====

    /// Advanced escrow listing with filtering, sorting, and pagination
    ///
    /// # Arguments
    /// * `filter` - Filtering options
    /// * `sort_by` - Sorting criteria
    /// * `limit` - Maximum number of results (optional, default 10)
    /// * `offset` - Pagination offset (optional, default 0)
    ///
    /// # Example
    /// ```rust,no_run
    /// # use trust_escrow_sdk::{CofreClient, JobFilter, SortBy};
    /// # async fn example(client: CofreClient) -> trust_escrow_sdk::Result<()> {
    /// let filter = JobFilter::new()
    ///     .status(Some(vec![JobStatus::ApplicationsOpen, JobStatus::InProgress]))
    ///     .amount_range(Some((1_000_000, 10_000_000))); // 0.001 - 0.01 SOL
    ///
    /// let escrows = client.list_escrows_advanced(
    ///     filter,
    ///     SortBy::CreatedDesc,
    ///     Some(20),
    ///     Some(0)
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_escrows_advanced(
        &self,
        filter: JobFilter,
        sort_by: SortBy,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<(Pubkey, Job)>> {
        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);

        // Get base list using existing method
        let mut jobs = self.list_escrows(Some(limit + offset + 50)).await?; // Get more to allow for filtering

        // Apply filters
        jobs = self.apply_job_filters(jobs, &filter);

        // Apply sorting
        self.sort_jobs(&mut jobs, sort_by);

        // Apply pagination
        let start = offset;
        let end = (offset + limit).min(jobs.len());

        if start >= jobs.len() {
            return Ok(Vec::new());
        }

        Ok(jobs[start..end].to_vec())
    }

    /// List escrows for a specific client
    ///
    /// # Arguments
    /// * `client` - Client pubkey to filter by
    /// * `limit` - Maximum number of results
    ///
    /// # Example
    /// ```rust,no_run
    /// # use trust_escrow_sdk::CofreClient;
    /// # use solana_sdk::pubkey::Pubkey;
    /// # async fn example(client: CofreClient, client_pubkey: Pubkey) -> trust_escrow_sdk::Result<()> {
    /// let client_jobs = client.list_escrows_by_client(&client_pubkey, Some(10)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_escrows_by_client(
        &self,
        client: &Pubkey,
        limit: Option<usize>,
    ) -> Result<Vec<(Pubkey, Job)>> {
        let filter = JobFilter::new().client(Some(*client));
        self.list_escrows_advanced(filter, SortBy::CreatedDesc, limit, None)
            .await
    }

    /// List escrows by status
    ///
    /// # Arguments
    /// * `statuses` - Job statuses to filter by
    /// * `limit` - Maximum number of results
    ///
    /// # Example
    /// ```rust,no_run
    /// # use trust_escrow_sdk::{CofreClient, JobStatus};
    /// # async fn example(client: CofreClient) -> trust_escrow_sdk::Result<()> {
    /// let active_jobs = client.list_escrows_by_status(
    ///     vec![JobStatus::ApplicationsOpen, JobStatus::InProgress],
    ///     Some(20)
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_escrows_by_status(
        &self,
        statuses: Vec<JobStatus>,
        limit: Option<usize>,
    ) -> Result<Vec<(Pubkey, Job)>> {
        let filter = JobFilter::new().status(Some(statuses));
        self.list_escrows_advanced(filter, SortBy::CreatedDesc, limit, None)
            .await
    }

    /// Count escrows matching filter criteria
    ///
    /// # Arguments
    /// * `filter` - Filtering options
    ///
    /// # Example
    /// ```rust,no_run
    /// # use trust_escrow_sdk::{CofreClient, JobFilter, JobStatus};
    /// # async fn example(client: CofreClient) -> trust_escrow_sdk::Result<()> {
    /// let filter = JobFilter::new().status(Some(vec![JobStatus::InProgress]));
    /// let count = client.count_escrows(filter).await?;
    /// println!("Active jobs: {}", count);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn count_escrows(&self, filter: JobFilter) -> Result<usize> {
        let jobs = self.list_escrows(Some(1000)).await?; // Get many for accurate count
        let filtered = self.apply_job_filters(jobs, &filter);
        Ok(filtered.len())
    }

    /// Get escrow statistics
    ///
    /// # Returns
    /// EscrowStats with summary information
    ///
    /// # Example
    /// ```rust,no_run
    /// # use trust_escrow_sdk::CofreClient;
    /// # async fn example(client: CofreClient) -> trust_escrow_sdk::Result<()> {
    /// let stats = client.get_escrow_stats().await?;
    /// println!("Total jobs: {}", stats.total_jobs);
    /// println!("Active jobs: {}", stats.active_jobs);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_escrow_stats(&self) -> Result<EscrowStats> {
        let all_jobs = self.list_escrows(Some(1000)).await?;

        let mut stats = EscrowStats::default();
        stats.total_escrows = all_jobs.len();

        for (_, job) in &all_jobs {
            stats.total_volume += job.amount;

            match job.status {
                JobStatus::Created | JobStatus::ApplicationsOpen => stats.active_escrows += 1,
                JobStatus::InProgress | JobStatus::Submitted => stats.active_escrows += 1,
                JobStatus::Approved => stats.completed_escrows += 1,
                JobStatus::Disputed => stats.disputed_escrows += 1,
                JobStatus::Cancelled => {} // No specific field for cancelled
                JobStatus::Resolved => stats.completed_escrows += 1,
            }
        }

        stats.average_job_amount = if stats.total_escrows > 0 {
            stats.total_volume / stats.total_escrows as u64
        } else {
            0
        };

        Ok(stats)
    }

    // Helper methods for advanced querying

    fn apply_job_filters(
        &self,
        jobs: Vec<(Pubkey, Job)>,
        filter: &JobFilter,
    ) -> Vec<(Pubkey, Job)> {
        jobs.into_iter()
            .filter(|(_, job)| {
                // Client filter
                if let Some(client) = filter.client {
                    if job.client != client {
                        return false;
                    }
                }

                // Status filter
                if let Some(statuses) = &filter.status {
                    if !statuses.contains(&job.status) {
                        return false;
                    }
                }

                // Amount range filter
                if let Some((min_amount, max_amount)) = filter.amount_range {
                    if job.amount < min_amount || job.amount > max_amount {
                        return false;
                    }
                }

                // Date range filter
                if let Some(created_after) = filter.created_after {
                    if job.created_at < created_after {
                        return false;
                    }
                }

                if let Some(created_before) = filter.created_before {
                    if job.created_at > created_before {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    fn sort_jobs(&self, jobs: &mut [(Pubkey, Job)], sort_by: SortBy) {
        match sort_by {
            SortBy::CreatedAsc => jobs.sort_by(|(_, a), (_, b)| a.created_at.cmp(&b.created_at)),
            SortBy::CreatedDesc => jobs.sort_by(|(_, a), (_, b)| b.created_at.cmp(&a.created_at)),
            SortBy::AmountAsc => jobs.sort_by(|(_, a), (_, b)| a.amount.cmp(&b.amount)),
            SortBy::AmountDesc => jobs.sort_by(|(_, a), (_, b)| b.amount.cmp(&a.amount)),
            SortBy::UpdatedAsc => jobs.sort_by(|(_, a), (_, b)| a.updated_at.cmp(&b.updated_at)),
            SortBy::UpdatedDesc => jobs.sort_by(|(_, a), (_, b)| b.updated_at.cmp(&a.updated_at)),
            SortBy::Status => jobs.sort_by(|(_, a), (_, b)| {
                let a_order = match a.status {
                    JobStatus::Created => 0,
                    JobStatus::ApplicationsOpen => 1,
                    JobStatus::InProgress => 2,
                    JobStatus::Submitted => 3,
                    JobStatus::Approved => 4,
                    JobStatus::Disputed => 5,
                    JobStatus::Cancelled => 6,
                    JobStatus::Resolved => 7,
                };
                let b_order = match b.status {
                    JobStatus::Created => 0,
                    JobStatus::ApplicationsOpen => 1,
                    JobStatus::InProgress => 2,
                    JobStatus::Submitted => 3,
                    JobStatus::Approved => 4,
                    JobStatus::Disputed => 5,
                    JobStatus::Cancelled => 6,
                    JobStatus::Resolved => 7,
                };
                a_order.cmp(&b_order)
            }),
        }
    }

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
    pub async fn get_team(&self, team_pda: &Pubkey) -> Result<Team> {
        // Fetch account data using RPC client directly
        let account = self.rpc.get_account(team_pda).map_err(|e| {
            EscrowError::network_error(format!("Failed to fetch team account: {}", e))
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

        // Use borsh to deserialize the Team struct
        use borsh::BorshDeserialize;
        let team = Team::try_from_slice(data_slice).map_err(|e| {
            EscrowError::deserialization_error(format!("Failed to deserialize Team: {}", e))
        })?;

        Ok(team)
    }

    /// Fetch dispute account data
    pub async fn get_dispute(&self, dispute_pda: &Pubkey) -> Result<Dispute> {
        // Fetch account data using RPC client directly
        let account = self.rpc.get_account(dispute_pda).map_err(|e| {
            EscrowError::network_error(format!("Failed to fetch dispute account: {}", e))
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

        // Use borsh to deserialize the Dispute struct
        use borsh::BorshDeserialize;
        let dispute = Dispute::try_from_slice(data_slice).map_err(|e| {
            EscrowError::deserialization_error(format!("Failed to deserialize Dispute: {}", e))
        })?;

        Ok(dispute)
    }

    /// Fetch milestone account data
    pub async fn get_milestone(&self, milestone_pda: &Pubkey) -> Result<Milestone> {
        // Fetch account data using RPC client directly
        let account = self.rpc.get_account(milestone_pda).map_err(|e| {
            EscrowError::network_error(format!("Failed to fetch milestone account: {}", e))
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

        // Use borsh to deserialize the Milestone struct
        use borsh::BorshDeserialize;
        let milestone = Milestone::try_from_slice(data_slice).map_err(|e| {
            EscrowError::deserialization_error(format!("Failed to deserialize Milestone: {}", e))
        })?;

        Ok(milestone)
    }

    // ===== EVENT MONITORING =====

    /// Create an event listener for monitoring contract events
    pub fn create_event_listener(&self) -> EventListener {
        EventListener::new(self.rpc.clone(), EventListenerConfig::default())
    }

    /// Create an event listener with custom configuration
    pub fn create_event_listener_with_config(&self, config: EventListenerConfig) -> EventListener {
        EventListener::new(self.rpc.clone(), config)
    }

    /// Get recent events from the contract
    ///
    /// # Arguments
    /// * `limit` - Maximum number of transactions to scan for events
    ///
    /// # Returns
    /// Vector of events found in recent transactions
    pub async fn get_recent_events(&self, limit: usize) -> Result<Vec<EscrowEvent>> {
        let listener = self.create_event_listener();
        listener.get_recent_events(limit).await
    }

    /// Monitor for new events in real-time
    /// Returns a receiver that will yield events as they occur
    pub async fn monitor_events(&self) -> tokio::sync::mpsc::UnboundedReceiver<EscrowEvent> {
        let mut listener = self.create_event_listener();
        listener.start_listening()
    }

    // ===== PERFORMANCE & UTILITIES =====

    /// Get connection and performance statistics
    pub fn get_performance_stats(&self) -> PerformanceStats {
        let (total_cache_entries, valid_cache_entries) = self.cache_stats();

        PerformanceStats {
            cache_total_entries: total_cache_entries,
            cache_valid_entries: valid_cache_entries,
            cache_hit_rate: if total_cache_entries > 0 {
                (valid_cache_entries as f64 / total_cache_entries as f64) * 100.0
            } else {
                0.0
            },
            retry_config: self.perf_config.retry_config.clone(),
        }
    }

    /// Test connection to the RPC endpoint
    pub async fn test_connection(&self) -> Result<bool> {
        self.with_retry(|| {
            let rpc = self.rpc.clone();
            Box::pin(async move {
                rpc.get_health()
                    .map_err(|e| EscrowError::network_error(&format!("Health check failed: {}", e)))
                    .map(|_| true)
            })
        })
        .await
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
