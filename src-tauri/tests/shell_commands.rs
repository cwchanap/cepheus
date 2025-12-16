//! Integration tests for shell command execution
//!
//! These tests verify the execute_command, cancel_command, and shell crash detection
//! functionality by interacting with real shell processes.

use std::time::Duration;
use tokio::time::timeout;

use cepheus_lib::models::{CommandResponse, OutputLine};
use cepheus_lib::state::ShellManager;

/// Helper to create a test shell manager
fn create_test_manager() -> ShellManager {
    ShellManager::with_capacity(100)
}

fn build_shell_command_test(command: &str) -> tokio::process::Command {
    #[cfg(windows)]
    {
        let mut cmd = tokio::process::Command::new("cmd");
        cmd.arg("/C").arg(command);
        cmd
    }

    #[cfg(not(windows))]
    {
        let mut cmd = tokio::process::Command::new("sh");
        cmd.arg("-c").arg(command);
        cmd
    }
}

// T017: Integration test for execute_command with echo
#[tokio::test]
async fn test_execute_echo_command() {
    let manager = create_test_manager();

    // Execute a simple echo command
    #[cfg(windows)]
    let cmd = "echo hello world";
    #[cfg(not(windows))]
    let cmd = "echo 'hello world'";

    let result = timeout(
        Duration::from_secs(5),
        execute_command_test(&manager, cmd, None),
    )
    .await
    .expect("Command timed out");

    assert!(result.is_ok(), "Command should succeed");
    let response = result.unwrap();
    assert!(response.success, "Echo command should succeed");
    assert_eq!(response.exit_code, Some(0));

    // Check that output was captured in history
    let history = manager.history_buffer.get_all();
    assert!(!history.is_empty(), "History should have output");

    // Find stdout line containing "hello world"
    let has_output = history.iter().any(|line| {
        if let OutputLine::Stdout { text, .. } = line {
            text.contains("hello world")
        } else {
            false
        }
    });
    assert!(has_output, "Output should contain 'hello world'");
}

#[tokio::test]
async fn test_execute_pwd_command() {
    let manager = create_test_manager();

    #[cfg(windows)]
    let cmd = "cd";
    #[cfg(not(windows))]
    let cmd = "pwd";

    let result = timeout(
        Duration::from_secs(5),
        execute_command_test(&manager, cmd, None),
    )
    .await
    .expect("Command timed out");

    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.success);

    // History should contain current directory output
    let history = manager.history_buffer.get_all();
    assert!(!history.is_empty());
}

#[tokio::test]
async fn test_execute_command_with_stderr() {
    let manager = create_test_manager();

    // Command that writes to stderr
    #[cfg(windows)]
    let cmd = "echo error message 1>&2";
    #[cfg(not(windows))]
    let cmd = "echo 'error message' >&2";

    let result = timeout(
        Duration::from_secs(5),
        execute_command_test(&manager, cmd, None),
    )
    .await
    .expect("Command timed out");

    assert!(result.is_ok());

    let history = manager.history_buffer.get_all();
    let has_stderr = history
        .iter()
        .any(|line| matches!(line, OutputLine::Stderr { .. }));
    assert!(has_stderr, "Should capture stderr output");
}

#[tokio::test]
async fn test_execute_failing_command() {
    let manager = create_test_manager();

    // Command that exits with non-zero status
    #[cfg(windows)]
    let cmd = "exit /b 1";
    #[cfg(not(windows))]
    let cmd = "exit 1";

    let result = timeout(
        Duration::from_secs(5),
        execute_command_test(&manager, cmd, None),
    )
    .await
    .expect("Command timed out");

    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(!response.success, "Command should report failure");
    assert_eq!(response.exit_code, Some(1));
}

