use leptos::prelude::*;

use crate::models::TerminalState;

/// Displays transient system notifications (non-modal).
#[component]
pub fn NotificationBar() -> impl IntoView {
    let state = use_context::<TerminalState>().expect("TerminalState context missing");

    // Auto-dismiss effect
    Effect::new(move |_| {
        if state.notification.get().is_some() {
            // Set a timeout to clear the notification after 3 seconds
            let state_clone = state;
            set_timeout(
                move || {
                    state_clone.clear_notification();
                },
                std::time::Duration::from_secs(3),
            );
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
