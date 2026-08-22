//! Type definitions extending Anchor-generated account types
//!
//! This module provides enhanced types that extend the auto-generated Anchor client types
//! with additional business logic validation and utility methods.

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::time::Duration;

use crate::error::{EscrowError, Result};
use crate::{MAX_ARBITERS, MAX_MILESTONES, MAX_WALLETS, MIN_JOB_AMOUNT};

/// Configuration account for the Trust Escrow program
/// Extended with validation and utility methods
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Config {
    /// The admin authority who can pause/unpause and manage treasury
    pub admin: Pubkey,
    /// Whether the program is currently paused
    pub is_paused: bool,
    /// Treasury address where fees are collected
    pub treasury: Pubkey,
    /// Fee percentage for job transactions (0-100)
    pub fee_percentage: u8,
    /// Program bump seed
    pub bump: u8,
}

impl Config {
    /// Validate config parameters
    pub fn validate(&self) -> Result<()> {
        if self.fee_percentage > 100 {
            return Err(EscrowError::invalid_parameter(
                "Fee percentage cannot exceed 100%",
            ));
        }

        if self.treasury == Pubkey::default() {
            return Err(EscrowError::invalid_parameter(
                "Treasury address cannot be default pubkey",
            ));
        }

        Ok(())
    }

    /// Calculate fee amount for given job amount
    pub fn calculate_fee(&self, amount: u64) -> u64 {
        (amount * self.fee_percentage as u64) / 100
    }

    /// Check if program operations are allowed
    pub fn ensure_not_paused(&self) -> Result<()> {
        if self.is_paused {
            return Err(EscrowError::not_permitted("Program is currently paused"));
        }
        Ok(())
    }
}

/// User account with multi-wallet support
/// Extended with validation methods
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct User {
    /// User's chosen username (max 32 chars)
    pub username: String,
    /// User's bio/description (max 500 chars)  
    pub bio: String,
    /// List of associated wallet addresses (max 5)
    pub wallets: Vec<Pubkey>,
    /// Currently active wallet for transactions
    pub active_wallet: Pubkey,
    /// Account creation timestamp
    pub created_at: i64,
    /// Last update timestamp
    pub updated_at: i64,
    /// Account bump seed
    pub bump: u8,
}

impl User {
    /// Validate user data constraints
    pub fn validate(&self) -> Result<()> {
        if self.username.len() > 32 {
            return Err(EscrowError::invalid_parameter(
                "Username cannot exceed 32 characters",
            ));
        }

        if self.username.trim().is_empty() {
            return Err(EscrowError::invalid_parameter("Username cannot be empty"));
        }

        if self.bio.len() > 500 {
            return Err(EscrowError::invalid_parameter(
                "Bio cannot exceed 500 characters",
            ));
        }

        if self.wallets.len() > MAX_WALLETS {
            return Err(EscrowError::invalid_parameter(format!(
                "Cannot have more than {} wallets",
                MAX_WALLETS
            )));
        }

        if !self.wallets.contains(&self.active_wallet) {
            return Err(EscrowError::invalid_parameter(
                "Active wallet must be in user's wallet list",
            ));
        }

        Ok(())
    }

    /// Check if user can add another wallet
    pub fn can_add_wallet(&self) -> bool {
        self.wallets.len() < MAX_WALLETS
    }

    /// Check if wallet is associated with this user
    pub fn has_wallet(&self, wallet: &Pubkey) -> bool {
        self.wallets.contains(wallet)
    }
}

/// Job states matching the smart contract JobStatus enum
#[derive(
    Serialize, Deserialize, BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq, Eq,
)]
pub enum JobStatus {
    Created,
    ApplicationsOpen,
    InProgress,
    Submitted,
    Approved,
    Disputed,
    Resolved,
    Cancelled,
}

impl JobStatus {
    /// Check if job accepts new applications
    pub fn accepts_applications(&self) -> bool {
        matches!(self, JobStatus::ApplicationsOpen)
    }

    /// Check if job can be cancelled
    pub fn can_be_cancelled(&self) -> bool {
        matches!(self, JobStatus::Created | JobStatus::ApplicationsOpen)
    }

    /// Check if work can be submitted
    pub fn can_submit_work(&self) -> bool {
        matches!(self, JobStatus::InProgress)
    }

    /// Check if work can be approved/rejected
    pub fn can_review_work(&self) -> bool {
        matches!(self, JobStatus::Submitted)
    }
}

/// Application status for job applications
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationStatus {
    Pending,
    Accepted,
    Rejected,
}

/// Job posting with enhanced validation
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct Job {
    /// Unique job ID for this client
    pub job_id: u64,
    /// Client who posted the job
    pub client: Pubkey,
    /// Assigned freelancer (None until accepted)
    pub freelancer: Option<Pubkey>,
    /// Job title
    pub title: String,
    /// Job description
    pub description: String,
    /// Job amount in lamports
    pub amount: u64,
    /// Current job status
    pub status: JobStatus,
    /// Creation timestamp
    pub created_at: i64,
    /// Last update timestamp
    pub updated_at: i64,
    /// Account bump seed
    pub bump: u8,
}

