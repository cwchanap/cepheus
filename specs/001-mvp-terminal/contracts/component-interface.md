# Component Interface Contract

**Feature**: 001-mvp-terminal
**Date**: 2025-11-30
**Framework**: Leptos 0.7 (Reactive Components)

## Overview

This document defines the Leptos component interfaces for the MVP terminal frontend. All components use Leptos signals for reactivity and communicate via props and context.

---

## Component Hierarchy

```
App
└── Terminal
    ├── PromptIndicator
    ├── OutputDisplay
    ├── CommandInput
    └── NotificationBar
```

---

## Components

### 1. App

Root application component that provides global context and mounts the Terminal.

**Location**: `src/app.rs`

**Signature**:
```rust
#[component]
pub fn App() -> impl IntoView
```

**Props**: None (root component)

**Context Provided**:
```rust
provide_context(TerminalState::new());
```

**Responsibilities**:
- Initialize TerminalState
- Provide context to child components
- Mount Terminal component
- Set up global styles/layout

**Template**:
```rust
view! {
    <div class="app">
        <Terminal />
    </div>
}
```

---

### 2. Terminal

Main terminal container that orchestrates all sub-components.

**Location**: `src/components/terminal.rs`

**Signature**:
```rust
#[component]
pub fn Terminal() -> impl IntoView
```

**Props**: None (uses context)

**Context Consumed**:
```rust
let state = use_context::<TerminalState>().expect("TerminalState context missing");
```

**Responsibilities**:
- Set up Tauri event listeners (output-line, shell-notification)
- Fetch initial history on mount
- Coordinate child components
- Handle keyboard events (Ctrl+C bubbles from CommandInput)

**Reactive Effects**:
```rust
create_effect(move |_| {
    // Listen for output-line events
    listen("output-line", move |event| {
        let line: OutputLine = serde_wasm_bindgen::from_value(event.payload).unwrap();
        state.history.update(|h| h.push(line));
    });
});
```

**Template**:
```rust
view! {
    <div class="terminal-container">
        <NotificationBar />
        <OutputDisplay />
        <div class="input-row">
            <PromptIndicator />
            <CommandInput />
        </div>
    </div>
}
```

---

### 3. PromptIndicator

Displays the shell prompt with current working directory.

**Location**: `src/components/prompt_indicator.rs`

**Signature**:
```rust
#[component]
pub fn PromptIndicator() -> impl IntoView
```

**Props**: None (uses context)

**Context Consumed**:
```rust
let state = use_context::<TerminalState>().unwrap();
```

**Reactive Dependencies**:
- `state.cwd` - Updates when directory changes
- `state.is_busy` - Changes prompt appearance when command running

**Responsibilities**:
- Display current working directory (abbreviated)
- Show prompt symbol (e.g., "$" or ">" )
- Visual indicator when command is executing (e.g., spinner)

**Template**:
```rust
view! {
    <div class="prompt-indicator">
        <span class="cwd">{move || format_cwd(state.cwd.get())}</span>
        <span class="symbol">
            {move || if state.is_busy.get() { "⏳" } else { "$" }}
        </span>
    </div>
}
```

**Styling Behavior**:
- Abbreviate home directory as `~`
- Abbreviate long paths (e.g., `.../parent/current`)
- Change color when busy

---

### 4. OutputDisplay

Scrollable display of terminal history.

**Location**: `src/components/output_display.rs`

**Signature**:
```rust
#[component]
pub fn OutputDisplay() -> impl IntoView
```

**Props**: None (uses context)

**Context Consumed**:
```rust
let state = use_context::<TerminalState>().unwrap();
```

**Reactive Dependencies**:
- `state.history` - Re-renders when new output added

**Responsibilities**:
- Render all OutputLine entries
- Apply appropriate styling per line type (Command/Stdout/Stderr/Notification)
- Auto-scroll to bottom when new output arrives
- Support manual scroll (user can scroll up)

**Template**:
```rust
view! {
    <div class="output-display" id="output-container">
        <For
            each=move || state.history.get()
            key=|line| line.timestamp()
            children=|line| view! { <OutputLineView line=line /> }
        />
    </div>
}
```

**Auto-Scroll Logic**:
```rust
create_effect(move |_| {
    let _ = state.history.get(); // Track changes

    if let Some(container) = document()
        .get_element_by_id("output-container") {
        container.set_scroll_top(container.scroll_height());
    }
});
```

**Styling Rules**:
- Command lines: Bold text, different color
- Stdout: Normal text
- Stderr: Red text
- Notification: Yellow/orange background, centered

