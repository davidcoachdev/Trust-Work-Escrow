use crate::app::{App, MessageType, Screen};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

// ─── Main Render Dispatch ────────────────────────────────────────────────────

pub fn render(f: &mut Frame, app: &App) {
    let bg = app.theme.bg;
    // Fill background
    let full = f.area();
    f.render_widget(
        Block::default().style(Style::default().bg(bg)),
        full,
    );

    // Main layout: header + body + status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Min(1),   // body
            Constraint::Length(3), // status bar
        ])
        .split(full);

    render_header(f, app, chunks[0]);
    render_status_bar(f, app, chunks[2]);

    match &app.screen {
        Screen::WalletSelect | Screen::SettingsWallets => render_list_screen(f, app, chunks[1], "Select Wallet", "↑↓ Navigate  Enter: Select  d: Delete  q: Quit"),
        Screen::RoleSelect => render_list_screen(f, app, chunks[1], "Select Role", "↑↓ Navigate  Enter: Select  Esc: Back"),
        Screen::MainMenu => render_list_screen(f, app, chunks[1], &format!("Main Menu — {}", app.role.label()), "↑↓ Navigate  Enter: Select  q: Quit"),
        Screen::SettingsMenu => render_list_screen(f, app, chunks[1], "Settings", "↑↓ Navigate  Enter: Select  Esc: Back"),
        Screen::SettingsTheme => render_list_screen(f, app, chunks[1], "Select Theme", "↑↓ Navigate  Enter: Apply  Esc: Back"),
        Screen::Result => render_result(f, app, chunks[1]),
        // All form screens
        _ => render_form(f, app, chunks[1]),
    }

    // Overlay message if any
    if let Some((msg, mt)) = &app.message {
        render_message(f, app, msg, mt);
    }
}

// ─── Header ──────────────────────────────────────────────────────────────────

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let title = Line::from(vec![
        Span::styled(
            " Trust Work Escrow ",
            Style::default()
                .fg(t.title)
                .bg(t.bg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("TUI", Style::default().fg(t.accent).bg(t.bg)),
    ]);

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(t.border))
        .style(Style::default().bg(t.bg));

    let header = Paragraph::new(title).block(block);
    f.render_widget(header, area);
}

// ─── Status Bar ──────────────────────────────────────────────────────────────

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let wallet_name = app.active_wallet_name();
    let pubkey = if app.active_pubkey.len() > 16 {
        format!("{}…{}", &app.active_pubkey[..8], &app.active_pubkey[app.active_pubkey.len()-8..])
    } else {
        app.active_pubkey.clone()
    };

    let status = Line::from(vec![
        Span::styled(" 👛 ", Style::default().fg(t.accent)),
        Span::styled(&wallet_name, Style::default().fg(t.fg).add_modifier(Modifier::BOLD)),
        Span::styled(
            format!(" [{}]", t.name),
            Style::default().fg(t.muted),
        ),
        Span::styled(
            format!(" ({pubkey})"),
            Style::default().fg(t.muted),
        ),
        Span::styled(" │ ", Style::default().fg(t.border)),
        Span::styled(
            format!("🔗 {}", app.settings.rpc_url),
            Style::default().fg(t.muted),
        ),
        Span::styled(" │ ", Style::default().fg(t.border)),
        Span::styled(
            format!("🎨 {}", app.settings.theme),
            Style::default().fg(t.muted),
        ),
    ]);

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(t.border))
        .style(Style::default().bg(t.bg));

    let bar = Paragraph::new(status).block(block);
    f.render_widget(bar, area);
}

// ─── List Screen (menus, wallet select, etc.) ────────────────────────────────

fn render_list_screen(f: &mut Frame, app: &App, area: Rect, title: &str, help: &str) {
    let t = &app.theme;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    let items: Vec<ListItem> = app
        .menu_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.list_index {
                Style::default()
                    .fg(t.bg)
                    .bg(t.highlight)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(t.fg).bg(t.bg)
            };
            ListItem::new(Line::from(Span::styled(
                format!("  {}  ", item.label),
                style,
            )))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(t.border))
            .title(Span::styled(
                format!(" {title} "),
                Style::default().fg(t.title).add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(t.bg)),
    );
    f.render_widget(list, chunks[0]);

    let help_text = Paragraph::new(Line::from(Span::styled(
        format!(" {help}"),
        Style::default().fg(t.muted),
    )))
    .style(Style::default().bg(t.bg));
    f.render_widget(help_text, chunks[1]);
}

