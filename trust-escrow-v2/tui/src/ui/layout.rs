//! Three-Panel Dashboard Layout System for Trust Work Escrow TUI
//!
//! This module provides a comprehensive responsive layout system that adapts to different
//! terminal sizes and user roles. Features include focus management, keyboard navigation,
//! role-specific layouts, and modal overlay support.
//!
//! ## Layout Architecture
//!
//! ### Main Layout (Vertical)
//! ```
//! ┌─────────────── Header (1-3 lines) ────────────────┐
//! │ Title | Network Status | User Info | Notifications │
//! ├──────────── Main Content Area ──────────────────┤
//! │ ┌─ Left Panel ─┐ │ ┌──── Right Panel ──────┐ │
//! │ │ Navigation   │ │ │ Details/Forms/Content │ │
//! │ │ Lists/Menus  │ │ │                       │ │
//! │ │              │ │ │                       │ │
//! │ └──────────────┘ │ └───────────────────────┘ │
//! ├──────────── Footer (1-2 lines) ──────────────────┤
//! │ Help Text | Status | Keyboard Shortcuts          │
//! └─────────────────────────────────────────────────┘
//! ```
//!
//! ### Responsive Breakpoints
//! - Minimum: 80x24 (stacked layout)
//! - Medium: 100x30 (two-panel horizontal)
//! - Large: 120x40+ (full three-panel with sidebars)

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, Widget},
    Frame,
};
use std::cmp::{max, min};

use crate::app::state::{ConnectionStatus, UIFocus, UserRole};
use crate::app::{AppState, AppView, StatusType};

/// Terminal size breakpoints for responsive design
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TerminalSize {
    /// < 80x24 - Critical minimum (mobile-like)
    Tiny,
    /// 80x24 to 100x30 - Basic layout
    Small,
    /// 100x30 to 120x40 - Standard layout
    Medium,
    /// >= 120x40 - Full featured layout
    Large,
}

/// Layout configuration based on terminal size and user preferences
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    /// Terminal size category
    pub size: TerminalSize,
    /// Available area
    pub area: Rect,
    /// Show sidebar panels
    pub show_sidebar: bool,
    /// Header height (1-3 lines)
    pub header_height: u16,
    /// Footer height (1-2 lines)  
    pub footer_height: u16,
    /// Left panel width percentage (10-40%)
    pub left_panel_width: u16,
    /// Right panel width percentage (20-50%)
    pub right_panel_width: u16,
    /// Minimum content width
    pub min_content_width: u16,
    /// Enable modal overlays
    pub enable_modals: bool,
}

/// Panel layout areas for three-panel dashboard
#[derive(Debug, Clone)]
pub struct PanelLayout {
    /// Header area (title, status, notifications)
    pub header: Rect,
    /// Left panel (navigation, lists)
    pub left_panel: Option<Rect>,
    /// Main content area (center)
    pub main_content: Rect,
    /// Right panel (details, forms)
    pub right_panel: Option<Rect>,
    /// Footer area (help, shortcuts)
    pub footer: Rect,
    /// Modal overlay area (if active)
    pub modal_area: Option<Rect>,
}

/// Focus indicator styling for panels
#[derive(Debug, Clone, Copy)]
pub struct FocusStyle {
    /// Border style when focused
    pub focused_border: Style,
    /// Border style when not focused
    pub unfocused_border: Style,
    /// Title style when focused
    pub focused_title: Style,
    /// Title style when not focused
    pub unfocused_title: Style,
    /// Border type
    pub border_type: BorderType,
}

/// Role-specific layout preferences
#[derive(Debug, Clone)]
pub struct RoleLayoutConfig {
    /// User role
    pub role: UserRole,
    /// Default view for this role
    pub default_view: AppView,
    /// Preferred left panel content
    pub left_panel_type: LeftPanelType,
    /// Preferred right panel content
    pub right_panel_type: RightPanelType,
    /// Show additional role-specific elements
    pub show_role_indicators: bool,
}

/// Left panel content types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LeftPanelType {
    /// Main navigation menu
    Navigation,
    /// Jobs list (for freelancers)
    JobsList,
    /// Teams list (for team owners)
    TeamsList,
    /// Disputes list (for arbiters)
    DisputesList,
    /// Quick actions menu
    QuickActions,
}

/// Right panel content types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RightPanelType {
    /// Item details view
    Details,
    /// Form for creating/editing
    Form,
    /// Notification panel
    Notifications,
    /// Wallet and balance info
    WalletInfo,
    /// Statistics dashboard
    Statistics,
    /// Help and documentation
    Help,
}

