//! State module - Account structs for Trust Work Escrow v2

pub mod config;
pub mod job;
pub mod team;
pub mod user;

pub use config::Config;
pub use job::{Application, ApplicationStatus, Job, JobStatus};
pub use team::{Member, MemberRole, Team};
pub use user::User;

// Constants
pub const MAX_WALLETS: usize = 5;
pub const MAX_TEAM_MEMBERS: usize = 20;
pub const MIN_JOB_AMOUNT: u64 = 100_000; // 0.0001 SOL
pub const MAX_TITLE_LENGTH: usize = 64;
pub const MAX_DESCRIPTION_LENGTH: usize = 1024;
pub const MAX_PROPOSAL_LENGTH: usize = 512;
pub const MAX_USERNAME_LENGTH: usize = 32;
pub const MAX_BIO_LENGTH: usize = 500;
pub const MAX_MULTISIG_OWNERS: usize = 5;
