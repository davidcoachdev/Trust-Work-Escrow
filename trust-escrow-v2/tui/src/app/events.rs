//! Event System Architecture for Trust Work Escrow TUI
//!
//! Comprehensive event handling system supporting:
//! - Keyboard input events with navigation and actions
//! - Async blockchain operation updates via channels
//! - Tick-based periodic refresh system
//! - Resize events for responsive layouts
//! - Application lifecycle events with graceful shutdown

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// Comprehensive application event enum for all TUI interactions
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Keyboard input events
    Key(KeyInput),
    
    /// Mouse events (future expansion)
    Mouse {
        column: u16,
        row: u16,
        kind: MouseEventKind,
    },
    
    /// Terminal resize event
    Resize {
        width: u16,
        height: u16,
    },
    
    /// Periodic tick for refresh operations
    Tick,
    
    /// Fast tick for animations and real-time updates
    FastTick,
    
    /// Blockchain operation updates from async channels
    BlockchainUpdate(BlockchainEvent),
    
    /// Application lifecycle events
    Lifecycle(LifecycleEvent),
    
    /// Navigation events
    Navigation(NavigationEvent),
    
    /// User interface events
    UI(UIEvent),
}

/// Enhanced keyboard input with modifiers and key types
#[derive(Debug, Clone, PartialEq)]
pub struct KeyInput {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
    pub kind: KeyEventKind,
}

impl From<KeyEvent> for KeyInput {
    fn from(key_event: KeyEvent) -> Self {
        Self {
            code: key_event.code,
            modifiers: key_event.modifiers,
            kind: key_event.kind,
        }
    }
}

/// Mouse event types for future expansion
#[derive(Debug, Clone, PartialEq)]
pub enum MouseEventKind {
    Down,
    Up,
    Drag,
    Moved,
    ScrollDown,
    ScrollUp,
}

/// Blockchain operation updates received via async channels
#[derive(Debug, Clone)]
pub enum BlockchainEvent {
    /// Transaction status update
    TransactionUpdate {
        signature: String,
        status: TransactionStatus,
        confirmations: u32,
    },
    
    /// Legacy transaction update format
    TransactionUpdateLegacy {
        tx_id: String,
        status: TransactionStatus,
        message: String,
    },
    
    /// New job posting detected
    NewJob {
        job_id: u64,
        title: String,
        client: String,
    },
    
    /// Job application received
    JobApplication {
        job_id: u64,
        applicant: String,
    },
    
    /// Work submission notification
    WorkSubmitted {
        job_id: u64,
        submitter: String,
    },
    
    /// Dispute raised notification
    DisputeRaised {
        job_id: u64,
        disputer: String,
        reason: String,
    },
    
    /// Milestone status update
    MilestoneUpdate {
        milestone_id: u64,
        job_id: u64,
        status: String,
    },
    
    /// Balance change notification
    BalanceUpdate {
        wallet: String,
        new_balance: u64,
    },
    
    /// Network status change
    NetworkStatus {
        status: crate::app::state::ConnectionStatus,
        message: String,
    },
    
    /// Generic async error
    AsyncError {
        operation: String,
        error: String,
    },
    
    /// Data update notification (for async loading)
    DataUpdate {
        data_type: crate::app::state::DataType,
        loading_status: crate::app::state::LoadingStatus,
    },
    
    /// Background task status update
    TaskUpdate {
        task_name: String,
        status: crate::app::state::TaskStatus,
    },
}

/// Transaction status for blockchain updates
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
    Finalized,
}

/// Application lifecycle events
#[derive(Debug, Clone, PartialEq)]
pub enum LifecycleEvent {
    /// Request to quit application
    Quit,
    
    /// Force quit without confirmation
    ForceQuit,
    
    /// Application suspended/backgrounded
    Suspend,
    
    /// Application resumed/foregrounded
    Resume,
    
    /// Fatal error requiring shutdown
    FatalError(String),
    
    /// Graceful shutdown initiated
    Shutdown,
}

/// Navigation events for TUI views
#[derive(Debug, Clone, PartialEq)]
pub enum NavigationEvent {
    /// Navigate to specific view
    GoTo(ViewTarget),
    
    /// Navigate to specific view using the new pattern
    View(ViewTarget),
    
    /// Go back to previous view
    Back,
    
    /// Move focus to next element
    Next,
    
    /// Move focus to previous element
    Previous,
    
    /// Move up in lists/menus
    Up,
    
    /// Move down in lists/menus
    Down,
    
    /// Move left in horizontal navigation
    Left,
    
    /// Move right in horizontal navigation
    Right,
    
    /// Enter/select current item
    Select,
    
