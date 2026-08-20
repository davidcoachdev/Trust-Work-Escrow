#![allow(unused_imports)]
use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer, ID as SYSTEM_PROGRAM_ID};
use crate::errors::ErrorCode;
use crate::state::*;
use crate::{ARBITER_FEE_BPS_PER_PARTY, AUTO_APPROVAL_DELAY, BASIS_POINTS, DISPUTE_ACCEPT_GRACE, INITIAL_AUTHORITY, MAX_APPLICATIONS, MAX_ARBITERS, MAX_EVIDENCE_COUNT, MAX_MILESTONES, MAX_PAUSE_DURATION, MIN_JOB_AMOUNT};
use crate::{check_not_paused, cleanup_job_applications, close_evidence_account, compute_fee, compute_shortfall, transfer_job_lamports, validate_treasury_destination};

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct CreateJob<'info> {
    #[account(mut)]
    pub client: Signer<'info>,
    #[account(
        init,
        payer = client,
        space = Job::INIT_SPACE + 8,
        seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()],
        bump
    )]
    pub job: Account<'info, Job>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

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
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(job_id: u64, application_index: u8)]
pub struct ApplyToJob<'info> {
    #[account(mut)]
    pub applicant: Signer<'info>,
    /// CHECK: client validado por el PDA del job.
    pub client: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()],
        bump = job.bump
    )]
    pub job: Account<'info, Job>,
    #[account(
        init,
        payer = applicant,
        space = Application::INIT_SPACE + 8,
        seeds = [b"application", job.key().as_ref(), &[application_index], applicant.key().as_ref()],
        bump
    )]
    pub application: Account<'info, Application>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(job_id: u64, application_index: u8)]
pub struct AcceptApplication<'info> {
    #[account(mut)]
    pub client: Signer<'info>,
    #[account(
        mut,
        seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()],
        bump = job.bump
    )]
    pub job: Account<'info, Job>,
    #[account(mut)]
    pub applicant: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [b"application", job.key().as_ref(), &[application_index], applicant.key().as_ref()],
        bump = application.bump
    )]
    pub application: Account<'info, Application>,
}

#[derive(Accounts)]
#[instruction(job_id: u64, application_index: u8)]
pub struct RejectApplication<'info> {
    #[account(mut)]
    pub client: Signer<'info>,
    #[account(
        mut,
        seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()],
        bump = job.bump
    )]
    pub job: Account<'info, Job>,
    #[account(mut)]
    pub applicant: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [b"application", job.key().as_ref(), &[application_index], applicant.key().as_ref()],
        bump = application.bump,
        close = applicant
    )]
    pub application: Account<'info, Application>,
}

#[derive(Accounts)]
#[instruction(job_id: u64, application_index: u8)]
pub struct WithdrawApplication<'info> {
    #[account(mut)]
    pub applicant: Signer<'info>,
    /// CHECK: client for PDA derivation
    pub client: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()],
        bump = job.bump
    )]
    pub job: Account<'info, Job>,
    #[account(
        mut,
        seeds = [b"application", job.key().as_ref(), &[application_index], applicant.key().as_ref()],
        bump = application.bump,
        close = applicant
    )]
    pub application: Account<'info, Application>,
}

