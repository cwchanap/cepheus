use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;

use crate::models::{OutputLine, TerminalState};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

/// Request structure for `execute_command` IPC
#[derive(Serialize, Deserialize)]
struct ExecuteCommandArgs {
    command: String,
    cwd: Option<String>,
}

/// Response structure from `execute_command` IPC
#[derive(Serialize, Deserialize, Debug)]
#[allow(dead_code)]
struct CommandResponse {
    success: bool,
    exit_code: Option<i32>,
    error: Option<String>,
}

/// Get current timestamp in milliseconds
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn current_timestamp_ms() -> u64 {
    js_sys::Date::now() as u64
}

/// Text input for entering shell commands.
#[component]
pub fn CommandInput() -> impl IntoView {
    let state = use_context::<TerminalState>().expect("TerminalState context missing");

    // Create a node reference for the input element
    let input_ref = NodeRef::<leptos::html::Input>::new();

    // Auto-focus the input on mount
    Effect::new(move |_| {
        if let Some(input) = input_ref.get() {
            let html_input: &HtmlInputElement = &input;
            let _ = html_input.focus();
        }
    });

    // Handle input changes
    let on_input = move |ev: leptos::ev::Event| {
        let value = event_target_value(&ev);
        state.current_input.set(value);
    };

    // Handle key down events
    let on_keydown = move |ev: leptos::ev::KeyboardEvent| {
        let key = ev.key();

        if key == "Enter" {
            ev.prevent_default();
            submit_command(state);
        } else if ev.ctrl_key() && key == "c" {
            ev.prevent_default();
            cancel_command(state);
        }
    };

    view! {
        <input
            type="text"
            class="command-input"
            node_ref=input_ref
            prop:value=move || state.current_input.get()
            on:input=on_input
            on:keydown=on_keydown
            prop:disabled=move || state.is_input_disabled()
            placeholder=move || {
                if state.listener_failed.get() {
                    "Terminal unavailable - connection failed"
                } else {
                    "Enter command..."
                }
            }
        />
    }
}

/// Submit the current command for execution
fn submit_command(state: TerminalState) {
    // Don't submit if listener failed (terminal non-functional)
    if state.listener_failed.get() {
        state.show_notification("Cannot execute: terminal connection failed".to_string());
        return;
    }

    let cmd = state.current_input.get();

    // Don't submit empty commands
    if cmd.trim().is_empty() {
        return;
    }

    // Clear input immediately
    state.clear_input();

    // Set busy state
    state.is_busy.set(true);

    // Spawn async task to call IPC
    spawn_local(async move {
        let args = match serde_wasm_bindgen::to_value(&ExecuteCommandArgs {
            command: cmd.clone(),
            cwd: None,
        }) {
            Ok(args) => args,
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to serialize command args: {e}").into());
                let err_line = OutputLine::Stderr {
                    text: "Failed to serialize command arguments".to_string(),
                    timestamp: current_timestamp_ms(),
                };
                state.push_history(err_line);
                state.is_busy.set(false);
                return;
            }
        };

        match invoke("execute_command", args).await {
            Ok(result) => {
                // Deserialize the structured response
                match serde_wasm_bindgen::from_value::<CommandResponse>(result) {
                    Ok(response) => {
                        // Use structured fields to detect failure
                        if let Some(error_msg) = response.error {
                            let err_line = OutputLine::Stderr {
                                text: error_msg,
                                timestamp: current_timestamp_ms(),
                            };
                            state.push_history(err_line);
                        }
                        // If no error, command completed successfully (output was streamed via events)
                    }
                    Err(e) => {
                        // Failed to deserialize response
                        web_sys::console::error_1(
                            &format!("Failed to parse command response: {e}").into(),
                        );
                        let err_line = OutputLine::Stderr {
                            text: "Failed to parse command response".to_string(),
                            timestamp: current_timestamp_ms(),
                        };
                        state.push_history(err_line);
                    }
                }
            }
            Err(e) => {
                // IPC failure
                let error_msg = e
                    .as_string()
                    .unwrap_or_else(|| "Unknown IPC error".to_string());
                web_sys::console::error_1(
                    &format!("execute_command IPC failed: {error_msg}").into(),
                );
                let err_line = OutputLine::Stderr {
                    text: format!("IPC Error: {error_msg}"),
                    timestamp: current_timestamp_ms(),
                };
                state.push_history(err_line);
                state.show_notification("Failed to execute command".to_string());
            }
        }

        // Clear busy state
        state.is_busy.set(false);
    });
}

/// Cancel the currently running command
fn cancel_command(state: TerminalState) {
    // Can't cancel if terminal is non-functional
    if state.listener_failed.get() {
        return;
    }

    // Only cancel if a command is running
    if !state.is_busy.get() {
        return;
    }

    spawn_local(async move {
        let args = JsValue::NULL;

        match invoke("cancel_command", args).await {
            Ok(result) => {
                if let Some(error) = result.as_string() {
                    if !error.is_empty() {
                        state.show_notification(format!("Cancel failed: {error}"));
                    }
                }
                // Command cancelled successfully (silence on success)
            }
            Err(e) => {
                let error_msg = e.as_string().unwrap_or_else(|| "Unknown error".to_string());
                web_sys::console::error_1(
                    &format!("cancel_command IPC failed: {error_msg}").into(),
                );
                state.show_notification(format!("Cancel failed: {error_msg}"));
                if error_msg.contains("No command currently running") {
                    state.is_busy.set(false);
                }
            }
        }
    });
}
