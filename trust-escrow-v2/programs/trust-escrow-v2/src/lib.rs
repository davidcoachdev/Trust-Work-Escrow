use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

declare_id!("TesT3XPqD3WFFVTY4BTwZ3sJpY7C7hF3Z6K2oX3i7jB");

const MAX_USERNAME_LENGTH: usize = 32;
const MAX_BIO_LENGTH: usize = 500;
const MIN_JOB_AMOUNT: u64 = 100_000;
const MAX_TITLE_LENGTH: usize = 64;
const MAX_DESCRIPTION_LENGTH: usize = 1024;

#[error_code]
pub enum ErrorCode {
    #[msg("User already exists")]
    UserAlreadyExists,
    #[msg("Wallet already associated")]
    WalletAlreadyAssociated,
    #[msg("Wallet not associated")]
    WalletNotAssociated,
    #[msg("Not authorized")]
    NotAuthorized,
    #[msg("Program paused")]
    ProgramPaused,
    #[msg("Invalid job status")]
    InvalidJobStatus,
    #[msg("Amount too small")]
    AmountTooSmall,
    #[msg("Title empty")]
    EmptyTitle,
    #[msg("Title too long")]
    TitleTooLong,
    #[msg("Description too long")]
    DescriptionTooLong,
    #[msg("Username empty")]
    EmptyUsername,
    #[msg("Username too long")]
    UsernameTooLong,
    #[msg("Bio too long")]
    BioTooLong,
    #[msg("Invalid fee")]
    InvalidFeePercentage,
    #[msg("Insufficient funds")]
    InsufficientFunds,
    #[msg("No freelancer")]
    NoFreelancerAssigned,
    #[msg("Cannot accept own job")]
    CannotAcceptOwnJob,
}

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub admin: Pubkey,
    pub treasury: Pubkey,
    #[max_len(5)]
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
    #[max_len(5)]
    pub wallets: Vec<Pubkey>,
    pub active_wallet: Pubkey,
    #[max_len(32)]
    pub username: String,
    #[max_len(500)]
    pub bio: Option<String>,
    pub created_at: i64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Job {
    pub client: Pubkey,
    pub freelancer: Option<Pubkey>,
    #[max_len(64)]
    pub title: String,
    #[max_len(1024)]
    pub description: String,
    pub amount: u64,
    pub fee: u64,
    pub deadline: i64,
    pub status: u8,
    pub bump: u8,
    pub created_at: i64,
    pub updated_at: i64,
}

