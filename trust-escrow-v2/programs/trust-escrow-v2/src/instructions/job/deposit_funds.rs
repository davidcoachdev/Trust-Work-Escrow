//! Deposit Funds instruction

use crate::state::{Config, Job, JobStatus};
use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct DepositFunds<'info> {
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
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<DepositFunds>, job_id: u64) -> Result<()> {
    let job_account = ctx.accounts.job.to_account_info();
    let client_account = ctx.accounts.client.to_account_info();
    let system_account = ctx.accounts.system_program.to_account_info();

    let job = &mut ctx.accounts.job;

    require!(
        matches!(job.status, JobStatus::Created | JobStatus::ApplicationsOpen),
        crate::ErrorCode::InvalidJobStatus
    );

    let amount_to_deposit = job.amount + job.entry_fee;

    let cpi_ctx = CpiContext::new(
        system_account,
        Transfer {
            from: client_account,
            to: job_account,
        },
    );
    transfer(cpi_ctx, amount_to_deposit)?;

    job.total_deposited = amount_to_deposit;
    job.updated_at = Clock::get()?.unix_timestamp;

    msg!("Deposited {} lamports to job {}", amount_to_deposit, job_id);
    Ok(())
}
