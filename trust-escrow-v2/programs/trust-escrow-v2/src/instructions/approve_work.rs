//! Approve work instruction - Client approves and releases funds

use crate::state::{Job, JobStatus, Config};
use anchor_lang::prelude::*;
use anchor_lang::system_program;

#[derive(Accounts)]
pub struct ApproveWork<'info> {
    #[account(mut)]
    pub client: Signer<'info>,
    #[account(
        mut,
        close = client,
        seeds = [Job::SEED, client.key().as_ref(), &job_id.to_le_bytes()],
        bump = job.bump
    )]
    pub job: Account<'info, Job>,
    #[account(mut)]
    pub freelancer: SystemAccount<'info>,
    #[account(mut)]
    pub treasury: SystemAccount<'info>,
    #[account(seeds = [Config::SEED], bump = config.bump)]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ApproveWork>, job_id: u64) -> Result<()> {
    let job = &ctx.accounts.job;
    let config = &ctx.accounts.config;

    require!(
        job.status == JobStatus::Submitted,
        crate::ErrorCode::InvalidJobStatus
    );
    require!(
        job.client == ctx.accounts.client.key(),
        crate::ErrorCode::NotJobClient
    );
    require!(
        ctx.accounts.treasury.key() == config.treasury,
        crate::ErrorCode::InvalidTreasury
    );

    let freelancer = &mut ctx.accounts.freelancer;
    let treasury = &mut ctx.accounts.treasury;
    
    // Transfer to freelancer (net amount)
    let net_amount = job.amount - job.fee_amount;
    **freelancer.lamports.borrow_mut() += net_amount;
    
    // Transfer fee to treasury
    **treasury.lamports.borrow_mut() += job.fee_amount * 2;

    msg!(
        "Job {} approved: {} to freelancer, {} to treasury",
        job_id,
        net_amount,
        job.fee_amount * 2
    );

    // Status will be set to Released when we update (but account is closed)
    // In practice, we'd keep the account or emit an event

    Ok(())
}
