use chrono::Local;
use env_logger::Builder;
use log::{debug, error, info, trace, warn, Level, LevelFilter, Record};
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
            RedFilter::Internal => !is_internal, // Swapped logic for "internal"
            RedFilter::External => is_internal,  // Swapped logic for "external"
            RedFilter::Both => true,
        }
    }
}

/// Initialize the logger, reading log settings from the `config.toml`
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

    // 3) Read log level and filter from config
    let level_filter = match config.logging.log_level.to_lowercase().as_str() {
        "critical" => LevelFilter::Error,
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" | _ => LevelFilter::Trace,
    };

    let red_filter = RedFilter::from_str(&config.logging.log_filter);

    // 4) Build the logger
    let mut builder = Builder::new();

    builder.filter(None, level_filter).format(move |_, record| {
        if !red_filter.should_show(record) {
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
