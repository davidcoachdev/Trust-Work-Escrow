use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

declare_id!("J1c4QsjbV9bFEPrFQZZGe8GrGWFxNhtAhhrxJFK2xc1h");


const BASIS_POINTS: u16 = 10_000;

const ARBITER_FEE_BPS_PER_PARTY: u16 = 250;

const DISPUTE_ACCEPT_GRACE: i64 = 7 * 24 * 60 * 60;

const MAX_PAUSE_DURATION: i64 = 30 * 24 * 60 * 60;

const MIN_JOB_AMOUNT: u64 = 100_000;
const MAX_TITLE_LENGTH: usize = 100;
const MAX_DESCRIPTION_LENGTH: usize = 500;
const MAX_PROPOSAL_LENGTH: usize = 512;
const MAX_DISPUTE_REASON: usize = 500;
const MAX_DISPUTE_EVIDENCE: usize = 2048;
const MAX_MILESTONE_TITLE: usize = 64;
const MAX_MILESTONES: usize = 20;
const MAX_APPLICATIONS: usize = 50;
const MAX_ARBITERS: usize = 50;


#[error_code]
pub enum ErrorCode {
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Program is paused")]
    ProgramPaused,
    #[msg("Amount too small")]
    AmountTooSmall,
    #[msg("Title cannot be empty")]
    EmptyTitle,
    #[msg("Title exceeds maximum length")]
    TitleTooLong,
    #[msg("Description exceeds maximum length")]
    DescriptionTooLong,
    #[msg("Proposal exceeds maximum length")]
    ProposalTooLong,
    #[msg("Invalid fee basis points (must be 0-10000)")]
    InvalidFeeBps,
    #[msg("Not authorized")]
    NotAuthorized,
    #[msg("Not authorized - not the job client")]
    NotJobClient,
    #[msg("Not authorized - not the job freelancer")]
    NotJobFreelancer,
    #[msg("Cannot work on your own job")]
    CannotWorkOnOwnJob,
    #[msg("Invalid job status for this operation")]
    InvalidJobStatus,
    #[msg("Job pausado")]
    JobPaused,
    #[msg("Job pausado demasiado tiempo; cancela el job")]
    JobPausedExpired,
    #[msg("No se puede pausar un job con freelancer asignado")]
    CannotPauseWithFreelancer,
    #[msg("No freelancer assigned")]
    NoFreelancerAssigned,
    #[msg("Treasury invalido (no coincide con config.treasury)")]
    InvalidTreasury,
    #[msg("Invalid job id / PDA mismatch")]
    InvalidJob,
    #[msg("Deadline must be in the future")]
    DeadlineMustBeFuture,
    #[msg("Insufficient funds in source account")]
    InsufficientFunds,
    #[msg("Cannot raise dispute at this stage")]
    CannotDisputeAtStage,
    #[msg("Dispute reason cannot be empty")]
    EmptyDisputeReason,
    #[msg("Evidence exceeds maximum length")]
    EvidenceTooLong,
    #[msg("Dispute already resolved")]
    DisputeAlreadyResolved,
    #[msg("Dispute deadline passed; only platform advisor can resolve")]
    DisputeDeadlinePassed,
    #[msg("Not a valid arbiter")]
    NotValidArbiter,
    #[msg("Not the assigned arbiter")]
    NotArbiter,
    #[msg("El arbitro no puede ser el cliente ni el freelancer")]
    ArbiterCannotBeParty,
    #[msg("Payout percent exceeds 100")]
    InvalidPercent,
    #[msg("Milestone not found")]
    MilestoneNotFound,
    #[msg("Milestone already completed")]
    MilestoneAlreadyCompleted,
    #[msg("Invalid milestone index (must be sequential: == milestones_total)")]
    InvalidMilestoneIndex,
    #[msg("Milestone amount exceeds remaining job funds")]
    MilestoneAmountExceedsFunds,
    #[msg("All milestones must be completed before release")]
    AllMilestonesRequired,
    #[msg("Already applied to this job")]
    AlreadyApplied,
    #[msg("Invalid application index")]
    InvalidApplicationIndex,
    #[msg("A dispute or support ticket is already open for this job")]
    CaseAlreadyOpen,
}


#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum JobStatus {
    Created,
    Funded,
    InProgress,
    Submitted,
    Released,
    Disputed,
    Resolved,
    Cancelled,
}

