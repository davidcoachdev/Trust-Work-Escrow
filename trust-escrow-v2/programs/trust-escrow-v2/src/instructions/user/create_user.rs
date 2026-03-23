//! Create User instruction

use crate::state::{User, MAX_BIO_LENGTH, MAX_USERNAME_LENGTH};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CreateUser<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = User::INIT_SPACE + 8,
        seeds = [b"user", authority.key().as_ref()],
        bump
    )]
    pub user: Account<'info, User>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CreateUser>, username: String) -> Result<()> {
    require!(!username.is_empty(), crate::ErrorCode::EmptyUsername);
    require!(
        username.len() <= MAX_USERNAME_LENGTH,
        crate::ErrorCode::UsernameTooLong
    );

    let user = &mut ctx.accounts.user;
    user.wallet_principal = ctx.accounts.authority.key();
    user.wallets_asociadas = Vec::new();
    user.active_wallet = ctx.accounts.authority.key();
    user.username = username;
    user.bio = None;
    user.created_at = Clock::get()?.unix_timestamp;
    user.bump = ctx.bumps.user;

    msg!("User created: {}", user.username);
    Ok(())
}
