//! Add Wallet instruction

use crate::state::{User, MAX_WALLETS};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct AddWallet<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"user", authority.key().as_ref()],
        bump = user.bump
    )]
    pub user: Account<'info, User>,
    pub new_wallet: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AddWallet>, _new_wallet: Pubkey) -> Result<()> {
    let user = &mut ctx.accounts.user;

    require!(
        user.wallet_principal != ctx.accounts.new_wallet.key(),
        crate::ErrorCode::WalletAlreadyAssociated
    );
    require!(
        !user
            .wallets_asociadas
            .contains(&ctx.accounts.new_wallet.key()),
        crate::ErrorCode::WalletAlreadyAssociated
    );
    require!(
        user.wallets_asociadas.len() < MAX_WALLETS,
        crate::ErrorCode::MaxWalletsReached
    );

    user.wallets_asociadas.push(ctx.accounts.new_wallet.key());

    msg!("Wallet added to user {}", user.username);
    Ok(())
}
