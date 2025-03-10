use log::{error, warn};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;
use toml;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Config {
    pub red: RedConfig,
    pub logging: LoggingConfig,
    pub logrotate: LogRotateConfig,
    pub debug: DebugConfig,
}

/* Bot token and shard configuration */
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct RedConfig {
    #[serde(default = "default_token")]
    pub token: String,
    #[serde(default = "default_shards")]
    pub shards: u64,
}

fn default_token() -> String {
    "placeholder_token".to_string()
}

fn default_shards() -> u64 {
    1
}

impl Default for RedConfig {
    fn default() -> Self {
        RedConfig {
            token: default_token(),
            shards: default_shards(),
        }
    }
}

/* Logging configuration */
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_directory")]
    pub directory: String,
    #[serde(default = "default_extra_logs")]
    pub extra_logs: bool,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_directory() -> String {
    "logs".to_string()
}

fn default_extra_logs() -> bool {
    true
}

impl Default for LoggingConfig {
    fn default() -> Self {
        LoggingConfig {
            log_level: default_log_level(),
            directory: default_directory(),
            extra_logs: default_extra_logs(),
        }
    }
}

/* Log rotation configuration */
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct LogRotateConfig {
    #[serde(default = "default_frequency")]
    pub frequency: String,
    #[serde(default = "default_rotation_time")]
    pub rotation_time: String,
}

fn default_frequency() -> String {
    "7d".to_string()
}

fn default_rotation_time() -> String {
    "00:00".to_string()
}

impl Default for LogRotateConfig {
    fn default() -> Self {
        LogRotateConfig {
            frequency: default_frequency(),
            rotation_time: default_rotation_time(),
        }
    }
}

impl LogRotateConfig {
    /* Parses the frequency string (e.g., "7d") into a u64 */
    pub fn parse_frequency(&self) -> u64 {
        self.frequency
            .trim()
            .strip_suffix('d')
            .and_then(|s| s.parse().ok())
            .or_else(|| self.frequency.trim().parse().ok())
            .unwrap_or(7)
    }
}

/* Debug configuration */
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct DebugConfig {
    #[serde(default = "default_enable_debug")]
    pub enable_debug: bool,
    #[serde(default = "default_debug_server_id")]
    pub debug_server_id: String,
}

fn default_enable_debug() -> bool {
    false
}

fn default_debug_server_id() -> String {
    "placeholder_id".to_string()
}

impl Default for DebugConfig {
    fn default() -> Self {
        DebugConfig {
            enable_debug: default_enable_debug(),
            debug_server_id: default_debug_server_id(),
        }
    }
}

impl Config {
    /*
       Asynchronously loads the configuration from "config.toml".
       If the file is missing, it is created with default values.
       Extra keys are removed by reserializing the config, then the configuration is validated.
    */
    pub async fn load_or_create_and_validate_async() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Path::new("config.toml");

        if fs::metadata(config_path).await.is_err() {
            Self::create_default_config_async(config_path).await?;
        }

        let contents = fs::read_to_string(config_path).await?;
        let config: Self = toml::from_str(&contents).unwrap_or_else(|err| {
            warn!(
                "Failed to parse {}: {}. Using defaults.",
                config_path.display(),
                err
            );
            Self::default()
        });

        fs::write(config_path, toml::to_string_pretty(&config)?).await?;
        config.validate();
        Ok(config)
    }

    /* Asynchronously creates a default configuration file. */
    async fn create_default_config_async(
        config_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        warn!(
            "Configuration file '{}' not found. Creating default config.",
            config_path.display()
        );
        let default_config = Self::default();
        let toml_str = toml::to_string_pretty(&default_config)?;
        fs::write(config_path, toml_str).await?;
        error!(
            "Created '{}'. Please update the 'token' field with your actual Discord bot token.",
            config_path.display()
        );
        std::process::exit(1);
    }

    /* Validates the configuration by checking critical fields */
    fn validate(&self) {
        if self.red.token == "placeholder_token" {
            error!(
                "The 'token' field in 'config.toml' is set to 'placeholder_token'. Please update it."
            );
            std::process::exit(1);
        }
        let valid_levels = ["info", "debug", "trace", "warn", "error"];
        if !valid_levels.contains(&self.logging.log_level.to_lowercase().as_str()) {
            error!(
                "Invalid 'log-level' in 'config.toml'. Use one of: {}.",
                valid_levels.join(", ")
            );
            std::process::exit(1);
        }
    }
}
