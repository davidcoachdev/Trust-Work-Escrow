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

use ratatui::style::Color;

/// Theme for consistent UI colors
#[derive(Clone, Debug)]
pub struct Theme {
    pub name: &'static str,
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub highlight: Color,
    pub border: Color,
    pub title: Color,
    pub muted: Color,
    pub success: Color,
    pub error: Color,
    pub warning: Color,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            name: "dark",
            bg: Color::Rgb(26, 26, 46),
            fg: Color::Rgb(224, 224, 224),
            accent: Color::Rgb(0, 212, 255),
            highlight: Color::Rgb(100, 100, 200),
            border: Color::Rgb(80, 80, 120),
            title: Color::Rgb(0, 212, 255),
            muted: Color::Rgb(100, 100, 130),
            success: Color::Rgb(80, 250, 123),
            error: Color::Rgb(255, 85, 85),
            warning: Color::Rgb(255, 183, 77),
        }
    }

    pub fn role_color(&self, role: UserRole) -> Color {
        match role {
            UserRole::Admin => Color::Rgb(255, 85, 85),
            UserRole::Client => Color::Rgb(0, 212, 255),
            UserRole::Freelancer => Color::Rgb(80, 250, 123),
            UserRole::Arbiter => Color::Rgb(255, 183, 77),
            UserRole::Treasury => Color::Rgb(200, 130, 255),
            _ => Color::White,
        }
    }

    pub fn status_color(&self, status: &JobStatus) -> Color {
        match status {
            JobStatus::Created => self.muted,
            JobStatus::ApplicationsOpen => Color::Rgb(0, 212, 255),
            JobStatus::InProgress => self.warning,
            JobStatus::Submitted => Color::Rgb(200, 130, 255),
            JobStatus::Approved => self.success,
            JobStatus::Cancelled => self.error,
            JobStatus::Disputed => self.error,
            JobStatus::Resolved => self.success,
        }
    }
}

/// Different views/screens in the TUI (comprehensive state version)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppView {
    RoleSelection,
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

/// Actions triggered by menu items (role-specific)
#[derive(Debug, Clone, PartialEq)]
pub enum MenuAction {
    // Admin
    InitializeConfig,
    PauseProgram,
    UnpauseProgram,
    // Client
    CreateJob,
    DepositFunds,
    ApproveWork,
    RejectWork,
    UpdateJob,
    CancelJob,
    // Freelancer
    AcceptJob,
    SubmitWork,
    RaiseDispute,
    // Arbiter
    ResolveDispute,
    // Treasury
    WithdrawFunds,
    // Common
    ShowJob,
    ViewBalances,
    ChangeRole,
    Settings,
}

/// A menu item shown in the left panel
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub label: String,
    pub action: MenuAction,
}

impl MenuItem {
    pub fn new(label: impl Into<String>, action: MenuAction) -> Self {
        Self {
            label: label.into(),
            action,
        }
    }
}

impl UserRole {
    /// Get menu items for this role (matching original TUI structure)
    pub fn menu_items(&self) -> Vec<MenuItem> {
        match self {
            UserRole::Admin => vec![
                MenuItem::new("[0] Initialize Config", MenuAction::InitializeConfig),
                MenuItem::new("[1] Pause Program", MenuAction::PauseProgram),
                MenuItem::new("[2] Unpause Program", MenuAction::UnpauseProgram),
                MenuItem::new("[3] Show Job", MenuAction::ShowJob),
                MenuItem::new("[4] View Balances", MenuAction::ViewBalances),
                MenuItem::new("[5] Change Role", MenuAction::ChangeRole),
                MenuItem::new("[6] Settings", MenuAction::Settings),
            ],
            UserRole::Client => vec![
                MenuItem::new("[0] Create Job", MenuAction::CreateJob),
                MenuItem::new("[1] Deposit Funds", MenuAction::DepositFunds),
                MenuItem::new("[2] Approve Work", MenuAction::ApproveWork),
                MenuItem::new("[3] Reject Work", MenuAction::RejectWork),
                MenuItem::new("[4] Update Job", MenuAction::UpdateJob),
                MenuItem::new("[5] Cancel Job", MenuAction::CancelJob),
                MenuItem::new("[6] Show Job", MenuAction::ShowJob),
                MenuItem::new("[7] View Balances", MenuAction::ViewBalances),
                MenuItem::new("[8] Change Role", MenuAction::ChangeRole),
                MenuItem::new("[9] Settings", MenuAction::Settings),
            ],
            UserRole::Freelancer => vec![
                MenuItem::new("[0] Accept Job", MenuAction::AcceptJob),
                MenuItem::new("[1] Submit Work", MenuAction::SubmitWork),
                MenuItem::new("[2] Raise Dispute", MenuAction::RaiseDispute),
                MenuItem::new("[3] Show Job", MenuAction::ShowJob),
                MenuItem::new("[4] View Balances", MenuAction::ViewBalances),
                MenuItem::new("[5] Change Role", MenuAction::ChangeRole),
                MenuItem::new("[6] Settings", MenuAction::Settings),
            ],
            UserRole::Arbiter => vec![
                MenuItem::new("[0] Resolve Dispute", MenuAction::ResolveDispute),
                MenuItem::new("[1] Show Job", MenuAction::ShowJob),
                MenuItem::new("[2] View Balances", MenuAction::ViewBalances),
                MenuItem::new("[3] Change Role", MenuAction::ChangeRole),
                MenuItem::new("[4] Settings", MenuAction::Settings),
            ],
            UserRole::Treasury => vec![
                MenuItem::new("[0] Withdraw Funds", MenuAction::WithdrawFunds),
                MenuItem::new("[1] Show Job", MenuAction::ShowJob),
                MenuItem::new("[2] View Balances", MenuAction::ViewBalances),
                MenuItem::new("[3] Change Role", MenuAction::ChangeRole),
                MenuItem::new("[4] Settings", MenuAction::Settings),
            ],
            _ => vec![
                MenuItem::new("[0] Show Job", MenuAction::ShowJob),
                MenuItem::new("[1] View Balances", MenuAction::ViewBalances),
                MenuItem::new("[2] Change Role", MenuAction::ChangeRole),
                MenuItem::new("[3] Settings", MenuAction::Settings),
            ],
        }
    }
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
    
