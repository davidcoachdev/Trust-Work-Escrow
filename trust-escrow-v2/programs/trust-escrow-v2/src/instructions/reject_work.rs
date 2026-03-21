//! Reject work instruction - Client rejects and opens dispute

use crate::state::{Job, JobStatus, MAX_DISPUTE_REASON_LENGTH};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct RejectWork<'info> {
    #[account(mut)]
    pub client: Signer<'info>,
    #[account(
        mut,
        seeds = [Job::SEED, client.key().as_ref(), &job_id.to_le_bytes()],
        bump = job.bump
    )]
    pub job: Account<'info, Job>,
}

pub fn handler(ctx: Context<RejectWork>, job_id: u64, reason: String) -> Result<()> {
    let job = &mut ctx.accounts.job;

    require!(
        job.status == JobStatus::Submitted,
        crate::ErrorCode::InvalidJobStatus
    );
    require!(
        job.client == ctx.accounts.client.key(),
        crate::ErrorCode::NotJobClient
    );
    require!(!reason.is_empty(), crate::ErrorCode::EmptyDisputeReason)?;
    require!(
        reason.len() <= MAX_DISPUTE_REASON_LENGTH,
        crate::ErrorCode::DisputeReasonTooLong
    );

    job.status = JobStatus::Disputed;
    job.dispute_reason = reason;
    job.updated_at = Clock::get()?.unix_timestamp;

    msg!("Job {} rejected, dispute opened", job_id);

    Ok(())
}
