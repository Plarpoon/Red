use chrono::{Duration as ChronoDuration, Local, NaiveDate, NaiveTime};
use log;
use std::path::Path;
use std::time::Duration;
use tokio::fs;
use tokio::time;

/* Calculates the next rotation time based on the current time and a configured rotation time */
fn get_next_rotation_time(
    now: chrono::DateTime<Local>,
    rotation_time: NaiveTime,
) -> chrono::NaiveDateTime {
    let today_rt = now.date_naive().and_time(rotation_time);
    if now.time() < rotation_time {
        today_rt
    } else {
        today_rt + ChronoDuration::days(1)
    }
}

/* Parses a rotation time string formatted as "HH:MM" into a NaiveTime */
fn parse_rotation_time(rotation_time: &str) -> Option<NaiveTime> {
    let parts: Vec<&str> = rotation_time.split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    let hour = parts[0].parse::<u32>().ok()?;
    let minute = parts[1].parse::<u32>().ok()?;
    NaiveTime::from_hms_opt(hour, minute, 0)
}

/* Asynchronously processes a single directory entry.
   Valid entries:
     - Files: keep if the name is "heartbeat.log", "red.log", or "serenity.log"; otherwise, delete.
     - Directories: keep if the directory name is a valid date ("YYYY-MM-DD") and its age is less than the rotation limit;
       otherwise, delete.
*/
async fn process_entry_async(
    entry: &fs::DirEntry,
    rotation_limit: ChronoDuration,
    today: NaiveDate,
) -> std::io::Result<()> {
    let path = entry.path();
    let file_name = entry
        .file_name()
        .into_string()
        .unwrap_or_else(|_| "InvalidName".to_string());
    let metadata = entry.metadata().await?;

    if !metadata.is_dir() {
        if file_name == "heartbeat.log" || file_name == "red.log" || file_name == "serenity.log" {
            log::info!("Keeping valid log file: {}", file_name);
            return Ok(());
        }
        log::warn!("Deleting unwanted file: {}", file_name);
        return fs::remove_file(&path).await;
    }

    match NaiveDate::parse_from_str(&file_name, "%Y-%m-%d") {
        Ok(dir_date) => {
            if today.signed_duration_since(dir_date) >= rotation_limit {
                log::warn!("Deleting log directory: {}", file_name);
                fs::remove_dir_all(&path).await
            } else {
                log::info!("Keeping log directory: {}", file_name);
                Ok(())
            }
        }
        Err(_) => {
            log::warn!("Deleting directory with invalid date name: {}", file_name);
            fs::remove_dir_all(&path).await
        }
    }
}

/* Asynchronously rotates logs by deleting unwanted entries inside the base directory */
async fn rotate_logs_async(base_dir: &str, rotation_frequency_days: u64) -> std::io::Result<()> {
    log::info!("Log rotation has started.");
    log::info!(
        "Deleting log entries older than {} days or invalid.",
        rotation_frequency_days
    );

    let base_path = Path::new(base_dir);
    if fs::metadata(base_path).await.is_err() {
        log::info!("Base directory does not exist. Exiting log rotation.");
        return Ok(());
    }

    let rotation_limit = ChronoDuration::days(rotation_frequency_days as i64);
    let today = Local::now().date_naive();
    let mut read_dir = fs::read_dir(base_path).await?;

    while let Some(entry) = read_dir.next_entry().await? {
        process_entry_async(&entry, rotation_limit, today).await?;
    }
    Ok(())
}

/* Asynchronously schedules log rotation in an infinite loop.
   It performs an immediate rotation, then calculates the next rotation time,
   sleeps until then, and rotates logs by deleting unwanted files and directories.
*/
pub async fn schedule_log_rotation(
    base_dir: &str,
    rotation_frequency_days: u64,
    rotation_time_str: &str,
) {
    let rotation_time =
        parse_rotation_time(rotation_time_str).expect("Invalid rotation time format");

    /* Perform initial rotation immediately */
    match rotate_logs_async(base_dir, rotation_frequency_days).await {
        Ok(()) => log::info!("Initial log rotation completed successfully."),
        Err(e) => log::error!("Initial log rotation failed: {}", e),
    }

    loop {
        let now = Local::now();
        let next_rotation = get_next_rotation_time(now, rotation_time);
        let sleep_duration = (next_rotation - now.naive_local())
            .to_std()
            .unwrap_or(Duration::ZERO);
        log::info!(
            "Next log rotation scheduled at {} (in {:?}).",
            next_rotation,
            sleep_duration
        );
        time::sleep(sleep_duration).await;

        match rotate_logs_async(base_dir, rotation_frequency_days).await {
            Ok(()) => log::info!("Log rotation completed successfully."),
            Err(e) => log::error!("Log rotation failed: {}", e),
        }
    }
}
