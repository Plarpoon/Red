use crate::bot::utils::config::Config;
use crate::bot::utils::logrotate;

use chrono::Local;
use colored::Colorize;
use std::io;
use std::path::Path;
use tokio::fs;
use tokio::task;
use tracing::{info, Level, Subscriber};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    filter::{filter_fn, LevelFilter},
    fmt::{format::Writer, FmtContext, FormatEvent, FormatFields},
    layer::SubscriberExt,
    registry::LookupSpan,
    util::SubscriberInitExt,
    Layer,
};

/* Custom event formatter that uses our colorize_level function */
struct ColorizedFormatter;

impl<S, N> FormatEvent<S, N> for ColorizedFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        /* Get the current time */
        let now = Local::now();
        let time = now.format("%Y-%m-%d %H:%M:%S");

        /* Format the level with our colorized function */
        let level = event.metadata().level();
        let level_str = match *level {
            Level::TRACE => colorize_level("trace"),
            Level::DEBUG => colorize_level("debug"),
            Level::INFO => colorize_level("info"),
            Level::WARN => colorize_level("warn"),
            Level::ERROR => colorize_level("error"),
        };

        /* Get the target/module path */
        let target = event.metadata().target();

        /* Write the prefix first */
        write!(writer, "{} {} [{}]: ", time, level_str, target)?;

        /* Format the fields using the current API */
        ctx.field_format().format_fields(writer.by_ref(), event)?;

        /* End with a newline */
        writeln!(writer)
    }
}

/* Initializes the logger asynchronously:
 *  1. Creates a timestamped log directory for today
 *  2. Sets up tracing subscribers with:
 *     - Console layer (colorized bot logs)
 *     - red.log file layer (bot logs only)
 *     - serenity.log file layer (serenity logs only)
 *  3. Sets up log rotation
 *
 * Returns a `WorkerGuard` that must be kept alive to ensure logs are flushed.
 */
pub async fn init_logger_with_config(config: &Config) -> io::Result<Vec<WorkerGuard>> {
    /* Create daily log directory */
    let log_folder = &config.logging.directory;
    let today_dir = create_daily_log_directory(log_folder).await?;

    let mut guards = Vec::new();

    /* Parse log level from config */
    let level_filter = parse_log_level(&config.logging.log_level);

    /* Copy the hide_serenity_logs value to use in the closure */
    let hide_serenity = config.logging.hide_serenity_logs;

    /* Set up the console layer with custom colorized formatting */
    let console_layer = tracing_subscriber::fmt::layer()
        .with_writer(io::stdout)
        .event_format(ColorizedFormatter)
        .with_ansi(true)
        .with_filter(level_filter)
        .with_filter(filter_fn(move |metadata| {
            if hide_serenity {
                !metadata.target().starts_with("serenity")
            } else {
                true
            }
        }));

    /* Set up the red.log file layer */
    let bot_log_path = format!("{}/red.log", today_dir);
    let bot_file_appender = tracing_appender::rolling::never(&today_dir, "red.log");
    let (bot_writer, bot_guard) = tracing_appender::non_blocking(bot_file_appender);
    guards.push(bot_guard);

    let bot_file_layer = tracing_subscriber::fmt::layer()
        .with_writer(bot_writer)
        .with_ansi(false)
        .with_filter(level_filter)
        .with_filter(filter_fn(|metadata| {
            !metadata.target().starts_with("serenity")
        }));

    /* Set up the serenity.log file layer */
    let serenity_file_appender = tracing_appender::rolling::never(&today_dir, "serenity.log");
    let (serenity_writer, serenity_guard) = tracing_appender::non_blocking(serenity_file_appender);
    guards.push(serenity_guard);

    let serenity_file_layer = tracing_subscriber::fmt::layer()
        .with_writer(serenity_writer)
        .with_ansi(false)
        .with_filter(LevelFilter::TRACE)
        .with_filter(filter_fn(|metadata| {
            metadata.target().starts_with("serenity")
        }));

    /* Register all layers with the global subscriber */
    tracing_subscriber::registry()
        .with(console_layer)
        .with(bot_file_layer)
        .with(serenity_file_layer)
        .init();

    info!("Logger initialized with daily log directories");
    info!("Bot logs: {}", bot_log_path);

    /* Set up log rotation using configuration values */
    let base_dir = log_folder.to_string();
    let rotation_frequency_days = config.logrotate.parse_frequency();
    let rotation_time = config.logrotate.rotation_time.clone();

    info!(
        "Setting up log rotation: every {} days at {}",
        rotation_frequency_days, rotation_time
    );

    /* Spawn a task to handle log rotation in the background */
    task::spawn(async move {
        logrotate::schedule_log_rotation(&base_dir, rotation_frequency_days, &rotation_time).await;
    });

    Ok(guards)
}

/* Creates a daily timestamped log directory.
 *  Format: {base_dir}/{YYYY-MM-DD}/
 */
async fn create_daily_log_directory(base_dir: &str) -> io::Result<String> {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let log_dir = format!("{}/{}", base_dir, today);

    if !Path::new(&log_dir).exists() {
        fs::create_dir_all(&log_dir).await?;
    }

    Ok(log_dir)
}

/* Convert the user config's log_level string into a `tracing_subscriber::filter::LevelFilter`. */
fn parse_log_level(level_str: &str) -> LevelFilter {
    match level_str.to_lowercase().as_str() {
        "error" => LevelFilter::ERROR,
        "warn" => LevelFilter::WARN,
        "info" => LevelFilter::INFO,
        "debug" => LevelFilter::DEBUG,
        "trace" => LevelFilter::TRACE,
        _ => LevelFilter::TRACE, /* fallback */
    }
}

/* Formats the log level with appropriate color */
pub fn colorize_level(level: &str) -> colored::ColoredString {
    match level.to_lowercase().as_str() {
        "error" => "ERROR".red().bold(),
        "warn" => "WARN".yellow().bold(),
        "info" => "INFO".green().bold(),
        "debug" => "DEBUG".blue().bold(),
        "trace" => "TRACE".cyan().bold(),
        _ => level.normal(),
    }
}
