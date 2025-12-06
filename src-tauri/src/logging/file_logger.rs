use tracing_appender::rolling;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Setup file-based logging to ~/.cepheus/terminal.log
///
/// # Errors
/// Returns an error if the log directory cannot be created or logging fails to initialize.
pub fn setup_logging() -> Result<(), Box<dyn std::error::Error>> {
    let log_dir = dirs::home_dir()
        .ok_or("Cannot find home directory")?
        .join(".cepheus");

    std::fs::create_dir_all(&log_dir)?;

    let file_appender = rolling::daily(&log_dir, "terminal.log");

    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(file_appender))
        .with(
            EnvFilter::from_default_env()
                .add_directive("cepheus=debug".parse()?)
                .add_directive("cepheus_lib=debug".parse()?),
        )
        .init();

    tracing::info!(
        "Logging initialized to {:?}/terminal.log (daily rotation)",
        log_dir
    );

    Ok(())
}
