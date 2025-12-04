use serde::{Deserialize, Serialize};

/// Represents a single line in the terminal history buffer.
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
    pub fn text(&self) -> &str {
        match self {
            Self::Command { text, .. } | Self::Stdout { text, .. } | Self::Stderr { text, .. } => {
                text
            }
            Self::Notification { message, .. } => message,
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

#[cfg(test)]
mod tests {
    use super::*;

    // T005: Unit test for OutputLine serde serialization
    #[test]
    fn test_output_line_command_serialization() {
        let line = OutputLine::Command {
            text: "ls -la".to_string(),
            timestamp: 1701360000000,
        };

        let json = serde_json::to_string(&line).unwrap();
        assert!(json.contains("\"type\":\"Command\""));
        assert!(json.contains("\"text\":\"ls -la\""));
        assert!(json.contains("\"timestamp\":1701360000000"));

        // Deserialize back
        let deserialized: OutputLine = serde_json::from_str(&json).unwrap();
        assert_eq!(line, deserialized);
    }

    #[test]
    fn test_output_line_stdout_serialization() {
        let line = OutputLine::Stdout {
            text: "hello world".to_string(),
            timestamp: 1701360000050,
        };

        let json = serde_json::to_string(&line).unwrap();
        assert!(json.contains("\"type\":\"Stdout\""));

        let deserialized: OutputLine = serde_json::from_str(&json).unwrap();
        assert_eq!(line, deserialized);
    }

    #[test]
    fn test_output_line_stderr_serialization() {
        let line = OutputLine::Stderr {
            text: "error: file not found".to_string(),
            timestamp: 1701360000100,
        };

        let json = serde_json::to_string(&line).unwrap();
        assert!(json.contains("\"type\":\"Stderr\""));

        let deserialized: OutputLine = serde_json::from_str(&json).unwrap();
        assert_eq!(line, deserialized);
    }

    #[test]
    fn test_output_line_notification_serialization() {
        let line = OutputLine::Notification {
            message: "Output truncated: line limit exceeded".to_string(),
            level: NotificationLevel::Warning,
            timestamp: 1701360000200,
        };

        let json = serde_json::to_string(&line).unwrap();
        assert!(json.contains("\"type\":\"Notification\""));
        assert!(json.contains("\"level\":\"Warning\""));

        let deserialized: OutputLine = serde_json::from_str(&json).unwrap();
        assert_eq!(line, deserialized);
    }

    #[test]
    fn test_output_line_timestamp_accessor() {
        let cmd = OutputLine::Command {
            text: "test".to_string(),
            timestamp: 12345,
        };
        assert_eq!(cmd.timestamp(), 12345);

        let stdout = OutputLine::Stdout {
            text: "test".to_string(),
            timestamp: 67890,
        };
        assert_eq!(stdout.timestamp(), 67890);
    }

    #[test]
    fn test_output_line_text_accessor() {
        let cmd = OutputLine::Command {
            text: "echo hello".to_string(),
            timestamp: 0,
        };
        assert_eq!(cmd.text(), "echo hello");

        let notification = OutputLine::Notification {
            message: "Shell restarted".to_string(),
            level: NotificationLevel::Info,
            timestamp: 0,
        };
        assert_eq!(notification.text(), "Shell restarted");
    }
}
