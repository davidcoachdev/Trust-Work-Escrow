//! Comprehensive state management for Trust Work Escrow TUI
//! 
//! This module provides the core state management system that drives all TUI
//! rendering and interactions. It integrates with trust-escrow-sdk types and
//! trust-escrow-shared configuration for a complete user experience.

use anyhow::Result;
use chrono::{DateTime, Utc};
use crossterm::event::KeyCode;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use tokio::time;

use trust_escrow_shared::{EscrowClient, EscrowConfig};
use trust_escrow_sdk::types::{
    Job, JobStatus, User, Team, Milestone, MilestoneStatus, 
    Dispute, DisputeStatus, Config, ApplicationStatus
};

use super::config::TuiConfig;

/// Different views/screens in the TUI (comprehensive state version)
#[derive(Debug, Clone, PartialEq)]
pub enum AppView {
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

/// Main application state - the central hub for all TUI data and interactions
pub struct AppState {
    /// User context and authentication
    pub user_context: UserContext,
    
    /// Data state tracking (jobs, milestones, notifications)
    pub data_state: DataState,
    
    /// Network and connectivity status
    pub network_state: NetworkState,
    
    /// UI-specific state (navigation, selection, focus)
    pub ui_state: UIState,
    
    /// Performance and caching state
    pub performance_state: PerformanceState,
    
    /// Configuration
    config: TuiConfig,
    
    /// Escrow client for backend operations
    client: EscrowClient,
    
    /// Last update timestamp
    last_update: Instant,
}

/// User context with roles and permissions
#[derive(Debug, Clone)]
pub struct UserContext {
    /// Current user profile (None if not logged in)
    pub current_user: Option<User>,
    
    /// Active wallet pubkey
    pub active_wallet: Option<Pubkey>,
    
    /// User role in current context
    pub current_role: UserRole,
    
    /// User permissions based on role and context
    pub permissions: UserPermissions,
    
    /// Authentication status
    pub auth_status: AuthStatus,
    
    /// Wallet balance (cached)
    pub wallet_balance: Option<u64>,
    
    /// Balance last updated
    pub balance_updated_at: Option<Instant>,
    
    /// User teams
    pub teams: Vec<Team>,
}

/// User roles in the platform
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UserRole {
    Guest,          // Not authenticated
    Freelancer,     // Individual freelancer
    Client,         // Job poster
    TeamMember,     // Part of a team
    TeamOwner,      // Owns a team
    Arbiter,        // Dispute resolver
}

/// User permissions for different actions
#[derive(Debug, Clone, Default)]
pub struct UserPermissions {
    pub can_post_jobs: bool,
    pub can_apply_to_jobs: bool,
    pub can_create_teams: bool,
    pub can_submit_work: bool,
    pub can_approve_work: bool,
    pub can_raise_disputes: bool,
    pub can_resolve_disputes: bool,
    pub can_manage_profile: bool,
}

/// Authentication status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AuthStatus {
    NotAuthenticated,
    Authenticating,
    Authenticated,
    AuthError,
}

/// Data state tracking all business objects
#[derive(Debug, Clone)]
pub struct DataState {
    /// Jobs (by pubkey)
    pub jobs: HashMap<Pubkey, Job>,
    
    /// Jobs the user has applied to
    pub user_applications: Vec<JobApplication>,
    
    /// Milestones (by pubkey)
    pub milestones: HashMap<Pubkey, Milestone>,
    
    /// Active disputes
    pub disputes: HashMap<Pubkey, Dispute>,
    
    /// Notifications queue
    pub notifications: VecDeque<Notification>,
    
    /// Cached user profiles
    pub users: HashMap<Pubkey, User>,
    
    /// User's teams
    pub teams: HashMap<Pubkey, Team>,
    
    /// Platform configuration
    pub platform_config: Option<Config>,
    
    /// Data loading states
    pub loading_states: LoadingStates,
    
    /// Last data refresh
    pub last_refresh: HashMap<DataType, Instant>,
    
    /// Data staleness tracking
    pub stale_data: std::collections::HashSet<DataType>,
}