---

### 5. OutputLineView (Sub-component)

Renders a single OutputLine with appropriate styling.

**Location**: `src/components/output_display.rs`

**Signature**:
```rust
#[component]
fn OutputLineView(line: OutputLine) -> impl IntoView
```

**Props**:
```rust
#[prop]
line: OutputLine  // The line to render
```

**Responsibilities**:
- Match on OutputLine type
- Apply CSS class based on type
- Render text content

**Template**:
```rust
view! {
    <div class={move || line_class(&line)}>
        {line_text(&line)}
    </div>
}

fn line_class(line: &OutputLine) -> &'static str {
    match line {
        OutputLine::Command { .. } => "line-command",
        OutputLine::Stdout { .. } => "line-stdout",
        OutputLine::Stderr { .. } => "line-stderr",
        OutputLine::Notification { .. } => "line-notification",
    }
}

fn line_text(line: &OutputLine) -> String {
    match line {
        OutputLine::Command { text, .. } => format!("$ {}", text),
        OutputLine::Stdout { text, .. } => text.clone(),
        OutputLine::Stderr { text, .. } => text.clone(),
        OutputLine::Notification { message, .. } => format!("⚠️  {}", message),
    }
}
```

---

### 6. CommandInput

Text input for entering shell commands.

**Location**: `src/components/command_input.rs`

**Signature**:
```rust
#[component]
pub fn CommandInput() -> impl IntoView
```

**Props**: None (uses context)

**Context Consumed**:
```rust
let state = use_context::<TerminalState>().unwrap();
```

**Reactive Dependencies**:
- `state.current_input` - Bidirectional binding
- `state.is_busy` - Disable input when command running

**Responsibilities**:
- Capture user text input
- Handle Enter key → submit command
- Handle Ctrl+C → cancel running command
- Handle Backspace → edit current input
- Clear input after submission

**Event Handlers**:
```rust
// on:input - Update state.current_input
on:input=move |ev| {
    state.current_input.set(event_target_value(&ev));
}

// on:keydown - Handle special keys
on:keydown=move |ev| {
    if ev.key() == "Enter" {
        ev.prevent_default();
        spawn_local(async move {
            submit_command(&state).await;
        });
    } else if ev.ctrl_key() && ev.key() == "c" {
        ev.prevent_default();
        spawn_local(async move {
            cancel_running_command().await;
        });
    }
}
```

**Template**:
```rust
view! {
    <input
        type="text"
        class="command-input"
        prop:value=move || state.current_input.get()
        on:input=handle_input
        on:keydown=handle_keydown
        prop:disabled=move || state.is_busy.get()
        placeholder="Enter command..."
    />
}
```

**Command Submission**:
```rust
async fn submit_command(state: &TerminalState) {
    let cmd = state.current_input.get();
    if cmd.trim().is_empty() {
        return;
    }

    // Add command to history immediately (optimistic)
    let cmd_line = OutputLine::Command {
        text: cmd.clone(),
        timestamp: current_timestamp_ms(),
    };
    state.history.update(|h| h.push(cmd_line));

    state.current_input.set(String::new());
    state.is_busy.set(true);

    // Call Tauri IPC
    let args = serde_wasm_bindgen::to_value(&CommandRequest {
        command: cmd,
        cwd: None,
    }).unwrap();

    match invoke("execute_command", args).await {
        Ok(_) => {
            state.is_busy.set(false);
        }
        Err(e) => {
            let err_line = OutputLine::Stderr {
                text: format!("Error: {:?}", e),
                timestamp: current_timestamp_ms(),
            };
            state.history.update(|h| h.push(err_line));
            state.is_busy.set(false);
        }
    }
}
```

---

### 7. NotificationBar

Displays transient system notifications (non-modal).

**Location**: `src/components/notification_bar.rs`

**Signature**:
```rust
#[component]
pub fn NotificationBar() -> impl IntoView
```

**Props**: None (uses context)

**Context Consumed**:
```rust
let state = use_context::<TerminalState>().unwrap();
```

**Reactive Dependencies**:
- `state.notification` - Shows/hides based on value

**Responsibilities**:
- Display notification message when present
- Auto-dismiss after timeout (3 seconds)
- Style based on notification level (Info/Warning/Error)

**Template**:
```rust
view! {
    <Show when=move || state.notification.get().is_some()>
        <div class="notification-bar">
            {move || state.notification.get().unwrap_or_default()}
        </div>
    </Show>
}
```

