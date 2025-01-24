use crate::bot::utils::logger;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;
use toml;

/// Main configuration struct holding top-level sections.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub red: RedConfig,
    pub logging: LoggingConfig,
    pub logrotate: LogRotateConfig,
}

/// Sub-configuration for the bot's token and shard count.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedConfig {
    pub token: String,
    pub shards: u64,
}

/// Sub-configuration for logging
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggingConfig {
    #[serde(rename = "log-level")]
    pub log_level: String,
    pub directory: String,
}

/// Sub-configuration for log rotation
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogRotateConfig {
    pub frequency: String,
}

/// Implementation of the Config struct.
impl Config {
    pub fn new(token: String, shards: u64) -> Self {
        Self {
            red: RedConfig { token, shards },
            logging: LoggingConfig {
                log_level: "info".to_string(),
                directory: "logs".to_string(),
            },
            logrotate: LogRotateConfig {
                frequency: "7d".to_string(),
            },
        }
    }

    /// Loads the config asynchronously, creating a default if not found.
    pub async fn load_or_create_and_validate_async() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Path::new("config.toml");

        // If the file doesn't exist, create a default config.
        if !config_path.exists() {
            Self::create_default_config_async(config_path).await?;
        }

        // Read the file and parse the contents.
        let contents = fs::read_to_string(config_path).await?;
        let parsed: Config = toml::from_str(&contents)?;

        // Validate the parsed config.
        parsed.validate();
        Ok(parsed)
    }

    /// Creates a default config file with placeholder values.
    async fn create_default_config_async(
        config_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        logger::log_warn(
            "Configuration file 'config.toml' not found. Creating a new one with default values.",
        );
        let default_config = Self::new("placeholder_token".to_string(), 1);
        let toml_str = toml::to_string_pretty(&default_config)?;
        fs::write(config_path, toml_str).await?;
        logger::log_error(
            "A new 'config.toml' has been created. Please update the 'token' field with your actual Discord bot token."
        );
        std::process::exit(1);
    }

    /// Validates the config struct, checking for placeholder values.
    fn validate(&self) {
        self.validate_token();
        self.validate_log_level();
    }

    /// Validates the token field in the config.
    fn validate_token(&self) {
        if self.red.token == "placeholder_token" {
            logger::log_error("The 'token' field in 'config.toml' is still set to 'placeholder_token'. Please replace it with your actual Discord bot token.");
            std::process::exit(1);
        }
    }

    /// Validates the log level field in the config.
    fn validate_log_level(&self) {
        match self.logging.log_level.to_lowercase().as_str() {
            "info" | "debug" | "trace" | "warn" | "error" => {}
            _ => {
                logger::log_error("The 'log-level' field in 'config.toml' is not valid. Please use one of: 'info', 'debug', 'trace', 'warn', 'error'.");
                std::process::exit(1);
            }
        }
    }
}
