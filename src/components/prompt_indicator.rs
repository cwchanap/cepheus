use leptos::prelude::*;
use std::path::Path;

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

    // Replace home directory with ~ using Path-based comparison
    let cwd_path = Path::new(cwd);
    let home_path = Path::new(&home);

    if let Ok(stripped) = cwd_path.strip_prefix(home_path) {
        if stripped.as_os_str().is_empty() {
            return "~".to_string();
        }
        return format!("~/{}", stripped.display());
    }

    // Truncate very long paths
    if cwd.len() > 50 {
        let path = Path::new(cwd);
        let components: Vec<_> = path.components().collect();

        if components.len() >= 3 {
            // Get the last two components for display
            let second_last = components[components.len() - 2].as_os_str();
            let last = components[components.len() - 1].as_os_str();
            return format!(".../{}/{}", second_last.display(), last.display());
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

    // No default path — return None (no macOS home path fallback)
    None
}
