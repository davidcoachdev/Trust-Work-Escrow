//! Backend v3 contracts that sit above `trust-escrow-sdk`.
//!
//! This crate intentionally contains deterministic, dependency-light domain
//! boundaries first. It does not talk to RPC, persist secrets, or treat a DB
//! projection as contractual authority.

pub mod application;
pub mod intents;
pub mod projection;
pub mod signer;

pub use intents::Finality;
