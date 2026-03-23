//! User account - Per-user profile with multi-wallet support

use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct User {
    pub wallet_principal: Pubkey,
    #[max_len(5)]
    pub wallets_asociadas: Vec<Pubkey>,
    pub active_wallet: Pubkey,
    #[max_len(32)]
    pub username: String,
    #[max_len(500)]
    pub bio: Option<String>,
    pub created_at: i64,
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
