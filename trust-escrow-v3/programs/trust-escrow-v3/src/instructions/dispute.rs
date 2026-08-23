#![allow(unused_imports)]
use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer, ID as SYSTEM_PROGRAM_ID};
use crate::errors::ErrorCode;
use crate::state::*;
use crate::{ARBITER_FEE_BPS_PER_PARTY, AUTO_APPROVAL_DELAY, BASIS_POINTS, DISPUTE_ACCEPT_GRACE, INITIAL_AUTHORITY, MAX_APPLICATIONS, MAX_ARBITERS, MAX_EVIDENCE_COUNT, MAX_MILESTONES, MAX_PAUSE_DURATION, MIN_JOB_AMOUNT, RemainingAccounts};
use crate::{assert_not_paused, check_not_paused, cleanup_job_applications, close_evidence_account, compute_fee, compute_shortfall, transfer_from_pda, transfer_job_lamports, validate_treasury_destination};

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct RaiseDispute<'info> {
    #[account(mut)]
    pub raiser: Signer<'info>,
    /// CHECK: client validado por el PDA del job.
    pub client: UncheckedAccount<'info>,
    #[account(mut, seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
    #[account(seeds = [b"support", job.key().as_ref()], bump)]
    pub ticket: Option<Account<'info, SupportTicket>>,
    #[account(init, payer = raiser, space = Dispute::INIT_SPACE + 8, seeds = [b"dispute", job.key().as_ref()], bump)]
    pub dispute: Account<'info, Dispute>,
    #[account(init, payer = raiser, space = ArbitrationEscrow::INIT_SPACE + 8, seeds = [b"arb_fee", job.key().as_ref()], bump)]
    pub escrow: Account<'info, ArbitrationEscrow>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct AcceptDispute<'info> {
    #[account(mut)]
    pub accepter: Signer<'info>,
    /// CHECK: client validado por el PDA del job.
    pub client: UncheckedAccount<'info>,
    #[account(seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
    #[account(mut, seeds = [b"dispute", job.key().as_ref()], bump = dispute.bump)]
    pub dispute: Account<'info, Dispute>,
    #[account(mut, seeds = [b"arb_fee", job.key().as_ref()], bump = escrow.bump)]
    pub escrow: Account<'info, ArbitrationEscrow>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(job_id: u64, index: u8)]
pub struct SubmitEvidence<'info> {
    #[account(mut)]
    pub submitter: Signer<'info>,
    /// CHECK: client validado por el PDA del job.
    pub client: UncheckedAccount<'info>,
    #[account(seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
    #[account(mut, seeds = [b"dispute", job.key().as_ref()], bump = dispute.bump)]
    pub dispute: Account<'info, Dispute>,
    #[account(
        init,
        payer = submitter,
        space = Evidence::INIT_SPACE + 8,
        seeds = [b"evidence", dispute.key().as_ref(), &[index]],
        bump
    )]
    pub evidence: Account<'info, Evidence>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct AssignArbiter<'info> {
    pub authority: Signer<'info>,
    /// CHECK: client validado por el PDA del job.
    pub client: UncheckedAccount<'info>,
    #[account(seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
    #[account(mut, seeds = [b"dispute", job.key().as_ref()], bump = dispute.bump)]
    pub dispute: Account<'info, Dispute>,
    #[account(seeds = [b"arbiter_pool"], bump = pool.bump)]
    pub pool: Account<'info, ArbiterPool>,
    /// CHECK: Arbitro a asignar (validado contra el pool).
    pub arbiter: UncheckedAccount<'info>,
    #[account(
        seeds = [b"config"],
        bump = config.bump,
        constraint = config.authority == authority.key() @ ErrorCode::NotAuthorized
    )]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct ResolveDispute<'info> {
    pub arbiter: Signer<'info>,
    /// CHECK: client validado por el PDA del job.
    pub client: UncheckedAccount<'info>,
    #[account(seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
    #[account(mut, seeds = [b"dispute", job.key().as_ref()], bump = dispute.bump)]
    pub dispute: Account<'info, Dispute>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct ResolvePlatformCase<'info> {
    pub advisor: Signer<'info>,
    /// CHECK: client validado por el PDA del job.
    pub client: UncheckedAccount<'info>,
    #[account(seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
    #[account(mut, seeds = [b"dispute", job.key().as_ref()], bump = dispute.bump)]
    pub dispute: Account<'info, Dispute>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct RequestPlatformIntervention<'info> {
    pub requester: Signer<'info>,
    /// CHECK: client validado por el PDA del job.
    pub client: UncheckedAccount<'info>,
    #[account(seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
    #[account(mut, seeds = [b"dispute", job.key().as_ref()], bump = dispute.bump)]
    pub dispute: Account<'info, Dispute>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct OpenSupportTicket<'info> {
    #[account(mut)]
    pub opener: Signer<'info>,
    /// CHECK: client validado por el PDA del job.
    pub client: UncheckedAccount<'info>,
    #[account(seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
    #[account(seeds = [b"dispute", job.key().as_ref()], bump)]
    pub dispute: Option<Account<'info, Dispute>>,
    #[account(
        init,
        payer = opener,
        space = SupportTicket::INIT_SPACE + 8,
        seeds = [b"support", job.key().as_ref()],
        bump
    )]
    pub ticket: Account<'info, SupportTicket>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(job_id: u64, remaining_metas: RemainingAccounts)]
pub struct ResolveSupportTicket<'info> {
    pub advisor: Signer<'info>,
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
    #[account(
        mut,
        seeds = [b"support", job.key().as_ref()],
        bump = ticket.bump,
        close = opener
    )]
    pub ticket: Account<'info, SupportTicket>,
    /// CHECK: quien abrio el ticket; validado contra ticket.opened_by.
    #[account(constraint = opener.key() == ticket.opened_by @ ErrorCode::NotAuthorized)]
    pub opener: UncheckedAccount<'info>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(job_id: u64, remaining_metas: RemainingAccounts)]
