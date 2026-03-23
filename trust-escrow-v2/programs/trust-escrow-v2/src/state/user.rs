use anchor_lang::prelude::*;

// Using fixed-size arrays to avoid InitSpace complexity
const MAX_USERNAME: usize = 32;
const MAX_BIO: usize = 256;
const MAX_SKILLS: usize = 128;
const MAX_WALLETS: usize = 5;
const WALLET_ENTRY_SIZE: usize = 34; // 32 pubkey + 1 bump + 1 is_primary

#[account]
#[derive(InitSpace)]
pub struct User {
    /// Owner of the user account (wallet)
    pub owner: Pubkey,
    /// Username (max 32 chars)
    #[max_len(MAX_USERNAME)]
    pub username: String,
    /// Bio description (max 256 chars)
    #[max_len(MAX_BIO)]
    pub bio: String,
    /// Skills as comma-separated string (max 128 chars)
    #[max_len(MAX_SKILLS)]
    pub skills: String,
    /// Reputation score (0-100)
    pub reputation: u8,
    /// Number of completed jobs
    pub jobs_completed: u32,
    /// Number of disputes won
    pub disputes_won: u32,
    /// Number of disputes lost
    pub disputes_lost: u32,
    /// Is the user an arbiter
    pub is_arbiter: bool,
    /// Number of wallets stored
    pub wallet_count: u8,
    /// Wallets data - fixed size array
    /// Format: [wallet1_pubkey(32) + bump(1) + is_primary(1), ...]
    #[max_len(MAX_WALLETS * WALLET_ENTRY_SIZE)]
    pub wallets: Vec<u8>,
    /// Active wallet index
    pub active_wallet_index: u8,
    /// Bump for PDA
    pub bump: u8,
    /// Created timestamp
    pub created_at: i64,
    /// Updated timestamp
    pub updated_at: i64,
}

impl Default for User {
    fn default() -> Self {
        Self {
            owner: Pubkey::default(),
            username: String::new(),
            bio: String::new(),
            skills: String::new(),
            reputation: 50,
            jobs_completed: 0,
            disputes_won: 0,
            disputes_lost: 0,
            is_arbiter: false,
            wallet_count: 0,
            wallets: Vec::new(),
            active_wallet_index: 0,
            bump: 0,
            created_at: 0,
            updated_at: 0,
        }
    }
}