#[program]
pub mod escrow {
    use super::*;

    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        multisig_owners: Vec<Pubkey>,
        multisig_threshold: u8,
        treasury: Pubkey,
        fee_percent: u8,
    ) -> Result<()> {
        require!(fee_percent <= 100, ErrorCode::InvalidFeePercentage);
        require!(!multisig_owners.is_empty(), ErrorCode::NotAuthorized);
        require!(multisig_threshold >= 1, ErrorCode::InvalidFeePercentage);
        require!(
            multisig_threshold as usize <= multisig_owners.len(),
            ErrorCode::InvalidFeePercentage
        );
        let c = &mut ctx.accounts.config;
        c.admin = ctx.accounts.authority.key();
        c.treasury = treasury;
        c.multisig_owners = multisig_owners;
        c.multisig_threshold = multisig_threshold;
        c.fee_percent = fee_percent;
        c.paused = false;
        c.bump = ctx.bumps.config;
        Ok(())
    }

    pub fn pause(ctx: Context<Pause>) -> Result<()> {
        let c = &mut ctx.accounts.config;
        require!(
            ctx.accounts.authority.key() == c.admin,
            ErrorCode::NotAuthorized
        );
        c.paused = true;
        Ok(())
    }

    pub fn unpause(ctx: Context<Unpause>) -> Result<()> {
        let c = &mut ctx.accounts.config;
        require!(
            ctx.accounts.authority.key() == c.admin,
            ErrorCode::NotAuthorized
        );
        c.paused = false;
        Ok(())
    }

    pub fn withdraw_treasury(ctx: Context<WithdrawTreasury>, amount: u64) -> Result<()> {
        let c = &ctx.accounts.config;
        require!(
            ctx.accounts.authority.key() == c.admin,
            ErrorCode::NotAuthorized
        );
        require!(
            ctx.accounts.treasury.lamports() >= amount,
            ErrorCode::InsufficientFunds
        );
        let t = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.treasury.to_account_info(),
                to: ctx.accounts.authority.to_account_info(),
            },
        );
        transfer(t, amount)?;
        Ok(())
    }

    pub fn create_user(ctx: Context<CreateUser>, username: String) -> Result<()> {
        require!(!username.is_empty(), ErrorCode::EmptyUsername);
        require!(
            username.len() <= MAX_USERNAME_LENGTH,
            ErrorCode::UsernameTooLong
        );
        let u = &mut ctx.accounts.user;
        u.wallet_principal = ctx.accounts.authority.key();
        u.wallets = Vec::new();
        u.active_wallet = ctx.accounts.authority.key();
        u.username = username;
        u.bio = None;
        u.created_at = Clock::get()?.unix_timestamp;
        u.bump = ctx.bumps.user;
        Ok(())
    }

    pub fn add_wallet(ctx: Context<AddWallet>, new_wallet: Pubkey) -> Result<()> {
        let u = &mut ctx.accounts.user;
        require!(
            u.wallet_principal == ctx.accounts.authority.key(),
            ErrorCode::NotAuthorized
        );
        require!(
            !u.wallets.contains(&new_wallet),
            ErrorCode::WalletAlreadyAssociated
        );
        u.wallets.push(new_wallet);
        Ok(())
    }

    pub fn set_active_wallet(ctx: Context<SetActiveWallet>, wallet: Pubkey) -> Result<()> {
        let u = &mut ctx.accounts.user;
        require!(
            u.wallet_principal == ctx.accounts.authority.key(),
            ErrorCode::NotAuthorized
        );
        let is_assoc = u.wallet_principal == wallet || u.wallets.contains(&wallet);
        require!(is_assoc, ErrorCode::WalletNotAssociated);
        u.active_wallet = wallet;
        Ok(())
    }

    pub fn update_user(ctx: Context<UpdateUser>, bio: Option<String>) -> Result<()> {
        let u = &mut ctx.accounts.user;
        require!(
            u.wallet_principal == ctx.accounts.authority.key(),
            ErrorCode::NotAuthorized
        );
        if let Some(b) = bio {
            require!(b.len() <= MAX_BIO_LENGTH, ErrorCode::BioTooLong);
            u.bio = Some(b);
        }
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
        let c = &ctx.accounts.config;
        require!(!c.paused, ErrorCode::ProgramPaused);
        require!(!title.is_empty(), ErrorCode::EmptyTitle);
        require!(title.len() <= MAX_TITLE_LENGTH, ErrorCode::TitleTooLong);
        require!(
            description.len() <= MAX_DESCRIPTION_LENGTH,
            ErrorCode::DescriptionTooLong
        );
        require!(amount >= MIN_JOB_AMOUNT, ErrorCode::AmountTooSmall);
        let fee = amount * c.fee_percent as u64 / 10000;
        let j = &mut ctx.accounts.job;
        j.client = ctx.accounts.client.key();
        j.freelancer = None;
        j.title = title;
        j.description = description;
        j.amount = amount;
        j.fee = fee;
        j.deadline = deadline;
        j.status = 0;
        j.bump = ctx.bumps.job;
        j.created_at = Clock::get()?.unix_timestamp;
        j.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn deposit_funds(ctx: Context<DepositFunds>, _job_id: u64) -> Result<()> {
        let total = ctx.accounts.job.amount + ctx.accounts.job.fee;
        let client_key = ctx.accounts.client.key();
        let t = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.client.to_account_info(),
                to: ctx.accounts.job.to_account_info(),
            },
        );
        transfer(t, total)?;
        let job = &mut ctx.accounts.job;
        require!(job.client == client_key, ErrorCode::NotAuthorized);
        require!(job.status == 0, ErrorCode::InvalidJobStatus);
        job.status = 1;
        job.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn accept_job(ctx: Context<AcceptJob>, _job_id: u64) -> Result<()> {
        let j = &mut ctx.accounts.job;
        require!(
            j.client != ctx.accounts.freelancer.key(),
            ErrorCode::CannotAcceptOwnJob
        );
        require!(j.status == 1, ErrorCode::InvalidJobStatus);
        j.freelancer = Some(ctx.accounts.freelancer.key());
        j.status = 2;
        j.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn submit_work(ctx: Context<SubmitWork>, _job_id: u64) -> Result<()> {
        let j = &mut ctx.accounts.job;
        require!(
            j.freelancer == Some(ctx.accounts.freelancer.key()),
            ErrorCode::NotAuthorized
        );
        require!(j.status == 2, ErrorCode::InvalidJobStatus);
        j.status = 3;
        j.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn approve_work(ctx: Context<ApproveWork>, _job_id: u64) -> Result<()> {
        let client_key = ctx.accounts.client.key();
        let job = &mut ctx.accounts.job;
        require!(job.client == client_key, ErrorCode::NotAuthorized);
        require!(job.status == 3, ErrorCode::InvalidJobStatus);
        require!(job.freelancer.is_some(), ErrorCode::NoFreelancerAssigned);
        let amount = job.amount;
        let fee = job.fee;
        job.status = 4;
        job.updated_at = Clock::get()?.unix_timestamp;
        let tf = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.job.to_account_info(),
                to: ctx.accounts.freelancer.to_account_info(),
            },
        );
        transfer(tf, amount)?;
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
        let j = &mut ctx.accounts.job;
        require!(
            j.client == ctx.accounts.client.key(),
            ErrorCode::NotAuthorized
        );
        require!(j.status == 3, ErrorCode::InvalidJobStatus);
        j.status = 5;
        j.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn cancel_job(ctx: Context<CancelJob>, _job_id: u64) -> Result<()> {
        let client_key = ctx.accounts.client.key();
        let refund = ctx.accounts.job.amount + ctx.accounts.job.fee;
        let job = &mut ctx.accounts.job;
        require!(job.client == client_key, ErrorCode::NotAuthorized);
        require!(
            job.status == 0 || job.status == 1,
            ErrorCode::InvalidJobStatus
        );
        job.status = 6;
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
}

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
    pub authority: Signer<'info>,
    #[account(mut, seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub treasury: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}
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
pub struct AcceptJob<'info> {
    pub freelancer: Signer<'info>,
    #[account(mut, seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    pub client: UncheckedAccount<'info>,
}
#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct SubmitWork<'info> {
    pub freelancer: Signer<'info>,
    #[account(mut, seeds = [b"job", client.key().as_ref(), &job_id.to_le_bytes()], bump = job.bump)]
    pub job: Account<'info, Job>,
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