    /// Whether mock data is loaded (for demo/hackathon)
    mock_mode: bool,
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
    Admin,          // Platform admin
    Treasury,       // Financial overview
}

impl UserRole {
    /// Get all selectable roles (for role selection screen)
    pub fn selectable() -> &'static [UserRole] {
        &[UserRole::Admin, UserRole::Client, UserRole::Freelancer, UserRole::Arbiter, UserRole::Treasury]
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            UserRole::Guest => "Guest",
            UserRole::Freelancer => "Freelancer",
            UserRole::Client => "Client",
            UserRole::TeamMember => "Team Member",
            UserRole::TeamOwner => "Team Owner",
            UserRole::Arbiter => "Arbiter",
            UserRole::Admin => "Admin",
            UserRole::Treasury => "Treasury",
        }
    }

    /// Get role color name for UI
    pub fn color_name(&self) -> &'static str {
        match self {
            UserRole::Admin => "Red",
            UserRole::Client => "Blue",
            UserRole::Freelancer => "Green",
            UserRole::Arbiter => "Yellow",
            UserRole::Treasury => "Magenta",
            _ => "White",
        }
    }

    /// Get the number key (1-5) for this role
    pub fn number_key(&self) -> Option<u8> {
        match self {
            UserRole::Admin => Some(1),
            UserRole::Client => Some(2),
            UserRole::Freelancer => Some(3),
            UserRole::Arbiter => Some(4),
            UserRole::Treasury => Some(5),
            _ => None,
        }
    }

    /// Get role from number key (1-5)
    pub fn from_number(n: u8) -> Option<UserRole> {
        match n {
            1 => Some(UserRole::Admin),
            2 => Some(UserRole::Client),
            3 => Some(UserRole::Freelancer),
            4 => Some(UserRole::Arbiter),
            5 => Some(UserRole::Treasury),
            _ => None,
        }
    }

    /// Cycle to the next role
    pub fn next(&self) -> UserRole {
        let roles = Self::selectable();
        let idx = roles.iter().position(|r| r == self).unwrap_or(0);
        roles[(idx + 1) % roles.len()]
    }

    /// Get mock user data for this role
    pub fn mock_user_data(&self) -> MockUserData {
        match self {
            UserRole::Admin => MockUserData {
                name: "Platform Admin".to_string(),
                bio: "Trust Work Escrow platform administrator".to_string(),
                balance: 100_000_000_000, // 100 SOL
            },
            UserRole::Client => MockUserData {
                name: "Alice Client".to_string(),
                bio: "Startup founder posting jobs".to_string(),
                balance: 25_000_000_000, // 25 SOL
            },
            UserRole::Freelancer => MockUserData {
                name: "Bob Freelancer".to_string(),
                bio: "Full-stack developer accepting jobs".to_string(),
                balance: 8_000_000_000, // 8 SOL
            },
            UserRole::Arbiter => MockUserData {
                name: "Carol Arbiter".to_string(),
                bio: "Experienced dispute resolver".to_string(),
                balance: 15_000_000_000, // 15 SOL
            },
            UserRole::Treasury => MockUserData {
                name: "Treasury Manager".to_string(),
                bio: "Platform financial overview".to_string(),
                balance: 500_000_000_000, // 500 SOL
            },
            _ => MockUserData {
                name: "Guest".to_string(),
                bio: "Not authenticated".to_string(),
                balance: 0,
            },
        }
    }
}

/// Mock user data for a role
pub struct MockUserData {
    pub name: String,
    pub bio: String,
    pub balance: u64,
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

    /// Role selection state
    pub role_selection: RoleSelectionState,

    /// Create job form state
    pub create_job_form: CreateJobForm,

    /// Job context menu state
    pub job_context_menu: Option<JobContextMenu>,

    /// Currently selected menu item index (left panel)
    pub menu_selection: usize,

    /// Current menu items based on role
    pub menu_items: Vec<MenuItem>,