impl LayoutConfig {
    /// Create layout configuration from terminal area
    pub fn from_area(area: Rect) -> Self {
        let size = Self::determine_size(area);

        let (header_height, footer_height) = match size {
            TerminalSize::Tiny => (1, 1),
            TerminalSize::Small => (2, 1),
            TerminalSize::Medium => (2, 2),
            TerminalSize::Large => (3, 2),
        };

        let show_sidebar = matches!(size, TerminalSize::Medium | TerminalSize::Large);
        let (left_width, right_width) = match size {
            TerminalSize::Tiny | TerminalSize::Small => (0, 0),
            TerminalSize::Medium => (25, 30),
            TerminalSize::Large => (20, 25),
        };

        Self {
            size,
            area,
            show_sidebar,
            header_height,
            footer_height,
            left_panel_width: left_width,
            right_panel_width: right_width,
            min_content_width: 40,
            enable_modals: true,
        }
    }

    /// Determine terminal size category
    fn determine_size(area: Rect) -> TerminalSize {
        match (area.width, area.height) {
            (w, h) if w < 80 || h < 24 => TerminalSize::Tiny,
            (w, h) if w < 100 || h < 30 => TerminalSize::Small,
            (w, h) if w < 120 || h < 40 => TerminalSize::Medium,
            _ => TerminalSize::Large,
        }
    }

    /// Calculate optimal panel layout
    pub fn calculate_layout(&self) -> PanelLayout {
        // Main vertical layout: header, main, footer
        let main_layout = Layout::vertical([
            Constraint::Length(self.header_height),
            Constraint::Min(10), // Main content minimum
            Constraint::Length(self.footer_height),
        ]);
        let [header, main_area, footer] = main_layout.areas(self.area);

        // Calculate main content area panels
        let (left_panel, main_content, right_panel) = if self.show_sidebar {
            self.calculate_three_panel_layout(main_area)
        } else {
            self.calculate_single_panel_layout(main_area)
        };

        PanelLayout {
            header,
            left_panel,
            main_content,
            right_panel,
            footer,
            modal_area: None, // Calculated separately when needed
        }
    }

    /// Calculate three-panel horizontal layout
    fn calculate_three_panel_layout(&self, area: Rect) -> (Option<Rect>, Rect, Option<Rect>) {
        // Calculate constraints ensuring minimum content width
        let left_width = min(
            (area.width * self.left_panel_width) / 100,
            (area.width - self.min_content_width) / 2,
        );
        let right_width = min(
            (area.width * self.right_panel_width) / 100,
            area.width - self.min_content_width - left_width,
        );

        // Create horizontal layout
        let horizontal = Layout::horizontal([
            Constraint::Length(left_width),
            Constraint::Min(self.min_content_width),
            Constraint::Length(right_width),
        ]);
        let [left, main, right] = horizontal.areas(area);

        (
            if left_width > 0 { Some(left) } else { None },
            main,
            if right_width > 0 { Some(right) } else { None },
        )
    }

    /// Calculate single panel layout (for small screens)
    fn calculate_single_panel_layout(&self, area: Rect) -> (Option<Rect>, Rect, Option<Rect>) {
        (None, area, None)
    }

    /// Calculate modal overlay area
    pub fn calculate_modal_area(&self, modal_width: u16, modal_height: u16) -> Rect {
        let area = self.area;

        let modal_width = min(modal_width, area.width.saturating_sub(4));
        let modal_height = min(modal_height, area.height.saturating_sub(4));

        let x = (area.width.saturating_sub(modal_width)) / 2;
        let y = (area.height.saturating_sub(modal_height)) / 2;

        Rect {
            x: area.x + x,
            y: area.y + y,
            width: modal_width,
            height: modal_height,
        }
    }
}

impl FocusStyle {
    /// Create default focus style
    pub fn default() -> Self {
        Self {
            focused_border: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            unfocused_border: Style::default().fg(Color::DarkGray),
            focused_title: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            unfocused_title: Style::default().fg(Color::White),
            border_type: BorderType::Rounded,
        }
    }

    /// Create focus style for specific role
    pub fn for_role(role: UserRole) -> Self {
        let mut style = Self::default();

        style.focused_border = match role {
            UserRole::Freelancer => Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            UserRole::Client => Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            UserRole::TeamOwner => Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
            UserRole::Arbiter => Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            _ => Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        };

        style
    }
}

