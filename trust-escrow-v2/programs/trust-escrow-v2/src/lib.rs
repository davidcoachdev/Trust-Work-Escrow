use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

declare_id!("TesT3XPqD3WFFVTY4BTwZ3sJpY7C7hF3Z6K2oX3i7jB");

// =============================================================================
// CONSTANTS
// =============================================================================

const MAX_USERNAME_LENGTH: usize = 32;
const MAX_BIO_LENGTH: usize = 500;
const MIN_JOB_AMOUNT: u64 = 100_000;
const MAX_TITLE_LENGTH: usize = 64;
const MAX_DESCRIPTION_LENGTH: usize = 1024;
const MAX_PROPOSAL_LENGTH: usize = 512;
const MAX_WALLETS: usize = 5;
const MAX_MULTISIG_OWNERS: usize = 5;
const MAX_ARBITERS: usize = 50;
const MAX_TEAM_MEMBERS: usize = 20;
const MAX_APPLICATIONS: usize = 50;
const MAX_DISPUTE_EVIDENCE: usize = 2048;
const MAX_MILESTONE_TITLE: usize = 64;
const MAX_MILESTONES: usize = 20;

// =============================================================================
// ERRORS
// =============================================================================

#[error_code]
pub enum ErrorCode {
    #[msg("User account already exists")]
    UserAlreadyExists,

    #[msg("Wallet already associated to this user")]
    WalletAlreadyAssociated,

    #[msg("Wallet not associated to this user")]
    WalletNotAssociated,

    #[msg("No active wallet set")]
    NoActiveWallet,

    #[msg("Maximum number of wallets reached")]
    MaxWalletsReached,

    #[msg("Maximum number of arbiters reached (50)")]
    MaxArbitersReached,

    #[msg("Maximum number of multisig owners reached (5)")]
    MaxMultisigOwnersReached,

    #[msg("Multisig threshold must be at least 1")]
    InvalidMultisigThreshold,

    #[msg("Multisig threshold exceeds number of owners")]
    ThresholdExceedsOwners,

    #[msg("Not authorized")]
    NotAuthorized,

    #[msg("Not authorized - not the admin")]
    NotAdmin,

    #[msg("Not authorized - not a valid arbiter")]
    NotArbiter,

    #[msg("Not authorized - not the job client")]
    NotJobClient,

    #[msg("Not authorized - not the job freelancer")]
    NotJobFreelancer,

    #[msg("Cannot accept your own job")]
    CannotAcceptOwnJob,

    #[msg("Program is paused")]
    ProgramPaused,

    #[msg("Invalid job status for this operation")]
    InvalidJobStatus,

    #[msg("Amount too small")]
    AmountTooSmall,

    #[msg("Title cannot be empty")]
    EmptyTitle,

    #[msg("Title exceeds maximum length")]
    TitleTooLong,

    #[msg("Description exceeds maximum length")]
    DescriptionTooLong,

    #[msg("Username cannot be empty")]
    EmptyUsername,

    #[msg("Username exceeds maximum length (32)")]
    UsernameTooLong,

    #[msg("Bio exceeds maximum length (500)")]
    BioTooLong,

    #[msg("Invalid fee percentage (must be 0-100)")]
    InvalidFeePercentage,

    #[msg("Insufficient funds for this operation")]
    InsufficientFunds,

    #[msg("Job not funded")]
    JobNotFunded,

    #[msg("No freelancer assigned")]
    NoFreelancerAssigned,

    #[msg("No arbiter assigned")]
    NoArbiterAssigned,

    #[msg("Dispute already resolved")]
    DisputeAlreadyResolved,

    #[msg("Invalid application status")]
    InvalidApplicationStatus,

    #[msg("Proposal too long")]
    ProposalTooLong,

    #[msg("Deadline must be in the future")]
    DeadlineInPast,

    #[msg("Deadline must be after current time")]
    DeadlineMustBeFuture,

    #[msg("Milestone not found")]
    MilestoneNotFound,

    #[msg("Milestone already completed")]
    MilestoneAlreadyCompleted,

    #[msg("Milestone not due yet")]
    MilestoneNotDue,

    #[msg("All milestones must be completed")]
    AllMilestonesRequired,

    #[msg("Cannot dispute at this stage")]
    CannotDisputeAtStage,

    #[msg("Evidence too long")]
    EvidenceTooLong,

    #[msg("Dispute period expired")]
    DisputePeriodExpired,

    #[msg("Not a valid arbiter for this dispute")]
    NotValidArbiter,
}

