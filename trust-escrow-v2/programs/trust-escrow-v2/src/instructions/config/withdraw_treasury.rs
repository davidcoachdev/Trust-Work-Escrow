//! Withdraw Treasury instruction

use crate::state::Config;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct WithdrawTreasury<'info> {
    #[account(mut, seeds = [Config::SEED], bump = config.bump)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub treasury: SystemAccount<'info>,
    #[account(mut)]
    pub recipient: SystemAccount<'info>,
    // Multiple signers for multisig - in real implementation would be >1 signer
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<WithdrawTreasury>, amount: u64) -> Result<()> {
    let config = &ctx.accounts.config;

    // Verify treasury has enough balance
    require!(
        ctx.accounts.treasury.lamports() >= amount,
        crate::ErrorCode::InsufficientFunds
    );

    // In a full implementation, verify multisig signatures
    // For now, just require admin signature
    require!(
        ctx.accounts.authority.key() == config.admin,
        crate::ErrorCode::NotAdmin
    );

    // Transfer funds
    let treasury = &mut ctx.accounts.treasury;
    let recipient = &mut ctx.accounts.recipient;

    **treasury.lamports.borrow_mut() -= amount;
    **recipient.lamports.borrow_mut() += amount;

    msg!("Withdrawn {} lamports to {}", amount, recipient.key());

    Ok(())
}
