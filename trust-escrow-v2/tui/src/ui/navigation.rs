//! Navigation and Interaction System for Trust Work Escrow TUI
//!
//! This module provides comprehensive navigation and interaction capabilities including:
//! - Keyboard navigation with customizable key bindings
//! - Focus management across UI components
//! - Interactive forms and input handling
//! - Menu navigation and shortcuts
//! - Context-sensitive help system
//!
//! ## Architecture
//!
//! The navigation system is built around a centralized `NavigationManager` that:
//! - Tracks current focus state across panels and components
//! - Provides keyboard shortcuts for common actions
//! - Manages input modes (normal, insert, command)
//! - Handles view transitions and state persistence
//! - Integrates with the layout system for responsive navigation

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::collections::HashMap;

use crate::app::{
    events::{AppEvent, LifecycleEvent, NavigationEvent, SortCriteria, UIEvent, ViewTarget},
    state::{AppState, AppView, InputMode, UIFocus},
};

/// Key binding for navigation actions
#[derive(Debug, Clone, PartialEq)]
pub struct KeyBinding {
    /// The key code
    pub key: KeyCode,
    /// Required modifiers (Ctrl, Alt, Shift)
    pub modifiers: KeyModifiers,
    /// Description for help system
    pub description: &'static str,
}

impl KeyBinding {
    /// Create a simple key binding without modifiers
    pub fn new(key: KeyCode, description: &'static str) -> Self {
        Self {
            key,
            modifiers: KeyModifiers::NONE,
            description,
        }
    }

    /// Create a key binding with modifiers
    pub fn with_modifiers(
        key: KeyCode,
        modifiers: KeyModifiers,
        description: &'static str,
    ) -> Self {
        Self {
            key,
            modifiers,
            description,
        }
    }

    /// Check if this binding matches the given key event
    pub fn matches(&self, event: &KeyEvent) -> bool {
        event.code == self.key && event.modifiers == self.modifiers
    }
}

/// Navigation action that can be triggered by keyboard input
#[derive(Debug, Clone, PartialEq)]
pub enum NavigationAction {
    // Global navigation
    Quit,
    Help,
    Refresh,
    ToggleMode,

    // View navigation
    GoToWelcome,
    GoToDashboard,
    GoToJobs,
    GoToProfile,
    GoToTeams,
    GoToSettings,
    GoToDisputes,
    GoToMilestones,
    GoBack,

    // Focus management
    NextFocus,
    PrevFocus,
    FocusLeft,
    FocusRight,
    FocusUp,
    FocusDown,

    // List navigation
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,
    Home,
    End,

    // Input/Form actions
    Confirm,
    Cancel,
    Edit,
    Delete,

    // Job-specific actions
    CreateJob,
    ApplyToJob,
    SubmitWork,
    ApproveWork,
    RejectWork,

    // Team actions
    CreateTeam,
    InviteMember,

    // Dispute actions
    RaiseDispute,
    SubmitEvidence,

    // Sorting and filtering
    SortByDate,
    SortByAmount,
    SortByStatus,
    FilterActive,

    // Custom action with parameters
    Custom(String),
}

/// Navigation manager that handles keyboard input and focus management
pub struct NavigationManager {
    /// Key bindings for navigation actions
    key_bindings: HashMap<(KeyCode, KeyModifiers), NavigationAction>,

    /// Context-specific bindings based on current view
    context_bindings: HashMap<AppView, HashMap<(KeyCode, KeyModifiers), NavigationAction>>,

    /// Current input mode
    input_mode: InputMode,

    /// Command buffer for command mode
    command_buffer: String,

    /// Focus manager for handling UI component focus
    focus_manager: FocusManager,
}

impl NavigationManager {
    /// Create a new navigation manager with default key bindings
    pub fn new() -> Self {
        let mut manager = Self {
            key_bindings: HashMap::new(),
            context_bindings: HashMap::new(),
            input_mode: InputMode::Normal,
            command_buffer: String::new(),
            focus_manager: FocusManager::new(),
        };

        manager.setup_default_bindings();
        manager.setup_context_bindings();
        manager
    }

