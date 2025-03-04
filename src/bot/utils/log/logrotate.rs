use chrono::{Duration as ChronoDuration, Local, NaiveTime};
use log;
use std::path::Path;
use std::time::{Duration, SystemTime};
use tokio::fs;
use tokio::time;

/* Calculates the next rotation time based on the current time and a configured rotation time. */
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

/* Asynchronously schedules log rotation in an infinite loop.
   It calculates the next rotation time, sleeps until then, and rotates logs by deleting old directories.
*/
pub async fn schedule_log_rotation(
    base_dir: &str,
    rotation_frequency_days: u64,
    rotation_time_str: &str,
) {
    let rotation_time =
        parse_rotation_time(rotation_time_str).expect("Invalid rotation time format");
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
        if let Err(e) = rotate_logs_async(base_dir, rotation_frequency_days).await {
            log::error!("Log rotation failed: {}", e);
        } else {
            log::info!("Log rotation completed successfully.");
        }
    }
}

/* Parses a rotation time string formatted as "HH:MM" into a NaiveTime. */
fn parse_rotation_time(rotation_time: &str) -> Option<NaiveTime> {
    let parts: Vec<&str> = rotation_time.split(':').collect();
    if parts.len() == 2 {
        if let (Ok(hour), Ok(minute)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
            return NaiveTime::from_hms_opt(hour, minute, 0);
        }
    }
    None
}

/* Asynchronously rotates logs by deleting directories older than the specified number of days.
   Iterates over each subdirectory in the base directory.
*/
async fn rotate_logs_async(base_dir: &str, rotation_frequency_days: u64) -> std::io::Result<()> {
    log::info!("Log rotation has started.");
    let base_path = Path::new(base_dir);
    let allowed_duration = Duration::from_secs(rotation_frequency_days * 24 * 60 * 60);
    log::info!(
        "Logs older than {} days will be deleted.",
        rotation_frequency_days
    );

    if fs::metadata(base_path).await.is_err() {
        log::info!("Base directory does not exist. Exiting log rotation.");
        return Ok(());
    }

    let mut read_dir = fs::read_dir(base_path).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        if let Err(e) = process_entry_async(&entry, allowed_duration).await {
            log::warn!("Failed to process {:?}: {}", entry.path(), e);
        }
    }
    Ok(())
}

/* Asynchronously processes a single directory entry:
   - If the entry is not a directory, it is skipped.
   - If the directory's last modified time is older than the allowed duration, it is deleted.
*/
async fn process_entry_async(
    entry: &fs::DirEntry,
    allowed_duration: Duration,
) -> std::io::Result<()> {
    let metadata = entry.metadata().await?;
    if !metadata.is_dir() {
        return Ok(());
    }
    let directory_name = entry
        .file_name()
        .into_string()
        .unwrap_or_else(|_| "Unknown".to_string());
    log::info!("Checking log directory: {}", directory_name);

    let modified = metadata.modified()?;
    let elapsed = SystemTime::now()
        .duration_since(modified)
        .unwrap_or(Duration::ZERO);
    if elapsed > allowed_duration {
        log::warn!("Deleting log directory: {}", directory_name);
        fs::remove_dir_all(entry.path()).await?;
    } else {
        log::info!("Keeping log directory: {}", directory_name);
    }
    Ok(())
}
