//! Unpause instruction

use crate::state::Config;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Unpause<'info> {
    #[account(mut, seeds = [Config::SEED], bump = config.bump)]
    pub config: Account<'info, Config>,
    pub authority: Signer<'info>,
}

pub fn handler(ctx: Context<Unpause>) -> Result<()> {
    let config = &mut ctx.accounts.config;
    
    require!(
        ctx.accounts.authority.key() == config.admin,
        crate::ErrorCode::NotAdmin
    );

    config.paused = false;
    msg!("Program unpaused");

    Ok(())
}