// =============================================================================
// STATE ENUMS
// =============================================================================

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum JobStatus {
    Created,
    ApplicationsOpen,
    InProgress,
    Submitted,
    Approved,
    Disputed,
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
pub enum MemberRole {
    Owner,
    ProjectManager,
    Contributor,
}

impl anchor_lang::Space for MemberRole {
    const INIT_SPACE: usize = 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum DisputeStatus {
    Open,
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

// =============================================================================
// STATE STRUCTS
// =============================================================================

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub admin: Pubkey,
    pub treasury: Pubkey,
    #[max_len(MAX_MULTISIG_OWNERS)]
    pub multisig_owners: Vec<Pubkey>,
    pub multisig_threshold: u8,
    pub fee_percent: u8,
    pub paused: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct User {
    pub wallet_principal: Pubkey,
    #[max_len(MAX_WALLETS)]
    pub wallets: Vec<Pubkey>,
    pub active_wallet: Pubkey,
    #[max_len(MAX_USERNAME_LENGTH)]
    pub username: String,
    #[max_len(MAX_BIO_LENGTH)]
    pub bio: Option<String>,
    pub created_at: i64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Job {
    pub client: Pubkey,
    pub freelancer: Option<Pubkey>,
    pub team: Option<Pubkey>,
    #[max_len(MAX_TITLE_LENGTH)]
    pub title: String,
    #[max_len(MAX_DESCRIPTION_LENGTH)]
    pub description: String,
    pub amount: u64,
    pub fee: u64,
    pub total_deposited: u64,
    pub deadline: i64,
    pub status: JobStatus,
    #[max_len(MAX_APPLICATIONS)]
    pub applications: Vec<Application>,
    pub bump: u8,
    pub created_at: i64,
    pub updated_at: i64,
    pub submitted_at: Option<i64>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub struct Application {
    pub applicant: Pubkey,
    pub is_team: bool,
    #[max_len(MAX_PROPOSAL_LENGTH)]
    pub proposal: String,
    pub applied_at: i64,
    pub status: ApplicationStatus,
}

#[account]
#[derive(InitSpace)]
pub struct Team {
    pub owner: Pubkey,
    #[max_len(MAX_TEAM_MEMBERS)]
    pub members: Vec<Member>,
    #[max_len(MAX_TITLE_LENGTH)]
    pub name: String,
    #[max_len(MAX_DESCRIPTION_LENGTH)]
    pub description: String,
    pub bump: u8,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub struct Member {
    pub user: Pubkey,
    pub role: MemberRole,
    pub joined_at: i64,
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
    #[max_len(500)]
    pub reason: String,
    pub created_at: i64,
    pub deadline: i64,
    pub resolved_at: Option<i64>,
    #[max_len(500)]
    pub resolution: Option<String>,
    pub client_payout_percent: u8,
    pub freelancer_payout_percent: u8,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub struct Evidence {
    pub submitter: Pubkey,
    #[max_len(2048)]
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

// =============================================================================
// PROGRAM MODULE
// =============================================================================

#[program]
pub mod escrow {
    use super::*;

    // -------------------------------------------------------------------------
    // CONFIG INSTRUCTIONS
    // -------------------------------------------------------------------------

    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        multisig_owners: Vec<Pubkey>,
        multisig_threshold: u8,
        treasury: Pubkey,
        fee_percent: u8,
    ) -> Result<()> {
        require!(fee_percent <= 100, ErrorCode::InvalidFeePercentage);
        require!(!multisig_owners.is_empty(), ErrorCode::NotAuthorized);
        require!(multisig_threshold >= 1, ErrorCode::InvalidMultisigThreshold);
        require!(
            multisig_threshold as usize <= multisig_owners.len(),
            ErrorCode::ThresholdExceedsOwners
        );

        let config = &mut ctx.accounts.config;
        config.admin = ctx.accounts.authority.key();
        config.treasury = treasury;
        config.multisig_owners = multisig_owners;
        config.multisig_threshold = multisig_threshold;
        config.fee_percent = fee_percent;
        config.paused = false;
        config.bump = ctx.bumps.config;

        Ok(())
    }

    pub fn pause(ctx: Context<Pause>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        require!(
            ctx.accounts.authority.key() == config.admin,
            ErrorCode::NotAdmin
        );
        config.paused = true;
        Ok(())
    }

    pub fn unpause(ctx: Context<Unpause>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        require!(
            ctx.accounts.authority.key() == config.admin,
            ErrorCode::NotAdmin
        );
        config.paused = false;
        Ok(())
    }

    pub fn withdraw_treasury(ctx: Context<WithdrawTreasury>, amount: u64) -> Result<()> {
        let config = &ctx.accounts.config;
        require!(
            ctx.accounts.admin.key() == config.admin,
            ErrorCode::NotAdmin
        );
        require!(
            ctx.accounts.treasury.lamports() >= amount,
            ErrorCode::InsufficientFunds
        );
        require!(
            ctx.accounts.treasury.lamports() >= amount,
            ErrorCode::InsufficientFunds
        );

        let cpi_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.treasury.to_account_info(),
                to: ctx.accounts.admin.to_account_info(),
            },
        );
        transfer(cpi_ctx, amount)?;
        Ok(())
    }

    pub fn update_treasury(ctx: Context<UpdateTreasury>, new_treasury: Pubkey) -> Result<()> {
        let config = &mut ctx.accounts.config;
        require!(
            ctx.accounts.admin.key() == config.admin,
            ErrorCode::NotAdmin
        );
        config.treasury = new_treasury;
        Ok(())
    }

    // -------------------------------------------------------------------------
    // USER INSTRUCTIONS
    // -------------------------------------------------------------------------

    pub fn create_user(ctx: Context<CreateUser>, username: String) -> Result<()> {
        require!(!username.is_empty(), ErrorCode::EmptyUsername);
        require!(
            username.len() <= MAX_USERNAME_LENGTH,
            ErrorCode::UsernameTooLong
        );

        let user = &mut ctx.accounts.user;
        user.wallet_principal = ctx.accounts.authority.key();
        user.wallets = Vec::new();
        user.active_wallet = ctx.accounts.authority.key();
        user.username = username;
        user.bio = None;
        user.created_at = Clock::get()?.unix_timestamp;
        user.bump = ctx.bumps.user;

        Ok(())
    }

    pub fn add_wallet(ctx: Context<AddWallet>, new_wallet: Pubkey) -> Result<()> {
        let user = &mut ctx.accounts.user;
        require!(
            user.wallet_principal == ctx.accounts.authority.key(),
            ErrorCode::NotAuthorized
        );
        require!(
            !user.wallets.contains(&new_wallet),
            ErrorCode::WalletAlreadyAssociated
        );
        require!(
            user.wallets.len() < MAX_WALLETS,
            ErrorCode::MaxWalletsReached
        );

        user.wallets.push(new_wallet);
        Ok(())
    }

    pub fn set_active_wallet(ctx: Context<SetActiveWallet>, wallet: Pubkey) -> Result<()> {
        let user = &mut ctx.accounts.user;
        require!(
            user.wallet_principal == ctx.accounts.authority.key(),
            ErrorCode::NotAuthorized
        );

        let is_associated = user.wallet_principal == wallet || user.wallets.contains(&wallet);
        require!(is_associated, ErrorCode::WalletNotAssociated);

        user.active_wallet = wallet;
        Ok(())
    }

    pub fn update_user(ctx: Context<UpdateUser>, bio: Option<String>) -> Result<()> {
        let user = &mut ctx.accounts.user;
        require!(
            user.wallet_principal == ctx.accounts.authority.key(),
            ErrorCode::NotAuthorized
        );

        if let Some(b) = bio {
            require!(b.len() <= MAX_BIO_LENGTH, ErrorCode::BioTooLong);
            user.bio = Some(b);
        }
        Ok(())
    }

    // -------------------------------------------------------------------------
    // TEAM INSTRUCTIONS
    // -------------------------------------------------------------------------

    pub fn create_team(ctx: Context<CreateTeam>, name: String, description: String) -> Result<()> {
        require!(!name.is_empty(), ErrorCode::EmptyTitle);
        require!(name.len() <= MAX_TITLE_LENGTH, ErrorCode::TitleTooLong);
        require!(
            description.len() <= MAX_DESCRIPTION_LENGTH,
            ErrorCode::DescriptionTooLong
        );

        let team = &mut ctx.accounts.team;
        team.owner = ctx.accounts.owner.key();
        team.members = vec![Member {
            user: ctx.accounts.owner.key(),
            role: MemberRole::Owner,
            joined_at: Clock::get()?.unix_timestamp,
        }];
        team.name = name;
        team.description = description;
        team.bump = ctx.bumps.team;
        team.created_at = Clock::get()?.unix_timestamp;
        team.updated_at = Clock::get()?.unix_timestamp;

        Ok(())
    }

    pub fn add_team_member(ctx: Context<AddTeamMember>, user: Pubkey) -> Result<()> {
        let team = &mut ctx.accounts.team;
        require!(
            team.owner == ctx.accounts.owner.key(),
            ErrorCode::NotAuthorized
        );
        require!(
            team.members.len() < MAX_TEAM_MEMBERS,
            ErrorCode::MaxWalletsReached
        );
        require!(
            !team.members.iter().any(|m| m.user == user),
            ErrorCode::WalletAlreadyAssociated
        );

        team.members.push(Member {
            user,
            role: MemberRole::Contributor,
            joined_at: Clock::get()?.unix_timestamp,
        });
        team.updated_at = Clock::get()?.unix_timestamp;

        Ok(())
    }

    // -------------------------------------------------------------------------
    // JOB INSTRUCTIONS
    // -------------------------------------------------------------------------

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
        require!(!title.is_empty(), ErrorCode::EmptyTitle);
        require!(title.len() <= MAX_TITLE_LENGTH, ErrorCode::TitleTooLong);
        require!(
            description.len() <= MAX_DESCRIPTION_LENGTH,
            ErrorCode::DescriptionTooLong
        );
        require!(amount >= MIN_JOB_AMOUNT, ErrorCode::AmountTooSmall);
        require!(
            deadline > Clock::get()?.unix_timestamp,
            ErrorCode::DeadlineMustBeFuture
        );

        let fee = amount * config.fee_percent as u64 / 10000;

        let job = &mut ctx.accounts.job;
        job.client = ctx.accounts.client.key();
        job.freelancer = None;
        job.team = None;
        job.title = title;
        job.description = description;
        job.amount = amount;
        job.fee = fee;
        job.total_deposited = 0;
        job.deadline = deadline;
        job.status = JobStatus::Created;
        job.applications = Vec::new();
        job.bump = ctx.bumps.job;
        job.created_at = Clock::get()?.unix_timestamp;
        job.updated_at = Clock::get()?.unix_timestamp;
        job.submitted_at = None;

        Ok(())
    }

    pub fn deposit_funds(ctx: Context<DepositFunds>, _job_id: u64) -> Result<()> {
        let client_key = ctx.accounts.client.key();
        let total = ctx.accounts.job.amount + ctx.accounts.job.fee;

        require!(
            ctx.accounts.job.status == JobStatus::Created,
            ErrorCode::InvalidJobStatus
        );
        require!(
            ctx.accounts.job.client == client_key,
            ErrorCode::NotJobClient
        );

        let cpi_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.client.to_account_info(),
                to: ctx.accounts.job.to_account_info(),
            },
        );
        transfer(cpi_ctx, total)?;

        let job = &mut ctx.accounts.job;
        job.status = JobStatus::ApplicationsOpen;
        job.total_deposited = total;
        job.updated_at = Clock::get()?.unix_timestamp;

        Ok(())
    }

    pub fn apply_to_job(
        ctx: Context<ApplyToJob>,
        _job_id: u64,
        proposal: String,
        is_team: bool,
    ) -> Result<()> {
        let job = &mut ctx.accounts.job;
        require!(
            job.status == JobStatus::ApplicationsOpen,
            ErrorCode::InvalidJobStatus
        );
        require!(
            proposal.len() <= MAX_PROPOSAL_LENGTH,
            ErrorCode::ProposalTooLong
        );

        let applicant = if is_team {
            ctx.accounts.team.as_ref().unwrap().key()
        } else {
            ctx.accounts.applicant.key()
        };

        require!(
            !job.applications.iter().any(|a| a.applicant == applicant),
            ErrorCode::UserAlreadyExists
        );

        job.applications.push(Application {
            applicant,
            is_team,
            proposal,
            applied_at: Clock::get()?.unix_timestamp,
            status: ApplicationStatus::Pending,
        });
        job.updated_at = Clock::get()?.unix_timestamp;

        Ok(())
    }

    pub fn accept_application(
        ctx: Context<AcceptApplication>,
        _job_id: u64,
        applicant: Pubkey,
        is_team: bool,
    ) -> Result<()> {
        let job = &mut ctx.accounts.job;
        require!(
            job.client == ctx.accounts.client.key(),
            ErrorCode::NotJobClient
        );
        require!(
            job.status == JobStatus::ApplicationsOpen,
            ErrorCode::InvalidJobStatus
        );

        let app_idx = job
            .applications
            .iter()
            .position(|a| a.applicant == applicant && a.is_team == is_team)
            .ok_or(ErrorCode::InvalidApplicationStatus)?;

        job.applications[app_idx].status = ApplicationStatus::Accepted;
        job.freelancer = Some(applicant);
        job.team = if is_team { Some(applicant) } else { None };
        job.status = JobStatus::InProgress;
        job.updated_at = Clock::get()?.unix_timestamp;

        Ok(())
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
        job.updated_at = Clock::get()?.unix_timestamp;

        Ok(())
    }

    pub fn approve_work(ctx: Context<ApproveWork>, _job_id: u64) -> Result<()> {
        let job = &mut ctx.accounts.job;
        let client_key = ctx.accounts.client.key();
        require!(job.client == client_key, ErrorCode::NotJobClient);
        require!(
            job.status == JobStatus::Submitted,
            ErrorCode::InvalidJobStatus
        );
        require!(job.freelancer.is_some(), ErrorCode::NoFreelancerAssigned);

        let amount = job.amount;
        let fee = job.fee;

        job.status = JobStatus::Approved;
        job.updated_at = Clock::get()?.unix_timestamp;

        // Transfer to freelancer
        let tf = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.job.to_account_info(),
                to: ctx.accounts.freelancer.to_account_info(),
            },
        );
        transfer(tf, amount)?;

        // Transfer fee to treasury
        let tt = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.job.to_account_info(),
                to: ctx.accounts.config.to_account_info(),
            },
        );
        transfer(tt, fee)?;

        Ok(())
    }

    pub fn reject_work(ctx: Context<RejectWork>, _job_id: u64, _reason: String) -> Result<()> {
        let job = &mut ctx.accounts.job;
        require!(
            job.client == ctx.accounts.client.key(),
            ErrorCode::NotJobClient
        );
        require!(
            job.status == JobStatus::Submitted,
            ErrorCode::InvalidJobStatus
        );

        job.status = JobStatus::Disputed;
        job.updated_at = Clock::get()?.unix_timestamp;

        Ok(())
    }

    pub fn cancel_job(ctx: Context<CancelJob>, _job_id: u64) -> Result<()> {
        let job = &mut ctx.accounts.job;
        let client_key = ctx.accounts.client.key();
        require!(job.client == client_key, ErrorCode::NotJobClient);
        require!(
            job.status == JobStatus::Created || job.status == JobStatus::ApplicationsOpen,
            ErrorCode::InvalidJobStatus
        );

        let refund = job.amount + job.fee;
        job.status = JobStatus::Cancelled;
        job.updated_at = Clock::get()?.unix_timestamp;

        let t = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.job.to_account_info(),
                to: ctx.accounts.client.to_account_info(),
            },
        );
        transfer(t, refund)?;

        Ok(())
    }

    // -------------------------------------------------------------------------
    // ARBITER INSTRUCTIONS
    // -------------------------------------------------------------------------

    pub fn create_arbiter_pool(ctx: Context<CreateArbiterPool>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.authority = ctx.accounts.admin.key();
        pool.arbiters = Vec::new();
        pool.bump = ctx.bumps.pool;

        Ok(())
    }

    pub fn add_arbiter(ctx: Context<AddArbiter>, new_arbiter: Pubkey) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        require!(
            pool.authority == ctx.accounts.admin.key(),
            ErrorCode::NotAdmin
        );
        require!(
            !pool.arbiters.contains(&new_arbiter),
            ErrorCode::WalletAlreadyAssociated
        );
        require!(
            pool.arbiters.len() < MAX_ARBITERS,
            ErrorCode::MaxArbitersReached
        );

        pool.arbiters.push(new_arbiter);
        Ok(())
    }

    pub fn remove_arbiter(ctx: Context<RemoveArbiter>, arbiter: Pubkey) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        require!(
            pool.authority == ctx.accounts.admin.key(),
            ErrorCode::NotAdmin
        );

        let idx = pool
            .arbiters
            .iter()
            .position(|&a| a == arbiter)
            .ok_or(ErrorCode::NotArbiter)?;
        pool.arbiters.remove(idx);

        Ok(())
    }

    // -------------------------------------------------------------------------
    // DISPUTE INSTRUCTIONS
    // -------------------------------------------------------------------------

    pub fn raise_dispute(
        ctx: Context<RaiseDispute>,
        _job_id: u64,
        reason: String,
        deadline: i64,
    ) -> Result<()> {
        let job = &mut ctx.accounts.job;
        require!(
            job.status == JobStatus::Submitted || job.status == JobStatus::InProgress,
            ErrorCode::CannotDisputeAtStage
        );
        require!(
            job.client == ctx.accounts.raiser.key()
                || job.freelancer == Some(ctx.accounts.raiser.key()),
            ErrorCode::NotAuthorized
        );

        let dispute = &mut ctx.accounts.dispute;
        dispute.job = job.key();
        dispute.raised_by = ctx.accounts.raiser.key();
        dispute.arbiter = None;
        dispute.status = DisputeStatus::Open;
        dispute.evidence = Vec::new();
        dispute.reason = reason;
        dispute.created_at = Clock::get()?.unix_timestamp;
        dispute.deadline = deadline;
        dispute.resolved_at = None;
        dispute.resolution = None;
        dispute.client_payout_percent = 0;
        dispute.freelancer_payout_percent = 0;
        dispute.bump = ctx.bumps.dispute;

        job.status = JobStatus::Disputed;
        job.updated_at = Clock::get()?.unix_timestamp;

        Ok(())
    }

    pub fn submit_evidence(
        ctx: Context<SubmitEvidence>,
        _job_id: u64,
        content: String,
    ) -> Result<()> {
        let dispute = &mut ctx.accounts.dispute;
        require!(
            dispute.status == DisputeStatus::Open
                || dispute.status == DisputeStatus::EvidenceSubmitted,
            ErrorCode::DisputeAlreadyResolved
        );
        require!(
            content.len() <= MAX_DISPUTE_EVIDENCE,
            ErrorCode::EvidenceTooLong
        );

        dispute.evidence.push(Evidence {
            submitter: ctx.accounts.submitter.key(),
            content,
            submitted_at: Clock::get()?.unix_timestamp,
        });
        dispute.status = DisputeStatus::EvidenceSubmitted;

        Ok(())
    }

    pub fn assign_arbiter(ctx: Context<AssignArbiter>, _job_id: u64) -> Result<()> {
        let dispute = &mut ctx.accounts.dispute;
        let pool = &ctx.accounts.pool;

        require!(dispute.arbiter.is_none(), ErrorCode::NoArbiterAssigned);
        require!(
            pool.arbiters.contains(&ctx.accounts.arbiter.key()),
            ErrorCode::NotValidArbiter
        );

        dispute.arbiter = Some(ctx.accounts.arbiter.key());
        dispute.status = DisputeStatus::ArbiterAssigned;

        Ok(())
    }

    pub fn resolve_dispute(
        ctx: Context<ResolveDispute>,
        _job_id: u64,
        resolution: String,
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
        require!(
            client_payout_percent <= 100,
            ErrorCode::InvalidFeePercentage
        );

        dispute.resolution = Some(resolution);
        dispute.resolved_at = Some(Clock::get()?.unix_timestamp);
        dispute.client_payout_percent = client_payout_percent;
        dispute.freelancer_payout_percent = 100 - client_payout_percent;
        dispute.status = DisputeStatus::Resolved;

        Ok(())
    }

    pub fn finalize_dispute_payouts(
        ctx: Context<FinalizeDisputePayouts>,
        _job_id: u64,
    ) -> Result<()> {
        let dispute = &ctx.accounts.dispute;
        let job = &ctx.accounts.job;

        require!(
            dispute.status == DisputeStatus::Resolved,
            ErrorCode::DisputeAlreadyResolved
        );
        require!(
            dispute.arbiter == Some(ctx.accounts.arbiter.key()),
            ErrorCode::NotArbiter
        );

        let total = job.amount;
        let client_share = total * dispute.client_payout_percent as u64 / 100;
        let freelancer_share = total * dispute.freelancer_payout_percent as u64 / 100;

        // Pay client
        if dispute.client_payout_percent > 0 {
            let tc = CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.job.to_account_info(),
                    to: ctx.accounts.client.to_account_info(),
                },
            );
            transfer(tc, client_share)?;
        }

        // Pay freelancer
        if dispute.freelancer_payout_percent > 0 {
            let tf = CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.job.to_account_info(),
                    to: ctx.accounts.freelancer.to_account_info(),
                },
            );
            transfer(tf, freelancer_share)?;
        }

        Ok(())
    }

    // -------------------------------------------------------------------------
    // MILESTONE INSTRUCTIONS
    // -------------------------------------------------------------------------

    pub fn create_milestone(
        ctx: Context<CreateMilestone>,
        _job_id: u64,
        title: String,
        description: String,
        amount: u64,
        deadline: i64,
        index: u8,
    ) -> Result<()> {
        let job = &ctx.accounts.job;
        require!(
            job.client == ctx.accounts.client.key(),
            ErrorCode::NotJobClient
        );
        require!(
            job.status == JobStatus::InProgress,
            ErrorCode::InvalidJobStatus
        );
        require!(!title.is_empty(), ErrorCode::EmptyTitle);
        require!(
            deadline > Clock::get()?.unix_timestamp,
            ErrorCode::DeadlineMustBeFuture
        );

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

        Ok(())
    }

    pub fn submit_milestone(
        ctx: Context<SubmitMilestone>,
        _job_id: u64,
        _milestone_index: u8,
    ) -> Result<()> {
        let milestone = &mut ctx.accounts.milestone;
        require!(
            milestone.status == MilestoneStatus::Pending,
            ErrorCode::MilestoneAlreadyCompleted
        );
        require!(
            milestone.deadline >= Clock::get()?.unix_timestamp,
            ErrorCode::MilestoneNotDue
        );

        milestone.status = MilestoneStatus::Submitted;
        milestone.submitted_at = Some(Clock::get()?.unix_timestamp);

        Ok(())
    }

    pub fn approve_milestone(
        ctx: Context<ApproveMilestone>,
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
            milestone.status == MilestoneStatus::Submitted,
            ErrorCode::InvalidApplicationStatus
        );

        let amount = milestone.amount;

        milestone.status = MilestoneStatus::Approved;
        milestone.approved_at = Some(Clock::get()?.unix_timestamp);

        // Transfer milestone amount to freelancer
        let tf = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.job.to_account_info(),
                to: ctx.accounts.freelancer.to_account_info(),
            },
        );
        transfer(tf, amount)?;

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
            milestone.status == MilestoneStatus::Submitted,
            ErrorCode::InvalidApplicationStatus
        );

        milestone.status = MilestoneStatus::Rejected;

        Ok(())
    }
}

