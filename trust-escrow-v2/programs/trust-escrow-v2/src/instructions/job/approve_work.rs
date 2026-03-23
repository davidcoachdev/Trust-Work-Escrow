//! Approve Work instruction

use crate::error::ErrorCode;
use crate::state::{Config, Job, JobStatus};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct ApproveWork<'info> {
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
    pub freelancer: UncheckedAccount<'info>,
    pub treasury: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<ApproveWork>, job_id: u64) -> Result<()> {
    let job_account_info = ctx.accounts.job.to_account_info();
    let freelancer_info = ctx.accounts.freelancer.to_account_info();
    let treasury_info = ctx.accounts.treasury.to_account_info();

    let job = &mut ctx.accounts.job;
    let config = &ctx.accounts.config;

    require!(
        job.status == JobStatus::Submitted,
        ErrorCode::InvalidJobStatus
    );
    require!(job.freelancer.is_some(), ErrorCode::NoFreelancerAssigned);

    let amount = job.amount;
    let fee = amount * config.fee_percent as u64 / 10000;
    let payment = amount - fee;

    **job_account_info.try_borrow_mut_lamports()? -= payment;
    **freelancer_info.try_borrow_mut_lamports()? += payment;

    **job_account_info.try_borrow_mut_lamports()? -= fee;
    **treasury_info.try_borrow_mut_lamports()? += fee;

    job.status = JobStatus::Approved;
    job.updated_at = Clock::get()?.unix_timestamp;

    msg!(
        "Work approved for job {}. Paid {} lamports",
        job_id,
        payment
    );
    Ok(())
}
