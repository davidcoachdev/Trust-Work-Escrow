//! State module - Account structs for Trust Work Escrow v2

pub mod config;
pub mod user;
pub mod job;
pub mod arbiter_pool;

pub use config::Config;
pub use user::User;
pub use job::{Job, JobStatus};
pub use arbiter_pool::ArbiterPool;

// Constants
pub const MAX_WALLETS: usize = 10;
pub const MAX_ARBITERS: usize = 50;
pub const MAX_MULTISIG_OWNERS: usize = 5;
pub const MAX_USERNAME_LENGTH: usize = 32;
pub const MAX_BIO_LENGTH: usize = 500;
pub const MAX_TITLE_LENGTH: usize = 100;
pub const MAX_DESCRIPTION_LENGTH: usize = 500;
pub const MAX_DISPUTE_REASON_LENGTH: usize = 200;
pub const MIN_JOB_AMOUNT: u64 = 100_000; // 0.0001 SOL