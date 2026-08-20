use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ApplicationStatus {
    Pending,
    Accepted,
    Rejected,
    Withdrawn,
}

impl anchor_lang::Space for ApplicationStatus {
    const INIT_SPACE: usize = 1;
}

#[account]
#[derive(InitSpace)]
pub struct Application {
    pub job: Pubkey,
    pub index: u8,
    pub applicant: Pubkey,
    pub proposal_hash: [u8; 32],
    pub status: ApplicationStatus,
    pub bump: u8,
}

