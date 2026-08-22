//! TUI Application state and logic
//! 
//! Core application structure following Ratatui v0.30+ patterns
//! Integrates with trust-escrow-shared for configuration and client management
//! 
//! This module now provides both the legacy App struct (for compatibility with Task 3.1)
//! and access to the new comprehensive state management system (for Task 3.2+).

use anyhow::Result;
use crossterm::event::KeyCode;
use trust_escrow_shared::{EscrowClient, EscrowConfig};

// Import the comprehensive state management
use crate::app::{AppState, state::StatusType};

/// Main TUI application state (legacy compatibility for Task 3.1)
/// For new features in Task 3.2+, use the comprehensive AppState in app::state
pub struct App {
    /// Comprehensive application state (new in Task 3.2)
    state: AppState,
}

/// Different views/screens in the TUI (prepared for future phases)
#[derive(Debug, Clone, PartialEq)]
pub enum AppView {
    Welcome,    // Current phase - basic welcome screen
    Dashboard,  // Phase 3.2+
    Jobs,       // Phase 3.2+
    Profile,    // Phase 3.2+
    Settings,   // Phase 3.2+
}

impl App {
    /// Create new app instance with default configuration
    pub async fn new() -> Result<Self> {
        let config = EscrowConfig::load().unwrap_or_default();
        Self::with_config(config).await
    }
    
    /// Create new app instance with provided configuration
    pub async fn with_config(config: EscrowConfig) -> Result<Self> {
        // Create comprehensive state with the new system
        let state = AppState::new(config).await?;
        
        Ok(Self { state })
    }

    /// Handle keyboard input (legacy method for compatibility)
    pub async fn handle_input(&mut self, key: KeyCode) -> Result<()> {
        // Delegate to the new state management system
        self.state.handle_input(key).await
    }

    /// Update app state - called periodically in main loop
    pub async fn update(&mut self) -> Result<()> {
        // Delegate to the new state management system
        self.state.update().await
    }

    /// Refresh connection and update status
    pub async fn refresh_connection(&mut self) -> Result<()> {
        self.state.refresh_connection().await
    }
    
    /// Check connection status
    pub async fn check_connection(&mut self) -> Result<()> {
        self.state.check_connection().await
    }

    // Getters for UI display (maintain compatibility)
    
    /// Get current status message
    pub fn get_status(&self) -> &str {
        self.state.get_status()
    }
    
    /// Set status message
    pub fn set_status(&mut self, status: &str) {
        self.state.set_status(status, StatusType::Info);
    }
    
    /// Get application title
    pub fn get_title(&self) -> &str {
        self.state.get_title()
    }
    
    /// Get network name from configuration
    pub fn get_network_name(&self) -> &str {
        self.state.get_network_name()
    }
    
    /// Get RPC URL from configuration
    pub fn get_rpc_url(&self) -> &str {
        self.state.get_rpc_url()
    }
    
    /// Get current view
    pub fn get_current_view(&self) -> &AppView {
        &self.state.ui_state.current_view
    }

    /// Get wallet balance as string (async)
    pub async fn get_balance_string(&self) -> String {
        self.state.get_balance_string().await
    }

    /// Get wallet address as string
    pub fn get_wallet_address(&self) -> String {
        self.state.get_wallet_address()
    }
    
    /// Get configuration reference
    pub fn config(&self) -> &EscrowConfig {
        self.state.config()
    }
    
    /// Get client reference
    pub fn client(&self) -> &EscrowClient {
        self.state.client()
    }

    /// Get comprehensive state reference (new in Task 3.2)
    pub fn state(&self) -> &AppState {
        &self.state
    }

    /// Get mutable comprehensive state reference (new in Task 3.2)
    pub fn state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }
}