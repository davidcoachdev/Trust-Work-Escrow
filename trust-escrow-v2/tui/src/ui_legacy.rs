//! UI rendering for TUI - Legacy and Enhanced Layout Integration
//!
//! This module provides backward-compatible UI rendering while integrating
//! the new three-panel layout system from Task 3.4. It maintains compatibility
//! with existing Task 3.1-3.3 functionality while providing enhanced layout
//! capabilities.
//!
//! ## Layout Modes
//!
//! ### Enhanced Mode (Default)
//! - Three-panel responsive dashboard
//! - Role-specific layouts
//! - Focus management
//! - Modal support
//!
//! ### Legacy Mode (Compatibility)
//! - Original single-panel layout
//! - Task 3.1 welcome screen
//! - Basic header/footer

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{state::AppView, App};
use crate::ui::{modal, navigation, utils, TerminalSize, UIRenderer};

// Global UI renderer instance (will be initialized in main.rs)
thread_local! {
    static UI_RENDERER: std::cell::RefCell<Option<UIRenderer>> = const { std::cell::RefCell::new(None) };
}

/// Initialize the UI system - call this in main.rs
pub fn initialize_ui() {
    UI_RENDERER.with(|renderer| {
        *renderer.borrow_mut() = Some(UIRenderer::new());
    });
}

/// Main UI drawing function - enhanced with layout system
pub fn draw_enhanced(f: &mut Frame, app: &App) {
    UI_RENDERER.with(|renderer| {
        if let Some(ref mut ui_renderer) = renderer.borrow_mut().as_mut() {
            ui_renderer.render(f, app);
        } else {
            // Fallback to legacy if renderer not initialized
            draw_legacy(f, app);
        }
    });
}

/// Toggle between enhanced and legacy layout modes
pub fn toggle_layout_mode() {
    UI_RENDERER.with(|renderer| {
        if let Some(ref mut ui_renderer) = renderer.borrow_mut().as_mut() {
            ui_renderer.toggle_layout_mode();
        }
    });
}

/// Check if using enhanced layout
pub fn is_enhanced_mode() -> bool {
    UI_RENDERER.with(|renderer| {
        renderer
            .borrow()
            .as_ref()
            .map(|ui| ui.is_enhanced_mode())
            .unwrap_or(false)
    })
}

/// Get current terminal size category
pub fn get_terminal_size() -> Option<TerminalSize> {
    UI_RENDERER.with(|renderer| {
        renderer
            .borrow()
            .as_ref()
            .and_then(|ui| ui.get_terminal_size())
    })
}

/// Main drawing function - backward compatible with enhanced layout support
pub fn draw(f: &mut Frame, app: &App) {
    // Try enhanced layout first, fallback to legacy
    if is_enhanced_mode() {
        draw_enhanced(f, app);
    } else {
        draw_legacy(f, app);
    }
}

/// Legacy drawing function - maintains Task 3.1-3.3 compatibility
pub fn draw_legacy(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Footer
        ])
        .split(f.area());

    // Draw header
    draw_header(f, app, chunks[0]);

    // Draw main content based on current view
    match app.get_current_view() {
        AppView::Welcome => draw_welcome(f, app, chunks[1]),
        AppView::Dashboard => draw_dashboard(f, app, chunks[1]),
        AppView::Jobs => draw_jobs(f, app, chunks[1]),
        AppView::JobDetail(_job_id) => draw_jobs(f, app, chunks[1]), // Use jobs view for now
        AppView::Profile => draw_profile(f, app, chunks[1]),
        AppView::Teams => draw_jobs(f, app, chunks[1]), // Use jobs view for now
        AppView::TeamDetail(_team_id) => draw_jobs(f, app, chunks[1]), // Use jobs view for now
        AppView::Settings => draw_settings(f, app, chunks[1]),
        AppView::Help => draw_welcome(f, app, chunks[1]), // Use welcome view for now
        AppView::Disputes => draw_jobs(f, app, chunks[1]), // Use jobs view for now
        AppView::Milestones => draw_jobs(f, app, chunks[1]), // Use jobs view for now
    }

    // Draw footer
    draw_footer(f, app, chunks[2]);
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            app.get_title(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" - "),
        Span::styled(app.get_status(), Style::default().fg(Color::Yellow)),
    ]))
    .block(Block::default().borders(Borders::ALL));

    f.render_widget(header, area);
}