impl anchor_lang::Space for JobStatus {
    const INIT_SPACE: usize = 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ApplicationStatus {
    Pending,
    Accepted,
    Rejected,
    Withdrawn,
}

impl anchor_lang::Space for ApplicationStatus {
    const INIT_SPACE: usize = 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum DisputeStatus {
    Open,
    Active,
    EvidenceSubmitted,
    ArbiterAssigned,
    Resolved,
    Expired,
}

impl anchor_lang::Space for DisputeStatus {
    const INIT_SPACE: usize = 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum MilestoneStatus {
    Pending,
    Submitted,
    Approved,
    Rejected,
}

impl anchor_lang::Space for MilestoneStatus {
    const INIT_SPACE: usize = 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum SupportTicketStatus {
    Open,
    Resolved,
}


#[account]
#[derive(InitSpace)]
pub struct Config {
    pub authority: Pubkey,
    pub advisor: Pubkey,
    pub treasury: Pubkey,
    pub arbitration_treasury: Pubkey,
    pub fee_bps: u16,
    pub paused: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Job {
    pub client: Pubkey,
    pub freelancer: Option<Pubkey>,
    pub amount: u64,
    pub fee_amount: u64,
    pub status: JobStatus,
    pub paused: bool,
    pub paused_at: i64,
    #[max_len(MAX_TITLE_LENGTH)]
    pub title: String,
    #[max_len(MAX_DESCRIPTION_LENGTH)]
    pub description: String,
    pub deadline: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub submitted_at: Option<i64>,
    pub milestones_total: u8,
    pub milestones_approved: u8,
    pub milestones_amount_total: u64,
    #[max_len(MAX_APPLICATIONS)]
    pub applications: Vec<Application>,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub struct Application {
    pub applicant: Pubkey,
    #[max_len(MAX_PROPOSAL_LENGTH)]
    pub proposal: String,
    pub applied_at: i64,
    pub status: ApplicationStatus,
}

#[account]
#[derive(InitSpace)]
pub struct ArbiterPool {
    pub authority: Pubkey,
    #[max_len(MAX_ARBITERS)]
    pub arbiters: Vec<Pubkey>,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Dispute {
    pub job: Pubkey,
    pub raised_by: Pubkey,
    pub arbiter: Option<Pubkey>,
    pub status: DisputeStatus,
    #[max_len(10)]
    pub evidence: Vec<Evidence>,
    #[max_len(MAX_DISPUTE_REASON)]
    pub reason: String,
    pub created_at: i64,
    pub deadline: i64,
    pub resolved_at: Option<i64>,
    #[max_len(MAX_DISPUTE_REASON)]
    pub resolution: Option<String>,
    pub client_payout_percent: u8,
    pub freelancer_payout_percent: u8,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub struct Evidence {
    pub submitter: Pubkey,
    #[max_len(MAX_DISPUTE_EVIDENCE)]
    pub content: String,
    pub submitted_at: i64,
}

#[account]
#[derive(InitSpace)]
pub struct Milestone {
    pub job: Pubkey,
    #[max_len(MAX_MILESTONE_TITLE)]
    pub title: String,
    #[max_len(MAX_DESCRIPTION_LENGTH)]
    pub description: String,
    pub amount: u64,
    pub deadline: i64,
    pub status: MilestoneStatus,
    pub index: u8,
    pub submitted_at: Option<i64>,
    pub approved_at: Option<i64>,
    pub bump: u8,
    pub created_at: i64,
}

#[account]
#[derive(InitSpace)]
pub struct SupportTicket {
    pub job: Pubkey,
    pub opened_by: Pubkey,
    #[max_len(MAX_DISPUTE_REASON)]
    pub reason: String,
    pub status: SupportTicketStatus,
    pub created_at: i64,
    pub resolved_at: Option<i64>,
    #[max_len(MAX_DISPUTE_REASON)]
    pub resolution: Option<String>,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct ArbitrationEscrow {
    pub job: Pubkey,
    pub client_bond: u64,
    pub freelancer_bond: u64,
    pub bump: u8,
}


pub fn compute_fee(amount: u64, fee_bps: u16) -> Result<u64> {
    let fee = (amount as u128)
        .checked_mul(fee_bps as u128)
        .ok_or(ErrorCode::MathOverflow)?
        / BASIS_POINTS as u128;
    Ok(fee as u64)
}

pub fn check_not_paused(job: &Job) -> Result<()> {
    if job.paused {
        let now = Clock::get()?.unix_timestamp;
        if now.saturating_sub(job.paused_at) > MAX_PAUSE_DURATION {
            return err!(ErrorCode::JobPausedExpired);
        }
        return err!(ErrorCode::JobPaused);
    }
    Ok(())
}


#[program]
pub mod escrow {
    use super::*;


    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        advisor: Pubkey,
        treasury: Pubkey,
        arbitration_treasury: Pubkey,
        fee_bps: u16,
    ) -> Result<()> {
        require!(fee_bps <= BASIS_POINTS, ErrorCode::InvalidFeeBps);

        let config = &mut ctx.accounts.config;
        config.authority = ctx.accounts.authority.key();
        config.advisor = advisor;
        config.treasury = treasury;
        config.arbitration_treasury = arbitration_treasury;
        config.fee_bps = fee_bps;
        config.paused = false;
        config.bump = ctx.bumps.config;

        msg!("Config initialized by: {}", config.authority);
        Ok(())
    }

    pub fn pause(ctx: Context<Pause>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        require!(config.authority == ctx.accounts.authority.key(), ErrorCode::NotAuthorized);
        config.paused = true;
        msg!("Program paused");
        Ok(())
    }

    pub fn unpause(ctx: Context<Unpause>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        require!(config.authority == ctx.accounts.authority.key(), ErrorCode::NotAuthorized);
        config.paused = false;
        msg!("Program unpaused");
        Ok(())
    }

    pub fn update_treasury(ctx: Context<UpdateTreasury>, new_treasury: Pubkey) -> Result<()> {
        let config = &mut ctx.accounts.config;
        require!(config.authority == ctx.accounts.authority.key(), ErrorCode::NotAuthorized);
        config.treasury = new_treasury;
        msg!("Treasury updated");
        Ok(())
    }

    pub fn update_arbitration_treasury(
        ctx: Context<UpdateArbitrationTreasury>,
        new_arbitration_treasury: Pubkey,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        require!(config.authority == ctx.accounts.authority.key(), ErrorCode::NotAuthorized);
        config.arbitration_treasury = new_arbitration_treasury;
        msg!("Arbitration treasury updated");
        Ok(())
    }

    pub fn withdraw_treasury(ctx: Context<WithdrawTreasury>, amount: u64) -> Result<()> {
        require!(amount > 0, ErrorCode::AmountTooSmall);
        let balance = ctx.accounts.treasury.get_lamports();
        require!(balance >= amount, ErrorCode::InsufficientFunds);

        transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.treasury.to_account_info(),
                    to: ctx.accounts.destination.to_account_info(),
                },
            ),
            amount,
        )?;

        msg!("Treasury withdrew {} lamports", amount);
        Ok(())
    }

    pub fn withdraw_arbitration(
        ctx: Context<WithdrawArbitration>,
        amount: u64,
    ) -> Result<()> {
        require!(amount > 0, ErrorCode::AmountTooSmall);
        let balance = ctx.accounts.arbitration_treasury.get_lamports();
        require!(balance >= amount, ErrorCode::InsufficientFunds);

        transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.arbitration_treasury.to_account_info(),
                    to: ctx.accounts.destination.to_account_info(),
                },
            ),
            amount,
        )?;

