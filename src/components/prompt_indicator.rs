use leptos::prelude::*;

use crate::models::TerminalState;

/// Displays the shell prompt with current working directory.
#[component]
pub fn PromptIndicator() -> impl IntoView {
    let state = use_context::<TerminalState>().expect("TerminalState context missing");

    view! {
        <div class="prompt-indicator">
            <span class="cwd">{move || format_cwd(&state.cwd.get(), state.has_home_dir.get())}</span>
            <span class="symbol">
                {move || if state.is_busy.get() { "⏳ " } else { "$ " }}
            </span>
        </div>
    }
}

/// Format the current working directory for display.
/// - Replaces home directory with ~
/// - Truncates long paths
fn format_cwd(cwd: &str, has_home: bool) -> String {
    if cwd.is_empty() {
        return String::from("(loading cwd)");
    }

    let mut display = cwd.to_string();

    // Replace home directory with ~ when available (presence only, not exact path).
    if has_home {
        // Best-effort: show "~" if cwd equals home, otherwise leave unchanged.
        if display == "/" || display == "\\" {
            // Root edge case: leave as-is.
        } else {
            display = display
                .trim_start_matches(std::path::MAIN_SEPARATOR)
                .to_string();
            display = format!("~{}", if display.is_empty() { "" } else { "/" }) + &display;
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
