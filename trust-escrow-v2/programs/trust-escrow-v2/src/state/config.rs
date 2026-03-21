//! Config account - Global configuration

use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Config {
    /// Admin (first owner of multisig)
    pub admin: Pubkey,
    /// Treasury wallet for fees
    pub treasury: Pubkey,
    /// Multisig owners (max 5)
    #[max_len(MAX_MULTISIG_OWNERS)]
    pub multisig_owners: Vec<Pubkey>,
    /// Required signatures for multisig (2 default)
    pub multisig_threshold: u8,
    /// Fee percentage (5 = 5%)
    pub fee_percent: u8,
    /// Program paused flag
    pub paused: bool,
    /// PDA bump
    pub bump: u8,
}

impl Config {
    pub const SEED: &'static [u8] = b"config";

    pub fn seeds(&self) -> [&[u8]; 2] {
        [Self::SEED, &[self.bump]]
    }
}