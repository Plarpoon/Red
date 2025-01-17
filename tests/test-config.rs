#[cfg(test)]
mod tests {
    use std::fs;
    use toml::Value;

    #[test]
    fn test_fresh_config_creation() {
        let test_path = "config.toml";
        let _ = fs::remove_file(test_path);

        let cfg = red::config::Config::load_or_create("discord_token".to_string(), 2).unwrap();
        assert_eq!(cfg.red.token, "discord_token");
        assert_eq!(cfg.red.shards, 2);

        let contents = fs::read_to_string(test_path).unwrap();
        assert!(contents.contains("discord_token"));

        let _ = fs::remove_file(test_path);
    }

    #[test]
    fn test_extra_top_level_removal() {
        let test_path = "config.toml";
        let _ = fs::remove_file(test_path);

        // Create valid config
        let _ = red::config::Config::load_or_create("discord_token".into(), 2);

        // Add an extra top-level section
        let mut value: Value = toml::from_str(&fs::read_to_string(test_path).unwrap()).unwrap();
        value.as_table_mut().unwrap().insert(
            "extra_section".into(),
            Value::String("should_remove".into()),
        );
        fs::write(test_path, toml::to_string_pretty(&value).unwrap()).unwrap();

        // Reload config - this should remove unknown top-level sections
        let updated_cfg = red::config::Config::load_or_create("irrelevant".into(), 9).unwrap();
        assert_eq!(updated_cfg.red.token, "discord_token");
        assert_eq!(updated_cfg.red.shards, 2);

        let updated_str = fs::read_to_string(test_path).unwrap();
        assert!(!updated_str.contains("extra_section"));

        let _ = fs::remove_file(test_path);
    }
}
