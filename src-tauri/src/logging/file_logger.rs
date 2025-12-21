use std::cmp::Ordering;
use std::fs;
use std::path::Path;
use tracing_appender::rolling;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

const MAX_LOG_FILES: usize = 14; // keep roughly two weeks of daily logs

/// Setup file-based logging to ~/.cepheus/terminal.log
///
/// # Errors
/// Returns an error if the log directory cannot be created or logging fails to initialize.
pub fn setup_logging() -> Result<(), Box<dyn std::error::Error>> {
    let log_dir = dirs_next::home_dir()
        .ok_or("Cannot find home directory")?
        .join(".cepheus");

    std::fs::create_dir_all(&log_dir)?;

    // Best-effort cleanup of old rotated logs to avoid unbounded disk growth.
    // We perform this before initializing tracing so we can log subsequent issues normally.
    cleanup_old_logs(&log_dir, MAX_LOG_FILES);

    let file_appender = rolling::daily(&log_dir, "terminal.log");

    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(file_appender))
        .with(
            EnvFilter::from_default_env()
                .add_directive("cepheus=debug".parse()?)
                .add_directive("cepheus_lib=debug".parse()?),
        )
        .try_init()?;

    tracing::info!(
        "Logging initialized to {:?}/terminal.log (daily rotation)",
        log_dir
    );

    Ok(())
}

fn cleanup_old_logs(log_dir: &Path, max_files: usize) {
    let Ok(entries) = fs::read_dir(log_dir) else {
        eprintln!("log retention: failed to read log dir {:?}", log_dir);
        return;
    };

    let mut logs: Vec<_> = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
        .filter(|entry| entry.file_name().to_string_lossy().contains("terminal.log"))
        .filter_map(|entry| {
            let modified = entry.metadata().ok().and_then(|m| m.modified().ok());
            Some((entry.path(), modified))
        })
        .collect();

    logs.sort_by(|a, b| match (a.1, b.1) {
        (Some(a_time), Some(b_time)) => b_time.cmp(&a_time),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    });

    if logs.len() > max_files {
        for (path, _) in logs.into_iter().skip(max_files) {
            if let Err(err) = fs::remove_file(&path) {
                eprintln!("log retention: failed to remove {:?}: {err:?}", path);
            }
        }
    }
}