// ─── Form Screen ─────────────────────────────────────────────────────────────

fn render_form(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let title = match &app.screen {
        Screen::InitForm => "Initialize Config",
        Screen::CreateJobForm => "Create Job",
        Screen::DepositForm => "Deposit Funds",
        Screen::AcceptForm => "Accept Job",
        Screen::SubmitForm => "Submit Work",
        Screen::ApproveForm => "Approve Work",
        Screen::RejectForm => "Reject Work",
        Screen::RaiseDisputeForm => "Raise Dispute",
        Screen::ResolveDisputeForm => "Resolve Dispute",
        Screen::CancelForm => "Cancel Job",
        Screen::ShowForm => "Show Job",
        Screen::UpdateJobLookupForm => "Update Job — Buscar",
        Screen::UpdateJobEditForm => "Update Job — Editar",
        Screen::SettingsNetwork => "Network Settings",
        Screen::AddWalletForm => "Add Wallet",
        _ => "Form",
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    // Build form lines
    let mut lines = Vec::new();
    for (i, field) in app.form_fields.iter().enumerate() {
        let is_active = i == app.form_index;

        // Label
        let label_style = if is_active {
            Style::default()
                .fg(t.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.muted)
        };
        lines.push(Line::from(Span::styled(
            format!(
                "  {} {}",
                if is_active { "▸" } else { " " },
                field.label
            ),
            label_style,
        )));

        // Input field
        let display_value = if field.value.is_empty() {
            field.placeholder.clone()
        } else {
            field.value.clone()
        };
        let value_style = if field.value.is_empty() {
            Style::default().fg(t.muted)
        } else if is_active {
            Style::default().fg(t.fg).add_modifier(Modifier::UNDERLINED)
        } else {
            Style::default().fg(t.fg)
        };

        let cursor = if is_active { "█" } else { "" };
        lines.push(Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled(
                if is_active {
                    format!("┃ {display_value}{cursor}")
                } else {
                    format!("│ {display_value}")
                },
                value_style,
            ),
        ]));
        lines.push(Line::from(""));
    }

    // Submit hint
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Press Enter to submit",
        Style::default().fg(t.success),
    )));

    let form = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(t.border))
                .title(Span::styled(
                    format!(" {title} "),
                    Style::default().fg(t.title).add_modifier(Modifier::BOLD),
                ))
                .style(Style::default().bg(t.bg)),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(form, chunks[0]);

    let help = Paragraph::new(Line::from(Span::styled(
        " Tab/↑↓: Navigate fields  Enter: Submit  Esc: Back",
        Style::default().fg(t.muted),
    )))
    .style(Style::default().bg(t.bg));
    f.render_widget(help, chunks[1]);
}

// ─── Result Screen ───────────────────────────────────────────────────────────

fn render_result(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    let is_error = app.result_text.starts_with('❌');
    let text_color = if is_error { t.error } else { t.success };

    let result = Paragraph::new(app.result_text.as_str())
        .style(Style::default().fg(text_color).bg(t.bg))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(t.border))
                .title(Span::styled(
                    " Result ",
                    Style::default().fg(t.title).add_modifier(Modifier::BOLD),
                ))
                .style(Style::default().bg(t.bg)),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(result, chunks[0]);

    let help = Paragraph::new(Line::from(Span::styled(
        " Enter/Esc: Back to menu",
        Style::default().fg(t.muted),
    )))
    .style(Style::default().bg(t.bg));
    f.render_widget(help, chunks[1]);
}

// ─── Message Overlay ─────────────────────────────────────────────────────────

fn render_message(f: &mut Frame, app: &App, msg: &str, mt: &MessageType) {
    let t = &app.theme;
    let color = match mt {
        MessageType::Success => t.success,
        MessageType::Error => t.error,
        MessageType::Info => t.warning,
    };

    let area = f.area();
    let popup_height = 3;
    let popup_width = (msg.len() as u16 + 6).min(area.width.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + area.height.saturating_sub(popup_height + 4);

    let popup_area = Rect::new(x, y, popup_width, popup_height);
    f.render_widget(Clear, popup_area);

    let popup = Paragraph::new(Line::from(Span::styled(
        msg,
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    )))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color))
            .style(Style::default().bg(t.bg)),
    );
    f.render_widget(popup, popup_area);
}
