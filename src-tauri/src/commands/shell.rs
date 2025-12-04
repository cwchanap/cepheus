use std::process::Stdio;

use tauri::{AppHandle, Emitter, State};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::models::{CommandResponse, NotificationLevel, OutputLine};
use crate::state::{current_timestamp_ms, ShellManager};

/// Execute a shell command and stream output to the terminal.
///
/// # Arguments
/// * `command` - The shell command to execute
/// * `cwd` - Optional working directory (defaults to current)
/// * `state` - Tauri managed `ShellManager` state
/// * `app` - Tauri app handle for emitting events
///
/// # Returns
/// * `Ok(CommandResponse)` - Command execution result
/// * `Err(String)` - Error message if execution failed
#[tauri::command]
pub async fn execute_command(
    command: String,
    cwd: Option<String>,
    state: State<'_, ShellManager>,
    app: AppHandle,
) -> Result<CommandResponse, String> {
    tracing::info!("Executing command: {}", command);

    // Check if empty command
    if command.trim().is_empty() {
        return Err("Command cannot be empty".to_string());
    }

    // Check if already busy
    if state.is_busy().await {
        tracing::warn!("Attempted to execute command while busy");
        return Err("Command already running".to_string());
    }

    // Set busy state
    state.shell_state.set_busy(true).await;

    // Add command to history
    let cmd_line = OutputLine::Command {
        text: command.clone(),
        timestamp: current_timestamp_ms(),
    };
    state.history_buffer.push(cmd_line.clone());

    // Emit command line event
    if let Err(e) = app.emit("output-line", &cmd_line) {
        tracing::error!("Failed to emit output-line event: {}", e);
    }

    // Determine working directory
    let working_dir = match cwd {
        Some(path) => {
            // Validate directory exists
            if !std::path::Path::new(&path).is_dir() {
                state.shell_state.set_busy(false).await;
                return Err(format!("Directory does not exist: {path}"));
            }
            path
        }
        None => state.get_cwd().await,
    };

    tracing::debug!("Working directory: {}", working_dir);

    // Spawn the process
    let child_result = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .current_dir(&working_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut child = match child_result {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to spawn process: {}", e);
            state.shell_state.set_busy(false).await;
            return Err(format!("Failed to spawn process: {e}"));
        }
    };

    // Store the child process PID
    let pid = child.id();
    *state.shell_state.pid.lock().await = pid;
    tracing::debug!("Process spawned with PID: {:?}", pid);

    // Take stdout and stderr
    let stdout = child.stdout.take().expect("stdout not captured");
    let stderr = child.stderr.take().expect("stderr not captured");

    // Clone state and app for background tasks
    let state_stdout = state.inner().clone();
    let app_stdout = app.clone();

    // Spawn task to read stdout
    let stdout_handle = tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            let output_line = OutputLine::Stdout {
                text: line,
                timestamp: current_timestamp_ms(),
            };
            state_stdout.history_buffer.push(output_line.clone());

            if let Err(e) = app_stdout.emit("output-line", &output_line) {
                tracing::error!("Failed to emit stdout event: {}", e);
            }
        }
    });

    // Clone state and app for stderr task
    let state_stderr = state.inner().clone();
    let app_stderr = app.clone();

    // Spawn task to read stderr
    let stderr_handle = tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            let output_line = OutputLine::Stderr {
                text: line,
                timestamp: current_timestamp_ms(),
            };
            state_stderr.history_buffer.push(output_line.clone());

            if let Err(e) = app_stderr.emit("output-line", &output_line) {
                tracing::error!("Failed to emit stderr event: {}", e);
            }
        }
    });

    // Wait for process to complete
    let status = match child.wait().await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to wait for process: {}", e);
            // Wait for output readers to complete
            let _ = stdout_handle.await;
            let _ = stderr_handle.await;

            state.shell_state.set_busy(false).await;
            *state.shell_state.pid.lock().await = None;

            return Err(format!("Failed to wait for process: {e}"));
        }
    };

    // Wait for output readers to complete
    let _ = stdout_handle.await;
    let _ = stderr_handle.await;

    // Clear busy state
    state.shell_state.set_busy(false).await;
    *state.shell_state.pid.lock().await = None;

    let exit_code = status.code();
    let success = status.success();

    tracing::info!(
        "Command completed with exit code: {:?}, success: {}",
        exit_code,
        success
    );

    Ok(CommandResponse {
        success,
        exit_code,
        error: if success {
            None
        } else {
            Some(format!("Command exited with code {exit_code:?}"))
        },
    })
}