pub struct FinalizeDisputePayouts<'info> {
    pub resolver: Signer<'info>,
    /// CHECK: client validado por el PDA del job.
    #[account(constraint = client.owner == &SYSTEM_PROGRAM_ID @ ErrorCode::NotAuthorized)]
    pub client: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()],
        bump = job.bump,
        close = client
    )]
    pub job: Account<'info, Job>,
    #[account(
        mut,
        seeds = [b"dispute", job.key().as_ref()],
        bump = dispute.bump,
        close = client
    )]
    pub dispute: Account<'info, Dispute>,
    #[account(
        mut,
        seeds = [b"arb_fee", job.key().as_ref()],
        bump = escrow.bump,
        close = arbitration_treasury
    )]
    pub escrow: Account<'info, ArbitrationEscrow>,
    #[account(mut, constraint = job.freelancer == Some(freelancer.key()) @ ErrorCode::NotJobFreelancer)]
    pub freelancer: SystemAccount<'info>,
    /// CHECK: Treasury que recibe la comision de protocolo; validado contra config.treasury.
    #[account(
        mut,
        constraint = treasury.key() == config.treasury @ ErrorCode::InvalidTreasury,
        constraint = treasury.owner == &SYSTEM_PROGRAM_ID @ ErrorCode::InvalidTreasury
    )]
    pub treasury: UncheckedAccount<'info>,
    /// CHECK: Cuenta SEPARADA de la empresa que recibe las fees de arbitraje (5%).
    #[account(
        mut,
        constraint = arbitration_treasury.key() == config.arbitration_treasury @ ErrorCode::InvalidTreasury,
        constraint = arbitration_treasury.owner == &SYSTEM_PROGRAM_ID @ ErrorCode::InvalidTreasury
    )]
    pub arbitration_treasury: UncheckedAccount<'info>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(job_id: u64, remaining_metas: RemainingAccounts)]
pub struct CleanupDisputeEvidence<'info> {
    pub resolver: Signer<'info>,
    /// CHECK: client validado por el PDA del job y debe ser una cuenta System.
    #[account(constraint = client.owner == &SYSTEM_PROGRAM_ID @ ErrorCode::NotAuthorized)]
    pub client: UncheckedAccount<'info>,
    #[account(seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
    #[account(mut, seeds = [b"dispute", job.key().as_ref()], bump = dispute.bump)]
    pub dispute: Account<'info, Dispute>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

