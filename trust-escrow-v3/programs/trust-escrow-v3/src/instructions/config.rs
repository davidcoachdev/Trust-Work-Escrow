#![allow(unused_imports)]
use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer, ID as SYSTEM_PROGRAM_ID};
use crate::errors::ErrorCode;
use crate::state::*;
use crate::{ARBITER_FEE_BPS_PER_PARTY, AUTHORITY_TIMELOCK, AUTO_APPROVAL_DELAY, BASIS_POINTS, DISPUTE_ACCEPT_GRACE, INITIAL_AUTHORITY, MAX_APPLICATIONS, MAX_ARBITERS, MAX_EVIDENCE_COUNT, MAX_MILESTONES, MAX_PAUSE_DURATION, MIN_JOB_AMOUNT};
use crate::{check_not_paused, cleanup_job_applications, close_evidence_account, compute_fee, compute_shortfall, transfer_job_lamports, validate_treasury_destination};

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
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump,
        constraint = config.authority == authority.key() @ ErrorCode::NotAuthorized
    )]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
pub struct Unpause<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump,
        constraint = config.authority == authority.key() @ ErrorCode::NotAuthorized
    )]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
pub struct UpdateTreasury<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump,
        constraint = config.authority == authority.key() @ ErrorCode::NotAuthorized
    )]
    pub config: Account<'info, Config>,
    /// CHECK: Validated in the instruction as a non-default System account distinct from arbitration_treasury.
    pub new_treasury: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct UpdateArbitrationTreasury<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump,
        constraint = config.authority == authority.key() @ ErrorCode::NotAuthorized
    )]
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
pub struct CreateArbiterPool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init, payer = authority, space = ArbiterPool::INIT_SPACE + 8, seeds = [b"arbiter_pool"], bump)]
    pub pool: Account<'info, ArbiterPool>,
    #[account(
        seeds = [b"config"],
        bump = config.bump,
        constraint = config.authority == authority.key() @ ErrorCode::NotAuthorized
    )]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddArbiter<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"arbiter_pool"],
        bump = pool.bump,
        constraint = pool.authority == authority.key() @ ErrorCode::NotAuthorized,
        constraint = config.authority == authority.key() @ ErrorCode::NotAuthorized
    )]
    pub pool: Account<'info, ArbiterPool>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
pub struct RemoveArbiter<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"arbiter_pool"],
        bump = pool.bump,
        constraint = pool.authority == authority.key() @ ErrorCode::NotAuthorized,
        constraint = config.authority == authority.key() @ ErrorCode::NotAuthorized
    )]
    pub pool: Account<'info, ArbiterPool>,
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
}

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
    config.pending_authority = None;
    config.pending_authority_at = 0;

    msg!("Config initialized by: {}", config.authority);
    Ok(())
}

pub fn pause(ctx: Context<Pause>) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.paused = true;
    msg!("Program paused");
    Ok(())
}

pub fn unpause(ctx: Context<Unpause>) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.paused = false;
    msg!("Program unpaused");
    Ok(())
}

pub fn update_treasury(ctx: Context<UpdateTreasury>, new_treasury: Pubkey) -> Result<()> {
    let config = &mut ctx.accounts.config;
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

pub fn create_arbiter_pool(ctx: Context<CreateArbiterPool>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    pool.authority = ctx.accounts.authority.key();
    pool.arbiters = Vec::new();
    pool.bump = ctx.bumps.pool;
    Ok(())
}

pub fn add_arbiter(ctx: Context<AddArbiter>, new_arbiter: Pubkey) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
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
    let idx = pool
        .arbiters
        .iter()
        .position(|&a| a == arbiter)
        .ok_or(ErrorCode::NotValidArbiter)?;
    pool.arbiters.remove(idx);
    Ok(())
}

#[derive(Accounts)]
pub struct ProposeAuthority<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump,
        constraint = config.authority == authority.key() @ ErrorCode::NotAuthorized
    )]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
pub struct UpdateAuthority<'info> {
    /// CHECK: pending authority must sign to accept ownership after timelock.
    pub pending_authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
pub struct CancelAuthorityProposal<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump,
        constraint = config.authority == authority.key() @ ErrorCode::NotAuthorized
    )]
    pub config: Account<'info, Config>,
}

/// Step 1 (propose): current authority proposes a new authority.
/// Timelock 2 days starts now. Compatible with Squads multisig: the
/// Config.authority should be a Squads vault PDA; the Squads proposal
/// executes this instruction, starting the timelock. The second step
/// (update_authority) can only succeed after AUTHORITY_TIMELOCK.
pub fn propose_authority(ctx: Context<ProposeAuthority>, new_authority: Pubkey) -> Result<()> {
    require!(new_authority != Pubkey::default(), ErrorCode::InvalidNewAuthority);
    require!(
        new_authority != ctx.accounts.config.authority,
        ErrorCode::InvalidNewAuthority
    );
    let clock = Clock::get()?;
    ctx.accounts.config.pending_authority = Some(new_authority);
    ctx.accounts.config.pending_authority_at = clock.unix_timestamp;
    msg!(
        "Authority rotation proposed: {} -> {} at {}",
        ctx.accounts.config.authority,
        new_authority,
        ctx.accounts.config.pending_authority_at
    );
    Ok(())
}

/// Step 2 (approve/execute): pending authority accepts after timelock.
/// Must be signed by the pending_authority itself (proves key control) and
/// after AUTHORITY_TIMELOCK (2 days). This is the second signature in the
/// 2-step flow; when combined with Squads multisig as current authority,
/// it yields multisig + timelock + new-key acceptance.
pub fn update_authority(ctx: Context<UpdateAuthority>) -> Result<()> {
    let pending = ctx
        .accounts
        .config
        .pending_authority
        .ok_or(ErrorCode::NoPendingAuthority)?;
    require!(
        ctx.accounts.pending_authority.key() == pending,
        ErrorCode::NotAuthorized
    );
    let clock = Clock::get()?;
    let elapsed = clock
        .unix_timestamp
        .checked_sub(ctx.accounts.config.pending_authority_at)
        .ok_or(ErrorCode::AuthorityTimelockNotExpired)?;
    require!(
        elapsed >= AUTHORITY_TIMELOCK,
        ErrorCode::AuthorityTimelockNotExpired
    );
    let old = ctx.accounts.config.authority;
    ctx.accounts.config.authority = pending;
    ctx.accounts.config.pending_authority = None;
    ctx.accounts.config.pending_authority_at = 0;
    msg!("Authority rotated: {} -> {}", old, pending);
    Ok(())
}

pub fn cancel_authority_proposal(ctx: Context<CancelAuthorityProposal>) -> Result<()> {
    require!(
        ctx.accounts.config.pending_authority.is_some(),
        ErrorCode::NoPendingAuthority
    );
    ctx.accounts.config.pending_authority = None;
    ctx.accounts.config.pending_authority_at = 0;
    msg!("Authority proposal cancelled by {}", ctx.accounts.authority.key());
    Ok(())
}