    /// Set up default global key bindings
    fn setup_default_bindings(&mut self) {
        let bindings = [
            // Global actions
            (
                (KeyCode::Char('q'), KeyModifiers::NONE),
                NavigationAction::Quit,
            ),
            ((KeyCode::Esc, KeyModifiers::NONE), NavigationAction::GoBack),
            ((KeyCode::F(1), KeyModifiers::NONE), NavigationAction::Help),
            (
                (KeyCode::F(5), KeyModifiers::NONE),
                NavigationAction::Refresh,
            ),
            (
                (KeyCode::F(9), KeyModifiers::NONE),
                NavigationAction::ToggleMode,
            ),
            // View navigation with number keys
            (
                (KeyCode::Char('1'), KeyModifiers::NONE),
                NavigationAction::GoToDashboard,
            ),
            (
                (KeyCode::Char('2'), KeyModifiers::NONE),
                NavigationAction::GoToJobs,
            ),
            (
                (KeyCode::Char('3'), KeyModifiers::NONE),
                NavigationAction::GoToProfile,
            ),
            (
                (KeyCode::Char('4'), KeyModifiers::NONE),
                NavigationAction::GoToTeams,
            ),
            (
                (KeyCode::Char('5'), KeyModifiers::NONE),
                NavigationAction::GoToSettings,
            ),
            // Alternative view navigation with letters
            (
                (KeyCode::Char('d'), KeyModifiers::NONE),
                NavigationAction::GoToDashboard,
            ),
            (
                (KeyCode::Char('j'), KeyModifiers::NONE),
                NavigationAction::GoToJobs,
            ),
            (
                (KeyCode::Char('p'), KeyModifiers::NONE),
                NavigationAction::GoToProfile,
            ),
            (
                (KeyCode::Char('t'), KeyModifiers::NONE),
                NavigationAction::GoToTeams,
            ),
            (
                (KeyCode::Char('s'), KeyModifiers::NONE),
                NavigationAction::GoToSettings,
            ),
            // Focus navigation
            (
                (KeyCode::Tab, KeyModifiers::NONE),
                NavigationAction::NextFocus,
            ),
            (
                (KeyCode::BackTab, KeyModifiers::NONE),
                NavigationAction::PrevFocus,
            ),
            (
                (KeyCode::Left, KeyModifiers::NONE),
                NavigationAction::FocusLeft,
            ),
            (
                (KeyCode::Right, KeyModifiers::NONE),
                NavigationAction::FocusRight,
            ),
            // List navigation - Arrow keys for moving up/down in lists
            ((KeyCode::Up, KeyModifiers::NONE), NavigationAction::MoveUp),
            (
                (KeyCode::Down, KeyModifiers::NONE),
                NavigationAction::MoveDown,
            ),
            // NOTE: 'j' and 'k' are intentionally NOT mapped here because they
            // conflict with view navigation (j=Jobs). Use arrow keys for list
            // navigation instead. Vim-style navigation available in Jobs view only.
            (
                (KeyCode::PageUp, KeyModifiers::NONE),
                NavigationAction::PageUp,
            ),
            (
                (KeyCode::PageDown, KeyModifiers::NONE),
                NavigationAction::PageDown,
            ),
            ((KeyCode::Home, KeyModifiers::NONE), NavigationAction::Home),
            ((KeyCode::End, KeyModifiers::NONE), NavigationAction::End),
            // Common actions
            (
                (KeyCode::Enter, KeyModifiers::NONE),
                NavigationAction::Confirm,
            ),
            (
                (KeyCode::Char(' '), KeyModifiers::NONE),
                NavigationAction::Confirm,
            ),
            (
                (KeyCode::Char('e'), KeyModifiers::NONE),
                NavigationAction::Edit,
            ),
            (
                (KeyCode::Char('x'), KeyModifiers::NONE),
                NavigationAction::Delete,
            ),
            // Refresh and utility
            (
                (KeyCode::Char('r'), KeyModifiers::NONE),
                NavigationAction::Refresh,
            ),
            (
                (KeyCode::Char('h'), KeyModifiers::NONE),
                NavigationAction::Help,
            ),
        ];

        for (key, action) in bindings {
            self.key_bindings.insert(key, action);
        }
    }

