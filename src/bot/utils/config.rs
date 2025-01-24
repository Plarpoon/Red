use crate::bot::utils::logger;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use toml::{self};

/// Main configuration struct holding top-level sections.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub red: RedConfig,
    pub logging: LoggingConfig,
    pub logrotate: LogRotateConfig,
}

/// Sub-configuration defining the 'red' bot settings.
#[derive(Debug, Serialize, Deserialize)]
pub struct RedConfig {
    pub token: String,
    pub shards: u64,
}

/// Sub-configuration for logging
#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(rename = "log-level")]
    pub log_level: String,
    #[serde(rename = "log-filter")]
    pub log_filter: String,
    pub directory: String,
}

/// Sub-configuration for log rotation
#[derive(Debug, Serialize, Deserialize)]
pub struct LogRotateConfig {
    pub frequency: String,
}

impl Config {
    pub fn new(token: String, shards: u64) -> Self {
        Self {
            red: RedConfig { token, shards },
            logging: LoggingConfig {
                log_level: "info".to_string(),
                // We can still default to "both" if we want everything by default.
                log_filter: "both".to_string(),
                directory: "logs".to_string(),
            },
            logrotate: LogRotateConfig {
                frequency: "7d".to_string(),
            },
        }
    }

    pub fn load_or_create_and_validate() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Path::new("config.toml");

        if !config_path.exists() {
            Self::create_default_config(config_path)?;
        }

        let contents = fs::read_to_string(config_path)?;
        let parsed: Config = toml::from_str(&contents)?;

        parsed.validate();

        Ok(parsed)
    }

    fn create_default_config(config_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        logger::log_warn(
            "Configuration file 'config.toml' not found. Creating a new one with default values.",
        );
        let default_config = Self::new("placeholder_token".to_string(), 1);
        fs::write(config_path, toml::to_string_pretty(&default_config)?)?;
        logger::log_error(
            "A new 'config.toml' has been created. Please update the 'token' field with your actual Discord bot token."
        );
        std::process::exit(1);
    }

    fn validate(&self) {
        self.validate_token();
        self.validate_log_level();
        self.validate_log_filter();
    }

    fn validate_token(&self) {
        if self.red.token == "placeholder_token" {
            logger::log_error("The 'token' field in 'config.toml' is still set to 'placeholder_token'. Please replace it with your actual Discord bot token.");
            std::process::exit(1);
        }
    }

    fn validate_log_level(&self) {
        match self.logging.log_level.to_lowercase().as_str() {
            "info" | "debug" | "trace" | "warn" | "error" => {}
            _ => {
                logger::log_error("The 'log-level' field in 'config.toml' is not valid. Please use one of: 'info', 'debug', 'trace', 'warn', 'error'.");
                std::process::exit(1);
            }
        }
    }

    fn validate_log_filter(&self) {
        match self.logging.log_filter.to_lowercase().as_str() {
            "internal" | "external" | "both" => {}
            _ => {
                logger::log_error("The 'log-filter' field in 'config.toml' is not valid. Please use one of: 'internal', 'external', 'both'.");
                std::process::exit(1);
            }
        }
    }
}