// =============================================================================
// ACCOUNTS CONTEXTS
// =============================================================================

// Config Accounts
#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init, payer = authority, space = Config::INIT_SPACE + 8, seeds = [b"config"], bump)]
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
pub struct WithdrawTreasury<'info> {
    pub admin: Signer<'info>,
    #[account(mut, seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub treasury: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateTreasury<'info> {
    pub admin: Signer<'info>,
    #[account(mut, seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
}

// User Accounts
#[derive(Accounts)]
pub struct CreateUser<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init, payer = authority, space = User::INIT_SPACE + 8, seeds = [b"user", authority.key().as_ref()], bump)]
    pub user: Account<'info, User>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddWallet<'info> {
    pub authority: Signer<'info>,
    #[account(mut, seeds = [b"user", authority.key().as_ref()], bump = user.bump)]
    pub user: Account<'info, User>,
}

#[derive(Accounts)]
pub struct SetActiveWallet<'info> {
    pub authority: Signer<'info>,
    #[account(mut, seeds = [b"user", authority.key().as_ref()], bump = user.bump)]
    pub user: Account<'info, User>,
}

#[derive(Accounts)]
pub struct UpdateUser<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut, seeds = [b"user", authority.key().as_ref()], bump = user.bump)]
    pub user: Account<'info, User>,
}