/// Send SIGINT to the currently running command (Ctrl+C).
///
/// # Arguments
/// * `state` - Tauri managed `ShellManager` state
///
/// # Returns
/// * `Ok(())` - Signal sent successfully
/// * `Err(String)` - Error message if no command is running
#[tauri::command]
pub async fn cancel_command(state: State<'_, ShellManager>) -> Result<(), String> {
    use nix::sys::signal::{self, Signal};
    use nix::unistd::Pid;

    tracing::info!("Cancel command requested");

    let pid = state.shell_state.get_pid().await;

    if let Some(pid) = pid {
        tracing::info!("Sending SIGINT to PID: {}", pid);
        signal::kill(Pid::from_raw(pid as i32), Signal::SIGINT)
            .map_err(|e| format!("Failed to send SIGINT: {e}"))
    } else {
        tracing::warn!("Cancel requested but no command is running");
        Err("No command currently running".to_string())
    }
}

/// Retrieve the full terminal history buffer.
///
/// # Arguments
/// * `state` - Tauri managed `ShellManager` state
///
/// # Returns
/// * `Ok(Vec<OutputLine>)` - All lines in the history buffer
#[tauri::command]
pub async fn get_history(state: State<'_, ShellManager>) -> Result<Vec<OutputLine>, String> {
    tracing::debug!("Getting history buffer");
    Ok(state.history_buffer.get_all())
}

/// Get the current working directory of the shell.
///
/// # Arguments
/// * `state` - Tauri managed `ShellManager` state
///
/// # Returns
/// * `Ok(String)` - Current working directory path
#[tauri::command]
pub async fn get_cwd(state: State<'_, ShellManager>) -> Result<String, String> {
    let cwd = state.get_cwd().await;
    tracing::debug!("Getting CWD: {}", cwd);
    Ok(cwd)
}

/// Change the working directory for subsequent commands.
///
/// # Arguments
/// * `path` - New working directory path (absolute or relative)
/// * `state` - Tauri managed `ShellManager` state
/// * `app` - Tauri app handle for emitting events
///
/// # Returns
/// * `Ok(String)` - New absolute path
/// * `Err(String)` - Error message if path is invalid
#[tauri::command]
pub async fn change_directory(
    path: String,
    state: State<'_, ShellManager>,
    app: AppHandle,
) -> Result<String, String> {
    tracing::info!("Changing directory to: {}", path);

    let target_path = std::path::Path::new(&path);

    // Handle relative paths
    let absolute_path = if target_path.is_relative() {
        let current = state.get_cwd().await;
        std::path::Path::new(&current)
            .join(target_path)
            .canonicalize()
            .map_err(|e| format!("Invalid path: {e}"))?
    } else {
        target_path
            .canonicalize()
            .map_err(|e| format!("Invalid path: {e}"))?
    };

    // Verify it's a directory
    if !absolute_path.is_dir() {
        return Err(format!("Not a directory: {}", absolute_path.display()));
    }

    let new_cwd = absolute_path.to_string_lossy().to_string();
    state.shell_state.set_cwd(new_cwd.clone()).await;

    tracing::info!("Directory changed to: {}", new_cwd);

    // Emit notification
    let notification = OutputLine::Notification {
        message: format!("Changed directory to: {new_cwd}"),
        level: NotificationLevel::Info,
        timestamp: current_timestamp_ms(),
    };

    if let Err(e) = app.emit("shell-notification", &notification) {
        tracing::error!("Failed to emit notification: {}", e);
    }

    Ok(new_cwd)
}