pub fn raise_dispute(ctx: Context<RaiseDispute>, _job_id: u64) -> Result<()> {
    assert_not_paused(&ctx.accounts.config)?;
    require!(
        ctx.accounts.job.status == JobStatus::Submitted
            || ctx.accounts.job.status == JobStatus::InProgress,
        ErrorCode::CannotDisputeAtStage
    );

    require!(ctx.accounts.ticket.is_none(), ErrorCode::CaseAlreadyOpen);
    let raiser = ctx.accounts.raiser.key();
    require!(
        raiser == ctx.accounts.job.client || ctx.accounts.job.freelancer == Some(raiser),
        ErrorCode::NotAuthorized
    );

    let now = Clock::get()?.unix_timestamp;
    let dispute_amount = ctx
        .accounts
        .job
        .amount
        .checked_sub(ctx.accounts.job.milestones_amount_total)
        .ok_or(ErrorCode::MathOverflow)?;
    let bond = compute_fee(dispute_amount, ARBITER_FEE_BPS_PER_PARTY)?;

    transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.raiser.to_account_info(),
                to: ctx.accounts.escrow.to_account_info(),
            },
        ),
        bond,
    )?;

    let escrow = &mut ctx.accounts.escrow;
    escrow.job = ctx.accounts.job.key();
    if raiser == ctx.accounts.job.client {
        escrow.client_bond = bond;
        escrow.freelancer_bond = 0;
    } else {
        escrow.freelancer_bond = bond;
        escrow.client_bond = 0;
    }
    escrow.bump = ctx.bumps.escrow;

    let dispute = &mut ctx.accounts.dispute;
    dispute.job = ctx.accounts.job.key();
    dispute.raised_by = raiser;
    dispute.arbiter = None;
    dispute.status = DisputeStatus::Open;
    dispute.evidence_count = 0;
    dispute.evidence_cleanup_cursor = 0;
    dispute.deadline = now
        .checked_add(DISPUTE_ACCEPT_GRACE)
        .ok_or(ErrorCode::MathOverflow)?;
    dispute.client_payout_percent = 0;
    dispute.freelancer_payout_percent = 0;
    dispute.bump = ctx.bumps.dispute;

    let job = &mut ctx.accounts.job;
    job.status = JobStatus::Disputed;

    msg!("Dispute raised for job: {}", job.key());
    Ok(())
}

pub fn accept_dispute(ctx: Context<AcceptDispute>, _job_id: u64) -> Result<()> {
    assert_not_paused(&ctx.accounts.config)?;
    let dispute = &mut ctx.accounts.dispute;
    require!(
        dispute.status == DisputeStatus::Open,
        ErrorCode::DisputeAlreadyResolved
    );
    let clock = Clock::get()?;
    require!(
        clock.unix_timestamp <= dispute.deadline,
        ErrorCode::DisputeDeadlinePassed
    );

    let accepter = ctx.accounts.accepter.key();
    require!(accepter != dispute.raised_by, ErrorCode::NotAuthorized);
    require!(
        accepter == ctx.accounts.job.client || ctx.accounts.job.freelancer == Some(accepter),
        ErrorCode::NotAuthorized
    );

    let dispute_amount = ctx
        .accounts
        .job
        .amount
        .checked_sub(ctx.accounts.job.milestones_amount_total)
        .ok_or(ErrorCode::MathOverflow)?;
    let bond = compute_fee(dispute_amount, ARBITER_FEE_BPS_PER_PARTY)?;
    transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.accepter.to_account_info(),
                to: ctx.accounts.escrow.to_account_info(),
            },
        ),
        bond,
    )?;

    let escrow = &mut ctx.accounts.escrow;
    if accepter == ctx.accounts.job.client {
        escrow.client_bond = escrow
            .client_bond
            .checked_add(bond)
            .ok_or(ErrorCode::MathOverflow)?;
    } else {
        escrow.freelancer_bond = escrow
            .freelancer_bond
            .checked_add(bond)
            .ok_or(ErrorCode::MathOverflow)?;
    }
    dispute.status = DisputeStatus::Active;

    msg!("Dispute accepted for job: {}", ctx.accounts.job.key());
    Ok(())
}