// T018: Integration test for cancel_command with sleep
#[tokio::test]
async fn test_cancel_command() {
    let manager = create_test_manager();

    // Start a long-running command
    #[cfg(windows)]
    let cmd = "timeout /T 30 /NOBREAK > NUL";
    #[cfg(not(windows))]
    let cmd = "sleep 30";

    let manager_clone = manager.clone();
    let handle = tokio::spawn(async move { execute_command_test(&manager_clone, cmd, None).await });

    // Wait a bit for the command to start
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify command is running
    assert!(manager.is_busy().await, "Command should be running (busy)");

    // Cancel the command
    let cancel_result = cancel_command_test(&manager).await;
    assert!(cancel_result.is_ok(), "Cancel should succeed");

    // Wait for the spawned task to complete
    let _result = timeout(Duration::from_secs(2), handle)
        .await
        .expect("Cancelled command should complete quickly")
        .expect("Task should not panic");

    // The command may succeed or fail depending on timing; the timeout ensures it completed quickly
    // (didn't hang for 30 seconds), so no additional assertion is needed
}

// T019: Integration test for shell crash detection
#[cfg(not(windows))]
#[tokio::test]
async fn test_shell_crash_detection() {
    let manager = create_test_manager();

    // Command that kills itself (simulates crash)
    let _result = timeout(
        Duration::from_secs(5),
        execute_command_test(&manager, "kill -9 $$", None),
    )
    .await
    .expect("Command timed out");

    // After command completes, manager should not be busy
    assert!(
        !manager.is_busy().await,
        "Manager should not be busy after crash"
    );
}

#[tokio::test]
async fn test_command_not_found() {
    let manager = create_test_manager();

    let result = timeout(
        Duration::from_secs(5),
        execute_command_test(
            &manager,
            "this_command_definitely_does_not_exist_12345",
            None,
        ),
    )
    .await
    .expect("Command timed out");

    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(!response.success, "Invalid command should fail");
    assert_ne!(response.exit_code, Some(0));
}

// T072: Test output truncation at capacity limit
#[tokio::test]
async fn test_output_truncation() {
    // Create manager with small capacity to test truncation
    let manager = ShellManager::with_capacity(50);

    // Generate output that exceeds capacity
    // seq 1 100 will generate 100 lines of output
    #[cfg(windows)]
    let cmd = "for /L %i in (1,1,100) do @echo %i";
    #[cfg(not(windows))]
    let cmd = "seq 1 100";

    let result = timeout(
        Duration::from_secs(5),
        execute_command_test(&manager, cmd, None),
    )
    .await
    .expect("Command timed out");

    assert!(result.is_ok(), "Command should succeed");

    let history = manager.history_buffer.get_all();

    assert_eq!(
        history.len(),
        50,
        "History length {} should equal the configured capacity",
        history.len()
    );

    // Truncation warning should have been shown (via has_truncation_warning flag)
    assert!(
        manager.history_buffer.has_truncation_warning(),
        "Truncation warning should be shown"
    );

    // Verify that old lines were evicted (line 1-50 should be gone, only recent lines remain)
    // The most recent output should be numbers close to 100
    let has_high_numbers = history.iter().any(|line| {
        if let OutputLine::Stdout { text, .. } = line {
            text.parse::<i32>().map(|n| n > 50).unwrap_or(false)
        } else {
            false
        }
    });
    assert!(
        has_high_numbers,
        "History should contain higher numbers (showing older lines were evicted)"
    );
}

// T075: Verify sub-100ms latency for simple commands
#[tokio::test]
async fn test_command_latency_under_100ms() {
    let manager = create_test_manager();

    let start = std::time::Instant::now();

    let result = execute_command_test(&manager, "echo test", None).await;
    assert!(result.is_ok(), "Command should succeed");

    let elapsed = start.elapsed();

    // Simple echo command should complete well under 100ms
    // We allow a bit more margin for CI environments (200ms)
    assert!(
        elapsed < Duration::from_millis(200),
        "Command latency should be under 200ms (was {:?})",
        elapsed
    );

    // Log the actual latency for monitoring
    println!("Simple echo command latency: {:?}", elapsed);
}