        msg!("Arbitration treasury withdrew {} lamports", amount);
        Ok(())
    }


    pub fn create_job(
        ctx: Context<CreateJob>,
        _job_id: u64,
        title: String,
        description: String,
        amount: u64,
        deadline: i64,
    ) -> Result<()> {
        let config = &ctx.accounts.config;
        require!(!config.paused, ErrorCode::ProgramPaused);
        require!(amount >= MIN_JOB_AMOUNT, ErrorCode::AmountTooSmall);
        require!(!title.is_empty(), ErrorCode::EmptyTitle);
        require!(title.len() <= MAX_TITLE_LENGTH, ErrorCode::TitleTooLong);
        require!(
            description.len() <= MAX_DESCRIPTION_LENGTH,
            ErrorCode::DescriptionTooLong
        );
        require!(
            deadline > Clock::get()?.unix_timestamp,
            ErrorCode::DeadlineMustBeFuture
        );

        let fee_amount = compute_fee(amount, config.fee_bps)?;

        let now = Clock::get()?.unix_timestamp;
        let job = &mut ctx.accounts.job;
        job.client = ctx.accounts.client.key();
        job.freelancer = None;
        job.amount = amount;
        job.fee_amount = fee_amount;
        job.status = JobStatus::Created;
        job.title = title;
        job.description = description;
        job.deadline = deadline;
        job.created_at = now;
        job.updated_at = now;
        job.submitted_at = None;
        job.milestones_total = 0;
        job.milestones_approved = 0;
        job.milestones_amount_total = 0;
        job.applications = Vec::new();
        job.bump = ctx.bumps.job;

        msg!("Job created: {} - amount {}, fee {}", job.key(), amount, fee_amount);
        Ok(())
    }

    pub fn deposit_funds(ctx: Context<DepositFunds>, _job_id: u64) -> Result<()> {
        let job = &mut ctx.accounts.job;
        require!(job.status == JobStatus::Created, ErrorCode::InvalidJobStatus);
        require!(job.client == ctx.accounts.client.key(), ErrorCode::NotJobClient);
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
        job.updated_at = Clock::get()?.unix_timestamp;

        msg!("Funds deposited: {}", total);
        Ok(())
    }

    pub fn apply_to_job(
        ctx: Context<ApplyToJob>,
        _job_id: u64,
        proposal: String,
    ) -> Result<()> {
        let job = &mut ctx.accounts.job;
        require!(job.status == JobStatus::Funded, ErrorCode::InvalidJobStatus);
        check_not_paused(job)?;
        require!(
            ctx.accounts.applicant.key() != job.client,
            ErrorCode::CannotWorkOnOwnJob
        );
        require!(proposal.len() <= MAX_PROPOSAL_LENGTH, ErrorCode::ProposalTooLong);
        require!(
            job.applications.len() < MAX_APPLICATIONS,
            ErrorCode::InvalidApplicationIndex
        );
        require!(
            !job
                .applications
                .iter()
                .any(|a| a.applicant == ctx.accounts.applicant.key()),
            ErrorCode::AlreadyApplied
        );

        let now = Clock::get()?.unix_timestamp;
        let new_index = job.applications.len() as u8;
        job.applications.push(Application {
            applicant: ctx.accounts.applicant.key(),
            proposal,
            applied_at: now,
            status: ApplicationStatus::Pending,
        });
        job.updated_at = now;

        msg!("Application {} submitted for job: {}", new_index, job.key());
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
        require!(job.client == ctx.accounts.client.key(), ErrorCode::NotJobClient);

        let idx = application_index as usize;
        require!(idx < job.applications.len(), ErrorCode::InvalidApplicationIndex);
        let applicant = job.applications[idx].applicant;
        job.applications[idx].status = ApplicationStatus::Accepted;

        job.freelancer = Some(applicant);
        job.status = JobStatus::InProgress;
        job.updated_at = Clock::get()?.unix_timestamp;

        msg!("Application accepted: freelancer {}", applicant);
        Ok(())
    }

