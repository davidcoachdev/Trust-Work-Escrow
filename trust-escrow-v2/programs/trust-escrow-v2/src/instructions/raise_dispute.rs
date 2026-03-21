//! Raise dispute instruction - Freelancer raises a dispute

use crate::state::{Job, JobStatus, MAX_DISPUTE_REASON_LENGTH};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct RaiseDispute<'info> {
    #[account(mut)]
    pub freelancer: Signer<'info>,
    #[account(
        mut,
        seeds = [Job::SEED, client.key().as_ref(), &job_id.to_le_bytes()],
        bump = job.bump
    )]
    pub job: Account<'info, Job>,
}

pub fn handler(ctx: Context<RaiseDispute>, job_id: u64, reason: String) -> Result<()> {
    let job = &mut ctx.accounts.job;

    require!(
        job.status == JobStatus::Submitted,
        crate::ErrorCode::InvalidJobStatus
    );
    require!(
        job.freelancer == Some(ctx.accounts.freelancer.key()),
        crate::ErrorCode::NotJobFreelancer
    );
    require!(!reason.is_empty(), crate::ErrorCode::EmptyDisputeReason)?;
    require!(
        reason.len() <= MAX_DISPUTE_REASON_LENGTH,
        crate::ErrorCode::DisputeReasonTooLong
    );

    job.status = JobStatus::Disputed;
    job.dispute_reason = reason;
    job.updated_at = Clock::get()?.unix_timestamp;

    msg!("Dispute raised for job: {}", job_id);

    Ok(())
}
