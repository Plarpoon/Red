use chrono::Local;
use env_logger::Builder;
use log::{debug, error, info, trace, warn, Level, LevelFilter, Record};
use std::env;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::bot::utils::config::Config;

/// An enum representing which "channel" of logs to show.
#[derive(Debug)]
enum RedFilter {
    Internal,
    External,
    Both,
}

impl RedFilter {
    /// Construct from a string, ignoring case.
    fn from_str(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "internal" => RedFilter::Internal,
            "external" => RedFilter::External,
            _ => RedFilter::Both,
        }
    }

    /// Returns `true` if this filter says "show the given record".
    fn should_show(&self, record: &Record) -> bool {
        let is_internal = record.target().starts_with("red::");
        match self {
            RedFilter::Internal => is_internal,
            RedFilter::External => !is_internal,
            RedFilter::Both => true,
        }
    }
}

/// Initialize the logger, reading log settings from the `config.toml`
/// under `[logging]`, with optional overrides by environment variables
/// `RUST_LOG_LEVEL` (for overall level) and `RED_FILTER` (for which channel).
pub fn init_logger_with_config(config: &Config) {
    // 1) Determine log folder from config
    let log_folder = &config.logging.directory;
    if let Err(e) = create_dir_all(log_folder) {
        eprintln!("Failed to create logs directory '{}': {}", log_folder, e);
        std::process::exit(1);
    }

    // 2) Open or create the log file in that folder
    let log_file_path = format!("{}/bot.log", log_folder);
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)
        .unwrap_or_else(|e| {
            eprintln!(
                "Failed to open or create log file '{}': {}",
                log_file_path, e
            );
            std::process::exit(1);
        });

    // 3) Determine the maximum verbosity from either env or config
    let log_level_env = env::var("RUST_LOG_LEVEL").ok();
    let log_level_str = log_level_env
        .as_deref()
        .unwrap_or(&config.logging.log_level); // fallback to config if env not set

    let level_filter = match log_level_str.to_lowercase().as_str() {
        "critical" => LevelFilter::Error, // "critical" -> treat as Error
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" | _ => LevelFilter::Trace,
    };

    // 4) Determine which logs to show (internal/external/both) from either env or config
    let red_filter_env = env::var("RED_FILTER").ok();
    let red_filter_str = red_filter_env
        .as_deref()
        .unwrap_or(&config.logging.log_filter); // fallback to config

    let red_filter = RedFilter::from_str(red_filter_str);

    // 5) Build the logger using env_logger
    let mut builder = Builder::new();

    // Keep your original filters:
    builder
        // Global log level
        .filter(None, level_filter)
        // Serenity modules, etc:
        .filter_module("serenity::gateway", LevelFilter::Trace)
        .filter_module("serenity::http", LevelFilter::Debug)
        .filter_module("rustls", LevelFilter::Warn)
        .filter_module("tungstenite", LevelFilter::Warn)
        .filter_module(
            "serenity::gateway::bridge::shard_runner",
            LevelFilter::Debug,
        )
        .filter_module(
            "serenity::gateway::bridge::shard_queuer",
            LevelFilter::Debug,
        )
        .filter_module("serenity::gateway::shard", LevelFilter::Debug)
        .filter_module("tracing::span", LevelFilter::Warn);

    // 6) Define our custom formatting
    builder.format(move |_, record| {
        // If we are skipping logs based on red_filter, do so
        if !red_filter.should_show(record) {
            return Ok(());
        }

        // Otherwise, format as usual:
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let level = record.level();

        // Shorten the target name
        let mut displayed_target = record.target().to_owned();
        if displayed_target.starts_with("serenity::") {
            displayed_target = "[Serenity]".into();
        } else if displayed_target.starts_with("red::") {
            displayed_target = "[Red]".into();
        }

        // Build final console message
        let message = format!(
            "[{}] [{}] {}: {}",
            timestamp,
            level,
            displayed_target,
            record.args()
        );

        // Colorize based on level
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        let color = match level {
            Level::Error => Color::Red,
            Level::Warn => Color::Yellow,
            Level::Info => Color::Green,
            Level::Debug => Color::Cyan,
            Level::Trace => Color::Blue,
        };
        let mut color_spec = ColorSpec::new();
        color_spec.set_fg(Some(color)).set_bold(true);
        stdout.set_color(&color_spec).ok();
        write!(stdout, "{}", message).ok();
        stdout.reset().ok();
        writeln!(stdout).ok();

        // Also write JSON-ish to file
        let mut file = log_file
            .try_clone()
            .expect("Failed to clone log file handle");
        writeln!(
            file,
            "{{\"timestamp\":\"{}\",\"level\":\"{}\",\"target\":\"{}\",\"message\":\"{}\"}}",
            timestamp,
            level,
            displayed_target,
            record.args()
        )
        .ok();

        Ok(())
    });

    // 7) Initialize the logger
    builder.init();

    // Optional: produce a few example logs
    info!("Logger initialized successfully.");
    info!("Application is starting...");
}

//------------------------------------------------------------------
// The same convenience log methods (unchanged):
//------------------------------------------------------------------
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
