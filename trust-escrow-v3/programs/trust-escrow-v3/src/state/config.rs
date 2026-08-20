use anchor_lang::prelude::*;
use crate::MAX_ARBITERS;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub authority: Pubkey,
    pub advisor: Pubkey,
    pub treasury: Pubkey,
    pub arbitration_treasury: Pubkey,
    pub fee_bps: u16,
    pub paused: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct ArbiterPool {
    pub authority: Pubkey,
    #[max_len(MAX_ARBITERS)]
    pub arbiters: Vec<Pubkey>,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct ArbitrationEscrow {
    pub job: Pubkey,
    pub client_bond: u64,
    pub freelancer_bond: u64,
    pub bump: u8,
}

