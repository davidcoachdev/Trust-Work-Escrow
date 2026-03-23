//! UI rendering for TUI

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{App, AppView};

/// Main drawing function
pub fn draw(f: &mut Frame, app: &App) {
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
    match app.current_view {
        AppView::Dashboard => draw_dashboard(f, app, chunks[1]),
        AppView::Jobs => draw_jobs(f, app, chunks[1]),
        AppView::Profile => draw_profile(f, app, chunks[1]),
        AppView::Settings => draw_settings(f, app, chunks[1]),
    }

    // Draw footer
    draw_footer(f, app, chunks[2]);
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "Trust Work Escrow v2",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" - "),
        Span::styled(&app.status, Style::default().fg(Color::Yellow)),
    ]))
    .block(Block::default().borders(Borders::ALL));

    f.render_widget(header, area);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let help_text = vec![
        Span::raw("Press "),
        Span::styled("1", Style::default().fg(Color::Yellow)),
        Span::raw(":Dashboard "),
        Span::styled("2", Style::default().fg(Color::Yellow)),
        Span::raw(":Jobs "),
        Span::styled("3", Style::default().fg(Color::Yellow)),
        Span::raw(":Profile "),
        Span::styled("4", Style::default().fg(Color::Yellow)),
        Span::raw(":Settings "),
        Span::styled("r", Style::default().fg(Color::Yellow)),
        Span::raw(":Refresh "),
        Span::styled("q", Style::default().fg(Color::Red)),
        Span::raw(":Quit"),
    ];

    let footer =
        Paragraph::new(Line::from(help_text)).block(Block::default().borders(Borders::ALL));

    f.render_widget(footer, area);
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
            Span::raw(&app.client.config().network.cluster),
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

fn draw_jobs(f: &mut Frame, app: &App, area: Rect) {
    let placeholder = Paragraph::new("📋 Jobs view will be implemented in Phase 2\n\nThis screen will show:\n• Available jobs to apply for\n• Your active jobs\n• Job creation interface\n• Application management")
        .block(Block::default().title("Jobs").borders(Borders::ALL))
        .style(Style::default().fg(Color::Gray));

    f.render_widget(placeholder, area);
}

fn draw_profile(f: &mut Frame, app: &App, area: Rect) {
    let placeholder = Paragraph::new("👤 Profile view will be implemented in Phase 2\n\nThis screen will show:\n• User profile information\n• Reputation and ratings\n• Portfolio/work history\n• Skills and certifications")
        .block(Block::default().title("Profile").borders(Borders::ALL))
        .style(Style::default().fg(Color::Gray));

    f.render_widget(placeholder, area);
}

fn draw_settings(f: &mut Frame, app: &App, area: Rect) {
    let settings_text = format!(
        "⚙️ Settings\n\nNetwork: {}\nRPC URL: {}\nWallet Type: {:?}\n\nFull settings management will be implemented in Phase 2",
        app.client.config().network.cluster,
        app.client.config().network.rpc_url,
        app.client.config().wallet.wallet_type
    );

    let settings_widget = Paragraph::new(settings_text)
        .block(Block::default().title("Settings").borders(Borders::ALL));

    f.render_widget(settings_widget, area);
}
