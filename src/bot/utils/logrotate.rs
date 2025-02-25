use chrono::{Duration as ChronoDuration, Local, NaiveTime};
use log;
use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};
use tokio::time;

/* Asynchronously schedules log rotation in an infinite loop.
   It calculates the next rotation time, waits until that time,
   and then rotates logs by deleting old log directories.
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
        let today_rotation_time = now.date_naive().and_time(rotation_time);
        let next_rotation_time = if now.time() < rotation_time {
            today_rotation_time
        } else {
            today_rotation_time + ChronoDuration::days(1)
        };
        let duration_until_rotation = (next_rotation_time - now.naive_local())
            .to_std()
            .unwrap_or(Duration::ZERO);
        log::info!(
            "Next log rotation scheduled at {} (in {:?}).",
            next_rotation_time,
            duration_until_rotation
        );
        time::sleep(duration_until_rotation).await;
        match rotate_logs(base_dir, rotation_frequency_days) {
            Ok(_) => log::info!("Log rotation completed successfully."),
            Err(e) => log::error!("Log rotation failed: {}", e),
        }
    }
}

/* Parses a rotation time string formatted as "HH:MM" into a NaiveTime.
   Returns Some(NaiveTime) if parsing succeeds; otherwise, returns None.
*/
fn parse_rotation_time(rotation_time: &str) -> Option<NaiveTime> {
    let parts: Vec<&str> = rotation_time.split(':').collect();
    if parts.len() == 2 {
        if let (Ok(hour), Ok(minute)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
            return NaiveTime::from_hms_opt(hour, minute, 0);
        }
    }
    None
}

/* Rotates logs by deleting directories older than the specified number of days.
   It checks each subdirectory in the base directory and removes those that are too old.
*/
pub fn rotate_logs(base_dir: &str, rotation_frequency_days: u64) -> std::io::Result<()> {
    log::info!("Log rotation has started.");
    let base_path = Path::new(base_dir);
    let allowed_duration = Duration::from_secs(rotation_frequency_days * 24 * 60 * 60);
    log::info!(
        "Logs older than {} days will be deleted.",
        rotation_frequency_days
    );
    if !base_path.exists() {
        log::info!("Base directory does not exist. Exiting log rotation.");
        return Ok(());
    }
    for entry in fs::read_dir(base_path)? {
        let entry = entry?;
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(e) => {
                log::warn!(
                    "Failed to retrieve metadata for {:?}. Error: {}",
                    entry.path(),
                    e
                );
                continue;
            }
        };
        if !metadata.is_dir() {
            continue;
        }
        let directory_name = entry
            .file_name()
            .into_string()
            .unwrap_or_else(|_| "Unknown".to_string());
        log::info!("Checking log directory: {}", directory_name);
        let modified = match metadata.modified() {
            Ok(m) => m,
            Err(e) => {
                log::warn!(
                    "Failed to retrieve modified time for {}. Error: {}",
                    directory_name,
                    e
                );
                continue;
            }
        };
        let duration_since_modified = match SystemTime::now().duration_since(modified) {
            Ok(d) => d,
            Err(e) => {
                log::warn!(
                    "Failed to calculate duration since modified for {}. Error: {}",
                    directory_name,
                    e
                );
                continue;
            }
        };
        if duration_since_modified > allowed_duration {
            log::warn!("Deleting log directory: {}", directory_name);
            if let Err(e) = fs::remove_dir_all(entry.path()) {
                log::warn!("Failed to delete {}. Error: {}", directory_name, e);
            }
        } else {
            log::info!("Keeping log directory: {}", directory_name);
        }
    }
    Ok(())
}
