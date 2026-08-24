//! # Trust Escrow SDK (v3)
//!
//! Rust SDK for the Trust Work Escrow v3 backend. Implements PDA derivation and
//! caching (T2) and a configurable client with typed account getters and errors
//! (T3). Instruction wrappers and event listeners land in later tasks (T4–T6+).
//!
//! The v3 program is immutable with id
//! `7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh` (see `trust-escrow-v3/Anchor.toml`).
//!
//! The heavy Solana/Anchor dependencies are optional behind the `solana` feature;
//! enable it (`--features solana`) to use PDA derivation, the client and types.

pub mod client;
pub mod cluster;
pub mod error;
pub mod events;
pub mod pda;
#[cfg(feature = "solana")]
pub mod relay;
pub mod types;
pub mod utils;

/// Program ID of the immutable `trust_escrow_v3` contract, as a string.
///
/// Stored as `&str` in the scaffold so the default build does not depend on
/// `solana-sdk`; later tasks parse it into a `Pubkey` behind the `solana` feature.
pub const PROGRAM_ID_STR: &str = "7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh";
