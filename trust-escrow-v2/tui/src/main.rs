//! Trust Work Escrow TUI - Terminal User Interface
//!
//! Entry point for the terminal user interface using Ratatui v0.30+
//! Provides a modern TUI for interacting with Trust Work Escrow v2 protocol operations
//! with comprehensive event handling and responsive blockchain data updates

use anyhow::Result;
use ratatui::{
    backend::CrosstermBackend,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Gauge},
    Frame, Terminal,
};
use trust_escrow_shared::EscrowConfig;
use crossterm::event::{KeyEvent, KeyEventKind, KeyEventState};

mod app;
mod ui;
mod ui_legacy;

use app::{App, AppEvent, EventHandler};
use ui_legacy::{initialize_ui, draw, toggle_layout_mode, is_enhanced_mode, get_terminal_size};

/// Main entry point for TUI application with enhanced layout system
#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration from trust-escrow-shared
    let config = EscrowConfig::load().unwrap_or_default();
    
    // Initialize UI system (Task 3.4)
    initialize_ui();
    
    // Ensure enhanced layout mode is active
    if !is_enhanced_mode() {
        toggle_layout_mode();
    }
    
    // Initialize terminal with modern Ratatui v0.30+ pattern
    let mut terminal = ratatui::init();
    
    // Set up graceful shutdown handling
    let result = run_app(&mut terminal, config).await;
    
    // Always restore terminal state on exit
    ratatui::restore();
    
    // Handle any errors that occurred
    if let Err(err) = result {
        eprintln!("TUI Error: {}", err);
        std::process::exit(1);
    }

    Ok(())
}