/// Job application tracking
#[derive(Debug, Clone)]
pub struct JobApplication {
    pub job: Pubkey,
    pub applicant: Pubkey,
    pub status: ApplicationStatus,
    pub applied_at: DateTime<Utc>,
    pub proposal: Option<String>,
}

/// Notification system
#[derive(Debug, Clone)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub priority: NotificationPriority,
    pub created_at: DateTime<Utc>,
    pub read: bool,
    pub related_job: Option<Pubkey>,
    pub related_milestone: Option<Pubkey>,
}

/// Types of notifications
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NotificationType {
    JobUpdate,
    MilestoneUpdate,
    DisputeUpdate,
    PaymentReceived,
    ApplicationUpdate,
    SystemAlert,
    NetworkStatus,
}

/// Notification priority levels
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum NotificationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Data types for tracking loading and refresh states
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum DataType {
    Jobs,
    UserJobs,
    Milestones,
    Disputes,
    Teams,
    UserProfile,
    WalletBalance,
    PlatformConfig,
    Notifications,
}

/// Loading states for different data types
#[derive(Debug, Clone, Default)]
pub struct LoadingStates {
    pub jobs: LoadingStatus,
    pub user_jobs: LoadingStatus,
    pub milestones: LoadingStatus,
    pub disputes: LoadingStatus,
    pub teams: LoadingStatus,
    pub user_profile: LoadingStatus,
    pub wallet_balance: LoadingStatus,
    pub notifications: LoadingStatus,
}

/// Loading status for individual data types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoadingStatus {
    Idle,
    Loading,
    Success,
    Error,
}

impl Default for LoadingStatus {
    fn default() -> Self {
        LoadingStatus::Idle
    }
}

/// Network and connectivity state
#[derive(Debug, Clone)]
pub struct NetworkState {
    /// Connection status to Solana RPC
    pub rpc_status: ConnectionStatus,
    
    /// Last successful RPC call
    pub last_rpc_success: Option<Instant>,
    
    /// Connection error details
    pub connection_error: Option<String>,
    
    /// Network health metrics
    pub health: NetworkHealth,
    
    /// RPC endpoint info
    pub rpc_endpoint: String,
    
    /// Network cluster (devnet, mainnet, etc.)
    pub cluster: String,
    
    /// Connection retry count
    pub retry_count: u32,
    
    /// Connection attempt in progress
    pub connecting: bool,
}

/// Connection status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error,
    Degraded,   // Partially functional
}

/// Network health metrics
#[derive(Debug, Clone, Default)]
pub struct NetworkHealth {
    /// Average response time (ms)
    pub avg_response_time: Option<u64>,
    
    /// Success rate (0.0 - 1.0)
    pub success_rate: f64,
    
    /// Recent response times
    pub recent_response_times: VecDeque<u64>,
    
    /// Error count in last window
    pub recent_errors: u32,
    
    /// Health score (0-100)
    pub health_score: u8,
}

/// UI-specific state for navigation and interaction
#[derive(Debug, Clone)]
pub struct UIState {
    /// Current view/screen
    pub current_view: AppView,
    
    /// Previous view (for navigation)
    pub previous_view: Option<AppView>,
    
    /// Selected items in lists
    pub selections: HashMap<String, usize>,
    
    /// Current focus element
    pub focus: UIFocus,
    
    /// Navigation history
    pub navigation_history: VecDeque<AppView>,
    
    /// Modal dialogs state
    pub modal_state: Option<ModalState>,
    
    /// Status message
    pub status_message: String,
    
    /// Status message type
    pub status_type: StatusType,
    
    /// Status message timestamp
    pub status_updated_at: Instant,
    
    /// Application title
    pub title: String,
    
    /// Scrolling state for different views
    pub scroll_states: HashMap<String, ScrollState>,
    
    /// Input mode (normal, insert, etc.)
    pub input_mode: InputMode,
    
    /// Current input buffer
    pub input_buffer: String,
}

/// UI focus tracking
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UIFocus {
    MainContent,
    JobList,
    MilestoneList,
    NotificationPanel,
    InputField,
    Modal,
    Menu,
}

/// Modal dialog state
#[derive(Debug, Clone)]
pub struct ModalState {
    pub modal_type: ModalType,
    pub title: String,
    pub content: String,
    pub buttons: Vec<ModalButton>,
    pub selected_button: usize,
}

