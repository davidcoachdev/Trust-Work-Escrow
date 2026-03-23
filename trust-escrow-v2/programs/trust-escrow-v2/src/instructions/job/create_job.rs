//! Create Job instruction

use crate::state::{
    Config, Job, JobStatus, MAX_DESCRIPTION_LENGTH, MAX_TITLE_LENGTH, MIN_JOB_AMOUNT,
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct CreateJob<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = Job::INIT_SPACE + 8,
        seeds = [b"job", authority.key().as_ref(), &job_id.to_le_bytes()],
        bump
    )]
    pub job: Account<'info, Job>,
    #[account(
        seeds = [Config::SEED],
        bump = config.bump
    )]
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
    require!(!title.is_empty(), crate::ErrorCode::EmptyTitle);
    require!(
        title.len() <= MAX_TITLE_LENGTH,
        crate::ErrorCode::TitleTooLong
    );
    require!(
        description.len() <= MAX_DESCRIPTION_LENGTH,
        crate::ErrorCode::DescriptionTooLong
    );
    require!(amount >= MIN_JOB_AMOUNT, crate::ErrorCode::AmountTooSmall);

    let config = &ctx.accounts.config;
    require!(!config.paused, crate::ErrorCode::ProgramPaused);

    let entry_fee = amount * config.fee_percent as u64 / 10000;

    let job = &mut ctx.accounts.job;
    job.client = ctx.accounts.authority.key();
    job.title = title;
    job.description = description;
    job.amount = amount;
    job.entry_fee = entry_fee;
    job.total_deposited = 0;
    job.deadline = deadline;
    job.status = JobStatus::Created;
    job.freelancer = None;
    job.team = None;
    job.applications = Vec::new();
    job.bump = ctx.bumps.job;
    job.created_at = Clock::get()?.unix_timestamp;
    job.updated_at = Clock::get()?.unix_timestamp;
    job.submitted_at = None;

    msg!("Job created: {} (ID: {})", job.title, job_id);
    Ok(())
}
