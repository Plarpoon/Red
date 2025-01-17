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

    /// Load the config file, or create it if missing.
    ///
    /// 1) If file missing, create a fresh config with provided defaults.
    /// 2) If file is present but invalid, recreate a fresh config.
    /// 3) If file is valid, remove unknown top-level sections,
    pub fn load_or_create(token: String, shards: u64) -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Path::new("config.toml");

        // If config.toml doesn't exist, create new from defaults.
        if !config_path.exists() {
            return Self::create_fresh_config(config_path, token, shards);
        }

        // Try reading the file
        let contents = fs::read_to_string(config_path)?;
        // Parse into a TOML Value
        let mut parsed: Value = match toml::from_str(&contents) {
            Ok(val) => val,
            Err(_) => {
                // Malformed file: overwrite with fresh
                return Self::create_fresh_config(config_path, token, shards);
            }
        };

        // Keep only recognized sections at top-level.
        let table = parsed
            .as_table_mut()
            .ok_or("Root of config is not a TOML table")?;
        let known_sections = ["red"];
        table.retain(|k, _| known_sections.contains(&&k[..]));

        // Make sure "red" section exists
        let red_section = table
            .entry("red")
            .or_insert_with(|| Value::Table(toml::map::Map::new()));
        // Clean "red" section by removing unknown fields
        if let Some(red_table) = red_section.as_table_mut() {
            let known_fields = ["token", "shards"];
            red_table.retain(|k, _| known_fields.contains(&&k[..]));

            // Insert missing fields
            if !red_table.contains_key("token") {
                red_table.insert("token".into(), Value::String(token));
            }
            if !red_table.contains_key("shards") {
                red_table.insert("shards".into(), Value::Integer(shards as i64));
            }
        }

        // Write the updated TOML back to file
        fs::write(config_path, toml::to_string_pretty(&parsed)?)?;
        // Convert it to our Config struct
        let config: Config = toml::from_str(&toml::to_string(&parsed)?)?;
        Ok(config)
    }

    /// Helper to create a fresh config file and return as struct.
    fn create_fresh_config(
        config_path: &Path,
        token: String,
        shards: u64,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let config = Self::new(token, shards);
        fs::write(config_path, toml::to_string_pretty(&config)?)?;
        Ok(config)
    }
}
