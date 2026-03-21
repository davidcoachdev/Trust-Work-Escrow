//! Create user instruction - Create a new user account

use crate::state::User;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CreateUser<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = User::INIT_SPACE + 8,
        seeds = [b"user", payer.key().as_ref()],
        bump
    )]
    pub user: Account<'info, User>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CreateUser>, username: String) -> Result<()> {
    // Validations
    require!(
        !username.is_empty(),
        crate::ErrorCode::EmptyUsername
    )?;
    require!(
        username.len() <= crate::state::MAX_USERNAME_LENGTH,
        crate::ErrorCode::UsernameTooLong
    )?;

    // Initialize user account
    let user = &mut ctx.accounts.user;
    user.wallet_principal = ctx.accounts.payer.key();
    user.wallets_asociadas = vec![ctx.accounts.payer.key()];
    user.active_wallet = ctx.accounts.payer.key();
    user.username = username.clone();
    user.bio = None;
    user.created_at = Clock::get()?.unix_timestamp;
    user.bump = ctx.bumps.user;

    msg!("User created: {} with username: {}", user.key(), username);

    Ok(())
}