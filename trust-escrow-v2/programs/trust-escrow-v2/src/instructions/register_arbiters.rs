//! Register arbiters instruction - Add arbiters to the pool

use crate::state::{ArbiterPool, Config, MAX_ARBITERS};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct RegisterArbiters<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init if missing,
        payer = authority,
        space = ArbiterPool::INIT_SPACE + 8,
        seeds = [ArbiterPool::SEED],
        bump,
        seeds::program = crate::ID
    )]
    pub arbiter_pool: Account<'info, ArbiterPool>,
    #[account(seeds = [Config::SEED], bump = config.bump)]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<RegisterArbiters>, arbiters: Vec<Pubkey>) -> Result<()> {
    let config = &ctx.accounts.config;

    require!(
        ctx.accounts.authority.key() == config.admin,
        crate::ErrorCode::NotAdmin
    );

    // If account doesn't exist, initialize it
    if ctx.accounts.arbiter_pool.to_account_info().data_is_empty() {
        ctx.accounts.arbiter_pool.authority = config.admin;
        ctx.accounts.arbiter_pool.arbiters = Vec::new();
        ctx.accounts.arbiter_pool.bump = ctx.bumps.arbiter_pool;
    }

    // Add new arbiters
    for arbiter in arbiters.iter() {
        if !ctx.accounts.arbiter_pool.arbiters.contains(arbiter) {
            require!(
                ctx.accounts.arbiter_pool.arbiters.len() < MAX_ARBITERS,
                crate::ErrorCode::MaxArbitersReached
            );
            ctx.accounts.arbiter_pool.arbiters.push(*arbiter);
        }
    }

    msg!("Registered arbiters: {} total", ctx.accounts.arbiter_pool.arbiters.len());

    Ok(())
}
