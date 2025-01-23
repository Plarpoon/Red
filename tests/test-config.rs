#[cfg(test)]
mod tests {
    use red::bot::utils::config::Config;
    use std::fs;
    use toml::Value;

    #[test]
    fn test_fresh_config_creation() {
        let test_path = "config.toml";
        let _ = fs::remove_file(test_path);

        let cfg = Config::load_or_create_and_validate().unwrap();
        assert_eq!(cfg.red.token, "placeholder_token");
        assert_eq!(cfg.red.shards, 1);

        let contents = fs::read_to_string(test_path).unwrap();
        assert!(contents.contains("placeholder_token"));

        let _ = fs::remove_file(test_path);
    }

    #[test]
    fn test_extra_top_level_removal() {
        let test_path = "config.toml";
        let _ = fs::remove_file(test_path);

        // Create valid config
        let _ = Config::load_or_create_and_validate();

        // Add an extra top-level section
        let mut value: Value = toml::from_str(&fs::read_to_string(test_path).unwrap()).unwrap();
        value.as_table_mut().unwrap().insert(
            "extra_section".into(),
            Value::String("should_remove".into()),
        );
        fs::write(test_path, toml::to_string_pretty(&value).unwrap()).unwrap();

        // Reload config - this should remove unknown top-level sections
        let updated_cfg = Config::load_or_create_and_validate().unwrap();
        assert_eq!(updated_cfg.red.token, "placeholder_token");
        assert_eq!(updated_cfg.red.shards, 1);

        let updated_str = fs::read_to_string(test_path).unwrap();
        assert!(!updated_str.contains("extra_section"));

        let _ = fs::remove_file(test_path);
    }

    #[test]
    fn test_missing_field_addition() {
        let test_path = "config.toml";
        let _ = fs::remove_file(test_path);

        // Create valid config
        let _ = Config::load_or_create_and_validate();

        // Remove a required field
        let mut value: Value = toml::from_str(&fs::read_to_string(test_path).unwrap()).unwrap();
        value.as_table_mut().unwrap()["red"]
            .as_table_mut()
            .unwrap()
            .remove("token");
        fs::write(test_path, toml::to_string_pretty(&value).unwrap()).unwrap();

        // Reload config - this should add the missing field with default value
        let updated_cfg = Config::load_or_create_and_validate().unwrap();
        assert_eq!(updated_cfg.red.token, "placeholder_token");

        let updated_str = fs::read_to_string(test_path).unwrap();
        assert!(updated_str.contains("placeholder_token"));

        let _ = fs::remove_file(test_path);
    }

    #[test]
    fn test_invalid_field_type_correction() {
        let test_path = "config.toml";
        let _ = fs::remove_file(test_path);

        // Create valid config
        let _ = Config::load_or_create_and_validate();

        // Change field type to invalid type
        let mut value: Value = toml::from_str(&fs::read_to_string(test_path).unwrap()).unwrap();
        value.as_table_mut().unwrap()["red"].as_table_mut().unwrap()["shards"] =
            Value::String("invalid_type".into());
        fs::write(test_path, toml::to_string_pretty(&value).unwrap()).unwrap();

        // Reload config - this should correct the invalid type
        let updated_cfg = Config::load_or_create_and_validate().unwrap();
        assert_eq!(updated_cfg.red.shards, 1);

        let updated_str = fs::read_to_string(test_path).unwrap();
        assert!(updated_str.contains("shards = 1"));

        let _ = fs::remove_file(test_path);
    }
}
