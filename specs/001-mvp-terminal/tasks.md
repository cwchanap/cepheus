# Tasks: MVP Terminal Application

**Input**: Design documents from `/specs/001-mvp-terminal/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [ ] T001 Add dependencies to src-tauri/Cargo.toml (tokio, nix, tracing, tracing-subscriber, tracing-appender, dirs, serde)
- [ ] T002 [P] Setup logging infrastructure in src-tauri/src/logging/file_logger.rs
- [ ] T003 [P] Create project structure directories (src/components/, src/models/, src-tauri/src/commands/, src-tauri/src/state/, src-tauri/src/models/)
- [ ] T004 Initialize logging in src-tauri/src/main.rs before Tauri app starts

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

**TDD Note**: Tests written FIRST (should fail), then implementation makes them pass

### Foundational Tests (Write First - Should FAIL)

- [ ] T005 [P] Write unit test for OutputLine serde serialization in src-tauri/src/models/output.rs
- [ ] T006 [P] Write unit test for CommandRequest/CommandResponse serialization in src-tauri/src/models/command.rs
- [ ] T007 Write unit tests for HistoryBuffer capacity, wraparound, and truncation warning in src-tauri/tests/unit/history_buffer.rs
- [ ] T008 Write property-based test for HistoryBuffer wraparound using proptest in src-tauri/tests/unit/history_buffer.rs

### Foundational Implementation (Make Tests Pass)

- [ ] T009 [P] Define OutputLine enum in src-tauri/src/models/output.rs with serde serialization (make T005 pass)
- [ ] T010 [P] Define CommandRequest and CommandResponse structs in src-tauri/src/models/command.rs (make T006 pass)
- [ ] T011 Create HistoryBuffer struct with circular buffer logic in src-tauri/src/state/history_buffer.rs (make T007-T008 pass)
- [ ] T012 Implement ShellState struct for process management in src-tauri/src/state/shell_manager.rs
- [ ] T013 Create ShellManager struct combining ShellState and HistoryBuffer in src-tauri/src/state/shell_manager.rs
- [ ] T014 Initialize ShellManager in Tauri state in src-tauri/src/lib.rs
- [ ] T015 [P] Mirror OutputLine type in frontend at src/models/output_line.rs with serde compatibility
- [ ] T016 Create TerminalState context with reactive signals in src/models/terminal_state.rs

**Checkpoint**: Foundation ready - all foundational tests passing - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Execute Commands (Priority: P1) 🎯 MVP

**Goal**: Enable users to type commands and see output - the core terminal functionality

**Independent Test**: Type "echo hello" and press Enter → verify "hello" appears in output

**TDD Note**: Write integration tests FIRST, watch them fail, then implement to make them pass

### Tests for User Story 1 (Write First - Should FAIL)

- [ ] T017 [P] [US1] Write integration test for execute_command with echo in src-tauri/tests/integration/shell_commands.rs
- [ ] T018 [P] [US1] Write integration test for cancel_command with sleep in src-tauri/tests/integration/shell_commands.rs
- [ ] T019 [P] [US1] Write integration test for shell crash detection and restart in src-tauri/tests/integration/shell_commands.rs

### Implementation for User Story 1 (Make Tests Pass)

- [ ] T020 [US1] Implement execute_command Tauri command in src-tauri/src/commands/shell.rs (make T017 pass)
- [ ] T021 [US1] Add process spawning logic with stdout/stderr capture in execute_command
- [ ] T022 [US1] Implement output streaming to HistoryBuffer in execute_command
- [ ] T023 [US1] Add output-line event emission from execute_command
- [ ] T024 [US1] Register execute_command in Tauri invoke_handler in src-tauri/src/lib.rs
- [ ] T025 [US1] Implement shell process monitoring for crash detection in src-tauri/src/state/shell_manager.rs (make T019 pass)
- [ ] T026 [US1] Add shell restart logic with notification emission in src-tauri/src/state/shell_manager.rs
- [ ] T027 [US1] Add tracing logs for command execution lifecycle in execute_command

**Checkpoint**: Backend can execute commands - all US1 tests passing - ready for frontend integration

---

## Phase 4: User Story 2 - Command Input (Priority: P1) 🎯 MVP

**Goal**: Provide a clear input area where users can type commands

**Independent Test**: Open terminal → verify input area is visible and accepts text → verify backspace works → verify Enter clears input

**Note**: Frontend component testing is optional for Leptos/WASM in MVP - focus on integration via manual testing

### Implementation for User Story 2

- [ ] T028 [P] [US2] Create CommandInput component in src/components/command_input.rs
- [ ] T029 [US2] Add text input with value binding to TerminalState.current_input in CommandInput
- [ ] T030 [US2] Implement onKeyDown handler for Enter key in CommandInput
- [ ] T031 [US2] Implement command submission logic calling execute_command IPC in CommandInput
- [ ] T032 [US2] Add input clearing after submission in CommandInput
- [ ] T033 [US2] Add disabled state when is_busy is true in CommandInput
- [ ] T034 [US2] Add optimistic command line insertion to history in CommandInput

**Checkpoint**: Users can type and submit commands through the UI

---

## Phase 5: User Story 3 - Output Display (Priority: P1) 🎯 MVP

**Goal**: Display scrollable history of commands and outputs

**Independent Test**: Execute multiple commands → verify all outputs visible → verify scrolling works → verify stdout/stderr are distinguishable

### Implementation for User Story 3

- [ ] T035 [P] [US3] Create OutputDisplay component in src/components/output_display.rs
- [ ] T036 [P] [US3] Create OutputLineView sub-component in src/components/output_display.rs
- [ ] T037 [US3] Implement line rendering with type-based styling in OutputLineView
- [ ] T038 [US3] Add For loop rendering history from TerminalState in OutputDisplay
- [ ] T039 [US3] Implement auto-scroll to bottom logic in OutputDisplay
- [ ] T040 [US3] Add CSS classes for line types (line-command, line-stdout, line-stderr, line-notification)
- [ ] T041 [US3] Implement get_history Tauri command in src-tauri/src/commands/shell.rs
- [ ] T042 [US3] Register get_history in Tauri invoke_handler in src-tauri/src/lib.rs
- [ ] T043 [US3] Call get_history on component mount to fetch initial history in OutputDisplay

**Checkpoint**: Terminal displays command output with proper formatting

---

## Phase 6: User Story 4 - Visual Prompt (Priority: P2)

**Goal**: Show a prompt indicator so users know the terminal is ready

**Independent Test**: Open terminal → verify prompt symbol ($ or >) is visible → run command → verify prompt changes during execution

### Implementation for User Story 4

- [ ] T044 [P] [US4] Create PromptIndicator component in src/components/prompt_indicator.rs
- [ ] T045 [US4] Implement cwd display with home directory abbreviation in PromptIndicator
- [ ] T046 [US4] Add prompt symbol rendering ($ when idle, ⏳ when busy) in PromptIndicator
- [ ] T047 [US4] Implement get_cwd Tauri command in src-tauri/src/commands/shell.rs
- [ ] T048 [US4] Register get_cwd in Tauri invoke_handler in src-tauri/src/lib.rs
- [ ] T049 [US4] Call get_cwd on mount to initialize cwd in TerminalState
- [ ] T050 [US4] Add CSS styling for prompt indicator

**Checkpoint**: Terminal shows professional prompt with working directory

---

## Phase 7: Cross-Cutting Features

**Purpose**: Features that enhance multiple user stories

### Command Cancellation (Ctrl+C)

- [ ] T051 [P] Implement cancel_command Tauri command using nix SIGINT in src-tauri/src/commands/shell.rs
- [ ] T052 [P] Register cancel_command in Tauri invoke_handler in src-tauri/src/lib.rs
- [ ] T053 Add Ctrl+C keydown handler in CommandInput calling cancel_command
- [ ] T054 Add process termination handling in shell monitor task

### History Buffer Management

- [ ] T055 [P] Add truncation warning insertion logic in HistoryBuffer.push()
- [ ] T056 Add clear_history method to HistoryBuffer (if needed for future commands)

### Notifications

- [ ] T057 [P] Create NotificationBar component in src/components/notification_bar.rs
- [ ] T058 Implement notification display with auto-dismiss in NotificationBar
- [ ] T059 Add shell-notification event listener in Terminal component
- [ ] T060 Add CSS styling for notification bar

### Directory Management

- [ ] T061 [P] Implement change_directory Tauri command in src-tauri/src/commands/shell.rs
- [ ] T062 [P] Register change_directory in Tauri invoke_handler in src-tauri/src/lib.rs
- [ ] T063 Update ShellState.cwd when change_directory succeeds

---

## Phase 8: Integration & Assembly

**Purpose**: Wire all components together

- [ ] T064 Create Terminal container component in src/components/terminal.rs
- [ ] T065 Add output-line event listener in Terminal component
- [ ] T066 Assemble component hierarchy in Terminal (PromptIndicator + OutputDisplay + CommandInput + NotificationBar)
- [ ] T067 Create components module exports in src/components/mod.rs
- [ ] T068 Update App component to mount Terminal in src/app.rs
- [ ] T069 Provide TerminalState context in App component

---

## Phase 9: Manual Testing & Refinement

**Purpose**: Manual validation, refinement, and quality assurance

**Note**: Unit and integration tests already written in Phases 2-3 following TDD. This phase focuses on manual E2E testing and polish.

### Manual Testing

- [ ] T070 Test basic command execution (ls, pwd, echo) via cargo tauri dev
- [ ] T071 Test long-running command cancellation (sleep 10 + Ctrl+C)
- [ ] T072 Test output truncation at 10,000 lines
- [ ] T073 Test shell crash recovery (sh -c 'kill -9 $$')
- [ ] T074 Test commands with stderr output
- [ ] T075 Verify sub-100ms latency for simple commands

### Refinement

- [ ] T076 [P] Add CSS styling for terminal container layout
- [ ] T077 [P] Ensure terminal takes full window height with fixed input row
- [ ] T078 [P] Verify auto-focus on CommandInput on mount
- [ ] T079 Add error handling for all IPC call failures in frontend
- [ ] T080 Verify logging to ~/.cepheus/terminal.log works correctly

### Documentation

- [ ] T081 [P] Verify quickstart.md matches implemented structure
- [ ] T082 [P] Update CLAUDE.md if any patterns changed during implementation
- [ ] T083 Run all quality gates (cargo fmt, cargo clippy, cargo test)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phases 3-6)**: All depend on Foundational phase completion
  - US1 (Execute Commands): Independent after Foundation
  - US2 (Command Input): Independent after Foundation
  - US3 (Output Display): Independent after Foundation
  - US4 (Visual Prompt): Independent after Foundation
- **Cross-Cutting (Phase 7)**: Depends on relevant user stories being complete
- **Integration (Phase 8)**: Depends on US1, US2, US3 being complete (US4 optional)
- **Polish (Phase 9)**: Depends on Integration being complete

### User Story Dependencies

- **US1 (Execute Commands - P1)**: Foundation only → Core terminal functionality
- **US2 (Command Input - P1)**: Foundation only → Requires US1 for IPC but can be implemented in parallel
- **US3 (Output Display - P1)**: Foundation only → Requires US1 for data but can be implemented in parallel
- **US4 (Visual Prompt - P2)**: Foundation only → Enhancement, not blocking MVP

### Within Each Phase

- **Setup** (T001-T004): T001 blocks T004; T002-T003 can be parallel
- **Foundational** (T005-T016):
  - Tests T005-T008 must be written FIRST (all should fail initially)
  - Implementation T009-T016: T009-T010 parallel, T011 sequential, T012-T016 sequential
- **US1** (T017-T027):
  - Tests T017-T019 written FIRST (should fail)
  - Implementation T020-T027 sequential, make tests pass
- **US2** (T028-T034): Mostly sequential (component logic)
- **US3** (T035-T043): T035-T036 parallel (components), T037-T043 sequential (integration)
- **US4** (T044-T050): Mostly sequential
- **Cross-cutting** (T051-T063): Most tasks marked [P] can run in parallel within subsections
- **Integration** (T064-T069): Sequential (assembly order matters)
- **Manual Testing & Refinement** (T070-T083): Manual tests sequential; refinement tasks parallel within subsections

### Critical Path (MVP)

1. **Phase 1: Setup** (T001-T004)
2. **Phase 2: Foundational** (T005-T016) ← BLOCKS EVERYTHING (includes TDD tests)
3. **Phase 3: US1 - Execute Commands** (T017-T027) ← MVP Core (includes TDD tests)
4. **Phase 4: US2 - Command Input** (T028-T034) ← MVP Core
5. **Phase 5: US3 - Output Display** (T035-T043) ← MVP Core
6. **Phase 7: Command Cancellation** (T051-T054) ← MVP Core
7. **Phase 8: Integration** (T064-T069) ← MVP Assembly
8. **Phase 9: Manual Testing** (T070-T075) ← MVP Validation

**MVP Scope**: Phases 1-2-3-4-5 + Command Cancellation + Phase 8 + Manual Testing = ~50 tasks (includes TDD tests)

---

## Parallel Opportunities

### Phase 2 Foundation - Test Writing (Parallel):

**All TDD tests can be written simultaneously**:
```
T005: OutputLine serialization test
T006: CommandRequest/Response serialization test
T007: HistoryBuffer unit tests
T008: HistoryBuffer property-based tests
```

### After Foundation Complete (Phase 2 done):

**Parallel Stream A** (Backend - Execute Commands with TDD):
```
T017-T019: Write integration tests FIRST (fail)
T020-T027: Implement command execution backend (make tests pass)
```

**Parallel Stream B** (Frontend - Input):
```
T028-T034: Implement command input UI
```

**Parallel Stream C** (Frontend - Output):
```
T035-T043: Implement output display UI
```

**Parallel Stream D** (Frontend - Prompt):
```
T044-T050: Implement prompt indicator UI
```

### During Cross-Cutting (Phase 7):

**Parallel**:
```
T051-T052: Implement cancel_command backend
T055: Add buffer truncation warning
T057: Create notification component
T061-T062: Implement change_directory backend
```

### During Refinement (Phase 9):

**Parallel**:
```
T076-T078: All styling/refinement tasks
T081-T082: All documentation tasks
```

---

## Implementation Strategy

### MVP First (Core Terminal Only)

**Deliverable**: Users can execute commands and see output

1. **Foundation** (Phase 1-2): Setup + Core Infrastructure **WITH TDD TESTS**
   - Write tests FIRST (T005-T008) - they should fail
   - Implement to make tests pass (T009-T016)
2. **MVP Core** (Phase 3-5): Execute + Input + Output (US1-US3)
   - US1 includes TDD tests (T017-T019) written before implementation
3. **MVP Polish** (Partial Phase 7-8): Cancel + Integration
4. **Validate**: Manual tests with basic commands (echo, ls, pwd)
5. **Deploy**: Working terminal application

**Task Count**: ~50 tasks (includes TDD tests as part of implementation)
**Estimated Effort**: 2-3 days for experienced Rust/Tauri developer

### Incremental Delivery

1. **Foundation Ready** (Phase 1-2) → Infrastructure complete
2. **Command Execution** (Phase 3 + US1) → Backend can run commands
3. **Basic UI** (Phase 4-5 + US2-US3) → Full terminal interaction
4. **Enhanced UX** (Phase 6 + US4) → Professional prompt
5. **Full Featured** (Phase 7) → Cancellation, notifications, etc.

### Parallel Team Strategy

With 2-3 developers after Foundation (Phase 2) is complete:

- **Developer A**: Backend focus (US1 execution → cancellation → directory commands)
- **Developer B**: Frontend focus (US2 input → US3 output → integration)
- **Developer C**: Enhancement focus (US4 prompt → notifications → testing)

---

## Notes

- **TDD Compliance**: Tests written FIRST in Phases 2-3, following Constitution Principle IV
- **Test-First Workflow**: Write test → watch it fail → implement → watch it pass → refactor
- **[P] marker**: Tasks that can run in parallel (different files, no blocking dependencies)
- **[Story] label**: Maps task to user story for traceability (US1, US2, US3, US4)
- **File paths**: All tasks include exact file paths for clarity
- **MVP boundary**: Phases 1-5 + Command Cancellation + Integration = Minimum viable product
- **Checkpoint validation**: Each phase should be testable independently before moving forward
- **Constitution compliance**: Code quality gates (clippy, fmt, tests) enforced in Phase 9
- **Performance target**: Verify sub-100ms command latency in T075

---

## Summary

- **Total Tasks**: 83 tasks (renumbered to follow TDD principles)
- **MVP Tasks**: ~50 tasks (Phases 1-5 + Command Cancellation + Integration + Manual Testing)
- **TDD Tests Included**: 7 test tasks (T005-T008 foundational, T017-T019 for US1)
- **User Story Breakdown**:
  - US1 (Execute Commands): 11 tasks (includes 3 TDD tests written first)
  - US2 (Command Input): 7 tasks
  - US3 (Output Display): 9 tasks
  - US4 (Visual Prompt): 7 tasks
  - Cross-Cutting: 13 tasks
  - Integration: 6 tasks
  - Manual Testing & Refinement: 14 tasks
  - Foundation: 12 tasks (includes 4 TDD tests written first)
  - Setup: 4 tasks

- **Parallel Opportunities**:
  - Setup phase: 2 parallel tasks (T002-T003)
  - Foundation TDD tests: 4 parallel tasks (T005-T008)
  - Foundation implementation: 2 parallel tasks (T009-T010)
  - US1 TDD tests: 3 parallel tasks (T017-T019)
  - User stories can be worked in parallel after foundation
  - Refinement phase: 5+ parallel tasks

- **Independent Test Criteria**:
  - US1: Execute "echo hello" → see "hello" output (automated test in T017)
  - US2: Type in input → verify text appears → press Enter → input clears
  - US3: Execute multiple commands → verify all visible → verify scrolling
  - US4: Open terminal → verify prompt visible → run command → verify prompt changes

- **TDD Workflow**:
  - Phase 2: Write T005-T008 (fail) → Implement T009-T016 (pass)
  - Phase 3: Write T017-T019 (fail) → Implement T020-T027 (pass)
  - Constitution Principle IV compliance: ✅ Tests written BEFORE implementation

- **Suggested MVP Scope**:
  - **Minimum**: Phases 1-5 + Command Cancellation + Integration (~50 tasks, includes TDD)
  - **Recommended**: Add Visual Prompt (US4) + Notifications for better UX (~57 tasks)
  - **Full Feature**: All 83 tasks for complete specification compliance