    /// Set up context-specific key bindings for different views
    fn setup_context_bindings(&mut self) {
        // Jobs view bindings
        let mut jobs_bindings = HashMap::new();
        jobs_bindings.insert(
            (KeyCode::Char('n'), KeyModifiers::NONE),
            NavigationAction::CreateJob,
        );
        jobs_bindings.insert(
            (KeyCode::Char('a'), KeyModifiers::NONE),
            NavigationAction::ApplyToJob,
        );
        jobs_bindings.insert(
            (KeyCode::Char('w'), KeyModifiers::NONE),
            NavigationAction::SubmitWork,
        );
        jobs_bindings.insert(
            (KeyCode::Char('v'), KeyModifiers::NONE),
            NavigationAction::ApproveWork,
        );
        jobs_bindings.insert(
            (KeyCode::Char('z'), KeyModifiers::NONE),
            NavigationAction::RejectWork,
        );
        jobs_bindings.insert(
            (KeyCode::Char('D'), KeyModifiers::NONE),
            NavigationAction::RaiseDispute,
        );

        // Sorting in jobs view
        jobs_bindings.insert(
            (KeyCode::Char('1'), KeyModifiers::CONTROL),
            NavigationAction::SortByDate,
        );
        jobs_bindings.insert(
            (KeyCode::Char('2'), KeyModifiers::CONTROL),
            NavigationAction::SortByAmount,
        );
        jobs_bindings.insert(
            (KeyCode::Char('3'), KeyModifiers::CONTROL),
            NavigationAction::SortByStatus,
        );

        self.context_bindings.insert(AppView::Jobs, jobs_bindings);

        // Teams view bindings
        let mut teams_bindings = HashMap::new();
        teams_bindings.insert(
            (KeyCode::Char('n'), KeyModifiers::NONE),
            NavigationAction::CreateTeam,
        );
        teams_bindings.insert(
            (KeyCode::Char('i'), KeyModifiers::NONE),
            NavigationAction::InviteMember,
        );

        self.context_bindings.insert(AppView::Teams, teams_bindings);

        // Disputes view bindings
        let mut disputes_bindings = HashMap::new();
        disputes_bindings.insert(
            (KeyCode::Char('e'), KeyModifiers::NONE),
            NavigationAction::SubmitEvidence,
        );
        disputes_bindings.insert(
            (KeyCode::Char('r'), KeyModifiers::NONE),
            NavigationAction::RaiseDispute,
        );

        self.context_bindings
            .insert(AppView::Disputes, disputes_bindings);
    }

    /// Handle keyboard input and return navigation events
    pub fn handle_key_event(
        &mut self,
        key_event: KeyEvent,
        current_view: &AppView,
    ) -> Vec<AppEvent> {
        // Only handle press events
        if key_event.kind != KeyEventKind::Press {
            return vec![];
        }

        let mut events = vec![];
        let key_combo = (key_event.code, key_event.modifiers);

        match self.input_mode {
            InputMode::Normal => {
                // Try context-specific bindings first
                if let Some(context_bindings) = self.context_bindings.get(current_view) {
                    if let Some(action) = context_bindings.get(&key_combo) {
                        events.extend(self.action_to_events(action.clone()));
                        return events;
                    }
                }

                // Fall back to global bindings
                if let Some(action) = self.key_bindings.get(&key_combo) {
                    events.extend(self.action_to_events(action.clone()));
                }
            }

            InputMode::Insert => {
                // In insert mode, most keys are for text input
                match key_event.code {
                    KeyCode::Esc => {
                        self.input_mode = InputMode::Normal;
                        events.push(AppEvent::UI(UIEvent::ClearStatus));
                    }
                    KeyCode::Enter => {
                        events.push(AppEvent::Navigation(NavigationEvent::Submit));
                        self.input_mode = InputMode::Normal;
                    }
                    // Let other keys pass through for text input
                    _ => {}
                }
            }

            InputMode::Command => match key_event.code {
                KeyCode::Esc => {
                    self.command_buffer.clear();
                    self.input_mode = InputMode::Normal;
                    events.push(AppEvent::UI(UIEvent::ClearStatus));
                }
                KeyCode::Enter => {
                    let command = self.command_buffer.clone();
                    self.command_buffer.clear();
                    self.input_mode = InputMode::Normal;
                    events.push(AppEvent::Navigation(NavigationEvent::Command(command)));
                }
                KeyCode::Char(c) => {
                    self.command_buffer.push(c);
                }
                KeyCode::Backspace => {
                    self.command_buffer.pop();
                }
                _ => {}
            },
        }

        events
    }

