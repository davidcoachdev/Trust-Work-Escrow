//! Add wallet instruction - Add a secondary wallet to user account

use crate::state::{User, MAX_WALLETS};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct AddWallet<'info> {
    #[account(mut, seeds = [b"user", authority.key().as_ref()], bump = user.bump)]
    pub user: Account<'info, User>,
    pub authority: Signer<'info>,  // Must be owner of the user account
    pub new_wallet: UncheckedAccount<'info>,  // The wallet to add
}

pub fn handler(ctx: Context<AddWallet>, new_wallet: Pubkey) -> Result<()> {
    let user = &mut ctx.accounts.user;

    // Verify authority is the active wallet (owner)
    require!(
        user.active_wallet == ctx.accounts.authority.key(),
        crate::ErrorCode::NotAuthorized
    )?;

    // Check wallet not already associated
    require!(
        !user.is_wallet_associated(&new_wallet),
        crate::ErrorCode::WalletAlreadyAssociated
    )?;

    // Check max wallets not reached
    require!(
        user.wallets_asociadas.len() < MAX_WALLETS,
        crate::ErrorCode::MaxWalletsReached
    )?;

    // Add the wallet
    user.wallets_asociadas.push(new_wallet);

    msg!("Wallet added to user: {}", new_wallet);

    Ok(())
}