    /// Submit form or confirm action
    Submit,
    
    /// Execute command
    Command(String),
    
    /// Cancel current operation
    Cancel,
    
    /// Page up
    PageUp,
    
    /// Page down
    PageDown,
    
    /// Go to start/home
    Home,
    
    /// Go to end
    End,
}

/// Target views for navigation
#[derive(Debug, Clone, PartialEq)]
pub enum ViewTarget {
    Welcome,
    Dashboard,
    Jobs,
    JobDetail(u64),
    Profile,
    Teams,
    TeamDetail(u64),
    Settings,
    Help,
    Disputes,
    Milestones,
}

/// UI-specific events
#[derive(Debug, Clone, PartialEq)]
pub enum UIEvent {
    /// Refresh data
    Refresh,
    
    /// Toggle help overlay
    ToggleHelp,
    
    /// Show notification
    ShowNotification(String),
    
    /// Clear status messages
    ClearStatus,
    
    /// Filter/search action
    Search(String),
    
    /// Filter action with criteria
    Filter(String),
    
    /// Sort data
    Sort(SortCriteria),
    
    /// Toggle between views
    Toggle,
    
    /// Copy to clipboard
    Copy(String),
    
    /// Paste from clipboard
    Paste,
    
    /// Show context menu
    ContextMenu,
    
    // Navigation-specific events
    /// Focus next component
    FocusNext,
    
    /// Focus previous component
    FocusPrevious,
    
    /// Select next item in list
    SelectNext,
    
    /// Select previous item in list
    SelectPrevious,
    
    /// Select first item in list
    SelectFirst,
    
    /// Select last item in list
    SelectLast,
    
    /// Edit selected item
    Edit,
    
    /// Delete selected item
    Delete,
    
    /// Show form with given form type
    ShowForm(String),
    
    /// Confirm action with optional parameter
    Confirm(String),
    
    /// Custom UI event
    Custom(String),
}

/// Sort criteria for data display
#[derive(Debug, Clone, PartialEq)]
pub enum SortCriteria {
    Date,
    Amount,
    Status,
    Name,
    Priority,
}

/// Event handler for processing all application events
pub struct EventHandler {
    /// Receiver for blockchain events from async operations
    blockchain_rx: mpsc::UnboundedReceiver<BlockchainEvent>,
    
    /// Sender for blockchain events (for async operations to send updates)
    blockchain_tx: mpsc::UnboundedSender<BlockchainEvent>,
    
    /// Last tick time for timing control
    last_tick: Instant,
    
    /// Fast tick interval for animations
    fast_tick_rate: Duration,
    
    /// Normal tick interval for data refresh
    tick_rate: Duration,
    
    /// Input timeout for responsive polling
    input_timeout: Duration,
}

impl EventHandler {
    /// Create a new event handler with default timing settings
    pub fn new() -> Self {
        let (blockchain_tx, blockchain_rx) = mpsc::unbounded_channel();
        
        Self {
            blockchain_rx,
            blockchain_tx,
            last_tick: Instant::now(),
            fast_tick_rate: Duration::from_millis(50),   // 20 FPS for smooth animations
            tick_rate: Duration::from_millis(5000),      // 5 seconds for data refresh
            input_timeout: Duration::from_millis(100),   // Responsive input polling
        }
    }
    
    /// Create event handler with custom timing settings
    pub fn with_timing(
        fast_tick_ms: u64,
        tick_ms: u64,
        input_timeout_ms: u64,
    ) -> Self {
        let (blockchain_tx, blockchain_rx) = mpsc::unbounded_channel();
        
        Self {
            blockchain_rx,
            blockchain_tx,
            last_tick: Instant::now(),
            fast_tick_rate: Duration::from_millis(fast_tick_ms),
            tick_rate: Duration::from_millis(tick_ms),
            input_timeout: Duration::from_millis(input_timeout_ms),
        }
    }
    
    /// Get a sender handle for async blockchain operations
    pub fn blockchain_sender(&self) -> mpsc::UnboundedSender<BlockchainEvent> {
        self.blockchain_tx.clone()
    }
    
