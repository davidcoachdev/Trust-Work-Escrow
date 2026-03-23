use anchor_lang::prelude::*;

pub mod errors;
pub mod state;

declare_id!("FgTaqMwswZFMje4wM8jmPN2iNZezgkhM3h1JGWoL2ZKM");

#[program]
pub mod trust_escrow_v2 {
    use super::*;
    use crate::errors::EscrowError;
    use crate::state::{Config, User};

    // ============ CONFIG INSTRUCTIONS ============

    #[derive(AnchorSerialize, AnchorDeserialize)]
    pub struct InitializeConfigParams {
        pub treasury_wallet: Pubkey,
        pub entry_fee_bps: u16,
        pub exit_fee_bps: u16,
        pub dispute_stake_bps: u16,
        pub max_job_duration_days: u32,
        pub auto_approve_days: u8,
    }

    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        params: InitializeConfigParams,
    ) -> Result<()> {
        require!(
            params.entry_fee_bps <= 1000,
            EscrowError::InvalidFeePercentage
        );
        require!(
            params.exit_fee_bps <= 1000,
            EscrowError::InvalidFeePercentage
        );
        require!(
            params.dispute_stake_bps <= 1000,
            EscrowError::InvalidFeePercentage
        );

        let config = &mut ctx.accounts.config;
        config.admin = ctx.accounts.admin.key();
        config.treasury_wallet = params.treasury_wallet;
        config.treasurer = params.treasury_wallet;
        config.entry_fee_bps = params.entry_fee_bps;
        config.exit_fee_bps = params.exit_fee_bps;
        config.dispute_stake_bps = params.dispute_stake_bps;
        config.max_job_duration_days = params.max_job_duration_days;
        config.auto_approve_days = params.auto_approve_days;
        config.paused = false;
        config.bump = ctx.bumps.config;

        msg!("Config initialized");
        Ok(())
    }

    #[derive(Accounts)]
    #[instruction(params: InitializeConfigParams)]
    pub struct InitializeConfig<'info> {
        #[account(init, payer = admin, space = 8 + Config::INIT_SPACE, seeds = [b"config"], bump)]
        pub config: Account<'info, Config>,
        #[account(mut)]
        pub admin: Signer<'info>,
        pub system_program: Program<'info, System>,
    }

    #[derive(AnchorSerialize, AnchorDeserialize)]
    pub struct UpdateConfigParams {
        pub treasury_wallet: Option<Pubkey>,
        pub entry_fee_bps: Option<u16>,
        pub exit_fee_bps: Option<u16>,
        pub dispute_stake_bps: Option<u16>,
    }

    pub fn update_config(ctx: Context<UpdateConfig>, params: UpdateConfigParams) -> Result<()> {
        require!(
            ctx.accounts.admin.key() == ctx.accounts.config.admin,
            EscrowError::UnauthorizedAdmin
        );
        require!(!ctx.accounts.config.paused, EscrowError::ProgramPaused);

        let config = &mut ctx.accounts.config;
        if let Some(tw) = params.treasury_wallet {
            config.treasury_wallet = tw;
        }
        if let Some(f) = params.entry_fee_bps {
            require!(f <= 1000, EscrowError::InvalidFeePercentage);
            config.entry_fee_bps = f;
        }
        if let Some(f) = params.exit_fee_bps {
            require!(f <= 1000, EscrowError::InvalidFeePercentage);
            config.exit_fee_bps = f;
        }
        if let Some(s) = params.dispute_stake_bps {
            require!(s <= 1000, EscrowError::InvalidFeePercentage);
            config.dispute_stake_bps = s;
        }

        msg!("Config updated");
        Ok(())
    }

    #[derive(Accounts)]
    #[instruction(params: UpdateConfigParams)]
    pub struct UpdateConfig<'info> {
        #[account(mut, seeds = [b"config"], bump = config.bump)]
        pub config: Account<'info, Config>,
        pub admin: Signer<'info>,
    }

    pub fn pause(ctx: Context<Pause>) -> Result<()> {
        require!(
            ctx.accounts.admin.key() == ctx.accounts.config.admin,
            EscrowError::UnauthorizedAdmin
        );
        ctx.accounts.config.paused = true;
        msg!("Program paused");
        Ok(())
    }

    #[derive(Accounts)]
    pub struct Pause<'info> {
        #[account(mut, seeds = [b"config"], bump = config.bump)]
        pub config: Account<'info, Config>,
        pub admin: Signer<'info>,
    }

    pub fn unpause(ctx: Context<Unpause>) -> Result<()> {
        require!(
            ctx.accounts.admin.key() == ctx.accounts.config.admin,
            EscrowError::UnauthorizedAdmin
        );
        ctx.accounts.config.paused = false;
        msg!("Program unpaused");
        Ok(())
    }

    #[derive(Accounts)]
    pub struct Unpause<'info> {
        #[account(mut, seeds = [b"config"], bump = config.bump)]
        pub config: Account<'info, Config>,
        pub admin: Signer<'info>,
    }

    // ============ USER INSTRUCTIONS ============

    #[derive(AnchorSerialize, AnchorDeserialize)]
    pub struct CreateUserParams {
        pub username: String,
        pub bio: String,
        pub skills: String,
    }

    pub fn create_user(ctx: Context<CreateUser>, params: CreateUserParams) -> Result<()> {
        require!(
            params.username.len() <= 32,
            EscrowError::InvalidFeePercentage
        );
        require!(params.bio.len() <= 256, EscrowError::InvalidFeePercentage);
        require!(
            params.skills.len() <= 128,
            EscrowError::InvalidFeePercentage
        );

        let user = &mut ctx.accounts.user;
        user.owner = ctx.accounts.owner.key();
        user.username = params.username;
        user.bio = params.bio;
        user.skills = params.skills;
        user.reputation = 50;
        user.jobs_completed = 0;
        user.disputes_won = 0;
        user.disputes_lost = 0;
        user.is_arbiter = false;
        user.wallet_count = 0;
        user.wallets = Vec::new();
        user.active_wallet_index = 0;
        user.bump = ctx.bumps.user;
        user.created_at = ctx.accounts.clock.unix_timestamp;
        user.updated_at = ctx.accounts.clock.unix_timestamp;

        msg!("User created");
        Ok(())
    }

    #[derive(Accounts)]
    #[instruction(params: CreateUserParams)]
    pub struct CreateUser<'info> {
        #[account(init, payer = owner, space = 8 + User::INIT_SPACE, seeds = [b"user", owner.key().as_ref()], bump)]
        pub user: Account<'info, User>,
        #[account(mut)]
        pub owner: Signer<'info>,
        pub system_program: Program<'info, System>,
        pub clock: Sysvar<'info, Clock>,
    }

    #[derive(AnchorSerialize, AnchorDeserialize)]
    pub struct UpdateUserParams {
        pub username: Option<String>,
        pub bio: Option<String>,
        pub skills: Option<String>,
    }

    pub fn update_user(ctx: Context<UpdateUser>, params: UpdateUserParams) -> Result<()> {
        let user = &mut ctx.accounts.user;
        if let Some(u) = params.username {
            require!(u.len() <= 32, EscrowError::InvalidFeePercentage);
            user.username = u;
        }
        if let Some(b) = params.bio {
            require!(b.len() <= 256, EscrowError::InvalidFeePercentage);
            user.bio = b;
        }
        if let Some(s) = params.skills {
            require!(s.len() <= 128, EscrowError::InvalidFeePercentage);
            user.skills = s;
        }
        user.updated_at = ctx.accounts.clock.unix_timestamp;
        msg!("User updated");
        Ok(())
    }

    #[derive(Accounts)]
    #[instruction(params: UpdateUserParams)]
    pub struct UpdateUser<'info> {
        #[account(mut, seeds = [b"user", owner.key().as_ref()], bump = user.bump)]
        pub user: Account<'info, User>,
        pub owner: Signer<'info>,
        pub clock: Sysvar<'info, Clock>,
    }

    // ============ WALLET INSTRUCTIONS ============

    const MAX_WALLETS: usize = 5;
    const WALLET_ENTRY_SIZE: usize = 34;

    pub fn add_wallet(ctx: Context<AddWallet>) -> Result<()> {
        let user = &mut ctx.accounts.user;
        require!(
            user.wallet_count < MAX_WALLETS as u8,
            EscrowError::MaxWalletsReached
        );

        let new_wallet = ctx.accounts.new_wallet.key();
        for i in 0..user.wallet_count as usize {
            let start = i * WALLET_ENTRY_SIZE;
            let wallet_bytes: [u8; 32] = user.wallets[start..start + 32].try_into().unwrap();
            let existing = Pubkey::new_from_array(wallet_bytes);
            require!(existing != new_wallet, EscrowError::WalletAlreadyAdded);
        }

        user.wallets.extend_from_slice(&new_wallet.to_bytes());
        user.wallets.push(0);
        user.wallets.push(0);
        user.wallet_count += 1;

        msg!("Wallet added");
        Ok(())
    }

    #[derive(Accounts)]
    pub struct AddWallet<'info> {
        #[account(mut, seeds = [b"user", owner.key().as_ref()], bump = user.bump)]
        pub user: Account<'info, User>,
        pub new_wallet: Signer<'info>,
        pub owner: Signer<'info>,
    }

    #[derive(AnchorSerialize, AnchorDeserialize)]
    pub struct SetActiveWalletParams {
        pub wallet_index: u8,
    }

    pub fn set_active_wallet(
        ctx: Context<SetActiveWallet>,
        params: SetActiveWalletParams,
    ) -> Result<()> {
        let user = &mut ctx.accounts.user;
        require!(
            params.wallet_index < user.wallet_count,
            EscrowError::InvalidWalletIndex
        );
        user.active_wallet_index = params.wallet_index;
        msg!("Active wallet set");
        Ok(())
    }

    #[derive(Accounts)]
    #[instruction(params: SetActiveWalletParams)]
    pub struct SetActiveWallet<'info> {
        #[account(mut, seeds = [b"user", owner.key().as_ref()], bump = user.bump)]
        pub user: Account<'info, User>,
        pub owner: Signer<'info>,
    }

    #[derive(AnchorSerialize, AnchorDeserialize)]
    pub struct RemoveWalletParams {
        pub wallet_index: u8,
    }

    pub fn remove_wallet(ctx: Context<RemoveWallet>, params: RemoveWalletParams) -> Result<()> {
        let user = &mut ctx.accounts.user;
        require!(
            params.wallet_index < user.wallet_count,
            EscrowError::InvalidWalletIndex
        );
        require!(
            params.wallet_index != 0,
            EscrowError::CannotRemovePrimaryWallet
        );

        let start_idx = params.wallet_index as usize * WALLET_ENTRY_SIZE;
        for _ in 0..WALLET_ENTRY_SIZE {
            user.wallets.remove(start_idx);
        }
        user.wallet_count -= 1;

        if user.active_wallet_index > params.wallet_index {
            user.active_wallet_index -= 1;
        } else if user.active_wallet_index >= user.wallet_count {
            user.active_wallet_index = 0;
        }

        msg!("Wallet removed");
        Ok(())
    }

    #[derive(Accounts)]
    #[instruction(params: RemoveWalletParams)]
    pub struct RemoveWallet<'info> {
        #[account(mut, seeds = [b"user", owner.key().as_ref()], bump = user.bump)]
        pub user: Account<'info, User>,
        pub owner: Signer<'info>,
    }
}
