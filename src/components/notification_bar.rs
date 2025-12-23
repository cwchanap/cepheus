use leptos::prelude::*;
use leptos::tachys::dom::window;
use send_wrapper::SendWrapper;
use std::sync::{Arc, Mutex};
use wasm_bindgen::JsCast;

use crate::models::TerminalState;

type CallbackSlot = Arc<Mutex<Option<SendWrapper<wasm_bindgen::prelude::Closure<dyn FnMut()>>>>>;

/// Displays transient system notifications (non-modal).
#[component]
pub fn NotificationBar() -> impl IntoView {
    let state = use_context::<TerminalState>().expect("TerminalState context missing");

    // Simple timeout handling with local state (must be Send + Sync for on_cleanup)
    let last_notification_id: Arc<Mutex<Option<i32>>> = Arc::new(Mutex::new(None));
    let active_callback: CallbackSlot = Arc::new(Mutex::new(None));

    // Auto-dismiss effect
    Effect::new({
        let last_notification_id = Arc::clone(&last_notification_id);
        let active_callback = Arc::clone(&active_callback);
        move |_| {
            // Cancel any existing timeout before setting a new one
            if let Ok(mut timeout_guard) = last_notification_id.lock() {
                if let Some(timeout_id) = *timeout_guard {
                    window().clear_timeout_with_handle(timeout_id);
                    *timeout_guard = None;
                }
            }

            // Drop any existing callback
            if let Ok(mut callback_guard) = active_callback.lock() {
                callback_guard.take();
            }

            if state.notification.get().is_some() {
                // Set a timeout to clear the notification after 3 seconds
                let state_clone = state;

                let callback: wasm_bindgen::prelude::Closure<dyn FnMut()> =
                    wasm_bindgen::closure::Closure::new(move || {
                        state_clone.clear_notification();
                    });

                // Keep the closure alive until timeout fires or is cleared
                if let Ok(mut callback_guard) = active_callback.lock() {
                    *callback_guard = Some(SendWrapper::new(callback));
                }

                if let Ok(callback_guard) = active_callback.lock() {
                    if let Some(cb) = callback_guard.as_ref() {
                        if let Ok(handle) = window()
                            .set_timeout_with_callback_and_timeout_and_arguments_0(
                                cb.as_ref().unchecked_ref(),
                                3000,
                            )
                        {
                            if let Ok(mut timeout_guard) = last_notification_id.lock() {
                                *timeout_guard = Some(handle);
                            }
                        }
                    }
                }
            }
        }
    });

    // Ensure timeouts/closures are cleared when the component unmounts
    on_cleanup({
        let last_notification_id = Arc::clone(&last_notification_id);
        let active_callback = Arc::clone(&active_callback);
        move || {
            if let Ok(mut timeout_guard) = last_notification_id.lock() {
                if let Some(timeout_id) = *timeout_guard {
                    window().clear_timeout_with_handle(timeout_id);
                    *timeout_guard = None;
                }
            }
            if let Ok(mut callback_guard) = active_callback.lock() {
                callback_guard.take();
            }
        }
    });

    view! {
        <Show when=move || state.notification.get().is_some()>
            <div class="notification-bar">
                {move || state.notification.get().unwrap_or_default()}
            </div>
        </Show>
    }
}
