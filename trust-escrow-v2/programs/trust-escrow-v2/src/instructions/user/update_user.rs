//! Update User instruction

use crate::state::{User, MAX_BIO_LENGTH};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct UpdateUser<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"user", authority.key().as_ref()],
        bump = user.bump
    )]
    pub user: Account<'info, User>,
}

pub fn handler(ctx: Context<UpdateUser>, bio: Option<String>) -> Result<()> {
    let user = &mut ctx.accounts.user;

    if let Some(bio_str) = bio {
        require!(
            bio_str.len() <= MAX_BIO_LENGTH,
            crate::ErrorCode::BioTooLong
        );
        user.bio = Some(bio_str);
    }

    msg!("User {} updated", user.username);
    Ok(())
}
