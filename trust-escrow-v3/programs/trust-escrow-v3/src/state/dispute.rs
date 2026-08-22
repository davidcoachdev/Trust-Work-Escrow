use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum DisputeStatus {
    Open,
    Active,
    EvidenceSubmitted,
    ArbiterAssigned,
    Resolved,
    Expired,
}

impl anchor_lang::Space for DisputeStatus {
    const INIT_SPACE: usize = 1;
}

#[account]
#[derive(InitSpace)]
pub struct Dispute {
    pub job: Pubkey,
    pub raised_by: Pubkey,
    pub arbiter: Option<Pubkey>,
    pub status: DisputeStatus,
    pub evidence_count: u8,
    pub evidence_cleanup_cursor: u8,
    pub deadline: i64,
    pub client_payout_percent: u8,
    pub freelancer_payout_percent: u8,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum SupportTicketStatus {
    Open,
    Resolved,
}

#[account]
#[derive(InitSpace)]
pub struct SupportTicket {
    pub job: Pubkey,
    pub opened_by: Pubkey,
    pub status: SupportTicketStatus,
    pub bump: u8,
}

