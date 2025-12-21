use js_sys::Function;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

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
    let listeners = StoredValue::new_local(ListenerHandles::default());

    // Set up Tauri event listeners on mount - run only once per component instance
    let listeners_setup = std::cell::Cell::new(false);
    let listeners_for_cleanup = listeners;
    Effect::new(move |_| {
        if !listeners_setup.get() {
            listeners_setup.set(true);
            setup_event_listeners(state, listeners);
        }
    });
    on_cleanup(move || cleanup_listener_handles(listeners_for_cleanup));

    // Fetch initial history and cwd on mount - run only once per component instance
    let fetch_setup = std::cell::Cell::new(false);
    Effect::new(move |_| {
        if !fetch_setup.get() {
            fetch_setup.set(true);
            spawn_local(async move {
                fetch_initial_state(state).await;
            });
        }
    });

    view! {
        <div class="terminal-container">
            <NotificationBar />
            {move || {
                if state.listener_failed.get() {
                    view! {
                        <div class="terminal-error-banner">
                            <span class="error-icon">"⚠️"</span>
                            <span class="error-message">
                                {move || state.listener_error.get().unwrap_or_else(|| "Terminal connection failed".to_string())}
                            </span>
                        </div>
                    }.into_any()
                } else {
                    ().into_any()
                }
            }}
            <OutputDisplay />
            <div class="input-row">
                <PromptIndicator />
                <CommandInput />
            </div>
        </div>
    }
}

#[derive(Default)]
struct ListenerHandles {
    output: Option<ListenerHandle>,
    notify: Option<ListenerHandle>,
}

#[derive(Clone)]
struct ListenerHandle {
    callback: Rc<Closure<dyn Fn(JsValue)>>,
    unlisten: JsValue,
}

type ListenerStore = StoredValue<ListenerHandles, LocalStorage>;

fn cleanup_listener_handles(handles: ListenerStore) {
    handles.update_value(|handles| {
        if let Some(handle) = handles.output.take() {
            call_unlisten(handle.unlisten, "output-line");
            drop(handle.callback);
        }

        if let Some(handle) = handles.notify.take() {
            call_unlisten(handle.unlisten, "shell-notification");
            drop(handle.callback);
        }
    });
}

fn call_unlisten(unlisten: JsValue, label: &str) {
    match unlisten.dyn_into::<Function>() {
        Ok(func) => {
            if let Err(e) = func.call0(&JsValue::NULL) {
                web_sys::console::error_1(
                    &format!("Failed to unlisten {label} handler: {e:?}").into(),
                );
            }
        }
        Err(_) => {
            web_sys::console::warn_1(
                &format!("Unlisten handle for {label} was not a function").into(),
            );
        }
    }
}

/// Set up Tauri event listeners for output-line and shell-notification events
fn setup_event_listeners(state: TerminalState, listeners: ListenerStore) {
    // Output line listener
    let output_handler =
        Rc::new(Closure::new(
            move |event: JsValue| match serde_wasm_bindgen::from_value::<TauriEvent>(event) {
                Ok(tauri_event) => {
                    state.push_history(tauri_event.payload);
                }
                Err(e) => {
                    web_sys::console::error_1(
                        &format!("Failed to parse output-line event: {e:?}").into(),
                    );
                }
            },
        ));

    let state_output = state;
    let listeners_output = listeners;
    let output_handler_for_listen = output_handler.clone();
    spawn_local(async move {
        match listen("output-line", &output_handler_for_listen).await {
            Ok(unlisten) => {
                listeners_output.update_value(|handles| {
                    handles.output = Some(ListenerHandle {
                        callback: output_handler,
                        unlisten,
                    });
                });
            }
            Err(e) => {
                let error_msg =
                    format!("Terminal connection failed: output-line listener error: {e:?}");
                web_sys::console::error_1(&error_msg.clone().into());
                state_output.set_listener_failed(error_msg.clone());
                state_output.show_notification(format!("Terminal is non-functional: {error_msg}"));
            }
        }
    });

    // Notification listener
    let state_notify = state;
    let listeners_notify = listeners;
    let notify_handler =
        Rc::new(Closure::new(
            move |event: JsValue| match serde_wasm_bindgen::from_value::<TauriEvent>(event) {
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
            },
        ));

    let notify_handler_for_listen = notify_handler.clone();
    spawn_local(async move {
        match listen("shell-notification", &notify_handler_for_listen).await {
            Ok(unlisten) => {
                listeners_notify.update_value(|handles| {
                    handles.notify = Some(ListenerHandle {
                        callback: notify_handler,
                        unlisten,
                    });
                });
            }
            Err(e) => {
                let error_msg =
                    format!("Terminal connection failed: shell-notification listener error: {e:?}");
                web_sys::console::error_1(&error_msg.clone().into());
                state_notify.set_listener_failed(error_msg.clone());
                state_notify.show_notification(format!("Terminal is non-functional: {error_msg}"));
            }
        }
    });
}

/// Fetch and store the home directory in-memory to avoid persisting PII client-side.
#[allow(clippy::future_not_send)]
async fn set_home_dir_in_memory(state: TerminalState) {
    match invoke("get_home_dir", JsValue::NULL).await {
        Ok(home_result) => match home_result.as_string() {
            Some(home) => {
                // Presence only; do not store raw home path in memory.
                if !home.is_empty() {
                    state.has_home_dir.set(true);
                }
            }
            None => {
                web_sys::console::warn_1(&"home_dir response was not a string".into());
            }
        },
        Err(e) => {
            web_sys::console::warn_1(&format!("Failed to fetch home_dir: {e:?}").into());
        }
    }
}

/// Fetch initial history and cwd from backend
#[allow(clippy::future_not_send)]
async fn fetch_initial_state(state: TerminalState) {
    // We intentionally avoid storing the raw home directory; track only presence.
    set_home_dir_in_memory(state).await;

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
