# Data Model: MVP Terminal Application

**Feature**: 001-mvp-terminal
**Date**: 2025-11-30
**Phase**: 1 (Design & Contracts)

## Overview

This document defines the core data structures and entities for the MVP terminal application. All types are defined in Rust with serde serialization for crossing the Tauri IPC boundary between frontend (Leptos WASM) and backend (native).

## Core Entities

### 1. OutputLine

Represents a single line in the terminal history buffer.

**Location**: `src-tauri/src/models/output.rs` (backend), mirrored in `src/models/output_line.rs` (frontend)

**Definition**:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum OutputLine {
    /// User-entered command
    Command {
        text: String,
        timestamp: u64, // Unix timestamp milliseconds
    },
    /// Standard output from command
    Stdout {
        text: String,
        timestamp: u64,
    },
    /// Standard error from command
    Stderr {
        text: String,
        timestamp: u64,
    },
    /// System notification (e.g., "Shell restarted", "Output truncated...")
    Notification {
        message: String,
        level: NotificationLevel,
        timestamp: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
}
```

**Validation Rules**:
- `text` and `message` must not be empty strings
- `timestamp` should be monotonically increasing (enforced by insertion time)
- Maximum line length: 10,000 characters (truncate with "..." if exceeded)

**State Transitions**: Immutable once created

**Relationships**:
- Contained in `HistoryBuffer` (1:N relationship)
- Referenced by frontend UI components for rendering

---

### 2. HistoryBuffer

Manages the circular buffer of terminal output (max 10,000 lines).

**Location**: `src-tauri/src/state/history_buffer.rs`

**Definition**:
```rust
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

pub struct HistoryBuffer {
    lines: Arc<RwLock<VecDeque<OutputLine>>>,
    max_capacity: usize,
    truncation_warning_shown: Arc<RwLock<bool>>,
}

impl HistoryBuffer {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            lines: Arc::new(RwLock::new(VecDeque::with_capacity(max_capacity))),
            max_capacity,
            truncation_warning_shown: Arc::new(RwLock::new(false)),
        }
    }

    /// Add line to buffer; evict oldest if at capacity
    pub fn push(&self, line: OutputLine) {
        let mut lines = self.lines.write().unwrap();

        if lines.len() >= self.max_capacity {
            lines.pop_front(); // Remove oldest line

            // Show truncation warning once
            let mut warning_shown = self.truncation_warning_shown.write().unwrap();
            if !*warning_shown {
                let warning = OutputLine::Notification {
                    message: "Output truncated: line limit (10,000) exceeded".to_string(),
                    level: NotificationLevel::Warning,
                    timestamp: current_timestamp_ms(),
                };
                lines.push_back(warning);
                *warning_shown = true;
            }
        }

        lines.push_back(line);
    }

    /// Get all lines for rendering (cloned)
    pub fn get_all(&self) -> Vec<OutputLine> {
        self.lines.read().unwrap().iter().cloned().collect()
    }

    /// Get line count
    pub fn len(&self) -> usize {
        self.lines.read().unwrap().len()
    }

    /// Clear all lines
    pub fn clear(&self) {
        let mut lines = self.lines.write().unwrap();
        lines.clear();
        *self.truncation_warning_shown.write().unwrap() = false;
    }
}
```

**Validation Rules**:
- `max_capacity` must be > 0 (default: 10,000)
- Thread-safe: uses RwLock for concurrent access
- Truncation warning inserted exactly once when capacity exceeded

**Concurrency**: Read-heavy workload (frontend polling); write-light (only on command output)

---

### 3. CommandRequest

Request to execute a shell command (frontend → backend IPC).

**Location**: `src-tauri/src/models/command.rs`

**Definition**:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRequest {
    /// The shell command to execute
    pub command: String,
    /// Working directory (optional; defaults to current)
    pub cwd: Option<String>,
}
```

**Validation Rules**:
- `command` must not be empty
- `cwd` if provided must be a valid directory path (validated in backend)
- No length limit on command (shell will handle)

---

### 4. CommandResponse

Response from shell command execution (backend → frontend IPC).

**Location**: `src-tauri/src/models/command.rs`

**Definition**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResponse {
    /// Command execution succeeded
    pub success: bool,
    /// Exit code (if available)
    pub exit_code: Option<i32>,
    /// Error message (if execution failed)
    pub error: Option<String>,
}
```

**Validation Rules**:
- `success == false` implies `error` should be Some
- `exit_code` only present if process completed

---

### 5. ShellState

Tracks the current state of the shell process.

**Location**: `src-tauri/src/state/shell_manager.rs`

**Definition**:
```rust
use tokio::process::Child;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ShellState {
    /// Current running process (if any)
    pub process: Arc<Mutex<Option<Child>>>,
    /// Process ID of running command
    pub pid: Arc<Mutex<Option<u32>>>,
    /// Current working directory
    pub cwd: Arc<Mutex<String>>,
    /// Is shell currently executing a command?
    pub is_busy: Arc<Mutex<bool>>,
}

