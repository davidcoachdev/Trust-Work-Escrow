//! ArbiterPool account - Registry of authorized arbiters

use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct ArbiterPool {
    /// Authority (admin who can add/remove arbiters)
    pub authority: Pubkey,
    /// List of registered arbiters (max 50)
    #[max_len(MAX_ARBITERS)]
    pub arbiters: Vec<Pubkey>,
    /// PDA bump
    pub bump: u8,
}

impl ArbiterPool {
    pub const SEED: &'static [u8] = b"arbiter_pool";
}

use crate::MAX_ARBITERS;