use crate::bot::utils::config::Config;
use crate::bot::utils::logrotate;

use chrono::Local;
use colored::Colorize;
use fern::Dispatch;
use log::{Level, LevelFilter, info};
use std::fs;
use std::io;
use std::path::Path;

/**
 * Initializes the logger based on the provided configuration.
 * This function sets up logging to both a file and stdout,
 * and also spawns an asynchronous task to handle log rotation.
 */
pub async fn init_logger_with_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    /* Parse log level from the configuration. */
    let log_level = match config.logging.log_level.to_lowercase().as_str() {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info,
    };

    /* Create a log directory based on today's date. */
    let log_dir = create_log_directory(&config.logging.directory)?;
    let red_log_path = format!("{}/red.log", log_dir);
    let serenity_log_path = format!("{}/serenity.log", log_dir);

    /* Define console formatting with colored log levels. */
    let console_format =
        move |out: fern::FormatCallback, message: &std::fmt::Arguments, record: &log::Record| {
            let level_color = colorize_level(record.level());
            out.finish(format_args!(
                "{} [{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                level_color,
                message
            ))
        };

    /* Define file formatting (without color). */
    let file_format =
        move |out: fern::FormatCallback, message: &std::fmt::Arguments, record: &log::Record| {
            out.finish(format_args!(
                "{} [{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        };

    /* Set up the fern logger with separate chains for non-serenity and serenity logs. */
    Dispatch::new()
        .level(log_level)
        /* Log non-serenity logs to red.log */
        .chain(
            Dispatch::new()
                .filter(|record| !record.target().starts_with("serenity"))
                .format(file_format)
                .chain(fern::log_file(&red_log_path)?)
                .level(log_level),
        )
        /* Log non-serenity logs to console */
        .chain(
            Dispatch::new()
                .filter(|record| !record.target().starts_with("serenity"))
                .format(console_format)
                .chain(std::io::stdout())
                .level(log_level),
        )
        /* Log serenity logs exclusively to serenity.log */
        .chain(
            Dispatch::new()
                .filter(|record| record.target().starts_with("serenity"))
                .format(file_format)
                .chain(fern::log_file(&serenity_log_path)?)
                .level(log_level),
        )
        .apply()?;

    info!("Logger initialized with level {:?}", log_level);
    info!("Logging to file: {}", red_log_path);
    info!("Serenity logs to file: {}", serenity_log_path);

    /* Spawn the asynchronous log rotation task. */
    let base_dir = config.logging.directory.clone();
    let rotation_frequency = config.logrotate.parse_frequency();
    let rotation_time = config.logrotate.rotation_time.clone();
    tokio::spawn(async move {
        logrotate::schedule_log_rotation(&base_dir, rotation_frequency, &rotation_time).await;
    });

    Ok(())
}

/**
 * Creates a log directory based on the current date within the specified base directory.
 */
fn create_log_directory(base_dir: &str) -> io::Result<String> {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let log_dir = format!("{}/{}", base_dir, today);
    if !Path::new(&log_dir).exists() {
        fs::create_dir_all(&log_dir)?;
    }
    Ok(log_dir)
}

/**
 * Returns a colored string for the log level using the colored crate.
 */
fn colorize_level(level: Level) -> colored::ColoredString {
    match level {
        Level::Error => "ERROR".red().bold(),
        Level::Warn => "WARN".yellow().bold(),
        Level::Info => "INFO".green().bold(),
        Level::Debug => "DEBUG".blue().bold(),
        Level::Trace => "TRACE".cyan().bold(),
    }
}