// Team Accounts
#[derive(Accounts)]
pub struct CreateTeam<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(init, payer = owner, space = Team::INIT_SPACE + 8, seeds = [b"team", owner.key().as_ref()], bump)]
    pub team: Account<'info, Team>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddTeamMember<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut, seeds = [b"team", owner.key().as_ref()], bump = team.bump)]
    pub team: Account<'info, Team>,
}

// Job Accounts
#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct CreateJob<'info> {
    #[account(mut)]
    pub client: Signer<'info>,
    #[account(init, payer = client, space = Job::INIT_SPACE + 8, seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump)]
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
    #[account(mut, seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct ApplyToJob<'info> {
    #[account(mut)]
    pub applicant: Signer<'info>,
    #[account(mut, seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
    /// CHECK: Client account validated by job PDA
    pub client: UncheckedAccount<'info>,
    /// CHECK: Optional team account
    pub team: Option<UncheckedAccount<'info>>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct AcceptApplication<'info> {
    #[account(mut)]
    pub client: Signer<'info>,
    #[account(mut, seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct SubmitWork<'info> {
    pub freelancer: Signer<'info>,
    #[account(mut, seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
    /// CHECK: Client account validated by job PDA
    pub client: UncheckedAccount<'info>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct ApproveWork<'info> {
    pub client: Signer<'info>,
    #[account(mut, seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
    #[account(mut, seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    pub freelancer: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct RejectWork<'info> {
    pub client: Signer<'info>,
    #[account(mut, seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct CancelJob<'info> {
    pub client: Signer<'info>,
    #[account(mut, seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

// Arbiter Accounts
#[derive(Accounts)]
pub struct CreateArbiterPool<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(init, payer = admin, space = ArbiterPool::INIT_SPACE + 8, seeds = [b"arbiter_pool"], bump)]
    pub pool: Account<'info, ArbiterPool>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddArbiter<'info> {
    pub admin: Signer<'info>,
    #[account(mut, seeds = [b"arbiter_pool"], bump = pool.bump)]
    pub pool: Account<'info, ArbiterPool>,
}

#[derive(Accounts)]
pub struct RemoveArbiter<'info> {
    pub admin: Signer<'info>,
    #[account(mut, seeds = [b"arbiter_pool"], bump = pool.bump)]
    pub pool: Account<'info, ArbiterPool>,
}