    /// Convert navigation action to app events
    fn action_to_events(&self, action: NavigationAction) -> Vec<AppEvent> {
        let mut events = vec![];

        match action {
            NavigationAction::Quit => {
                events.push(AppEvent::Lifecycle(LifecycleEvent::Quit));
            }

            NavigationAction::Help => {
                events.push(AppEvent::Navigation(NavigationEvent::View(
                    ViewTarget::Help,
                )));
            }

            NavigationAction::Refresh => {
                events.push(AppEvent::UI(UIEvent::Refresh));
            }

            NavigationAction::ToggleMode => {
                events.push(AppEvent::UI(UIEvent::Toggle));
            }

            // View navigation
            NavigationAction::GoToWelcome => {
                events.push(AppEvent::Navigation(NavigationEvent::View(
                    ViewTarget::Welcome,
                )));
            }
            NavigationAction::GoToDashboard => {
                events.push(AppEvent::Navigation(NavigationEvent::View(
                    ViewTarget::Dashboard,
                )));
            }
            NavigationAction::GoToJobs => {
                events.push(AppEvent::Navigation(NavigationEvent::View(
                    ViewTarget::Jobs,
                )));
            }
            NavigationAction::GoToProfile => {
                events.push(AppEvent::Navigation(NavigationEvent::View(
                    ViewTarget::Profile,
                )));
            }
            NavigationAction::GoToTeams => {
                events.push(AppEvent::Navigation(NavigationEvent::View(
                    ViewTarget::Teams,
                )));
            }
            NavigationAction::GoToSettings => {
                events.push(AppEvent::Navigation(NavigationEvent::View(
                    ViewTarget::Settings,
                )));
            }
            NavigationAction::GoToDisputes => {
                events.push(AppEvent::Navigation(NavigationEvent::View(
                    ViewTarget::Disputes,
                )));
            }
            NavigationAction::GoToMilestones => {
                events.push(AppEvent::Navigation(NavigationEvent::View(
                    ViewTarget::Milestones,
                )));
            }
            NavigationAction::GoBack => {
                events.push(AppEvent::Navigation(NavigationEvent::Back));
            }

            // Focus management
            NavigationAction::NextFocus => {
                events.push(AppEvent::UI(UIEvent::FocusNext));
            }
            NavigationAction::PrevFocus => {
                events.push(AppEvent::UI(UIEvent::FocusPrevious));
            }
            NavigationAction::FocusLeft | NavigationAction::FocusUp => {
                events.push(AppEvent::UI(UIEvent::FocusPrevious));
            }
            NavigationAction::FocusRight | NavigationAction::FocusDown => {
                events.push(AppEvent::UI(UIEvent::FocusNext));
            }

            // List navigation - use Navigation events that actually update state
            NavigationAction::MoveUp => {
                events.push(AppEvent::Navigation(NavigationEvent::Up));
            }
            NavigationAction::MoveDown => {
                events.push(AppEvent::Navigation(NavigationEvent::Down));
            }
            NavigationAction::PageUp => {
                events.push(AppEvent::Navigation(NavigationEvent::PageUp));
            }
            NavigationAction::PageDown => {
                events.push(AppEvent::Navigation(NavigationEvent::PageDown));
            }
            NavigationAction::Home => {
                events.push(AppEvent::Navigation(NavigationEvent::Home));
            }
            NavigationAction::End => {
                events.push(AppEvent::Navigation(NavigationEvent::End));
            }

            // Common actions - Confirm triggers Select to handle item selection
            NavigationAction::Confirm => {
                events.push(AppEvent::Navigation(NavigationEvent::Select));
            }
            NavigationAction::Cancel => {
                events.push(AppEvent::Navigation(NavigationEvent::Back));
            }
            NavigationAction::Edit => {
                events.push(AppEvent::UI(UIEvent::Edit));
            }
            NavigationAction::Delete => {
                events.push(AppEvent::UI(UIEvent::Delete));
            }

            // Job actions
            NavigationAction::CreateJob => {
                events.push(AppEvent::UI(UIEvent::ShowForm("create_job".to_string())));
            }
            NavigationAction::ApplyToJob => {
                events.push(AppEvent::UI(UIEvent::ShowForm("apply_job".to_string())));
            }
            NavigationAction::SubmitWork => {
                events.push(AppEvent::UI(UIEvent::ShowForm("submit_work".to_string())));
            }
            NavigationAction::ApproveWork => {
                events.push(AppEvent::UI(UIEvent::Confirm("approve_work".to_string())));
            }
            NavigationAction::RejectWork => {
                events.push(AppEvent::UI(UIEvent::ShowForm("reject_work".to_string())));
            }

            // Team actions
            NavigationAction::CreateTeam => {
                events.push(AppEvent::UI(UIEvent::ShowForm("create_team".to_string())));
            }
            NavigationAction::InviteMember => {
                events.push(AppEvent::UI(UIEvent::ShowForm("invite_member".to_string())));
            }

            // Dispute actions
            NavigationAction::RaiseDispute => {
                events.push(AppEvent::UI(UIEvent::ShowForm("raise_dispute".to_string())));
            }
            NavigationAction::SubmitEvidence => {
                events.push(AppEvent::UI(UIEvent::ShowForm(
                    "submit_evidence".to_string(),
                )));
            }

            // Sorting
            NavigationAction::SortByDate => {
                events.push(AppEvent::UI(UIEvent::Sort(SortCriteria::Date)));
            }
            NavigationAction::SortByAmount => {
                events.push(AppEvent::UI(UIEvent::Sort(SortCriteria::Amount)));
            }
            NavigationAction::SortByStatus => {
                events.push(AppEvent::UI(UIEvent::Sort(SortCriteria::Status)));
            }
            NavigationAction::FilterActive => {
                events.push(AppEvent::UI(UIEvent::Filter("active".to_string())));
            }

            NavigationAction::Custom(action) => {
                events.push(AppEvent::UI(UIEvent::Custom(action)));
            }
        }

        events
    }

