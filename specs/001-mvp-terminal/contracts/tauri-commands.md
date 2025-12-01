# Tauri IPC Commands Contract

**Feature**: 001-mvp-terminal
**Date**: 2025-11-30
**Protocol**: Tauri IPC (JSON-serialized Rust types)

## Overview

This document defines the Tauri command interface between the Leptos frontend (WASM) and Tauri backend (native Rust). All commands follow Tauri's `#[tauri::command]` pattern with automatic serde JSON serialization.

---

## Commands

### 1. execute_command

Execute a shell command and stream output to the terminal.

**Function Signature**:
```rust
#[tauri::command]
async fn execute_command(
    command: String,
    cwd: Option<String>,
    state: State<'_, ShellManager>,
) -> Result<CommandResponse, String>
```

**Request**:
```typescript
// Frontend (JavaScript/TypeScript via wasm-bindgen)
const response = await invoke('execute_command', {
  command: 'ls -la',
  cwd: '/Users/username' // optional
});
```

**Request Schema**:
```json
{
  "command": "string (required, non-empty)",
  "cwd": "string | null (optional, defaults to current)"
}
```

**Response Schema**:
```json
{
  "success": "boolean",
  "exit_code": "number | null",
  "error": "string | null"
}
```

**Response Examples**:

Success:
```json
{
  "success": true,
  "exit_code": 0,
  "error": null
}
```

Failure (command not found):
```json
{
  "success": false,
  "exit_code": 127,
  "error": "command not found: invalidcmd"
}
```

Failure (shell crash):
```json
{
  "success": false,
  "exit_code": null,
  "error": "Shell process crashed"
}
```

**Side Effects**:
- Spawns tokio async process
- Streams stdout/stderr to HistoryBuffer
- Emits `output-line` events during execution
- Updates ShellState.is_busy
- Logs execution to ~/.cepheus/terminal.log

**Error Conditions**:
- Invalid cwd path → returns error
- Empty command → returns error
- Shell not available → returns error
- Process spawn failure → returns error

**Performance**: Target <100ms to first output for simple commands

---

### 2. cancel_command

Send SIGINT to the currently running command (Ctrl+C).

**Function Signature**:
```rust
#[tauri::command]
async fn cancel_command(
    state: State<'_, ShellManager>,
) -> Result<(), String>
```

**Request**:
```typescript
await invoke('cancel_command', {});
```

**Request Schema**: No parameters

**Response Schema**:
```json
{} // Empty success, or error string
```

**Response Examples**:

Success:
```json
{} // Unit type, serialized as empty
```

Failure (no running command):
```json
"No command currently running"
```

**Side Effects**:
- Sends SIGINT signal to child process
- Process termination triggers command completion flow
- HistoryBuffer receives Stderr output (signal termination message)
- Updates ShellState.is_busy to false

**Error Conditions**:
- No active process → returns error message
- Signal send failure → returns error
- Process already exited → silently succeeds

---

### 3. get_history

Retrieve the full terminal history buffer.

**Function Signature**:
```rust
#[tauri::command]
async fn get_history(
    state: State<'_, ShellManager>,
) -> Result<Vec<OutputLine>, String>
```

**Request**:
```typescript
const history = await invoke('get_history', {});
```

**Request Schema**: No parameters

**Response Schema**:
```json
[
  {
    "type": "Command",
    "data": {
      "text": "string",
      "timestamp": "number (ms)"
    }
  },
  {
    "type": "Stdout",
    "data": {
      "text": "string",
      "timestamp": "number (ms)"
    }
  },
  // ... more OutputLine entries
]
```

**Response Example**:
```json
[
  {
    "type": "Command",
    "data": {
      "text": "echo hello",
      "timestamp": 1701360000000
    }
  },
  {
    "type": "Stdout",
    "data": {
      "text": "hello",
      "timestamp": 1701360000050
    }
  },
  {
    "type": "Notification",
    "data": {
      "message": "Output truncated: line limit (10,000) exceeded",
      "level": "Warning",
      "timestamp": 1701360120000
    }
  }
]
```

**Side Effects**: None (read-only)

**Error Conditions**: Should not fail (returns empty vec if buffer empty)

**Performance**: O(n) where n = buffer size (up to 10k lines); ~1-5ms for full buffer

---

### 4. get_cwd

Get the current working directory of the shell.

**Function Signature**:
```rust
#[tauri::command]
async fn get_cwd(
    state: State<'_, ShellManager>,
) -> Result<String, String>
```

**Request**:
```typescript
const cwd = await invoke('get_cwd', {});
```

**Request Schema**: No parameters

**Response Schema**:
```json
"string (absolute path)"
```

**Response Example**:
```json
"/Users/username/projects/cepheus"
```

**Side Effects**: None (read-only)

**Error Conditions**:
- CWD unavailable → returns error
- Path no longer exists → returns error

---

### 5. change_directory

Change the working directory for subsequent commands.

**Function Signature**:
```rust
#[tauri::command]
async fn change_directory(
    path: String,
    state: State<'_, ShellManager>,
) -> Result<String, String>
```

