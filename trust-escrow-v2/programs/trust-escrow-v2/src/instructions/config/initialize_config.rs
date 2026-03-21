//! Initialize Config instruction

use crate::state::{Config, MAX_MULTISIG_OWNERS};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = Config::INIT_SPACE + 8,
        seeds = [Config::SEED],
        bump
    )]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializeConfig>,
    multisig_owners: Vec<Pubkey>,
    multisig_threshold: u8,
    treasury: Pubkey,
    fee_percent: u8,
) -> Result<()> {
    require!(
        !multisig_owners.is_empty(),
        crate::ErrorCode::InvalidMultisigThreshold
    );
    require!(
        multisig_owners.len() <= MAX_MULTISIG_OWNERS,
        crate::ErrorCode::MaxMultisigOwnersReached
    );
    require!(
        multisig_threshold >= 1 && multisig_threshold <= multisig_owners.len() as u8,
        crate::ErrorCode::InvalidMultisigThreshold
    );
    require!(
        fee_percent <= 100,
        crate::ErrorCode::InvalidFeePercentage
    );

    let config = &mut ctx.accounts.config;
    config.admin = multisig_owners[0];
    config.treasury = treasury;
    config.multisig_owners = multisig_owners;
    config.multisig_threshold = multisig_threshold;
    config.fee_percent = fee_percent;
    config.paused = false;
    config.bump = ctx.bumps.config;

    msg!(
        "Config initialized: admin={}, treasury={}, fee={}%",
        config.admin,
        config.treasury,
        config.fee_percent
    );

    Ok(())
}