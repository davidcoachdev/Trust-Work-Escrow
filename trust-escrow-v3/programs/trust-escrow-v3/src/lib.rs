use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

declare_id!("J1c4QsjbV9bFEPrFQZZGe8GrGWFxNhtAhhrxJFK2xc1h");

// =============================================================================
// CONSTANTS
// =============================================================================

/// Fee expresado en basis points (10000 = 100%). Corrige el bug de v2 que
/// dividía por 10000 pero validaba 0-100, cobrando 100x menos.
const BASIS_POINTS: u16 = 10_000;

/// Fee de arbitraje por parte, en basis points (250 = 2.5%).
/// Se cobra 2.5% al cliente Y 2.5% al freelancer (5% total del job) SOLO cuando
/// hay arbitraje, para pagar al arbitro neutral asignado por la plataforma.
const ARBITER_FEE_BPS_PER_PARTY: u16 = 250;

/// Ventana para que la contraparte acepte la disputa antes de ir a asesor.
const DISPUTE_ACCEPT_GRACE: i64 = 7 * 24 * 60 * 60; // 7 dias en segundos

/// Tiempo maximo que un job puede estar pausado antes de expirar (30 dias).
const MAX_PAUSE_DURATION: i64 = 30 * 24 * 60 * 60;

const MIN_JOB_AMOUNT: u64 = 100_000; // 0.0001 SOL
const MAX_TITLE_LENGTH: usize = 100;
const MAX_DESCRIPTION_LENGTH: usize = 500;
const MAX_PROPOSAL_LENGTH: usize = 512;
const MAX_DISPUTE_REASON: usize = 500;
const MAX_DISPUTE_EVIDENCE: usize = 2048;
const MAX_MILESTONE_TITLE: usize = 64;
const MAX_MILESTONES: usize = 20;
const MAX_APPLICATIONS: usize = 50;
const MAX_ARBITERS: usize = 50;

// =============================================================================
// ERRORS
// =============================================================================

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
    // Disputes
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
    // Milestones
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
}

// =============================================================================
// STATE ENUMS
// =============================================================================

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

// =============================================================================
// STATE STRUCTS
// =============================================================================

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub authority: Pubkey,
    /// Asesor de plataforma: resuelve PlatformCase / disputeos no mutuos.
    pub advisor: Pubkey,
    /// Wallet que recibe las fees del protocolo. Debe firmar en withdraw_treasury.
    pub treasury: Pubkey,
    /// Cuenta SEPARADA de la empresa que recibe las fees de arbitraje (5%).
    /// Aislarla de `treasury` permite llevar saldos de arbitraje por separado
    /// (buena gestion contable). NUNCA va al wallet personal del asesor/arbitro.
    pub arbitration_treasury: Pubkey,
    /// Fee en basis points (10000 = 100%).
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
    /// Control de milestones para no liberar de mas de lo depositado.
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
pub struct ArbitrationEscrow {
    pub job: Pubkey,
    /// Bono de 2.5% posteado por el cliente al abrir la disputa.
    pub client_bond: u64,
    /// Bono de 2.5% posteado por el freelancer al aceptar.
    pub freelancer_bond: u64,
    pub bump: u8,
}

// =============================================================================
// HELPERS
// =============================================================================

/// Calcula la fee con aritmetica chequeada. `fee_bps` debe validarse contra
/// BASIS_POINTS antes de llamar.
pub fn compute_fee(amount: u64, fee_bps: u16) -> Result<u64> {
    let fee = (amount as u128)
        .checked_mul(fee_bps as u128)
        .ok_or(ErrorCode::MathOverflow)?
        / BASIS_POINTS as u128;
    Ok(fee as u64)
}

/// Bloquea instrucciones si el job esta pausado (avisa si ya expiro).
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

// =============================================================================
// PROGRAM MODULE
// =============================================================================

#[program]
pub mod escrow {
    use super::*;

    // -------------------------------------------------------------------------
    // CONFIG MODULE  (portado de v1/v2, corregido)
    // -------------------------------------------------------------------------

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

    /// Retira lamports acumulados en la wallet `treasury`.
    /// CORRECCION v2: `treasury` es Signer (debe firmar) y se valida contra
    /// config.treasury. Los fondos provienen de la wallet treasury, no del PDA config.
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

    /// Retira lamports acumulados en la cuenta de arbitraje de la empresa.
    /// Misma lógica que `withdraw_treasury` pero sobre `arbitration_treasury`
    /// (cuenta separada para fees de arbitraje).
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

    // -------------------------------------------------------------------------
    // JOBS MODULE
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

        // Fee de plataforma calculada con aritmetica chequeada.
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

        // Cliente firma: transferencia normal al PDA job.
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

