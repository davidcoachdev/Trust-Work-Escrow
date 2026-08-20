use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum MilestoneStatus {
    Pending,
    Submitted,
    Approved,
    Rejected,
}

impl anchor_lang::Space for MilestoneStatus {
    const INIT_SPACE: usize = 1;
}

#[account]
#[derive(InitSpace)]
pub struct Milestone {
    pub job: Pubkey,
    pub amount: u64,
    pub status: MilestoneStatus,
    pub index: u8,
    pub bump: u8,
}

