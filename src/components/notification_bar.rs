use leptos::prelude::*;
use leptos::tachys::dom::window;
use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::JsCast;

use crate::models::TerminalState;

/// Displays transient system notifications (non-modal).
#[component]
pub fn NotificationBar() -> impl IntoView {
    let state = use_context::<TerminalState>().expect("TerminalState context missing");

    // Store the timeout handle so we can cancel it on cleanup/re-run
    let timeout_handle: Rc<Cell<Option<i32>>> = Rc::new(Cell::new(None));

    // Auto-dismiss effect with cleanup
    Effect::new({
        let timeout_handle = Rc::clone(&timeout_handle);
        move |_| {
            // Cancel any existing timeout before setting a new one
            if let Some(handle) = timeout_handle.get() {
                window().clear_timeout_with_handle(handle);
                timeout_handle.set(None);
            }

            if state.notification.get().is_some() {
                // Set a timeout to clear the notification after 3 seconds
                let state_clone = state;
                let timeout_handle_clone = Rc::clone(&timeout_handle);

                let callback = wasm_bindgen::closure::Closure::once(move || {
                    state_clone.clear_notification();
                    timeout_handle_clone.set(None);
                });

                // set_timeout_with_callback returns a handle we can use to cancel
                if let Ok(handle) = window().set_timeout_with_callback_and_timeout_and_arguments_0(
                    callback.as_ref().unchecked_ref(),
                    3000,
                ) {
                    timeout_handle.set(Some(handle));
                }

                // Prevent the closure from being dropped (it will run once and clean up)
                callback.forget();
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
