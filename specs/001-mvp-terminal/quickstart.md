# Quickstart: MVP Terminal Development

**Feature**: 001-mvp-terminal
**Audience**: Developers implementing the MVP terminal
**Last Updated**: 2025-11-30

## Overview

This guide helps you get started developing the MVP terminal application. It covers setup, architecture overview, development workflow, and key implementation patterns.

---

## Prerequisites

- **Rust**: Latest stable (verify: `rustc --version`)
- **Node.js**: v18+ (for Trunk)
- **Trunk**: Frontend build tool (`cargo install trunk`)
- **Tauri CLI**: Application builder (`cargo install tauri-cli`)
- **macOS**: MVP targets macOS only

---

## Project Structure

```
cepheus/
├── Cargo.toml                 # Workspace root (frontend)
├── src/                       # Leptos frontend (WASM)
│   ├── main.rs
│   ├── app.rs
│   ├── components/
│   └── models/
├── src-tauri/                 # Tauri backend (native)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── commands/
│   │   ├── state/
│   │   ├── models/
│   │   └── logging/
│   └── tests/
├── Trunk.toml                 # Frontend build config
└── specs/001-mvp-terminal/    # This feature's design docs
```

---

## Quick Start

### 1. Clone and Setup

```bash
cd /path/to/cepheus
git checkout 001-mvp-terminal

# Verify toolchain
rustc --version
trunk --version
cargo tauri --version
```

### 2. Run Development Server

```bash
# Starts Trunk (port 1420) + Tauri dev mode
# Hot-reloads frontend and backend
cargo tauri dev
```

**What this does**:
- Trunk serves frontend on `http://localhost:1420`
- Tauri watches `src-tauri/src/**/*.rs` for backend changes
- Opens desktop app window with devtools enabled

### 3. Make Changes

**Frontend (Leptos)**:
- Edit `src/components/*.rs`
- Save → Trunk rebuilds → App hot-reloads

**Backend (Tauri)**:
- Edit `src-tauri/src/**/*.rs`
- Save → Tauri rebuilds → App restarts

### 4. Run Tests

```bash
# All tests (frontend + backend)
cargo test

# Backend only
cargo test --manifest-path src-tauri/Cargo.toml

# Frontend only (WASM target)
cargo test --target wasm32-unknown-unknown

# Integration tests
cargo test --manifest-path src-tauri/Cargo.toml --test '*'
```

### 5. Code Quality Checks

```bash
# Format
cargo fmt

# Lint (must pass with no warnings)
cargo clippy

# Security audit
cargo audit
```

---

## Development Workflow

### Adding a New Tauri Command

1. **Define command in backend**:

```rust
// src-tauri/src/commands/shell.rs

#[tauri::command]
pub async fn my_new_command(
    param: String,
    state: State<'_, ShellManager>,
) -> Result<String, String> {
    // Implementation
    Ok("result".to_string())
}
```

2. **Register in lib.rs**:

```rust
// src-tauri/src/lib.rs

.invoke_handler(tauri::generate_handler![
    commands::shell::execute_command,
    commands::shell::my_new_command, // Add here
])
```

3. **Call from frontend**:

```rust
// src/components/my_component.rs

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

async fn call_command() {
    let args = serde_wasm_bindgen::to_value(&MyArgs {
        param: "value".to_string(),
    }).unwrap();

    match invoke("my_new_command", args).await {
        Ok(result) => {
            // Handle success
        }
        Err(e) => {
            // Handle error
        }
    }
}
```

### Adding a New Leptos Component

1. **Create component file**:

```rust
// src/components/my_component.rs

use leptos::prelude::*;

#[component]
pub fn MyComponent(
    #[prop(into)] message: String,
) -> impl IntoView {
    view! {
        <div class="my-component">
            {message}
        </div>
    }
}
```

2. **Export from components module**:

```rust
// src/components/mod.rs

pub mod my_component;
pub use my_component::MyComponent;
```

3. **Use in parent component**:

```rust
// src/components/terminal.rs

use crate::components::MyComponent;

view! {
    <Terminal>
        <MyComponent message="Hello" />
    </Terminal>
}
```

### Adding State to ShellManager

1. **Define state struct**:

