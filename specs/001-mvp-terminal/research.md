# Research: MVP Terminal Application

**Feature**: 001-mvp-terminal
**Date**: 2025-11-30
**Phase**: 0 (Outline & Research)

## Overview

This document captures technical research and decisions for implementing an MVP terminal application using Tauri v2 and Leptos 0.7.

## Research Areas

### 1. Shell Integration Approach

**Decision**: Use **std::process::Command** with tokio async wrappers for MVP

**Rationale**:
- MVP explicitly excludes full-screen PTY programs (vim, top, ssh)
- Only need to capture stdout/stderr separately and support basic interactive input
- std::process::Command provides sufficient control for password prompts and Y/N confirmations
- Significantly simpler than PTY implementation
- Performance target (<100ms) achievable with async process spawning
- PTY adds complexity that MVP doesn't require

**Implementation Notes**:
- Use `tokio::process::Command` for async shell execution
- Spawn with `.stdout(Stdio::piped())` and `.stderr(Stdio::piped())` for separate streams
- Use `tokio::io::BufReader` with `.lines()` to stream output line-by-line
- Store `Child` handle in Tauri state for signal management
- For interactive input: write to stdin pipe when needed (basic prompts only)

**Tradeoffs**:
- ✅ Gain: Simpler implementation, easier testing, sufficient for MVP requirements
- ✅ Gain: Direct control over stdout/stderr separation
- ✅ Gain: Lower complexity, fewer dependencies
- ❌ Lose: Cannot support full-screen programs (acceptable - explicitly out of scope)
- ❌ Lose: No terminal escape sequence handling (fine for MVP - just display raw output)

**Alternatives Considered**:
- **portable-pty**: Full PTY implementation, supports vim/top/etc., but adds significant complexity for features MVP doesn't need
- **alacritty_terminal**: Excellent PTY + VT parsing, but heavy dependency for MVP scope
- **Hybrid approach**: std::process now, migrate to PTY post-MVP if needed (chosen path enables this)

### 2. Signal Handling (Ctrl+C)

**Decision**: Frontend captures Ctrl+C keypress → sends IPC command → Backend sends SIGINT to child process

**Rationale**:
- Leptos frontend can capture keyboard events (onkeydown with Ctrl+C check)
- Tauri IPC provides clean separation: frontend handles UI, backend handles process management
- macOS supports sending signals to child processes via `nix` crate or `libc`
- Aligns with Tauri architecture (frontend = UI, backend = OS integration)

**Implementation Pattern**:

```rust
// Frontend (Leptos)
fn handle_keydown(ev: web_sys::KeyboardEvent) {
    if ev.ctrl_key() && ev.key() == "c" {
        ev.prevent_default();
        // Call Tauri command
        invoke("cancel_command", JsValue::NULL).await;
    }
}

// Backend (Tauri command)
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;

#[tauri::command]
async fn cancel_command(state: State<'_, ShellManager>) -> Result<(), String> {
    if let Some(child_pid) = state.get_running_pid() {
        signal::kill(Pid::from_raw(child_pid), Signal::SIGINT)
            .map_err(|e| format!("Failed to send SIGINT: {}", e))?;
    }
    Ok(())
}
```

**Dependencies**:
- `nix = "0.27"` - Unix signal handling (macOS compatible)
- Store `Child` in Arc<Mutex<Option<Child>>> in Tauri state

**Edge Cases**:
- **Process already exited**: Check if child exists before signaling; ignore error if process not found
- **Multiple Ctrl+C presses**: Debounce in frontend or check state in backend
- **Shell crash during signal**: SIGINT will fail; shell crash detection will trigger restart

### 3. Testing Strategy

**Decision**: Multi-layered testing approach

#### 3.1 Shell Process Testing

**Unit Tests**:
- Use mocks for testing shell command logic (e.g., command parsing, output formatting)
- No mocking for std::process - use real process execution in integration tests

**Integration Tests** (`src-tauri/tests/integration/`):
- Spawn real shell processes with simple commands (`echo`, `pwd`, `ls`)
- Test stdout/stderr separation with `echo "out" && echo "err" >&2`
- Test cancellation by spawning `sleep 10` and sending SIGINT
- Test crash detection with `sh -c 'exit 1'` or `sh -c 'kill -9 $$'`

**Pattern**:
```rust
#[tokio::test]
async fn test_execute_simple_command() {
    let manager = ShellManager::new().await;
    let result = manager.execute("echo hello").await.unwrap();
    assert_eq!(result.stdout, "hello\n");
    assert_eq!(result.stderr, "");
}
```

#### 3.2 Circular Buffer Testing

**Unit Tests** (`src-tauri/tests/unit/`):
- Property-based testing with `proptest` for buffer wraparound
- Test exact 10,000 line boundary
- Test truncation warning insertion
- Test concurrent reads during writes (if needed)

**Pattern**:
```rust
#[test]
fn test_buffer_truncation_at_10k() {
    let mut buffer = HistoryBuffer::new(10_000);

    // Add 10,001 lines
    for i in 0..=10_000 {
        buffer.push(format!("line {}", i));
    }

    assert_eq!(buffer.len(), 10_000);
    assert_eq!(buffer.first().unwrap(), "line 1"); // line 0 evicted
    assert!(buffer.contains_warning("Output truncated"));
}
```

#### 3.3 Tauri Command Testing

**Integration Tests**:
- Use Tauri's test utilities to test IPC commands
- Mock AppHandle with `tauri::test::mock_builder()`
- Test command serialization/deserialization

**Pattern**:
```rust
#[tokio::test]
async fn test_execute_command_ipc() {
    let app = tauri::test::mock_builder().build().unwrap();
    let result = execute_command("pwd".to_string())
        .await
        .unwrap();
    assert!(!result.is_empty());
}
```