    pub fn submit_work(ctx: Context<SubmitWork>, _job_id: u64) -> Result<()> {
        let job = &mut ctx.accounts.job;
        require!(
            job.freelancer == Some(ctx.accounts.freelancer.key()),
            ErrorCode::NotJobFreelancer
        );
        require!(job.status == JobStatus::InProgress, ErrorCode::InvalidJobStatus);

        job.status = JobStatus::Submitted;
        job.submitted_at = Some(Clock::get()?.unix_timestamp);
        job.updated_at = Clock::get()?.unix_timestamp;

        msg!("Work submitted for job: {}", job.key());
        Ok(())
    }

    pub fn approve_work(ctx: Context<ApproveWork>, _job_id: u64) -> Result<()> {
        let job = &mut ctx.accounts.job;
        require!(job.client == ctx.accounts.client.key(), ErrorCode::NotJobClient);
        require!(job.status == JobStatus::Submitted, ErrorCode::InvalidJobStatus);
        require!(job.freelancer.is_some(), ErrorCode::NoFreelancerAssigned);
        require!(
            job.milestones_total == 0 || job.milestones_approved == job.milestones_total,
            ErrorCode::AllMilestonesRequired
        );

        let amount = job
            .amount
            .checked_sub(job.milestones_amount_total)
            .ok_or(ErrorCode::MathOverflow)?;
        let fee_amount = job.fee_amount;

        let client_key = ctx.accounts.client.key();
        let job_id_bytes = _job_id.to_le_bytes();
        let seeds: &[&[&[u8]]] = &[&[
            b"job".as_ref(),
            client_key.as_ref(),
            job_id_bytes.as_ref(),
            &[job.bump],
        ]];

        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                Transfer {
                    from: job.to_account_info(),
                    to: ctx.accounts.freelancer.to_account_info(),
                },
                seeds,
            ),
            amount,
        )?;

        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                Transfer {
                    from: job.to_account_info(),
                    to: ctx.accounts.treasury.to_account_info(),
                },
                seeds,
            ),
            fee_amount,
        )?;

        job.status = JobStatus::Released;
        job.updated_at = Clock::get()?.unix_timestamp;

        msg!("Work approved: {} to freelancer, {} fee to treasury", amount, fee_amount);
        Ok(())
    }

    pub fn reject_work(ctx: Context<RejectWork>, _job_id: u64, _reason: String) -> Result<()> {
        let job = &mut ctx.accounts.job;
        require!(job.client == ctx.accounts.client.key(), ErrorCode::NotJobClient);
        require!(job.status == JobStatus::Submitted, ErrorCode::InvalidJobStatus);

        job.status = JobStatus::InProgress;
        job.updated_at = Clock::get()?.unix_timestamp;

        msg!("Work rejected, returned to InProgress: {}", job.key());
        Ok(())
    }

    pub fn cancel_job(ctx: Context<CancelJob>, _job_id: u64) -> Result<()> {
        let job = &mut ctx.accounts.job;
        require!(job.client == ctx.accounts.client.key(), ErrorCode::NotJobClient);
        require!(
            job.status == JobStatus::Created || job.status == JobStatus::Funded,
            ErrorCode::InvalidJobStatus
        );

        if job.status == JobStatus::Funded {
            let total = job
                .amount
                .checked_add(job.fee_amount)
                .ok_or(ErrorCode::MathOverflow)?;

            let client_key = ctx.accounts.client.key();
            let job_id_bytes = _job_id.to_le_bytes();
            let seeds: &[&[&[u8]]] = &[&[
                b"job".as_ref(),
                client_key.as_ref(),
                job_id_bytes.as_ref(),
                &[job.bump],
            ]];

            transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.system_program.to_account_info(),
                    Transfer {
                        from: job.to_account_info(),
                        to: ctx.accounts.client.to_account_info(),
                    },
                    seeds,
                ),
                total,
            )?;
        }

        job.status = JobStatus::Cancelled;
        job.updated_at = Clock::get()?.unix_timestamp;

        msg!("Job cancelled: {}", job.key());
        Ok(())
    }

    pub fn pause_job(ctx: Context<PauseJob>, _job_id: u64) -> Result<()> {
        let job = &mut ctx.accounts.job;
        require!(job.client == ctx.accounts.client.key(), ErrorCode::NotJobClient);
        require!(
            job.status == JobStatus::Created || job.status == JobStatus::Funded,
            ErrorCode::CannotPauseWithFreelancer
        );
        require!(!job.paused, ErrorCode::JobPaused);
        let now = Clock::get()?.unix_timestamp;
        job.paused = true;
        job.paused_at = now;
        job.updated_at = now;
        Ok(())
    }

    pub fn unpause_job(ctx: Context<UnpauseJob>, _job_id: u64) -> Result<()> {
        let job = &mut ctx.accounts.job;
        require!(job.client == ctx.accounts.client.key(), ErrorCode::NotJobClient);
        require!(job.paused, ErrorCode::JobPaused);
        job.paused = false;
        job.paused_at = 0;
        job.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn expire_paused_job(ctx: Context<ExpirePausedJob>, _job_id: u64) -> Result<()> {
        let job = &mut ctx.accounts.job;
        require!(job.paused, ErrorCode::JobPaused);
        let now = Clock::get()?.unix_timestamp;
        require!(
            now.saturating_sub(job.paused_at) > MAX_PAUSE_DURATION,
            ErrorCode::JobPaused
        );
        if job.status == JobStatus::Funded {
            let total = job
                .amount
                .checked_add(job.fee_amount)
                .ok_or(ErrorCode::MathOverflow)?;
            let client_key = ctx.accounts.client.key();
            let job_id_bytes = _job_id.to_le_bytes();
            let seeds: &[&[&[u8]]] = &[&[
                b"job".as_ref(),
                client_key.as_ref(),
                job_id_bytes.as_ref(),
                &[job.bump],
            ]];
            transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.system_program.to_account_info(),
                    Transfer {
                        from: job.to_account_info(),
                        to: ctx.accounts.client.to_account_info(),
                    },
                    seeds,
                ),
                total,
            )?;
        }
        job.status = JobStatus::Cancelled;
        job.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }


    pub fn create_arbiter_pool(ctx: Context<CreateArbiterPool>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.authority = ctx.accounts.authority.key();
        pool.arbiters = Vec::new();
        pool.bump = ctx.bumps.pool;
        Ok(())
    }

    pub fn add_arbiter(ctx: Context<AddArbiter>, new_arbiter: Pubkey) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        require!(pool.authority == ctx.accounts.authority.key(), ErrorCode::NotAuthorized);
        require!(!pool.arbiters.contains(&new_arbiter), ErrorCode::NotValidArbiter);
        require!(pool.arbiters.len() < MAX_ARBITERS, ErrorCode::NotValidArbiter);
        pool.arbiters.push(new_arbiter);
        Ok(())
    }

    pub fn remove_arbiter(ctx: Context<RemoveArbiter>, arbiter: Pubkey) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        require!(pool.authority == ctx.accounts.authority.key(), ErrorCode::NotAuthorized);
        let idx = pool
            .arbiters
            .iter()
            .position(|&a| a == arbiter)
            .ok_or(ErrorCode::NotValidArbiter)?;
        pool.arbiters.remove(idx);
        Ok(())
    }


    pub fn raise_dispute(ctx: Context<RaiseDispute>, _job_id: u64, reason: String) -> Result<()> {
        require!(
            ctx.accounts.job.status == JobStatus::Submitted
                || ctx.accounts.job.status == JobStatus::InProgress,
            ErrorCode::CannotDisputeAtStage
        );
        require!(!reason.is_empty(), ErrorCode::EmptyDisputeReason);
        require!(ctx.accounts.ticket.is_none(), ErrorCode::CaseAlreadyOpen);
        let raiser = ctx.accounts.raiser.key();
        require!(
            raiser == ctx.accounts.job.client
                || ctx.accounts.job.freelancer == Some(raiser),
            ErrorCode::NotAuthorized
        );

        let now = Clock::get()?.unix_timestamp;
        let dispute_amount = ctx.accounts.job.amount
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
        dispute.evidence = Vec::new();
        dispute.reason = reason;
        dispute.created_at = now;
        dispute.deadline = now.checked_add(DISPUTE_ACCEPT_GRACE).ok_or(ErrorCode::MathOverflow)?;
        dispute.resolved_at = None;
        dispute.resolution = None;
        dispute.client_payout_percent = 0;
        dispute.freelancer_payout_percent = 0;
        dispute.bump = ctx.bumps.dispute;

        let job = &mut ctx.accounts.job;
        job.status = JobStatus::Disputed;
        job.updated_at = now;

        msg!("Dispute raised for job: {}", job.key());
        Ok(())
    }

    pub fn accept_dispute(ctx: Context<AcceptDispute>, _job_id: u64) -> Result<()> {
        let dispute = &mut ctx.accounts.dispute;
        require!(dispute.status == DisputeStatus::Open, ErrorCode::DisputeAlreadyResolved);
        require!(
            Clock::get()?.unix_timestamp <= dispute.deadline,
            ErrorCode::DisputeDeadlinePassed
        );

        let accepter = ctx.accounts.accepter.key();
        require!(accepter != dispute.raised_by, ErrorCode::NotAuthorized);
        require!(
            accepter == ctx.accounts.job.client
                || ctx.accounts.job.freelancer == Some(accepter),
            ErrorCode::NotAuthorized
        );

        let dispute_amount = ctx.accounts.job.amount
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

    pub fn submit_evidence(ctx: Context<SubmitEvidence>, _job_id: u64, content: String) -> Result<()> {
        let dispute = &mut ctx.accounts.dispute;
        require!(
            dispute.status != DisputeStatus::Resolved && dispute.status != DisputeStatus::Expired,
            ErrorCode::DisputeAlreadyResolved
        );
        require!(content.len() <= MAX_DISPUTE_EVIDENCE, ErrorCode::EvidenceTooLong);
        let submitter = ctx.accounts.submitter.key();
        require!(
            submitter == ctx.accounts.job.client
                || ctx.accounts.job.freelancer == Some(submitter),
            ErrorCode::NotAuthorized
        );

        dispute.evidence.push(Evidence {
            submitter,
            content,
            submitted_at: Clock::get()?.unix_timestamp,
        });
        if dispute.status == DisputeStatus::Open || dispute.status == DisputeStatus::Active {
            dispute.status = DisputeStatus::EvidenceSubmitted;
        }
        Ok(())
    }

    pub fn assign_arbiter(ctx: Context<AssignArbiter>, _job_id: u64) -> Result<()> {
        require!(
            ctx.accounts.config.authority == ctx.accounts.authority.key(),
            ErrorCode::NotAuthorized
        );
        let pool = &ctx.accounts.pool;
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
                && job_freelancer.map_or(true, |f| f != ctx.accounts.arbiter.key()),
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
        let dispute = &mut ctx.accounts.dispute;
        require!(
            dispute.arbiter == Some(ctx.accounts.arbiter.key()),
            ErrorCode::NotArbiter
        );
        require!(dispute.status == DisputeStatus::ArbiterAssigned, ErrorCode::DisputeAlreadyResolved);
        require!(client_payout_percent <= 100, ErrorCode::InvalidPercent);

        dispute.client_payout_percent = client_payout_percent;
        dispute.freelancer_payout_percent = 100 - client_payout_percent;
        dispute.status = DisputeStatus::Resolved;
        dispute.resolved_at = Some(Clock::get()?.unix_timestamp);

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
        let config = &ctx.accounts.config;
        require!(config.advisor == ctx.accounts.advisor.key(), ErrorCode::NotAuthorized);
        let job_client = ctx.accounts.job.client;
        let job_freelancer = ctx.accounts.job.freelancer;
        require!(
            ctx.accounts.advisor.key() != job_client
                && job_freelancer.map_or(true, |f| f != ctx.accounts.advisor.key()),
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
        dispute.resolved_at = Some(Clock::get()?.unix_timestamp);
        Ok(())
    }

    pub fn request_platform_intervention(
        ctx: Context<RequestPlatformIntervention>,
        _job_id: u64,
    ) -> Result<()> {
        let dispute = &mut ctx.accounts.dispute;
        require!(dispute.status == DisputeStatus::Open, ErrorCode::DisputeAlreadyResolved);
        require!(
            Clock::get()?.unix_timestamp <= dispute.deadline,
            ErrorCode::DisputeDeadlinePassed
        );
        let requester = ctx.accounts.requester.key();
        require!(
            requester == ctx.accounts.job.client
                || ctx.accounts.job.freelancer == Some(requester),
            ErrorCode::NotAuthorized
        );
        dispute.status = DisputeStatus::EvidenceSubmitted;
        Ok(())
    }

    pub fn open_support_ticket(
        ctx: Context<OpenSupportTicket>,
        _job_id: u64,
        reason: String,
    ) -> Result<()> {
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
        require!(!reason.is_empty(), ErrorCode::EmptyDisputeReason);
        require!(ctx.accounts.dispute.is_none(), ErrorCode::CaseAlreadyOpen);

        let ticket = &mut ctx.accounts.ticket;
        ticket.job = job.key();
        ticket.opened_by = opener;
        ticket.reason = reason;
        ticket.status = SupportTicketStatus::Open;
        ticket.created_at = Clock::get()?.unix_timestamp;
        ticket.resolved_at = None;
        ticket.resolution = None;
        ticket.bump = ctx.bumps.ticket;

        msg!("Support ticket opened for job: {}", job.key());
        Ok(())
    }

    pub fn resolve_support_ticket(
        ctx: Context<ResolveSupportTicket>,
        _job_id: u64,
        resolution: String,
    ) -> Result<()> {
        let config = &ctx.accounts.config;
        require!(config.advisor == ctx.accounts.advisor.key(), ErrorCode::NotAuthorized);
        let job_client = ctx.accounts.job.client;
        let job_freelancer = ctx.accounts.job.freelancer;
        require!(
            ctx.accounts.advisor.key() != job_client
                && job_freelancer.map_or(true, |f| f != ctx.accounts.advisor.key()),
            ErrorCode::ArbiterCannotBeParty
        );
        let ticket = &mut ctx.accounts.ticket;
        require!(ticket.status == SupportTicketStatus::Open, ErrorCode::DisputeAlreadyResolved);
        let job = &mut ctx.accounts.job;
        require!(
            job.status == JobStatus::InProgress || job.status == JobStatus::Submitted,
            ErrorCode::InvalidJobStatus
        );

        job.status = JobStatus::Cancelled;
        job.updated_at = Clock::get()?.unix_timestamp;
        ticket.status = SupportTicketStatus::Resolved;
        ticket.resolved_at = Some(Clock::get()?.unix_timestamp);
        ticket.resolution = Some(resolution);

        msg!("Support ticket resolved (job cancelled): {}", job.key());
        Ok(())
    }

    pub fn finalize_dispute_payouts(
        ctx: Context<FinalizeDisputePayouts>,
        _job_id: u64,
    ) -> Result<()> {
        let dispute = &ctx.accounts.dispute;
        require!(dispute.status == DisputeStatus::Resolved, ErrorCode::DisputeAlreadyResolved);

        let resolver = ctx.accounts.resolver.key();
        require!(
            dispute.arbiter == Some(resolver) || ctx.accounts.config.advisor == resolver,
            ErrorCode::NotAuthorized
        );

        let job = &mut ctx.accounts.job;
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
        let shortfall = resolver_fee_total.saturating_sub(posted);
        let to_parties = amount
            .checked_sub(shortfall)
            .ok_or(ErrorCode::MathOverflow)?;

        let client_key = ctx.accounts.client.key();
        let job_id_bytes = _job_id.to_le_bytes();
        let seeds: &[&[&[u8]]] = &[&[
            b"job".as_ref(),
            client_key.as_ref(),
            job_id_bytes.as_ref(),
            &[job.bump],
        ]];

        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                Transfer {
                    from: job.to_account_info(),
                    to: ctx.accounts.treasury.to_account_info(),
                },
                seeds,
            ),
            fee_amount,
        )?;

        let client_net = (to_parties as u128 * client_pct as u128 / 100) as u64;
        if client_net > 0 {
            transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.system_program.to_account_info(),
                    Transfer {
                        from: job.to_account_info(),
                        to: ctx.accounts.client.to_account_info(),
                    },
                    seeds,
                ),
                client_net,
            )?;
        }

        let freelancer_net = (to_parties as u128 * freelancer_pct as u128 / 100) as u64;
        if freelancer_net > 0 {
            transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.system_program.to_account_info(),
                    Transfer {
                        from: job.to_account_info(),
                        to: ctx.accounts.freelancer.to_account_info(),
                    },
                    seeds,
                ),
                freelancer_net,
            )?;
        }

        if shortfall > 0 {
            transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.system_program.to_account_info(),
                    Transfer {
                        from: job.to_account_info(),
                        to: ctx.accounts.arbitration_treasury.to_account_info(),
                    },
                    seeds,
                ),
                shortfall,
            )?;
        }

        msg!("Dispute finalized for job: {}", job.key());
        Ok(())
    }


    pub fn create_milestone(
        ctx: Context<CreateMilestone>,
        _job_id: u64,
        index: u8,
        title: String,
        description: String,
        amount: u64,
        deadline: i64,
    ) -> Result<()> {
        let job = &mut ctx.accounts.job;
        require!(job.status == JobStatus::InProgress, ErrorCode::InvalidJobStatus);
        require!(!title.is_empty(), ErrorCode::EmptyTitle);
        require!(title.len() <= MAX_MILESTONE_TITLE, ErrorCode::TitleTooLong);
        require!(
            description.len() <= MAX_DESCRIPTION_LENGTH,
            ErrorCode::DescriptionTooLong
        );
        require!(
            deadline > Clock::get()?.unix_timestamp,
            ErrorCode::DeadlineMustBeFuture
        );
        require!(index == job.milestones_total, ErrorCode::InvalidMilestoneIndex);
        require!(
            job.milestones_total < MAX_MILESTONES as u8,
            ErrorCode::MilestoneAlreadyCompleted
        );

        let new_total = job
            .milestones_amount_total
            .checked_add(amount)
            .ok_or(ErrorCode::MathOverflow)?;
        require!(new_total <= job.amount, ErrorCode::MilestoneAmountExceedsFunds);

        let milestone = &mut ctx.accounts.milestone;
        milestone.job = job.key();
        milestone.title = title;
        milestone.description = description;
        milestone.amount = amount;
        milestone.deadline = deadline;
        milestone.status = MilestoneStatus::Pending;
        milestone.index = index;
        milestone.submitted_at = None;
        milestone.approved_at = None;
        milestone.bump = ctx.bumps.milestone;
        milestone.created_at = Clock::get()?.unix_timestamp;

        job.milestones_total = job.milestones_total.checked_add(1).ok_or(ErrorCode::MathOverflow)?;
        job.milestones_amount_total = new_total;
        job.updated_at = Clock::get()?.unix_timestamp;

        Ok(())
    }

    pub fn submit_milestone(ctx: Context<SubmitMilestone>, _job_id: u64, _milestone_index: u8) -> Result<()> {
        let job = &ctx.accounts.job;
        let milestone = &mut ctx.accounts.milestone;
        require!(
            job.freelancer == Some(ctx.accounts.freelancer.key()),
            ErrorCode::NotJobFreelancer
        );
        require!(job.status == JobStatus::InProgress, ErrorCode::InvalidJobStatus);
        require!(
            milestone.status == MilestoneStatus::Pending
                || milestone.status == MilestoneStatus::Rejected,
            ErrorCode::MilestoneAlreadyCompleted
        );

        milestone.status = MilestoneStatus::Submitted;
        milestone.submitted_at = Some(Clock::get()?.unix_timestamp);
        Ok(())
    }

    pub fn approve_milestone(ctx: Context<ApproveMilestone>, _job_id: u64, _milestone_index: u8) -> Result<()> {
        let job = &mut ctx.accounts.job;
        require!(job.client == ctx.accounts.client.key(), ErrorCode::NotJobClient);
        require!(job.status == JobStatus::InProgress, ErrorCode::InvalidJobStatus);
        let milestone = &mut ctx.accounts.milestone;
        require!(milestone.status == MilestoneStatus::Submitted, ErrorCode::MilestoneAlreadyCompleted);

        let amount = milestone.amount;

        let client_key = ctx.accounts.client.key();
        let job_id_bytes = _job_id.to_le_bytes();
        let seeds: &[&[&[u8]]] = &[&[
            b"job".as_ref(),
            client_key.as_ref(),
            job_id_bytes.as_ref(),
            &[job.bump],
        ]];

        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                Transfer {
                    from: job.to_account_info(),
                    to: ctx.accounts.freelancer.to_account_info(),
                },
                seeds,
            ),
            amount,
        )?;

        job.milestones_approved = job.milestones_approved.checked_add(1).ok_or(ErrorCode::MathOverflow)?;
        milestone.status = MilestoneStatus::Approved;
        milestone.approved_at = Some(Clock::get()?.unix_timestamp);
        job.updated_at = Clock::get()?.unix_timestamp;

        Ok(())
    }

    pub fn reject_milestone(ctx: Context<RejectMilestone>, _job_id: u64, _milestone_index: u8) -> Result<()> {
        let job = &ctx.accounts.job;
        let milestone = &mut ctx.accounts.milestone;
        require!(job.client == ctx.accounts.client.key(), ErrorCode::NotJobClient);
        require!(job.status == JobStatus::InProgress, ErrorCode::InvalidJobStatus);
        require!(milestone.status == MilestoneStatus::Submitted, ErrorCode::MilestoneAlreadyCompleted);

        milestone.status = MilestoneStatus::Rejected;
        Ok(())
    }

}


