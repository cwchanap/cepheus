use leptos::prelude::*;
use leptos::tachys::dom::window;
use wasm_bindgen::JsCast;

use crate::models::TerminalState;

/// Displays transient system notifications (non-modal).
#[component]
pub fn NotificationBar() -> impl IntoView {
    let state = use_context::<TerminalState>().expect("TerminalState context missing");

    // Simple timeout handling without complex cleanup
    let last_notification_id = std::sync::Arc::new(std::sync::Mutex::new(None::<i32>));
    let active_callback: std::sync::Arc<
        std::sync::Mutex<Option<wasm_bindgen::prelude::Closure<dyn FnMut()>>>,
    > = std::sync::Arc::new(std::sync::Mutex::new(None));

    // Cleanup any pending timeout/closure on unmount
    {
        let last_notification_id = std::sync::Arc::clone(&last_notification_id);
        let active_callback = std::sync::Arc::clone(&active_callback);
        on_cleanup(move || {
            if let Ok(mut id) = last_notification_id.lock() {
                if let Some(timeout_id) = *id {
                    window().clear_timeout_with_handle(timeout_id);
                }
                *id = None;
            }
            if let Ok(mut cb) = active_callback.lock() {
                cb.take();
            }
        });
    }

    // Auto-dismiss effect
    Effect::new({
        let last_notification_id = std::sync::Arc::clone(&last_notification_id);
        let active_callback = std::sync::Arc::clone(&active_callback);
        move |_| {
            // Cancel any existing timeout before setting a new one
            if let Ok(mut id) = last_notification_id.lock() {
                if let Some(timeout_id) = *id {
                    window().clear_timeout_with_handle(timeout_id);
                }
                *id = None;
            }

            // Drop any existing callback
            if let Ok(mut cb) = active_callback.lock() {
                cb.take();
            }

            if state.notification.get().is_some() {
                // Set a timeout to clear the notification after 3 seconds
                let state_clone = state;

                let callback: wasm_bindgen::prelude::Closure<dyn FnMut()> =
                    wasm_bindgen::closure::Closure::new(move || {
                        state_clone.clear_notification();
                    });

                // Keep the closure alive until timeout fires or is cleared
                if let Ok(mut cb_slot) = active_callback.lock() {
                    *cb_slot = Some(callback);
                }

                if let Ok(cb_slot) = active_callback.lock() {
                    if let Some(cb) = cb_slot.as_ref() {
                        if let Ok(handle) = window()
                            .set_timeout_with_callback_and_timeout_and_arguments_0(
                                cb.as_ref().unchecked_ref(),
                                3000,
                            )
                        {
                            if let Ok(mut id) = last_notification_id.lock() {
                                *id = Some(handle);
                            }
                        }
                    }
                }
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