#[derive(Accounts)]
#[instruction(job_id: u64, start_index: u8)]
pub struct CleanupApplications<'info> {
    pub client: Signer<'info>,
    #[account(
        mut,
        seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()],
        bump = job.bump,
        constraint = job.client == client.key() @ ErrorCode::NotJobClient
    )]
    pub job: Account<'info, Job>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct SubmitWork<'info> {
    pub freelancer: Signer<'info>,
    /// CHECK: client validado por el PDA del job (es job.client).
    pub client: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()],
        bump = job.bump
    )]
    pub job: Account<'info, Job>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct AutoApproveWork<'info> {
    #[account(mut)]
    pub keeper: Signer<'info>,
    /// CHECK: Debe ser el cliente ligado al PDA del job y recibe la rent restante.
    #[account(mut, constraint = client.owner == &SYSTEM_PROGRAM_ID @ ErrorCode::NotAuthorized)]
    pub client: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()],
        bump = job.bump,
        close = client
    )]
    pub job: Account<'info, Job>,
    #[account(mut, constraint = job.freelancer == Some(freelancer.key()) @ ErrorCode::NotJobFreelancer)]
    pub freelancer: SystemAccount<'info>,
    /// CHECK: Validada contra Config.treasury y que sea System-owned.
    #[account(
        mut,
        constraint = treasury.key() == config.treasury @ ErrorCode::InvalidTreasury,
        constraint = treasury.owner == &SYSTEM_PROGRAM_ID @ ErrorCode::InvalidTreasury
    )]
    pub treasury: UncheckedAccount<'info>,
    #[account(seeds = [b"dispute", job.key().as_ref()], bump)]
    pub dispute: Option<Account<'info, Dispute>>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct ApproveWork<'info> {
    pub client: Signer<'info>,
    #[account(
        mut,
        seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()],
        bump = job.bump,
        close = client
    )]
    pub job: Account<'info, Job>,
    #[account(mut, constraint = job.freelancer == Some(freelancer.key()) @ ErrorCode::NotJobFreelancer)]
    pub freelancer: SystemAccount<'info>,
    /// CHECK: Treasury que recibe la comision; validado contra config.treasury.
    #[account(
        mut,
        constraint = treasury.key() == config.treasury @ ErrorCode::InvalidTreasury,
        constraint = treasury.owner == &SYSTEM_PROGRAM_ID @ ErrorCode::InvalidTreasury
    )]
    pub treasury: UncheckedAccount<'info>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct RejectWork<'info> {
    pub client: Signer<'info>,
    #[account(
        mut,
        seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()],
        bump = job.bump
    )]
    pub job: Account<'info, Job>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct CancelJob<'info> {
    #[account(mut)]
    pub client: Signer<'info>,
    #[account(
        mut,
        seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()],
        bump = job.bump,
        close = client
    )]
    pub job: Account<'info, Job>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct PauseJob<'info> {
    pub client: Signer<'info>,
    #[account(
        mut,
        seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()],
        bump = job.bump
    )]
    pub job: Account<'info, Job>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct UnpauseJob<'info> {
    pub client: Signer<'info>,
    #[account(
        mut,
        seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()],
        bump = job.bump
    )]
    pub job: Account<'info, Job>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct ExpirePausedJob<'info> {
    pub caller: Signer<'info>,
    /// CHECK: client validado por el PDA del job (es job.client, a quien se reembolsa).
    #[account(constraint = client.owner == &SYSTEM_PROGRAM_ID @ ErrorCode::NotAuthorized)]
    pub client: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()],
        bump = job.bump,
        close = client
    )]
    pub job: Account<'info, Job>,
    pub system_program: Program<'info, System>,
}

pub fn create_job(
    ctx: Context<CreateJob>,
    _job_id: u64,
    amount: u64,
    deadline: i64,
) -> Result<()> {
    let config = &ctx.accounts.config;
    require!(!config.paused, ErrorCode::ProgramPaused);
    require!(amount >= MIN_JOB_AMOUNT, ErrorCode::AmountTooSmall);
    let clock = Clock::get()?;
    require!(deadline > clock.unix_timestamp, ErrorCode::DeadlineMustBeFuture);

    let fee_amount = compute_fee(amount, config.fee_bps)?;

    let job = &mut ctx.accounts.job;
    job.client = ctx.accounts.client.key();
    job.freelancer = None;
    job.amount = amount;
    job.fee_amount = fee_amount;
    job.status = JobStatus::Created;
    job.deadline = deadline;
    job.submitted_at = None;
    job.milestones_total = 0;
    job.milestones_approved = 0;
    job.milestones_amount_total = 0;
    job.bump = ctx.bumps.job;

    job.applicants = Vec::new();

    msg!("Job created");
    Ok(())
}

