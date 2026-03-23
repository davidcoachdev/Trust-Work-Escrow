//! Trust Work Escrow TUI library
//!
//! This library provides the terminal user interface functionality
//! for interacting with the Trust Work Escrow v2 protocol.

pub mod app;
pub mod ui;
pub mod events;

// Re-export commonly used types
pub use app::App;
pub use events::AppEvent;