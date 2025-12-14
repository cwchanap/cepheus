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
    let mut display = cwd.to_string();

    // Replace home directory with ~ when available.
    if let Some(home) = home_dir() {
        let home_norm = home.replace('\\', "/");
        let cwd_norm = cwd.replace('\\', "/");

        let home_trimmed = home_norm.trim_end_matches('/');
        if cwd_norm == home_trimmed {
            display = "~".to_string();
        } else {
            let prefix = format!("{home_trimmed}/");
            if cwd_norm.starts_with(&prefix) {
                display = format!("~{}", &cwd_norm[home_trimmed.len()..]);
            }
        }
    }

    // Truncate very long paths
    if display.len() > 50 {
        let components: Vec<&str> = display
            .split(['/', '\\'])
            .filter(|c| !c.is_empty())
            .collect();

        if components.len() >= 3 {
            // Get the last two components for display
            let second_last = components[components.len() - 2];
            let last = components[components.len() - 1];

            display = if components[0] == "~" {
                format!("~/.../{second_last}/{last}")
            } else if components[0].ends_with(':') {
                format!("{}/.../{second_last}/{last}", components[0])
            } else {
                format!(".../{second_last}/{last}")
            };
        }
    }

    display
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
