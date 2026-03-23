//! Trust Work Escrow TUI library
//!
//! This library provides the terminal user interface functionality
//! for interacting with the Trust Work Escrow v2 protocol.
//! 
//! ## Features
//! - Comprehensive event system with keyboard, blockchain, and UI events
//! - Async message channels for real-time blockchain updates
//! - Tick-based refresh system for periodic data updates
//! - Responsive navigation and state management
//! - Integration with trust-escrow-sdk for blockchain operations

pub mod app;
pub mod ui;

// Re-export commonly used types
pub use app::{
    App, AppEvent, EventHandler, BlockchainEvent, LifecycleEvent, 
    NavigationEvent, UIEvent, KeyInput, ViewTarget, TransactionStatus
};