pub fn deposit_funds(ctx: Context<DepositFunds>, _job_id: u64) -> Result<()> {
    let job = &mut ctx.accounts.job;
    require!(
        job.status == JobStatus::Created,
        ErrorCode::InvalidJobStatus
    );
    require!(
        job.client == ctx.accounts.client.key(),
        ErrorCode::NotJobClient
    );
    check_not_paused(job)?;

    let total = job
        .amount
        .checked_add(job.fee_amount)
        .ok_or(ErrorCode::MathOverflow)?;

    transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.client.to_account_info(),
                to: job.to_account_info(),
            },
        ),
        total,
    )?;

    job.status = JobStatus::Funded;

    msg!("Funds deposited: {}", total);
    Ok(())
}

pub fn apply_to_job(
    ctx: Context<ApplyToJob>,
    _job_id: u64,
    application_index: u8,
    proposal_hash: [u8; 32],
) -> Result<()> {
    let job = &mut ctx.accounts.job;
    require!(job.status == JobStatus::Funded, ErrorCode::InvalidJobStatus);
    check_not_paused(job)?;
    require!(
        ctx.accounts.applicant.key() != job.client,
        ErrorCode::CannotWorkOnOwnJob
    );

    require!(
        !job.applicants
            .iter()
            .any(|a| *a == ctx.accounts.applicant.key()),
        ErrorCode::AlreadyApplied
    );
    require!(
        job.applicants.len() < MAX_APPLICATIONS,
        ErrorCode::InvalidApplicationIndex
    );
    require!(
        application_index as usize == job.applicants.len(),
        ErrorCode::ApplicationIndexMismatch
    );
    // Texto vacío/excesivo: on-chain solo ve hash, pero rechaza hash nulo (propuesta vacía sin hashear)
    // La longitud excesiva ya se valida off-chain (512) y en el SDK antes de hashear; aquí defendemos hash vacío.
    require!(proposal_hash != [0u8; 32], ErrorCode::EmptyProposal);

    let application = &mut ctx.accounts.application;
    application.job = job.key();
    application.index = application_index;
    application.applicant = ctx.accounts.applicant.key();
    application.proposal_hash = proposal_hash;
    application.status = ApplicationStatus::Pending;
    application.bump = ctx.bumps.application;
    job.applicants.push(ctx.accounts.applicant.key());

    msg!(
        "Application {} submitted for job: {}",
        application_index,
        job.key()
    );
    Ok(())
}

pub fn accept_application(
    ctx: Context<AcceptApplication>,
    _job_id: u64,
    application_index: u8,
) -> Result<()> {
    let job = &mut ctx.accounts.job;
    require!(job.status == JobStatus::Funded, ErrorCode::InvalidJobStatus);
    check_not_paused(job)?;
    require!(
        job.client == ctx.accounts.client.key(),
        ErrorCode::NotJobClient
    );

    let application = &mut ctx.accounts.application;
    require!(
        application.job == job.key(),
        ErrorCode::InvalidApplicationAccount
    );
    require!(
        application.index == application_index,
        ErrorCode::InvalidApplicationIndex
    );
    require!(
        application.status == ApplicationStatus::Pending,
        ErrorCode::ApplicationNotPending
    );
    require!(
        job.applicants.get(application_index as usize) == Some(&application.applicant),
        ErrorCode::InvalidApplicationAccount
    );
    require!(
        application.applicant == ctx.accounts.applicant.key(),
        ErrorCode::InvalidApplicationAccount
    );
    require!(job.freelancer.is_none(), ErrorCode::InvalidJobStatus);
    let applicant = application.applicant;
    application.status = ApplicationStatus::Accepted;

    job.freelancer = Some(applicant);
    job.status = JobStatus::InProgress;

    msg!("Application accepted: freelancer {}", applicant);
    Ok(())
}