impl Job {
    /// Validate job parameters
    pub fn validate(&self) -> Result<()> {
        if self.title.trim().is_empty() {
            return Err(EscrowError::invalid_parameter("Job title cannot be empty"));
        }

        if self.title.len() > 100 {
            return Err(EscrowError::invalid_parameter(
                "Job title cannot exceed 100 characters",
            ));
        }

        if self.description.len() > 2000 {
            return Err(EscrowError::invalid_parameter(
                "Job description cannot exceed 2000 characters",
            ));
        }

        if self.amount < MIN_JOB_AMOUNT {
            return Err(EscrowError::invalid_parameter(format!(
                "Job amount must be at least {} lamports",
                MIN_JOB_AMOUNT
            )));
        }

        Ok(())
    }

    /// Check if user can apply to this job
    pub fn can_apply(&self, applicant: &Pubkey) -> Result<()> {
        if !self.status.accepts_applications() {
            return Err(EscrowError::not_permitted(
                "Job is not accepting applications",
            ));
        }

        if &self.client == applicant {
            return Err(EscrowError::not_permitted("Client cannot apply to own job"));
        }

        Ok(())
    }

    /// Check if user can perform client actions
    pub fn ensure_is_client(&self, user: &Pubkey) -> Result<()> {
        if &self.client != user {
            return Err(EscrowError::not_permitted(
                "Only job client can perform this action",
            ));
        }
        Ok(())
    }

    /// Check if user can perform freelancer actions
    pub fn ensure_is_freelancer(&self, user: &Pubkey) -> Result<()> {
        match &self.freelancer {
            Some(freelancer) if freelancer == user => Ok(()),
            Some(_) => Err(EscrowError::not_permitted(
                "Only assigned freelancer can perform this action",
            )),
            None => Err(EscrowError::not_permitted(
                "No freelancer assigned to this job",
            )),
        }
    }
}

/// Team member role (matches IDL)
#[derive(
    Serialize, Deserialize, BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq, Eq,
)]
pub enum MemberRole {
    Owner,
    ProjectManager,
    Contributor,
}

/// Team member (matches IDL)
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct Member {
    pub user: Pubkey,
    pub role: MemberRole,
    pub joined_at: i64,
}

/// Team account with member management (matches IDL)
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct Team {
    /// Team owner
    pub owner: Pubkey,
    /// Team members with roles
    pub members: Vec<Member>,
    /// Team name
    pub name: String,
    /// Team description
    pub description: String,
    /// Account bump seed
    pub bump: u8,
    /// Creation timestamp
    pub created_at: i64,
    /// Last update timestamp
    pub updated_at: i64,
}

impl Team {
    /// Validate team data
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(EscrowError::invalid_parameter("Team name cannot be empty"));
        }

        if self.name.len() > 50 {
            return Err(EscrowError::invalid_parameter(
                "Team name cannot exceed 50 characters",
            ));
        }

        if self.description.len() > 500 {
            return Err(EscrowError::invalid_parameter(
                "Team description cannot exceed 500 characters",
            ));
        }

        Ok(())
    }

    /// Check if user is team owner
    pub fn is_owner(&self, user: &Pubkey) -> bool {
        &self.owner == user
    }

    /// Check if user is team member
    pub fn is_member(&self, user: &Pubkey) -> bool {
        self.members.iter().any(|member| &member.user == user)
    }

    /// Get member role
    pub fn get_member_role(&self, user: &Pubkey) -> Option<MemberRole> {
        self.members
            .iter()
            .find(|member| &member.user == user)
            .map(|member| member.role)
    }
}

/// Dispute account for managing job disputes
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Dispute {
    /// The job this dispute is related to
    pub job: Pubkey,
    /// Who raised the dispute
    pub raised_by: Pubkey,
    /// Assigned arbiter (if any)
    pub arbiter: Option<Pubkey>,
    /// Current dispute status
    pub status: DisputeStatus,
    /// Evidence submitted
    pub evidence: Vec<Evidence>,
    /// Reason for the dispute
    pub reason: String,
    /// When the dispute was created
    pub created_at: i64,
    /// When the dispute was resolved (if any)
    pub resolved_at: Option<i64>,
    /// Bump seed for PDA derivation
    pub bump: u8,
}

/// Status of a dispute
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum DisputeStatus {
    Open,
    EvidenceSubmitted,
    ArbiterAssigned,
    Resolved,
    Expired,
}

/// Evidence submitted for a dispute
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Evidence {
    /// Who submitted the evidence
    pub submitter: Pubkey,
    /// Evidence content
    pub content: String,
    /// When submitted
    pub submitted_at: i64,
}

