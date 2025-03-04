use crate::bot::utils::config::Config;
use crate::bot::utils::log::logrotate;

use chrono::Local;
use colored::Colorize;
use fern::Dispatch;
use log::{Level, LevelFilter, Metadata, Record, info};
use std::io::{self, Write};
use tokio::fs;

/* A writer wrapper that filters out empty lines.
   It buffers incoming data until a newline is found and then writes the line
   only if it is not empty after trimming.
*/
struct NoEmptyLineWriter<W: Write> {
    inner: W,
    buffer: Vec<u8>,
}

impl<W: Write> NoEmptyLineWriter<W> {
    /* Creates a new NoEmptyLineWriter wrapping the given writer */
    fn new(inner: W) -> Self {
        Self {
            inner,
            buffer: Vec::new(),
        }
    }
}

impl<W: Write> Write for NoEmptyLineWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        /* Append incoming data to the internal buffer. */
        self.buffer.extend_from_slice(buf);
        /* Process complete lines. */
        while let Some(pos) = self.buffer.iter().position(|&b| b == b'\n') {
            /* Drain one complete line (including the newline). */
            let line_bytes: Vec<u8> = self.buffer.drain(..=pos).collect();
            /* Convert to string and trim whitespace. */
            let line_str = String::from_utf8_lossy(&line_bytes).trim().to_string();
            /* Write only non-empty lines. */
            if !line_str.is_empty() {
                self.inner.write_all(&line_bytes)?;
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        /* Flush any remaining data that does not end with a newline. */
        if !self.buffer.is_empty() {
            let line_str = String::from_utf8_lossy(&self.buffer).trim().to_string();
            if !line_str.is_empty() {
                self.inner.write_all(&self.buffer)?;
            }
            self.buffer.clear();
        }
        self.inner.flush()
    }
}

/* Returns true if the log record’s message exactly matches one of the spam words. */
fn is_spam(record: &Record) -> bool {
    let msg = format!("{}", record.args());
    matches!(
        msg.as_str(),
        "into_future;"
            | "start;"
            | "shutdown_all;"
            | "initialize;"
            | "run;"
            | "check_last_start;"
            | "recv;"
            | "do_heartbeat;"
            | "recv_event;"
            | "update_manager;"
            | "action;"
            | "identify;"
    )
}

/* Initializes the logger based on the provided configuration.
   Sets up logging to both a file and stdout, and spawns an asynchronous task for log rotation.
*/
pub async fn init_logger_with_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    /* Determine log level from configuration */
    let log_level = match config.logging.log_level.to_lowercase().as_str() {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info,
    };

    /* Create a log directory for today using Tokio's async file operations */
    let log_dir = create_log_directory(&config.logging.directory).await?;
    let red_log_path = format!("{}/red.log", log_dir);
    let serenity_log_path = format!("{}/serenity.log", log_dir);

    /* Define non-serenity console formatting with colored log levels.
       If the record is spam, output an empty string.
    */
    let non_serenity_console_format =
        move |out: fern::FormatCallback, message: &std::fmt::Arguments, record: &Record| {
            if is_spam(record) {
                out.finish(format_args!(""))
            } else {
                let level_color = colorize_level(record.level());
                out.finish(format_args!(
                    "{} [{}] {}",
                    Local::now().format("%Y-%m-%d %H:%M:%S"),
                    level_color,
                    message
                ))
            }
        };

    /* Define non-serenity file formatting (without colors).
       If the record is spam, output an empty string.
    */
    let non_serenity_file_format =
        move |out: fern::FormatCallback, message: &std::fmt::Arguments, record: &Record| {
            if is_spam(record) {
                out.finish(format_args!(""))
            } else {
                out.finish(format_args!(
                    "{} [{}] {}",
                    Local::now().format("%Y-%m-%d %H:%M:%S"),
                    record.level(),
                    message
                ))
            }
        };

    /* Define file formatting for other chains (no spam filtering here) */
    let file_format =
        move |out: fern::FormatCallback, message: &std::fmt::Arguments, record: &Record| {
            out.finish(format_args!(
                "{} [{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        };

    /* Define filters using Metadata.
       For non-serenity chains, we use a simple metadata filter.
       For serenity logs, we require that the target starts with "serenity" and the level is Warning or higher.
    */
    let non_serenity_filter = |metadata: &Metadata| !metadata.target().starts_with("serenity");
    let serenity_filter = |metadata: &Metadata| {
        metadata.target().starts_with("serenity") && metadata.level() >= Level::Warn
    };

    /* Wrap the writers so that empty lines are not written.
       We wrap the underlying writer in a Box<dyn Write + Send> to satisfy fern's requirements.
    */
    let stdout_writer =
        Box::new(NoEmptyLineWriter::new(std::io::stdout())) as Box<dyn Write + Send>;
    let red_file_writer =
        Box::new(NoEmptyLineWriter::new(fern::log_file(&red_log_path)?)) as Box<dyn Write + Send>;
    let serenity_file_writer = Box::new(NoEmptyLineWriter::new(fern::log_file(&serenity_log_path)?))
        as Box<dyn Write + Send>;

    /* Set up the fern logger with separate chains for non-serenity and serenity logs */
    Dispatch::new()
        .level(log_level)
        .chain(
            Dispatch::new()
                .filter(non_serenity_filter)
                .format(non_serenity_file_format)
                .chain(red_file_writer),
        )
        .chain(
            Dispatch::new()
                .filter(non_serenity_filter)
                .format(non_serenity_console_format)
                .chain(stdout_writer),
        )
        .chain(
            Dispatch::new()
                .filter(serenity_filter)
                .format(file_format)
                .chain(serenity_file_writer),
        )
        .apply()?;

    info!("Logger initialized with level {:?}", log_level);
    info!("Logging to file: {}", red_log_path);
    info!("Serenity logs to file: {}", serenity_log_path);

    /* Spawn asynchronous log rotation task */
    let base_dir = config.logging.directory.clone();
    let rotation_frequency = config.logrotate.parse_frequency();
    let rotation_time = config.logrotate.rotation_time.clone();
    tokio::spawn(async move {
        logrotate::schedule_log_rotation(&base_dir, rotation_frequency, &rotation_time).await;
    });

    Ok(())
}

/* Creates a log directory for the current date using Tokio's async file operations */
async fn create_log_directory(base_dir: &str) -> io::Result<String> {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let log_dir = format!("{}/{}", base_dir, today);
    if fs::metadata(&log_dir).await.is_err() {
        fs::create_dir_all(&log_dir).await?;
    }
    Ok(log_dir)
}

/* Returns a colored string for the log level */
fn colorize_level(level: Level) -> colored::ColoredString {
    match level {
        Level::Error => "ERROR".red().bold(),
        Level::Warn => "WARN".yellow().bold(),
        Level::Info => "INFO".green().bold(),
        Level::Debug => "DEBUG".blue().bold(),
        Level::Trace => "TRACE".cyan().bold(),
    }
}