/// Types of modal dialogs
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModalType {
    Confirmation,
    Error,
    Info,
    Input,
    JobDetails,
    MilestoneDetails,
}

/// Modal button configuration
#[derive(Debug, Clone)]
pub struct ModalButton {
    pub label: String,
    pub action: ModalAction,
    pub style: ModalButtonStyle,
}

/// Modal button actions
#[derive(Debug, Clone, PartialEq)]
pub enum ModalAction {
    Close,
    Confirm,
    Cancel,
    Navigate(AppView),
}

/// Modal button styles
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModalButtonStyle {
    Default,
    Primary,
    Danger,
    Success,
}

/// Status message types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StatusType {
    Info,
    Success,
    Warning,
    Error,
}

/// Scrolling state for lists and content
#[derive(Debug, Clone, Default)]
pub struct ScrollState {
    pub offset: usize,
    pub selected: usize,
    pub total_items: usize,
}

/// Input modes for the TUI
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,     // Navigation mode
    Insert,     // Text input mode
    Command,    // Command mode
}

/// Performance and caching state
#[derive(Debug, Clone, Default)]
pub struct PerformanceState {
    /// Cache hit/miss statistics
    pub cache_stats: CacheStats,
    
    /// Operation timing metrics
    pub operation_timings: HashMap<String, Duration>,
    
    /// Memory usage tracking
    pub memory_usage: MemoryUsage,
    
    /// Background task status
    pub background_tasks: HashMap<String, TaskStatus>,
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub total_entries: usize,
}

/// Memory usage tracking
#[derive(Debug, Clone, Default)]
pub struct MemoryUsage {
    pub jobs_count: usize,
    pub milestones_count: usize,
    pub notifications_count: usize,
    pub cached_users_count: usize,
}

/// Background task status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskStatus {
    Idle,
    Running,
    Completed,
    Failed,
}

// Implementation begins here
impl AppState {
    /// Create new application state
    pub async fn new(escrow_config: EscrowConfig) -> Result<Self> {
        let config = TuiConfig {
            escrow: escrow_config.clone(),
            ..TuiConfig::default()
        };
        
        let client = EscrowClient::from_config(escrow_config)?;
        
        Ok(Self {
            user_context: UserContext::new(),
            data_state: DataState::new(),
            network_state: NetworkState::new(config.escrow.network.cluster.clone(), 
                                           config.escrow.network.rpc_url.clone()),
            ui_state: UIState::new(),
            performance_state: PerformanceState::default(),
            config,
            client,
            last_update: Instant::now(),
        })
    }

    /// Handle keyboard input
    pub async fn handle_input(&mut self, key: KeyCode) -> Result<()> {
        match self.ui_state.input_mode {
            InputMode::Normal => self.handle_normal_input(key).await,
            InputMode::Insert => self.handle_insert_input(key).await,
            InputMode::Command => self.handle_command_input(key).await,
        }
    }

