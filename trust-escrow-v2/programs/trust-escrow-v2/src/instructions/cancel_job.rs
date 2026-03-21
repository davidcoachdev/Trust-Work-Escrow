//! Cancel job instruction - Client cancels before acceptance

use crate::state::{Job, JobStatus};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CancelJob<'info> {
    #[account(mut)]
    pub client: Signer<'info>,
    #[account(
        mut,
        close = client,
        seeds = [Job::SEED, client.key().as_ref(), &job_id.to_le_bytes()],
        bump = job.bump
    )]
    pub job: Account<'info, Job>,
}

pub fn handler(ctx: Context<CancelJob>, job_id: u64) -> Result<()> {
    let job = &ctx.accounts.job;

    require!(
        job.client == ctx.accounts.client.key(),
        crate::ErrorCode::NotJobClient
    );
    require!(
        job.status == JobStatus::Created || job.status == JobStatus::Funded,
        crate::ErrorCode::InvalidJobStatus
    );

    // If funded, funds would be refunded (simplified - just close account)

    msg!("Job {} cancelled", job_id);

    Ok(())
}
