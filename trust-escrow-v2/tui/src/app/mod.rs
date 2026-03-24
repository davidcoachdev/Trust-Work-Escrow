//! Application module for Trust Work Escrow TUI
//! 
//! This module provides comprehensive state management for the TUI application,
//! including user context, data tracking, network status management, and event handling.
//! 
//! Following Ratatui v0.30+ patterns with integration to trust-escrow-sdk.

pub mod state;
pub mod config;
pub mod events;

// Re-export key types for easier access
pub use state::{
    AppState, UserContext, DataState, NetworkState, UIState, AppView, StatusType,
    UserRole, UIFocus, AuthStatus, NotificationType, NotificationPriority, InputMode,
    MenuAction, MenuItem, CenterContent, Theme
};
pub use config::TuiConfig;
pub use events::{
    AppEvent, EventHandler, BlockchainEvent, LifecycleEvent, NavigationEvent, UIEvent,
    KeyInput, ViewTarget, TransactionStatus
};

// Maintain compatibility with existing app.rs structure
use anyhow::Result;
use crossterm::event::KeyCode;
use trust_escrow_shared::{EscrowClient, EscrowConfig};

/// Main TUI application - enhanced wrapper around AppState
pub struct App {
    /// Comprehensive application state
    state: AppState,
}

impl App {
    /// Create new app instance with default configuration
    pub async fn new() -> Result<Self> {
        let config = EscrowConfig::load().unwrap_or_default();
        Self::with_config(config).await
    }
    
    /// Create new app instance with provided configuration
    pub async fn with_config(config: EscrowConfig) -> Result<Self> {
        let state = AppState::new(config).await?;
        
        Ok(Self { state })
    }

    /// Handle keyboard input (legacy method for compatibility)
    pub async fn handle_input(&mut self, key: KeyCode) -> Result<()> {
        self.state.handle_input(key).await
    }

    /// Update app state - called periodically in main loop
    pub async fn update(&mut self) -> Result<()> {
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

    // Getters for UI display (delegate to state)
    
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

    /// Get state reference for advanced UI operations
    pub fn state(&self) -> &AppState {
        &self.state
    }

    /// Get mutable state reference for advanced UI operations
    pub fn state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }
}