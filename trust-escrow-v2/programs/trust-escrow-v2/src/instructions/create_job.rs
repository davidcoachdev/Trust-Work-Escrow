//! Create job instruction - Create a new job/escrow

use crate::state::{Job, JobStatus, Config, MAX_TITLE_LENGTH, MAX_DESCRIPTION_LENGTH, MIN_JOB_AMOUNT};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CreateJob<'info> {
    #[account(mut)]
    pub client: Signer<'info>,
    #[account(
        init,
        payer = client,
        space = Job::INIT_SPACE + 8,
        seeds = [Job::SEED, client.key().as_ref(), &job_id.to_le_bytes()],
        bump
    )]
    pub job: Account<'info, Job>,
    #[account(seeds = [Config::SEED], bump = config.bump)]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreateJob>,
    job_id: u64,
    title: String,
    description: String,
    amount: u64,
    deadline: i64,
    arbiter: Option<Pubkey>,
) -> Result<()> {
    let config = &ctx.accounts.config;

    // Check program not paused
    require!(!config.paused, crate::ErrorCode::ProgramPaused);

    // Validations
    require!(!title.is_empty(), crate::ErrorCode::EmptyTitle)?;
    require!(title.len() <= MAX_TITLE_LENGTH, crate::ErrorCode::TitleTooLong)?;
    require!(description.len() <= MAX_DESCRIPTION_LENGTH, crate::ErrorCode::DescriptionTooLong)?;
    require!(amount >= MIN_JOB_AMOUNT, crate::ErrorCode::AmountTooSmall)?;

    // Calculate fee
    let fee_amount = amount * config.fee_percent as u64 / 100;

    // Initialize job
    let job = &mut ctx.accounts.job;
    job.client = ctx.accounts.client.key();
    job.freelancer = None;
    job.arbiter = arbiter;
    job.amount = amount;
    job.fee_percent = config.fee_percent;
    job.fee_amount = fee_amount;
    job.status = JobStatus::Created;
    job.title = title.clone();
    job.description = description;
    job.deadline = deadline;
    job.created_at = Clock::get()?.unix_timestamp;
    job.updated_at = Clock::get()?.unix_timestamp;
    job.dispute_reason = String::new();
    job.bump = ctx.bumps.job;

    msg!("Job created: {} - Amount: {} lamports", title, amount);

    Ok(())
}