    /// Get current input mode
    pub fn get_input_mode(&self) -> InputMode {
        self.input_mode
    }

    /// Set input mode
    pub fn set_input_mode(&mut self, mode: InputMode) {
        self.input_mode = mode;
        if mode == InputMode::Command {
            self.command_buffer.clear();
        }
    }

    /// Get current command buffer (for command mode)
    pub fn get_command_buffer(&self) -> &str {
        &self.command_buffer
    }

    /// Get available key bindings for the current view
    pub fn get_key_bindings(&self, view: &AppView) -> Vec<(KeyBinding, NavigationAction)> {
        let mut bindings = vec![];

        // Add global bindings
        for ((key, modifiers), action) in &self.key_bindings {
            let description = self.get_action_description(action);
            bindings.push((
                KeyBinding::with_modifiers(*key, *modifiers, description),
                action.clone(),
            ));
        }

        // Add context-specific bindings
        if let Some(context_bindings) = self.context_bindings.get(view) {
            for ((key, modifiers), action) in context_bindings {
                let description = self.get_action_description(action);
                bindings.push((
                    KeyBinding::with_modifiers(*key, *modifiers, description),
                    action.clone(),
                ));
            }
        }

        bindings
    }

    /// Get description for an action
    fn get_action_description(&self, action: &NavigationAction) -> &'static str {
        match action {
            NavigationAction::Quit => "Quit application",
            NavigationAction::Help => "Show help",
            NavigationAction::Refresh => "Refresh data",
            NavigationAction::ToggleMode => "Toggle UI mode",
            NavigationAction::GoToWelcome => "Go to welcome",
            NavigationAction::GoToDashboard => "Go to dashboard",
            NavigationAction::GoToJobs => "Go to jobs",
            NavigationAction::GoToProfile => "Go to profile",
            NavigationAction::GoToTeams => "Go to teams",
            NavigationAction::GoToSettings => "Go to settings",
            NavigationAction::GoToDisputes => "Go to disputes",
            NavigationAction::GoToMilestones => "Go to milestones",
            NavigationAction::GoBack => "Go back",
            NavigationAction::NextFocus => "Next focus",
            NavigationAction::PrevFocus => "Previous focus",
            NavigationAction::FocusLeft => "Focus left",
            NavigationAction::FocusRight => "Focus right",
            NavigationAction::FocusUp => "Focus up",
            NavigationAction::FocusDown => "Focus down",
            NavigationAction::MoveUp => "Move up",
            NavigationAction::MoveDown => "Move down",
            NavigationAction::PageUp => "Page up",
            NavigationAction::PageDown => "Page down",
            NavigationAction::Home => "Go to top",
            NavigationAction::End => "Go to bottom",
            NavigationAction::Confirm => "Confirm/Select",
            NavigationAction::Cancel => "Cancel",
            NavigationAction::Edit => "Edit item",
            NavigationAction::Delete => "Delete item",
            NavigationAction::CreateJob => "Create new job",
            NavigationAction::ApplyToJob => "Apply to job",
            NavigationAction::SubmitWork => "Submit work",
            NavigationAction::ApproveWork => "Approve work",
            NavigationAction::RejectWork => "Reject work",
            NavigationAction::CreateTeam => "Create team",
            NavigationAction::InviteMember => "Invite member",
            NavigationAction::RaiseDispute => "Raise dispute",
            NavigationAction::SubmitEvidence => "Submit evidence",
            NavigationAction::SortByDate => "Sort by date",
            NavigationAction::SortByAmount => "Sort by amount",
            NavigationAction::SortByStatus => "Sort by status",
            NavigationAction::FilterActive => "Filter active",
            NavigationAction::Custom(_) => "Custom action",
        }
    }

    // ============================================
    // Focus management methods (delegate to FocusManager)
    // ============================================

    /// Move focus to the next component and update app state
    pub fn next_focus(&mut self, state: &mut AppState) {
        let focus = self.focus_manager.focus_next();
        state.ui_state.focus = focus;
    }

    /// Move focus to the previous component and update app state
    pub fn previous_focus(&mut self, state: &mut AppState) {
        let focus = self.focus_manager.focus_previous();
        state.ui_state.focus = focus;
    }

    /// Get the current focus
    pub fn current_focus(&self) -> UIFocus {
        self.focus_manager.get_current_focus()
    }

    /// Set focus to a specific component and update app state
    pub fn set_focus(&mut self, focus: UIFocus, state: &mut AppState) {
        if self.focus_manager.set_focus(focus) {
            state.ui_state.focus = focus;
        }
    }

    /// Set the available focusable components for the current layout
    pub fn set_focusable_components(&mut self, components: Vec<UIFocus>) {
        self.focus_manager.set_focusable_components(components);
    }
}

