// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at <https://mozilla.org/MPL/2.0/>.

//! Tests for TOML parsing functionality via the public Config API

use std::{fs::File, io::Write};

use noemoji::{
    config::{Config, ConfigError},
    logging::LogLevel,
};
use tempfile::TempDir;

fn load_config_from_toml(toml_str: &str) -> Result<Config, ConfigError> {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join(".noemoji.toml");
    let mut file = File::create(&config_path).unwrap();
    write!(file, "{}", toml_str).unwrap();
    drop(file);
    Config::load_from(temp_dir.path())
}

#[test]
fn parse_config_valid_toml_succeeds() {
    let toml_str = r#"
        [log]
        level = "debug"
    "#;

    let config = load_config_from_toml(toml_str).unwrap();
    assert_eq!(config.log.level, Some(LogLevel::Debug));
}

#[test]
fn parse_config_empty_toml_uses_defaults() {
    let config = load_config_from_toml("").unwrap();
    assert_eq!(config.log.level, None);
    assert!(config.inherit);
}

#[test]
fn parse_config_partial_log_section() {
    let toml_str = r#"
        [log]
        # level intentionally omitted
    "#;

    let config = load_config_from_toml(toml_str).unwrap();
    assert_eq!(config.log.level, None);
}

#[test]
fn parse_config_all_log_levels() {
    let test_cases = [
        ("disabled", LogLevel::Disabled),
        ("error", LogLevel::Error),
        ("warn", LogLevel::Warn),
        ("info", LogLevel::Info),
        ("debug", LogLevel::Debug),
        ("trace", LogLevel::Trace),
    ];

    for (level_str, expected_level) in test_cases {
        let toml_str = format!(
            r#"
            [log]
            level = "{level_str}"
        "#
        );

        let config = load_config_from_toml(&toml_str).unwrap();
        assert_eq!(config.log.level, Some(expected_level));
    }
}

#[test]
fn parse_config_invalid_toml_syntax_returns_error() {
    let toml_str = r#"
        [log
        level = "debug"
    "#;

    let error = load_config_from_toml(toml_str).unwrap_err();
    assert!(matches!(error, ConfigError::InvalidToml(_)));
}

#[test]
fn parse_config_invalid_log_level_returns_error() {
    let toml_str = r#"
        [log]
        level = "invalid"
    "#;

    let error = load_config_from_toml(toml_str).unwrap_err();
    assert!(matches!(error, ConfigError::InvalidToml(_)));
    assert!(error.to_string().contains("invalid"));
}

#[test]
fn parse_config_wrong_type_for_level_returns_error() {
    let toml_str = r#"
        [log]
        level = 42
    "#;

    let error = load_config_from_toml(toml_str).unwrap_err();
    assert!(matches!(error, ConfigError::InvalidToml(_)));
}

#[test]
fn config_error_display_mentions_toml() {
    let toml_str = r#"
        [log
        level = "debug"
    "#;

    let error = load_config_from_toml(toml_str).unwrap_err();
    assert!(error.to_string().contains("TOML"));
}

// EOF