**Request**:
```typescript
const newCwd = await invoke('change_directory', {
  path: '/Users/username/Documents'
});
```

**Request Schema**:
```json
{
  "path": "string (required, absolute or relative)"
}
```

**Response Schema**:
```json
"string (new absolute path)"
```

**Response Example**:
```json
"/Users/username/Documents"
```

**Side Effects**:
- Updates ShellState.cwd
- All subsequent commands use new cwd

**Error Conditions**:
- Path doesn't exist → returns error
- Path is not a directory → returns error
- Permission denied → returns error

---

## Events

Tauri backend emits events that frontend can listen to.

### Event: output-line

Emitted when new output is available during command execution.

**Event Name**: `output-line`

**Payload Schema**:
```json
{
  "type": "Stdout" | "Stderr",
  "data": {
    "text": "string",
    "timestamp": "number (ms)"
  }
}
```

**Frontend Listener**:
```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen('output-line', (event) => {
  const outputLine = event.payload as OutputLine;
  // Append to history
  terminalState.history.update(h => [...h, outputLine]);
});
```

**Emission Frequency**: Line-by-line as output is produced (real-time streaming)

---

### Event: shell-notification

Emitted for system notifications (truncation, restart, etc.).

**Event Name**: `shell-notification`

**Payload Schema**:
```json
{
  "type": "Notification",
  "data": {
    "message": "string",
    "level": "Info" | "Warning" | "Error",
    "timestamp": "number (ms)"
  }
}
```

**Frontend Listener**:
```typescript
await listen('shell-notification', (event) => {
  const notification = event.payload as OutputLine;
  terminalState.notification.set(notification.data.message);
  // Auto-dismiss after 3 seconds
  setTimeout(() => terminalState.notification.set(null), 3000);
});
```

**Emission Triggers**:
- Buffer truncation occurs
- Shell process crashes and restarts
- Error conditions during execution

---

## Error Handling

All commands return `Result<T, String>` where the error case is a human-readable error message.

**Error Response Format** (serialized):
```json
"Error message string"
```

**Frontend Error Handling Pattern**:
```typescript
try {
  const response = await invoke('execute_command', { command: 'ls' });
  // Handle success
} catch (error) {
  // error is a string
  console.error('Command failed:', error);
  terminalState.notification.set(error);
}
```

---

## Concurrency & State Management

**Thread Safety**:
- All state (ShellManager, HistoryBuffer) uses Arc<Mutex<T>> or Arc<RwLock<T>>
- Tauri ensures commands are thread-safe
- Frontend calls can happen concurrently; backend serializes access to shared state

**Command Execution Sequencing**:
- Only one command can execute at a time (enforced by ShellState.is_busy check)
- Attempting to execute while busy returns error: "Command already running"
- cancel_command can be called while busy

**State Consistency**:
- HistoryBuffer is append-only during execution
- get_history returns a snapshot (cloned Vec)
- Events provide incremental updates

---

## Performance Targets

| Operation | Target Latency |
|-----------|----------------|
| execute_command (to first output) | <100ms |
| cancel_command | <50ms |
| get_history | <10ms |
| get_cwd / change_directory | <5ms |
| output-line event emission | Real-time (no buffering) |

---

## Security Considerations

**Input Validation**:
- Command strings: No validation (user responsible, shell will handle)
- Path validation: Verify directory exists before changing cwd
- No command filtering per spec requirement

**IPC Boundary**:
- All inputs deserialized and type-checked by serde
- String lengths implicitly limited by serde max (no attack vector)
- No eval or code injection (commands passed directly to shell)

**File System Access**:
- Backend runs with user permissions (Tauri default)
- No elevated privileges required
- Logging to ~/.cepheus/ respects user home directory

---

## Testing

**Integration Tests** (`src-tauri/tests/integration/commands.rs`):
```rust
#[tokio::test]
async fn test_execute_command() {
    let manager = ShellManager::new().await;
    let response = execute_command(
        "echo test".to_string(),
        None,
        State::new(manager),
    ).await.unwrap();

    assert!(response.success);
    assert_eq!(response.exit_code, Some(0));
}

#[tokio::test]
async fn test_cancel_command() {
    let manager = ShellManager::new().await;

    // Start long-running command
    tokio::spawn(execute_command(
        "sleep 10".to_string(),
        None,
        State::new(manager.clone()),
    ));

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Cancel it
    let result = cancel_command(State::new(manager)).await;
    assert!(result.is_ok());
}
```

---

## Summary

| Command | Purpose | Mutates State | Returns |
|---------|---------|---------------|---------|
| execute_command | Run shell command | Yes (HistoryBuffer, ShellState) | CommandResponse |
| cancel_command | Stop running command | Yes (ShellState) | () |
| get_history | Fetch terminal history | No | Vec<OutputLine> |
| get_cwd | Get current directory | No | String |
| change_directory | Change working directory | Yes (ShellState.cwd) | String |

| Event | Trigger | Payload |
|-------|---------|---------|
| output-line | Command output available | OutputLine |
| shell-notification | System notification | OutputLine (Notification) |
