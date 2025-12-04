use leptos::prelude::*;

use crate::models::TerminalState;

/// Displays the shell prompt with current working directory.
#[component]
pub fn PromptIndicator() -> impl IntoView {
    let state = use_context::<TerminalState>().expect("TerminalState context missing");

    view! {
        <div class="prompt-indicator">
            <span class="cwd">{move || format_cwd(&state.cwd.get())}</span>
            <span class="symbol">
                {move || if state.is_busy.get() { "⏳ " } else { "$ " }}
            </span>
        </div>
    }
}

/// Format the current working directory for display.
/// - Replaces home directory with ~
/// - Truncates long paths
fn format_cwd(cwd: &str) -> String {
    // Get home directory for abbreviation
    let Some(home) = home_dir() else {
        return cwd.to_string();
    };

    // Replace home directory with ~
    if let Some(stripped) = cwd.strip_prefix(&home) {
        if stripped.is_empty() {
            return "~".to_string();
        }
        return format!("~{stripped}");
    }

    // Truncate very long paths
    if cwd.len() > 50 {
        let parts: Vec<&str> = cwd.split('/').collect();
        if parts.len() > 3 {
            return format!(".../{}/{}", parts[parts.len() - 2], parts[parts.len() - 1]);
        }
    }

    cwd.to_string()
}

/// Get the home directory path
fn home_dir() -> Option<String> {
    // In WASM, we can't access filesystem directly
    // So we'll use a heuristic or return a placeholder
    // This will be set properly when get_cwd is called
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(home)) = storage.get_item("home_dir") {
                return Some(home);
            }
        }
    }

    // Default to common macOS home path pattern
    None
}
