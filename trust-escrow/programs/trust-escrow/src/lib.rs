use anchor_lang::prelude::*;
use anchor_lang::system_program;

declare_id!("5gu5JCSpB8MKyJzhXpGaCt8SruAMnRD6cTPbwPX6JTYo");

const FEE_PERCENT: u8 = 5;
const MAX_TITLE_LENGTH: usize = 100;
const MAX_DESCRIPTION_LENGTH: usize = 500;
const MIN_JOB_AMOUNT: u64 = 100_000; // 0.0001 SOL minimum

#[program]
pub mod trust_escrow {
    use super::*;

    pub fn initialize_config(ctx: Context<InitializeConfig>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.authority = ctx.accounts.authority.key();
        config.treasury = ctx.accounts.treasury.key();
        config.fee_percent = FEE_PERCENT;
        config.paused = false;
        config.bump = ctx.bumps.config;
        msg!("Config initialized by: {}", config.authority);
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

        let job = &mut ctx.accounts.job;
        let clock = Clock::get()?;

        job.client = ctx.accounts.client.key();
        job.freelancer = None;
        job.arbiter = ctx.accounts.arbiter.key();
        job.amount = amount;
        job.fee_percent = FEE_PERCENT;
        job.fee_amount = (amount as u128 * FEE_PERCENT as u128 / 100) as u64;
        job.status = JobStatus::Created;
        job.title = title;
        job.description = description;
        job.deadline = deadline;
        job.created_at = clock.unix_timestamp;
        job.updated_at = clock.unix_timestamp;
        job.dispute_reason = String::new();
        job.bump = ctx.bumps.job;

        msg!("Job created: {} - Amount: {} lamports", job.key(), amount);
        Ok(())
    }