pub fn reject_application(
    ctx: Context<RejectApplication>,
    _job_id: u64,
    application_index: u8,
) -> Result<()> {
    let job = &ctx.accounts.job;
    require!(job.status == JobStatus::Funded, ErrorCode::InvalidJobStatus);
    check_not_paused(job)?;
    require!(
        job.client == ctx.accounts.client.key(),
        ErrorCode::NotJobClient
    );
    let application = &ctx.accounts.application;
    require!(
        application.job == job.key(),
        ErrorCode::InvalidApplicationAccount
    );
    require!(
        application.index == application_index,
        ErrorCode::InvalidApplicationIndex
    );
    require!(
        application.status == ApplicationStatus::Pending,
        ErrorCode::ApplicationNotPending
    );
    require!(
        job.applicants.get(application_index as usize) == Some(&application.applicant),
        ErrorCode::InvalidApplicationAccount
    );
    require!(
        application.applicant == ctx.accounts.applicant.key(),
        ErrorCode::InvalidApplicationAccount
    );
    msg!(
        "Application rejected: index {} applicant {}",
        application_index,
        application.applicant
    );
    Ok(())
}

pub fn withdraw_application(
    ctx: Context<WithdrawApplication>,
    _job_id: u64,
    application_index: u8,
) -> Result<()> {
    let job = &ctx.accounts.job;
    require!(job.status == JobStatus::Funded, ErrorCode::InvalidJobStatus);
    check_not_paused(job)?;
    let application = &ctx.accounts.application;
    require!(
        application.job == job.key(),
        ErrorCode::InvalidApplicationAccount
    );
    require!(
        application.index == application_index,
        ErrorCode::InvalidApplicationIndex
    );
    require!(
        application.status == ApplicationStatus::Pending,
        ErrorCode::ApplicationNotPending
    );
    require!(
        job.applicants.get(application_index as usize) == Some(&application.applicant),
        ErrorCode::InvalidApplicationAccount
    );
    require!(
        application.applicant == ctx.accounts.applicant.key(),
        ErrorCode::InvalidApplicationAccount
    );
    msg!(
        "Application withdrawn: index {} applicant {}",
        application_index,
        application.applicant
    );
    Ok(())
}

pub fn cleanup_applications(
    ctx: Context<CleanupApplications>,
    _job_id: u64,
    start_index: u8,
) -> Result<()> {
    let job = &ctx.accounts.job;
    require!(
        job.status == JobStatus::InProgress
            || job.status == JobStatus::Submitted
            || job.status == JobStatus::Disputed,
        ErrorCode::InvalidJobStatus
    );
    require!(job.freelancer.is_some(), ErrorCode::NoFreelancerAssigned);
    require!(
        !ctx.remaining_accounts.is_empty(),
        ErrorCode::InvalidApplicationCleanupAccounts
    );
    cleanup_job_applications(
        job,
        &job.key(),
        start_index,
        ctx.remaining_accounts,
        false,
        false,
    )
}

pub fn submit_work(ctx: Context<SubmitWork>, _job_id: u64) -> Result<()> {
    let job = &mut ctx.accounts.job;
    require!(
        job.freelancer == Some(ctx.accounts.freelancer.key()),
        ErrorCode::NotJobFreelancer
    );
    require!(
        job.status == JobStatus::InProgress,
        ErrorCode::InvalidJobStatus
    );

    let clock = Clock::get()?;
    job.status = JobStatus::Submitted;
    job.submitted_at = Some(clock.unix_timestamp);

    msg!("Work submitted for job: {}", job.key());
    Ok(())
}