    /// Handle input in normal navigation mode
    async fn handle_normal_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                // Handled in main loop
            }
            KeyCode::Char('r') => {
                self.refresh_connection().await?;
            }
            KeyCode::Char('c') => {
                self.check_connection().await?;
            }
            KeyCode::Char('h') => {
                self.show_help();
            }
            KeyCode::Char('d') => {
                self.navigate_to(AppView::Dashboard);
            }
            KeyCode::Char('j') => {
                self.navigate_to(AppView::Jobs);
            }
            KeyCode::Char('p') => {
                self.navigate_to(AppView::Profile);
            }
            KeyCode::Char('s') => {
                self.navigate_to(AppView::Settings);
            }
            KeyCode::Up => {
                self.navigate_up();
            }
            KeyCode::Down => {
                self.navigate_down();
            }
            KeyCode::Enter => {
                self.select_current().await?;
            }
            KeyCode::Backspace => {
                self.navigate_back();
            }
            _ => {
                self.set_status(&format!("❓ Unknown key. Press 'h' for help"), StatusType::Info);
            }
        }
        Ok(())
    }

    /// Handle input in text insertion mode
    async fn handle_insert_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Esc => {
                self.ui_state.input_mode = InputMode::Normal;
                self.ui_state.input_buffer.clear();
            }
            KeyCode::Enter => {
                self.process_input().await?;
                self.ui_state.input_mode = InputMode::Normal;
            }
            KeyCode::Backspace => {
                self.ui_state.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.ui_state.input_buffer.push(c);
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle input in command mode
    async fn handle_command_input(&mut self, key: KeyCode) -> Result<()> {
        // Command mode implementation for future phases
        match key {
            KeyCode::Esc => {
                self.ui_state.input_mode = InputMode::Normal;
                self.ui_state.input_buffer.clear();
            }
            _ => {}
        }
        Ok(())
    }

    /// Update application state
    pub async fn update(&mut self) -> Result<()> {
        let now = Instant::now();
        
        // Update network health
        self.update_network_health();
        
        // Update data staleness
        self.update_data_staleness();
        
        // Perform background refresh if enabled
        if self.should_auto_refresh() {
            self.background_refresh().await?;
        }
        
        // Update performance metrics
        self.update_performance_metrics();
        
        // Clean up old notifications
        self.cleanup_notifications();
        
        self.last_update = now;
        Ok(())
    }

    /// Check if auto refresh should occur
    fn should_auto_refresh(&self) -> bool {
        if !self.config.auto_refresh_enabled() {
            return false;
        }
        
        let auto_refresh_duration = self.config.auto_refresh_duration();
        self.last_update.elapsed() >= auto_refresh_duration
    }

    /// Perform background data refresh
    async fn background_refresh(&mut self) -> Result<()> {
        if self.network_state.rpc_status != ConnectionStatus::Connected {
            return Ok(());
        }

        // Refresh critical data
        self.refresh_user_context().await?;
        self.refresh_jobs_data().await?;
        self.refresh_notifications().await?;
        
        Ok(())
    }

    /// Refresh user context and authentication
    async fn refresh_user_context(&mut self) -> Result<()> {
        self.data_state.loading_states.user_profile = LoadingStatus::Loading;
        
        // Check wallet balance
        match self.client.get_wallet_balance().await {
            Ok(balance) => {
                self.user_context.wallet_balance = Some(balance);
                self.user_context.balance_updated_at = Some(Instant::now());
                self.data_state.loading_states.wallet_balance = LoadingStatus::Success;
            }
            Err(e) => {
                self.data_state.loading_states.wallet_balance = LoadingStatus::Error;
                self.add_notification(
                    "Wallet Error",
                    &format!("Failed to fetch wallet balance: {}", e),
                    NotificationType::NetworkStatus,
                    NotificationPriority::Medium,
                );
            }
        }
        
        self.data_state.loading_states.user_profile = LoadingStatus::Success;
        Ok(())
    }

    /// Refresh jobs data
    async fn refresh_jobs_data(&mut self) -> Result<()> {
        self.data_state.loading_states.jobs = LoadingStatus::Loading;
        
        // In a full implementation, this would fetch jobs from the blockchain
        // For now, we simulate the loading state
        
        self.data_state.loading_states.jobs = LoadingStatus::Success;
        self.data_state.last_refresh.insert(DataType::Jobs, Instant::now());
        Ok(())
    }

    /// Refresh notifications
    async fn refresh_notifications(&mut self) -> Result<()> {
        self.data_state.loading_states.notifications = LoadingStatus::Loading;
        
        // In a full implementation, this would check for new notifications
        // For now, we just update the timestamp
        
        self.data_state.loading_states.notifications = LoadingStatus::Success;
        self.data_state.last_refresh.insert(DataType::Notifications, Instant::now());
        Ok(())
    }

    /// Refresh connection and update status
    pub async fn refresh_connection(&mut self) -> Result<()> {
        self.set_status("🔄 Refreshing...", StatusType::Info);
        self.network_state.connecting = true;
        
        match self.client.check_connection().await {
            Ok(_) => {
                self.network_state.rpc_status = ConnectionStatus::Connected;
                self.network_state.last_rpc_success = Some(Instant::now());
                self.network_state.connection_error = None;
                self.network_state.retry_count = 0;
                self.set_status("✅ Connection refreshed successfully", StatusType::Success);
            }
            Err(e) => {
                self.network_state.rpc_status = ConnectionStatus::Error;
                self.network_state.connection_error = Some(e.to_string());
                self.network_state.retry_count += 1;
                self.set_status(&format!("❌ Connection error: {}", e), StatusType::Error);
            }
        }
        
        self.network_state.connecting = false;
        Ok(())
    }
    
    /// Check connection status
    pub async fn check_connection(&mut self) -> Result<()> {
        self.set_status("🔍 Checking connection...", StatusType::Info);
        
        match self.client.check_connection().await {
            Ok(_) => {
                self.network_state.rpc_status = ConnectionStatus::Connected;
                self.network_state.last_rpc_success = Some(Instant::now());
                self.set_status("✅ Connected to Solana network", StatusType::Success);
            }
            Err(e) => {
                self.network_state.rpc_status = ConnectionStatus::Error;
                self.network_state.connection_error = Some(e.to_string());
                self.set_status(&format!("❌ Connection failed: {}", e), StatusType::Error);
            }
        }
        Ok(())
    }

    /// Navigate to a different view
    pub fn navigate_to(&mut self, view: AppView) {
        if self.ui_state.current_view != view {
            self.ui_state.previous_view = Some(self.ui_state.current_view.clone());
            self.ui_state.navigation_history.push_back(self.ui_state.current_view.clone());
            self.ui_state.current_view = view;
            
            // Limit navigation history
            while self.ui_state.navigation_history.len() > 10 {
                self.ui_state.navigation_history.pop_front();
            }
        }
    }

    /// Navigate back to previous view
    pub fn navigate_back(&mut self) {
        if let Some(previous) = self.ui_state.previous_view.take() {
            self.ui_state.current_view = previous;
        } else if let Some(previous) = self.ui_state.navigation_history.pop_back() {
            self.ui_state.current_view = previous;
        }
    }

    /// Navigate up in current list
    pub fn navigate_up(&mut self) {
        let view_key = format!("{:?}", self.ui_state.current_view);
        let selection = self.ui_state.selections.entry(view_key).or_insert(0);
        if *selection > 0 {
            *selection -= 1;
        }
    }

    /// Navigate down in current list
    pub fn navigate_down(&mut self) {
        let view_key = format!("{:?}", self.ui_state.current_view);
        let selection = self.ui_state.selections.entry(view_key).or_insert(0);
        
        // In a full implementation, we'd check the actual list length
        // For now, we'll set a reasonable limit
        let max_items = match self.ui_state.current_view {
            AppView::Jobs => self.data_state.jobs.len(),
            _ => 10, // Default for other views
        };
        
        if *selection < max_items.saturating_sub(1) {
            *selection += 1;
        }
    }

    /// Select current item
    async fn select_current(&mut self) -> Result<()> {
        match self.ui_state.current_view {
            AppView::Jobs => {
                self.select_job().await?;
            }
            AppView::Dashboard => {
                // Handle dashboard selection
            }
            _ => {}
        }
        Ok(())
    }

    /// Select a job from the jobs list
    async fn select_job(&mut self) -> Result<()> {
        let view_key = format!("{:?}", self.ui_state.current_view);
        if let Some(selection) = self.ui_state.selections.get(&view_key) {
            // In a full implementation, we'd show job details
            self.set_status(&format!("Selected job #{}", selection), StatusType::Info);
        }
        Ok(())
    }

    /// Process input from insert mode
    async fn process_input(&mut self) -> Result<()> {
        let input = self.ui_state.input_buffer.trim();
        if !input.is_empty() {
            self.set_status(&format!("Processed input: {}", input), StatusType::Success);
        }
        self.ui_state.input_buffer.clear();
        Ok(())
    }

    /// Show help information
    pub fn show_help(&mut self) {
        self.set_status("📚 Help: h=help, d=dashboard, j=jobs, p=profile, s=settings, q=quit", StatusType::Info);
    }

    /// Add a notification
    pub fn add_notification(&mut self, title: &str, message: &str, 
                           notification_type: NotificationType, priority: NotificationPriority) {
        let notification = Notification {
            id: format!("{}-{}", chrono::Utc::now().timestamp_millis(), title),
            title: title.to_string(),
            message: message.to_string(),
            notification_type,
            priority,
            created_at: chrono::Utc::now(),
            read: false,
            related_job: None,
            related_milestone: None,
        };

        self.data_state.notifications.push_front(notification);
        
        // Limit notification count
        while self.data_state.notifications.len() > self.config.performance.max_notifications {
            self.data_state.notifications.pop_back();
        }
    }

    /// Update network health metrics
    fn update_network_health(&mut self) {
        // Update health score based on connection status and recent performance
        self.network_state.health.health_score = match self.network_state.rpc_status {
            ConnectionStatus::Connected => {
                let base_score = 100;
                let error_penalty = self.network_state.health.recent_errors * 5;
                (base_score - error_penalty).max(0) as u8
            }
            ConnectionStatus::Degraded => 50,
            ConnectionStatus::Connecting => 25,
            ConnectionStatus::Error => 0,
            ConnectionStatus::Disconnected => 0,
        };
        
        // Update success rate
        if self.network_state.health.recent_response_times.len() > 0 {
            let total_requests = self.network_state.health.recent_response_times.len() + 
                               self.network_state.health.recent_errors as usize;
            let successful_requests = self.network_state.health.recent_response_times.len();
            self.network_state.health.success_rate = 
                successful_requests as f64 / total_requests as f64;
        }
    }

    /// Update data staleness tracking
    fn update_data_staleness(&mut self) {
        let stale_threshold = Duration::from_secs(300); // 5 minutes
        let now = Instant::now();
        
        for (data_type, last_refresh) in &self.data_state.last_refresh {
            if now.duration_since(*last_refresh) > stale_threshold {
                self.data_state.stale_data.insert(*data_type);
            } else {
                self.data_state.stale_data.remove(data_type);
            }
        }
    }

    /// Update performance metrics
    fn update_performance_metrics(&mut self) {
        let stats = &mut self.performance_state.cache_stats;
        if stats.hits + stats.misses > 0 {
            // Cache hit rate calculation is done in accessor methods
        }
        
        // Update memory usage
        let memory = &mut self.performance_state.memory_usage;
        memory.jobs_count = self.data_state.jobs.len();
        memory.milestones_count = self.data_state.milestones.len();
        memory.notifications_count = self.data_state.notifications.len();
        memory.cached_users_count = self.data_state.users.len();
    }

    /// Clean up old notifications
    fn cleanup_notifications(&mut self) {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(7);
        self.data_state.notifications.retain(|n| n.created_at > cutoff);
    }

    // Getters for UI access
    
    /// Get current status message
    pub fn get_status(&self) -> &str {
        &self.ui_state.status_message
    }
    
    /// Set status message
    pub fn set_status(&mut self, message: &str, status_type: StatusType) {
        self.ui_state.status_message = message.to_string();
        self.ui_state.status_type = status_type;
        self.ui_state.status_updated_at = Instant::now();
    }
    
    /// Get application title
    pub fn get_title(&self) -> &str {
        &self.ui_state.title
    }
    
    /// Get network name from configuration
    pub fn get_network_name(&self) -> &str {
        &self.network_state.cluster
    }
    
    /// Get RPC URL from configuration
    pub fn get_rpc_url(&self) -> &str {
        &self.network_state.rpc_endpoint
    }

    /// Get wallet balance as string (async)
    pub async fn get_balance_string(&self) -> String {
        match self.user_context.wallet_balance {
            Some(balance) => {
                let sol_balance = balance as f64 / 1_000_000_000.0;
                format!("{:.6} SOL", sol_balance)
            }
            None => "Loading...".to_string(),
        }
    }

    /// Get wallet address as string
    pub fn get_wallet_address(&self) -> String {
        self.client
            .wallet_pubkey()
            .map(|pk| pk.to_string())
            .unwrap_or("No wallet".to_string())
    }
    
    /// Get configuration reference
    pub fn config(&self) -> &EscrowConfig {
        &self.config.escrow
    }
    
    /// Get client reference
    pub fn client(&self) -> &EscrowClient {
        &self.client
    }

    /// Get network connection status
    pub fn get_connection_status(&self) -> ConnectionStatus {
        self.network_state.rpc_status
    }

    /// Get unread notification count
    pub fn get_unread_notifications(&self) -> usize {
        self.data_state.notifications.iter()
            .filter(|n| !n.read)
            .count()
    }

    /// Get user role
    pub fn get_user_role(&self) -> UserRole {
        self.user_context.current_role
    }

    /// Get loading status for data type
    pub fn get_loading_status(&self, data_type: DataType) -> LoadingStatus {
        match data_type {
            DataType::Jobs => self.data_state.loading_states.jobs,
            DataType::UserJobs => self.data_state.loading_states.user_jobs,
            DataType::Milestones => self.data_state.loading_states.milestones,
            DataType::Disputes => self.data_state.loading_states.disputes,
            DataType::Teams => self.data_state.loading_states.teams,
            DataType::UserProfile => self.data_state.loading_states.user_profile,
            DataType::WalletBalance => self.data_state.loading_states.wallet_balance,
            DataType::Notifications => self.data_state.loading_states.notifications,
            DataType::PlatformConfig => LoadingStatus::Success, // Platform config is static
        }
    }

    /// Check if data is stale
    pub fn is_data_stale(&self, data_type: DataType) -> bool {
        self.data_state.stale_data.contains(&data_type)
    }
}