pub fn submit_evidence(
    ctx: Context<SubmitEvidence>,
    _job_id: u64,
    index: u8,
    content_hash: [u8; 32],
) -> Result<()> {
    assert_not_paused(&ctx.accounts.config)?;
    let dispute = &mut ctx.accounts.dispute;
    require!(
        dispute.status != DisputeStatus::Resolved && dispute.status != DisputeStatus::Expired,
        ErrorCode::DisputeAlreadyResolved
    );
    require!(
        dispute.evidence_count < MAX_EVIDENCE_COUNT,
        ErrorCode::EvidenceLimitReached
    );
    require!(
        index == dispute.evidence_count,
        ErrorCode::InvalidEvidenceIndex
    );

    let submitter = ctx.accounts.submitter.key();
    require!(
        submitter == ctx.accounts.job.client || ctx.accounts.job.freelancer == Some(submitter),
        ErrorCode::NotAuthorized
    );

    let evidence = &mut ctx.accounts.evidence;
    evidence.dispute = dispute.key();
    evidence.index = index;
    evidence.author = submitter;
    evidence.content_hash = content_hash;
    evidence.bump = ctx.bumps.evidence;
    dispute.evidence_count = dispute
        .evidence_count
        .checked_add(1)
        .ok_or(ErrorCode::MathOverflow)?;
    if dispute.status == DisputeStatus::Open || dispute.status == DisputeStatus::Active {
        dispute.status = DisputeStatus::EvidenceSubmitted;
    }
    Ok(())
}

pub fn assign_arbiter(ctx: Context<AssignArbiter>, _job_id: u64) -> Result<()> {
    assert_not_paused(&ctx.accounts.config)?;
    let pool = &ctx.accounts.pool;
    require!(
        pool.authority == ctx.accounts.config.authority,
        ErrorCode::InvalidAuthority
    );
    require!(
        pool.arbiters.contains(&ctx.accounts.arbiter.key()),
        ErrorCode::NotValidArbiter
    );
    let dispute = &mut ctx.accounts.dispute;
    require!(dispute.arbiter.is_none(), ErrorCode::NotValidArbiter);
    let job_client = ctx.accounts.job.client;
    let job_freelancer = ctx.accounts.job.freelancer;
    require!(
        ctx.accounts.arbiter.key() != job_client
            && job_freelancer.is_none_or(|f| f != ctx.accounts.arbiter.key()),
        ErrorCode::ArbiterCannotBeParty
    );
    require!(
        dispute.status == DisputeStatus::Active
            || dispute.status == DisputeStatus::EvidenceSubmitted,
        ErrorCode::InvalidJobStatus
    );
    dispute.arbiter = Some(ctx.accounts.arbiter.key());
    dispute.status = DisputeStatus::ArbiterAssigned;
    Ok(())
}

pub fn resolve_dispute(
    ctx: Context<ResolveDispute>,
    _job_id: u64,
    client_payout_percent: u8,
) -> Result<()> {
    assert_not_paused(&ctx.accounts.config)?;
    let dispute = &mut ctx.accounts.dispute;
    require!(
        dispute.arbiter == Some(ctx.accounts.arbiter.key()),
        ErrorCode::NotArbiter
    );
    require!(
        dispute.status == DisputeStatus::ArbiterAssigned,
        ErrorCode::DisputeAlreadyResolved
    );
    require!(client_payout_percent <= 100, ErrorCode::InvalidPercent);

    dispute.client_payout_percent = client_payout_percent;
    dispute.freelancer_payout_percent = 100 - client_payout_percent;
    dispute.status = DisputeStatus::Resolved;

    msg!(
        "Dispute resolved: {}% client, {}% freelancer",
        client_payout_percent,
        100 - client_payout_percent
    );
    Ok(())
}

