use leptos::html::Div;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::models::{OutputLine, TerminalState};

/// Scrollable display of terminal history.
#[component]
pub fn OutputDisplay() -> impl IntoView {
    let state = use_context::<TerminalState>().expect("TerminalState context missing");
    let container_ref = NodeRef::<Div>::new();
    // Track if we should auto-scroll (sticky bottom)
    let is_sticky = StoredValue::new(true);

    // Auto-scroll effect when history changes
    Effect::new(move |_| {
        let _ = state.history.get(); // Track changes

        // Only scroll if we are sticky
        if is_sticky.get_value() {
            // Schedule scrolling after the next paint to ensure DOM is updated
            if let Some(window) = web_sys::window() {
                let window_clone = window.clone();
                let scroll_closure: Closure<dyn Fn()> = Closure::wrap(Box::new(move || {
                    if let Some(document) = window_clone.document() {
                        if let Some(container) = document.get_element_by_id("output-container") {
                            let scroll_height = container.scroll_height();
                            container.set_scroll_top(scroll_height);
                        }
                    }
                }));

                // Schedule the scroll callback for after the next paint
                window
                    .request_animation_frame(scroll_closure.as_ref().unchecked_ref())
                    .expect("Failed to request animation frame");

                // Keep the closure alive by forgetting it
                scroll_closure.forget();
            }
        }
    });

    let on_scroll = move |_| {
        if let Some(div) = container_ref.get() {
            // Check if we are at the bottom (with small tolerance)
            let at_bottom = (div.scroll_top() + div.client_height()) >= (div.scroll_height() - 10);
            is_sticky.set_value(at_bottom);
        }
    };

    view! {
        <div
            class="output-display"
            id="output-container"
            node_ref=container_ref
            on:scroll=on_scroll
        >
            <For
                each=move || state.history.get()
                key=|line| line.unique_key()
                children=move |line| view! { <OutputLineView line=line /> }
            />
        </div>
    }
}

/// Renders a single OutputLine with appropriate styling.
#[component]
fn OutputLineView(line: OutputLine) -> impl IntoView {
    let css_class = line.css_class();
    let content = format_line_content(&line);

    view! {
        <div class=css_class>
            {content}
        </div>
    }
}

/// Format the content of an output line for display
fn format_line_content(line: &OutputLine) -> String {
    match line {
        OutputLine::Command { text, .. } => format!("$ {text}"),
        OutputLine::Stdout { text, .. } | OutputLine::Stderr { text, .. } => text.clone(),
        OutputLine::Notification { message, .. } => format!("⚠️  {message}"),
    }
}