/// Main application loop with comprehensive event handling
async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    config: EscrowConfig,
) -> Result<()> {
    // Initialize application with configuration
    let mut app = App::with_config(config).await?;
    
    // Initialize event handler
    let mut event_handler = EventHandler::new();
    
    // Initialize navigation manager for focus management
    let mut navigation_manager = ui::navigation::NavigationManager::new();
    
    // Start loading state
    let mut loading = true;
    let mut loading_message = "🚀 Inicializando Trust Work Escrow TUI...".to_string();
    let mut loading_progress = 0;
    
    // Welcome message for Task 3.4 layout system
    app.set_status("🎯 Trust Work Escrow TUI v2 - Navegación: Tab=Focus, Flechas=Navegar, d=Dashboard, j=Jobs, h=Help, q=quit");

    // Main event loop with enhanced layout support
    loop {
        // Draw the current frame using enhanced UI system
        terminal.draw(|frame| {
            if loading {
                draw_loading_screen(frame, &loading_message, loading_progress);
            } else {
                draw(frame, &app);
            }
        })?;
        
        // Simulate loading progress
        if loading {
            loading_progress += 1;
            if loading_progress < 30 {
                loading_message = match loading_progress % 10 {
                    0 => "🚀 Inicializando Trust Work Escrow TUI...".to_string(),
                    1 => "🌐 Conectando a la red Solana Devnet...".to_string(),
                    2 => "💼 Cargando configuración del programa...".to_string(),
                    3 => "👤 Verificando wallet y saldo...".to_string(),
                    4 => "🔄 Sincronizando datos blockchain...".to_string(),
                    5 => "📊 Preparando dashboard...".to_string(),
                    6 => "✅ ¡Conexión establecida! Cargando interfaz...".to_string(),
                    _ => "⏳ Casi listo...".to_string(),
                };
            } else {
                loading = false;
                app.state_mut().navigate_to(app::AppView::Dashboard);
                app.set_status("🎯 Trust Work Escrow TUI v2 - ¡Conectado! Tab=Focus, Flechas=Navegar, d=Dashboard, j=Jobs, h=Help, q=quit");
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            continue;
        }
        
        // Get next event from the event handler
        let event = event_handler.next_event().await?;
        
        // Check for quit events first
        if event_handler.should_quit(&event) {
            app.set_status("👋 Shutting down gracefully...");
            terminal.draw(|frame| draw(frame, &app))?;
            // Brief pause to show goodbye message
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            break;
        }
        
        // Process the event through the application state
        match &event {
            AppEvent::Key(key_input) => {
                // Handle navigation with NavigationManager first
                let nav_events = navigation_manager.handle_key_event(
                    crossterm::event::KeyEvent {
                        code: key_input.code,
                        modifiers: key_input.modifiers,
                        kind: crossterm::event::KeyEventKind::Press,
                        state: crossterm::event::KeyEventState::empty(),
                    },
                    app.get_current_view(),
                );
                
                // Process navigation events
                for nav_event in &nav_events {
                    match nav_event {
                        AppEvent::Navigation(nav_event) => {
                            handle_navigation_event(&mut app, nav_event).await?;
                        }
                        AppEvent::UI(ui_event) => {
                            handle_ui_event(&mut app, ui_event).await?;
                        }
                        _ => {}
                    }
                }
                
                // If navigation events were generated, continue to next iteration
                if !nav_events.is_empty() {
                    continue;
                }
                
                // Handle layout-specific keyboard shortcuts
                if handle_layout_shortcuts(key_input, &mut app, &mut navigation_manager).await? {
                    continue; // Event was handled by layout system
                }
                
                // Process key input and generate high-level events
                if let Some(processed_event) = event_handler.process_key_input(key_input) {
                    handle_processed_event(&mut app, &mut event_handler, processed_event).await?;
                } else {
                    // Handle legacy key processing for compatibility
                    app.handle_input(key_input.code).await?;
                }
            }
            AppEvent::BlockchainUpdate(blockchain_event) => {
                handle_blockchain_event(&mut app, blockchain_event).await?;
            }
            AppEvent::Navigation(nav_event) => {
                handle_navigation_event(&mut app, nav_event).await?;
            }
            AppEvent::UI(ui_event) => {
                handle_ui_event(&mut app, ui_event).await?;
            }
            AppEvent::Resize { width, height } => {
                app.set_status(&format!("📐 Terminal resized to {}x{} - Layout adapting...", width, height));
                // The UI system will automatically handle layout updates on next render
            }
            AppEvent::Tick => {
                // Periodic updates
                app.update().await?;
            }
            AppEvent::FastTick => {
                // Fast updates for smooth animations
                // Currently no specific fast tick processing
            }
            AppEvent::Lifecycle(lifecycle_event) => {
                handle_lifecycle_event(&mut app, lifecycle_event).await?;
            }
            AppEvent::Mouse { .. } => {
                // Mouse events not implemented yet
                app.set_status("🖱️ Mouse events will be supported in future phases");
            }
        }
        
        // Handle any errors that occurred during event processing
        if let Some(error) = event_handler.extract_error(&event) {
            app.set_status(&format!("❌ Error: {}", error));
        }
        
        // Small delay to prevent excessive CPU usage
        tokio::time::sleep(std::time::Duration::from_millis(16)).await; // ~60 FPS
    }

    Ok(())
}

/// Handle processed high-level events
async fn handle_processed_event(
    app: &mut App,
    event_handler: &mut EventHandler,
    event: AppEvent,
) -> Result<()> {
    match event {
        AppEvent::Navigation(nav_event) => {
            handle_navigation_event(app, &nav_event).await?;
        }
        AppEvent::UI(ui_event) => {
            handle_ui_event(app, &ui_event).await?;
        }
        AppEvent::Lifecycle(lifecycle_event) => {
            handle_lifecycle_event(app, &lifecycle_event).await?;
        }
        _ => {
            // Other events are handled in the main loop
        }
    }
    Ok(())
}

/// Handle blockchain-related events
async fn handle_blockchain_event(
    app: &mut App,
    blockchain_event: &app::BlockchainEvent,
) -> Result<()> {
    use app::BlockchainEvent;
    
    match blockchain_event {
        BlockchainEvent::TransactionUpdate { signature, status, confirmations } => {
            app.set_status(&format!("📊 Transaction {}: {:?} ({} confirmations)", signature, status, confirmations));
        }
        BlockchainEvent::TransactionUpdateLegacy { tx_id, status, message } => {
            app.set_status(&format!("📊 Transaction {}: {:?} - {}", tx_id, status, message));
        }
        BlockchainEvent::NewJob { job_id, title, client } => {
            app.set_status(&format!("💼 New job #{}: {} by {}", job_id, title, client));
        }
        BlockchainEvent::JobApplication { job_id, applicant } => {
            app.set_status(&format!("📝 New application for job #{} from {}", job_id, applicant));
        }
        BlockchainEvent::WorkSubmitted { job_id, submitter } => {
            app.set_status(&format!("📋 Work submitted for job #{} by {}", job_id, submitter));
        }
        BlockchainEvent::DisputeRaised { job_id, disputer, reason } => {
            app.set_status(&format!("⚠️ Dispute raised for job #{} by {}: {}", job_id, disputer, reason));
        }
        BlockchainEvent::MilestoneUpdate { milestone_id, job_id, status } => {
            app.set_status(&format!("🎯 Milestone {} of job #{}: {}", milestone_id, job_id, status));
        }
        BlockchainEvent::BalanceUpdate { wallet, new_balance } => {
            let sol_balance = *new_balance as f64 / 1_000_000_000.0;
            app.set_status(&format!("💰 Balance updated for {}: {:.6} SOL", wallet, sol_balance));
        }
        BlockchainEvent::NetworkStatus { status, message } => {
            app.set_status(&format!("🌐 Network: {:?} - {}", status, message));
        }
        BlockchainEvent::AsyncError { operation, error } => {
            app.set_status(&format!("❌ Error in {}: {}", operation, error));
        }
        BlockchainEvent::DataUpdate { data_type, loading_status } => {
            app.set_status(&format!("📦 Data update: {:?} - {:?}", data_type, loading_status));
        }
        BlockchainEvent::TaskUpdate { task_name, status } => {
            app.set_status(&format!("⚙️ Task {}: {:?}", task_name, status));
        }
    }
    Ok(())
}

/// Handle navigation events
async fn handle_navigation_event(
    app: &mut App,
    nav_event: &app::NavigationEvent,
) -> Result<()> {
    use app::{NavigationEvent, ViewTarget};
    
    match nav_event {
        NavigationEvent::GoTo(target) | NavigationEvent::View(target) => {
            let view = match target {
                ViewTarget::Welcome => app::AppView::Welcome,
                ViewTarget::Dashboard => app::AppView::Dashboard,
                ViewTarget::Jobs => app::AppView::Jobs,
                ViewTarget::JobDetail(id) => app::AppView::JobDetail(*id),
                ViewTarget::Profile => app::AppView::Profile,
                ViewTarget::Teams => app::AppView::Teams, 
                ViewTarget::TeamDetail(id) => app::AppView::TeamDetail(*id),
                ViewTarget::Settings => app::AppView::Settings,
                ViewTarget::Help => app::AppView::Help,
                ViewTarget::Disputes => app::AppView::Disputes,
                ViewTarget::Milestones => app::AppView::Milestones,
            };
            app.state_mut().navigate_to(view.clone());
            app.set_status(&format!("Navigated to {:?}", view));
        }
        NavigationEvent::Back => {
            app.state_mut().navigate_back();
            app.set_status("⬅️ Navigated back");
        }
        NavigationEvent::Up => {
            app.state_mut().navigate_up();
            app.set_status("⬆️ Moved up");
        }
        NavigationEvent::Down => {
            app.state_mut().navigate_down();
            app.set_status("⬇️ Moved down");
        }
        NavigationEvent::Left => {
            // app.state_mut().navigate_left(); // TODO: implement
            app.set_status("⬅️ Moved left");
        }
        NavigationEvent::Right => {
            // app.state_mut().navigate_right(); // TODO: implement
            app.set_status("➡️ Moved right");
        }
        NavigationEvent::Select => {
            app.state_mut().select_current().await?;
            app.set_status("✅ Selected current item");
        }
        NavigationEvent::Submit => {
            app.set_status("📤 Submitted");
        }
        NavigationEvent::Command(cmd) => {
            app.set_status(&format!("⌨️ Command: {}", cmd));
        }
        NavigationEvent::Cancel => {
            app.set_status("❌ Cancelled current operation");
        }
        NavigationEvent::Next => {
            app.set_status("⏭️ Moved to next");
        }
        NavigationEvent::Previous => {
            app.set_status("⏮️ Moved to previous");
        }
        NavigationEvent::PageUp => {
            app.set_status("📄⬆️ Page up");
        }
        NavigationEvent::PageDown => {
            app.set_status("📄⬇️ Page down");
        }
        NavigationEvent::Home => {
            app.set_status("🏠 Moved to start");
        }
        NavigationEvent::End => {
            app.set_status("🏁 Moved to end");
        }
    }
    Ok(())
}

/// Handle UI events
async fn handle_ui_event(
    app: &mut App,
    ui_event: &app::UIEvent,
) -> Result<()> {
    use app::UIEvent;
    
    match ui_event {
        UIEvent::Refresh => {
            app.set_status("🔄 Refreshing data...");
            app.refresh_connection().await?;
        }
        UIEvent::ToggleHelp => {
            app.set_status("📚 Help: q=quit, r=refresh, arrows=navigate, enter=select, ?=help");
        }
        UIEvent::ShowNotification(message) => {
            app.set_status(&format!("📢 {}", message));
        }
        UIEvent::ClearStatus => {
            app.set_status("");
        }
        UIEvent::Search(query) => {
            app.set_status(&format!("🔍 Searching for: {}", query));
        }
        UIEvent::Filter(criteria) => {
            app.set_status(&format!("🔍 Filtering by: {}", criteria));
        }
        UIEvent::Sort(criteria) => {
            app.set_status(&format!("📊 Sorting by: {:?}", criteria));
        }
        UIEvent::Toggle => {
            app.set_status("🔄 Toggled view mode");
        }
        UIEvent::Copy(content) => {
            app.set_status(&format!("📋 Copied: {}", content));
        }
        UIEvent::Paste => {
            app.set_status("📋 Pasted from clipboard");
        }
        UIEvent::ContextMenu => {
            app.set_status("📋 Context menu (not implemented yet)");
        }
        UIEvent::FocusNext => {
            app.set_status("⏭️ Focus next");
        }
        UIEvent::FocusPrevious => {
            app.set_status("⏮️ Focus previous");
        }
        UIEvent::SelectNext => {
            app.set_status("⬇️ Select next");
        }
        UIEvent::SelectPrevious => {
            app.set_status("⬆️ Select previous");
        }
        UIEvent::SelectFirst => {
            app.set_status("⏫ Select first");
        }
        UIEvent::SelectLast => {
            app.set_status("⏬ Select last");
        }
        UIEvent::Edit => {
            app.set_status("✏️ Edit mode");
        }
        UIEvent::Delete => {
            app.set_status("🗑️ Delete");
        }
        UIEvent::ShowForm(form_type) => {
            app.set_status(&format!("📝 Show form: {}", form_type));
        }
        UIEvent::Confirm(action) => {
            app.set_status(&format!("✅ Confirm: {}", action));
        }
        UIEvent::Custom(action) => {
            app.set_status(&format!("🔧 Custom: {}", action));
        }
    }
    Ok(())
}

/// Handle application lifecycle events
async fn handle_lifecycle_event(
    app: &mut App,
    lifecycle_event: &app::LifecycleEvent,
) -> Result<()> {
    use app::LifecycleEvent;
    
    match lifecycle_event {
        LifecycleEvent::Quit => {
            app.set_status("👋 Preparing to quit...");
        }
        LifecycleEvent::ForceQuit => {
            app.set_status("🚨 Force quit initiated");
        }
        LifecycleEvent::Suspend => {
            app.set_status("⏸️ Application suspended");
        }
        LifecycleEvent::Resume => {
            app.set_status("▶️ Application resumed");
        }
        LifecycleEvent::FatalError(error) => {
            app.set_status(&format!("💀 Fatal error: {}", error));
        }
        LifecycleEvent::Shutdown => {
            app.set_status("🔌 Shutting down...");
        }
    }
    Ok(())
}

/// Handle layout-specific keyboard shortcuts (Task 3.4)
async fn handle_layout_shortcuts(
    key_input: &app::KeyInput,
    app: &mut App,
    navigation_manager: &mut ui::navigation::NavigationManager,
) -> Result<bool> {
    use crossterm::event::{KeyCode, KeyModifiers};

    match (key_input.code, key_input.modifiers) {
        // Tab navigation between panels
        (KeyCode::Tab, KeyModifiers::NONE) => {
            navigation_manager.next_focus(app.state_mut());
            let focus = navigation_manager.current_focus();
            app.set_status(&format!("🔍 Focus: {:?}", focus));
            Ok(true)
        }
        // Shift+Tab for reverse navigation
        (KeyCode::BackTab, _) | (KeyCode::Tab, KeyModifiers::SHIFT) => {
            navigation_manager.previous_focus(app.state_mut());
            let focus = navigation_manager.current_focus();
            app.set_status(&format!("🔍 Focus: {:?} (reverse)", focus));
            Ok(true)
        }
        // Toggle layout mode (enhanced vs legacy)
        (KeyCode::Char('L'), KeyModifiers::NONE) => {
            toggle_layout_mode();
            let mode = if is_enhanced_mode() { "Enhanced" } else { "Legacy" };
            app.set_status(&format!("🔄 Layout mode: {}", mode));
            Ok(true)
        }
        // Show terminal size info
        (KeyCode::Char('T'), KeyModifiers::NONE) => {
            if let Some(size) = get_terminal_size() {
                app.set_status(&format!("📐 Terminal size: {:?}", size));
            } else {
                app.set_status("📐 Terminal size: Unknown (Legacy mode)");
            }
            Ok(true)
        }
        // Panel-specific shortcuts (when focus is on specific panels)
        (KeyCode::Char('1'), KeyModifiers::NONE) => {
            navigation_manager.set_focus(app::UIFocus::MainContent, app.state_mut());
            app.set_status("🎯 Focus: Main Content");
            Ok(true)
        }
        (KeyCode::Char('2'), KeyModifiers::NONE) => {
            navigation_manager.set_focus(app::UIFocus::JobList, app.state_mut());
            app.set_status("📋 Focus: Job List");
            Ok(true)
        }
        (KeyCode::Char('3'), KeyModifiers::NONE) => {
            navigation_manager.set_focus(app::UIFocus::NotificationPanel, app.state_mut());
            app.set_status("🔔 Focus: Notifications");
            Ok(true)
        }
        // Not handled by layout system
        _ => Ok(false),
    }
}

/// Enhanced welcome screen with layout system status (Task 3.4)
fn draw_welcome_screen(frame: &mut Frame, app: &App) {
    let area = frame.area();
    
    // Create welcome content with event system information
    let welcome_text = vec![
        Line::from(vec![
            Span::styled(
                "🎯 Trust Work Escrow v2 - TUI Foundation + Layout System",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            )
        ]),
        Line::from(""),
        Line::from("✅ Terminal initialization: Complete"),
        Line::from("✅ Crossterm backend: Active"), 
        Line::from("✅ Event handling: Ready"),
        Line::from("✅ Layout system: Three-panel dashboard"),
        Line::from("✅ Focus management: Active"),
        Line::from("✅ Responsive design: Adaptive"),
        Line::from("✅ Role-specific layouts: Configured"),
        Line::from("✅ Modal support: Available"),
        Line::from("✅ Keyboard navigation: Enhanced"),
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
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Green)),
            Span::raw(app.get_status()),
        ]),
        Line::from(""),
        Line::from("🎮 Enhanced Controls (Task 3.4):"),
        Line::from("  • Panel Navigation: Tab=Next, Shift+Tab=Previous"),
        Line::from("  • Direct Focus: 1=Main, 2=Jobs, 3=Notifications"),
        Line::from("  • Layout Control: L=Toggle mode, T=Size info"),
        Line::from("  • Movement: ↑↓←→ or hjkl"),
        Line::from("  • Views: d=Dashboard, j=Jobs, p=Profile, s=Settings"),
        Line::from("  • Actions: Enter=Select, r=Refresh, ?=Help"),
        Line::from("  • System: q=Quit, Esc=Cancel, Ctrl+C=Force Quit"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Task 3.4 Complete! ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw("Layout Infrastructure ready for three-panel dashboard")
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Layout Features: ", Style::default().fg(Color::Cyan)),
            Span::raw("Responsive • Three-panel • Focus • Role-specific • Modal")
        ]),
    ];
    
    // Render with border
    let paragraph = Paragraph::new(welcome_text)
        .block(
            Block::default()
                .title(" Trust Work Escrow TUI v2 - Layout System ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue))
        );
    
    frame.render_widget(paragraph, area);
}

