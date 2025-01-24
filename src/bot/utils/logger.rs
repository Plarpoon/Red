use crate::bot::utils::config::Config;
use std::io;
use std::path::Path;
use tokio::fs;
use tracing::{debug, error, info, trace, warn};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{filter::filter_fn, layer::SubscriberExt, util::SubscriberInitExt, Layer};

/// Initializes the logger asynchronously:
///  1. Ensures the log directory exists
///  2. Sets up a global tracing subscriber with:
///     - A console layer (colorized)
///     - A file layer (JSON lines, async buffered)
///     - A custom filter to handle "internal"/"external"/"both" & log level
///
/// Returns a `WorkerGuard` that must be kept alive to ensure logs are flushed.
pub async fn init_logger_with_config(config: &Config) -> io::Result<WorkerGuard> {
    let log_folder = &config.logging.directory;
    ensure_log_directory_exists(log_folder).await?;

    // Create a non-blocking writer for "bot.log" in the specified directory
    let file_appender = tracing_appender::rolling::never(log_folder, "bot.log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    // Determine the maximum level from config
    let level_filter = parse_log_level(&config.logging.log_level);

    // Create a filter function for internal/external/both
    let filter_clone = config.logging.log_filter.to_lowercase();
    let target_filter = filter_fn(move |metadata| {
        // 1. Check level
        if metadata.level() > &level_filter {
            return false;
        }

        // 2. Check "internal"/"external" vs. target
        let is_external = metadata.target().starts_with("red::");
        match filter_clone.as_str() {
            "external" => is_external,
            "internal" => !is_external,
            // "both" => no filtering based on target
            "both" => true,
            // fallback
            _ => true,
        }
    });

    // Build the console layer (colorized pretty printing)
    let console_layer = tracing_subscriber::fmt::layer()
        .with_ansi(true)
        .with_filter(target_filter.clone());

    // Build the file layer (JSON output)
    let file_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_ansi(false)
        .with_writer(file_writer)
        .with_filter(target_filter);

    // Optionally, add time formatting or other config
    // e.g. .with_timer(...)
    // .with_timer(OffsetTime::local_rfc_3339().unwrap_or_else(|_| OffsetTime::local()))
    // If local offset time fails, fallback to UTC, etc.

    // Combine them into a single subscriber with layering
    tracing_subscriber::registry()
        // We can optionally override with an EnvFilter, but we'll rely on our custom filter above.
        // .with(EnvFilter::from_default_env())
        .with(console_layer)
        .with(file_layer)
        .init();

    // We hold onto "guard" so logs are flushed before exit
    info!("Tracing-based async logger initialized successfully.");
    Ok(guard)
}

/// Convenience function to ensure the logs directory exists (async).
async fn ensure_log_directory_exists(dir: &str) -> io::Result<()> {
    if !Path::new(dir).exists() {
        fs::create_dir_all(dir).await?;
    }
    Ok(())
}

/// Maps your string to a tracing `Level`.
/// "trace" is the lowest, "error" is the highest severity only, etc.
fn parse_log_level(level_str: &str) -> tracing::Level {
    match level_str.to_lowercase().as_str() {
        "error" => tracing::Level::ERROR,
        "warn" => tracing::Level::WARN,
        "info" => tracing::Level::INFO,
        "debug" => tracing::Level::DEBUG,
        "trace" => tracing::Level::TRACE,
        _ => tracing::Level::TRACE, // fallback
    }
}

// Below are your old convenience methods, but for `tracing`:
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
