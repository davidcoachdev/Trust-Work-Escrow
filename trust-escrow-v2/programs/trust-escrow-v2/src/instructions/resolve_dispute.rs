//! Resolve dispute instruction - Arbiter resolves a dispute

use crate::state::{Job, JobStatus, Config};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct ResolveDispute<'info> {
    #[account(mut)]
    pub arbiter: Signer<'info>,
    #[account(
        mut,
        seeds = [Job::SEED, client.key().as_ref(), &job_id.to_le_bytes()],
        bump = job.bump
    )]
    pub job: Account<'info, Job>,
    #[account(mut)]
    pub client: SystemAccount<'info>,
    #[account(mut)]
    pub freelancer: SystemAccount<'info>,
    #[account(mut)]
    pub treasury: SystemAccount<'info>,
    #[account(seeds = [Config::SEED], bump = config.bump)]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ResolveDispute>, job_id: u64, freelancer_percent: u8) -> Result<()> {
    let job = &ctx.accounts.job;
    let config = &ctx.accounts.config;

    require!(
        job.status == JobStatus::Disputed,
        crate::ErrorCode::InvalidJobStatus
    );
    require!(
        job.arbiter == Some(ctx.accounts.arbiter.key()),
        crate::ErrorCode::NotArbiter
    );
    require!(
        freelancer_percent <= 100,
        crate::ErrorCode::InvalidFreelancerPercent
    );

    let net_amount = job.amount - job.fee_amount;
    let freelancer_share = net_amount * freelancer_percent as u64 / 100;
    let client_share = net_amount - freelancer_share;

    // Transfer to freelancer
    let freelancer = &mut ctx.accounts.freelancer;
    **freelancer.lamports.borrow_mut() += freelancer_share;

    // Transfer to client
    let client = &mut ctx.accounts.client;
    **client.lamports.borrow_mut() += client_share;

    // Transfer fee to treasury
    let treasury = &mut ctx.accounts.treasury;
    **treasury.lamports.borrow_mut() += job.fee_amount * 2;

    msg!(
        "Dispute resolved: job {} - {}% to freelancer ({}), {}% to client ({})",
        job_id,
        freelancer_percent,
        freelancer_share,
        100 - freelancer_percent,
        client_share
    );

    Ok(())
}