    pub fn accept_job(ctx: Context<AcceptJob>, _job_id: u64) -> Result<()> {
        let job = &mut ctx.accounts.job;
        require!(job.status == JobStatus::Funded, ErrorCode::InvalidJobStatus);
        check_not_paused(job)?;
        require!(
            ctx.accounts.freelancer.key() != job.client,
            ErrorCode::CannotWorkOnOwnJob
        );

        job.freelancer = Some(ctx.accounts.freelancer.key());
        job.status = JobStatus::InProgress;
        job.updated_at = Clock::get()?.unix_timestamp;

        msg!("Job accepted by: {}", ctx.accounts.freelancer.key());
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
        // Si hay milestones, deben estar todos aprobados antes del release final.
        require!(
            job.milestones_total == 0 || job.milestones_approved == job.milestones_total,
            ErrorCode::AllMilestonesRequired
        );

        // Paga el resto (lo no cubierto por milestones ya aprobados).
        let amount = job
            .amount
            .checked_sub(job.milestones_amount_total)
            .ok_or(ErrorCode::MathOverflow)?;
        let fee_amount = job.fee_amount;

        // El PDA job firma las transferencias de salida (new_with_signer).
        let client_key = ctx.accounts.client.key();
        let job_id_bytes = _job_id.to_le_bytes();
        let seeds: &[&[&[u8]]] = &[&[
            b"job".as_ref(),
            client_key.as_ref(),
            job_id_bytes.as_ref(),
            &[job.bump],
        ]];

        // Paga al freelancer el principal.
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

        // Paga la comision de plataforma al treasury.
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

        // Vuelve a InProgress para que el freelancer corrija y reenvie.
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

    /// Pausa el job (solo si no hay freelancer asignado: Created/Funded).
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

    /// Reanuda un job pausado (cliente).
    pub fn unpause_job(ctx: Context<UnpauseJob>, _job_id: u64) -> Result<()> {
        let job = &mut ctx.accounts.job;
        require!(job.client == ctx.accounts.client.key(), ErrorCode::NotJobClient);
        require!(job.paused, ErrorCode::JobPaused);
        job.paused = false;
        job.paused_at = 0;
        job.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Expira un job pausado hace mas de MAX_PAUSE_DURATION: reembolsa (si fondeado)
    /// y cierra. Cualquiera puede llamarlo para liberar fondos atrapados.
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

    // -------------------------------------------------------------------------
    // ARBITER POOL MODULE
    // -------------------------------------------------------------------------

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

    // -------------------------------------------------------------------------
    // DISPUTES MODULE
    // -------------------------------------------------------------------------

    /// Abre una disputa. El que abre (raiser) firma y postea su bono de 2.5%.
    /// Crea `Dispute` y `ArbitrationEscrow`. Job -> Disputed.
    pub fn raise_dispute(ctx: Context<RaiseDispute>, _job_id: u64, reason: String) -> Result<()> {
        require!(
            ctx.accounts.job.status == JobStatus::Submitted
                || ctx.accounts.job.status == JobStatus::InProgress,
            ErrorCode::CannotDisputeAtStage
        );
        require!(!reason.is_empty(), ErrorCode::EmptyDisputeReason);
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

        // Raiser postea su bono al escrow de arbitraje (firma como origen).
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

    /// La contraparte acepta la disputa y postea su bono de 2.5%. Dispute -> Active.
    pub fn accept_dispute(ctx: Context<AcceptDispute>, _job_id: u64) -> Result<()> {
        let dispute = &mut ctx.accounts.dispute;
        require!(dispute.status == DisputeStatus::Open, ErrorCode::DisputeAlreadyResolved);
        // La aceptacion solo es valida dentro de la gracia; despues resuelve el asesor.
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

    /// Plataforma asigna un arbitro del pool (solo si la disputa es mutua: Active).
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
        // El arbitro no puede ser el cliente ni el freelancer (neutralidad).
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

    /// Arbitro asignado resuelve el reparto (mutuo).
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

    /// Asesor resuelve cuando NO hubo arbitro mutuo (una parte no acepto / fallo).
    pub fn resolve_platform_case(
        ctx: Context<ResolvePlatformCase>,
        _job_id: u64,
        client_payout_percent: u8,
    ) -> Result<()> {
        let config = &ctx.accounts.config;
        require!(config.advisor == ctx.accounts.advisor.key(), ErrorCode::NotAuthorized);
        // El asesor no puede ser el cliente ni el freelancer.
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
        // El asesor resuelve de oficio SOLO si no hubo interaccion del arbitro:
        // (a) arbitro asignado pero fallo (status ArbiterAssigned), o
        // (b) ningun arbitro fue asignado Y vencio la gracia (Open / Active /
        //     EvidenceSubmitted). No se secuestra una disputa que el arbitro
        //     esta tratando activamente; la gracia solo fuerza el takeover de
        //     plataforma cuando nadie interactuo.
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

    /// Cualquiera abre caso de plataforma si la contraparte no acepto.
    pub fn request_platform_intervention(
        ctx: Context<RequestPlatformIntervention>,
        _job_id: u64,
    ) -> Result<()> {
        let dispute = &mut ctx.accounts.dispute;
        require!(dispute.status == DisputeStatus::Open, ErrorCode::DisputeAlreadyResolved);
        // Solo dentro de la gracia; despues solo el asesor puede resolver.
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
        // La disputa queda habilitada para que el asesor resuelva.
        dispute.status = DisputeStatus::EvidenceSubmitted;
        Ok(())
    }

    /// Paga y cierra. Resolver = arbitro asignado o asesor de plataforma.
    /// El PDA `job` firma. El `ArbitrationEscrow` se cierra hacia el resolver (5%).
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
        // El reparto de la disputa aplica solo al resto no pagado por milestones.
        let amount = job
            .amount
            .checked_sub(job.milestones_amount_total)
            .ok_or(ErrorCode::MathOverflow)?;
        let fee_amount = job.fee_amount;
        let client_pct = dispute.client_payout_percent;
        let freelancer_pct = dispute.freelancer_payout_percent;
        // Fee de arbitraje = 5% del monto en disputa (lo mismo que postearon las partes).
        let resolver_fee_total = compute_fee(amount, ARBITER_FEE_BPS_PER_PARTY * 2)?;
        // Bonos efectivamente posteados en el ArbitrationEscrow (1 o 2 de 2.5%).
        let posted = ctx
            .accounts
            .escrow
            .client_bond
            .checked_add(ctx.accounts.escrow.freelancer_bond)
            .ok_or(ErrorCode::MathOverflow)?;
        // Lo no posteado se recupera del reparto y se paga al resolutor (5% "les guste o no").
        let shortfall = resolver_fee_total.saturating_sub(posted);
        // Lo que queda para cliente y freelancer.
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

        // Comision de plataforma a treasury.
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

        // Parte del cliente.
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

        // Parte del freelancer.
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

        // Recupera del reparto lo que la contraparte no posteo y lo envia a la cuenta
        // de arbitraje de la empresa (no al wallet del resolver).
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

        // El cierre de `escrow` (close = arbitration_treasury) envia los bonos
        // posteados (hasta 5% de lo disputado) a la cuenta de arbitraje de la
        // empresa; el PDA job cierra hacia el cliente.
        msg!("Dispute finalized for job: {}", job.key());
        Ok(())
    }

    // -------------------------------------------------------------------------
    // MILESTONES MODULE
    // -------------------------------------------------------------------------

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
        // Los milestones son secuenciales: el indice debe coincidir con el conteo
        // actual (0, 1, 2, ...). Evita indices duplicados/saltados y desalineacion
        // con el PDA `milestone` (seed incluye el indice).
        require!(index == job.milestones_total, ErrorCode::InvalidMilestoneIndex);
        require!(
            job.milestones_total < MAX_MILESTONES as u8,
            ErrorCode::MilestoneAlreadyCompleted
        );

        // La suma de montos de milestones no puede superar el depositado.
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
        require!(milestone.status == MilestoneStatus::Pending, ErrorCode::MilestoneAlreadyCompleted);

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

    // -------------------------------------------------------------------------
    // (fin de modulos)
    // -------------------------------------------------------------------------
}

// =============================================================================
// ACCOUNTS CONTEXTS
// =============================================================================

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
    /// Debe coincidir con config.treasury y firmar la transferencia.
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
    /// Debe coincidir con config.arbitration_treasury y firmar la transferencia.
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

// -------------------------------------------------------------------------
// JOBS ACCOUNTS
// -------------------------------------------------------------------------

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

// -------------------------------------------------------------------------
// ACCEPT JOB ACCOUNTS
// -------------------------------------------------------------------------

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct AcceptJob<'info> {
    pub freelancer: Signer<'info>,
    /// CHECK: client validado por el PDA del job.
    pub client: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()],
        bump = job.bump
    )]
    pub job: Account<'info, Job>,
}

// -------------------------------------------------------------------------
// ARBITER POOL ACCOUNTS
// -------------------------------------------------------------------------

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

// -------------------------------------------------------------------------
// DISPUTES ACCOUNTS
// -------------------------------------------------------------------------

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct RaiseDispute<'info> {
    #[account(mut)]
    pub raiser: Signer<'info>,
    /// CHECK: client validado por el PDA del job.
    pub client: UncheckedAccount<'info>,
    #[account(mut, seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
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
    /// Validada contra config.arbitration_treasury. Nunca va al wallet del resolver.
    #[account(
        mut,
        constraint = arbitration_treasury.key() == config.arbitration_treasury @ ErrorCode::InvalidTreasury
    )]
    pub arbitration_treasury: UncheckedAccount<'info>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

// -------------------------------------------------------------------------
// MILESTONES ACCOUNTS
// -------------------------------------------------------------------------

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