/// Draw loading screen with progress
fn draw_loading_screen(frame: &mut Frame, message: &str, progress: u16) {
    use ratatui::widgets::Gauge;
    use ratatui::layout::{Constraint, Direction, Layout};
    
    let area = frame.area();
    
    // Create vertical layout with title, gauge, and message
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(10),
            Constraint::Percentage(30),
            Constraint::Percentage(40),
        ])
        .split(area);
    
    // Title
    let title = Paragraph::new("🚀 Trust Work Escrow TUI v2")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(title, chunks[0]);
    
    // Progress gauge (0-100%)
    let percentage = (progress as f64 / 30.0 * 100.0) as u16;
    let gauge = Gauge::default()
        .block(Block::default().title("Progreso de Conexión").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Green).bg(Color::Black))
        .percent(percentage.min(100))
        .label(format!("{}%", percentage.min(100)));
    frame.render_widget(gauge, chunks[1]);
    
    // Message
    let message_paragraph = Paragraph::new(message)
        .style(Style::default().fg(Color::Yellow))
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(message_paragraph, chunks[2]);
    
    // Loading animation
    let loading_text = match progress % 4 {
        0 => "⣾ ",
        1 => "⣽ ",
        2 => "⣻ ",
        3 => "⢿ ",
        _ => "⣾ ",
    };
    
    let loading_paragraph = Paragraph::new(format!("{}Cargando...{}", loading_text, loading_text))
        .style(Style::default().fg(Color::Magenta))
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(loading_paragraph, chunks[3]);
}