impl RoleLayoutConfig {
    /// Get layout configuration for user role
    pub fn for_role(role: UserRole) -> Self {
        match role {
            UserRole::Freelancer => Self {
                role,
                default_view: AppView::Jobs,
                left_panel_type: LeftPanelType::JobsList,
                right_panel_type: RightPanelType::Details,
                show_role_indicators: true,
            },
            UserRole::Client => Self {
                role,
                default_view: AppView::Dashboard,
                left_panel_type: LeftPanelType::Navigation,
                right_panel_type: RightPanelType::Statistics,
                show_role_indicators: true,
            },
            UserRole::TeamOwner => Self {
                role,
                default_view: AppView::Teams,
                left_panel_type: LeftPanelType::TeamsList,
                right_panel_type: RightPanelType::Details,
                show_role_indicators: true,
            },
            UserRole::Arbiter => Self {
                role,
                default_view: AppView::Disputes,
                left_panel_type: LeftPanelType::DisputesList,
                right_panel_type: RightPanelType::Details,
                show_role_indicators: true,
            },
            _ => Self {
                role,
                default_view: AppView::Welcome,
                left_panel_type: LeftPanelType::Navigation,
                right_panel_type: RightPanelType::Help,
                show_role_indicators: false,
            },
        }
    }
}

/// Main layout renderer - renders complete three-panel dashboard
pub struct DashboardLayout {
    config: LayoutConfig,
    focus_style: FocusStyle,
    role_config: RoleLayoutConfig,
}

impl DashboardLayout {
    /// Create new dashboard layout
    pub fn new(area: Rect, user_role: UserRole) -> Self {
        let config = LayoutConfig::from_area(area);
        let focus_style = FocusStyle::for_role(user_role);
        let role_config = RoleLayoutConfig::for_role(user_role);

        Self {
            config,
            focus_style,
            role_config,
        }
    }

    /// Render complete dashboard layout
    pub fn render(&self, frame: &mut Frame, app_state: &AppState) {
        let layout = self.config.calculate_layout();

        // Render header
        self.render_header(frame, app_state, layout.header);

        // Render main content panels
        if let Some(left_area) = layout.left_panel {
            self.render_left_panel(frame, app_state, left_area);
        }

        self.render_main_content(frame, app_state, layout.main_content);

        if let Some(right_area) = layout.right_panel {
            self.render_right_panel(frame, app_state, right_area);
        }

        // Render footer
        self.render_footer(frame, app_state, layout.footer);

        // Render modal if present
        if let Some(modal_state) = &app_state.ui_state.modal_state {
            self.render_modal(frame, modal_state, app_state);
        }
    }

    /// Render header with title, status, and user info
    fn render_header(&self, frame: &mut Frame, app_state: &AppState, area: Rect) {
        let header_style = match app_state.get_connection_status() {
            ConnectionStatus::Connected => Style::default().fg(Color::Green),
            ConnectionStatus::Connecting => Style::default().fg(Color::Yellow),
            ConnectionStatus::Error => Style::default().fg(Color::Red),
            _ => Style::default().fg(Color::DarkGray),
        };

        // Multi-line header for larger terminals
        if self.config.header_height >= 3 {
            self.render_full_header(frame, app_state, area);
        } else if self.config.header_height >= 2 {
            self.render_medium_header(frame, app_state, area);
        } else {
            self.render_compact_header(frame, app_state, area);
        }
    }

    /// Render full three-line header
    fn render_full_header(&self, frame: &mut Frame, app_state: &AppState, area: Rect) {
        let header_areas = Layout::vertical([
            Constraint::Length(1), // Title line
            Constraint::Length(1), // Network line
            Constraint::Length(1), // User line
        ])
        .split(area);

        // Title line
        let title_line = Line::from(vec![
            Span::styled("🎯 ", Style::default().fg(Color::Cyan)),
            Span::styled(
                app_state.get_title(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - "),
            Span::styled(
                format!("{:?}", self.role_config.role),
                self.focus_style.focused_border,
            ),
        ]);
        frame.render_widget(
            Paragraph::new(title_line).block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(Color::DarkGray)),
            ),
            header_areas[0],
        );

        // Network status line
        let network_text = format!(
            "🌐 {} | {} | {}",
            app_state.get_network_name(),
            app_state.get_connection_status_text(),
            app_state
                .get_rpc_url()
                .split('/')
                .last()
                .unwrap_or("Unknown")
        );
        frame.render_widget(
            Paragraph::new(network_text)
                .style(self.get_connection_style(app_state.get_connection_status())),
            header_areas[1],
        );

        // User info line
        let user_info = format!(
            "👤 {} | 💰 {} | 🔔 {}",
            app_state
                .get_wallet_address()
                .chars()
                .take(8)
                .collect::<String>()
                + "...",
            "Loading...", // Will be updated with real balance
            app_state.get_unread_notifications()
        );
        frame.render_widget(
            Paragraph::new(user_info).style(Style::default().fg(Color::White)),
            header_areas[2],
        );
    }