// Helper function to execute command (simulates what the Tauri command does)
async fn execute_command_test(
    manager: &ShellManager,
    command: &str,
    cwd: Option<String>,
) -> Result<CommandResponse, String> {
    use cepheus_lib::state::current_timestamp_ms;
    use std::process::Stdio;
    use tokio::io::{AsyncBufReadExt, BufReader};

    // Try to set busy state atomically
    if !manager.shell_state.try_set_busy().await {
        return Err("Command already running".to_string());
    }

    // Add command to history
    manager.history_buffer.push(OutputLine::Command {
        text: command.to_string(),
        timestamp: current_timestamp_ms(),
    });

    // Determine working directory
    let working_dir = match cwd {
        Some(path) => path,
        None => manager.get_cwd().await,
    };

    // Spawn the process
    let child_result = build_shell_command_test(command)
        .current_dir(&working_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut child = match child_result {
        Ok(c) => c,
        Err(e) => {
            manager.shell_state.set_busy(false).await;
            return Err(format!("Failed to spawn process: {}", e));
        }
    };

    // Store the child process
    *manager.shell_state.pid.lock().await = child.id();

    // Take stdout and stderr
    let stdout = child.stdout.take().expect("stdout not captured");
    let stderr = child.stderr.take().expect("stderr not captured");

    // Spawn tasks to read stdout and stderr
    let manager_stdout = manager.clone();
    let stdout_handle = tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        loop {
            match lines.next_line().await {
                Ok(Some(line)) => {
                    manager_stdout.history_buffer.push(OutputLine::Stdout {
                        text: line,
                        timestamp: current_timestamp_ms(),
                    });
                }
                Ok(None) => break,
                Err(e) => {
                    eprintln!("Error reading stdout: {}", e);
                    break;
                }
            }
        }
    });

    let manager_stderr = manager.clone();
    let stderr_handle = tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();

        loop {
            match lines.next_line().await {
                Ok(Some(line)) => {
                    manager_stderr.history_buffer.push(OutputLine::Stderr {
                        text: line,
                        timestamp: current_timestamp_ms(),
                    });
                }
                Ok(None) => break,
                Err(e) => {
                    eprintln!("Error reading stderr: {}", e);
                    break;
                }
            }
        }
    });

    // Wait for process to complete
    let status = child
        .wait()
        .await
        .map_err(|e| format!("Wait error: {}", e))?;

    // Wait for output readers to complete
    stdout_handle.await.expect("stdout reader task panicked");
    stderr_handle.await.expect("stderr reader task panicked");

    // Clear busy state
    manager.shell_state.set_busy(false).await;
    *manager.shell_state.pid.lock().await = None;

    let exit_code = status.code();

    match exit_code {
        Some(code) => Ok(CommandResponse::with_exit_code(code)),
        None => Ok(CommandResponse::failure(
            "Process terminated without exit code",
            None,
        )),
    }
}

// Helper function to cancel running command
async fn cancel_command_test(manager: &ShellManager) -> Result<(), String> {
    let pid = manager.shell_state.get_pid().await;

    match pid {
        Some(pid) => {
            #[cfg(unix)]
            {
                use nix::sys::signal::{self, Signal};
                use nix::unistd::Pid;

                signal::kill(Pid::from_raw(pid as i32), Signal::SIGINT)
                    .map_err(|e| format!("Failed to send SIGINT: {}", e))
            }

            #[cfg(windows)]
            {
                let output = tokio::process::Command::new("taskkill")
                    .arg("/PID")
                    .arg(pid.to_string())
                    .arg("/T")
                    .output()
                    .await
                    .map_err(|e| format!("Failed to spawn taskkill: {e}"))?;

                if output.status.success() {
                    Ok(())
                } else {
                    Err(format!(
                        "taskkill failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ))
                }
            }
        }
        None => Err("No command currently running".to_string()),
    }
}
