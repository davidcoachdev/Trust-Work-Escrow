//! TUI Application state and logic

use anyhow::Result;
use crossterm::event::KeyCode;
use trust_escrow_shared::EscrowClient;

/// Main TUI application state
pub struct App {
    /// Escrow client for backend operations
    pub client: EscrowClient,
    
    /// Application title
    pub title: String,
    
    /// Current status message
    pub status: String,
    
    /// Whether the app should exit
    pub should_quit: bool,
    
    /// Current view/screen
    pub current_view: AppView,
}

/// Different views/screens in the TUI
#[derive(Debug, Clone, PartialEq)]
pub enum AppView {
    Dashboard,
    Jobs,
    Profile,
    Settings,
}

impl App {
    /// Create new app instance
    pub async fn new() -> Result<Self> {
        let client = EscrowClient::new()?;
        
        Ok(Self {
            client,
            title: "Trust Work Escrow v2".to_string(),
            status: "Ready".to_string(),
            should_quit: false,
            current_view: AppView::Dashboard,
        })
    }

    /// Handle keyboard input
    pub async fn handle_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('1') => self.current_view = AppView::Dashboard,
            KeyCode::Char('2') => self.current_view = AppView::Jobs,
            KeyCode::Char('3') => self.current_view = AppView::Profile,
            KeyCode::Char('4') => self.current_view = AppView::Settings,
            KeyCode::Char('r') => {
                self.status = "Refreshing...".to_string();
                // Refresh data
                self.refresh().await?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Update app state
    pub async fn update(&mut self) -> Result<()> {
        // Periodic updates can go here
        Ok(())
    }

    /// Refresh data from network
    async fn refresh(&mut self) -> Result<()> {
        // Check connection
        match self.client.check_connection().await {
            Ok(_) => {
                self.status = "Connected".to_string();
            }
            Err(e) => {
                self.status = format!("Connection error: {}", e);
            }
        }
        Ok(())
    }

    /// Get wallet balance as string
    pub async fn get_balance_string(&self) -> String {
        match self.client.get_wallet_balance().await {
            Ok(balance) => {
                let sol_balance = balance as f64 / 1_000_000_000.0;
                format!("{:.6} SOL", sol_balance)
            }
            Err(_) => "N/A".to_string(),
        }
    }

    /// Get wallet address as string
    pub fn get_wallet_address(&self) -> String {
        self.client
            .wallet_pubkey()
            .map(|pk| pk.to_string())
            .unwrap_or("No wallet".to_string())
    }
}