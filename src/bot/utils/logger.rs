use crate::bot::utils::config::Config;
use std::io;
use std::path::Path;
use tokio::fs;
use tracing::{debug, error, info, trace, warn};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    filter::LevelFilter, layer::SubscriberExt, util::SubscriberInitExt, Layer,
};

/// Initializes the logger asynchronously:
///  1. Ensures the log directory exists
///  2. Sets up a global tracing subscriber with:
///     - A console layer (colorized)
///     - A file layer (JSON lines, async buffered)
///  3. Applies only the log-level filter (no internal/external distinction)
///
/// Returns a `WorkerGuard` that must be kept alive to ensure logs are flushed.
pub async fn init_logger_with_config(config: &Config) -> io::Result<WorkerGuard> {
    let log_folder = &config.logging.directory;
    ensure_log_directory_exists(log_folder).await?;

    // Create a non-blocking writer for "bot.log" in the specified directory
    let file_appender = tracing_appender::rolling::never(log_folder, "bot.log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    // Convert our string to a `LevelFilter` for tracing
    let level_filter = parse_log_level(&config.logging.log_level);

    // Build a console layer (colorized)
    let console_layer = tracing_subscriber::fmt::layer()
        .with_ansi(true)
        .with_filter(level_filter);

    // Build a file layer (JSON output)
    let file_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_ansi(false)
        .with_writer(file_writer)
        .with_filter(level_filter);

    // Combine them into a single subscriber with layering
    tracing_subscriber::registry()
        // Could also add an EnvFilter, e.g. EnvFilter::from_default_env()
        .with(console_layer)
        .with(file_layer)
        .init();

    info!("logger initialized successfully.");
    Ok(guard)
}

/// Convenience function to ensure the logs directory exists (async).
async fn ensure_log_directory_exists(dir: &str) -> io::Result<()> {
    if !Path::new(dir).exists() {
        fs::create_dir_all(dir).await?;
    }
    Ok(())
}

/// Convert the user config's log_level string into a `tracing_subscriber::filter::LevelFilter`.
fn parse_log_level(level_str: &str) -> LevelFilter {
    match level_str.to_lowercase().as_str() {
        "error" => LevelFilter::ERROR,
        "warn" => LevelFilter::WARN,
        "info" => LevelFilter::INFO,
        "debug" => LevelFilter::DEBUG,
        "trace" => LevelFilter::TRACE,
        _ => LevelFilter::TRACE, // fallback
    }
}

// -----------------------------------------------------------------
// Convenience log methods using tracing macros
// -----------------------------------------------------------------
pub fn log_critical(message: &str) {
    error!("{}", message);
}
pub fn log_error(message: &str) {
    error!("{}", message);
}
pub fn log_warn(message: &str) {
    warn!("{}", message);
}
pub fn log_info(message: &str) {
    info!("{}", message);
}
pub fn log_debug(message: &str) {
    debug!("{}", message);
}
pub fn log_trace(message: &str) {
    trace!("{}", message);
}
