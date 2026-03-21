//! Pause instruction

use crate::state::Config;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Pause<'info> {
    #[account(mut, seeds = [Config::SEED], bump = config.bump)]
    pub config: Account<'info, Config>,
    pub authority: Signer<'info>,
}

pub fn handler(ctx: Context<Pause>) -> Result<()> {
    let config = &mut ctx.accounts.config;
    
    // Verify authority is admin
    require!(
        ctx.accounts.authority.key() == config.admin,
        crate::ErrorCode::NotAdmin
    );

    config.paused = true;
    msg!("Program paused");

    Ok(())
}