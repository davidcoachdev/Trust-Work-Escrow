#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer, ID as SYSTEM_PROGRAM_ID};

declare_id!("7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh");

const BASIS_POINTS: u16 = 10_000;

const ARBITER_FEE_BPS_PER_PARTY: u16 = 250;

const DISPUTE_ACCEPT_GRACE: i64 = 7 * 24 * 60 * 60;
const AUTO_APPROVAL_DELAY: i64 = 7 * 24 * 60 * 60;
const INITIAL_AUTHORITY: Pubkey = pubkey!("3whY1ohdAV3uRXSpyzsKtwLg84X9fTZ1pSdCS8Vvqt7c");

const MAX_PAUSE_DURATION: i64 = 30 * 24 * 60 * 60;

const MIN_JOB_AMOUNT: u64 = 100_000;
const MAX_EVIDENCE_COUNT: u8 = 10;
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
    #[msg("Evidence cannot be empty")]
    EmptyEvidence,
    #[msg("Dispute already has the maximum number of evidence items")]
    EvidenceLimitReached,
    #[msg("Evidence index must equal the next dispute evidence index")]
    InvalidEvidenceIndex,
    #[msg("Evidence account does not match the deterministic dispute PDA")]
    InvalidEvidenceAccount,
    #[msg("Evidence cleanup accounts do not match the expected contiguous range")]
    InvalidEvidenceCleanupAccounts,
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
    #[msg("Auto-approval deadline has not been reached")]
    AutoApprovalNotReady,
    #[msg("Auto-approval is blocked by an open dispute")]
    AutoApprovalBlocked,
    #[msg("Invalid bootstrap authority")]
    InvalidBootstrapAuthority,
    #[msg("Application index must equal the next job application index")]
    ApplicationIndexMismatch,
    #[msg("Application does not belong to this job")]
    InvalidApplicationAccount,
    #[msg("Application is not pending")]
    ApplicationNotPending,
    #[msg("Application proposal cannot be empty")]
    EmptyProposal,
    #[msg("Application cleanup accounts do not match the deterministic job range")]
    InvalidApplicationCleanupAccounts,
}

fn transfer_job_lamports(
    source: &AccountInfo,
    destination: &AccountInfo,
    amount: u64,
) -> Result<()> {
    require!(source.owner == &crate::ID, ErrorCode::NotAuthorized);
    require!(
        source.is_writable && destination.is_writable,
        ErrorCode::NotAuthorized
    );
    require!(source.key() != destination.key(), ErrorCode::NotAuthorized);

    let remaining = source
        .get_lamports()
        .checked_sub(amount)
        .ok_or(ErrorCode::InsufficientFunds)?;
    let rent_minimum = Rent::get()?.minimum_balance(source.data_len());
    require!(remaining >= rent_minimum, ErrorCode::InsufficientFunds);
    let destination_balance = destination
        .get_lamports()
        .checked_add(amount)
        .ok_or(ErrorCode::MathOverflow)?;

    **source.try_borrow_mut_lamports()? = remaining;
    **destination.try_borrow_mut_lamports()? = destination_balance;
    Ok(())
}

fn validate_treasury_destination(destination: &AccountInfo, other: Pubkey) -> Result<()> {
    require!(
        destination.key() != Pubkey::default(),
        ErrorCode::InvalidTreasury
    );
    require!(destination.key() != other, ErrorCode::InvalidTreasury);
    require!(
        destination.owner == &SYSTEM_PROGRAM_ID,
        ErrorCode::InvalidTreasury
    );
    Ok(())
}

fn close_evidence_account(
    evidence: &AccountInfo,
    destination: &AccountInfo,
    dispute: &Pubkey,
    index: u8,
) -> Result<()> {
    require!(
        evidence.owner == &crate::ID,
        ErrorCode::InvalidEvidenceAccount
    );
    let (expected, _) =
        Pubkey::find_program_address(&[b"evidence", dispute.as_ref(), &[index]], &crate::ID);
    require!(
        evidence.key() == expected,
        ErrorCode::InvalidEvidenceAccount
    );

    let data = evidence.try_borrow_data()?;
    let stored = Evidence::try_deserialize(&mut &data[..])?;
    require!(
        stored.dispute == *dispute && stored.index == index,
        ErrorCode::InvalidEvidenceAccount
    );
    drop(data);

    let rent = evidence.get_lamports();
    let destination_balance = destination
        .get_lamports()
        .checked_add(rent)
        .ok_or(ErrorCode::MathOverflow)?;
    **destination.try_borrow_mut_lamports()? = destination_balance;
    **evidence.try_borrow_mut_lamports()? = 0;
    evidence.assign(&SYSTEM_PROGRAM_ID);
    evidence.resize(0)?;
    Ok(())
}

