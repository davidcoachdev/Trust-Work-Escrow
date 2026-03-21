//! Accept job instruction - Freelancer accepts a job

use crate::state::{Job, JobStatus};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct AcceptJob<'info> {
    #[account(mut)]
    pub freelancer: Signer<'info>,
    #[account(
        mut,
        seeds = [Job::SEED, client.key().as_ref(), &job_id.to_le_bytes()],
        bump = job.bump
    )]
    pub job: Account<'info, Job>,
}

pub fn handler(ctx: Context<AcceptJob>, job_id: u64) -> Result<()> {
    let job = &mut ctx.accounts.job;

    require!(
        job.status == JobStatus::Funded,
        crate::ErrorCode::InvalidJobStatus
    );
    require!(
        job.freelancer.is_none(),
        crate::ErrorCode::InvalidJobStatus
    );
    require!(
        job.client != ctx.accounts.freelancer.key(),
        crate::ErrorCode::CannotAcceptOwnJob
    );

    job.freelancer = Some(ctx.accounts.freelancer.key());
    job.status = JobStatus::InProgress;
    job.updated_at = Clock::get()?.unix_timestamp;

    msg!("Job {} accepted by: {}", job_id, ctx.accounts.freelancer.key());

    Ok(())
}
