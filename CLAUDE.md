# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Cepheus is a desktop application built with **Tauri v2** (Rust backend) and **Leptos 0.7** (Rust frontend compiled to WebAssembly). This is a Rust-first, type-safe stack where both frontend and backend are written in Rust.

### Tech Stack
- **Frontend**: Leptos 0.7 with CSR (Client-Side Rendering), compiled to WASM via Trunk
- **Backend**: Tauri v2 with Rust, provides native OS capabilities and IPC
- **Build Tool**: Trunk (serves on port 1420 during development)
- **Workspace**: Cargo workspace with two members: `cepheus-ui` (frontend) and `cepheus` (backend)

## Development Commands

### Running the Application
```bash
# Development mode (runs trunk serve + tauri dev in parallel)
cargo tauri dev

# The above command:
# - Starts Trunk server on http://localhost:1420
# - Watches frontend changes (src/*.rs)
# - Watches backend changes (src-tauri/src/*.rs)
# - Hot-reloads both frontend and backend
```

### Building
```bash
# Build for production (creates distributable app bundle)
cargo tauri build

# Build frontend only (outputs to dist/)
trunk build

# Build backend only
cargo build --manifest-path src-tauri/Cargo.toml --release
```

### Testing
```bash
# Run all tests in workspace
cargo test

# Run backend tests only
cargo test --manifest-path src-tauri/Cargo.toml

# Run frontend tests (WASM tests)
cargo test --target wasm32-unknown-unknown

# Note: Currently no test directory exists - tests should be added per constitution
```

### Code Quality
```bash
# Format all code
cargo fmt

# Lint with Clippy (must pass with no warnings)
cargo clippy

# Lint with strict settings
cargo clippy -- -D warnings

# Security audit
cargo audit
```

### Development Workflow
```bash
# Install trunk if not already installed
cargo install trunk

# Install tauri-cli if not already installed
cargo install tauri-cli

# Clean build artifacts
cargo clean
rm -rf dist/
```

## Architecture

### Workspace Structure
This is a Cargo workspace with two crates:
- **`cepheus-ui`** (root Cargo.toml): Frontend Leptos app, compiles to WASM
- **`cepheus`** (src-tauri/Cargo.toml): Tauri backend, native binary with embedded webview

### Frontend (Leptos/WASM)
- **Entry**: `src/main.rs` - mounts root `<App/>` component
- **Components**: `src/app.rs` - currently contains main App component
- **Architecture**: Component-based reactive UI using Leptos signals
- **IPC**: Calls Tauri commands via `window.__TAURI__.core.invoke()`
- **Patterns**:
  - Use signals for reactive state: `let (value, set_value) = signal(T::default())`
  - Components marked with `#[component]` macro
  - View templates in `view! {}` macro with RSX syntax

### Backend (Tauri/Rust)
- **Entry**: `src-tauri/src/main.rs` - calls `cepheus_lib::run()`
- **Library**: `src-tauri/src/lib.rs` - contains Tauri app setup and command handlers
- **Commands**: Marked with `#[tauri::command]` macro, registered in `invoke_handler`
- **IPC**: Commands invoked from frontend, serialized via serde
- **Plugins**: Currently uses `tauri-plugin-opener`

### IPC Communication Pattern
Frontend (Leptos) → WASM Bindgen → Tauri IPC → Backend (Rust Commands)

Example:
```rust
// Frontend: src/app.rs
let args = serde_wasm_bindgen::to_value(&GreetArgs { name: &name }).unwrap();
let result = invoke("greet", args).await;

// Backend: src-tauri/src/lib.rs
#[tauri::command]
fn greet(name: &str) -> String { ... }
```

## Project Constitution

This project follows a **constitution-based development model** defined in `.specify/memory/constitution.md` (v1.0.0). Key principles:

### Core Principles (Must Follow)
1. **Rust-First Development**: All core functionality in Rust (no JS/TS)
2. **Type Safety**: Leverage Rust's type system and borrow checker, avoid `unsafe`
3. **Component-Based UI**: Small, focused, reusable Leptos components
4. **Test-Driven Development**: Write tests before implementation when practical
5. **Performance & Resource Efficiency**: Profile memory, lazy-load resources
6. **Cross-Platform Compatibility**: Test on macOS, Windows, Linux

### Code Organization (Recommended Structure)
- **Frontend**: `src/components/`, `src/pages/`, `src/utils/`
- **Backend**: `src-tauri/src/commands/`, `src-tauri/src/state/`, `src-tauri/src/models/`
- **Tests**: Unit tests alongside source, integration tests in `src-tauri/tests/`

### Quality Gates
Before committing:
- [ ] `cargo clippy` passes with no warnings
- [ ] `cargo test` passes
- [ ] `cargo fmt` applied
- [ ] No `unsafe` without documentation

Before merging:
- [ ] Constitution compliance verified
- [ ] Platform compatibility checked

## Speckit Workflow

This project uses **Speckit** slash commands for structured development:

### Available Commands
- `/speckit.constitution` - Create/update project constitution
- `/speckit.specify` - Create feature specification from description
- `/speckit.plan` - Generate implementation plan with design artifacts
- `/speckit.tasks` - Generate dependency-ordered tasks from plan
- `/speckit.implement` - Execute implementation plan tasks
- `/speckit.analyze` - Cross-artifact consistency analysis
- `/speckit.clarify` - Ask targeted clarification questions
- `/speckit.taskstoissues` - Convert tasks to GitHub issues

### Workflow Pattern
1. `/speckit.specify` - Define feature requirements
2. `/speckit.plan` - Design implementation approach
3. `/speckit.tasks` - Break down into actionable tasks
4. `/speckit.implement` - Execute tasks with tests

Templates located in `.specify/templates/` define artifact structure.

## Configuration Files

- **`Cargo.toml`** (root): Frontend workspace configuration
- **`src-tauri/Cargo.toml`**: Backend crate with `staticlib`, `cdylib`, `rlib` outputs
- **`src-tauri/tauri.conf.json`**: Tauri app config (window size, build commands, CSP)
- **`Trunk.toml`**: Frontend build settings (port 1420, ignore src-tauri/)
- **`.vscode/settings.json`**: Rust HTML emmet support for RSX syntax

## Common Patterns

### Adding a New Tauri Command
```rust
// 1. Define in src-tauri/src/lib.rs
#[tauri::command]
fn my_command(param: String) -> Result<String, String> {
    // Implementation
}

// 2. Register in invoke_handler
.invoke_handler(tauri::generate_handler![greet, my_command])

// 3. Call from frontend
let result = invoke("my_command", args).await;
```

### Creating a Leptos Component
```rust
// src/components/my_component.rs
use leptos::prelude::*;

#[component]
pub fn MyComponent(initial_value: String) -> impl IntoView {
    let (state, set_state) = signal(initial_value);

    view! {
        <div>{move || state.get()}</div>
    }
}
```

### Shared Types Between Frontend/Backend
Define types with `#[derive(Serialize, Deserialize)]` in backend, re-export for frontend via workspace members or feature flags.

## Important Notes

- **WASM Target**: Frontend runs in browser as WebAssembly, limited APIs available
- **No Direct File Access**: Frontend must call Tauri commands for file operations
- **Security**: Validate all IPC inputs on backend, use Tauri's permission system
- **Trunk Config**: Development server ignores `src-tauri/` directory to avoid rebuild loops
- **Window Subsystem**: Release builds hide console window on Windows (see src-tauri/main.rs)
- **Global Tauri**: `withGlobalTauri: true` enables `window.__TAURI__` object for IPC

## VS Code Integration

The project includes VS Code chat prompt recommendations for Speckit commands. Suggested prompts appear in chat interface for constitution, specify, plan, tasks, and implement workflows.