pub fn resolve_platform_case(
    ctx: Context<ResolvePlatformCase>,
    _job_id: u64,
    client_payout_percent: u8,
) -> Result<()> {
    assert_not_paused(&ctx.accounts.config)?;
    let config = &ctx.accounts.config;
    require!(
        config.advisor == ctx.accounts.advisor.key(),
        ErrorCode::NotAuthorized
    );
    let job_client = ctx.accounts.job.client;
    let job_freelancer = ctx.accounts.job.freelancer;
    require!(
        ctx.accounts.advisor.key() != job_client
            && job_freelancer.is_none_or(|f| f != ctx.accounts.advisor.key()),
        ErrorCode::ArbiterCannotBeParty
    );
    let dispute = &mut ctx.accounts.dispute;
    let now = Clock::get()?.unix_timestamp;
    let expired = now > dispute.deadline;
    require!(
        dispute.status == DisputeStatus::ArbiterAssigned
            || (dispute.arbiter.is_none()
                && (dispute.status == DisputeStatus::Open
                    || dispute.status == DisputeStatus::Active
                    || dispute.status == DisputeStatus::EvidenceSubmitted)
                && expired),
        ErrorCode::NotArbiter
    );
    require!(
        dispute.status != DisputeStatus::Resolved && dispute.status != DisputeStatus::Expired,
        ErrorCode::DisputeAlreadyResolved
    );
    require!(client_payout_percent <= 100, ErrorCode::InvalidPercent);

    dispute.client_payout_percent = client_payout_percent;
    dispute.freelancer_payout_percent = 100 - client_payout_percent;
    dispute.status = DisputeStatus::Resolved;
    Ok(())
}

pub fn request_platform_intervention(
    ctx: Context<RequestPlatformIntervention>,
    _job_id: u64,
) -> Result<()> {
    assert_not_paused(&ctx.accounts.config)?;
    let dispute = &mut ctx.accounts.dispute;
    require!(
        dispute.status == DisputeStatus::Open,
        ErrorCode::DisputeAlreadyResolved
    );
    let clock = Clock::get()?;
    require!(
        clock.unix_timestamp <= dispute.deadline,
        ErrorCode::DisputeDeadlinePassed
    );
    let requester = ctx.accounts.requester.key();
    require!(
        requester == ctx.accounts.job.client || ctx.accounts.job.freelancer == Some(requester),
        ErrorCode::NotAuthorized
    );
    dispute.status = DisputeStatus::EvidenceSubmitted;
    Ok(())
}

pub fn open_support_ticket(ctx: Context<OpenSupportTicket>, _job_id: u64) -> Result<()> {
    assert_not_paused(&ctx.accounts.config)?;
    let job = &ctx.accounts.job;
    require!(
        job.status == JobStatus::InProgress || job.status == JobStatus::Submitted,
        ErrorCode::InvalidJobStatus
    );

    let opener = ctx.accounts.opener.key();
    require!(
        opener == job.client || job.freelancer == Some(opener),
        ErrorCode::NotAuthorized
    );

    require!(ctx.accounts.dispute.is_none(), ErrorCode::CaseAlreadyOpen);

    let ticket = &mut ctx.accounts.ticket;
    ticket.job = job.key();
    ticket.opened_by = opener;
    ticket.status = SupportTicketStatus::Open;
    ticket.bump = ctx.bumps.ticket;

    msg!("Support ticket opened for job: {}", job.key());
    Ok(())
}