impl Default for NavigationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Focus manager for handling UI component focus
pub struct FocusManager {
    /// Available focusable components
    focusable_components: Vec<UIFocus>,
    /// Current focus index
    current_index: usize,
}

impl FocusManager {
    /// Create a new focus manager
    pub fn new() -> Self {
        Self {
            focusable_components: vec![UIFocus::MainContent],
            current_index: 0,
        }
    }

    /// Set the available focusable components for the current layout
    pub fn set_focusable_components(&mut self, components: Vec<UIFocus>) {
        self.focusable_components = components;
        // Reset to first component if current index is out of bounds
        if self.current_index >= self.focusable_components.len() {
            self.current_index = 0;
        }
    }

    /// Move focus to the next component
    pub fn focus_next(&mut self) -> UIFocus {
        if !self.focusable_components.is_empty() {
            self.current_index = (self.current_index + 1) % self.focusable_components.len();
        }
        self.get_current_focus()
    }

    /// Move focus to the previous component
    pub fn focus_previous(&mut self) -> UIFocus {
        if !self.focusable_components.is_empty() {
            self.current_index = if self.current_index > 0 {
                self.current_index - 1
            } else {
                self.focusable_components.len() - 1
            };
        }
        self.get_current_focus()
    }

    /// Set focus to a specific component
    pub fn set_focus(&mut self, focus: UIFocus) -> bool {
        if let Some(index) = self.focusable_components.iter().position(|&f| f == focus) {
            self.current_index = index;
            true
        } else {
            false
        }
    }

