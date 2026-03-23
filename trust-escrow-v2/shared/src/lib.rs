//! Shared utilities for Trust Work Escrow CLI/TUI applications
//!
//! This crate provides common functionality used across the CLI and TUI
//! applications, including configuration management, error handling, and
//! SDK client wrapper utilities.

pub mod config;
pub mod error;
pub mod client;

// Re-export commonly used types
pub use config::{EscrowConfig, NetworkConfig};
pub use error::{AppError, AppResult};
pub use client::EscrowClient;