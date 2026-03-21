//! Update user instruction - Update user profile

use crate::state::{User, MAX_BIO_LENGTH};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct UpdateUser<'info> {
    #[account(mut, seeds = [b"user", authority.key().as_ref()], bump = user.bump)]
    pub user: Account<'info, User>,
    pub authority: Signer<'info>,
}

pub fn handler(ctx: Context<UpdateUser>, bio: Option<String>) -> Result<()> {
    let user = &mut ctx.accounts.user;

    // Verify authority is associated with this user
    require!(
        user.is_wallet_associated(&ctx.accounts.authority.key()),
        crate::ErrorCode::NotAuthorized
    );

    // Update bio if provided
    if let Some(bio_str) = bio {
        require!(
            bio_str.len() <= MAX_BIO_LENGTH,
            crate::ErrorCode::BioTooLong
        );
        user.bio = Some(bio_str);
    }

    msg!("User profile updated: {}", user.username);

    Ok(())
}