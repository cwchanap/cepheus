pub mod commands;
pub mod logging;
pub mod models;
pub mod state;

use commands::shell::{
    cancel_command, change_directory, execute_command, get_cwd, get_history, get_home_dir,
};
use logging::setup_logging;
use state::ShellManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging before starting Tauri
    if let Err(e) = setup_logging() {
        eprintln!("Warning: Failed to setup logging: {e}");
    }

    tracing::info!("Starting Cepheus terminal application");

    // Initialize shell manager state
    // Get CWD once before creating ShellManager to ensure logged value matches actual initialization
    let current_dir = match std::env::current_dir() {
        Ok(path) => path,
        Err(e) => {
            tracing::warn!("Failed to get current directory: {}", e);

            // Platform-aware fallback for when current_dir fails
            if cfg!(windows) {
                // Windows: try USERPROFILE first, then derive drive root from env vars
                let fallback_drive_root = std::env::var("SystemDrive")
                    .or_else(|_| std::env::var("HOMEDRIVE"))
                    .map(|drive| format!("{}\\", drive.trim_end_matches('\\')))
                    .unwrap_or_else(|_| "C:\\".to_string());
                std::env::var("USERPROFILE").map_or_else(
                    |_| std::path::PathBuf::from(fallback_drive_root),
                    std::path::PathBuf::from,
                )
            } else {
                // Unix-like systems: try HOME first, then fallback to root
                std::env::var("HOME")
                    .map_or_else(|_| std::path::PathBuf::from("/"), std::path::PathBuf::from)
            }
        }
    };

    let cwd_display = current_dir.display().to_string();
    let initial_cwd = current_dir.to_string_lossy().to_string();

    let shell_manager = ShellManager::new_with_cwd(initial_cwd);
    tracing::info!("Shell manager initialized with cwd: {}", cwd_display);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(shell_manager)
        .invoke_handler(tauri::generate_handler![
            execute_command,
            cancel_command,
            get_history,
            get_cwd,
            get_home_dir,
            change_directory
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
