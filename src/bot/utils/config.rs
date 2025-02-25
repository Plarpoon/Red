use log::{error, warn};

use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;
use toml;

/* Main configuration struct holding top-level sections */
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub red: RedConfig,
    pub logging: LoggingConfig,
    pub logrotate: LogRotateConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            red: RedConfig::default(),
            logging: LoggingConfig::default(),
            logrotate: LogRotateConfig::default(),
        }
    }
}

/* Sub-configuration for the bot's token and shard count */
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct RedConfig {
    pub token: String,
    pub shards: u64,
}

impl Default for RedConfig {
    fn default() -> Self {
        Self {
            token: "placeholder_token".to_string(),
            shards: 1,
        }
    }
}

/* Sub-configuration for logging */
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct LoggingConfig {
    #[serde(rename = "log-level")]
    pub log_level: String,
    pub directory: String,
    #[serde(rename = "hide serenity logs")]
    pub hide_serenity_logs: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            directory: "logs".to_string(),
            hide_serenity_logs: true,
        }
    }
}

/* Sub-configuration for log rotation */
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct LogRotateConfig {
    pub frequency: String,
}

impl Default for LogRotateConfig {
    fn default() -> Self {
        Self {
            frequency: "7d".to_string(),
        }
    }
}

impl Config {
    /*
       Asynchronously loads the configuration file from "config.toml".
       If the file does not exist, it is created with default values.
       The file is then reserialized to remove any extraneous keys and
       ensure that all required sections are present (using defaults where needed).
       Finally, the configuration is validated.
    */
    pub async fn load_or_create_and_validate_async() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Path::new("config.toml");

        /* Create a default config file if it does not exist */
        if !config_path.exists() {
            Self::create_default_config_async(config_path).await?;
        }

        /* Read the file and parse its contents.
           Any missing sections are filled in with defaults.
        */
        let contents = fs::read_to_string(config_path).await?;
        let config: Config = match toml::from_str(&contents) {
            Ok(cfg) => cfg,
            Err(err) => {
                warn!(
                    "Failed to parse {}: {}. Using defaults.",
                    config_path.display(),
                    err
                );
                Config::default()
            }
        };

        /* Write back a "clean" configuration (removing extra keys) */
        let clean = toml::to_string_pretty(&config)?;
        fs::write(config_path, clean).await?;

        config.validate();
        Ok(config)
    }

    /*
       Asynchronously creates a default configuration file with placeholder values.
       Logs a warning and error message, then exits the process.
    */
    async fn create_default_config_async(
        config_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        warn!(
            "Configuration file '{}' not found. Creating a new one with default values.",
            config_path.display()
        );
        let default_config = Config::default();
        let toml_str = toml::to_string_pretty(&default_config)?;
        fs::write(config_path, toml_str).await?;
        error!(
            "A new '{}' has been created. Please update the 'token' field with your actual Discord bot token.",
            config_path.display()
        );
        std::process::exit(1);
    }

    /* Validates the configuration by checking for placeholder values */
    fn validate(&self) {
        self.validate_token();
        self.validate_log_level();
    }

    /* Validates the token field.
       Exits the process if the token is still set to the placeholder.
    */
    fn validate_token(&self) {
        if self.red.token == "placeholder_token" {
            error!("The 'token' field in 'config.toml' is still set to 'placeholder_token'. Please replace it with your actual Discord bot token.");
            std::process::exit(1);
        }
    }

    /* Validates the log level field.
       Exits the process if an invalid log level is provided.
    */
    fn validate_log_level(&self) {
        match self.logging.log_level.to_lowercase().as_str() {
            "info" | "debug" | "trace" | "warn" | "error" => {}
            _ => {
                error!("The 'log-level' field in 'config.toml' is not valid. Please use one of: 'info', 'debug', 'trace', 'warn', 'error'.");
                std::process::exit(1);
            }
        }
    }
}
