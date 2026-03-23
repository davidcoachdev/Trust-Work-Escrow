//! Accept Job instruction

use crate::state::{Job, JobStatus};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct AcceptJob<'info> {
    #[account(mut)]
    pub freelancer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()],
        bump = job.bump
    )]
    pub job: Account<'info, Job>,
    pub client: Signer<'info>,
}

pub fn handler(ctx: Context<AcceptJob>, job_id: u64) -> Result<()> {
    let job = &mut ctx.accounts.job;

    require!(
        job.status == JobStatus::ApplicationsOpen,
        crate::ErrorCode::InvalidJobStatus
    );
    require!(
        job.total_deposited >= job.amount + job.entry_fee,
        crate::ErrorCode::JobNotFunded
    );

    job.freelancer = Some(ctx.accounts.freelancer.key());
    job.status = JobStatus::InProgress;
    job.updated_at = Clock::get()?.unix_timestamp;

    msg!("Job {} accepted by freelancer", job_id);
    Ok(())
}