// Helper implementations for state components
impl UserContext {
    fn new() -> Self {
        Self {
            current_user: None,
            active_wallet: None,
            current_role: UserRole::Guest,
            permissions: UserPermissions::default(),
            auth_status: AuthStatus::NotAuthenticated,
            wallet_balance: None,
            balance_updated_at: None,
            teams: Vec::new(),
        }
    }
}

impl DataState {
    fn new() -> Self {
        Self {
            jobs: HashMap::new(),
            user_applications: Vec::new(),
            milestones: HashMap::new(),
            disputes: HashMap::new(),
            notifications: VecDeque::new(),
            users: HashMap::new(),
            teams: HashMap::new(),
            platform_config: None,
            loading_states: LoadingStates::default(),
            last_refresh: HashMap::new(),
            stale_data: std::collections::HashSet::new(),
        }
    }
}

impl NetworkState {
    fn new(cluster: String, rpc_endpoint: String) -> Self {
        Self {
            rpc_status: ConnectionStatus::Disconnected,
            last_rpc_success: None,
            connection_error: None,
            health: NetworkHealth::default(),
            rpc_endpoint,
            cluster,
            retry_count: 0,
            connecting: false,
        }
    }
}

impl UIState {
    fn new() -> Self {
        Self {
            current_view: AppView::Welcome,
            previous_view: None,
            selections: HashMap::new(),
            focus: UIFocus::MainContent,
            navigation_history: VecDeque::new(),
            modal_state: None,
            status_message: "Initializing...".to_string(),
            status_type: StatusType::Info,
            status_updated_at: Instant::now(),
            title: "Trust Work Escrow v2".to_string(),
            scroll_states: HashMap::new(),
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
        }
    }
}

impl UserPermissions {
    /// Update permissions based on user role
    pub fn update_for_role(&mut self, role: UserRole) {
        match role {
            UserRole::Guest => {
                *self = UserPermissions::default();
            }
            UserRole::Freelancer => {
                self.can_apply_to_jobs = true;
                self.can_submit_work = true;
                self.can_raise_disputes = true;
                self.can_manage_profile = true;
            }
            UserRole::Client => {
                self.can_post_jobs = true;
                self.can_approve_work = true;
                self.can_raise_disputes = true;
                self.can_manage_profile = true;
            }
            UserRole::TeamMember => {
                self.can_apply_to_jobs = true;
                self.can_submit_work = true;
                self.can_raise_disputes = true;
                self.can_manage_profile = true;
            }
            UserRole::TeamOwner => {
                self.can_post_jobs = true;
                self.can_apply_to_jobs = true;
                self.can_create_teams = true;
                self.can_submit_work = true;
                self.can_approve_work = true;
                self.can_raise_disputes = true;
                self.can_manage_profile = true;
            }
            UserRole::Arbiter => {
                self.can_resolve_disputes = true;
                self.can_manage_profile = true;
            }
        }
    }
}