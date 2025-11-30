<!--
Sync Impact Report
==================
Version Change: (Initial) → 1.0.0
Modified Principles: N/A (Initial creation)
Added Sections: All sections created
Removed Sections: None
Templates Status:
  - .specify/templates/plan-template.md: ✅ Reviewed (Constitution Check section already references constitution file)
  - .specify/templates/spec-template.md: ✅ Reviewed (No constitution-specific constraints required)
  - .specify/templates/tasks-template.md: ✅ Reviewed (Phase structure aligns with test-driven principles)
Follow-up TODOs: None
-->

# Cepheus Constitution

## Core Principles

### I. Rust-First Development

All core functionality MUST be implemented in Rust. Tauri's architecture requires Rust for backend logic and native capabilities. Leptos provides type-safe reactive UI components compiled to WebAssembly.

**Rationale**: Rust ensures memory safety, prevents data races at compile time, and provides predictable performance for desktop applications. The Tauri + Leptos stack leverages Rust's strengths across frontend and backend.

### II. Type Safety & Compile-Time Guarantees

Code MUST leverage Rust's type system and borrow checker. Use strongly-typed APIs, avoid unsafe code unless absolutely necessary and documented, and prefer compile-time validation over runtime checks.

**Rationale**: Rust's ownership model prevents entire classes of bugs (null pointers, data races, buffer overflows) at compile time. This reduces runtime errors and improves application stability.

### III. Component-Based UI Architecture

UI MUST be structured as composable Leptos components following reactive programming patterns. Components should be small, focused, and reusable with clear props/signals interfaces.

**Rationale**: Component-based architecture enables code reuse, improves maintainability, and aligns with Leptos's reactive model. Clear boundaries between components reduce coupling and enable parallel development.

### IV. Test-Driven Development (RECOMMENDED)

Write tests before implementation when practical. All public APIs and critical user flows MUST have automated tests. Integration tests should verify Tauri command interactions and UI state management.

**Rationale**: Desktop applications require reliability across different OS environments. Tests provide confidence during refactoring and catch regressions early. Tauri's architecture makes components independently testable.

### V. Performance & Resource Efficiency

Desktop applications MUST be resource-conscious. Profile memory usage and startup time. Lazy-load heavy resources. Prefer streaming over batch loading for large datasets.

**Rationale**: Users expect desktop apps to be responsive and not drain system resources. Rust's zero-cost abstractions enable performance optimization without sacrificing safety. Leptos's fine-grained reactivity minimizes unnecessary re-renders.

### VI. Cross-Platform Compatibility

Code MUST work on macOS, Windows, and Linux unless platform-specific behavior is explicitly documented and justified. Use Tauri's platform-agnostic APIs. Test on all target platforms before release.

**Rationale**: Tauri enables true cross-platform desktop apps. Platform-specific code creates maintenance burden and fragments the user experience. Abstracting platform differences early prevents technical debt.

## Development Standards

### Code Organization

- **Frontend**: Leptos components in `src/components/`, pages in `src/pages/`, utilities in `src/utils/`
- **Backend**: Tauri commands in `src-tauri/src/commands/`, state management in `src-tauri/src/state/`, models in `src-tauri/src/models/`
- **Shared Types**: Shared Rust types between frontend and backend via workspace members or feature flags
- **Tests**: Unit tests alongside source files, integration tests in `src-tauri/tests/`, E2E tests for critical user flows

**Rationale**: Clear separation between frontend (Leptos/WASM) and backend (Tauri/native) simplifies mental model and enables independent testing. Workspace structure supports code sharing while maintaining module boundaries.

### Testing Requirements

- **Unit Tests**: Required for business logic, data transformations, state management
- **Integration Tests**: Required for Tauri commands, IPC boundaries, file system operations
- **Component Tests**: Recommended for complex UI components with non-trivial state
- **Platform Tests**: Required before releases to verify cross-platform compatibility

**Rationale**: Different test types validate different concerns. Unit tests verify logic, integration tests verify boundaries, platform tests verify deployment targets.

### Security & Privacy

- **Input Validation**: All user input and IPC messages MUST be validated on the Tauri backend
- **File System Access**: Use Tauri's permission system and scope APIs to restrict file access
- **Secure Communication**: Validate all data crossing the IPC boundary between frontend and backend
- **Dependency Audits**: Run `cargo audit` regularly to detect security vulnerabilities in dependencies

**Rationale**: Desktop apps have privileged file system and OS access. Tauri's architecture requires securing the IPC boundary to prevent malicious frontend code from compromising the system.

## Quality Gates

### Before Committing

- [ ] Code compiles without warnings (`cargo clippy` passes)
- [ ] Tests pass locally (`cargo test`, `cargo test --manifest-path src-tauri/Cargo.toml`)
- [ ] Code formatted with `cargo fmt`
- [ ] No unsafe code added without documentation and justification

### Before Merging

- [ ] All tests pass in CI
- [ ] Code reviewed by at least one other developer
- [ ] Constitution compliance verified (principles not violated without justification)
- [ ] Platform compatibility verified for changed areas

### Before Releasing

- [ ] Manual testing on macOS, Windows, and Linux
- [ ] Performance profiling shows no regressions
- [ ] Security audit completed (`cargo audit`)
- [ ] User-facing changes documented in CHANGELOG

**Rationale**: Progressive quality gates catch issues at appropriate stages. Local checks provide fast feedback, CI catches environment-specific issues, release checks ensure production readiness.

## Governance

### Amendment Process

1. Propose constitution change via issue or discussion
2. Document rationale and impact on existing code/templates
3. Update constitution with version bump following semantic versioning
4. Update all affected templates and documentation
5. Communicate changes to all contributors

### Version Semantics

- **MAJOR (X.0.0)**: Removal of principles, incompatible governance changes, architectural shifts
- **MINOR (0.X.0)**: New principles added, expanded guidance, new mandatory sections
- **PATCH (0.0.X)**: Clarifications, wording improvements, typo fixes, non-semantic refinements

### Compliance

- All pull requests MUST verify compliance with this constitution
- Violations require explicit justification documented in PR description or code comments
- Complexity beyond constitution guidelines MUST be justified with rationale
- Constitution supersedes all other practices in case of conflict

### Runtime Guidance

For day-to-day development guidance and workflow instructions, refer to `.specify/templates/commands/*.md` for agent-specific execution workflows and `.specify/templates/*.md` for artifact templates.

**Version**: 1.0.0 | **Ratified**: 2025-11-29 | **Last Amended**: 2025-11-29
