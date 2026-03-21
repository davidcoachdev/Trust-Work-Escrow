//! Set active wallet instruction - Change active wallet for the session

use crate::state::User;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct SetActiveWallet<'info> {
    #[account(mut, seeds = [b"user", authority.key().as_ref()], bump = user.bump)]
    pub user: Account<'info, User>,
    pub authority: Signer<'info>,
}

pub fn handler(ctx: Context<SetActiveWallet>, wallet: Pubkey) -> Result<()> {
    let user = &mut ctx.accounts.user;

    // Verify the wallet is associated
    require!(
        user.is_wallet_associated(&wallet),
        crate::ErrorCode::WalletNotAssociated
    )?;

    // Set as active
    user.active_wallet = wallet;

    msg!("Active wallet set to: {}", wallet);

    Ok(())
}