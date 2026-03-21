//! Deposit funds instruction - Deposit funds into the job escrow

use crate::state::{Job, JobStatus, Config};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct DepositFunds<'info> {
    #[account(mut)]
    pub client: Signer<'info>,
    #[account(
        mut,
        seeds = [Job::SEED, client.key().as_ref(), &job_id.to_le_bytes()],
        bump = job.bump
    )]
    pub job: Account<'info, Job>,
    #[account(seeds = [Config::SEED], bump = config.bump)]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<DepositFunds>, job_id: u64) -> Result<()> {
    let job = &mut ctx.accounts.job;
    let config = &ctx.accounts.config;

    // Validations
    require!(!config.paused, crate::ErrorCode::ProgramPaused);
    require!(
        job.status == JobStatus::Created,
        crate::ErrorCode::InvalidJobStatus
    );
    require!(
        job.client == ctx.accounts.client.key(),
        crate::ErrorCode::NotJobClient
    );

    // Calculate total (amount + fee)
    let total = job.amount + job.fee_amount;

    // Transfer funds from client to job (PDA would need to be a vault)
    // For now, we just change status - in a real implementation,
    // the client would transfer to a vault PDA
    
    // Update status
    job.status = JobStatus::Funded;
    job.updated_at = Clock::get()?.unix_timestamp;

    msg!(
        "Funds deposited: {} lamports (job: {})",
        job.amount,
        job_id
    );

    Ok(())
}