fn draw_footer(f: &mut Frame, _app: &App, area: Rect) {
    let help_text = vec![
        Span::raw("Press "),
        Span::styled("h", Style::default().fg(Color::Yellow)),
        Span::raw(":Help "),
        Span::styled("r", Style::default().fg(Color::Yellow)),
        Span::raw(":Refresh "),
        Span::styled("c", Style::default().fg(Color::Yellow)),
        Span::raw(":Check "),
        Span::styled("q", Style::default().fg(Color::Red)),
        Span::raw(":Quit"),
    ];

    let footer =
        Paragraph::new(Line::from(help_text)).block(Block::default().borders(Borders::ALL));

    f.render_widget(footer, area);
}

/// Welcome screen for Task 3.1 - TUI foundation verification
fn draw_welcome(f: &mut Frame, app: &App, area: Rect) {
    let welcome_text = vec![
        Line::from(vec![Span::styled(
            "🎯 Trust Work Escrow v2 - TUI Foundation",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("✅ Terminal initialization: Complete"),
        Line::from("✅ Crossterm backend: Active"),
        Line::from("✅ Event handling: Ready"),
        Line::from("✅ Graceful shutdown: Enabled"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Network: ", Style::default().fg(Color::Yellow)),
            Span::raw(app.get_network_name()),
        ]),
        Line::from(vec![
            Span::styled("RPC URL: ", Style::default().fg(Color::Yellow)),
            Span::raw(app.get_rpc_url()),
        ]),
        Line::from(""),
        Line::from("🎮 Controls:"),
        Line::from("  • h = Help"),
        Line::from("  • r = Refresh Connection"),
        Line::from("  • c = Check Connection"),
        Line::from("  • q = Quit"),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Task 3.1 Complete! ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("TUI foundation ready for Phase 3.2+ features"),
        ]),
    ];

    let paragraph = Paragraph::new(welcome_text).block(
        Block::default()
            .title(" Welcome - TUI Foundation ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue)),
    );

    f.render_widget(paragraph, area);
}

fn draw_dashboard(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left panel - Wallet info
    let wallet_address = app.get_wallet_address();
    let wallet_info = vec![
        Line::from(vec![
            Span::styled("Wallet: ", Style::default().fg(Color::Green)),
            Span::raw(&wallet_address),
        ]),
        Line::from(vec![
            Span::styled("Balance: ", Style::default().fg(Color::Green)),
            Span::raw("Loading..."), // Will be updated in real implementation
        ]),
        Line::from(vec![
            Span::styled("Network: ", Style::default().fg(Color::Green)),
            Span::raw(app.get_network_name()),
        ]),
    ];

    let wallet_widget = Paragraph::new(wallet_info)
        .block(Block::default().title("Wallet Info").borders(Borders::ALL));

    f.render_widget(wallet_widget, chunks[0]);

    // Right panel - Quick stats
    let stats = vec![
        "📋 Active Jobs: 0",
        "⏳ Pending Milestones: 0",
        "🏆 Completed Jobs: 0",
        "💰 Total Earnings: 0 SOL",
    ];

    let stats_items: Vec<ListItem> = stats.iter().map(|s| ListItem::new(*s)).collect();

    let stats_widget =
        List::new(stats_items).block(Block::default().title("Quick Stats").borders(Borders::ALL));

    f.render_widget(stats_widget, chunks[1]);
}

fn draw_jobs(f: &mut Frame, _app: &App, area: Rect) {
    let placeholder = Paragraph::new("📋 Jobs view will be implemented in Phase 3.2+\n\nThis screen will show:\n• Available jobs to apply for\n• Your active jobs\n• Job creation interface\n• Application management")
        .block(Block::default().title("Jobs").borders(Borders::ALL))
        .style(Style::default().fg(Color::Gray));

    f.render_widget(placeholder, area);
}

fn draw_profile(f: &mut Frame, _app: &App, area: Rect) {
    let placeholder = Paragraph::new("👤 Profile view will be implemented in Phase 3.2+\n\nThis screen will show:\n• User profile information\n• Reputation and ratings\n• Portfolio/work history\n• Skills and certifications")
        .block(Block::default().title("Profile").borders(Borders::ALL))
        .style(Style::default().fg(Color::Gray));

    f.render_widget(placeholder, area);
}

fn draw_settings(f: &mut Frame, app: &App, area: Rect) {
    let settings_text = format!(
        "⚙️ Settings\n\nNetwork: {}\nRPC URL: {}\nWallet Type: {:?}\n\nFull settings management will be implemented in Phase 3.2+",
        app.get_network_name(),
        app.get_rpc_url(),
        app.config().wallet.wallet_type
    );

    let settings_widget = Paragraph::new(settings_text)
        .block(Block::default().title("Settings").borders(Borders::ALL));

    f.render_widget(settings_widget, area);
}
