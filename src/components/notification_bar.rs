use leptos::prelude::*;
use leptos::tachys::dom::window;
use wasm_bindgen::JsCast;

use crate::models::TerminalState;

/// Displays transient system notifications (non-modal).
#[component]
pub fn NotificationBar() -> impl IntoView {
    let state = use_context::<TerminalState>().expect("TerminalState context missing");

    // Simple timeout handling without complex cleanup
    let last_notification_id = std::rc::Rc::new(std::cell::Cell::new(None::<i32>));

    // Auto-dismiss effect
    Effect::new({
        let last_notification_id = std::rc::Rc::clone(&last_notification_id);
        move |_| {
            // Cancel any existing timeout before setting a new one
            if let Some(timeout_id) = last_notification_id.get() {
                window().clear_timeout_with_handle(timeout_id);
            }

            if state.notification.get().is_some() {
                // Set a timeout to clear the notification after 3 seconds
                let state_clone = state;
                let _last_notification_id_clone = std::rc::Rc::clone(&last_notification_id);

                let callback: wasm_bindgen::prelude::Closure<dyn FnMut()> =
                    wasm_bindgen::closure::Closure::new(move || {
                        state_clone.clear_notification();
                    });

                // set_timeout_with_callback returns a handle we can use to cancel
                if let Ok(handle) = window().set_timeout_with_callback_and_timeout_and_arguments_0(
                    callback.as_ref().unchecked_ref(),
                    3000,
                ) {
                    last_notification_id.set(Some(handle));
                    // Keep the closure alive for the timeout duration
                    callback.forget();
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
