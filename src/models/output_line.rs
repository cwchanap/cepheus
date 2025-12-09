use serde::{Deserialize, Serialize};

/// Represents a single line in the terminal history buffer.
/// Mirrors the backend `OutputLine` type for IPC serialization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "data")]
pub enum OutputLine {
    /// User-entered command
    Command {
        text: String,
        timestamp: u64, // Unix timestamp milliseconds
    },
    /// Standard output from command
    Stdout { text: String, timestamp: u64 },
    /// Standard error from command
    Stderr { text: String, timestamp: u64 },
    /// System notification (e.g., "Shell restarted", "Output truncated...")
    Notification {
        message: String,
        level: NotificationLevel,
        timestamp: u64,
    },
}

impl OutputLine {
    /// Get the timestamp of this output line
    pub const fn timestamp(&self) -> u64 {
        match self {
            Self::Command { timestamp, .. }
            | Self::Stdout { timestamp, .. }
            | Self::Stderr { timestamp, .. }
            | Self::Notification { timestamp, .. } => *timestamp,
        }
    }

    /// Get the text content of this output line
    #[allow(dead_code)]
    pub fn text(&self) -> &str {
        match self {
            Self::Command { text, .. } | Self::Stdout { text, .. } | Self::Stderr { text, .. } => {
                text
            }
            Self::Notification { message, .. } => message,
        }
    }

    /// Get the CSS class for styling this line type
    pub const fn css_class(&self) -> &'static str {
        match self {
            Self::Command { .. } => "line-command",
            Self::Stdout { .. } => "line-stdout",
            Self::Stderr { .. } => "line-stderr",
            Self::Notification { .. } => "line-notification",
        }
    }

    /// Generate a stable unique key for this output line
    /// Combines timestamp with text content to ensure uniqueness even when
    /// multiple lines share the same timestamp (millisecond resolution)
    pub fn unique_key(&self) -> String {
        let timestamp = self.timestamp();

        match self {
            Self::Command { text, .. } => format!("cmd_{timestamp}_{text}"),
            Self::Stdout { text, .. } => format!("out_{timestamp}_{text}"),
            Self::Stderr { text, .. } => format!("err_{timestamp}_{text}"),
            Self::Notification { message, .. } => format!("not_{timestamp}_{message}"),
        }
    }
}

/// Notification severity level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
}
