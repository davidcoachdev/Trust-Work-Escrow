use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Config {
    /// Admin wallet that can pause/update config
    pub admin: Pubkey,
    /// Treasury wallet where fees are collected
    pub treasury_wallet: Pubkey,
    /// Treasurer wallet that can withdraw from treasury
    pub treasurer: Pubkey,
    /// Fee percentage charged to client on publish (in basis points, 500 = 5%)
    pub entry_fee_bps: u16,
    /// Fee percentage charged to freelancer on payout (in basis points)
    pub exit_fee_bps: u16,
    /// Dispute stake percentage (in basis points, 250 = 2.5%)
    pub dispute_stake_bps: u16,
    /// Maximum days for a job deadline
    pub max_job_duration_days: u32,
    /// Auto-approve days after submit (7 days)
    pub auto_approve_days: u8,
    /// Pause state
    pub paused: bool,
    /// Bump for PDA
    pub bump: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            admin: Pubkey::default(),
            treasury_wallet: Pubkey::default(),
            treasurer: Pubkey::default(),
            entry_fee_bps: 500,     // 5%
            exit_fee_bps: 500,      // 5%
            dispute_stake_bps: 250, // 2.5%
            max_job_duration_days: 90,
            auto_approve_days: 7,
            paused: false,
            bump: 0,
        }
    }
}
