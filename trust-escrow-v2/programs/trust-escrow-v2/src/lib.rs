//! Trust Work Escrow v2 - Smart Contract
//! 
//! Features:
//! - User accounts with multi-wallet support
//! - Job management with escrow
//! - Arbiter pool
//! - Multisig governance (2-of-3)

pub mod error;
pub mod state;
pub mod instructions;

use anchor_lang::prelude::*;

declare_id!("TRUST2XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");

pub use error::ErrorCode;
pub use state::*;

/// Program entrypoint - Anchor style
#[program]
pub mod trust_escrow_v2 {
    use super::*;

    // ============ CONFIG INSTRUCTIONS ============
    
    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        multisig_owners: Vec<Pubkey>,
        multisig_threshold: u8,
        treasury: Pubkey,
        fee_percent: u8,
    ) -> Result<()> {
        instructions::initialize_config::handler(
            ctx,
            multisig_owners,
            multisig_threshold,
            treasury,
            fee_percent,
        )
    }

    pub fn pause(ctx: Context<Pause>) -> Result<()> {
        instructions::pause::handler(ctx)
    }

    pub fn unpause(ctx: Context<Unpause>) -> Result<()> {
        instructions::unpause::handler(ctx)
    }

    pub fn withdraw_treasury(ctx: Context<WithdrawTreasury>, amount: u64) -> Result<()> {
        instructions::withdraw_treasury::handler(ctx, amount)
    }

    // ============ USER INSTRUCTIONS ============

    pub fn create_user(ctx: Context<CreateUser>, username: String) -> Result<()> {
        instructions::create_user::handler(ctx, username)
    }

    pub fn add_wallet(ctx: Context<AddWallet>, new_wallet: Pubkey) -> Result<()> {
        instructions::add_wallet::handler(ctx, new_wallet)
    }

    pub fn set_active_wallet(ctx: Context<SetActiveWallet>, wallet: Pubkey) -> Result<()> {
        instructions::set_active_wallet::handler(ctx, wallet)
    }

    pub fn update_user(ctx: Context<UpdateUser>, bio: Option<String>) -> Result<()> {
        instructions::update_user::handler(ctx, bio)
    }

    // ============ JOB INSTRUCTIONS ============

    pub fn create_job(
        ctx: Context<CreateJob>,
        job_id: u64,
        title: String,
        description: String,
        amount: u64,
        deadline: i64,
        arbiter: Option<Pubkey>,
    ) -> Result<()> {
        instructions::create_job::handler(ctx, job_id, title, description, amount, deadline, arbiter)
    }

    pub fn deposit_funds(ctx: Context<DepositFunds>, job_id: u64) -> Result<()> {
        instructions::deposit_funds::handler(ctx, job_id)
    }

    pub fn accept_job(ctx: Context<AcceptJob>, job_id: u64) -> Result<()> {
        instructions::accept_job::handler(ctx, job_id)
    }

    pub fn submit_work(ctx: Context<SubmitWork>, job_id: u64) -> Result<()> {
        instructions::submit_work::handler(ctx, job_id)
    }

    pub fn approve_work(ctx: Context<ApproveWork>, job_id: u64) -> Result<()> {
        instructions::approve_work::handler(ctx, job_id)
    }

    pub fn reject_work(ctx: Context<RejectWork>, job_id: u64, reason: String) -> Result<()> {
        instructions::reject_work::handler(ctx, job_id, reason)
    }

    pub fn cancel_job(ctx: Context<CancelJob>, job_id: u64) -> Result<()> {
        instructions::cancel_job::handler(ctx, job_id)
    }

    // ============ ARBITER INSTRUCTIONS ============

    pub fn register_arbiters(ctx: Context<RegisterArbiters>, arbiters: Vec<Pubkey>) -> Result<()> {
        instructions::register_arbiters::handler(ctx, arbiters)
    }

    pub fn raise_dispute(ctx: Context<RaiseDispute>, job_id: u64, reason: String) -> Result<()> {
        instructions::raise_dispute::handler(ctx, job_id, reason)
    }

    pub fn resolve_dispute(
        ctx: Context<ResolveDispute>,
        job_id: u64,
        freelancer_percent: u8,
    ) -> Result<()> {
        instructions::resolve_dispute::handler(ctx, job_id, freelancer_percent)
    }
}