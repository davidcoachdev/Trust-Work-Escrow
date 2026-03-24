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
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Padding, Paragraph, Widget},
    Frame,
};
use std::cmp::{max, min};

use crate::app::state::{
    CenterContent, ConnectionStatus, InputMode, MenuAction, UIFocus, UserRole,
};
use crate::app::{AppState, AppView, StatusType};
use trust_escrow_sdk::types::JobStatus;

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

    /// Render complete dashboard layout - clean 3-panel design
    pub fn render(&self, frame: &mut Frame, app_state: &AppState) {
        // Role selection gets a full-screen treatment
        if app_state.ui_state.current_view == AppView::RoleSelection {
            self.render_role_selection(frame, app_state);
            return;
        }

        let t = &app_state.ui_state.theme;
        let area = frame.area();

        // Fill background
        frame.render_widget(Block::default().style(Style::default().bg(t.bg)), area);

        // Simple 3-line layout: header, body, footer
        let vertical = Layout::vertical([
            Constraint::Length(1), // header
            Constraint::Min(0),    // body
            Constraint::Length(1), // footer
        ]);
        let [header_area, body_area, footer_area] = vertical.areas(area);

        // Header
        self.render_header(frame, app_state, header_area);

        // Body: left + center + right
        let body_h = Layout::horizontal([
            Constraint::Length(28), // left panel
            Constraint::Min(30),    // center panel
            Constraint::Length(25), // right panel
        ]);
        let [left_area, center_area, right_area] = body_h.areas(body_area);

        self.render_left_panel(frame, app_state, left_area);
        self.render_main_content(frame, app_state, center_area);
        self.render_right_panel(frame, app_state, right_area);

        // Footer
        self.render_footer(frame, app_state, footer_area);

        // Overlays
        if let Some(ref ctx_menu) = app_state.ui_state.job_context_menu {
            self.render_context_menu_overlay(frame, ctx_menu, app_state);
        }

        if app_state.ui_state.input_mode == InputMode::Form {
            self.render_create_job_form(frame, app_state);
        }

        if let Some(modal_state) = &app_state.ui_state.modal_state {
            self.render_modal(frame, modal_state, app_state);
        }
    }

    /// Render header - one clean line
    fn render_header(&self, frame: &mut Frame, app_state: &AppState, area: Rect) {
        let t = &app_state.ui_state.theme;
        let role = app_state.user_context.current_role;
        let role_color = t.role_color(role);

        let header = Line::from(vec![
            Span::styled(
                " Trust Work Escrow ",
                Style::default().fg(t.title).add_modifier(Modifier::BOLD),
            ),
            Span::styled("v2 ", Style::default().fg(t.accent)),
            Span::styled("│ ", Style::default().fg(t.border)),
            Span::styled(
                format!("{} ", role.display_name()),
                Style::default().fg(role_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled("│ ", Style::default().fg(t.border)),
            Span::styled("devnet ", Style::default().fg(t.muted)),
            Span::styled("│ ", Style::default().fg(t.border)),
            Span::styled(
                &app_state.ui_state.status_message,
                Style::default().fg(t.fg),
            ),
        ]);

        frame.render_widget(Paragraph::new(header), area);
    }

    /// Render footer - one clean line with help
    fn render_footer(&self, frame: &mut Frame, app_state: &AppState, area: Rect) {
        let t = &app_state.ui_state.theme;

        let help = if app_state.ui_state.focus == UIFocus::MainContent {
            " Esc:Back  ↑↓:Navigate  Enter:Action "
        } else {
            " ↑↓:Menu  Enter:Select  Tab:Focus  1-5:Role  q:Quit "
        };

        frame.render_widget(
            Paragraph::new(Span::styled(help, Style::default().fg(t.muted))),
            area,
        );
    }

    /// Render left panel - role-specific menu navigation
    fn render_left_panel(&self, frame: &mut Frame, app_state: &AppState, area: Rect) {
        let t = &app_state.ui_state.theme;
        let role = app_state.user_context.current_role;
        let role_color = t.role_color(role);
        let is_focused = app_state.ui_state.focus == UIFocus::Menu;

        let border_color = if is_focused { t.accent } else { t.border };
        let title = format!(" {} Menu ", role.display_name());

        let block = Block::default()
            .title(Span::styled(
                title,
                Style::default().fg(role_color).add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(t.bg));

        let menu_items = &app_state.ui_state.menu_items;
        let selected = app_state.ui_state.menu_selection;

        if menu_items.is_empty() {
            frame.render_widget(Paragraph::new("No menu items").block(block), area);
            return;
        }

        let items: Vec<ListItem> = menu_items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if i == selected {
                    Style::default()
                        .fg(t.bg)
                        .bg(t.highlight)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(t.fg).bg(t.bg)
                };
                ListItem::new(Line::from(Span::styled(format!("  {}", item.label), style)))
            })
            .collect();

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }

    /// Render main content area - routes based on CenterContent state
    fn render_main_content(&self, frame: &mut Frame, app_state: &AppState, area: Rect) {
        let t = &app_state.ui_state.theme;
        let is_focused = app_state.ui_state.focus == UIFocus::MainContent;
        let border_color = if is_focused { t.accent } else { t.border };

        if app_state.ui_state.current_view == AppView::RoleSelection {
            return;
        }

        let view_title = match &app_state.ui_state.center_content {
            CenterContent::Dashboard => "Dashboard",
            CenterContent::JobList | CenterContent::ShowJob => "Jobs",
            CenterContent::Balances => "Balances",
            CenterContent::Settings => "Settings",
            CenterContent::CreateJobForm => "Create Job",
            CenterContent::ChangeRole => "Change Role",
            CenterContent::Empty => "Welcome",
        };

        let block = Block::default()
            .title(Span::styled(
                format!(" {} ", view_title),
                Style::default().fg(t.title).add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(t.bg));

        match &app_state.ui_state.center_content {
            CenterContent::Dashboard => {
                self.render_dashboard_content(frame, app_state, area, block)
            }
            CenterContent::JobList | CenterContent::ShowJob => {
                self.render_jobs_content(frame, app_state, area, block)
            }
            CenterContent::Balances => self.render_balances_content(frame, app_state, area, block),
            CenterContent::Settings => self.render_settings_content(frame, app_state, area, block),
            CenterContent::CreateJobForm => {
                self.render_dashboard_content(frame, app_state, area, block)
            }
            CenterContent::ChangeRole => {
                self.render_change_role_content(frame, app_state, area, block)
            }
            CenterContent::Empty => self.render_welcome_content(frame, app_state, area, block),
        }
    }

    /// Render right panel - contextual info
    fn render_right_panel(&self, frame: &mut Frame, app_state: &AppState, area: Rect) {
        let t = &app_state.ui_state.theme;

        let block = Block::default()
            .title(Span::styled(
                " Info ",
                Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(t.border))
            .style(Style::default().bg(t.bg));

        let mut lines = Vec::new();
        let role = app_state.user_context.current_role;

        lines.push(Line::from(Span::styled(
            format!(" Role: {}", role.display_name()),
            Style::default()
                .fg(t.role_color(role))
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " Network",
            Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  devnet",
            Style::default().fg(t.fg),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " Balance",
            Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  1.00 SOL",
            Style::default().fg(t.success),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " Jobs",
            Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
        )));
        let job_count = app_state.data_state.jobs.len();
        lines.push(Line::from(Span::styled(
            format!("  {} loaded", job_count),
            Style::default().fg(t.fg),
        )));

        frame.render_widget(Paragraph::new(lines).block(block), area);
    }

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
        let jobs = app_state.get_jobs_sorted();
        let selected_index = app_state
            .ui_state
            .selections
            .get("Jobs")
            .copied()
            .unwrap_or(0);

        if jobs.is_empty() {
            let content =
                "No jobs available\n\nPress 'j' to refresh\njobs list or create\na new job.";
            frame.render_widget(
                Paragraph::new(content)
                    .block(block)
                    .wrap(ratatui::widgets::Wrap { trim: true }),
                area,
            );
            return;
        }

        let mut lines = Vec::new();
        lines.push(Line::from(Span::styled(
            "📋 Recent Jobs:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from("")); // empty line

        for (i, (_pubkey, job)) in jobs.iter().enumerate() {
            let is_selected = i == selected_index;
            let prefix = if is_selected { "▶ " } else { "• " };
            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let status_color = match job.status {
                trust_escrow_sdk::types::JobStatus::Created => Color::DarkGray,
                trust_escrow_sdk::types::JobStatus::ApplicationsOpen => Color::Blue,
                trust_escrow_sdk::types::JobStatus::InProgress => Color::Yellow,
                trust_escrow_sdk::types::JobStatus::Submitted => Color::Magenta,
                trust_escrow_sdk::types::JobStatus::Approved => Color::Green,
                trust_escrow_sdk::types::JobStatus::Cancelled => Color::Red,
                trust_escrow_sdk::types::JobStatus::Disputed => Color::Red,
                trust_escrow_sdk::types::JobStatus::Resolved => Color::Green,
            };

            let amount_sol = job.amount as f64 / 1_000_000_000.0;
            let line = Line::from(vec![
                Span::raw(prefix),
                Span::styled(job.title.clone(), style),
                Span::raw(" ("),
                Span::styled(
                    format!("{:.2} SOL", amount_sol),
                    Style::default().fg(Color::Green),
                ),
                Span::raw(") "),
                Span::styled(
                    format!("{:?}", job.status),
                    Style::default().fg(status_color),
                ),
            ]);
            lines.push(line);
        }

        let content = Text::from(lines);
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
        let role = app_state.user_context.current_role;
        let role_color = match role {
            UserRole::Admin => Color::Red,
            UserRole::Client => Color::Blue,
            UserRole::Freelancer => Color::Green,
            UserRole::Arbiter => Color::Yellow,
            UserRole::Treasury => Color::Magenta,
            _ => Color::White,
        };

        let mut lines = Vec::new();
        lines.push(Line::from(vec![
            Span::styled("📊 ", Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("Dashboard - {}", role.display_name()),
                Style::default().fg(role_color).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(""));

        // Common stats
        lines.push(Line::from(vec![
            Span::styled("Wallet: ", Style::default().fg(Color::Yellow)),
            Span::raw(
                app_state
                    .get_wallet_address()
                    .chars()
                    .take(16)
                    .collect::<String>()
                    + "...",
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Balance: ", Style::default().fg(Color::Yellow)),
            Span::raw(app_state.get_balance_string_sync()),
        ]));
        lines.push(Line::from(""));

        // Role-specific dashboard
        match role {
            UserRole::Admin => {
                lines.push(Line::from(Span::styled(
                    "Platform Overview:",
                    Style::default().fg(Color::Cyan),
                )));
                lines.push(Line::from(format!(
                    "  📋 Total Jobs: {}",
                    app_state.data_state.jobs.len()
                )));
                lines.push(Line::from(format!(
                    "  ✅ Active Jobs: {}",
                    app_state.get_active_jobs_count()
                )));
                lines.push(Line::from(format!(
                    "  🏆 Completed: {}",
                    app_state.get_completed_jobs_count()
                )));
                lines.push(Line::from(format!(
                    "  ⚠️  Disputed: {}",
                    app_state.get_disputed_jobs_count()
                )));
                lines.push(Line::from(format!(
                    "  🎯 Milestones: {}",
                    app_state.data_state.milestones.len()
                )));
                let total: u64 = app_state.data_state.jobs.values().map(|j| j.amount).sum();
                lines.push(Line::from(format!(
                    "  💰 Total Volume: {:.2} SOL",
                    total as f64 / 1_000_000_000.0
                )));
            }
            UserRole::Client => {
                lines.push(Line::from(Span::styled(
                    "Your Jobs:",
                    Style::default().fg(Color::Blue),
                )));
                lines.push(Line::from(format!(
                    "  📋 Posted Jobs: {}",
                    app_state.data_state.jobs.len()
                )));
                lines.push(Line::from(format!(
                    "  ✅ Active: {}",
                    app_state.get_active_jobs_count()
                )));
                lines.push(Line::from(format!(
                    "  🏆 Completed: {}",
                    app_state.get_completed_jobs_count()
                )));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "Quick Actions:",
                    Style::default().fg(Color::DarkGray),
                )));
                lines.push(Line::from("  • j: View Jobs  • c: Create Job"));
            }
            UserRole::Freelancer => {
                lines.push(Line::from(Span::styled(
                    "Your Work:",
                    Style::default().fg(Color::Green),
                )));
                lines.push(Line::from(format!(
                    "  📋 Available Jobs: {}",
                    app_state.data_state.jobs.len()
                )));
                lines.push(Line::from(format!(
                    "  ✅ Active: {}",
                    app_state.get_active_jobs_count()
                )));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "Quick Actions:",
                    Style::default().fg(Color::DarkGray),
                )));
                lines.push(Line::from("  • j: Browse Jobs  • Enter: View Details"));
            }
            UserRole::Arbiter => {
                lines.push(Line::from(Span::styled(
                    "Disputes:",
                    Style::default().fg(Color::Yellow),
                )));
                lines.push(Line::from(format!(
                    "  ⚖️  Open Disputes: {}",
                    app_state.data_state.disputes.len()
                )));
                lines.push(Line::from(format!(
                    "  📋 Jobs in Dispute: {}",
                    app_state.get_disputed_jobs_count()
                )));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "Quick Actions:",
                    Style::default().fg(Color::DarkGray),
                )));
                lines.push(Line::from("  • j: View Disputed Jobs  • Enter: Resolve"));
            }
            UserRole::Treasury => {
                let total: u64 = app_state.data_state.jobs.values().map(|j| j.amount).sum();
                lines.push(Line::from(Span::styled(
                    "Financial Overview:",
                    Style::default().fg(Color::Magenta),
                )));
                lines.push(Line::from(format!(
                    "  💰 Total Locked: {:.2} SOL",
                    total as f64 / 1_000_000_000.0
                )));
                lines.push(Line::from(format!(
                    "  📋 Total Jobs: {}",
                    app_state.data_state.jobs.len()
                )));
                lines.push(Line::from(format!(
                    "  ✅ Active: {}",
                    app_state.get_active_jobs_count()
                )));
                lines.push(Line::from(format!(
                    "  🏆 Completed: {}",
                    app_state.get_completed_jobs_count()
                )));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "Quick Actions:",
                    Style::default().fg(Color::DarkGray),
                )));
                lines.push(Line::from("  • j: View All Jobs"));
            }
            _ => {}
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "All systems operational ✅",
            Style::default().fg(Color::Green),
        )));

        frame.render_widget(
            Paragraph::new(lines)
                .block(block)
                .wrap(ratatui::widgets::Wrap { trim: true }),
            area,
        );
    }

    fn render_jobs_content(
        &self,
        frame: &mut Frame,
        app_state: &AppState,
        area: Rect,
        block: Block,
    ) {
        let t = &app_state.ui_state.theme;
        let role = app_state.user_context.current_role;
        let jobs = app_state.get_jobs_sorted();
        let selected = app_state
            .ui_state
            .selections
            .get("Jobs")
            .copied()
            .unwrap_or(0);

        let role_header = match role {
            UserRole::Client => "Your Posted Jobs",
            UserRole::Freelancer => "Available & Assigned Jobs",
            UserRole::Arbiter => "Jobs with Disputes",
            UserRole::Admin => "All Platform Jobs",
            UserRole::Treasury => "Financial Overview",
            _ => "Jobs",
        };

        let mut lines = Vec::new();
        lines.push(Line::from(Span::styled(
            role_header,
            Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        if jobs.is_empty() {
            lines.push(Line::from(Span::styled(
                "No jobs available for this role.",
                Style::default().fg(t.muted),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Press 'c' to create a new job",
                Style::default().fg(t.muted),
            )));
        } else {
            for (i, (_pk, job)) in jobs.iter().enumerate() {
                let is_selected = i == selected;
                let prefix = if is_selected { "▶ " } else { "  " };
                let style = if is_selected {
                    Style::default()
                        .fg(t.bg)
                        .bg(t.highlight)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(t.fg).bg(t.bg)
                };

                let status_color = t.status_color(&job.status);
                let amount_sol = job.amount as f64 / 1_000_000_000.0;

                lines.push(Line::from(vec![
                    Span::raw(prefix),
                    Span::styled(format!("#{} ", job.job_id), Style::default().fg(t.muted)),
                    Span::styled(&job.title, style),
                    Span::raw(" "),
                    Span::styled(
                        format!("{:.1} SOL", amount_sol),
                        Style::default().fg(t.success),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        format!("{:?}", job.status),
                        Style::default().fg(status_color),
                    ),
                ]));
            }
        }

        frame.render_widget(
            Paragraph::new(lines)
                .block(block)
                .wrap(ratatui::widgets::Wrap { trim: true }),
            area,
        );
    }

    fn render_job_detail_content(
        &self,
        frame: &mut Frame,
        app_state: &AppState,
        job_id: u64,
        area: Rect,
        block: Block,
    ) {
        // Find job by job_id
        let job_opt = app_state
            .data_state
            .jobs
            .values()
            .find(|j| j.job_id == job_id);

        let content = match job_opt {
            Some(job) => {
                let amount_sol = job.amount as f64 / 1_000_000_000.0;
                let client_str = job.client.to_string();
                let freelancer_str = job
                    .freelancer
                    .map(|pk| pk.to_string())
                    .unwrap_or_else(|| "Not assigned".to_string());

                let status_color = match job.status {
                    trust_escrow_sdk::types::JobStatus::Created => Color::DarkGray,
                    trust_escrow_sdk::types::JobStatus::ApplicationsOpen => Color::Blue,
                    trust_escrow_sdk::types::JobStatus::InProgress => Color::Yellow,
                    trust_escrow_sdk::types::JobStatus::Submitted => Color::Magenta,
                    trust_escrow_sdk::types::JobStatus::Approved => Color::Green,
                    trust_escrow_sdk::types::JobStatus::Cancelled => Color::Red,
                    trust_escrow_sdk::types::JobStatus::Disputed => Color::Red,
                    trust_escrow_sdk::types::JobStatus::Resolved => Color::Green,
                };

                format!(
                    "📋 Job Details\n\n\
                    Title: {}\n\n\
                    Description:\n{}\n\n\
                    Amount: {:.2} SOL\n\
                    Status: {:?}\n\n\
                    Client: {}\n\
                    Freelancer: {}\n\n\
                    Created: {}\n\
                    Updated: {}\n\n\
                    Press Backspace to return to list",
                    job.title,
                    job.description,
                    amount_sol,
                    job.status,
                    client_str,
                    freelancer_str,
                    chrono::DateTime::from_timestamp(job.created_at, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                        .unwrap_or_else(|| "Unknown".to_string()),
                    chrono::DateTime::from_timestamp(job.updated_at, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                        .unwrap_or_else(|| "Unknown".to_string())
                )
            }
            None => format!("Job #{} not found\n\nPress Backspace to return", job_id),
        };

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
            "Settings\n\n\
            Network Configuration:\n\
            - Cluster: {}\n\
            - RPC URL: {}\n\n\
            Layout Configuration:\n\
            - Terminal Size: {:?}\n\
            - Panel Layout: Three-panel\n\
            - Theme: Default\n\n\
            Role: {}\n\
            Press 1-5 to switch role",
            app_state.get_network_name(),
            app_state.get_rpc_url(),
            self.config.size,
            app_state.user_context.current_role.display_name()
        );

        frame.render_widget(
            Paragraph::new(content)
                .block(block)
                .wrap(ratatui::widgets::Wrap { trim: true }),
            area,
        );
    }

    fn render_balances_content(
        &self,
        frame: &mut Frame,
        app_state: &AppState,
        area: Rect,
        block: Block,
    ) {
        let role = app_state.user_context.current_role;
        let total_jobs = app_state.data_state.jobs.len();
        let active_jobs = app_state.get_active_jobs_count();
        let completed_jobs = app_state.get_completed_jobs_count();
        let disputed_jobs = app_state.get_disputed_jobs_count();
        let total_treasury = app_state.get_total_treasury();

        let mut lines = Vec::new();
        lines.push(Line::from(Span::styled(
            "Account Balances",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Role: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                role.display_name(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Wallet: ", Style::default().fg(Color::Yellow)),
            Span::raw(
                app_state
                    .get_wallet_address()
                    .chars()
                    .take(16)
                    .collect::<String>()
                    + "...",
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Balance: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                app_state.get_balance_string_sync(),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Platform Stats:",
            Style::default().fg(Color::Cyan),
        )));
        lines.push(Line::from(format!("  Total Jobs: {}", total_jobs)));
        lines.push(Line::from(format!("  Active Jobs: {}", active_jobs)));
        lines.push(Line::from(format!("  Completed: {}", completed_jobs)));
        lines.push(Line::from(format!("  Disputed: {}", disputed_jobs)));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Treasury: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("{:.2} SOL", total_treasury as f64 / 1_000_000_000.0),
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        frame.render_widget(
            Paragraph::new(lines)
                .block(block)
                .wrap(ratatui::widgets::Wrap { trim: true }),
            area,
        );
    }

    fn render_change_role_content(
        &self,
        frame: &mut Frame,
        app_state: &AppState,
        area: Rect,
        block: Block,
    ) {
        let current_role = app_state.user_context.current_role;
        let roles = UserRole::selectable();
        let role_icons = ["1", "2", "3", "4", "5"];
        let role_colors = [
            Color::Red,
            Color::Blue,
            Color::Green,
            Color::Yellow,
            Color::Magenta,
        ];

        let mut lines = Vec::new();
        lines.push(Line::from(Span::styled(
            "Switch Role",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Current: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                current_role.display_name(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Available Roles:",
            Style::default().fg(Color::White),
        )));
        lines.push(Line::from(""));

        for (i, role) in roles.iter().enumerate() {
            let is_current = *role == current_role;
            let prefix = if is_current { "  " } else { "  " };
            let indicator = if is_current { " <-- current" } else { "" };
            let style = if is_current {
                Style::default()
                    .fg(role_colors[i])
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            lines.push(Line::from(vec![
                Span::raw(prefix),
                Span::styled(
                    format!("[{}] ", role_icons[i]),
                    Style::default().fg(role_colors[i]),
                ),
                Span::styled(format!("{}", role.display_name()), style),
                Span::styled(indicator, Style::default().fg(Color::DarkGray)),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Press 1-5 to switch role",
            Style::default().fg(Color::DarkGray),
        )));

        frame.render_widget(
            Paragraph::new(lines)
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
            Balance:\n{}\n\n\
            Network:\n{}\n\n\
            Status: {}\n\
            Last updated: Now",
            app_state
                .get_wallet_address()
                .chars()
                .take(24)
                .collect::<String>()
                + "...",
            app_state.get_balance_string_sync(),
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

    /// Render the role selection screen (full screen)
    fn render_role_selection(&self, frame: &mut Frame, app_state: &AppState) {
        use ratatui::widgets::Clear;
        let area = frame.area();

        // Clear the screen
        frame.render_widget(Clear, area);

        let selected = app_state.ui_state.role_selection.selected_index;
        let roles = UserRole::selectable();

        let mut lines = Vec::new();

        // ASCII art banner
        lines.push(Line::from(Span::styled(
            "╔══════════════════════════════════════════════════════════════╗",
            Style::default().fg(Color::Cyan),
        )));
        lines.push(Line::from(Span::styled(
            "║                                                              ║",
            Style::default().fg(Color::Cyan),
        )));
        lines.push(Line::from(vec![
            Span::styled("║     ", Style::default().fg(Color::Cyan)),
            Span::styled(
                "🔷 TRUST WORK ESCROW v2 🔷",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("               ║", Style::default().fg(Color::Cyan)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("║       ", Style::default().fg(Color::Cyan)),
            Span::styled(
                "Decentralized Freelance Platform",
                Style::default().fg(Color::White),
            ),
            Span::styled("       ║", Style::default().fg(Color::Cyan)),
        ]));
        lines.push(Line::from(Span::styled(
            "║                                                              ║",
            Style::default().fg(Color::Cyan),
        )));
        lines.push(Line::from(Span::styled(
            "╚══════════════════════════════════════════════════════════════╝",
            Style::default().fg(Color::Cyan),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "                    SELECT YOUR ROLE",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        // Role options
        let role_icons = ["👑", "💼", "💻", "⚖️ ", "💰"];
        let role_colors = [
            Color::Red,
            Color::Blue,
            Color::Green,
            Color::Yellow,
            Color::Magenta,
        ];
        let role_descs = [
            "Full platform access, manage all jobs & users",
            "Post jobs, approve work, release funds",
            "Browse jobs, apply, submit work & earn",
            "Resolve disputes between clients & freelancers",
            "Platform financial overview & treasury",
        ];

        for (i, role) in roles.iter().enumerate() {
            let is_selected = i == selected;
            let prefix = if is_selected { " ▶ " } else { "   " };
            let style = if is_selected {
                Style::default()
                    .fg(role_colors[i])
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let num = (i + 1).to_string();

            lines.push(Line::from(vec![
                Span::raw("          "),
                Span::styled(prefix, style),
                Span::styled(format!("[{}] ", num), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{} ", role_icons[i]), style),
                Span::styled(format!("{:<12}", role.display_name()), style),
                Span::raw("  "),
                Span::styled(
                    role_descs[i],
                    if is_selected {
                        Style::default().fg(Color::White)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                ),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "         Press 1-5 for quick select, or ↑↓ + Enter",
            Style::default().fg(Color::DarkGray),
        )));

        let paragraph = Paragraph::new(lines)
            .alignment(ratatui::layout::Alignment::Left)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Cyan)),
            );

        // Center the content
        let vertical_chunks = Layout::vertical([
            Constraint::Percentage(15),
            Constraint::Min(20),
            Constraint::Percentage(15),
        ])
        .split(area);

        frame.render_widget(paragraph, vertical_chunks[1]);
    }

    /// Render the create job form as an overlay
    fn render_create_job_form(&self, frame: &mut Frame, app_state: &AppState) {
        use ratatui::widgets::Clear;

        let form = &app_state.ui_state.create_job_form;
        let modal_area = self.config.calculate_modal_area(60, 18);

        // Clear background
        frame.render_widget(Clear, modal_area);

        let mut lines = Vec::new();
        lines.push(Line::from(Span::styled(
            "Create New Job",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        let fields = ["Title", "Description", "Amount (SOL)"];
        for (i, field) in fields.iter().enumerate() {
            let is_active = i == form.active_field;
            let value = form.get_field_value(i);

            let label_style = if is_active {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let cursor = if is_active { "▶ " } else { "  " };
            let display_value = if value.is_empty() {
                "(empty)".to_string()
            } else {
                value.to_string()
            };

            lines.push(Line::from(vec![
                Span::styled(format!("{}{}: ", cursor, field), label_style),
                Span::styled(
                    display_value,
                    if is_active {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                ),
            ]));
            lines.push(Line::from(""));
        }

        lines.push(Line::from(Span::styled(
            "Tab/↑↓: Navigate | Enter: Submit | Esc: Cancel",
            Style::default().fg(Color::DarkGray),
        )));

        let block = Block::default()
            .title(" Create Job ")
            .title_style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(Color::Green));

        frame.render_widget(
            Paragraph::new(lines)
                .block(block)
                .wrap(ratatui::widgets::Wrap { trim: true }),
            modal_area,
        );
    }

    /// Render context menu overlay for job actions
    fn render_context_menu_overlay(
        &self,
        frame: &mut Frame,
        ctx_menu: &crate::app::state::JobContextMenu,
        _app_state: &AppState,
    ) {
        use ratatui::widgets::Clear;

        let menu_height = (ctx_menu.actions.len() + 4) as u16;
        let modal_area = self.config.calculate_modal_area(40, menu_height);

        // Clear background
        frame.render_widget(Clear, modal_area);

        let mut lines = Vec::new();
        lines.push(Line::from(Span::styled(
            format!("Actions for: {}", ctx_menu.job_title),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        for (i, action) in ctx_menu.actions.iter().enumerate() {
            let is_selected = i == ctx_menu.selected_index;
            let prefix = if is_selected { "▶ " } else { "  " };
            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            lines.push(Line::from(vec![
                Span::raw(prefix),
                Span::styled(&action.label, style),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "↑↓: Navigate | Enter: Confirm | Esc: Cancel",
            Style::default().fg(Color::DarkGray),
        )));

        let block = Block::default()
            .title(" Job Actions ")
            .title_style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(Color::Magenta));

        frame.render_widget(
            Paragraph::new(lines)
                .block(block)
                .wrap(ratatui::widgets::Wrap { trim: true }),
            modal_area,
        );
    }

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
        let role = app_state.user_context.current_role;
        "Up/Down:Navigate Menu | Enter:Select | 1-5:Switch Role | Esc:Back | q:Quit".to_string()
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
