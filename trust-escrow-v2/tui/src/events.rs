//! Event handling for TUI

/// Application events
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Key press event
    Key(crossterm::event::KeyCode),

    /// Network update event
    NetworkUpdate,

    /// Timer/periodic update
    Tick,

    /// Application should quit
    Quit,
}

impl AppEvent {
    /// Check if this is a quit event
    pub fn is_quit(&self) -> bool {
        matches!(self, AppEvent::Quit)
    }
}
