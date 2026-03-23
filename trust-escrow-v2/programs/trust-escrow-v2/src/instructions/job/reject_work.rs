//! Reject Work instruction

use crate::state::{Job, JobStatus};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct RejectWork<'info> {
    #[account(mut)]
    pub client: Signer<'info>,
    #[account(
        mut,
        seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()],
        bump = job.bump
    )]
    pub job: Account<'info, Job>,
    pub freelancer: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<RejectWork>, job_id: u64, _reason: String) -> Result<()> {
    let job = &mut ctx.accounts.job;

    require!(
        job.status == JobStatus::Submitted,
        crate::ErrorCode::InvalidJobStatus
    );
    require!(
        job.client == ctx.accounts.client.key(),
        crate::ErrorCode::NotJobClient
    );

    job.status = JobStatus::InProgress;
    job.submitted_at = None;
    job.updated_at = Clock::get()?.unix_timestamp;

    msg!("Work rejected for job {}", job_id);
    Ok(())
}
