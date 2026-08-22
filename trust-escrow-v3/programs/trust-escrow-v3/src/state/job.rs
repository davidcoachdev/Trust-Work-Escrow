use anchor_lang::prelude::*;
use crate::MAX_APPLICATIONS;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum JobStatus {
    Created,
    Funded,
    InProgress,
    Submitted,
    Released,
    Disputed,
    Resolved,
    Cancelled,
}

impl anchor_lang::Space for JobStatus {
    const INIT_SPACE: usize = 1;
}

#[account]
#[derive(InitSpace)]
pub struct Job {
    pub client: Pubkey,
    pub freelancer: Option<Pubkey>,
    pub amount: u64,
    pub fee_amount: u64,
    pub status: JobStatus,
    pub paused: bool,
    pub paused_at: i64,
    pub deadline: i64,
    pub submitted_at: Option<i64>,
    pub milestones_total: u8,
    pub milestones_approved: u8,
    pub milestones_amount_total: u64,
    // Bounded applicant list. Stored as a `Vec` with a fixed maximum capacity
    // (MAX_APPLICATIONS). Anchor reserves the full capacity inside the account's
    // allocated space, so no runtime heap allocation occurs, while avoiding the
    // large stack frames that a fixed `[Pubkey; N]` array produced (the cause of
    // the previous SBF stack overflow at build time).
    #[max_len(MAX_APPLICATIONS)]
    pub applicants: Vec<Pubkey>,
    pub bump: u8,
}