#### 3.4 Frontend (WASM) Testing

**Approach**: Minimal unit tests for pure logic; rely on integration tests for component behavior

- Test output line parsing/formatting logic
- Test state management (signal updates)
- Skip full component testing (Leptos in WASM test environment is complex)
- Manual E2E testing via `cargo tauri dev`

**Recommended Crates**:
- `proptest = "1.4"` - Property-based testing for buffer logic
- No additional mocking crates needed (use real processes in integration tests)

**Test Structure**:
```
src-tauri/tests/
├── integration/
│   ├── shell_commands.rs    # Real process execution tests
│   └── history_buffer.rs    # Buffer behavior with real data
└── unit/
    ├── models.rs            # Serde serialization tests
    └── buffer_logic.rs      # Circular buffer unit tests
```

### 4. Logging Implementation

**Decision**: Use `tracing` crate with file appender to `~/.cepheus/terminal.log`

**Rationale**:
- `tracing` is industry-standard for Rust async logging
- Supports structured logging (better than simple println debugging)
- `tracing-appender` provides non-blocking file writer
- Compatible with tokio async runtime

**Implementation**:
```rust
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use tracing_appender::rolling;

pub fn setup_logging() -> Result<(), Box<dyn std::error::Error>> {
    let log_dir = dirs::home_dir()
        .ok_or("Cannot find home directory")?
        .join(".cepheus");

    std::fs::create_dir_all(&log_dir)?;

    let file_appender = rolling::never(&log_dir, "terminal.log");

    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(file_appender))
        .with(EnvFilter::from_default_env()
            .add_directive("cepheus=debug".parse()?))
        .init();

    Ok(())
}
```

**Dependencies**:
- `tracing = "0.1"`
- `tracing-subscriber = { version = "0.3", features = ["env-filter"] }`
- `tracing-appender = "0.2"`
- `dirs = "5.0"` - Cross-platform home directory

**Logged Events**:
- Command execution start/end
- Shell process crashes
- Buffer truncation events
- IPC command invocations
- Error conditions

### 5. Circular Buffer Implementation

**Decision**: Use `VecDeque<OutputLine>` with manual capacity management

**Rationale**:
- `VecDeque` provides efficient pop_front() for FIFO behavior
- Manual capacity check simpler than custom circular buffer
- Maximum 10,000 lines is manageable in memory (~1-2MB assuming 100 chars/line avg)

**Implementation Pattern**:
```rust
use std::collections::VecDeque;

pub struct HistoryBuffer {
    lines: VecDeque<OutputLine>,
    max_capacity: usize,
}

impl HistoryBuffer {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            lines: VecDeque::with_capacity(max_capacity),
            max_capacity,
        }
    }

    pub fn push(&mut self, line: OutputLine) {
        if self.lines.len() >= self.max_capacity {
            self.lines.pop_front(); // Remove oldest
            // Insert truncation warning if not recently added
        }
        self.lines.push_back(line);
    }
}
```

**Memory Estimate**:
- 10,000 lines × ~150 bytes/line (including enum overhead) ≈ 1.5MB
- Acceptable for desktop application

### 6. Shell Crash Detection

**Decision**: Monitor child process with `wait()` in background task; restart on unexpected exit

**Implementation Pattern**:
```rust
async fn monitor_shell_process(mut child: Child, state: Arc<ShellManager>) {
    let status = child.wait().await;

    match status {
        Ok(exit_status) if !exit_status.success() => {
            // Shell crashed or exited with error
            tracing::error!("Shell process crashed: {:?}", exit_status);
            state.restart_shell().await;
            state.emit_notification("Shell restarted").await;
        }
        Err(e) => {
            tracing::error!("Failed to wait for shell: {}", e);
            state.restart_shell().await;
        }
        _ => {
            // Normal exit (shouldn't happen in long-running shell)
            tracing::warn!("Shell process exited normally");
        }
    }
}
```

**Restart Strategy**:
- Spawn new shell process with same configuration
- Preserve history buffer (don't clear on restart)
- Emit Tauri event to frontend for UI notification
- Log restart event to file

## Summary of Key Decisions

| Area | Decision | Primary Crate(s) |
|------|----------|------------------|
| Shell Integration | std::process::Command + tokio | `tokio` |
| Signal Handling | nix crate for SIGINT | `nix` |
| Logging | tracing to file | `tracing`, `tracing-appender` |
| Buffer | VecDeque with manual capacity | std library |
| Testing | Real processes + property tests | `proptest` |
| Crash Recovery | Background monitor task + restart | tokio tasks |

## Dependencies to Add

```toml
# src-tauri/Cargo.toml
[dependencies]
tokio = { version = "1", features = ["process", "io-util", "rt-multi-thread"] }
nix = { version = "0.27", features = ["signal", "process"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-appender = "0.2"
dirs = "5.0"
serde = { version = "1", features = ["derive"] }

[dev-dependencies]
proptest = "1.4"
```

## Open Questions / Future Considerations

1. **Password prompt detection**: How to detect when shell needs stdin input? (Defer to post-MVP - may need heuristics or timeout)
2. **ANSI escape sequence handling**: Should we strip or preserve color codes? (MVP: preserve as-is, may render as text)
3. **Multi-line command editing**: Support for commands spanning multiple lines? (MVP: single-line only)
4. **Command history navigation**: Up/down arrows for previous commands? (Nice-to-have, defer to post-MVP)

## Next Steps

Proceed to Phase 1: Design & Contracts
- Generate data-model.md with entity definitions
- Create API contracts for Tauri commands
- Write quickstart.md for developer onboarding