    /// Current center panel content description
    pub center_content: CenterContent,

    /// UI Theme
    pub theme: Theme,
}

/// What the center panel currently shows
#[derive(Debug, Clone, PartialEq)]
pub enum CenterContent {
    Dashboard,
    JobList,
    Balances,
    Settings,
    CreateJobForm,
    ShowJob,
    ChangeRole,
    Empty,
}

impl Default for CenterContent {
    fn default() -> Self {
        CenterContent::Dashboard
    }
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
    RoleSelect, // Role selection screen
    Form,       // Form input mode
    ContextMenu, // Context menu active
}

/// Create Job form state
#[derive(Debug, Clone)]
pub struct CreateJobForm {
    pub title: String,
    pub description: String,
    pub amount: String,
    pub active_field: usize,
    pub submitted: bool,
    pub success_message: Option<String>,
}

impl CreateJobForm {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            description: String::new(),
            amount: String::new(),
            active_field: 0,
            submitted: false,
            success_message: None,
        }
    }

    pub fn fields() -> &'static [&'static str] {
        &["Title", "Description", "Amount (SOL)"]
    }

    pub fn field_count() -> usize {
        3
    }

    pub fn get_field_value(&self, idx: usize) -> &str {
        match idx {
            0 => &self.title,
            1 => &self.description,
            2 => &self.amount,
            _ => "",
        }
    }

    pub fn get_field_value_mut(&mut self, idx: usize) -> &mut String {
        match idx {
            0 => &mut self.title,
            1 => &mut self.description,
            2 => &mut self.amount,
            _ => panic!("invalid field index"),
        }
    }

    pub fn next_field(&mut self) {
        self.active_field = (self.active_field + 1) % Self::field_count();
    }

    pub fn prev_field(&mut self) {
        self.active_field = if self.active_field > 0 {
            self.active_field - 1
        } else {
            Self::field_count() - 1
        };
    }

    pub fn reset(&mut self) {
        self.title.clear();
        self.description.clear();
        self.amount.clear();
        self.active_field = 0;
        self.submitted = false;
        self.success_message = None;
    }
}

/// Context menu state for job actions
#[derive(Debug, Clone)]
pub struct JobContextMenu {
    pub selected_index: usize,
    pub actions: Vec<ContextMenuAction>,
    pub job_pubkey: Option<Pubkey>,
    pub job_title: String,
    pub completed_message: Option<String>,
}

/// Actions available in the context menu
#[derive(Debug, Clone)]
pub struct ContextMenuAction {
    pub label: String,
    pub action_type: ContextActionType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContextActionType {
    ReleaseFunds,
    RaiseDispute,
    ViewDetails,
    SubmitWork,
    Abandon,
    ResolveDispute,
}

impl JobContextMenu {
    pub fn new_for_role(role: UserRole, job_title: String) -> Self {
        let actions = match role {
            UserRole::Client => vec![
                ContextMenuAction { label: "Release Funds".to_string(), action_type: ContextActionType::ReleaseFunds },
                ContextMenuAction { label: "Raise Dispute".to_string(), action_type: ContextActionType::RaiseDispute },
                ContextMenuAction { label: "View Details".to_string(), action_type: ContextActionType::ViewDetails },
            ],
            UserRole::Freelancer => vec![
                ContextMenuAction { label: "Submit Work".to_string(), action_type: ContextActionType::SubmitWork },
                ContextMenuAction { label: "View Details".to_string(), action_type: ContextActionType::ViewDetails },
                ContextMenuAction { label: "Abandon".to_string(), action_type: ContextActionType::Abandon },
            ],
            UserRole::Arbiter => vec![
                ContextMenuAction { label: "Resolve Dispute".to_string(), action_type: ContextActionType::ResolveDispute },
                ContextMenuAction { label: "View Details".to_string(), action_type: ContextActionType::ViewDetails },
            ],
            UserRole::Admin => vec![
                ContextMenuAction { label: "Release Funds".to_string(), action_type: ContextActionType::ReleaseFunds },
                ContextMenuAction { label: "Raise Dispute".to_string(), action_type: ContextActionType::RaiseDispute },
                ContextMenuAction { label: "Submit Work".to_string(), action_type: ContextActionType::SubmitWork },
                ContextMenuAction { label: "Resolve Dispute".to_string(), action_type: ContextActionType::ResolveDispute },
                ContextMenuAction { label: "View Details".to_string(), action_type: ContextActionType::ViewDetails },
            ],
            _ => vec![
                ContextMenuAction { label: "View Details".to_string(), action_type: ContextActionType::ViewDetails },
            ],
        };

        Self {
            selected_index: 0,
            actions,
            job_pubkey: None,
            job_title,
            completed_message: None,
        }
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else {
            self.selected_index = self.actions.len() - 1;
        }
    }

    pub fn move_down(&mut self) {
        self.selected_index = (self.selected_index + 1) % self.actions.len();
    }