    /// Get the current focus
    pub fn get_current_focus(&self) -> UIFocus {
        self.focusable_components
            .get(self.current_index)
            .copied()
            .unwrap_or(UIFocus::MainContent)
    }

    /// Check if a specific component is currently focused
    pub fn is_focused(&self, focus: UIFocus) -> bool {
        self.get_current_focus() == focus
    }
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Help system for displaying available key bindings
pub struct HelpSystem {
    /// Current view for context-sensitive help
    current_view: AppView,
}

impl HelpSystem {
    /// Create a new help system
    pub fn new() -> Self {
        Self {
            current_view: AppView::Welcome,
        }
    }

    /// Set the current view for context-sensitive help
    pub fn set_current_view(&mut self, view: AppView) {
        self.current_view = view;
    }

    /// Get help text for the current view
    pub fn get_help_text(&self, nav_manager: &NavigationManager) -> Vec<String> {
        let mut help_lines = vec![];

        help_lines.push("Trust Work Escrow TUI - Keyboard Shortcuts".to_string());
        help_lines.push("".to_string());

        // Global shortcuts
        help_lines.push("Global Navigation:".to_string());
        help_lines.push("  q - Quit application".to_string());
        help_lines.push("  h/F1 - Show this help".to_string());
        help_lines.push("  r/F5 - Refresh data".to_string());
        help_lines.push("  F9 - Toggle layout mode".to_string());
        help_lines.push("  ESC - Go back/Cancel".to_string());
        help_lines.push("".to_string());

        help_lines.push("View Navigation:".to_string());
        help_lines.push("  1/d - Dashboard".to_string());
        help_lines.push("  2/j - Jobs".to_string());
        help_lines.push("  3/p - Profile".to_string());
        help_lines.push("  4/t - Teams".to_string());
        help_lines.push("  5/s - Settings".to_string());
        help_lines.push("".to_string());

        help_lines.push("Navigation:".to_string());
        help_lines.push("  ↑/k - Move up".to_string());
        help_lines.push("  ↓/j - Move down".to_string());
        help_lines.push("  ←/→ - Move left/right".to_string());
        help_lines.push("  Tab - Next focus".to_string());
        help_lines.push("  Shift+Tab - Previous focus".to_string());
        help_lines.push("  Enter/Space - Confirm/Select".to_string());
        help_lines.push("  e - Edit item".to_string());
        help_lines.push("  x - Delete item".to_string());
        help_lines.push("".to_string());

        // Context-specific help
        match self.current_view {
            AppView::Jobs => {
                help_lines.push("Jobs View:".to_string());
                help_lines.push("  n - Create new job".to_string());
                help_lines.push("  a - Apply to selected job".to_string());
                help_lines.push("  w - Submit work".to_string());
                help_lines.push("  v - Approve work".to_string());
                help_lines.push("  z - Reject work".to_string());
                help_lines.push("  D - Raise dispute".to_string());
                help_lines.push("  Ctrl+1 - Sort by date".to_string());
                help_lines.push("  Ctrl+2 - Sort by amount".to_string());
                help_lines.push("  Ctrl+3 - Sort by status".to_string());
            }

            AppView::Teams => {
                help_lines.push("Teams View:".to_string());
                help_lines.push("  n - Create new team".to_string());
                help_lines.push("  i - Invite member".to_string());
            }

            AppView::Disputes => {
                help_lines.push("Disputes View:".to_string());
                help_lines.push("  e - Submit evidence".to_string());
                help_lines.push("  r - Raise new dispute".to_string());
            }

            _ => {}
        }

        help_lines.push("".to_string());
        help_lines.push("Press ESC to close help".to_string());

        help_lines
    }