#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: Treasury wallet que recibe fees. Almacenada en config.
    pub treasury: UncheckedAccount<'info>,
    #[account(
        init,
        payer = authority,
        space = Config::INIT_SPACE + 8,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Pause<'info> {
    pub authority: Signer<'info>,
    #[account(mut, seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
pub struct Unpause<'info> {
    pub authority: Signer<'info>,
    #[account(mut, seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
pub struct UpdateTreasury<'info> {
    pub authority: Signer<'info>,
    #[account(mut, seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
pub struct UpdateArbitrationTreasury<'info> {
    pub authority: Signer<'info>,
    #[account(mut, seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
pub struct WithdrawTreasury<'info> {
    #[account(
        mut,
        constraint = treasury.key() == config.treasury @ ErrorCode::NotAuthorized
    )]
    pub treasury: Signer<'info>,
    /// CHECK: Destino libre; lo decide el tesorero.
    #[account(mut)]
    pub destination: UncheckedAccount<'info>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawArbitration<'info> {
    #[account(
        mut,
        constraint = arbitration_treasury.key() == config.arbitration_treasury @ ErrorCode::NotAuthorized
    )]
    pub arbitration_treasury: Signer<'info>,
    /// CHECK: Destino libre; lo decide la empresa.
    #[account(mut)]
    pub destination: UncheckedAccount<'info>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}


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
        constraint = treasury.key() == config.treasury @ ErrorCode::InvalidTreasury
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


