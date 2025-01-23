use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use toml::{self, Value};

/// Main configuration struct holding top-level sections.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Red bot configuration
    pub red: RedConfig,
}

/// Sub-configuration defining the 'red' bot settings.
#[derive(Debug, Serialize, Deserialize)]
pub struct RedConfig {
    /// Discord bot token
    pub token: String,
    /// Number of shards to run
    pub shards: u64,
}

impl Config {
    /// Create a new `Config`.
    pub fn new(token: String, shards: u64) -> Self {
        Self {
            red: RedConfig { token, shards },
        }
    }

    /// Load the config file, or create it if missing, validate and sanitize its content.
    pub fn load_or_create_and_validate() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Path::new("config.toml");

        // If config.toml doesn't exist, create a new one and exit.
        if !config_path.exists() {
            println!(
                "Configuration file 'config.toml' not found. Creating a new one with default values."
            );
            let default_config = Self::new("placeholder_token".to_string(), 1);
            fs::write(config_path, toml::to_string_pretty(&default_config)?)?;
            eprintln!(
                "A new 'config.toml' has been created. Please update the 'token' field with your actual Discord bot token.\nRefer to this guide for help: https://www.discordbothosting.com/help/bots/bottoken"
            );
            std::process::exit(1);
        }

        // Read and parse the existing config file.
        let contents = fs::read_to_string(config_path)?;
        let mut parsed: Value = match toml::from_str(&contents) {
            Ok(val) => val,
            Err(_) => {
                eprintln!(
                    "The 'config.toml' file is malformed. Attempting to fix it by regenerating missing or incorrect values."
                );
                Value::Table(toml::map::Map::new())
            }
        };

        // Ensure known sections exist and sanitize their content.
        let table = parsed
            .as_table_mut()
            .ok_or("Root of config is not a TOML table")?;
        let known_sections = ["red"];
        table.retain(|k, _| known_sections.contains(&&k[..]));

        // Ensure "red" section exists.
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

        // Write the sanitized TOML back to the file.
        fs::write(config_path, toml::to_string_pretty(&parsed)?)?;

        // Convert it to our Config struct.
        let config: Config = toml::from_str(&toml::to_string(&parsed)?)?;
        config.validate();

        Ok(config)
    }

    /// Validate the configuration and handle errors appropriately.
    fn validate(&self) {
        if self.red.token == "placeholder_token" {
            eprintln!(
                "The 'token' field in 'config.toml' is still set to 'placeholder_token'. Please replace it with your actual Discord bot token.\nRefer to this guide for help: https://www.discordbothosting.com/help/bots/bottoken"
            );
            std::process::exit(1);
        }
    }
}