    /// Get quick reference for footer display
    pub fn get_quick_reference(&self) -> String {
        match self.current_view {
            AppView::Jobs => "n:New j/k:Navigate a:Apply v:Approve q:Quit h:Help".to_string(),
            AppView::Teams => "n:New Team i:Invite j/k:Navigate q:Quit h:Help".to_string(),
            AppView::Disputes => "e:Evidence r:Raise j/k:Navigate q:Quit h:Help".to_string(),
            _ => "j/k:Navigate Tab:Focus 1-5:Views q:Quit h:Help".to_string(),
        }
    }
}

impl Default for HelpSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Interactive form manager for handling user input
pub struct FormManager {
    /// Current form fields
    fields: HashMap<String, String>,
    /// Current field being edited
    current_field: Option<String>,
    /// Field order for tab navigation
    field_order: Vec<String>,
    /// Current field index
    field_index: usize,
}

impl FormManager {
    /// Create a new form manager
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
            current_field: None,
            field_order: vec![],
            field_index: 0,
        }
    }

    /// Set up a form with specified fields
    pub fn setup_form(&mut self, fields: Vec<String>) {
        self.fields.clear();
        self.field_order = fields.clone();

        for field in fields {
            self.fields.insert(field, String::new());
        }

        self.field_index = 0;
        self.current_field = self.field_order.get(0).cloned();
    }

    /// Move to next field
    pub fn next_field(&mut self) {
        if !self.field_order.is_empty() {
            self.field_index = (self.field_index + 1) % self.field_order.len();
            self.current_field = self.field_order.get(self.field_index).cloned();
        }
    }

    /// Move to previous field
    pub fn previous_field(&mut self) {
        if !self.field_order.is_empty() {
            self.field_index = if self.field_index > 0 {
                self.field_index - 1
            } else {
                self.field_order.len() - 1
            };
            self.current_field = self.field_order.get(self.field_index).cloned();
        }
    }

    /// Set field value
    pub fn set_field_value(&mut self, field: &str, value: String) {
        self.fields.insert(field.to_string(), value);
    }

    /// Get field value
    pub fn get_field_value(&self, field: &str) -> Option<&String> {
        self.fields.get(field)
    }

    /// Get current field
    pub fn get_current_field(&self) -> Option<&String> {
        self.current_field.as_ref()
    }

    /// Check if field is current
    pub fn is_current_field(&self, field: &str) -> bool {
        self.current_field.as_ref().map_or(false, |f| f == field)
    }

    /// Get all field values
    pub fn get_all_values(&self) -> &HashMap<String, String> {
        &self.fields
    }

    /// Clear form
    pub fn clear_form(&mut self) {
        self.fields.clear();
        self.field_order.clear();
        self.current_field = None;
        self.field_index = 0;
    }
}

impl Default for FormManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Menu system for handling menu navigation
pub struct MenuManager {
    /// Available menu items
    items: Vec<MenuItem>,
    /// Currently selected index
    selected_index: usize,
}

/// Menu item with label and action
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub label: String,
    pub action: NavigationAction,
    pub enabled: bool,
    pub description: Option<String>,
}

impl MenuItem {
    /// Create a new menu item
    pub fn new(label: impl Into<String>, action: NavigationAction) -> Self {
        Self {
            label: label.into(),
            action,
            enabled: true,
            description: None,
        }
    }

    /// Set description for the menu item
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set enabled state
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl MenuManager {
    /// Create a new menu manager
    pub fn new() -> Self {
        Self {
            items: vec![],
            selected_index: 0,
        }
    }

    /// Set menu items
    pub fn set_items(&mut self, items: Vec<MenuItem>) {
        self.items = items;
        self.selected_index = 0;
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        if !self.items.is_empty() {
            self.selected_index = if self.selected_index > 0 {
                self.selected_index - 1
            } else {
                self.items.len() - 1
            };
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        if !self.items.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.items.len();
        }
    }

    /// Get currently selected item
    pub fn get_selected_item(&self) -> Option<&MenuItem> {
        self.items.get(self.selected_index)
    }

    /// Get selected index
    pub fn get_selected_index(&self) -> usize {
        self.selected_index
    }

    /// Get all items
    pub fn get_items(&self) -> &[MenuItem] {
        &self.items
    }

    /// Set selected index
    pub fn set_selected_index(&mut self, index: usize) {
        if index < self.items.len() {
            self.selected_index = index;
        }
    }
}

impl Default for MenuManager {
    fn default() -> Self {
        Self::new()
    }
}
