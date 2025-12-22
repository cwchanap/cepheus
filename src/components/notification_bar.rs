use leptos::prelude::*;
use leptos::tachys::dom::window;
use wasm_bindgen::JsCast;

use crate::models::TerminalState;

type CallbackSlot =
    std::rc::Rc<std::cell::RefCell<Option<wasm_bindgen::prelude::Closure<dyn FnMut()>>>>;

/// Displays transient system notifications (non-modal).
#[component]
pub fn NotificationBar() -> impl IntoView {
    let state = use_context::<TerminalState>().expect("TerminalState context missing");

    // Simple timeout handling with local state
    let last_notification_id = std::rc::Rc::new(std::cell::Cell::new(None::<i32>));
    let active_callback: CallbackSlot = std::rc::Rc::new(std::cell::RefCell::new(None));

    // Auto-dismiss effect
    Effect::new({
        let last_notification_id = std::rc::Rc::clone(&last_notification_id);
        let active_callback = std::rc::Rc::clone(&active_callback);
        move |_| {
            // Cancel any existing timeout before setting a new one
            if let Some(timeout_id) = last_notification_id.get() {
                window().clear_timeout_with_handle(timeout_id);
                last_notification_id.set(None);
            }

            // Drop any existing callback
            active_callback.borrow_mut().take();

            if state.notification.get().is_some() {
                // Set a timeout to clear the notification after 3 seconds
                let state_clone = state;

                let callback: wasm_bindgen::prelude::Closure<dyn FnMut()> =
                    wasm_bindgen::closure::Closure::new(move || {
                        state_clone.clear_notification();
                    });

                // Keep the closure alive until timeout fires or is cleared
                *active_callback.borrow_mut() = Some(callback);

                if let Some(cb) = active_callback.borrow().as_ref() {
                    if let Ok(handle) = window()
                        .set_timeout_with_callback_and_timeout_and_arguments_0(
                            cb.as_ref().unchecked_ref(),
                            3000,
                        )
                    {
                        last_notification_id.set(Some(handle));
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
