use crate::bot::utils::config::Config;
use crate::bot::utils::log::logrotate;
use chrono::Local;
use colored::Colorize;
use fern::Dispatch;
use log::{Level, LevelFilter, Metadata, Record, info, warn};
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
        /* Append incoming data to the internal buffer */
        self.buffer.extend_from_slice(buf);
        /* Process complete lines */
        while let Some(pos) = self.buffer.iter().position(|&b| b == b'\n') {
            /* Drain one complete line (including the newline) */
            let line_bytes: Vec<u8> = self.buffer.drain(..=pos).collect();
            let line_str = String::from_utf8_lossy(&line_bytes).trim().to_string();
            if !line_str.is_empty() {
                self.inner.write_all(&line_bytes)?;
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        /* Flush any remaining data that does not end with a newline */
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

/* Returns the current timestamp formatted as "YYYY-MM-DD HH:MM:SS". */
fn current_timestamp() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/* Helper function to create a boxed NoEmptyLineWriter from a given writer. */
fn create_boxed_writer<W: Write + Send + 'static>(writer: W) -> Box<dyn Write + Send> {
    Box::new(NoEmptyLineWriter::new(writer))
}

/* Returns true if the log record’s message exactly matches one of the heartbeat words. */
fn is_heartbeat(record: &Record) -> bool {
    const HEARTBEAT_WORDS: &[&str] = &[
        "into_future;",
        "start;",
        "shutdown_all;",
        "initialize;",
        "run;",
        "latency;",
        "check_last_start;",
        "recv;",
        "do_heartbeat;",
        "recv_event;",
        "resume;",
        "update_manager;",
        "action;",
        "identify;",
        "heartbeat;",
    ];
    let msg = format!("{}", record.args());
    HEARTBEAT_WORDS.contains(&msg.as_str())
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

/* Creates a log directory for the current date */
async fn create_log_directory(base_dir: &str) -> io::Result<String> {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let log_dir = format!("{}/{}", base_dir, today);
    if fs::metadata(&log_dir).await.is_err() {
        fs::create_dir_all(&log_dir).await?;
    }
    Ok(log_dir)
}

/* Initializes the logger based on the provided configuration */
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

    /* Create a log directory for today */
    let log_dir = create_log_directory(&config.logging.directory).await?;
    let red_log_path = format!("{}/red.log", log_dir);
    let serenity_log_path = format!("{}/serenity.log", log_dir);
    let heartbeat_log_path = format!("{}/heartbeat.log", log_dir);

    /* Define non-serenity console formatting with colored log levels */
    let non_serenity_console_format =
        move |out: fern::FormatCallback, message: &std::fmt::Arguments, record: &Record| {
            if is_heartbeat(record) {
                return out.finish(format_args!(""));
            }
            let level_color = colorize_level(record.level());
            out.finish(format_args!(
                "{} [{}] {}",
                current_timestamp(),
                level_color,
                message
            ))
        };

    /* Define non-serenity file formatting without colors */
    let non_serenity_file_format =
        move |out: fern::FormatCallback, message: &std::fmt::Arguments, record: &Record| {
            if is_heartbeat(record) {
                return out.finish(format_args!(""));
            }
            out.finish(format_args!(
                "{} [{}] {}",
                current_timestamp(),
                record.level(),
                message
            ))
        };

    /* Define general file formatting */
    let file_format =
        move |out: fern::FormatCallback, message: &std::fmt::Arguments, record: &Record| {
            out.finish(format_args!(
                "{} [{}] {}",
                current_timestamp(),
                record.level(),
                message
            ))
        };

    /* Define heartbeat file formatting */
    let heartbeat_file_format =
        move |out: fern::FormatCallback, message: &std::fmt::Arguments, record: &Record| {
            if record.target() != "heartbeat" && !is_heartbeat(record) {
                return out.finish(format_args!(""));
            }
            out.finish(format_args!(
                "{} [{}] {}",
                current_timestamp(),
                record.level(),
                message
            ))
        };

    /* Define filters using Metadata */
    let non_serenity_filter = |metadata: &Metadata| !metadata.target().starts_with("serenity");
    let serenity_filter = |metadata: &Metadata| {
        metadata.target().starts_with("serenity") && metadata.level() >= Level::Warn
    };

    /* Wrap the writers so that empty lines are not written */
    let stdout_writer = create_boxed_writer(std::io::stdout());
    let red_file_writer = create_boxed_writer(fern::log_file(&red_log_path)?);

    /* Conditionally set up extra log file writers if extra_logs is true */
    let extra_logs = config.logging.extra_logs;
    let serenity_chain = if extra_logs {
        Some(
            Dispatch::new()
                .filter(serenity_filter)
                .format(file_format)
                .chain(create_boxed_writer(fern::log_file(&serenity_log_path)?)),
        )
    } else {
        None
    };
    let heartbeat_chain = if extra_logs {
        Some(
            Dispatch::new()
                .format(heartbeat_file_format)
                .chain(create_boxed_writer(fern::log_file(&heartbeat_log_path)?)),
        )
    } else {
        None
    };

    /* Build the dispatcher */
    let mut dispatch = Dispatch::new()
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
        );

    if let Some(serenity_disp) = serenity_chain {
        dispatch = dispatch.chain(serenity_disp);
    }
    if let Some(heartbeat_disp) = heartbeat_chain {
        dispatch = dispatch.chain(heartbeat_disp);
    }

    dispatch.apply()?;

    if extra_logs {
        warn!(target: "serenity", "Logging initialized.");
        warn!(target: "heartbeat", "Logging initialized.");
    }

    info!("Logger initialized with level {:?}", log_level);
    info!("Logging to file: {}", red_log_path);
    if extra_logs {
        info!("Serenity logs to file: {}", serenity_log_path);
        info!("Heartbeat logs to file: {}", heartbeat_log_path);
    } else {
        info!("Extra logs disabled; serenity.log and heartbeat.log will not be written.");
    }

    /* Spawn asynchronous log rotation task */
    let base_dir = config.logging.directory.clone();
    let rotation_frequency = config.logrotate.parse_frequency();
    let rotation_time = config.logrotate.rotation_time.clone();
    tokio::spawn(async move {
        logrotate::schedule_log_rotation(&base_dir, rotation_frequency, &rotation_time).await;
    });

    Ok(())
}