    /// Render medium two-line header
    fn render_medium_header(&self, frame: &mut Frame, app_state: &AppState, area: Rect) {
        let header_areas = Layout::vertical([
            Constraint::Length(1), // Title and network
            Constraint::Length(1), // User info
        ])
        .split(area);

        // Title and network line
        let title_network = Line::from(vec![
            Span::styled("🎯 ", Style::default().fg(Color::Cyan)),
            Span::styled(
                app_state.get_title(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | "),
            Span::styled(
                app_state.get_connection_status_text(),
                self.get_connection_style(app_state.get_connection_status()),
            ),
        ]);
        frame.render_widget(
            Paragraph::new(title_network).block(Block::default().borders(Borders::BOTTOM)),
            header_areas[0],
        );

        // User info line
        let user_info = format!(
            "👤 {} | 💰 {} | 🔔 {}",
            app_state
                .get_wallet_address()
                .chars()
                .take(8)
                .collect::<String>()
                + "...",
            "Loading...",
            app_state.get_unread_notifications()
        );
        frame.render_widget(
            Paragraph::new(user_info).style(Style::default().fg(Color::White)),
            header_areas[1],
        );
    }

    /// Render compact single-line header
    fn render_compact_header(&self, frame: &mut Frame, app_state: &AppState, area: Rect) {
        let header_text = Line::from(vec![
            Span::styled("🎯", Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::styled(
                app_state.get_title(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | "),
            Span::styled(
                app_state.get_connection_status_text(),
                self.get_connection_style(app_state.get_connection_status()),
            ),
        ]);

        frame.render_widget(
            Paragraph::new(header_text).block(Block::default().borders(Borders::BOTTOM)),
            area,
        );
    }

    /// Render left panel (navigation, lists)
    fn render_left_panel(&self, frame: &mut Frame, app_state: &AppState, area: Rect) {
        let is_focused = matches!(app_state.ui_state.focus, UIFocus::Menu | UIFocus::JobList);
        let border_style = if is_focused {
            self.focus_style.focused_border
        } else {
            self.focus_style.unfocused_border
        };

        let title = match self.role_config.left_panel_type {
            LeftPanelType::Navigation => "Navigation",
            LeftPanelType::JobsList => "Jobs",
            LeftPanelType::TeamsList => "Teams",
            LeftPanelType::DisputesList => "Disputes",
            LeftPanelType::QuickActions => "Quick Actions",
        };

        let block = Block::default()
            .title(title)
            .title_style(if is_focused {
                self.focus_style.focused_title
            } else {
                self.focus_style.unfocused_title
            })
            .borders(Borders::ALL)
            .border_type(self.focus_style.border_type)
            .border_style(border_style);

        // Render panel content based on type
        match self.role_config.left_panel_type {
            LeftPanelType::Navigation => {
                self.render_navigation_menu(frame, app_state, area, block);
            }
            LeftPanelType::JobsList => {
                self.render_jobs_list(frame, app_state, area, block);
            }
            LeftPanelType::TeamsList => {
                self.render_teams_list(frame, app_state, area, block);
            }
            LeftPanelType::DisputesList => {
                self.render_disputes_list(frame, app_state, area, block);
            }
            LeftPanelType::QuickActions => {
                self.render_quick_actions(frame, app_state, area, block);
            }
        }
    }

    /// Render main content area
    fn render_main_content(&self, frame: &mut Frame, app_state: &AppState, area: Rect) {
        let is_focused = matches!(app_state.ui_state.focus, UIFocus::MainContent);
        let border_style = if is_focused {
            self.focus_style.focused_border
        } else {
            self.focus_style.unfocused_border
        };

        let view_title = match app_state.ui_state.current_view {
            AppView::Welcome => "Welcome",
            AppView::Dashboard => "Dashboard",
            AppView::Jobs => "Jobs",
            AppView::Profile => "Profile",
            AppView::Settings => "Settings",
            AppView::Teams => "Teams",
            AppView::Disputes => "Disputes",
            AppView::Milestones => "Milestones",
            AppView::Help => "Help",
            AppView::JobDetail(_) => "Job Details",
            AppView::TeamDetail(_) => "Team Details",
        };

        let block = Block::default()
            .title(view_title)
            .title_style(if is_focused {
                self.focus_style.focused_title
            } else {
                self.focus_style.unfocused_title
            })
            .borders(Borders::ALL)
            .border_type(self.focus_style.border_type)
            .border_style(border_style);

        // Render based on current view
        match app_state.ui_state.current_view {
            AppView::Welcome => self.render_welcome_content(frame, app_state, area, block),
            AppView::Dashboard => self.render_dashboard_content(frame, app_state, area, block),
            AppView::Jobs => self.render_jobs_content(frame, app_state, area, block),
            AppView::Profile => self.render_profile_content(frame, app_state, area, block),
            AppView::Settings => self.render_settings_content(frame, app_state, area, block),
            _ => self.render_placeholder_content(frame, app_state, area, block),
        }
    }

    /// Render right panel (details, forms, notifications)
    fn render_right_panel(&self, frame: &mut Frame, app_state: &AppState, area: Rect) {
        let is_focused = matches!(
            app_state.ui_state.focus,
            UIFocus::NotificationPanel | UIFocus::InputField
        );
        let border_style = if is_focused {
            self.focus_style.focused_border
        } else {
            self.focus_style.unfocused_border
        };

        let title = match self.role_config.right_panel_type {
            RightPanelType::Details => "Details",
            RightPanelType::Form => "Form",
            RightPanelType::Notifications => "Notifications",
            RightPanelType::WalletInfo => "Wallet",
            RightPanelType::Statistics => "Statistics",
            RightPanelType::Help => "Help",
        };

        let block = Block::default()
            .title(title)
            .title_style(if is_focused {
                self.focus_style.focused_title
            } else {
                self.focus_style.unfocused_title
            })
            .borders(Borders::ALL)
            .border_type(self.focus_style.border_type)
            .border_style(border_style);

        // Render panel content based on type
        match self.role_config.right_panel_type {
            RightPanelType::Details => {
                self.render_details_panel(frame, app_state, area, block);
            }
            RightPanelType::Notifications => {
                self.render_notifications_panel(frame, app_state, area, block);
            }
            RightPanelType::WalletInfo => {
                self.render_wallet_panel(frame, app_state, area, block);
            }
            RightPanelType::Statistics => {
                self.render_statistics_panel(frame, app_state, area, block);
            }
            RightPanelType::Help => {
                self.render_help_panel(frame, app_state, area, block);
            }
            _ => {
                self.render_placeholder_panel(frame, app_state, area, block);
            }
        }
    }

    /// Render footer with help text and keyboard shortcuts
    fn render_footer(&self, frame: &mut Frame, app_state: &AppState, area: Rect) {
        if self.config.footer_height >= 2 {
            self.render_full_footer(frame, app_state, area);
        } else {
            self.render_compact_footer(frame, app_state, area);
        }
    }

    /// Render full two-line footer
    fn render_full_footer(&self, frame: &mut Frame, app_state: &AppState, area: Rect) {
        let footer_areas = Layout::vertical([
            Constraint::Length(1), // Status line
            Constraint::Length(1), // Help line
        ])
        .split(area);

        // Status line
        let status_style = match app_state.ui_state.status_type {
            StatusType::Success => Style::default().fg(Color::Green),
            StatusType::Error => Style::default().fg(Color::Red),
            StatusType::Warning => Style::default().fg(Color::Yellow),
            StatusType::Info => Style::default().fg(Color::Cyan),
        };

        frame.render_widget(
            Paragraph::new(app_state.get_status())
                .style(status_style)
                .block(Block::default().borders(Borders::TOP)),
            footer_areas[0],
        );

        // Help shortcuts
        let help_text = self.get_keyboard_shortcuts(app_state);
        frame.render_widget(
            Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray)),
            footer_areas[1],
        );
    }

    /// Render compact single-line footer
    fn render_compact_footer(&self, frame: &mut Frame, app_state: &AppState, area: Rect) {
        let combined_text = format!(
            "{} | {}",
            app_state.get_status(),
            "q:Quit h:Help Tab:Navigate"
        );

        frame.render_widget(
            Paragraph::new(combined_text)
                .style(Style::default().fg(Color::DarkGray))
                .block(Block::default().borders(Borders::TOP)),
            area,
        );
    }

    // Helper methods for rendering specific panels

    fn render_navigation_menu(
        &self,
        frame: &mut Frame,
        _app_state: &AppState,
        area: Rect,
        block: Block,
    ) {
        let menu_items = vec![
            "📊 Dashboard",
            "💼 Jobs",
            "👤 Profile",
            "👥 Teams",
            "⚖️  Disputes",
            "🎯 Milestones",
            "⚙️  Settings",
            "❓ Help",
        ];

        let content = menu_items.join("\n");
        frame.render_widget(
            Paragraph::new(content)
                .block(block)
                .wrap(ratatui::widgets::Wrap { trim: true }),
            area,
        );
    }

    fn render_jobs_list(&self, frame: &mut Frame, app_state: &AppState, area: Rect, block: Block) {
        let content = if app_state.data_state.jobs.is_empty() {
            "No jobs available\n\nPress 'j' to refresh\njobs list or create\na new job."
        } else {
            "📋 Recent Jobs:\n\n• Web Development\n• Logo Design\n• Smart Contract Audit"
        };

        frame.render_widget(
            Paragraph::new(content)
                .block(block)
                .wrap(ratatui::widgets::Wrap { trim: true }),
            area,
        );
    }

    fn render_teams_list(
        &self,
        frame: &mut Frame,
        _app_state: &AppState,
        area: Rect,
        block: Block,
    ) {
        let content = "👥 Your Teams:\n\n• Dev Team Alpha\n• Design Squad\n• Audit Experts\n\nPress '+' to create\na new team";

        frame.render_widget(
            Paragraph::new(content)
                .block(block)
                .wrap(ratatui::widgets::Wrap { trim: true }),
            area,
        );
    }

    fn render_disputes_list(
        &self,
        frame: &mut Frame,
        _app_state: &AppState,
        area: Rect,
        block: Block,
    ) {
        let content = "⚖️  Active Disputes:\n\n• Case #001: Payment\n• Case #002: Quality\n• Case #003: Deadline\n\nPress 'd' to view\ndispute details";

        frame.render_widget(
            Paragraph::new(content)
                .block(block)
                .wrap(ratatui::widgets::Wrap { trim: true }),
            area,
        );
    }

    fn render_quick_actions(
        &self,
        frame: &mut Frame,
        _app_state: &AppState,
        area: Rect,
        block: Block,
    ) {
        let content = "⚡ Quick Actions:\n\n• Create Job\n• Apply to Job\n• Submit Work\n• Raise Dispute\n• Check Balance";

        frame.render_widget(
            Paragraph::new(content)
                .block(block)
                .wrap(ratatui::widgets::Wrap { trim: true }),
            area,
        );
    }

    fn render_welcome_content(
        &self,
        frame: &mut Frame,
        app_state: &AppState,
        area: Rect,
        block: Block,
    ) {
        let content = format!(
            "🎯 Welcome to Trust Work Escrow v2\n\n\
            ✅ TUI Foundation: Complete\n\
            ✅ State Management: Ready\n\
            ✅ Event System: Active\n\
            ✅ Layout System: Operational\n\n\
            Network: {}\n\
            Connection: {}\n\n\
            Role-specific layout loaded for: {:?}\n\
            Terminal size: {:?} ({}x{})\n\n\
            Press Tab to navigate between panels\n\
            Press 'd' for Dashboard",
            app_state.get_network_name(),
            app_state.get_connection_status_text(),
            self.role_config.role,
            self.config.size,
            self.config.area.width,
            self.config.area.height
        );

        frame.render_widget(
            Paragraph::new(content)
                .block(block)
                .wrap(ratatui::widgets::Wrap { trim: true }),
            area,
        );
    }

    fn render_dashboard_content(
        &self,
        frame: &mut Frame,
        app_state: &AppState,
        area: Rect,
        block: Block,
    ) {
        let content = format!(
            "📊 Dashboard Overview\n\n\
            Wallet: {}\n\
            Balance: Loading...\n\
            Network: {}\n\n\
            📋 Jobs: {}\n\
            🎯 Milestones: {}\n\
            ⚖️  Disputes: {}\n\
            🔔 Notifications: {}\n\n\
            All systems operational ✅",
            app_state
                .get_wallet_address()
                .chars()
                .take(16)
                .collect::<String>()
                + "...",
            app_state.get_network_name(),
            app_state.data_state.jobs.len(),
            app_state.data_state.milestones.len(),
            app_state.data_state.disputes.len(),
            app_state.get_unread_notifications()
        );

        frame.render_widget(
            Paragraph::new(content)
                .block(block)
                .wrap(ratatui::widgets::Wrap { trim: true }),
            area,
        );
    }

    fn render_jobs_content(
        &self,
        frame: &mut Frame,
        _app_state: &AppState,
        area: Rect,
        block: Block,
    ) {
        let content = "📋 Jobs Management\n\nThis panel shows comprehensive job management\nfeatures including:\n\n• Browse available jobs\n• Create new job postings\n• Manage applications\n• Track job progress\n• Handle payments\n\nImplemented in Phase 3.5+";

        frame.render_widget(
            Paragraph::new(content)
                .block(block)
                .wrap(ratatui::widgets::Wrap { trim: true }),
            area,
        );
    }

    fn render_profile_content(
        &self,
        frame: &mut Frame,
        app_state: &AppState,
        area: Rect,
        block: Block,
    ) {
        let content = format!(
            "👤 User Profile\n\n\
            Role: {:?}\n\
            Wallet: {}\n\
            Member since: Today\n\n\
            📈 Statistics:\n\
            • Jobs completed: 0\n\
            • Reputation: ⭐⭐⭐⭐⭐\n\
            • Success rate: 100%\n\n\
            Profile management available in Phase 3.5+",
            self.role_config.role,
            app_state
                .get_wallet_address()
                .chars()
                .take(20)
                .collect::<String>()
                + "..."
        );

        frame.render_widget(
            Paragraph::new(content)
                .block(block)
                .wrap(ratatui::widgets::Wrap { trim: true }),
            area,
        );
    }

    fn render_settings_content(
        &self,
        frame: &mut Frame,
        app_state: &AppState,
        area: Rect,
        block: Block,
    ) {
        let content = format!(
            "⚙️  Settings\n\n\
            Network Configuration:\n\
            • Cluster: {}\n\
            • RPC URL: {}\n\n\
            Layout Configuration:\n\
            • Terminal Size: {:?}\n\
            • Panel Layout: Three-panel\n\
            • Theme: Default\n\n\
            Advanced settings in Phase 3.5+",
            app_state.get_network_name(),
            app_state.get_rpc_url(),
            self.config.size
        );

        frame.render_widget(
            Paragraph::new(content)
                .block(block)
                .wrap(ratatui::widgets::Wrap { trim: true }),
            area,
        );
    }

    fn render_placeholder_content(
        &self,
        frame: &mut Frame,
        _app_state: &AppState,
        area: Rect,
        block: Block,
    ) {
        let content = "🚧 Coming Soon\n\nThis view will be implemented\nin upcoming phases.\n\nCurrent phase: Layout Infrastructure (3.4)\nNext phase: Component Integration (3.5)";

        frame.render_widget(
            Paragraph::new(content)
                .block(block)
                .wrap(ratatui::widgets::Wrap { trim: true }),
            area,
        );
    }

    fn render_details_panel(
        &self,
        frame: &mut Frame,
        _app_state: &AppState,
        area: Rect,
        block: Block,
    ) {
        let content = "📋 Details Panel\n\nItem details will appear\nhere when you select\nitems from the left panel.\n\n• Job details\n• Team information\n• Milestone progress\n• Dispute status";

        frame.render_widget(
            Paragraph::new(content)
                .block(block)
                .wrap(ratatui::widgets::Wrap { trim: true }),
            area,
        );
    }

    fn render_notifications_panel(
        &self,
        frame: &mut Frame,
        app_state: &AppState,
        area: Rect,
        block: Block,
    ) {
        let content = format!(
            "🔔 Notifications ({})\n\n\
            Recent notifications will\nappear here:\n\n\
            • Job applications\n\
            • Payment updates\n\
            • System alerts\n\
            • Network status\n\n\
            No new notifications",
            app_state.get_unread_notifications()
        );

        frame.render_widget(
            Paragraph::new(content)
                .block(block)
                .wrap(ratatui::widgets::Wrap { trim: true }),
            area,
        );
    }

    fn render_wallet_panel(
        &self,
        frame: &mut Frame,
        app_state: &AppState,
        area: Rect,
        block: Block,
    ) {
        let content = format!(
            "💰 Wallet Info\n\n\
            Address:\n{}\n\n\
            Balance:\nLoading...\n\n\
            Network:\n{}\n\n\
            Status: {}\n\
            Last updated: Now",
            app_state
                .get_wallet_address()
                .chars()
                .take(24)
                .collect::<String>()
                + "...",
            app_state.get_network_name(),
            app_state.get_connection_status_text()
        );

        frame.render_widget(
            Paragraph::new(content)
                .block(block)
                .wrap(ratatui::widgets::Wrap { trim: true }),
            area,
        );
    }

    fn render_statistics_panel(
        &self,
        frame: &mut Frame,
        app_state: &AppState,
        area: Rect,
        block: Block,
    ) {
        let content = format!(
            "📈 Statistics\n\n\
            Platform Stats:\n\
            • Jobs: {}\n\
            • Users: {}\n\
            • Teams: {}\n\n\
            Your Stats:\n\
            • Active jobs: 0\n\
            • Completed: 0\n\
            • Earnings: 0 SOL\n\n\
            Network Health: {}%",
            app_state.data_state.jobs.len(),
            app_state.data_state.users.len(),
            app_state.data_state.teams.len(),
            app_state.network_state.health.health_score
        );

        frame.render_widget(
            Paragraph::new(content)
                .block(block)
                .wrap(ratatui::widgets::Wrap { trim: true }),
            area,
        );
    }

    fn render_help_panel(
        &self,
        frame: &mut Frame,
        _app_state: &AppState,
        area: Rect,
        block: Block,
    ) {
        let content = "❓ Help & Shortcuts\n\nNavigation:\n• Tab: Next panel\n• Shift+Tab: Previous\n• Arrow keys: Navigate\n• Enter: Select\n\nActions:\n• q: Quit\n• r: Refresh\n• h: Show help\n• d: Dashboard\n• j: Jobs\n• p: Profile\n• s: Settings";

        frame.render_widget(
            Paragraph::new(content)
                .block(block)
                .wrap(ratatui::widgets::Wrap { trim: true }),
            area,
        );
    }

    fn render_placeholder_panel(
        &self,
        frame: &mut Frame,
        _app_state: &AppState,
        area: Rect,
        block: Block,
    ) {
        let content = "📝 Panel Content\n\nThis panel will display\ncontext-sensitive content\nbased on your current\nselection and role.\n\nImplemented in Phase 3.5+";

        frame.render_widget(
            Paragraph::new(content)
                .block(block)
                .wrap(ratatui::widgets::Wrap { trim: true }),
            area,
        );
    }

    fn render_modal(
        &self,
        frame: &mut Frame,
        modal_state: &crate::app::state::ModalState,
        _app_state: &AppState,
    ) {
        // Calculate modal area (50% of screen, centered)
        let modal_area = self.config.calculate_modal_area(60, 20);

        // Clear background
        frame.render_widget(Clear, modal_area);

        // Render modal dialog
        let modal_block = Block::default()
            .title(modal_state.title.as_str())
            .title_style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(Color::Magenta));

        frame.render_widget(
            Paragraph::new(modal_state.content.as_str())
                .block(modal_block)
                .wrap(ratatui::widgets::Wrap { trim: true })
                .alignment(Alignment::Center),
            modal_area,
        );
    }

    // Helper methods

    fn get_connection_style(&self, status: ConnectionStatus) -> Style {
        match status {
            ConnectionStatus::Connected => Style::default().fg(Color::Green),
            ConnectionStatus::Connecting => Style::default().fg(Color::Yellow),
            ConnectionStatus::Error => Style::default().fg(Color::Red),
            ConnectionStatus::Degraded => Style::default().fg(Color::Magenta),
            ConnectionStatus::Disconnected => Style::default().fg(Color::DarkGray),
        }
    }

    fn get_keyboard_shortcuts(&self, app_state: &AppState) -> String {
        let base_shortcuts = "Tab:Navigate q:Quit r:Refresh h:Help";

        let view_shortcuts = match app_state.ui_state.current_view {
            AppView::Dashboard => " d:Dashboard j:Jobs p:Profile",
            AppView::Jobs => " Enter:Details n:New +: Apply",
            AppView::Profile => " e:Edit s:Settings",
            _ => " d:Dashboard j:Jobs p:Profile s:Settings",
        };

        format!("{}{}", base_shortcuts, view_shortcuts)
    }

    /// Get the layout configuration
    pub fn get_config(&self) -> &LayoutConfig {
        &self.config
    }

    /// Get the terminal area from config
    pub fn get_area(&self) -> Rect {
        self.config.area
    }

    /// Get the terminal size category
    pub fn get_terminal_size(&self) -> TerminalSize {
        self.config.size
    }
}

// Extension traits for app state to provide layout-specific methods
impl AppState {
    /// Get connection status as display text
    pub fn get_connection_status_text(&self) -> &'static str {
        match self.get_connection_status() {
            ConnectionStatus::Connected => "Connected",
            ConnectionStatus::Connecting => "Connecting...",
            ConnectionStatus::Error => "Error",
            ConnectionStatus::Degraded => "Degraded",
            ConnectionStatus::Disconnected => "Disconnected",
        }
    }
}
