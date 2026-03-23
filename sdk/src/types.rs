//! Type definitions extending Anchor-generated account types
//!
//! This module provides enhanced types that extend the auto-generated Anchor client types
//! with additional business logic validation and utility methods.

use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

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
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

/// Team member role
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemberRole {
    Owner,
    Admin,
    Member,
}

/// Team account with member management
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Team {
    /// Team owner
    pub owner: Pubkey,
    /// Team name
    pub name: String,
    /// Team description
    pub description: String,
    /// Team members with roles
    pub members: Vec<(Pubkey, MemberRole)>,
    /// Creation timestamp
    pub created_at: i64,
    /// Account bump seed
    pub bump: u8,
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
        self.members.iter().any(|(member, _)| member == user)
    }

    /// Get member role
    pub fn get_member_role(&self, user: &Pubkey) -> Option<MemberRole> {
        self.members
            .iter()
            .find(|(member, _)| member == user)
            .map(|(_, role)| *role)
    }
}

/// Dispute status
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisputeStatus {
    Open,
    UnderReview,
    Resolved,
}

/// Milestone status
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MilestoneStatus {
    Created,
    Submitted,
    Approved,
    Rejected,
}

/// Dispute account
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Dispute {
    /// Associated job
    pub job: Pubkey,
    /// User who raised the dispute
    pub raised_by: Pubkey,
    /// Evidence submitted
    pub evidence: String,
    /// Assigned arbiter
    pub arbiter: Option<Pubkey>,
    /// Dispute status
    pub status: DisputeStatus,
    /// Resolution percentage for client (0-100)
    pub client_percentage: Option<u8>,
    /// Creation timestamp
    pub created_at: i64,
    /// Account bump seed
    pub bump: u8,
}

impl Dispute {
    /// Validate dispute data
    pub fn validate(&self) -> Result<()> {
        if self.evidence.len() > 2048 {
            return Err(EscrowError::invalid_parameter(
                "Dispute evidence cannot exceed 2048 characters",
            ));
        }

        if let Some(percentage) = self.client_percentage {
            if percentage > 100 {
                return Err(EscrowError::invalid_parameter(
                    "Client percentage cannot exceed 100%",
                ));
            }
        }

        Ok(())
    }
}

/// Milestone account
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Milestone {
    /// Associated job
    pub job: Pubkey,
    /// Milestone index
    pub index: u8,
    /// Milestone title
    pub title: String,
    /// Milestone description
    pub description: String,
    /// Milestone amount
    pub amount: u64,
    /// Milestone status
    pub status: MilestoneStatus,
    /// Creation timestamp
    pub created_at: i64,
    /// Submission timestamp
    pub submitted_at: Option<i64>,
    /// Account bump seed
    pub bump: u8,
}

impl Milestone {
    /// Validate milestone data
    pub fn validate(&self, job_amount: u64) -> Result<()> {
        if self.title.trim().is_empty() {
            return Err(EscrowError::invalid_parameter(
                "Milestone title cannot be empty",
            ));
        }

        if self.title.len() > 100 {
            return Err(EscrowError::invalid_parameter(
                "Milestone title cannot exceed 100 characters",
            ));
        }

        if self.description.len() > 1000 {
            return Err(EscrowError::invalid_parameter(
                "Milestone description cannot exceed 1000 characters",
            ));
        }

        if self.amount > job_amount {
            return Err(EscrowError::invalid_parameter(
                "Milestone amount cannot exceed job amount",
            ));
        }

        if self.index as usize >= MAX_MILESTONES {
            return Err(EscrowError::invalid_parameter(format!(
                "Milestone index cannot exceed {}",
                MAX_MILESTONES - 1
            )));
        }

        Ok(())
    }
}

/// Arbiter pool for dispute resolution
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ArbiterPool {
    /// List of authorized arbiters
    pub arbiters: Vec<Pubkey>,
    /// Creation timestamp
    pub created_at: i64,
    /// Account bump seed
    pub bump: u8,
}

impl ArbiterPool {
    /// Validate arbiter pool
    pub fn validate(&self) -> Result<()> {
        if self.arbiters.len() > MAX_ARBITERS {
            return Err(EscrowError::invalid_parameter(format!(
                "Cannot have more than {} arbiters",
                MAX_ARBITERS
            )));
        }

        Ok(())
    }

    /// Check if user is an authorized arbiter
    pub fn is_arbiter(&self, user: &Pubkey) -> bool {
        self.arbiters.contains(user)
    }

    /// Check if can add another arbiter
    pub fn can_add_arbiter(&self) -> bool {
        self.arbiters.len() < MAX_ARBITERS
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