```rust
// src-tauri/src/state/my_state.rs

pub struct MyState {
    data: Arc<Mutex<HashMap<String, String>>>,
}

impl MyState {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.data.lock().unwrap().get(key).cloned()
    }
}
```

2. **Add to ShellManager**:

```rust
// src-tauri/src/state/shell_manager.rs

pub struct ShellManager {
    shell_state: ShellState,
    history_buffer: HistoryBuffer,
    my_state: MyState, // Add here
}
```

3. **Initialize in new()**:

```rust
impl ShellManager {
    pub fn new() -> Self {
        Self {
            shell_state: ShellState::new(initial_cwd()),
            history_buffer: HistoryBuffer::new(10_000),
            my_state: MyState::new(),
        }
    }
}
```

---

## Key Implementation Patterns

### Pattern 1: Streaming Command Output

```rust
// src-tauri/src/commands/shell.rs

use tokio::process::Command;
use tokio::io::{BufReader, AsyncBufReadExt};

pub async fn execute_with_streaming(
    command: String,
    app_handle: AppHandle,
) -> Result<(), String> {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    // Stream stdout
    tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await.unwrap() {
            let output_line = OutputLine::Stdout {
                text: line,
                timestamp: current_timestamp_ms(),
            };

            app_handle.emit("output-line", output_line).unwrap();
        }
    });

    // Stream stderr (similar pattern)
    // ...

    child.wait().await.map_err(|e| e.to_string())?;
    Ok(())
}
```

### Pattern 2: Reactive State in Leptos

```rust
// src/components/my_component.rs

#[component]
pub fn MyComponent() -> impl IntoView {
    let state = use_context::<TerminalState>().unwrap();

    // Create local reactive state
    let (count, set_count) = signal(0);

    // React to context changes
    create_effect(move |_| {
        let history_len = state.history.get().len();
        logging::log!("History length: {}", history_len);
    });

    // Event handler
    let on_click = move |_| {
        set_count.update(|c| *c += 1);
    };

    view! {
        <button on:click=on_click>
            "Clicks: " {move || count.get()}
        </button>
    }
}
```

### Pattern 3: Tauri Event Listeners

```rust
// src/components/terminal.rs

create_effect(move |_| {
    spawn_local(async move {
        listen("output-line", move |event: Event| {
            let payload = event.payload();
            let line: OutputLine = serde_wasm_bindgen::from_value(payload)
                .expect("Failed to deserialize OutputLine");

            state.history.update(|h| h.push(line));
        })
        .await
        .expect("Failed to set up listener");
    });
});
```

### Pattern 4: Circular Buffer Management

```rust
// src-tauri/src/state/history_buffer.rs

pub fn push(&self, line: OutputLine) {
    let mut lines = self.lines.write().unwrap();

    // Check capacity
    if lines.len() >= self.max_capacity {
        lines.pop_front(); // Evict oldest

        // One-time warning
        if !*self.truncation_warning_shown.read().unwrap() {
            let warning = OutputLine::Notification {
                message: "Output truncated: line limit (10,000) exceeded".to_string(),
                level: NotificationLevel::Warning,
                timestamp: current_timestamp_ms(),
            };
            lines.push_back(warning);
            *self.truncation_warning_shown.write().unwrap() = true;
        }
    }

    lines.push_back(line);
}
```

---

## Testing

### Unit Test Example

```rust
// src-tauri/src/state/history_buffer.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_truncation() {
        let buffer = HistoryBuffer::new(3);

        buffer.push(OutputLine::Stdout {
            text: "line1".to_string(),
            timestamp: 1000,
        });
        buffer.push(OutputLine::Stdout {
            text: "line2".to_string(),
            timestamp: 2000,
        });
        buffer.push(OutputLine::Stdout {
            text: "line3".to_string(),
            timestamp: 3000,
        });

        assert_eq!(buffer.len(), 3);

        buffer.push(OutputLine::Stdout {
            text: "line4".to_string(),
            timestamp: 4000,
        });

        // line1 should be evicted
        let lines = buffer.get_all();
        assert_eq!(lines.len(), 4); // 3 data + 1 warning
        assert!(lines.iter().any(|l| matches!(l, OutputLine::Notification { .. })));
    }
}
```