pub fn resolve_support_ticket(ctx: Context<ResolveSupportTicket>, _job_id: u64, remaining_metas: RemainingAccounts) -> Result<()> {
    assert_not_paused(&ctx.accounts.config)?;
    let config = &ctx.accounts.config;
    require!(
        config.advisor == ctx.accounts.advisor.key(),
        ErrorCode::NotAuthorized
    );
    let job_client = ctx.accounts.job.client;
    let job_freelancer = ctx.accounts.job.freelancer;
    require!(
        ctx.accounts.advisor.key() != job_client
            && job_freelancer.is_none_or(|f| f != ctx.accounts.advisor.key()),
        ErrorCode::ArbiterCannotBeParty
    );
    let ticket = &mut ctx.accounts.ticket;
    require!(
        ticket.status == SupportTicketStatus::Open,
        ErrorCode::DisputeAlreadyResolved
    );

    let job = &mut ctx.accounts.job;
    require!(
        job.client == ctx.accounts.client.key(),
        ErrorCode::NotJobClient
    );
    require!(
        job.status == JobStatus::InProgress || job.status == JobStatus::Submitted,
        ErrorCode::InvalidJobStatus
    );

    if !ctx.remaining_accounts.is_empty() {
        cleanup_job_applications(job, &job.key(), 0, ctx.remaining_accounts, &remaining_metas, false, true)?;
    } else {
        require!(remaining_metas.metas.is_empty(), ErrorCode::InvalidApplicationCleanupAccounts);
    }

    let remaining_principal = job
        .amount
        .checked_sub(job.milestones_amount_total)
        .ok_or(ErrorCode::MathOverflow)?;
    let refund = remaining_principal
        .checked_add(job.fee_amount)
        .ok_or(ErrorCode::MathOverflow)?;
    let job_seeds: &[&[u8]] = &[b"job", job.client.as_ref(), &_job_id.to_le_bytes(), &[job.bump]];
    transfer_from_pda(
        &job.to_account_info(),
        &ctx.accounts.client.to_account_info(),
        refund,
        job_seeds)?;

    job.status = JobStatus::Cancelled;
    ticket.status = SupportTicketStatus::Resolved;

    msg!("Support ticket resolved (job cancelled): {}", job.key());
    Ok(())
}

pub fn finalize_dispute_payouts<'info>(
    ctx: Context<'_, '_, '_, 'info, FinalizeDisputePayouts<'info>>,
    _job_id: u64,
    remaining_metas: RemainingAccounts,
) -> Result<()> {
    assert_not_paused(&ctx.accounts.config)?;
    let dispute = &ctx.accounts.dispute;
    require!(
        dispute.status == DisputeStatus::Resolved,
        ErrorCode::DisputeAlreadyResolved
    );

    let resolver = ctx.accounts.resolver.key();
    require!(
        dispute.arbiter == Some(resolver) || ctx.accounts.config.advisor == resolver,
        ErrorCode::NotAuthorized
    );

    let job = &mut ctx.accounts.job;
    require!(
        job.client == ctx.accounts.client.key(),
        ErrorCode::NotJobClient
    );
    let expected_evidence = dispute
        .evidence_count
        .checked_sub(dispute.evidence_cleanup_cursor)
        .ok_or(ErrorCode::InvalidEvidenceCleanupAccounts)?;
    require!(
        ctx.remaining_accounts.len() >= expected_evidence as usize,
        ErrorCode::InvalidEvidenceCleanupAccounts
    );
    let (evidence_accounts, application_accounts) =
        ctx.remaining_accounts.split_at(expected_evidence as usize);
    require!(
        evidence_accounts.len() <= crate::MAX_EVIDENCE_CLEANUP_BATCH,
        ErrorCode::InvalidEvidenceCleanupAccounts
    );
    // P0-1: validate metas split accordingly
    let evidence_metas_len = evidence_accounts.len();
    let _app_metas_len = application_accounts.len();
    // Validate overall metas length matches remaining_accounts
    remaining_metas.validate_infos(ctx.remaining_accounts)?;
    let evidence_metas = RemainingAccounts { metas: remaining_metas.metas[..evidence_metas_len].to_vec() };
    let app_metas = RemainingAccounts { metas: remaining_metas.metas[evidence_metas_len..].to_vec() };
    crate::validate_evidence_remaining(&evidence_metas, evidence_accounts)?;
    if !application_accounts.is_empty() {
        cleanup_job_applications(job, &job.key(), 0, application_accounts, &app_metas, false, true)?;
    }

    let amount = job
        .amount
        .checked_sub(job.milestones_amount_total)
        .ok_or(ErrorCode::MathOverflow)?;
    let fee_amount = job.fee_amount;
    let client_pct = dispute.client_payout_percent;
    let freelancer_pct = dispute.freelancer_payout_percent;
    let resolver_fee_total = compute_fee(amount, ARBITER_FEE_BPS_PER_PARTY * 2)?;
    let posted = ctx
        .accounts
        .escrow
        .client_bond
        .checked_add(ctx.accounts.escrow.freelancer_bond)
        .ok_or(ErrorCode::MathOverflow)?;
    let shortfall = compute_shortfall(resolver_fee_total, posted);
    let to_parties = amount
        .checked_sub(shortfall)
        .ok_or(ErrorCode::MathOverflow)?;

    let job_seeds: &[&[u8]] = &[b"job", job.client.as_ref(), &_job_id.to_le_bytes(), &[job.bump]];
    transfer_from_pda(
        &job.to_account_info(),
        &ctx.accounts.treasury.to_account_info(),
        fee_amount,
        job_seeds)?;

    let client_net = (to_parties as u128 * client_pct as u128 / 100) as u64;
    if client_net > 0 {
        transfer_from_pda(
            &job.to_account_info(),
            &ctx.accounts.client.to_account_info(),
            client_net,
            job_seeds)?;
    }

    let freelancer_net = (to_parties as u128 * freelancer_pct as u128 / 100) as u64;
    if freelancer_net > 0 {
        transfer_from_pda(
            &job.to_account_info(),
            &ctx.accounts.freelancer.to_account_info(),
            freelancer_net,
            job_seeds)?;
    }

    if shortfall > 0 {
        transfer_from_pda(
            &job.to_account_info(),
            &ctx.accounts.arbitration_treasury.to_account_info(),
            shortfall,
            job_seeds)?;
    }

    require!(
        evidence_accounts.len() == expected_evidence as usize,
        ErrorCode::InvalidEvidenceCleanupAccounts
    );
    for (offset, evidence) in evidence_accounts.iter().enumerate() {
        let index = dispute.evidence_cleanup_cursor + offset as u8;
        close_evidence_account(
            evidence,
            &ctx.accounts.client.to_account_info(),
            &dispute.key(),
            index
        )?;
    }

    msg!("Dispute finalized for job: {}", job.key());
    Ok(())
}

