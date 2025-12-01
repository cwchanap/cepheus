# Implementation Plan: MVP Terminal Application

**Branch**: `001-mvp-terminal` | **Date**: 2025-11-30 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-mvp-terminal/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Build a minimal viable product terminal application for macOS that allows users to execute shell commands, view output history, and interact with their default shell. The terminal will use Tauri v2 for the native backend (handling shell process management) and Leptos 0.7 for the reactive frontend UI (WASM). Core features include command input, output display with stdout/stderr distinction, scrollable 10,000-line history buffer, Ctrl+C interrupt support, automatic shell crash recovery, and file-based logging for debugging.

## Technical Context

**Language/Version**: Rust (latest stable, currently using workspace with Tauri v2 + Leptos 0.7)
**Primary Dependencies**:
- Backend: Tauri v2 (native OS integration), tokio (async runtime), tracing (logging)
- Frontend: Leptos 0.7 (reactive UI framework compiled to WASM)
- Shell Integration: NEEDS CLARIFICATION (pty vs std::process, signal handling approach)
- Build Tools: Trunk (frontend bundler), tauri-cli (application builder)

**Storage**:
- In-memory: Circular buffer for 10,000-line terminal history
- File-based: `~/.cepheus/terminal.log` for debugging logs (append-only)
- No database required for MVP

**Testing**:
- cargo test (unit tests for both frontend WASM and backend)
- Integration tests in src-tauri/tests/ for Tauri commands
- NEEDS CLARIFICATION (shell process mocking strategy, E2E testing approach)

**Target Platform**: macOS only for MVP (Tauri supports cross-platform but scoped to single platform initially)

**Project Type**: Desktop application (Tauri workspace: frontend WASM + backend native binary)

**Performance Goals**:
- Sub-100ms latency from Enter keypress to first output for simple commands
- Sub-500ms shell crash recovery time
- Smooth scrolling for 10,000 line history buffer

**Constraints**:
- 10,000 line output buffer maximum (hard limit to prevent memory exhaustion)
- No PTY support for full-screen programs (vim, top, ssh) in MVP
- Basic interactive input only (password prompts, Y/N confirmations)
- No command filtering/safety validation

**Scale/Scope**:
- Single terminal session (no tabs/splits)
- Single user, local execution only
- ~5-10 Tauri commands for IPC
- ~3-5 Leptos components (input, output, prompt, notifications)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Core Principles Compliance

| Principle | Status | Notes |
|-----------|--------|-------|
| **I. Rust-First Development** | ✅ PASS | All functionality in Rust: Tauri backend + Leptos frontend (WASM) |
| **II. Type Safety & Compile-Time Guarantees** | ✅ PASS | Leveraging Rust's type system for IPC messages, state management; avoiding unsafe code |
| **III. Component-Based UI Architecture** | ✅ PASS | Leptos components: Terminal, CommandInput, OutputDisplay, PromptIndicator, NotificationBar |
| **IV. Test-Driven Development** | ⚠️ NEEDS ATTENTION | Tests required for shell management, IPC commands, circular buffer logic |
| **V. Performance & Resource Efficiency** | ✅ PASS | 10k line buffer limit, sub-100ms latency target, circular buffer prevents unbounded growth |
| **VI. Cross-Platform Compatibility** | ⚠️ DEVIATION (JUSTIFIED) | macOS-only for MVP per spec clarification; cross-platform deferred to post-MVP |

### Quality Gates

**Before Committing**:
- [ ] cargo clippy passes with no warnings
- [ ] cargo test passes (unit + integration)
- [ ] cargo fmt applied
- [ ] No unsafe code without documentation

**Before Merging**:
- [ ] All tests pass
- [ ] Code review completed
- [ ] Constitution compliance verified
- [ ] Platform compatibility: macOS tested

**Before Releasing**:
- [ ] Manual testing on macOS (MVP scope)
- [ ] Performance profiling: <100ms command latency verified
- [ ] cargo audit security check passed
- [ ] CHANGELOG updated

### Violations Requiring Justification

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| macOS-only scope (violates Principle VI) | MVP focus reduces implementation complexity and testing burden | Cross-platform testing would delay MVP; Tauri abstractions allow future expansion without architectural changes |

### Post-Design Re-Check
*(Completed after Phase 1)*

- [x] Final architecture aligns with component-based UI principle
  - Leptos components: Terminal, CommandInput, OutputDisplay, PromptIndicator, NotificationBar
  - Each component has single responsibility and clear interface
  - State managed via TerminalState context with reactive signals

- [x] IPC boundary uses strongly-typed Rust structs
  - CommandRequest, CommandResponse, OutputLine all use serde serialization
  - No untyped JSON; all types defined in data-model.md
  - Tauri commands receive/return concrete Rust types

- [x] Shell process management includes error handling and restart logic
  - ShellManager monitors process with background task
  - Crash detection via wait() on child process
  - Automatic restart on unexpected exit with notification
  - Signal handling for Ctrl+C via nix crate