    /// Poll for next event with non-blocking behavior
    pub async fn next_event(&mut self) -> Result<AppEvent> {
        let now = Instant::now();
        
        // Check for blockchain updates first (highest priority)
        if let Ok(blockchain_event) = self.blockchain_rx.try_recv() {
            return Ok(AppEvent::BlockchainUpdate(blockchain_event));
        }
        
        // Check for terminal/keyboard events
        if event::poll(self.input_timeout)? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    return Ok(AppEvent::Key(KeyInput::from(key_event)));
                }
                Event::Key(_) => {
                    // Ignore key release events
                }
                Event::Mouse(mouse_event) => {
                    return Ok(AppEvent::Mouse {
                        column: mouse_event.column,
                        row: mouse_event.row,
                        kind: MouseEventKind::Down, // Simplified for now
                    });
                }
                Event::Resize(width, height) => {
                    return Ok(AppEvent::Resize { width, height });
                }
                Event::FocusGained | Event::FocusLost | Event::Paste(_) => {
                    // Handle other events as needed
                }
            }
        }
        
        // Generate tick events based on timing
        let elapsed = now.duration_since(self.last_tick);
        
        if elapsed >= self.tick_rate {
            self.last_tick = now;
            Ok(AppEvent::Tick)
        } else if elapsed >= self.fast_tick_rate {
            Ok(AppEvent::FastTick)
        } else {
            // No event ready, return a fast tick to keep the loop responsive
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok(AppEvent::FastTick)
        }
    }
    
    /// Send a blockchain event from async operations
    pub fn send_blockchain_event(&self, event: BlockchainEvent) -> Result<()> {
        self.blockchain_tx
            .send(event)
            .map_err(|e| anyhow::anyhow!("Failed to send blockchain event: {}", e))?;
        Ok(())
    }
    
    /// Process keyboard input into high-level events
    pub fn process_key_input(&self, input: &KeyInput) -> Option<AppEvent> {
        match (input.code, input.modifiers) {
            // Quit commands
            (KeyCode::Char('q'), KeyModifiers::NONE) => {
                Some(AppEvent::Lifecycle(LifecycleEvent::Quit))
            }
            (KeyCode::Esc, KeyModifiers::NONE) => {
                Some(AppEvent::Navigation(NavigationEvent::Cancel))
            }
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                Some(AppEvent::Lifecycle(LifecycleEvent::ForceQuit))
            }
            
            // Navigation
            (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::NONE) => {
                Some(AppEvent::Navigation(NavigationEvent::Up))
            }
            (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::NONE) => {
                Some(AppEvent::Navigation(NavigationEvent::Down))
            }
            (KeyCode::Left, _) | (KeyCode::Char('h'), KeyModifiers::NONE) => {
                Some(AppEvent::Navigation(NavigationEvent::Left))
            }
            (KeyCode::Right, _) | (KeyCode::Char('l'), KeyModifiers::NONE) => {
                Some(AppEvent::Navigation(NavigationEvent::Right))
            }
            (KeyCode::Enter, _) => {
                Some(AppEvent::Navigation(NavigationEvent::Select))
            }
            (KeyCode::Tab, KeyModifiers::NONE) => {
                Some(AppEvent::Navigation(NavigationEvent::Next))
            }
            (KeyCode::Tab, KeyModifiers::SHIFT) => {
                Some(AppEvent::Navigation(NavigationEvent::Previous))
            }
            (KeyCode::PageUp, _) => {
                Some(AppEvent::Navigation(NavigationEvent::PageUp))
            }
            (KeyCode::PageDown, _) => {
                Some(AppEvent::Navigation(NavigationEvent::PageDown))
            }
            (KeyCode::Home, _) => {
                Some(AppEvent::Navigation(NavigationEvent::Home))
            }
            (KeyCode::End, _) => {
                Some(AppEvent::Navigation(NavigationEvent::End))
            }
            
            // View navigation with number keys REMOVED - now used for role switching (1-5)
            
            // UI actions
            (KeyCode::Char('r'), KeyModifiers::NONE) => {
                Some(AppEvent::UI(UIEvent::Refresh))
            }
            (KeyCode::Char('?'), KeyModifiers::NONE) | (KeyCode::F(1), _) => {
                Some(AppEvent::UI(UIEvent::ToggleHelp))
            }
            (KeyCode::Char('/'), KeyModifiers::NONE) => {
                Some(AppEvent::UI(UIEvent::Search(String::new())))
            }
            (KeyCode::Char('x'), KeyModifiers::NONE) => {
                Some(AppEvent::UI(UIEvent::Copy(String::new())))
            }
            
            // Function keys for quick actions
            (KeyCode::F(2), _) => {
                Some(AppEvent::Navigation(NavigationEvent::GoTo(ViewTarget::Jobs)))
            }
            (KeyCode::F(3), _) => {
                Some(AppEvent::Navigation(NavigationEvent::GoTo(ViewTarget::Profile)))
            }
            (KeyCode::F(4), _) => {
                Some(AppEvent::Navigation(NavigationEvent::GoTo(ViewTarget::Teams)))
            }
            (KeyCode::F(5), _) => {
                Some(AppEvent::UI(UIEvent::Refresh))
            }
            
            _ => None,
        }
    }
    
    /// Check if the application should quit based on event
    pub fn should_quit(&self, event: &AppEvent) -> bool {
        matches!(
            event,
            AppEvent::Lifecycle(LifecycleEvent::Quit)
                | AppEvent::Lifecycle(LifecycleEvent::ForceQuit)
                | AppEvent::Lifecycle(LifecycleEvent::Shutdown)
                | AppEvent::Lifecycle(LifecycleEvent::FatalError(_))
        )
    }
    
    /// Extract error information if the event contains an error
    pub fn extract_error(&self, event: &AppEvent) -> Option<String> {
        match event {
            AppEvent::Lifecycle(LifecycleEvent::FatalError(msg)) => Some(msg.clone()),
            AppEvent::BlockchainUpdate(BlockchainEvent::AsyncError { operation, error }) => {
                Some(format!("Async error in {}: {}", operation, error))
            }
            _ => None,
        }
    }
    
    /// Create a blockchain transaction update event
    pub fn create_transaction_event(
        tx_id: String,
        status: TransactionStatus,
        message: String,
    ) -> AppEvent {
        AppEvent::BlockchainUpdate(BlockchainEvent::TransactionUpdate {
            signature: tx_id,
            status,
            confirmations: 1, // Default to 1 confirmation
        })
    }
    
    /// Create a network status update event
    pub fn create_network_status_event(
        connected: bool,
        rpc_url: String,
        block_height: Option<u64>,
    ) -> AppEvent {
        let status = if connected {
            crate::app::state::ConnectionStatus::Connected
        } else {
            crate::app::state::ConnectionStatus::Disconnected
        };
        
        AppEvent::BlockchainUpdate(BlockchainEvent::NetworkStatus {
            status,
            message: format!("RPC: {}, Height: {:?}", rpc_url, block_height),
        })
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl AppEvent {
    /// Check if this is a quit event
    pub fn is_quit(&self) -> bool {
        matches!(
            self,
            AppEvent::Lifecycle(LifecycleEvent::Quit)
                | AppEvent::Lifecycle(LifecycleEvent::ForceQuit)
                | AppEvent::Lifecycle(LifecycleEvent::Shutdown)
        )
    }
    
    /// Check if this is a navigation event
    pub fn is_navigation(&self) -> bool {
        matches!(self, AppEvent::Navigation(_))
    }
    
    /// Check if this is a blockchain event
    pub fn is_blockchain(&self) -> bool {
        matches!(self, AppEvent::BlockchainUpdate(_))
    }
    
    /// Check if this is a tick event
    pub fn is_tick(&self) -> bool {
        matches!(self, AppEvent::Tick | AppEvent::FastTick)
    }
    
    /// Check if this is a resize event
    pub fn is_resize(&self) -> bool {
        matches!(self, AppEvent::Resize { .. })
    }
    
    /// Get priority level for event processing (higher = more important)
    pub fn priority(&self) -> u8 {
        match self {
            AppEvent::Lifecycle(LifecycleEvent::FatalError(_)) => 255,
            AppEvent::Lifecycle(LifecycleEvent::ForceQuit) => 200,
            AppEvent::Lifecycle(LifecycleEvent::Quit) => 190,
            AppEvent::Key(_) => 180,
            AppEvent::BlockchainUpdate(BlockchainEvent::AsyncError { .. }) => 170,
            AppEvent::Resize { .. } => 160,
            AppEvent::BlockchainUpdate(_) => 150,
            AppEvent::Navigation(_) => 140,
            AppEvent::UI(_) => 130,
            AppEvent::Mouse { .. } => 120,
            AppEvent::Lifecycle(_) => 110,
            AppEvent::Tick => 50,
            AppEvent::FastTick => 30,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_priority() {
        let fatal_error = AppEvent::Lifecycle(LifecycleEvent::FatalError("test".to_string()));
        let tick = AppEvent::Tick;
        
        assert!(fatal_error.priority() > tick.priority());
    }
    
    #[test]
    fn test_key_input_processing() {
        let handler = EventHandler::new();
        
        let quit_key = KeyInput {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
        };
        
        let event = handler.process_key_input(&quit_key);
        assert!(matches!(
            event,
            Some(AppEvent::Lifecycle(LifecycleEvent::Quit))
        ));
    }
    
    #[test]
    fn test_blockchain_event_creation() {
        let event = EventHandler::create_transaction_event(
            "test_tx".to_string(),
            TransactionStatus::Confirmed,
            "Transaction confirmed".to_string(),
        );
        
        assert!(event.is_blockchain());
    }
}
