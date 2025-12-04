use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::components::{CommandInput, NotificationBar, OutputDisplay, PromptIndicator};
use crate::models::{OutputLine, TerminalState};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"], catch)]
    async fn listen(event: &str, handler: &Closure<dyn Fn(JsValue)>) -> Result<JsValue, JsValue>;
}

/// Tauri event payload structure
#[derive(Serialize, Deserialize, Debug)]
struct TauriEvent {
    payload: OutputLine,
}

/// Main terminal container that orchestrates all sub-components.
#[component]
pub fn Terminal() -> impl IntoView {
    let state = use_context::<TerminalState>().expect("TerminalState context missing");

    // Set up Tauri event listeners on mount
    Effect::new(move |_| {
        setup_event_listeners(state);
    });

    // Fetch initial history on mount
    Effect::new(move |_| {
        spawn_local(async move {
            fetch_initial_state(state).await;
        });
    });

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
}

/// Set up Tauri event listeners for output-line and shell-notification events
fn setup_event_listeners(state: TerminalState) {
    // Output line listener
    let output_handler = Closure::new(move |event: JsValue| {
        match serde_wasm_bindgen::from_value::<TauriEvent>(event) {
            Ok(tauri_event) => {
                state.push_history(tauri_event.payload);
            }
            Err(e) => {
                web_sys::console::error_1(
                    &format!("Failed to parse output-line event: {e:?}").into(),
                );
            }
        }
    });

    spawn_local(async move {
        match listen("output-line", &output_handler).await {
            Ok(_) => {
                // Keep the closure alive
                output_handler.forget();
            }
            Err(e) => {
                web_sys::console::error_1(
                    &format!("Failed to listen for output-line: {e:?}").into(),
                );
            }
        }
    });

    // Notification listener
    let state_notify = state;
    let notify_handler = Closure::new(move |event: JsValue| {
        match serde_wasm_bindgen::from_value::<TauriEvent>(event) {
            Ok(tauri_event) => {
                if let OutputLine::Notification { message, .. } = tauri_event.payload {
                    state_notify.show_notification(message);
                }
            }
            Err(e) => {
                web_sys::console::error_1(
                    &format!("Failed to parse notification event: {e:?}").into(),
                );
            }
        }
    });

    spawn_local(async move {
        match listen("shell-notification", &notify_handler).await {
            Ok(_) => {
                // Keep the closure alive
                notify_handler.forget();
            }
            Err(e) => {
                web_sys::console::error_1(
                    &format!("Failed to listen for shell-notification: {e:?}").into(),
                );
            }
        }
    });
}

/// Fetch initial history and cwd from backend
#[allow(clippy::future_not_send)]
async fn fetch_initial_state(state: TerminalState) {
    // Fetch history with error handling
    match invoke("get_history", JsValue::NULL).await {
        Ok(history_result) => {
            match serde_wasm_bindgen::from_value::<Vec<OutputLine>>(history_result) {
                Ok(history) => {
                    state.set_history(history);
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to parse history: {e:?}").into());
                    state.show_notification("Failed to load command history".to_string());
                }
            }
        }
        Err(e) => {
            web_sys::console::error_1(&format!("Failed to fetch history: {e:?}").into());
            state.show_notification("Failed to connect to shell service".to_string());
        }
    }

    // Fetch cwd with error handling
    match invoke("get_cwd", JsValue::NULL).await {
        Ok(cwd_result) => {
            if let Some(cwd) = cwd_result.as_string() {
                state.cwd.set(cwd);
            } else {
                web_sys::console::warn_1(&"CWD response was not a string".into());
            }
        }
        Err(e) => {
            web_sys::console::error_1(&format!("Failed to fetch cwd: {e:?}").into());
        }
    }
}