pub fn auto_approve_work(ctx: Context<AutoApproveWork>, _job_id: u64) -> Result<()> {
    let job = &mut ctx.accounts.job;
    require!(
        job.status == JobStatus::Submitted,
        ErrorCode::InvalidJobStatus
    );
    // Verificacion V3-SEC-009: keeper debe ser autorizado o permissionless con fee.
    // El job.client debe coincidir con la cuenta client que recibe el close (rent).
    require!(
        ctx.accounts.client.key() == job.client,
        ErrorCode::NotJobClient
    );
    let submitted_at = job.submitted_at.ok_or(ErrorCode::InvalidJobStatus)?;
    let deadline = submitted_at
        .checked_add(AUTO_APPROVAL_DELAY)
        .ok_or(ErrorCode::MathOverflow)?;
    let clock = Clock::get()?;
    require!(clock.unix_timestamp >= deadline, ErrorCode::AutoApprovalNotReady);
    require!(
        ctx.accounts.dispute.is_none(),
        ErrorCode::AutoApprovalBlocked
    );
    require!(
        job.freelancer == Some(ctx.accounts.freelancer.key()),
        ErrorCode::NotJobFreelancer
    );
    require!(
        job.milestones_total == 0 || job.milestones_approved == job.milestones_total,
        ErrorCode::AllMilestonesRequired
    );
    require!(
        ctx.accounts.treasury.key() == ctx.accounts.config.treasury,
        ErrorCode::InvalidTreasury
    );

    cleanup_job_applications(job, &job.key(), 0, ctx.remaining_accounts, true, true)?;

    let amount = job
        .amount
        .checked_sub(job.milestones_amount_total)
        .ok_or(ErrorCode::MathOverflow)?;
    let fee_amount = job.fee_amount;

    // V3-SEC-009 fix: keeper whitelist (client/freelancer/authority) sin fee,
    // permissionless con fee 1% (100 bps) para incentivar y evitar griefing.
    // close = client garantiza que la rent siempre vuelve al cliente, el keeper
    // solo recibe su fee via transfer directa (no via close ni remaining_accounts).
    let keeper_key = ctx.accounts.keeper.key();
    let is_privileged = keeper_key == job.client
        || Some(keeper_key) == job.freelancer
        || keeper_key == ctx.accounts.config.authority;
    let keeper_fee: u64 = if is_privileged {
        0
    } else {
        compute_fee(amount, 100)?
    };
    let freelancer_payout = amount
        .checked_sub(keeper_fee)
        .ok_or(ErrorCode::MathOverflow)?;

    // Evitar transfer a la misma cuenta si keeper == freelancer (ya es privileged, fee 0).
    // Si keeper es permissionless y coincide con freelancer no debería pasar (privileged), pero por robustez:
    if keeper_fee > 0 && keeper_key == ctx.accounts.freelancer.key() {
        // freelancer ya es keeper, no cobrar fee
        transfer_job_lamports(
            &job.to_account_info(),
            &ctx.accounts.freelancer.to_account_info(),
            amount,
        )?;
    } else {
        transfer_job_lamports(
            &job.to_account_info(),
            &ctx.accounts.freelancer.to_account_info(),
            freelancer_payout,
        )?;
        if keeper_fee > 0 {
            transfer_job_lamports(
                &job.to_account_info(),
                &ctx.accounts.keeper.to_account_info(),
                keeper_fee,
            )?;
        }
    }
    transfer_job_lamports(
        &job.to_account_info(),
        &ctx.accounts.treasury.to_account_info(),
        fee_amount,
    )?;
    job.status = JobStatus::Released;
    msg!(
        "Auto-approved: {} to freelancer, {} keeper fee, {} treasury fee",
        freelancer_payout,
        keeper_fee,
        fee_amount
    );
    // remaining rent refund via `close = client` on job account
    Ok(())
}

pub fn approve_work(ctx: Context<ApproveWork>, _job_id: u64) -> Result<()> {
    let job = &mut ctx.accounts.job;
    require!(
        job.client == ctx.accounts.client.key(),
        ErrorCode::NotJobClient
    );
    require!(
        job.status == JobStatus::Submitted,
        ErrorCode::InvalidJobStatus
    );
    require!(job.freelancer.is_some(), ErrorCode::NoFreelancerAssigned);
    require!(
        job.milestones_total == 0 || job.milestones_approved == job.milestones_total,
        ErrorCode::AllMilestonesRequired
    );

    cleanup_job_applications(job, &job.key(), 0, ctx.remaining_accounts, true, true)?;

    let amount = job
        .amount
        .checked_sub(job.milestones_amount_total)
        .ok_or(ErrorCode::MathOverflow)?;
    let fee_amount = job.fee_amount;

    transfer_job_lamports(
        &job.to_account_info(),
        &ctx.accounts.freelancer.to_account_info(),
        amount,
    )?;
    transfer_job_lamports(
        &job.to_account_info(),
        &ctx.accounts.treasury.to_account_info(),
        fee_amount,
    )?;

    job.status = JobStatus::Released;

    msg!(
        "Work approved: {} to freelancer, {} fee to treasury",
        amount,
        fee_amount
    );
    Ok(())
}