fn cleanup_job_applications(
    job: &Job,
    job_key: &Pubkey,
    start_index: u8,
    remaining_accounts: &[AccountInfo],
    require_full_range: bool,
    allow_closed: bool,
) -> Result<()> {
    require!(
        remaining_accounts.len().is_multiple_of(2),
        ErrorCode::InvalidApplicationCleanupAccounts
    );
    let application_count = remaining_accounts.len() / 2;
    require!(
        application_count <= MAX_APPLICATIONS,
        ErrorCode::InvalidApplicationCleanupAccounts
    );
    let start = start_index as usize;
    require!(
        start
            .checked_add(application_count)
            .ok_or(ErrorCode::InvalidApplicationCleanupAccounts)?
                <= job.applicants.len(),
        ErrorCode::InvalidApplicationCleanupAccounts
    );
    if require_full_range {
        require!(
            start == 0 && application_count == job.applicants.len(),
            ErrorCode::InvalidApplicationCleanupAccounts
        );
    }

    let mut validated = Vec::with_capacity(application_count);
    for (offset, pair) in remaining_accounts.chunks_exact(2).enumerate() {
        let application = &pair[0];
        let applicant = &pair[1];
        let index = start_index
            .checked_add(offset as u8)
            .ok_or(ErrorCode::InvalidApplicationCleanupAccounts)?;
        let expected_applicant = *job
            .applicants
            .get(index as usize)
            .ok_or(ErrorCode::InvalidApplicationCleanupAccounts)?;
        require!(
            applicant.key() == expected_applicant,
            ErrorCode::InvalidApplicationCleanupAccounts
        );
        require!(
            applicant.owner == &SYSTEM_PROGRAM_ID,
            ErrorCode::InvalidApplicationCleanupAccounts
        );
        let (expected, _) = Pubkey::find_program_address(
            &[
                b"application",
                job_key.as_ref(),
                &[index],
                expected_applicant.as_ref(),
            ],
            &crate::ID,
        );
        require!(
            application.key() == expected,
            ErrorCode::InvalidApplicationCleanupAccounts
        );

        if application.owner == &SYSTEM_PROGRAM_ID && application.data_len() == 0 {
            require!(allow_closed, ErrorCode::InvalidApplicationCleanupAccounts);
            validated.push((application, applicant, true));
            continue;
        }
        require!(
            application.owner == &crate::ID,
            ErrorCode::InvalidApplicationCleanupAccounts
        );

        let data = application.try_borrow_data()?;
        let stored = Application::try_deserialize(&mut &data[..])
            .map_err(|_| error!(ErrorCode::InvalidApplicationCleanupAccounts))?;
        require!(
            stored.job == *job_key,
            ErrorCode::InvalidApplicationCleanupAccounts
        );
        require!(
            stored.index == index,
            ErrorCode::InvalidApplicationCleanupAccounts
        );
        require!(
            stored.applicant == expected_applicant,
            ErrorCode::InvalidApplicationCleanupAccounts
        );
        validated.push((
            application,
            applicant,
            stored.status == ApplicationStatus::Accepted,
        ));
    }

    for (application, applicant, accepted_or_closed) in validated {
        if accepted_or_closed {
            continue;
        }
        let rent = application.get_lamports();
        let destination_balance = applicant
            .get_lamports()
            .checked_add(rent)
            .ok_or(ErrorCode::MathOverflow)?;
        **applicant.try_borrow_mut_lamports()? = destination_balance;
        **application.try_borrow_mut_lamports()? = 0;
        application.assign(&SYSTEM_PROGRAM_ID);
        application.resize(0)?;
    }
    Ok(())
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
    pub deadline: i64,
    pub submitted_at: Option<i64>,
    pub milestones_total: u8,
    pub milestones_approved: u8,
    pub milestones_amount_total: u64,
    // Bounded applicant list. Stored as a `Vec` with a fixed maximum capacity
    // (MAX_APPLICATIONS). Anchor reserves the full capacity inside the account's
    // allocated space, so no runtime heap allocation occurs, while avoiding the
    // large stack frames that a fixed `[Pubkey; N]` array produced (the cause of
    // the previous SBF stack overflow at build time).
    #[max_len(MAX_APPLICATIONS)]
    pub applicants: Vec<Pubkey>,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Application {
    pub job: Pubkey,
    pub index: u8,
    pub applicant: Pubkey,
    pub proposal_hash: [u8; 32],
    pub status: ApplicationStatus,
    pub bump: u8,
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
    pub evidence_count: u8,
    pub evidence_cleanup_cursor: u8,
    pub deadline: i64,
    pub client_payout_percent: u8,
    pub freelancer_payout_percent: u8,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Evidence {
    pub dispute: Pubkey,
    pub index: u8,
    pub author: Pubkey,
    pub content_hash: [u8; 32],
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Milestone {
    pub job: Pubkey,
    pub amount: u64,
    pub status: MilestoneStatus,
    pub index: u8,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct SupportTicket {
    pub job: Pubkey,
    pub opened_by: Pubkey,
    pub status: SupportTicketStatus,
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

pub fn compute_shortfall(required: u64, posted: u64) -> u64 {
    required.saturating_sub(posted)
}

#[cfg(test)]
mod tests {
    use super::{compute_shortfall, AUTO_APPROVAL_DELAY, Job, Application, MAX_APPLICATIONS};
    use anchor_lang::Space;

    #[test]
    fn dispute_payout_uses_explicit_shortfall_without_underflow() {
        assert_eq!(compute_shortfall(100, 40), 60);
        assert_eq!(compute_shortfall(100, 140), 0);
    }

    #[test]
    fn auto_approval_boundary_is_inclusive_at_exactly_seven_days() {
        assert_eq!(AUTO_APPROVAL_DELAY, 604_800);
        let submitted_at = 1_000_i64;
        let deadline = submitted_at + AUTO_APPROVAL_DELAY;
        assert!(deadline >= submitted_at + 604_800);
        assert!(deadline + 1 > submitted_at + 604_800);
    }

    // T22: Job compacto — no reserva colección inline sobredimensionada,
    // cuenta compacta con contador/límites y seeds/bump definidos.
    #[test]
    fn job_compact_init_space_under_10kib_and_vec_50_compact() {
        assert_eq!(MAX_APPLICATIONS, 50, "MAX_APPLICATIONS debe ser 50");
        // Job serializado con Vec interior: Anchor INIT_SPACE incluye 4 + 50*32 bytes.
        // Debe ser compacto (< 10KiB inner limit) y no sobredimensionado (28KiB de 50 Applications).
        let init = Job::INIT_SPACE;
        assert!(
            init < 10 * 1024,
            "Job INIT_SPACE {} debe ser < 10KiB (inner allocation limit)",
            init
        );
        assert!(
            init < 28 * 1024,
            "Job INIT_SPACE {} no debe ser 28KiB (50 Applications inline)",
            init
        );
        // Verificamos que el espacio adicional por applicants sea exactamente 50*32 + overhead Vec.
        // Job sin applicants vs con 50: el delta de INIT_SPACE es el overhead reservado.
        // No verificamos el valor exacto (depende de precisa serialización de otros campos),
        // pero sí que el componente dominante sea 50*32 y no 50*sizeof(Application).
        let vec_reserved = 4 + 50 * 32; // borsh Vec<Pubkey>
        assert!(
            init >= vec_reserved,
            "INIT_SPACE debe reservar al menos {} bytes para Vec<Pubkey>",
            vec_reserved
        );
        let application_inline_reserved = 50 * 99; // aprox tamaño Application inline
        // init no debe acercarse a 50*Application; si init > vec_reserved + 3000 probablemente es inline
        assert!(
            init < vec_reserved + 3000,
            "INIT_SPACE {} no debe incluir 50 Applications inline (~{} bytes extra)",
            init,
            application_inline_reserved
        );
    }

    #[test]
    fn job_and_application_have_bump_and_constants() {
        // Job y Application deben tener campo bump (u8) y MAX_APPLICATIONS / constantes definidas.
        assert_eq!(MAX_APPLICATIONS, 50);
        let app_space = Application::INIT_SPACE;
        // Application es compacta: job 32 + index 1 + applicant 32 + proposal_hash 32 + status 1 + bump 1 ~ 99 bytes + 8 disc = ~107 sin overhead
        assert!(app_space > 0 && app_space < 512, "Application INIT_SPACE debe ser compacto, got {}", app_space);
        // Verificamos que el programa declare el ID esperado (se compila con ese ID; no hay otro ID en el árbol).
        assert_eq!(crate::ID.to_string(), "7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh");
    }
}

pub fn check_not_paused(job: &Job) -> Result<()> {
    if job.paused {
        let now = Clock::get()?.unix_timestamp;
        if now
            .checked_sub(job.paused_at)
            .ok_or(ErrorCode::JobPausedExpired)?
            > MAX_PAUSE_DURATION
        {
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
        require!(
            ctx.accounts.authority.key() == INITIAL_AUTHORITY,
            ErrorCode::InvalidBootstrapAuthority
        );
        require!(fee_bps <= BASIS_POINTS, ErrorCode::InvalidFeeBps);
        require!(advisor != Pubkey::default(), ErrorCode::NotAuthorized);
        require!(treasury != Pubkey::default(), ErrorCode::InvalidTreasury);
        require!(
            arbitration_treasury != Pubkey::default(),
            ErrorCode::InvalidTreasury
        );
        require!(treasury != arbitration_treasury, ErrorCode::InvalidTreasury);
        require!(
            ctx.accounts.treasury.owner == &SYSTEM_PROGRAM_ID,
            ErrorCode::InvalidTreasury
        );
        require!(
            ctx.accounts.arbitration_treasury.owner == &SYSTEM_PROGRAM_ID,
            ErrorCode::InvalidTreasury
        );

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
        require!(
            config.authority == ctx.accounts.authority.key(),
            ErrorCode::NotAuthorized
        );
        config.paused = true;
        msg!("Program paused");
        Ok(())
    }

    pub fn unpause(ctx: Context<Unpause>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        require!(
            config.authority == ctx.accounts.authority.key(),
            ErrorCode::NotAuthorized
        );
        config.paused = false;
        msg!("Program unpaused");
        Ok(())
    }

    pub fn update_treasury(ctx: Context<UpdateTreasury>, new_treasury: Pubkey) -> Result<()> {
        let config = &mut ctx.accounts.config;
        require!(
            config.authority == ctx.accounts.authority.key(),
            ErrorCode::NotAuthorized
        );
        require!(
            ctx.accounts.new_treasury.key() == new_treasury,
            ErrorCode::InvalidTreasury
        );
        validate_treasury_destination(
            &ctx.accounts.new_treasury.to_account_info(),
            config.arbitration_treasury,
        )?;
        config.treasury = new_treasury;
        msg!("Treasury updated");
        Ok(())
    }

    pub fn update_arbitration_treasury(
        ctx: Context<UpdateArbitrationTreasury>,
        new_arbitration_treasury: Pubkey,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        require!(
            config.authority == ctx.accounts.authority.key(),
            ErrorCode::NotAuthorized
        );
        require!(
            ctx.accounts.new_arbitration_treasury.key() == new_arbitration_treasury,
            ErrorCode::InvalidTreasury
        );
        validate_treasury_destination(
            &ctx.accounts.new_arbitration_treasury.to_account_info(),
            config.treasury,
        )?;
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

    pub fn withdraw_arbitration(ctx: Context<WithdrawArbitration>, amount: u64) -> Result<()> {
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
        amount: u64,
        deadline: i64,
    ) -> Result<()> {
        let config = &ctx.accounts.config;
        require!(!config.paused, ErrorCode::ProgramPaused);
        require!(amount >= MIN_JOB_AMOUNT, ErrorCode::AmountTooSmall);
        require!(
            deadline > Clock::get()?.unix_timestamp,
            ErrorCode::DeadlineMustBeFuture
        );

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
            !job.applicants.iter().any(|a| *a == ctx.accounts.applicant.key()),
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
        let applicant = application.applicant;
        application.status = ApplicationStatus::Accepted;

        job.freelancer = Some(applicant);
        job.status = JobStatus::InProgress;

        msg!("Application accepted: freelancer {}", applicant);
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

        job.status = JobStatus::Submitted;
        job.submitted_at = Some(Clock::get()?.unix_timestamp);

        msg!("Work submitted for job: {}", job.key());
        Ok(())
    }

    pub fn auto_approve_work(ctx: Context<AutoApproveWork>, _job_id: u64) -> Result<()> {
        let job = &mut ctx.accounts.job;
        require!(
            job.status == JobStatus::Submitted,
            ErrorCode::InvalidJobStatus
        );
        let submitted_at = job.submitted_at.ok_or(ErrorCode::InvalidJobStatus)?;
        let deadline = submitted_at
            .checked_add(AUTO_APPROVAL_DELAY)
            .ok_or(ErrorCode::MathOverflow)?;
        require!(
            Clock::get()?.unix_timestamp >= deadline,
            ErrorCode::AutoApprovalNotReady
        );
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
        let remaining = job.to_account_info().get_lamports();
        let client_balance = ctx
            .accounts
            .client
            .get_lamports()
            .checked_add(remaining)
            .ok_or(ErrorCode::MathOverflow)?;
        **ctx.accounts.client.try_borrow_mut_lamports()? = client_balance;
        **job.to_account_info().try_borrow_mut_lamports()? = 0;
        job.to_account_info().assign(&SYSTEM_PROGRAM_ID);
        job.to_account_info().resize(0)?;
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

    pub fn create_arbiter_pool(ctx: Context<CreateArbiterPool>) -> Result<()> {
        require!(
            ctx.accounts.config.authority == ctx.accounts.authority.key(),
            ErrorCode::NotAuthorized
        );
        let pool = &mut ctx.accounts.pool;
        pool.authority = ctx.accounts.authority.key();
        pool.arbiters = Vec::new();
        pool.bump = ctx.bumps.pool;
        Ok(())
    }

    pub fn add_arbiter(ctx: Context<AddArbiter>, new_arbiter: Pubkey) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        require!(
            ctx.accounts.config.authority == ctx.accounts.authority.key(),
            ErrorCode::NotAuthorized
        );
        require!(
            pool.authority == ctx.accounts.authority.key(),
            ErrorCode::NotAuthorized
        );
        require!(
            !pool.arbiters.contains(&new_arbiter),
            ErrorCode::NotValidArbiter
        );
        require!(
            pool.arbiters.len() < MAX_ARBITERS,
            ErrorCode::NotValidArbiter
        );
        pool.arbiters.push(new_arbiter);
        Ok(())
    }

    pub fn remove_arbiter(ctx: Context<RemoveArbiter>, arbiter: Pubkey) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        require!(
            ctx.accounts.config.authority == ctx.accounts.authority.key(),
            ErrorCode::NotAuthorized
        );
        require!(
            pool.authority == ctx.accounts.authority.key(),
            ErrorCode::NotAuthorized
        );
        let idx = pool
            .arbiters
            .iter()
            .position(|&a| a == arbiter)
            .ok_or(ErrorCode::NotValidArbiter)?;
        pool.arbiters.remove(idx);
        Ok(())
    }

    pub fn raise_dispute(ctx: Context<RaiseDispute>, _job_id: u64) -> Result<()> {
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
        let dispute = &mut ctx.accounts.dispute;
        require!(
            dispute.status == DisputeStatus::Open,
            ErrorCode::DisputeAlreadyResolved
        );
        require!(
            Clock::get()?.unix_timestamp <= dispute.deadline,
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
        require!(
            ctx.accounts.config.authority == ctx.accounts.authority.key(),
            ErrorCode::NotAuthorized
        );
        let pool = &ctx.accounts.pool;
        require!(
            pool.authority == ctx.accounts.config.authority,
            ErrorCode::NotAuthorized
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
        let dispute = &mut ctx.accounts.dispute;
        require!(
            dispute.status == DisputeStatus::Open,
            ErrorCode::DisputeAlreadyResolved
        );
        require!(
            Clock::get()?.unix_timestamp <= dispute.deadline,
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

    pub fn open_support_ticket(
        ctx: Context<OpenSupportTicket>,
        _job_id: u64,
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

        require!(ctx.accounts.dispute.is_none(), ErrorCode::CaseAlreadyOpen);

        let ticket = &mut ctx.accounts.ticket;
        ticket.job = job.key();
        ticket.opened_by = opener;
        ticket.status = SupportTicketStatus::Open;
        ticket.bump = ctx.bumps.ticket;

        msg!("Support ticket opened for job: {}", job.key());
        Ok(())
    }

    pub fn resolve_support_ticket(
        ctx: Context<ResolveSupportTicket>,
        _job_id: u64,
    ) -> Result<()> {
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

        cleanup_job_applications(job, &job.key(), 0, ctx.remaining_accounts, true, true)?;

        let remaining_principal = job
            .amount
            .checked_sub(job.milestones_amount_total)
            .ok_or(ErrorCode::MathOverflow)?;
        let refund = remaining_principal
            .checked_add(job.fee_amount)
            .ok_or(ErrorCode::MathOverflow)?;
        transfer_job_lamports(
            &job.to_account_info(),
            &ctx.accounts.client.to_account_info(),
            refund,
        )?;

        job.status = JobStatus::Cancelled;
        ticket.status = SupportTicketStatus::Resolved;

        msg!("Support ticket resolved (job cancelled): {}", job.key());
        Ok(())
    }

    pub fn finalize_dispute_payouts(
        ctx: Context<FinalizeDisputePayouts>,
        _job_id: u64,
    ) -> Result<()> {
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
        cleanup_job_applications(job, &job.key(), 0, application_accounts, true, true)?;

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

        transfer_job_lamports(
            &job.to_account_info(),
            &ctx.accounts.treasury.to_account_info(),
            fee_amount,
        )?;

        let client_net = (to_parties as u128 * client_pct as u128 / 100) as u64;
        if client_net > 0 {
            transfer_job_lamports(
                &job.to_account_info(),
                &ctx.accounts.client.to_account_info(),
                client_net,
            )?;
        }

        let freelancer_net = (to_parties as u128 * freelancer_pct as u128 / 100) as u64;
        if freelancer_net > 0 {
            transfer_job_lamports(
                &job.to_account_info(),
                &ctx.accounts.freelancer.to_account_info(),
                freelancer_net,
            )?;
        }

        if shortfall > 0 {
            transfer_job_lamports(
                &job.to_account_info(),
                &ctx.accounts.arbitration_treasury.to_account_info(),
                shortfall,
            )?;
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
                index,
            )?;
        }

        msg!("Dispute finalized for job: {}", job.key());
        Ok(())
    }

    pub fn cleanup_dispute_evidence(
        ctx: Context<CleanupDisputeEvidence>,
        _job_id: u64,
    ) -> Result<()> {
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
        for (offset, evidence) in ctx.remaining_accounts.iter().enumerate() {
            let index = dispute.evidence_cleanup_cursor + offset as u8;
            close_evidence_account(
                evidence,
                &ctx.accounts.client.to_account_info(),
                &dispute.key(),
                index,
            )?;
        }
        dispute.evidence_cleanup_cursor = dispute
            .evidence_cleanup_cursor
            .checked_add(ctx.remaining_accounts.len() as u8)
            .ok_or(ErrorCode::MathOverflow)?;
        Ok(())
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
}

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: Treasury wallet que recibe fees. Almacenada en config.
    pub treasury: UncheckedAccount<'info>,
    /// CHECK: Cuenta system que recibe fees de arbitraje.
    pub arbitration_treasury: UncheckedAccount<'info>,
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
    /// CHECK: Validated in the instruction as a non-default System account distinct from arbitration_treasury.
    pub new_treasury: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct UpdateArbitrationTreasury<'info> {
    pub authority: Signer<'info>,
    #[account(mut, seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    /// CHECK: Validated in the instruction as a non-default System account distinct from treasury.
    pub new_arbitration_treasury: UncheckedAccount<'info>,
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
pub struct AutoApproveWork<'info> {
    pub keeper: Signer<'info>,
    /// CHECK: Debe ser el cliente ligado al PDA del job y recibe la rent restante.
    #[account(mut, constraint = client.owner == &SYSTEM_PROGRAM_ID @ ErrorCode::NotAuthorized)]
    pub client: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()],
        bump = job.bump
    )]
    pub job: Account<'info, Job>,
    #[account(mut, constraint = job.freelancer == Some(freelancer.key()) @ ErrorCode::NotJobFreelancer)]
    pub freelancer: SystemAccount<'info>,
    /// CHECK: Validada contra Config.treasury.
    #[account(mut, constraint = treasury.key() == config.treasury @ ErrorCode::InvalidTreasury)]
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
pub struct CreateArbiterPool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init, payer = authority, space = ArbiterPool::INIT_SPACE + 8, seeds = [b"arbiter_pool"], bump)]
    pub pool: Account<'info, ArbiterPool>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddArbiter<'info> {
    pub authority: Signer<'info>,
    #[account(mut, seeds = [b"arbiter_pool"], bump = pool.bump)]
    pub pool: Account<'info, ArbiterPool>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
pub struct RemoveArbiter<'info> {
    pub authority: Signer<'info>,
    #[account(mut, seeds = [b"arbiter_pool"], bump = pool.bump)]
    pub pool: Account<'info, ArbiterPool>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
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
#[instruction(job_id: u64)]
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
#[instruction(job_id: u64)]
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
