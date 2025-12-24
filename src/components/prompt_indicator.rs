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
/// - Truncates long paths
fn format_cwd(cwd: &str) -> String {
    if cwd.is_empty() {
        return String::from("(loading cwd)");
    }

    let truncate_component = |s: &str, max_len: usize| -> String {
        let chars: Vec<char> = s.chars().collect();
        if chars.len() <= max_len {
            return s.to_string();
        }
        let prefix_len = max_len / 2;
        let suffix_len = max_len.saturating_sub(prefix_len + 1);
        let prefix: String = chars.iter().take(prefix_len).collect();
        let suffix: String = chars
            .iter()
            .skip(chars.len().saturating_sub(suffix_len))
            .collect();
        format!("{prefix}…{suffix}")
    };

    let is_windows_drive_prefix = |s: &str| s.len() == 2 && s.ends_with(':');

    // For display we normalize backslashes to forward slashes for consistency
    let normalized = cwd.replace('\\', "/");
    if normalized.len() <= 50 {
        return normalized;
    }

    let components: Vec<&str> = normalized.split('/').filter(|c| !c.is_empty()).collect();
    if components.len() < 2 {
        return normalized;
    }

    let last = components[components.len() - 1];
    let second_last = components[components.len() - 2];

    if components[0] == "~" {
        format!("~/.../{second_last}/{last}")
    } else if is_windows_drive_prefix(components[0]) {
        format!("{}/.../{second_last}/{last}", components[0])
    } else if normalized.starts_with("//") {
        // UNC path: //server/share/...
        if components.len() >= 3 {
            let server = truncate_component(components[0], 12);
            let share = truncate_component(components[1], 12);
            format!("//{server}/{share}/.../{second_last}/{last}")
        } else {
            // Two-component UNC: //server/share — keep concise
            let server = truncate_component(second_last, 12);
            let share = truncate_component(last, 12);
            format!("//{server}/{share}")
        }
    } else {
        format!(".../{second_last}/{last}")
    }
}