impl ShellState {
    pub fn new(initial_cwd: String) -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
            pid: Arc::new(Mutex::new(None)),
            cwd: Arc::new(Mutex::new(initial_cwd)),
            is_busy: Arc::new(Mutex::new(false)),
        }
    }
}
```

**State Transitions**:
```
Idle → Busy (on execute_command)
Busy → Idle (on command complete)
Busy → Crashed → Restarting → Idle (on shell crash)
```

**Validation Rules**:
- `pid` should match `process` existence
- `is_busy` true implies `process` is Some

---

### 6. TerminalState (Frontend)

Frontend-only reactive state (Leptos signals).

**Location**: `src/models/terminal_state.rs`

**Definition**:
```rust
use leptos::prelude::*;
use crate::models::output_line::OutputLine;

#[derive(Clone)]
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
    pub fn new() -> Self {
        Self {
            current_input: RwSignal::new(String::new()),
            history: RwSignal::new(Vec::new()),
            cwd: RwSignal::new(String::from("~")),
            is_busy: RwSignal::new(false),
            notification: RwSignal::new(None),
        }
    }

    /// Submit current command
    pub async fn submit_command(&self) {
        let cmd = self.current_input.get();
        if cmd.is_empty() {
            return;
        }

        self.current_input.set(String::new());
        self.is_busy.set(true);

        // Call Tauri IPC
        // ... (implemented in components)
    }
}
```

**Reactivity**: All fields are reactive signals that trigger UI updates

---

## Entity Relationships

```
┌─────────────────────┐
│   TerminalState     │  (Frontend - Leptos signals)
│  - current_input    │
│  - history          │◄─────────┐
│  - cwd              │          │ Synced via
│  - is_busy          │          │ Tauri events
│  - notification     │          │
└─────────────────────┘          │
                                 │
        IPC Commands             │
            ↓                    │
┌─────────────────────┐          │
│   ShellManager      │          │
│  (Tauri State)      │          │
│  - ShellState       │          │
│  - HistoryBuffer    │──────────┘
└─────────────────────┘
         │
         │ manages
         ↓
┌─────────────────────┐
│   HistoryBuffer     │
│  VecDeque<          │
│   OutputLine        │
│  >                  │
│  max: 10,000        │
└─────────────────────┘
         │
         │ contains
         ↓
┌─────────────────────┐
│   OutputLine        │
│  - Command          │
│  - Stdout           │
│  - Stderr           │
│  - Notification     │
└─────────────────────┘
```

## Data Flow

### Command Execution Flow

```
1. User types command
   → TerminalState.current_input updated (signal)

2. User presses Enter
   → submit_command() called
   → IPC: execute_command(CommandRequest)

3. Backend receives request
   → ShellManager spawns process
   → Captures stdout/stderr
   → Creates OutputLine entries
   → Pushes to HistoryBuffer

4. Backend streams output
   → Emits Tauri events with new OutputLines
   → Frontend updates TerminalState.history

5. Command completes
   → CommandResponse sent back
   → ShellState.is_busy → false
   → Frontend updates TerminalState.is_busy
```

### Buffer Truncation Flow

```
1. HistoryBuffer.push() called with line 10,001
   → Pop oldest line (line 1)
   → Check truncation_warning_shown flag

2. If warning not yet shown
   → Create Notification OutputLine
   → Insert warning into buffer
   → Set truncation_warning_shown = true

3. Insert new line at end
   → Buffer maintains 10,000 lines + 1 warning
```

### Shell Crash Recovery Flow

```
1. Background monitor task detects process exit
   → ShellState.process.wait() returns non-success

2. ShellManager.restart_shell() called
   → Spawn new shell process
   → Update ShellState.process and pid

3. Emit notification event
   → Create Notification OutputLine
   → Push to HistoryBuffer
   → Frontend displays "Shell restarted"

4. HistoryBuffer preserved
   → User sees history before crash
   → New commands go to new shell instance
```

## Serialization Format

All IPC types use JSON serialization via serde:

**Example OutputLine (Command)**:
```json
{
  "type": "Command",
  "data": {
    "text": "ls -la",
    "timestamp": 1701360000000
  }
}
```

**Example OutputLine (Notification)**:
```json
{
  "type": "Notification",
  "data": {
    "message": "Shell restarted",
    "level": "Warning",
    "timestamp": 1701360120000
  }
}
```

## Memory Considerations

**Estimated Memory Usage**:
- `OutputLine::Command`: ~100-200 bytes (text + enum overhead + timestamp)
- `OutputLine::Stdout`: ~50-500 bytes (varies by output length, avg 150)
- Total buffer: 10,000 lines × 150 bytes avg = **~1.5MB**
- Acceptable for desktop application

**Optimization Notes**:
- Use `String::from` only when necessary
- Clone `OutputLine` only when sending over IPC
- Frontend receives incremental updates (not full buffer each time)
- Consider `Arc<str>` if cloning becomes bottleneck (profile first)

## Validation Summary

| Entity | Key Validation |
|--------|----------------|
| OutputLine | Non-empty text, valid timestamp, max 10k chars/line |
| HistoryBuffer | Max capacity enforced, thread-safe, single warning |
| CommandRequest | Non-empty command, valid cwd path |
| ShellState | pid ↔ process consistency, state transitions valid |
| TerminalState | Reactive signals, async command submission |

## Next Steps

Proceed to API contracts definition in `contracts/` directory.
