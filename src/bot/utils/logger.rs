use chrono::Local;
use env_logger::Builder;
use log::{debug, error, info, trace, warn, Level, LevelFilter, Record};
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::bot::utils::config::Config;

/// Initializes the logging directory.
fn setup_log_directory(log_folder: &str) {
    if let Err(e) = create_dir_all(log_folder) {
        eprintln!("Failed to create logs directory '{}': {}", log_folder, e);
        std::process::exit(1);
    }
}

/// Opens or creates the log file.
fn open_log_file(log_file_path: &str) -> std::fs::File {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file_path)
        .unwrap_or_else(|e| {
            eprintln!(
                "Failed to open or create log file '{}': {}",
                log_file_path, e
            );
            std::process::exit(1);
        })
}

/// Determines if a record should be displayed based on the filter.
fn should_display_record(filter: &str, record: &Record) -> bool {
    let is_external = record.target().starts_with("red::");

    match filter.to_lowercase().as_str() {
        "external" => is_external,
        "internal" => !is_external,
        "both" => true,
        _ => true, // Fallback to showing all logs
    }
}

/// Configures and initializes the logger.
pub fn init_logger_with_config(config: &Config) {
    let log_folder = &config.logging.directory;
    setup_log_directory(log_folder);

    let log_file_path = format!("{}/bot.log", log_folder);
    let log_file = open_log_file(&log_file_path);

    let level_filter = match config.logging.log_level.to_lowercase().as_str() {
        "critical" => LevelFilter::Error,
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" | _ => LevelFilter::Trace,
    };

    let log_filter = config.logging.log_filter.clone();

    let mut builder = Builder::new();

    builder.filter(None, level_filter).format(move |_, record| {
        if !should_display_record(&log_filter, record) {
            return Ok(());
        }

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let level = record.level();

        let mut displayed_target = record.target().to_owned();

        if displayed_target.starts_with("serenity::") {
            displayed_target = "[Serenity]".into();
        } else if displayed_target.starts_with("red::") {
            displayed_target = "[Red]".into();
        }

        let message = format!(
            "[{}] [{}] {}: {}",
            timestamp,
            level,
            displayed_target,
            record.args()
        );

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

    builder.init();

    info!("Logger initialized successfully.");
}

//------------------------------------------------------------------
// Convenience log methods:
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