// Dispute Accounts
#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct RaiseDispute<'info> {
    #[account(mut)]
    pub raiser: Signer<'info>,
    #[account(mut, seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
    #[account(init, payer = raiser, space = Dispute::INIT_SPACE + 8, seeds = [b"dispute", job.key().as_ref()], bump)]
    pub dispute: Account<'info, Dispute>,
    /// CHECK: Client account validated by job PDA
    pub client: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct SubmitEvidence<'info> {
    #[account(mut)]
    pub submitter: Signer<'info>,
    #[account(mut, seeds = [b"dispute", job.key().as_ref()], bump = dispute.bump)]
    pub dispute: Account<'info, Dispute>,
    #[account(mut, seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
    /// CHECK: Client account validated by job PDA
    pub client: UncheckedAccount<'info>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct AssignArbiter<'info> {
    pub arbiter: Signer<'info>,
    #[account(mut, seeds = [b"dispute", job.key().as_ref()], bump = dispute.bump)]
    pub dispute: Account<'info, Dispute>,
    #[account(mut, seeds = [b"arbiter_pool"], bump = pool.bump)]
    pub pool: Account<'info, ArbiterPool>,
    #[account(mut, seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
    /// CHECK: Client account validated by job PDA
    pub client: UncheckedAccount<'info>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct ResolveDispute<'info> {
    pub arbiter: Signer<'info>,
    #[account(mut, seeds = [b"dispute", job.key().as_ref()], bump = dispute.bump)]
    pub dispute: Account<'info, Dispute>,
    #[account(mut, seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
    /// CHECK: Client account validated by job PDA
    pub client: UncheckedAccount<'info>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct FinalizeDisputePayouts<'info> {
    pub arbiter: Signer<'info>,
    #[account(mut, seeds = [b"dispute", job.key().as_ref()], bump = dispute.bump)]
    pub dispute: Account<'info, Dispute>,
    #[account(mut, seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
    #[account(mut)]
    pub client: SystemAccount<'info>,
    #[account(mut)]
    pub freelancer: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

// Milestone Accounts
#[derive(Accounts)]
#[instruction(job_id: u64, index: u8)]
pub struct CreateMilestone<'info> {
    #[account(mut)]
    pub client: Signer<'info>,
    #[account(seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
    #[account(init, payer = client, space = Milestone::INIT_SPACE + 8, seeds = [b"milestone", job.key().as_ref(), &[index]], bump)]
    pub milestone: Account<'info, Milestone>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(job_id: u64, milestone_index: u8)]
pub struct SubmitMilestone<'info> {
    pub freelancer: Signer<'info>,
    #[account(mut, seeds = [b"milestone", job.key().as_ref(), &[milestone_index]], bump = milestone.bump)]
    pub milestone: Account<'info, Milestone>,
    #[account(mut, seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
    /// CHECK: Client account validated by job PDA
    pub client: UncheckedAccount<'info>,
}

#[derive(Accounts)]
#[instruction(job_id: u64, milestone_index: u8)]
pub struct ApproveMilestone<'info> {
    pub client: Signer<'info>,
    #[account(mut, seeds = [b"milestone", job.key().as_ref(), &[milestone_index]], bump = milestone.bump)]
    pub milestone: Account<'info, Milestone>,
    #[account(mut, seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
    #[account(mut)]
    pub freelancer: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(job_id: u64, milestone_index: u8)]
pub struct RejectMilestone<'info> {
    pub client: Signer<'info>,
    #[account(mut, seeds = [b"milestone", job.key().as_ref(), &[milestone_index]], bump = milestone.bump)]
    pub milestone: Account<'info, Milestone>,
    #[account(mut, seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
}
