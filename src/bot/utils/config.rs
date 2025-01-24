use crate::bot::utils::logger;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use toml::{self};

/// Main configuration struct holding top-level sections.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Red bot configuration
    pub red: RedConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Log rotate configuration
    pub logrotate: LogRotateConfig,
}

/// Sub-configuration defining the 'red' bot settings.
#[derive(Debug, Serialize, Deserialize)]
pub struct RedConfig {
    /// Discord bot token
    pub token: String,
    /// Number of shards to run
    pub shards: u64,
}

/// Sub-configuration for logging
#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Overall Rust log level: e.g. "info", "debug", "trace"
    #[serde(rename = "log-level")]
    pub log_level: String,
    /// Filter for the internal/external channels: e.g. "external", "internal", "both"
    #[serde(rename = "log-filter")]
    pub log_filter: String,
    /// Directory where logs should be stored.
    pub directory: String,
}

/// Sub-configuration for log rotation
#[derive(Debug, Serialize, Deserialize)]
pub struct LogRotateConfig {
    /// How often logs should rotate, e.g. "7d".
    pub frequency: String,
}

impl Config {
    /// Create a new `Config` with defaults.
    pub fn new(token: String, shards: u64) -> Self {
        Self {
            red: RedConfig { token, shards },
            logging: LoggingConfig {
                log_level: "info".to_string(),
                log_filter: "both".to_string(),
                directory: "logs".to_string(),
            },
            logrotate: LogRotateConfig {
                frequency: "7d".to_string(),
            },
        }
    }

    /// Load the config file, or create it if missing, validate and sanitize its content.
    pub fn load_or_create_and_validate() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Path::new("config.toml");

        // If config.toml doesn't exist, create a new one and exit.
        if !config_path.exists() {
            logger::log_warn(
                "Configuration file 'config.toml' not found. Creating a new one with default values.",
            );
            let default_config = Self::new("placeholder_token".to_string(), 1);
            fs::write(config_path, toml::to_string_pretty(&default_config)?)?;
            logger::log_error(
                "A new 'config.toml' has been created. Please update the 'token' field with your actual Discord bot token.",
            );
            std::process::exit(1);
        }

        // Read and parse the existing config file.
        let contents = fs::read_to_string(config_path)?;
        let parsed: Config = toml::from_str(&contents)?;

        // Validate the configuration
        parsed.validate();

        Ok(parsed)
    }

    /// Validate the configuration and handle errors appropriately.
    fn validate(&self) {
        // Check for the placeholder token
        if self.red.token == "placeholder_token" {
            logger::log_error(
                "The 'token' field in 'config.toml' is still set to 'placeholder_token'. Please replace it with your actual Discord bot token.",
            );
            std::process::exit(1);
        }

        // Validate `log-level`
        match self.logging.log_level.to_lowercase().as_str() {
            "info" | "debug" | "trace" | "warn" | "error" => {}
            _ => {
                logger::log_error(
                    "The 'log-level' field in 'config.toml' is not valid. Please use one of: 'info', 'debug', 'trace', 'warn', 'error'.",
                );
                std::process::exit(1);
            }
        }

        // Validate `log-filter`
        match self.logging.log_filter.to_lowercase().as_str() {
            "external" | "internal" | "both" => {}
            _ => {
                logger::log_error(
                    "The 'log-filter' field in 'config.toml' is not valid. Please use one of: 'external', 'internal', 'both'.",
                );
                std::process::exit(1);
            }
        }
    }
}
