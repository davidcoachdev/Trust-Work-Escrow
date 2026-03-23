//! Submit Work instruction

use crate::state::{Job, JobStatus};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct SubmitWork<'info> {
    #[account(mut)]
    pub freelancer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()],
        bump = job.bump
    )]
    pub job: Account<'info, Job>,
    pub client: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<SubmitWork>, job_id: u64) -> Result<()> {
    let job = &mut ctx.accounts.job;

    require!(
        job.status == JobStatus::InProgress,
        crate::ErrorCode::InvalidJobStatus
    );
    require!(
        job.freelancer.is_some() && job.freelancer.unwrap() == ctx.accounts.freelancer.key(),
        crate::ErrorCode::NotJobFreelancer
    );

    job.status = JobStatus::Submitted;
    job.submitted_at = Some(Clock::get()?.unix_timestamp);
    job.updated_at = Clock::get()?.unix_timestamp;

    msg!("Work submitted for job {}", job_id);
    Ok(())
}