### Integration Test Example

```rust
// src-tauri/tests/integration/shell_commands.rs

use cepheus::commands::shell::execute_command;

#[tokio::test]
async fn test_execute_echo_command() {
    let manager = ShellManager::new();
    let state = State::new(manager);

    let response = execute_command(
        "echo 'hello world'".to_string(),
        None,
        state,
    )
    .await
    .unwrap();

    assert!(response.success);
    assert_eq!(response.exit_code, Some(0));
}
```

---

## Debugging

### Backend Logging

```rust
// src-tauri/src/logging/file_logger.rs initialized on startup

// Use tracing macros
use tracing::{info, warn, error, debug};

#[tauri::command]
async fn my_command() -> Result<(), String> {
    info!("Command started");
    debug!("Debug details: {}", value);

    if let Err(e) = risky_operation() {
        error!("Operation failed: {}", e);
        return Err(e.to_string());
    }

    Ok(())
}
```

**View logs**:
```bash
tail -f ~/.cepheus/terminal.log
```

### Frontend Debugging

```rust
// Use web_sys console
use web_sys::console;

console::log_1(&"Debug message".into());
console::error_1(&format!("Error: {:?}", err).into());

// Or Leptos logging (to browser console)
use leptos::logging::log;

log!("Value: {}", my_value);
```

**Browser DevTools**:
- Right-click in app → Inspect Element
- Console tab shows all logs
- Network tab shows Tauri IPC (if intercepted)

---

## Common Issues

### Issue: "Command not found" when testing

**Solution**: Ensure the shell PATH is set correctly:

```rust
// src-tauri/src/commands/shell.rs

let mut cmd = Command::new("sh");
cmd.env("PATH", std::env::var("PATH").unwrap());
```

### Issue: Frontend doesn't update after backend change

**Solution**: Tauri dev mode should auto-restart. If not:

```bash
# Kill the process and restart
pkill -f "cargo tauri dev"
cargo tauri dev
```

### Issue: WASM compilation errors

**Solution**: Ensure wasm32 target is installed:

```bash
rustup target add wasm32-unknown-unknown
```

### Issue: Trunk build fails

**Solution**: Clear Trunk cache:

```bash
rm -rf dist/
trunk clean
cargo tauri dev
```

---

## Build for Release

```bash
# Creates distributable .app bundle (macOS)
cargo tauri build

# Output location
ls src-tauri/target/release/bundle/macos/
```

**Release Checklist**:
- [ ] All tests pass: `cargo test`
- [ ] No clippy warnings: `cargo clippy`
- [ ] Code formatted: `cargo fmt --check`
- [ ] Security audit: `cargo audit`
- [ ] Manual testing on macOS
- [ ] Performance check: sub-100ms command latency verified

---

## Next Steps

1. **Read design docs**:
   - `specs/001-mvp-terminal/spec.md` - Feature requirements
   - `specs/001-mvp-terminal/research.md` - Technical decisions
   - `specs/001-mvp-terminal/data-model.md` - Data structures
   - `specs/001-mvp-terminal/contracts/` - API contracts

2. **Implement in order**:
   - Backend: Shell process management (`ShellManager`, `HistoryBuffer`)
   - Backend: Tauri commands (`execute_command`, `cancel_command`)
   - Frontend: Leptos components (`Terminal`, `OutputDisplay`, `CommandInput`)
   - Integration: Wire up IPC and event listeners
   - Polish: Styling, error handling, notifications

3. **Test as you go**:
   - Write unit tests for buffer logic
   - Write integration tests for shell commands
   - Manual E2E testing via `cargo tauri dev`

---

## Resources

- **Tauri Docs**: https://tauri.app/v2/
- **Leptos Book**: https://leptos-rs.github.io/leptos/
- **Tokio Docs**: https://tokio.rs/
- **Project Constitution**: `.specify/memory/constitution.md`
- **CLAUDE.md**: Development guidelines for AI assistance

---

## Getting Help

- Check `CLAUDE.md` for project-specific patterns
- Review constitution for quality gates and principles
- Consult design docs in `specs/001-mvp-terminal/`
- Use `tracing` logs to debug backend issues
- Use browser DevTools to debug frontend issues

---

**Happy coding!** 🚀