    pub fn get_selected_action(&self) -> Option<&ContextMenuAction> {
        self.actions.get(self.selected_index)
    }
}

/// Role selection state
#[derive(Debug, Clone)]
pub struct RoleSelectionState {
    pub selected_index: usize,
}

impl RoleSelectionState {
    pub fn new() -> Self {
        Self { selected_index: 0 }
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else {
            self.selected_index = UserRole::selectable().len() - 1;
        }
    }

    pub fn move_down(&mut self) {
        let max = UserRole::selectable().len();
        self.selected_index = (self.selected_index + 1) % max;
    }

    pub fn get_selected_role(&self) -> UserRole {
        UserRole::selectable()[self.selected_index]
    }
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
        
        let mut state = Self {
            user_context: UserContext::new(),
            data_state: DataState::new(),
            network_state: NetworkState::new(config.escrow.network.cluster.clone(), 
                                           config.escrow.network.rpc_url.clone()),
            ui_state: UIState::new(),
            performance_state: PerformanceState::default(),
            config,
            client,
            last_update: Instant::now(),
            mock_mode: false,
        };
        state.load_mock_data();
        Ok(state)
    }

    /// Load mock data for demo/hackathon purposes
    fn load_mock_data(&mut self) {
        use solana_sdk::pubkey::Pubkey;
        use chrono::Utc;
        
        self.mock_mode = true;
        
        // Default to Freelancer role
        self.load_role_data(UserRole::Freelancer);
    }

    /// Load role-specific mock data
    pub fn load_role_data(&mut self, role: UserRole) {
        use solana_sdk::pubkey::Pubkey;
        use chrono::Utc;

        let user_data = role.mock_user_data();
        let mock_wallet = Pubkey::new_unique();
        let mock_user = User {
            username: user_data.name,
            bio: user_data.bio,
            wallets: vec![mock_wallet],
            active_wallet: mock_wallet,
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
            bump: 0,
        };
        self.user_context.current_user = Some(mock_user);
        self.user_context.active_wallet = Some(mock_wallet);
        self.user_context.wallet_balance = Some(user_data.balance);
        self.user_context.current_role = role;
        self.user_context.auth_status = AuthStatus::Authenticated;
        self.user_context.permissions.update_for_role(role);

        // Clear existing jobs
        self.data_state.jobs.clear();
        self.data_state.disputes.clear();

        // Generate role-specific jobs
        let client1 = Pubkey::new_unique();
        let client2 = Pubkey::new_unique();
        let freelancer1 = Pubkey::new_unique();
        let freelancer2 = Pubkey::new_unique();
        let _admin_pubkey = mock_wallet;

        let all_jobs = vec![
            (Pubkey::new_unique(), Job {
                job_id: 1,
                client: client1,
                freelancer: Some(freelancer1),
                title: "Web Development for E-commerce".to_string(),
                description: "Build a modern e-commerce website with React and Solana".to_string(),
                amount: 5_000_000_000,
                status: JobStatus::InProgress,
                created_at: Utc::now().timestamp(),
                updated_at: Utc::now().timestamp(),
                bump: 0,
            }),
            (Pubkey::new_unique(), Job {
                job_id: 2,
                client: client2,
                freelancer: None,
                title: "Smart Contract Audit".to_string(),
                description: "Comprehensive security audit for DeFi protocol".to_string(),
                amount: 10_000_000_000,
                status: JobStatus::ApplicationsOpen,
                created_at: Utc::now().timestamp(),
                updated_at: Utc::now().timestamp(),
                bump: 0,
            }),
            (Pubkey::new_unique(), Job {
                job_id: 3,
                client: client1,
                freelancer: Some(freelancer2),
                title: "UI/UX Design for Mobile App".to_string(),
                description: "Create modern responsive design for fintech app".to_string(),
                amount: 3_000_000_000,
                status: JobStatus::Submitted,
                created_at: Utc::now().timestamp(),
                updated_at: Utc::now().timestamp(),
                bump: 0,
            }),
            (Pubkey::new_unique(), Job {
                job_id: 4,
                client: client2,
                freelancer: Some(freelancer1),
                title: "Backend API Development".to_string(),
                description: "Build RESTful API with Node.js and PostgreSQL".to_string(),
                amount: 7_000_000_000,
                status: JobStatus::Disputed,
                created_at: Utc::now().timestamp(),
                updated_at: Utc::now().timestamp(),
                bump: 0,
            }),
            (Pubkey::new_unique(), Job {
                job_id: 5,
                client: client1,
                freelancer: None,
                title: "Landing Page Design".to_string(),
                description: "Design and implement a high-converting landing page".to_string(),
                amount: 2_000_000_000,
                status: JobStatus::Created,
                created_at: Utc::now().timestamp(),
                updated_at: Utc::now().timestamp(),
                bump: 0,
            }),
            (Pubkey::new_unique(), Job {
                job_id: 6,
                client: client2,
                freelancer: Some(freelancer2),
                title: "Mobile App Development".to_string(),
                description: "React Native cross-platform mobile application".to_string(),
                amount: 15_000_000_000,
                status: JobStatus::Approved,
                created_at: Utc::now().timestamp(),
                updated_at: Utc::now().timestamp(),
                bump: 0,
            }),
        ];

        // Filter jobs based on role
        match role {
            UserRole::Client => {
                // Client sees only their posted jobs (using client1 as "our" client)
                for (pk, job) in &all_jobs {
                    if job.client == client1 {
                        self.data_state.jobs.insert(*pk, job.clone());
                    }
                }
            }
            UserRole::Freelancer => {
                // Freelancer sees jobs they're working on + available jobs
                for (pk, job) in &all_jobs {
                    if job.freelancer == Some(freelancer1) || job.status == JobStatus::ApplicationsOpen || job.status == JobStatus::Created {
                        self.data_state.jobs.insert(*pk, job.clone());
                    }
                }
            }
            UserRole::Arbiter => {
                // Arbiter sees only disputed jobs
                for (pk, job) in &all_jobs {
                    if job.status == JobStatus::Disputed {
                        self.data_state.jobs.insert(*pk, job.clone());
                    }
                }
                // Add mock disputes
                let dispute = Dispute {
                    job: all_jobs[3].0,
                    raised_by: client2,
                    arbiter: None,
                    status: DisputeStatus::Open,
                    evidence: vec![],
                    reason: "Freelancer missed deadline".to_string(),
                    created_at: Utc::now().timestamp(),
                    resolved_at: None,
                    bump: 0,
                };
                self.data_state.disputes.insert(Pubkey::new_unique(), dispute);
            }
            UserRole::Admin => {
                // Admin sees ALL jobs
                for (pk, job) in all_jobs {
                    self.data_state.jobs.insert(pk, job);
                }
            }
            UserRole::Treasury => {
                // Treasury sees all jobs for financial overview
                for (pk, job) in all_jobs {
                    self.data_state.jobs.insert(pk, job);
                }
            }
            _ => {
                // Default: show a few jobs
                for (pk, job) in all_jobs.into_iter().take(3) {
                    self.data_state.jobs.insert(pk, job);
                }
            }
        }

        // Mock milestones
        if let Some(job_pubkey) = self.data_state.jobs.keys().next() {
            let milestone1 = Milestone {
                job: *job_pubkey,
                title: "Frontend Development".to_string(),
                description: "Complete React frontend with responsive design".to_string(),
                amount: 2_000_000_000,
                due_date: Some(Utc::now().timestamp() + 7 * 24 * 3600),
                status: MilestoneStatus::Approved,
                index: 0,
                submitted_at: Some(Utc::now().timestamp()),
                approved_at: Some(Utc::now().timestamp()),
                work_url: Some("https://github.com/example/frontend".to_string()),
                rejection_reason: None,
                created_at: Utc::now().timestamp(),
                bump: 0,
            };
            let milestone_pubkey = Pubkey::new_unique();
            self.data_state.milestones.insert(milestone_pubkey, milestone1);
        }

        // Mock notifications
        self.data_state.notifications.clear();
        self.add_notification(
            "Role Changed",
            &format!("Switched to {} role", role.display_name()),
            NotificationType::SystemAlert,
            NotificationPriority::High,
        );
        self.add_notification(
            "New job posted",
            "A new job 'Smart Contract Audit' has been posted",
            NotificationType::JobUpdate,
            NotificationPriority::Medium,
        );

        // Update network status
        self.network_state.rpc_status = ConnectionStatus::Connected;
        self.network_state.last_rpc_success = Some(Instant::now());
    }

