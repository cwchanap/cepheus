use serde::{Deserialize, Serialize};

/// Request to execute a shell command (frontend → backend IPC).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRequest {
    /// The shell command to execute
    pub command: String,
    /// Working directory (optional; defaults to current)
    pub cwd: Option<String>,
}

/// Response from shell command execution (backend → frontend IPC).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResponse {
    /// Command execution succeeded
    pub success: bool,
    /// Exit code (if available)
    pub exit_code: Option<i32>,
    /// Error message (if execution failed)
    pub error: Option<String>,
}

impl CommandResponse {
    /// Create a successful command response
    pub const fn success(exit_code: i32) -> Self {
        Self {
            success: exit_code == 0,
            exit_code: Some(exit_code),
            error: None,
        }
    }

    /// Create a failed command response
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            exit_code: None,
            error: Some(error.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // T006: Unit test for CommandRequest/CommandResponse serialization
    #[test]
    fn test_command_request_serialization() {
        let request = CommandRequest {
            command: "ls -la".to_string(),
            cwd: Some("/home/user".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"command\":\"ls -la\""));
        assert!(json.contains("\"cwd\":\"/home/user\""));

        let deserialized: CommandRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.command, "ls -la");
        assert_eq!(deserialized.cwd, Some("/home/user".to_string()));
    }

    #[test]
    fn test_command_request_without_cwd() {
        let request = CommandRequest {
            command: "pwd".to_string(),
            cwd: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: CommandRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.command, "pwd");
        assert!(deserialized.cwd.is_none());
    }

    #[test]
    fn test_command_response_success_serialization() {
        let response = CommandResponse::success(0);

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"exit_code\":0"));

        let deserialized: CommandResponse = serde_json::from_str(&json).unwrap();
        assert!(deserialized.success);
        assert_eq!(deserialized.exit_code, Some(0));
        assert!(deserialized.error.is_none());
    }

    #[test]
    fn test_command_response_failure_serialization() {
        let response = CommandResponse::failure("command not found");

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":false"));
        assert!(json.contains("\"error\":\"command not found\""));

        let deserialized: CommandResponse = serde_json::from_str(&json).unwrap();
        assert!(!deserialized.success);
        assert!(deserialized.error.is_some());
    }

    #[test]
    fn test_command_response_non_zero_exit_code() {
        let response = CommandResponse::success(1); // Non-zero exit code

        assert!(!response.success); // success field reflects exit code
        assert_eq!(response.exit_code, Some(1));
    }
}
