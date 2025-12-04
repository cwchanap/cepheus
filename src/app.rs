use leptos::prelude::*;

use crate::components::Terminal;
use crate::models::TerminalState;

/// Root application component that provides global context and mounts the Terminal.
#[component]
pub fn App() -> impl IntoView {
    // Initialize terminal state
    let state = TerminalState::new();

    // Provide context to all child components
    provide_context(state);

    view! {
        <main class="app">
            <Terminal />
        </main>
    }
}