pub fn reject_work(ctx: Context<RejectWork>, _job_id: u64) -> Result<()> {
    let job = &mut ctx.accounts.job;
    require!(
        job.client == ctx.accounts.client.key(),
        ErrorCode::NotJobClient
    );
    require!(
        job.status == JobStatus::Submitted,
        ErrorCode::InvalidJobStatus
    );

    job.status = JobStatus::InProgress;

    msg!("Work rejected, returned to InProgress: {}", job.key());
    Ok(())
}

pub fn cancel_job(ctx: Context<CancelJob>, _job_id: u64) -> Result<()> {
    let job = &mut ctx.accounts.job;
    require!(
        job.client == ctx.accounts.client.key(),
        ErrorCode::NotJobClient
    );
    require!(
        job.status == JobStatus::Created || job.status == JobStatus::Funded,
        ErrorCode::InvalidJobStatus
    );

    cleanup_job_applications(job, &job.key(), 0, ctx.remaining_accounts, true, true)?;

    if job.status == JobStatus::Funded {
        let total = job
            .amount
            .checked_add(job.fee_amount)
            .ok_or(ErrorCode::MathOverflow)?;

        transfer_job_lamports(
            &job.to_account_info(),
            &ctx.accounts.client.to_account_info(),
            total,
        )?;
    }

    job.status = JobStatus::Cancelled;

    msg!("Job cancelled: {}", job.key());
    Ok(())
}

pub fn pause_job(ctx: Context<PauseJob>, _job_id: u64) -> Result<()> {
    let job = &mut ctx.accounts.job;
    require!(
        job.client == ctx.accounts.client.key(),
        ErrorCode::NotJobClient
    );
    require!(
        job.status == JobStatus::Created || job.status == JobStatus::Funded,
        ErrorCode::CannotPauseWithFreelancer
    );
    require!(
        job.freelancer.is_none(),
        ErrorCode::CannotPauseWithFreelancer
    );
    require!(!job.paused, ErrorCode::JobPaused);
    let now = Clock::get()?.unix_timestamp;
    job.paused = true;
    job.paused_at = now;
    Ok(())
}

pub fn unpause_job(ctx: Context<UnpauseJob>, _job_id: u64) -> Result<()> {
    let job = &mut ctx.accounts.job;
    require!(
        job.client == ctx.accounts.client.key(),
        ErrorCode::NotJobClient
    );
    require!(job.paused, ErrorCode::JobPaused);
    job.paused = false;
    job.paused_at = 0;
    Ok(())
}

pub fn expire_paused_job(ctx: Context<ExpirePausedJob>, _job_id: u64) -> Result<()> {
    let job = &mut ctx.accounts.job;
    require!(
        job.client == ctx.accounts.client.key(),
        ErrorCode::NotJobClient
    );
    require!(job.paused, ErrorCode::JobPaused);
    let now = Clock::get()?.unix_timestamp;
    require!(
        now.checked_sub(job.paused_at)
            .ok_or(ErrorCode::JobPausedExpired)?
            > MAX_PAUSE_DURATION,
        ErrorCode::JobPaused
    );
    cleanup_job_applications(job, &job.key(), 0, ctx.remaining_accounts, true, true)?;
    if job.status == JobStatus::Funded {
        let total = job
            .amount
            .checked_add(job.fee_amount)
            .ok_or(ErrorCode::MathOverflow)?;
        transfer_job_lamports(
            &job.to_account_info(),
            &ctx.accounts.client.to_account_info(),
            total,
        )?;
    }
    job.status = JobStatus::Cancelled;
    Ok(())
}

