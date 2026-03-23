//! Cancel Job instruction

use crate::state::{Config, Job, JobStatus};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct CancelJob<'info> {
    #[account(mut)]
    pub client: Signer<'info>,
    #[account(
        mut,
        seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()],
        bump = job.bump
    )]
    pub job: Account<'info, Job>,
    #[account(
        seeds = [Config::SEED],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,
}

pub fn handler(ctx: Context<CancelJob>, job_id: u64) -> Result<()> {
    let job_account_info = ctx.accounts.job.to_account_info();
    let client_account_info = ctx.accounts.client.to_account_info();

    let job = &mut ctx.accounts.job;

    require!(
        job.client == ctx.accounts.client.key(),
        state::ErrorCode::NotJobClient
    );
    require!(
        matches!(
            job.status,
            JobStatus::Created | JobStatus::ApplicationsOpen | JobStatus::InProgress
        ),
        state::ErrorCode::InvalidJobStatus
    );

    let refund_amount = job.total_deposited;

    if refund_amount > 0 {
        **job_account_info.try_borrow_mut_lamports()? -= refund_amount;
        **client_account_info.try_borrow_mut_lamports()? += refund_amount;
    }

    job.status = JobStatus::Cancelled;
    job.updated_at = Clock::get()?.unix_timestamp;

    msg!(
        "Job {} cancelled. Refunded {} lamports",
        job_id,
        refund_amount
    );
    Ok(())
}
