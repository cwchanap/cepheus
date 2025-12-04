use leptos::prelude::*;

use crate::models::{OutputLine, TerminalState};

/// Scrollable display of terminal history.
#[component]
pub fn OutputDisplay() -> impl IntoView {
    let state = use_context::<TerminalState>().expect("TerminalState context missing");

    // Auto-scroll effect when history changes
    Effect::new(move |_| {
        let _ = state.history.get(); // Track changes

        // Use JavaScript to scroll to bottom
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(container) = document.get_element_by_id("output-container") {
                    let scroll_height = container.scroll_height();
                    container.set_scroll_top(scroll_height);
                }
            }
        }
    });

    view! {
        <div class="output-display" id="output-container">
            <For
                each=move || state.history.get()
                key=|line| line.timestamp()
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
