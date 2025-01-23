use crate::bot::utils::logger;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use toml::{self, Value};

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
    /// Filter for the internal/external channels: e.g. "Internal", "External", "Both"
    #[serde(rename = "log-filter")]
    pub log_filter: String,
    /// Directory where logs should be stored. (Not yet implemented)
    pub directory: String,
}

/// Sub-configuration for log rotation
#[derive(Debug, Serialize, Deserialize)]
pub struct LogRotateConfig {
    /// How often logs should rotate, e.g. "7d".
    /// Not yet implemented – for future use.
    pub frequency: String,
}

impl Config {
    /// Create a new `Config` with defaults.
    pub fn new(token: String, shards: u64) -> Self {
        Self {
            red: RedConfig { token, shards },
            logging: LoggingConfig {
                log_level: "info".to_string(),
                log_filter: "Both".to_string(),
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
                "A new 'config.toml' has been created. Please update the 'token' field with your actual Discord bot token. Refer to this guide for help: https://www.discordbothosting.com/help/bots/bottoken",
            );
            std::process::exit(1);
        }

        // Read and parse the existing config file.
        let contents = fs::read_to_string(config_path)?;
        let mut parsed: Value = match toml::from_str(&contents) {
            Ok(val) => val,
            Err(_) => {
                logger::log_warn(
                    "The 'config.toml' file is malformed. Attempting to fix it by regenerating \
                     missing or incorrect values.",
                );
                Value::Table(toml::map::Map::new())
            }
        };

        // We'll ensure known sections exist: "red", "logging", and "logrotate"
        let table = parsed
            .as_table_mut()
            .ok_or("Root of config is not a TOML table")?;

        let known_sections = ["red", "logging", "logrotate"];
        table.retain(|k, _| known_sections.contains(&&k[..]));

        //---------------------------------------------------------
        // 1) "red" section
        //---------------------------------------------------------
        let red_section = table
            .entry("red")
            .or_insert_with(|| Value::Table(toml::map::Map::new()));
        if let Some(red_table) = red_section.as_table_mut() {
            let known_fields = ["token", "shards"];
            red_table.retain(|k, _| known_fields.contains(&&k[..]));

            // Insert missing fields with default values.
            if !red_table.contains_key("token") {
                red_table.insert(
                    "token".into(),
                    Value::String("placeholder_token".to_string()),
                );
            }
            if !red_table.contains_key("shards") {
                red_table.insert("shards".into(), Value::Integer(1));
            }

            // Ensure types are correct.
            if let Some(token) = red_table.get_mut("token") {
                if !token.is_str() {
                    *token = Value::String("placeholder_token".to_string());
                }
            }
            if let Some(shards) = red_table.get_mut("shards") {
                if !shards.is_integer() {
                    *shards = Value::Integer(1);
                }
            }
        }

        //---------------------------------------------------------
        // 2) "logging" section
        //---------------------------------------------------------
        let logging_section = table
            .entry("logging")
            .or_insert_with(|| Value::Table(toml::map::Map::new()));
        if let Some(logging_table) = logging_section.as_table_mut() {
            // We expect: log-level, log-filter, directory
            let known_fields = ["log-level", "log-filter", "directory"];
            logging_table.retain(|k, _| known_fields.contains(&&k[..]));

            // Insert missing fields with default values
            if !logging_table.contains_key("log-level") {
                logging_table.insert("log-level".into(), Value::String("info".into()));
            }
            if !logging_table.contains_key("log-filter") {
                logging_table.insert("log-filter".into(), Value::String("Both".into()));
            }
            if !logging_table.contains_key("directory") {
                logging_table.insert("directory".into(), Value::String("logs".into()));
            }

            // Ensure each is the correct type
            if let Some(log_level) = logging_table.get_mut("log-level") {
                if !log_level.is_str() {
                    *log_level = Value::String("info".to_string());
                }
            }
            if let Some(log_filter) = logging_table.get_mut("log-filter") {
                if !log_filter.is_str() {
                    *log_filter = Value::String("Both".to_string());
                }
            }
            if let Some(directory) = logging_table.get_mut("directory") {
                if !directory.is_str() {
                    *directory = Value::String("logs".to_string());
                }
            }
        }

        //---------------------------------------------------------
        // 3) "logrotate" section
        //---------------------------------------------------------
        let logrotate_section = table
            .entry("logrotate")
            .or_insert_with(|| Value::Table(toml::map::Map::new()));
        if let Some(lr_table) = logrotate_section.as_table_mut() {
            // We expect: frequency
            let known_fields = ["frequency"];
            lr_table.retain(|k, _| known_fields.contains(&&k[..]));

            // Insert missing
            if !lr_table.contains_key("frequency") {
                lr_table.insert("frequency".into(), Value::String("7d".into()));
            }

            // Make sure frequency is a string
            if let Some(freq) = lr_table.get_mut("frequency") {
                if !freq.is_str() {
                    *freq = Value::String("7d".to_string());
                }
            }
        }

        // Now write the sanitized TOML back to the file so we keep it consistent
        fs::write(config_path, toml::to_string_pretty(&parsed)?)?;

        // Convert from `Value` to our actual `Config` struct
        let config: Config = toml::from_str(&toml::to_string(&parsed)?)?;
        config.validate();

        Ok(config)
    }

    /// Validate the configuration and handle errors appropriately.
    fn validate(&self) {
        // Check for the placeholder token
        if self.red.token == "placeholder_token" {
            logger::log_error(
                "The 'token' field in 'config.toml' is still set to 'placeholder_token'. \
                 Please replace it with your actual Discord bot token. \
                 Refer to this guide for help: \
                 https://www.discordbothosting.com/help/bots/bottoken",
            );
            std::process::exit(1);
        }

        // e.g., ensure log-level is one of the known strings: ["info", "debug", ...].
        match self.logging.log_level.to_lowercase().as_str() {
            "info" | "debug" | "trace" => {}
            _ => {
                logger::log_error(
                    "The 'log-level' field in 'config.toml' is not a valid value. \
                     Please use one of: 'info', 'debug', 'trace'.",
                );
                std::process::exit(1);
            }
        }

        // e.g., ensure log-filter is one of the known strings: ["Internal", "External", "Both"].
        match self.logging.log_filter.to_lowercase().as_str() {
            "internal" | "external" | "both" => {}
            _ => {
                logger::log_error(
                    "The 'log-filter' field in 'config.toml' is not a valid value. \
                     Please use one of: 'Internal', 'External', 'Both'.",
                );
                std::process::exit(1);
            }
        }
    }
}
