pub mod commands;
pub mod logging;
pub mod models;
pub mod state;

use commands::shell::{cancel_command, change_directory, execute_command, get_cwd, get_history};
use logging::setup_logging;
use state::ShellManager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! You've been greeted from Rust!")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging before starting Tauri
    if let Err(e) = setup_logging() {
        eprintln!("Warning: Failed to setup logging: {e}");
    }

    tracing::info!("Starting Cepheus terminal application");

    // Initialize shell manager state
    let shell_manager = ShellManager::new();
    let cwd_display = match std::env::current_dir() {
        Ok(path) => path.display().to_string(),
        Err(e) => format!("Error retrieving cwd: {}", e),
    };
    tracing::info!("Shell manager initialized with cwd: {}", cwd_display);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(shell_manager)
        .invoke_handler(tauri::generate_handler![
            greet,
            execute_command,
            cancel_command,
            get_history,
            get_cwd,
            change_directory
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