**Auto-Dismiss**:
```rust
create_effect(move |_| {
    if state.notification.get().is_some() {
        set_timeout(
            move || state.notification.set(None),
            Duration::from_secs(3)
        );
    }
});
```

---

## State Management

### TerminalState (Context)

Shared reactive state accessible to all components via `use_context()`.

**Definition**: See data-model.md for full definition

**Key Signals**:
- `current_input: RwSignal<String>` - CommandInput binds to this
- `history: RwSignal<Vec<OutputLine>>` - OutputDisplay renders this
- `cwd: RwSignal<String>` - PromptIndicator displays this
- `is_busy: RwSignal<bool>` - Multiple components react to this
- `notification: RwSignal<Option<String>>` - NotificationBar shows this

**Update Patterns**:
```rust
// Append to history
state.history.update(|h| h.push(new_line));

// Replace entire history (from get_history)
state.history.set(backend_history);

// Set scalar value
state.is_busy.set(true);
```

---

## Inter-Component Communication

**Pattern**: Context + Signals (no direct component-to-component calls)

**Example Flow**:
1. User types in `CommandInput` → updates `state.current_input`
2. User presses Enter in `CommandInput` → calls IPC, updates `state.is_busy`
3. `PromptIndicator` reacts to `state.is_busy` change → shows spinner
4. Backend emits `output-line` event → `Terminal` listener updates `state.history`
5. `OutputDisplay` reacts to `state.history` change → renders new line

**Benefits**:
- Loose coupling between components
- Testable in isolation
- Reactive updates automatic via Leptos signals

---

## Styling

**CSS Classes**:
```css
/* Terminal container */
.terminal-container { /* Flexbox layout */ }

/* Output display */
.output-display { /* Scrollable area */ }
.line-command { /* Bold, distinct color */ }
.line-stdout { /* Normal text */ }
.line-stderr { /* Red text */ }
.line-notification { /* Warning banner style */ }

/* Input area */
.input-row { /* Flexbox for prompt + input */ }
.prompt-indicator { /* Inline with input */ }
.command-input { /* Full-width text input */ }

/* Notifications */
.notification-bar { /* Fixed position, top */ }
```

**Responsive Behavior**:
- Terminal takes full window height
- Output display scrolls independently
- Input row fixed at bottom

---

## Accessibility

**Keyboard Navigation**:
- Input always focused (auto-focus on mount)
- Enter submits command
- Ctrl+C cancels command
- Arrow keys for text editing (native browser behavior)

**Screen Reader Support**:
- Use semantic HTML (`<input>`, `<div>`)
- ARIA labels for prompt indicator
- Output display is live region (aria-live="polite")

**Future Enhancements** (post-MVP):
- Command history navigation (Up/Down arrows)
- Tab completion
- Focus management for scrollable output

---

## Performance Considerations

**Rendering Optimization**:
- Use `<For>` with keys for efficient list rendering
- Avoid re-rendering entire history on each append
- Auto-scroll only when near bottom (avoid jump during manual scroll)

**Memory Management**:
- History capped at 10,000 lines by backend
- Frontend stores same data (no additional buffering)
- Old entries automatically evicted by backend circular buffer

**Event Handling**:
- Debounce input events if typing becomes laggy (profile first)
- Use spawn_local for async IPC calls (non-blocking)

---

## Testing

**Component Tests** (Leptos testing utils):
```rust
#[test]
fn test_command_input_submission() {
    let state = TerminalState::new();
    provide_context(state.clone());

    let input = mount(|| view! { <CommandInput /> });

    // Simulate typing
    state.current_input.set("echo test".to_string());

    // Simulate Enter keypress
    // ... (Leptos test event dispatch)

    assert_eq!(state.current_input.get(), ""); // Input cleared
    assert!(state.is_busy.get()); // Busy while executing
}
```

**Integration Tests**:
- Manual E2E testing via `cargo tauri dev`
- Test keyboard shortcuts
- Test output rendering with long text
- Test truncation notification display

---

## Summary

| Component | Inputs | Outputs | Side Effects |
|-----------|--------|---------|--------------|
| App | None | Context | Provides TerminalState |
| Terminal | Context | None | Sets up event listeners |
| PromptIndicator | state.cwd, state.is_busy | Visual | None |
| OutputDisplay | state.history | Visual | Auto-scrolls |
| CommandInput | state.current_input, state.is_busy | IPC calls | Updates state, calls backend |
| NotificationBar | state.notification | Visual | Auto-dismisses |