/// Milestone account for milestone-based payments
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Milestone {
    /// The job this milestone belongs to
    pub job: Pubkey,
    /// Milestone title
    pub title: String,
    /// Milestone description
    pub description: String,
    /// Payment amount for this milestone
    pub amount: u64,
    /// Due date (optional)
    pub due_date: Option<i64>,
    /// Current status
    pub status: MilestoneStatus,
    /// Milestone index
    pub index: u8,
    /// When work was submitted
    pub submitted_at: Option<i64>,
    /// When milestone was approved
    pub approved_at: Option<i64>,
    /// Work URL submitted by freelancer
    pub work_url: Option<String>,
    /// Rejection reason (if rejected)
    pub rejection_reason: Option<String>,
    /// When created
    pub created_at: i64,
    /// Bump seed for PDA derivation
    pub bump: u8,
}

/// Status of a milestone
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum MilestoneStatus {
    Pending,
    Submitted,
    Approved,
    Rejected,
}

/// Milestone specification for batch operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneSpec {
    pub title: String,
    pub amount: u64,
    pub description: Option<String>,
}

/// Advanced job filtering capabilities
#[derive(Debug, Clone)]
pub struct JobFilter {
    pub client: Option<Pubkey>,
    pub status: Option<Vec<JobStatus>>,
    pub amount_range: Option<(u64, u64)>,
    pub created_after: Option<i64>,
    pub created_before: Option<i64>,
}

impl JobFilter {
    pub fn new() -> Self {
        Self {
            client: None,
            status: None,
            amount_range: None,
            created_after: None,
            created_before: None,
        }
    }

    pub fn client(mut self, client: Option<Pubkey>) -> Self {
        self.client = client;
        self
    }

    pub fn status(mut self, status: Option<Vec<JobStatus>>) -> Self {
        self.status = status;
        self
    }

    pub fn amount_range(mut self, min: u64, max: u64) -> Self {
        self.amount_range = Some((min, max));
        self
    }
}

/// Sorting options for job queries
#[derive(Debug, Clone)]
pub enum SortBy {
    CreatedAsc,
    CreatedDesc,
    UpdatedAsc,
    UpdatedDesc,
    AmountAsc,
    AmountDesc,
    Status,
}

/// Escrow statistics summary
#[derive(Debug, Clone, Default)]
pub struct EscrowStats {
    pub total_escrows: usize,
    pub active_escrows: usize,
    pub completed_escrows: usize,
    pub disputed_escrows: usize,
    pub total_volume: u64,
    pub average_job_amount: u64,
}

/// Performance statistics for monitoring SDK operation
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    /// Total entries in the cache
    pub cache_total_entries: usize,
    /// Number of valid (non-expired) cache entries
    pub cache_valid_entries: usize,
    /// Cache hit rate as a percentage
    pub cache_hit_rate: f64,
    /// Current retry configuration
    pub retry_config: RetryConfig,
}

/// Retry configuration for failed operations
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: usize,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
}

impl PerformanceStats {
    /// Get cache efficiency percentage (valid entries / total entries)
    pub fn cache_efficiency(&self) -> f64 {
        if self.cache_total_entries == 0 {
            100.0
        } else {
            (self.cache_valid_entries as f64 / self.cache_total_entries as f64) * 100.0
        }
    }

    /// Check if cache performance is healthy
    pub fn is_cache_healthy(&self) -> bool {
        self.cache_efficiency() > 70.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let mut config = Config {
            admin: Pubkey::new_unique(),
            is_paused: false,
            treasury: Pubkey::new_unique(),
            fee_percentage: 5,
            bump: 255,
        };

        assert!(config.validate().is_ok());

        config.fee_percentage = 101;
        assert!(config.validate().is_err());

        config.fee_percentage = 5;
        config.treasury = Pubkey::default();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_job_status_transitions() {
        assert!(JobStatus::ApplicationsOpen.accepts_applications());
        assert!(!JobStatus::InProgress.accepts_applications());

        assert!(JobStatus::Created.can_be_cancelled());
        assert!(!JobStatus::InProgress.can_be_cancelled());

        assert!(JobStatus::InProgress.can_submit_work());
        assert!(!JobStatus::Created.can_submit_work());
    }

    #[test]
    fn test_user_wallet_management() {
        let wallet1 = Pubkey::new_unique();
        let wallet2 = Pubkey::new_unique();

        let user = User {
            username: "testuser".to_string(),
            bio: "Test bio".to_string(),
            wallets: vec![wallet1, wallet2],
            active_wallet: wallet1,
            created_at: 1000,
            updated_at: 1000,
            bump: 255,
        };

        assert!(user.validate().is_ok());
        assert!(user.has_wallet(&wallet1));
        assert!(user.has_wallet(&wallet2));
        assert!(!user.has_wallet(&Pubkey::new_unique()));
    }
}
