//! # Trust Escrow SDK
//!
//! A comprehensive Rust SDK for interacting with the Trust Work Escrow v2 smart contract on Solana.
//!
//! This SDK provides type-safe, high-level operations for escrow functionality including:
//! - User and team management
//! - Job lifecycle operations  
//! - Dispute handling and resolution
//! - Milestone-based payments
//! - Multi-wallet support
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use trust_escrow_sdk::{CofreClient, error::Result};
//! use solana_sdk::{signature::Keypair, commitment_config::CommitmentConfig};
//! use solana_client::rpc_client::RpcClient;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let rpc = Arc::new(RpcClient::new_with_commitment(
//!         "https://api.devnet.solana.com".to_string(),
//!         CommitmentConfig::confirmed()
//!     ));
//!     let payer = Arc::new(Keypair::new());
//!     
//!     let client = CofreClient::new(rpc, payer)?;
//!     
//!     // Create a user account
//!     let signature = client.create_user("alice", Some("Freelance developer")).await?;
//!     println!("User created: {}", signature);
//!     
//!     Ok(())
//! }
//! ```

use solana_sdk::pubkey::Pubkey;

pub mod client;
pub mod error;
pub mod pda;
pub mod types;
pub mod utils;

// Re-export key types for convenience
pub use client::CofreClient;
pub use error::{EscrowError, Result};
pub use pda::*;
pub use types::*;

/// The Trust Escrow v2 Program ID on Solana
/// This matches the deployed program ID
pub const PROGRAM_ID: Pubkey = solana_sdk::pubkey!("28QTH6qfG2iKDVXNY8nUTKDjx8yrBrQpnvXCPyJsrwuA");

/// Minimum job amount in lamports (0.0001 SOL)
pub const MIN_JOB_AMOUNT: u64 = 100_000;

/// Maximum number of wallets per user account
pub const MAX_WALLETS: usize = 5;

/// Maximum number of arbiters in arbiter pool
pub const MAX_ARBITERS: usize = 50;

/// Maximum number of milestones per job
pub const MAX_MILESTONES: usize = 20;

/// Maximum dispute evidence length
pub const MAX_DISPUTE_EVIDENCE: usize = 2048;

#[cfg(feature = "async")]
pub use tokio;

// Include generated Anchor client (this will be populated by build.rs in full implementation)
// For now, we define a placeholder module to satisfy the compiler
#[allow(dead_code)]
mod anchor_gen {
    // This module will contain generated Anchor client code
    // Generated content will be inserted here by build.rs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(MIN_JOB_AMOUNT, 100_000);
        assert_eq!(MAX_WALLETS, 5);
        assert_eq!(MAX_ARBITERS, 50);
        assert_eq!(MAX_MILESTONES, 20);
        assert_eq!(MAX_DISPUTE_EVIDENCE, 2048);
    }

    #[test]
    fn test_program_id() {
        // Ensure program ID is valid pubkey format
        assert_ne!(PROGRAM_ID, Pubkey::default());
    }
}