#[derive(Accounts)]
#[instruction(job_id: u64)]
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
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct AcceptApplication<'info> {
    #[account(mut)]
    pub client: Signer<'info>,
    #[account(
        mut,
        seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()],
        bump = job.bump
    )]
    pub job: Account<'info, Job>,
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
pub struct CreateArbiterPool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init, payer = authority, space = ArbiterPool::INIT_SPACE + 8, seeds = [b"arbiter_pool"], bump)]
    pub pool: Account<'info, ArbiterPool>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddArbiter<'info> {
    pub authority: Signer<'info>,
    #[account(mut, seeds = [b"arbiter_pool"], bump = pool.bump)]
    pub pool: Account<'info, ArbiterPool>,
}

#[derive(Accounts)]
pub struct RemoveArbiter<'info> {
    pub authority: Signer<'info>,
    #[account(mut, seeds = [b"arbiter_pool"], bump = pool.bump)]
    pub pool: Account<'info, ArbiterPool>,
}


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
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct AcceptDispute<'info> {
    pub accepter: Signer<'info>,
    /// CHECK: client validado por el PDA del job.
    pub client: UncheckedAccount<'info>,
    #[account(seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
    #[account(mut, seeds = [b"dispute", job.key().as_ref()], bump = dispute.bump)]
    pub dispute: Account<'info, Dispute>,
    #[account(mut, seeds = [b"arb_fee", job.key().as_ref()], bump = escrow.bump)]
    pub escrow: Account<'info, ArbitrationEscrow>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct SubmitEvidence<'info> {
    pub submitter: Signer<'info>,
    /// CHECK: client validado por el PDA del job.
    pub client: UncheckedAccount<'info>,
    #[account(seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
    #[account(mut, seeds = [b"dispute", job.key().as_ref()], bump = dispute.bump)]
    pub dispute: Account<'info, Dispute>,
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
    #[account(seeds = [b"config"], bump = config.bump)]
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
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct ResolveSupportTicket<'info> {
    pub advisor: Signer<'info>,
    /// CHECK: client validado por el PDA del job.
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
#[instruction(job_id: u64)]
pub struct FinalizeDisputePayouts<'info> {
    pub resolver: Signer<'info>,
    /// CHECK: client validado por el PDA del job.
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
        constraint = treasury.key() == config.treasury @ ErrorCode::InvalidTreasury
    )]
    pub treasury: UncheckedAccount<'info>,
    /// CHECK: Cuenta SEPARADA de la empresa que recibe las fees de arbitraje (5%).
    #[account(
        mut,
        constraint = arbitration_treasury.key() == config.arbitration_treasury @ ErrorCode::InvalidTreasury
    )]
    pub arbitration_treasury: UncheckedAccount<'info>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}


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