    /// Switch to a new role
    pub fn switch_role(&mut self, role: UserRole) {
        self.load_role_data(role);
        self.ui_state.menu_items = role.menu_items();
        self.ui_state.menu_selection = 0;
        self.ui_state.center_content = CenterContent::Dashboard;
        self.ui_state.current_view = AppView::Dashboard;
        self.ui_state.input_mode = InputMode::Normal;
    }
    
    /// Handle keyboard input
    pub async fn handle_input(&mut self, key: KeyCode) -> Result<()> {
        match self.ui_state.input_mode {
            InputMode::Normal => self.handle_normal_input(key).await,
            InputMode::Insert => self.handle_insert_input(key).await,
            InputMode::Command => self.handle_command_input(key).await,
            InputMode::RoleSelect => self.handle_role_select_input(key).await,
            InputMode::Form => self.handle_form_input(key).await,
            InputMode::ContextMenu => self.handle_context_menu_input(key).await,
        }
    }

    /// Handle input in role selection screen
    async fn handle_role_select_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                self.ui_state.role_selection.move_up();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.ui_state.role_selection.move_down();
            }
            KeyCode::Enter => {
                let role = self.ui_state.role_selection.get_selected_role();
                self.switch_role(role);
                self.set_status(&format!("Welcome, {}!", role.display_name()), StatusType::Success);
            }
            KeyCode::Char('1') => { self.switch_role(UserRole::Admin); self.set_status("Welcome, Admin!", StatusType::Success); }
            KeyCode::Char('2') => { self.switch_role(UserRole::Client); self.set_status("Welcome, Client!", StatusType::Success); }
            KeyCode::Char('3') => { self.switch_role(UserRole::Freelancer); self.set_status("Welcome, Freelancer!", StatusType::Success); }
            KeyCode::Char('4') => { self.switch_role(UserRole::Arbiter); self.set_status("Welcome, Arbiter!", StatusType::Success); }
            KeyCode::Char('5') => { self.switch_role(UserRole::Treasury); self.set_status("Welcome, Treasury!", StatusType::Success); }
            KeyCode::Esc | KeyCode::Char('q') => {
                // Allow quit from role selection
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle input in create job form
    async fn handle_form_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Esc => {
                self.ui_state.create_job_form.reset();
                self.ui_state.input_mode = InputMode::Normal;
                self.set_status("Form cancelled", StatusType::Info);
            }
            KeyCode::Tab => {
                self.ui_state.create_job_form.next_field();
            }
            KeyCode::BackTab => {
                self.ui_state.create_job_form.prev_field();
            }
            KeyCode::Up => {
                self.ui_state.create_job_form.prev_field();
            }
            KeyCode::Down => {
                self.ui_state.create_job_form.next_field();
            }
            KeyCode::Enter => {
                // Submit the form
                let form = &self.ui_state.create_job_form;
                if !form.title.trim().is_empty() && !form.amount.trim().is_empty() {
                    let amount: f64 = form.amount.trim().parse().unwrap_or(0.0);
                    let msg = format!(
                        "Job Created Successfully: '{}' for {:.2} SOL",
                        form.title, amount
                    );
                    self.ui_state.create_job_form.reset();
                    self.ui_state.input_mode = InputMode::Normal;
                    self.set_status(&msg, StatusType::Success);
                } else {
                    self.set_status("Please fill in Title and Amount fields", StatusType::Warning);
                }
            }
            KeyCode::Backspace => {
                let field = self.ui_state.create_job_form.active_field;
                self.ui_state.create_job_form.get_field_value_mut(field).pop();
            }
            KeyCode::Char(c) => {
                let field = self.ui_state.create_job_form.active_field;
                self.ui_state.create_job_form.get_field_value_mut(field).push(c);
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle input in context menu
    async fn handle_context_menu_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.ui_state.job_context_menu = None;
                self.ui_state.input_mode = InputMode::Normal;
                self.set_status("Action cancelled", StatusType::Info);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if let Some(ref mut menu) = self.ui_state.job_context_menu {
                    menu.move_up();
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some(ref mut menu) = self.ui_state.job_context_menu {
                    menu.move_down();
                }
            }
            KeyCode::Enter => {
                if let Some(menu) = self.ui_state.job_context_menu.take() {
                    if let Some(action) = menu.get_selected_action() {
                        let msg = match action.action_type {
                            ContextActionType::ReleaseFunds => format!("Funds Released for '{}' (mock)", menu.job_title),
                            ContextActionType::RaiseDispute => format!("Dispute Raised for '{}' (mock)", menu.job_title),
                            ContextActionType::ViewDetails => format!("Viewing details for '{}' (mock)", menu.job_title),
                            ContextActionType::SubmitWork => format!("Work Submitted for '{}' (mock)", menu.job_title),
                            ContextActionType::Abandon => format!("Job '{}' abandoned (mock)", menu.job_title),
                            ContextActionType::ResolveDispute => format!("Dispute Resolved for '{}' (mock)", menu.job_title),
                        };
                        self.set_status(&msg, StatusType::Success);
                    }
                }
                self.ui_state.input_mode = InputMode::Normal;
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle input in normal navigation mode
    async fn handle_normal_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                // If focus is on main content, go back to menu
                if self.ui_state.focus == UIFocus::MainContent {
                    self.ui_state.focus = UIFocus::Menu;
                    self.set_status("Back to menu", StatusType::Info);
                }
                // Otherwise handled in main loop (quit)
            }
            KeyCode::Tab => {
                // Toggle focus between menu and main content
                if self.ui_state.center_content != CenterContent::Empty
                    && self.ui_state.center_content != CenterContent::Dashboard
                {
                    if self.ui_state.focus == UIFocus::Menu {
                        self.ui_state.focus = UIFocus::MainContent;
                        self.set_status("Focus: Main Content (Tab to switch, Esc to go back)", StatusType::Info);
                    } else {
                        self.ui_state.focus = UIFocus::Menu;
                        self.set_status("Focus: Menu", StatusType::Info);
                    }
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                match self.ui_state.focus {
                    UIFocus::Menu | UIFocus::JobList => {
                        // Move menu selection up
                        if !self.ui_state.menu_items.is_empty() {
                            if self.ui_state.menu_selection > 0 {
                                self.ui_state.menu_selection -= 1;
                            } else {
                                self.ui_state.menu_selection = self.ui_state.menu_items.len() - 1;
                            }
                        }
                    }
                    UIFocus::MainContent => {
                        // Move job list selection up
                        let jobs = self.get_jobs_sorted();
                        if !jobs.is_empty() {
                            let sel = self.ui_state.selections.entry("Jobs".to_string()).or_insert(0);
                            if *sel > 0 {
                                *sel -= 1;
                            } else {
                                *sel = jobs.len() - 1;
                            }
                        }
                    }
                    _ => {}
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                match self.ui_state.focus {
                    UIFocus::Menu | UIFocus::JobList => {
                        // Move menu selection down
                        if !self.ui_state.menu_items.is_empty() {
                            self.ui_state.menu_selection = (self.ui_state.menu_selection + 1) % self.ui_state.menu_items.len();
                        }
                    }
                    UIFocus::MainContent => {
                        // Move job list selection down
                        let jobs = self.get_jobs_sorted();
                        if !jobs.is_empty() {
                            let sel = self.ui_state.selections.entry("Jobs".to_string()).or_insert(0);
                            *sel = (*sel + 1) % jobs.len();
                        }
                    }
                    _ => {}
                }
            }
            KeyCode::Enter => {
                match self.ui_state.focus {
                    UIFocus::Menu | UIFocus::JobList => {
                        // Trigger selected menu action
                        if let Some(item) = self.ui_state.menu_items.get(self.ui_state.menu_selection).cloned() {
                            self.handle_menu_action(item.action).await?;
                        }
                    }
                    UIFocus::MainContent => {
                        // Open context menu for selected job
                        let jobs = self.get_jobs_sorted();
                        let sel = self.ui_state.selections.get("Jobs").copied().unwrap_or(0);
                        if let Some((_pk, job)) = jobs.get(sel) {
                            let menu = JobContextMenu::new_for_role(
                                self.user_context.current_role,
                                job.title.clone(),
                            );
                            self.ui_state.job_context_menu = Some(menu);
                            self.ui_state.input_mode = InputMode::ContextMenu;
                            self.set_status("Select action for job (↑↓ + Enter, Esc to cancel)", StatusType::Info);
                        }
                    }
                    _ => {}
                }
            }
            KeyCode::Char('h') => {
                self.show_help();
            }
            // Direct role switching with number keys 1-5
            KeyCode::Char('1') => {
                self.switch_role(UserRole::Admin);
                self.set_status("Role changed to: Admin", StatusType::Success);
            }
            KeyCode::Char('2') => {
                self.switch_role(UserRole::Client);
                self.set_status("Role changed to: Client", StatusType::Success);
            }
            KeyCode::Char('3') => {
                self.switch_role(UserRole::Freelancer);
                self.set_status("Role changed to: Freelancer", StatusType::Success);
            }
            KeyCode::Char('4') => {
                self.switch_role(UserRole::Arbiter);
                self.set_status("Role changed to: Arbiter", StatusType::Success);
            }
            KeyCode::Char('5') => {
                self.switch_role(UserRole::Treasury);
                self.set_status("Role changed to: Treasury", StatusType::Success);
            }
            _ => {}
        }
        Ok(())
    }

    /// Helper: switch focus to main content after selecting a menu item that opens content
    fn focus_content(&mut self, content: CenterContent) {
        self.ui_state.center_content = content;
        self.ui_state.focus = UIFocus::MainContent;
    }

    /// Handle a menu action triggered by Enter on the left panel
    async fn handle_menu_action(&mut self, action: MenuAction) -> Result<()> {
        match action {
            MenuAction::CreateJob => {
                self.ui_state.create_job_form.reset();
                self.ui_state.input_mode = InputMode::Form;
                self.focus_content(CenterContent::CreateJobForm);
                self.set_status("Creating new job - fill in the form", StatusType::Info);
            }
            MenuAction::ShowJob => {
                self.focus_content(CenterContent::ShowJob);
                self.ui_state.current_view = AppView::Jobs;
                self.set_status("↑↓ navigate jobs, Enter for actions, Esc back to menu", StatusType::Info);
            }
            MenuAction::ViewBalances => {
                self.focus_content(CenterContent::Balances);
                self.set_status("Viewing balances", StatusType::Info);
            }
            MenuAction::ChangeRole => {
                self.focus_content(CenterContent::ChangeRole);
                self.set_status("Press 1-5 to switch role", StatusType::Info);
            }
            MenuAction::Settings => {
                self.focus_content(CenterContent::Settings);
                self.ui_state.current_view = AppView::Settings;
                self.set_status("Settings", StatusType::Info);
            }
            MenuAction::DepositFunds => {
                self.focus_content(CenterContent::JobList);
                self.ui_state.current_view = AppView::Jobs;
                self.set_status("Deposit Funds - select a job to deposit", StatusType::Info);
            }
            MenuAction::ApproveWork => {
                self.focus_content(CenterContent::JobList);
                self.ui_state.current_view = AppView::Jobs;
                self.set_status("Approve Work - select a submitted job", StatusType::Success);
            }
            MenuAction::RejectWork => {
                self.focus_content(CenterContent::JobList);
                self.ui_state.current_view = AppView::Jobs;
                self.set_status("Reject Work - select a submitted job", StatusType::Warning);
            }
            MenuAction::UpdateJob => {
                self.focus_content(CenterContent::JobList);
                self.ui_state.current_view = AppView::Jobs;
                self.set_status("Update Job - select a job to update", StatusType::Info);
            }
            MenuAction::CancelJob => {
                self.focus_content(CenterContent::JobList);
                self.ui_state.current_view = AppView::Jobs;
                self.set_status("Cancel Job - select a job to cancel", StatusType::Warning);
            }
            MenuAction::AcceptJob => {
                self.focus_content(CenterContent::JobList);
                self.ui_state.current_view = AppView::Jobs;
                self.set_status("Accept Job - select from job list", StatusType::Info);
            }
            MenuAction::SubmitWork => {
                self.focus_content(CenterContent::JobList);
                self.ui_state.current_view = AppView::Jobs;
                self.set_status("Submit Work - select an in-progress job", StatusType::Info);
            }
            MenuAction::RaiseDispute => {
                self.focus_content(CenterContent::JobList);
                self.ui_state.current_view = AppView::Jobs;
                self.set_status("Raise Dispute - select a job", StatusType::Warning);
            }
            MenuAction::ResolveDispute => {
                self.focus_content(CenterContent::JobList);
                self.ui_state.current_view = AppView::Jobs;
                self.set_status("Resolve Dispute - select a disputed job", StatusType::Info);
            }
            MenuAction::WithdrawFunds => {
                self.set_status("Withdraw Funds selected (mock)", StatusType::Info);
            }
            MenuAction::InitializeConfig => {
                self.set_status("Config initialized (mock)", StatusType::Success);
            }
            MenuAction::PauseProgram => {
                self.set_status("Program paused (mock)", StatusType::Warning);
            }
            MenuAction::UnpauseProgram => {
                self.set_status("Program unpaused (mock)", StatusType::Success);
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
    pub async fn select_current(&mut self) -> Result<()> {
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
            let jobs = self.get_jobs_sorted();
            if let Some((job_pubkey, job)) = jobs.get(*selection) {
                let job_id = job.job_id;
                let mut menu = JobContextMenu::new_for_role(
                    self.user_context.current_role,
                    job.title.clone(),
                );
                menu.job_pubkey = Some(*job_pubkey);
                self.ui_state.job_context_menu = Some(menu);
                self.ui_state.input_mode = InputMode::ContextMenu;
                self.set_status(&format!("Actions for: {}", job.title), StatusType::Info);
            } else {
                self.set_status("No job selected", StatusType::Warning);
            }
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
    
    /// Get wallet balance as string (sync, uses cached balance)
    pub fn get_balance_string_sync(&self) -> String {
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

    /// Get role description for header display
    pub fn get_role_description(&self) -> &'static str {
        match self.user_context.current_role {
            UserRole::Admin => "Platform Administrator - Full Access",
            UserRole::Client => "Client - Post jobs, approve work, release funds",
            UserRole::Freelancer => "Freelancer - Browse jobs, submit work",
            UserRole::Arbiter => "Arbiter - Resolve disputes",
            UserRole::Treasury => "Treasury - Financial overview",
            _ => "Select a role",
        }
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

    /// Get jobs sorted by job_id (ascending)
    pub fn get_jobs_sorted(&self) -> Vec<(Pubkey, Job)> {
        let mut jobs: Vec<(Pubkey, Job)> = self.data_state.jobs.clone().into_iter().collect();
        jobs.sort_by_key(|(_, job)| job.job_id);
        jobs
    }

    /// Get total treasury amount (sum of all job amounts)
    pub fn get_total_treasury(&self) -> u64 {
        self.data_state.jobs.values().map(|j| j.amount).sum()
    }

    /// Get active jobs count
    pub fn get_active_jobs_count(&self) -> usize {
        self.data_state.jobs.values()
            .filter(|j| matches!(j.status, JobStatus::InProgress | JobStatus::Submitted | JobStatus::ApplicationsOpen))
            .count()
    }

    /// Get completed jobs count
    pub fn get_completed_jobs_count(&self) -> usize {
        self.data_state.jobs.values()
            .filter(|j| j.status == JobStatus::Approved)
            .count()
    }

    /// Get disputed jobs count
    pub fn get_disputed_jobs_count(&self) -> usize {
        self.data_state.jobs.values()
            .filter(|j| j.status == JobStatus::Disputed)
            .count()
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
            current_view: AppView::RoleSelection,
            previous_view: None,
            selections: HashMap::new(),
            focus: UIFocus::MainContent,
            navigation_history: VecDeque::new(),
            modal_state: None,
            status_message: "Select your role to begin...".to_string(),
            status_type: StatusType::Info,
            status_updated_at: Instant::now(),
            title: "Trust Work Escrow v2".to_string(),
            scroll_states: HashMap::new(),
            input_mode: InputMode::RoleSelect,
            input_buffer: String::new(),
            role_selection: RoleSelectionState::new(),
            create_job_form: CreateJobForm::new(),
            job_context_menu: None,
            menu_selection: 0,
            menu_items: Vec::new(),
            center_content: CenterContent::Empty,
            theme: Theme::dark(),
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
            UserRole::Admin => {
                self.can_post_jobs = true;
                self.can_apply_to_jobs = true;
                self.can_create_teams = true;
                self.can_submit_work = true;
                self.can_approve_work = true;
                self.can_raise_disputes = true;
                self.can_resolve_disputes = true;
                self.can_manage_profile = true;
            }
            UserRole::Treasury => {
                self.can_manage_profile = true;
            }
        }
    }
}