    pub fn deposit_funds(ctx: Context<DepositFunds>, _job_id: u64) -> Result<()> {
        let job = &mut ctx.accounts.job;

        require!(
            job.status == JobStatus::Created,
            ErrorCode::InvalidJobStatus
        );
        require!(
            ctx.accounts.client.key() == job.client,
            ErrorCode::NotJobClient
        );

        let transfer_amount = job.amount + job.fee_amount;

        // CPI to system program for safe transfer
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.client.to_account_info(),
                    to: job.to_account_info(),
                },
            ),
            transfer_amount,
        )?;

        job.status = JobStatus::Funded;
        job.updated_at = Clock::get()?.unix_timestamp;

        msg!("Funds deposited: {} lamports", transfer_amount);
        Ok(())
    }

    pub fn accept_job(ctx: Context<AcceptJob>, _job_id: u64) -> Result<()> {
        let job = &mut ctx.accounts.job;

        require!(job.status == JobStatus::Funded, ErrorCode::InvalidJobStatus);
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
            job.status == JobStatus::InProgress,
            ErrorCode::InvalidJobStatus
        );
        require!(
            ctx.accounts.freelancer.key() == job.freelancer.unwrap(),
            ErrorCode::NotJobFreelancer
        );

        job.status = JobStatus::Submitted;
        job.updated_at = Clock::get()?.unix_timestamp;

        msg!("Work submitted for job: {}", job.key());
        Ok(())
    }

    pub fn approve_work(ctx: Context<ApproveWork>, _job_id: u64) -> Result<()> {
        let job = &mut ctx.accounts.job;

        require!(
            job.status == JobStatus::Submitted,
            ErrorCode::InvalidJobStatus
        );
        require!(
            ctx.accounts.client.key() == job.client,
            ErrorCode::NotJobClient
        );
        require!(job.freelancer.is_some(), ErrorCode::NoFreelancerAssigned);

        let payment_amount = job.amount;
        let fee_amount = job.fee_amount;

        // Pay freelancer their earned amount
        **job.to_account_info().lamports.borrow_mut() -= payment_amount;
        **ctx
            .accounts
            .freelancer
            .to_account_info()
            .lamports
            .borrow_mut() += payment_amount;

        // Pay treasury the protocol fee
        **job.to_account_info().lamports.borrow_mut() -= fee_amount;
        **ctx
            .accounts
            .treasury
            .to_account_info()
            .lamports
            .borrow_mut() += fee_amount;

        // Remaining rent-exempt lamports returned to client via close = client
        job.status = JobStatus::Released;
        job.updated_at = Clock::get()?.unix_timestamp;

        msg!(
            "Work approved. Payment: {} to freelancer, {} fee to treasury",
            payment_amount,
            fee_amount
        );
        Ok(())
    }

    pub fn reject_work(ctx: Context<RejectWork>, _job_id: u64, reason: String) -> Result<()> {
        let job = &mut ctx.accounts.job;

        require!(
            job.status == JobStatus::Submitted,
            ErrorCode::InvalidJobStatus
        );
        require!(
            ctx.accounts.client.key() == job.client,
            ErrorCode::NotJobClient
        );

        require!(!reason.is_empty(), ErrorCode::EmptyDisputeReason);

        job.status = JobStatus::Disputed;
        job.dispute_reason = reason;
        job.updated_at = Clock::get()?.unix_timestamp;

        msg!("Work rejected, dispute opened for job: {}", job.key());
        Ok(())
    }

    pub fn raise_dispute(ctx: Context<RaiseDispute>, _job_id: u64, reason: String) -> Result<()> {
        let job = &mut ctx.accounts.job;

        require!(
            job.status == JobStatus::Submitted,
            ErrorCode::InvalidJobStatus
        );
        require!(
            ctx.accounts.freelancer.key() == job.freelancer.unwrap(),
            ErrorCode::NotJobFreelancer
        );

        require!(!reason.is_empty(), ErrorCode::EmptyDisputeReason);

        job.status = JobStatus::Disputed;
        job.dispute_reason = reason;
        job.updated_at = Clock::get()?.unix_timestamp;

        msg!("Dispute raised for job: {}", job.key());
        Ok(())
    }

    pub fn resolve_dispute(
        ctx: Context<ResolveDispute>,
        _job_id: u64,
        freelancer_percent: u8,
    ) -> Result<()> {
        let job = &mut ctx.accounts.job;

        require!(
            job.status == JobStatus::Disputed,
            ErrorCode::InvalidJobStatus
        );
        require!(freelancer_percent <= 100, ErrorCode::InvalidPercent);

        let freelancer_amount = (job.amount as u128 * freelancer_percent as u128 / 100) as u64;
        let fee_amount = job.fee_amount;

        // Pay freelancer their portion
        if freelancer_amount > 0 {
            **job.to_account_info().lamports.borrow_mut() -= freelancer_amount;
            **ctx
                .accounts
                .freelancer
                .to_account_info()
                .lamports
                .borrow_mut() += freelancer_amount;
        }

        // Pay treasury the protocol fee
        **job.to_account_info().lamports.borrow_mut() -= fee_amount;
        **ctx
            .accounts
            .treasury
            .to_account_info()
            .lamports
            .borrow_mut() += fee_amount;

        // Remaining (client_amount + rent) returned to client via close = client
        job.status = JobStatus::Resolved;
        job.updated_at = Clock::get()?.unix_timestamp;

        msg!(
            "Dispute resolved: {}% freelancer, {}% client",
            freelancer_percent,
            100 - freelancer_percent
        );
        Ok(())
    }

    pub fn cancel_job(ctx: Context<CancelJob>, _job_id: u64) -> Result<()> {
        let job = &mut ctx.accounts.job;

        require!(
            job.status == JobStatus::Created || job.status == JobStatus::Funded,
            ErrorCode::InvalidJobStatus
        );
        require!(
            ctx.accounts.client.key() == job.client,
            ErrorCode::NotJobClient
        );

        if job.status == JobStatus::Funded {
            let total = job.amount + job.fee_amount;
            **job.to_account_info().lamports.borrow_mut() -= total;
            **ctx.accounts.client.to_account_info().lamports.borrow_mut() += total;
        }

        job.status = JobStatus::Cancelled;
        job.updated_at = Clock::get()?.unix_timestamp;

        msg!("Job cancelled: {}", job.key());
        Ok(())
    }

    pub fn pause_program(ctx: Context<PauseProgram>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        require!(
            config.authority == ctx.accounts.authority.key(),
            ErrorCode::NotAuthorized
        );

        config.paused = true;
        msg!("Program paused");
        Ok(())
    }

    pub fn unpause_program(ctx: Context<UnpauseProgram>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        require!(
            config.authority == ctx.accounts.authority.key(),
            ErrorCode::NotAuthorized
        );

        config.paused = false;
        msg!("Program unpaused");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: Treasury wallet to receive protocol fees. Set by admin.
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
#[instruction(job_id: u64)]
pub struct CreateJob<'info> {
    #[account(mut)]
    pub client: Signer<'info>,
    /// CHECK: Designated arbiter for dispute resolution. Stored in job.
    pub arbiter: UncheckedAccount<'info>,
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
pub struct AcceptJob<'info> {
    #[account(mut)]
    pub freelancer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"job", job.client.as_ref(), &job_id.to_le_bytes()],
        bump = job.bump
    )]
    pub job: Account<'info, Job>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct SubmitWork<'info> {
    #[account(mut)]
    pub freelancer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"job", job.client.as_ref(), &job_id.to_le_bytes()],
        bump = job.bump
    )]
    pub job: Account<'info, Job>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct ApproveWork<'info> {
    #[account(mut)]
    pub client: Signer<'info>,
    #[account(
        mut,
        seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()],
        bump = job.bump,
        close = client,
        constraint = job.freelancer == Some(freelancer.key()) @ ErrorCode::NotJobFreelancer
    )]
    pub job: Account<'info, Job>,
    #[account(mut)]
    pub freelancer: SystemAccount<'info>,
    /// CHECK: Treasury receives fees. Validated against config.treasury.
    #[account(
        mut,
        constraint = treasury.key() == config.treasury @ ErrorCode::InvalidTreasury
    )]
    pub treasury: UncheckedAccount<'info>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct RejectWork<'info> {
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
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct RaiseDispute<'info> {
    #[account(mut)]
    pub freelancer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"job", job.client.as_ref(), &job_id.to_le_bytes()],
        bump = job.bump
    )]
    pub job: Account<'info, Job>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct ResolveDispute<'info> {
    /// Arbiter resolving the dispute (must match job.arbiter)
    pub arbiter: Signer<'info>,
    #[account(mut)]
    pub client: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()],
        bump = job.bump,
        close = client,
        has_one = arbiter @ ErrorCode::NotAuthorized,
        constraint = job.freelancer == Some(freelancer.key()) @ ErrorCode::NotJobFreelancer
    )]
    pub job: Account<'info, Job>,
    #[account(mut)]
    pub freelancer: SystemAccount<'info>,
    /// CHECK: Treasury receives fees. Validated against config.treasury.
    #[account(
        mut,
        constraint = treasury.key() == config.treasury @ ErrorCode::InvalidTreasury
    )]
    pub treasury: UncheckedAccount<'info>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
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
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
pub struct PauseProgram<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
pub struct UnpauseProgram<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,
}

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub fee_percent: u8,
    pub paused: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Job {
    pub client: Pubkey,
    pub freelancer: Option<Pubkey>,
    pub arbiter: Pubkey,
    pub amount: u64,
    pub fee_percent: u8,
    pub fee_amount: u64,
    pub status: JobStatus,
    #[max_len(100)]
    pub title: String,
    #[max_len(500)]
    pub description: String,
    pub deadline: i64,
    pub created_at: i64,
    pub updated_at: i64,
    #[max_len(200)]
    pub dispute_reason: String,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug, InitSpace)]
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

#[error_code]
pub enum ErrorCode {
    #[msg("El programa está pausado")]
    ProgramPaused,
    #[msg("El monto es muy pequeño")]
    AmountTooSmall,
    #[msg("El título no puede estar vacío")]
    EmptyTitle,
    #[msg("El título excede el largo máximo")]
    TitleTooLong,
    #[msg("La descripción excede el largo máximo")]
    DescriptionTooLong,
    #[msg("Estado de job inválido para esta operación")]
    InvalidJobStatus,
    #[msg("No eres el cliente de este trabajo")]
    NotJobClient,
    #[msg("No eres el freelancer de este trabajo")]
    NotJobFreelancer,
    #[msg("No puedes trabajar en tu propio proyecto")]
    CannotWorkOnOwnJob,
    #[msg("No hay freelancer asignado")]
    NoFreelancerAssigned,
    #[msg("La razón de la disputa no puede estar vacía")]
    EmptyDisputeReason,
    #[msg("Porcentaje inválido")]
    InvalidPercent,
    #[msg("No autorizado")]
    NotAuthorized,
    #[msg("Treasury inválido")]
    InvalidTreasury,
}
