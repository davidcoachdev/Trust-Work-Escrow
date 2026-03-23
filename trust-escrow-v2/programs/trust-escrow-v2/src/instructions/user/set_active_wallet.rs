//! Set Active Wallet instruction

use crate::state::User;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct SetActiveWallet<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"user", authority.key().as_ref()],
        bump = user.bump
    )]
    pub user: Account<'info, User>,
}

pub fn handler(ctx: Context<SetActiveWallet>, wallet: Pubkey) -> Result<()> {
    let user = &mut ctx.accounts.user;

    require!(
        user.wallet_principal == wallet || user.wallets_asociadas.contains(&wallet),
        crate::ErrorCode::WalletNotAssociated
    );

    user.active_wallet = wallet;

    msg!("Active wallet set to {}", wallet);
    Ok(())
}
