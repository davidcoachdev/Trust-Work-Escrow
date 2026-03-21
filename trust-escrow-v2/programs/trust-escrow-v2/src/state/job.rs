//! Job account - Work/escrow state

use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Job {
    /// Client (from User.active_wallet)
    pub client: Pubkey,
    /// Freelancer (if accepted)
    pub freelancer: Option<Pubkey>,
    /// Assigned arbiter
    pub arbiter: Option<Pubkey>,
    /// Total amount (lamports)
    pub amount: u64,
    /// Fee percentage (from config)
    pub fee_percent: u8,
    /// Fee amount calculated
    pub fee_amount: u64,
    /// Job status
    pub status: JobStatus,
    /// Title (max 100 chars)
    #[max_len(MAX_TITLE_LENGTH)]
    pub title: String,
    /// Description (max 500 chars)
    #[max_len(MAX_DESCRIPTION_LENGTH)]
    pub description: String,
    /// Deadline timestamp
    pub deadline: i64,
    /// Creation timestamp
    pub created_at: i64,
    /// Last update timestamp
    pub updated_at: i64,
    /// Dispute reason (max 200 chars)
    #[max_len(MAX_DISPUTE_REASON_LENGTH)]
    pub dispute_reason: String,
    /// PDA bump
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum JobStatus {
    /// Created, waiting for deposit
    Created,
    /// Funds deposited, waiting for freelancer
    Funded,
    /// In progress
    InProgress,
    /// Work submitted
    Submitted,
    /// Completed - funds released
    Released,
    /// Disputed
    Disputed,
    /// Resolved by arbiter
    Resolved,
    /// Cancelled - funds refunded
    Cancelled,
}

impl Job {
    pub const SEED: &'static [u8] = b"job";
}