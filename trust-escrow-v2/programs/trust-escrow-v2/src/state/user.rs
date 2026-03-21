//! User account - Per-user profile with multi-wallet support

use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct User {
    /// Primary wallet (set during creation)
    pub wallet_principal: Pubkey,
    /// Associated wallets (max 10)
    #[max_len(MAX_WALLETS)]
    pub wallets_asociadas: Vec<Pubkey>,
    /// Currently active wallet for this session
    pub active_wallet: Pubkey,
    /// Username (max 32 chars)
    #[max_len(MAX_USERNAME_LENGTH)]
    pub username: String,
    /// Bio (optional, max 500 chars)
    #[max_len(MAX_BIO_LENGTH)]
    pub bio: Option<String>,
    /// Account creation timestamp
    pub created_at: i64,
    /// PDA bump
    pub bump: u8,
}

impl User {
    /// Check if a wallet is associated with this user
    pub fn is_wallet_associated(&self, wallet: &Pubkey) -> bool {
        if self.wallet_principal == *wallet {
            return true;
        }
        self.wallets_asociadas.iter().any(|w| w == wallet)
    }

    /// Check if wallet is the active one
    pub fn is_active_wallet(&self, wallet: &Pubkey) -> bool {
        self.active_wallet == *wallet
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum UserRole {
    Client,
    Freelancer,
    Arbiter,
}