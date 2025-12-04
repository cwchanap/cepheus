use leptos::prelude::*;

use crate::models::OutputLine;

/// Frontend-only reactive state (Leptos signals).
/// Shared state accessible to all components via `use_context()`.
#[derive(Clone, Copy)]
pub struct TerminalState {
    /// Current command being typed
    pub current_input: RwSignal<String>,
    /// Terminal history (synced from backend)
    pub history: RwSignal<Vec<OutputLine>>,
    /// Current working directory
    pub cwd: RwSignal<String>,
    /// Is a command currently running?
    pub is_busy: RwSignal<bool>,
    /// Active notification (if any)
    pub notification: RwSignal<Option<String>>,
}

impl TerminalState {
    /// Create a new terminal state with default values
    pub fn new() -> Self {
        Self {
            current_input: RwSignal::new(String::new()),
            history: RwSignal::new(Vec::new()),
            cwd: RwSignal::new(String::from("~")),
            is_busy: RwSignal::new(false),
            notification: RwSignal::new(None),
        }
    }

    /// Clear the current input
    pub fn clear_input(&self) {
        self.current_input.set(String::new());
    }

    /// Add a line to the history
    pub fn push_history(&self, line: OutputLine) {
        self.history.update(|h| h.push(line));
    }

    /// Set the history (replacing existing)
    pub fn set_history(&self, lines: Vec<OutputLine>) {
        self.history.set(lines);
    }

    /// Show a notification (auto-dismiss should be handled by component)
    pub fn show_notification(&self, message: impl Into<String>) {
        self.notification.set(Some(message.into()));
    }

    /// Clear the current notification
    pub fn clear_notification(&self) {
        self.notification.set(None);
    }
}

impl Default for TerminalState {
    fn default() -> Self {
        Self::new()
    }
}
