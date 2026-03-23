//! Config account - Global configuration

use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub admin: Pubkey,
    pub treasury: Pubkey,
    #[max_len(5)]
    pub multisig_owners: Vec<Pubkey>,
    pub multisig_threshold: u8,
    pub fee_percent: u8,
    pub paused: bool,
    pub bump: u8,
}

impl Config {
    pub const SEED: &'static [u8] = b"config";

    pub fn seeds(&self) -> Vec<u8> {
        let mut seed = Self::SEED.to_vec();
        seed.push(self.bump);
        seed
    }
}
