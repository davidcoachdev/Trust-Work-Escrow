#![allow(unused_imports)]
use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer, ID as SYSTEM_PROGRAM_ID};
use crate::errors::ErrorCode;
use crate::state::*;
use crate::{ARBITER_FEE_BPS_PER_PARTY, AUTO_APPROVAL_DELAY, BASIS_POINTS, DISPUTE_ACCEPT_GRACE, INITIAL_AUTHORITY, MAX_APPLICATIONS, MAX_ARBITERS, MAX_EVIDENCE_COUNT, MAX_MILESTONES, MAX_PAUSE_DURATION, MIN_JOB_AMOUNT};
use crate::{check_not_paused, cleanup_job_applications, close_evidence_account, compute_fee, compute_shortfall, transfer_job_lamports, validate_treasury_destination};

#[derive(Accounts)]
#[instruction(job_id: u64, index: u8)]
pub struct CreateMilestone<'info> {
    #[account(mut)]
    pub client: Signer<'info>,
    #[account(mut, seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
    #[account(init, payer = client, space = Milestone::INIT_SPACE + 8, seeds = [b"milestone", job.key().as_ref(), &[index]], bump)]
    pub milestone: Account<'info, Milestone>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(job_id: u64, milestone_index: u8)]
pub struct SubmitMilestone<'info> {
    pub freelancer: Signer<'info>,
    /// CHECK: client validado por el PDA del job.
    pub client: UncheckedAccount<'info>,
    #[account(seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
    #[account(mut, seeds = [b"milestone", job.key().as_ref(), &[milestone_index]], bump = milestone.bump)]
    pub milestone: Account<'info, Milestone>,
}

#[derive(Accounts)]
#[instruction(job_id: u64, milestone_index: u8)]
pub struct ApproveMilestone<'info> {
    pub client: Signer<'info>,
    #[account(mut, seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
    #[account(mut, seeds = [b"milestone", job.key().as_ref(), &[milestone_index]], bump = milestone.bump)]
    pub milestone: Account<'info, Milestone>,
    #[account(mut, constraint = job.freelancer == Some(freelancer.key()) @ ErrorCode::NotJobFreelancer)]
    pub freelancer: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(job_id: u64, milestone_index: u8)]
pub struct RejectMilestone<'info> {
    pub client: Signer<'info>,
    #[account(mut, seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
    #[account(mut, seeds = [b"milestone", job.key().as_ref(), &[milestone_index]], bump = milestone.bump)]
    pub milestone: Account<'info, Milestone>,
}

pub fn create_milestone(
    ctx: Context<CreateMilestone>,
    _job_id: u64,
    index: u8,
    amount: u64,
) -> Result<()> {
    let job = &mut ctx.accounts.job;
    require!(
        job.status == JobStatus::InProgress,
        ErrorCode::InvalidJobStatus
    );
    require!(
        index == job.milestones_total,
        ErrorCode::InvalidMilestoneIndex
    );
    require!(
        job.milestones_total < MAX_MILESTONES as u8,
        ErrorCode::MilestoneAlreadyCompleted
    );

    let new_total = job
        .milestones_amount_total
        .checked_add(amount)
        .ok_or(ErrorCode::MathOverflow)?;
    require!(
        new_total <= job.amount,
        ErrorCode::MilestoneAmountExceedsFunds
    );

    let milestone = &mut ctx.accounts.milestone;
    milestone.job = job.key();
    milestone.amount = amount;
    milestone.status = MilestoneStatus::Pending;
    milestone.index = index;
    milestone.bump = ctx.bumps.milestone;

    job.milestones_total = job
        .milestones_total
        .checked_add(1)
        .ok_or(ErrorCode::MathOverflow)?;
    job.milestones_amount_total = new_total;

    Ok(())
}

pub fn submit_milestone(
    ctx: Context<SubmitMilestone>,
    _job_id: u64,
    _milestone_index: u8,
) -> Result<()> {
    let job = &ctx.accounts.job;
    let milestone = &mut ctx.accounts.milestone;
    require!(
        job.freelancer == Some(ctx.accounts.freelancer.key()),
        ErrorCode::NotJobFreelancer
    );
    require!(
        job.status == JobStatus::InProgress,
        ErrorCode::InvalidJobStatus
    );
    require!(
        milestone.status == MilestoneStatus::Pending
            || milestone.status == MilestoneStatus::Rejected,
        ErrorCode::MilestoneAlreadyCompleted
    );

    milestone.status = MilestoneStatus::Submitted;

    Ok(())
}

pub fn approve_milestone(
    ctx: Context<ApproveMilestone>,
    _job_id: u64,
    _milestone_index: u8,
) -> Result<()> {
    let job = &mut ctx.accounts.job;
    require!(
        job.client == ctx.accounts.client.key(),
        ErrorCode::NotJobClient
    );
    require!(
        job.status == JobStatus::InProgress,
        ErrorCode::InvalidJobStatus
    );
    let milestone = &mut ctx.accounts.milestone;
    require!(
        milestone.status == MilestoneStatus::Submitted,
        ErrorCode::MilestoneAlreadyCompleted
    );

    let amount = milestone.amount;

    transfer_job_lamports(
        &job.to_account_info(),
        &ctx.accounts.freelancer.to_account_info(),
        amount,
    )?;

    job.milestones_approved = job
        .milestones_approved
        .checked_add(1)
        .ok_or(ErrorCode::MathOverflow)?;
    milestone.status = MilestoneStatus::Approved;

    Ok(())
}

pub fn reject_milestone(
    ctx: Context<RejectMilestone>,
    _job_id: u64,
    _milestone_index: u8,
) -> Result<()> {
    let job = &ctx.accounts.job;
    let milestone = &mut ctx.accounts.milestone;
    require!(
        job.client == ctx.accounts.client.key(),
        ErrorCode::NotJobClient
    );
    require!(
        job.status == JobStatus::InProgress,
        ErrorCode::InvalidJobStatus
    );
    require!(
        milestone.status == MilestoneStatus::Submitted,
        ErrorCode::MilestoneAlreadyCompleted
    );

    milestone.status = MilestoneStatus::Rejected;
    Ok(())
}

