use crate::bot::utils::config::Config;

use chrono::Local;
use colored::*;
use fern::Dispatch;
use log::{Level, LevelFilter};
use std::fs;
use std::io;
use std::path::Path;
use tokio::spawn;

/* Helper to format console output with ANSI colors */
fn console_format(out: fern::FormatCallback, message: &std::fmt::Arguments, record: &log::Record) {
    let level_color = colorize_level(record.level());
    out.finish(format_args!(
        "{} [{}] {}",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        level_color,
        message
    ))
}

/* Helper to format file output (plain text) */
fn file_format(out: fern::FormatCallback, message: &std::fmt::Arguments, record: &log::Record) {
    out.finish(format_args!(
        "{} [{}] {}",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        record.level(),
        message
    ))
}

/* Initializes the logger using the provided configuration.
   - Sets up a console logger and a file logger for bot-related logs.
   - If "hide serenity logs" is true, filters out serenity logs from the main channels and
     writes them to a separate file.
   - Schedules log rotation asynchronously.
*/
pub fn init_logger(config: &Config) -> Result<(), fern::InitError> {
    let log_level: LevelFilter = config
        .logging
        .log_level
        .parse()
        .unwrap_or(LevelFilter::Info);
    let log_dir = create_log_directory(&config.logging.directory)?;
    let bot_log_file = format!("{}/bot.log", log_dir);
    let hide_serenity = config.logging.hide_serenity_logs;

    /* Build base dispatch filtering out serenity logs if requested. */
    let base_dispatch = Dispatch::new().level(log_level).filter(move |metadata| {
        if hide_serenity {
            metadata.target() != "serenity"
        } else {
            true
        }
    });

    /* Dispatch for bot logs: file and console. */
    let file_dispatch = Dispatch::new()
        .format(file_format)
        .chain(fern::log_file(&bot_log_file)?)
        .level(log_level);
    let console_dispatch = Dispatch::new()
        .format(console_format)
        .chain(std::io::stdout())
        .level(log_level);

    let mut dispatch = base_dispatch.chain(file_dispatch).chain(console_dispatch);

    /* Optionally add a separate dispatch for serenity logs. */
    if hide_serenity {
        let serenity_log_file = format!("{}/serenity.log", log_dir);
        let serenity_dispatch = Dispatch::new()
            .filter(|metadata| metadata.target() == "serenity")
            .format(file_format)
            .chain(fern::log_file(&serenity_log_file)?)
            .level(log_level);
        dispatch = dispatch.chain(serenity_dispatch);
    }

    dispatch.apply()?;

    /* Schedule log rotation asynchronously. */
    if let Some(freq_days) = parse_rotation_frequency(&config.logrotate.frequency) {
        let log_dir_clone = config.logging.directory.clone();
        spawn(async move {
            crate::bot::utils::logrotate::schedule_log_rotation(&log_dir_clone, freq_days, "03:00")
                .await;
        });
    }

    Ok(())
}

/* Creates a log directory based on today's date. */
fn create_log_directory(base_dir: &str) -> io::Result<String> {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let log_dir = format!("{}/{}", base_dir, today);
    if !Path::new(&log_dir).exists() {
        fs::create_dir_all(&log_dir)?;
    }
    Ok(log_dir)
}

/* Returns a colored version of the log level using the colored crate. */
pub fn colorize_level(level: Level) -> ColoredString {
    match level {
        Level::Error => "ERROR".red().bold(),
        Level::Warn => "WARN".yellow().bold(),
        Level::Info => "INFO".green().bold(),
        Level::Debug => "DEBUG".blue().bold(),
        Level::Trace => "TRACE".cyan().bold(),
    }
}

/* Parses a rotation frequency string formatted as "Xd" (e.g. "7d") into a number of days. */
fn parse_rotation_frequency(frequency: &str) -> Option<u64> {
    if frequency.ends_with('d') {
        frequency[..frequency.len() - 1].parse::<u64>().ok()
    } else {
        frequency.parse::<u64>().ok()
    }
}