pub fn cleanup_dispute_evidence<'info>(
    ctx: Context<'_, '_, '_, 'info, CleanupDisputeEvidence<'info>>,
    _job_id: u64,
    remaining_metas: RemainingAccounts,
) -> Result<()> {
    assert_not_paused(&ctx.accounts.config)?;
    let dispute = &mut ctx.accounts.dispute;
    require!(
        dispute.status == DisputeStatus::Resolved || dispute.status == DisputeStatus::Expired,
        ErrorCode::DisputeAlreadyResolved
    );
    let resolver = ctx.accounts.resolver.key();
    require!(
        dispute.arbiter == Some(resolver) || ctx.accounts.config.advisor == resolver,
        ErrorCode::NotAuthorized
    );
    require!(
        !ctx.remaining_accounts.is_empty(),
        ErrorCode::InvalidEvidenceCleanupAccounts
    );

    let remaining = dispute
        .evidence_count
        .checked_sub(dispute.evidence_cleanup_cursor)
        .ok_or(ErrorCode::InvalidEvidenceCleanupAccounts)?;
    require!(
        ctx.remaining_accounts.len() <= remaining as usize,
        ErrorCode::InvalidEvidenceCleanupAccounts
    );
    require!(
        ctx.remaining_accounts.len() <= crate::MAX_EVIDENCE_CLEANUP_BATCH,
        ErrorCode::InvalidEvidenceCleanupAccounts
    );
    remaining_metas.validate_infos(ctx.remaining_accounts)?;
    crate::validate_evidence_remaining(&remaining_metas, ctx.remaining_accounts)?;
    for (offset, evidence) in ctx.remaining_accounts.iter().enumerate() {
        let index = dispute.evidence_cleanup_cursor + offset as u8;
        close_evidence_account(
            evidence,
            &ctx.accounts.client.to_account_info(),
            &dispute.key(),
            index
        )?;
    }
    dispute.evidence_cleanup_cursor = dispute
        .evidence_cleanup_cursor
        .checked_add(ctx.remaining_accounts.len() as u8)
        .ok_or(ErrorCode::MathOverflow)?;
    Ok(())
}