- [x] Test strategy covers critical paths
  - Unit tests: HistoryBuffer circular logic, serde serialization
  - Integration tests: shell command execution, cancellation, crash recovery
  - Property-based tests: buffer wraparound with proptest
  - Manual E2E: cargo tauri dev for full flow testing

**Constitution Compliance Status**: ✅ PASS

All core principles satisfied:
- Rust-first with Tauri + Leptos stack
- Type-safe IPC with strongly-typed commands
- Component-based UI with Leptos signals
- Test coverage planned for critical paths
- Performance targets defined (<100ms latency)
- macOS-only deviation justified for MVP scope

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
# Tauri + Leptos Workspace Structure

# Frontend (Leptos CSR → WASM)
src/
├── main.rs                    # Entry point - mounts App component
├── app.rs                     # Root App component
├── components/
│   ├── terminal.rs            # Main terminal container component
│   ├── command_input.rs       # Input area with editing support
│   ├── output_display.rs      # Scrollable history renderer
│   ├── prompt_indicator.rs    # Shows cwd and ready state
│   └── notification_bar.rs    # Displays system notifications (truncation, restart)
└── models/
    ├── terminal_state.rs      # Shared state (history buffer, current command)
    └── output_line.rs         # Output line types (Command, Stdout, Stderr, Notification)

# Backend (Tauri native)
src-tauri/
├── src/
│   ├── main.rs               # Tauri app entry (calls run())
│   ├── lib.rs                # Tauri setup, command registration
│   ├── commands/
│   │   ├── shell.rs          # execute_command, cancel_command, get_cwd
│   │   └── history.rs        # get_history, clear_history (if needed)
│   ├── state/
│   │   ├── shell_manager.rs  # Shell process lifecycle, crash detection, restart
│   │   └── history_buffer.rs # Circular buffer (10k lines), truncation logic
│   ├── models/
│   │   ├── command.rs        # Command execution request/response types
│   │   └── output.rs         # OutputLine, OutputType (Stdout/Stderr) - shared with frontend
│   └── logging/
│       └── file_logger.rs    # Setup tracing to ~/.cepheus/terminal.log
└── tests/
    ├── integration/
    │   ├── shell_commands.rs # Test execute, cancel, crash recovery
    │   └── history_buffer.rs # Test circular buffer, truncation
    └── unit/
        └── models.rs         # Test serialization of IPC types

# Build Configuration
Cargo.toml                    # Workspace root (frontend crate)
src-tauri/Cargo.toml          # Backend crate with Tauri dependencies
Trunk.toml                    # Frontend build config (port 1420)
src-tauri/tauri.conf.json     # Tauri app config (window, CSP, IPC)
```

**Structure Decision**: Tauri workspace architecture with frontend (Leptos WASM) in `src/` and backend (Tauri native) in `src-tauri/`. This follows Tauri's standard workspace pattern as documented in CLAUDE.md. Frontend components communicate with backend via Tauri IPC commands. Shared types (OutputLine, Command) defined in backend and re-exported or duplicated with matching shape in frontend for serialization across WASM boundary.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| macOS-only (violates cross-platform principle) | MVP scope reduction | Cross-platform testing across macOS/Windows/Linux would delay MVP delivery; Tauri abstractions preserve future portability |

No other complexity violations. Design follows Tauri + Leptos conventions with minimal abstractions.

---

## Phase Summary

### Phase 0: Research (Completed)
- ✅ Resolved shell integration approach (std::process::Command vs PTY)
- ✅ Defined signal handling strategy (nix crate for SIGINT)
- ✅ Established testing strategy (unit + integration + property-based)
- ✅ Selected logging framework (tracing with file appender)
- ✅ Designed circular buffer implementation (VecDeque with manual capacity)
- ✅ Planned shell crash detection and recovery

**Output**: `research.md` with all technical decisions documented

### Phase 1: Design & Contracts (Completed)
- ✅ Defined data model with 6 core entities (OutputLine, HistoryBuffer, CommandRequest, etc.)
- ✅ Specified 5 Tauri IPC commands (execute_command, cancel_command, get_history, get_cwd, change_directory)
- ✅ Designed 7 Leptos components with reactive state management
- ✅ Created developer quickstart guide
- ✅ Updated agent context (CLAUDE.md)
- ✅ Re-validated constitution compliance

**Outputs**:
- `data-model.md` - Entity definitions and relationships
- `contracts/tauri-commands.md` - Backend IPC contract
- `contracts/component-interface.md` - Frontend component contract
- `quickstart.md` - Developer onboarding guide

### Next Steps: Phase 2 (Not part of /speckit.plan)

Run `/speckit.tasks` to generate dependency-ordered implementation tasks based on this plan. The tasks command will create `tasks.md` with concrete implementation steps.

---

## Implementation Readiness

**Ready to Proceed**: ✅ YES

All prerequisites satisfied:
- [x] Technical unknowns resolved
- [x] Architecture designed and documented
- [x] Contracts defined for IPC and components
- [x] Constitution compliance verified
- [x] Test strategy established
- [x] Developer documentation complete

**Recommended Next Command**: `/speckit.tasks`

This will generate the task breakdown for implementation based on the research and design completed in this plan.
