// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at <https://mozilla.org/MPL/2.0/>.

//! Configuration management for noemoji
//!
//! This module provides configuration structures and handling for the noemoji application.

use crate::logging::LogLevel;
use serde::Deserialize;
use thiserror::Error;

/// Configuration parsing and validation errors
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Invalid TOML syntax or structure
    #[error("Invalid TOML configuration: {0}")]
    InvalidToml(#[from] toml::de::Error),
}

/// Logger configuration for noemoji.
///
/// Corresponds to the `[log]` section in .noemoji.toml:
/// ```toml
/// [log]
/// level = "debug"
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, Deserialize)]
pub struct LogConfig {
    /// Log level setting (None = use default)
    #[serde(default)]
    pub level: Option<LogLevel>,
}

/// Configuration settings for noemoji
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, Deserialize)]
pub struct Config {
    /// Log configuration section
    #[serde(default)]
    pub log: LogConfig,
}

/// Parse a TOML configuration string into a Config struct
///
/// # Arguments
///
/// * `toml_str` - TOML configuration string to parse
///
/// # Returns
///
/// Returns `Ok(Config)` on successful parse, or `ConfigError` on failure.
///
/// # Example
///
/// ```rust
/// # use noemoji::config::{parse_config, Config};
/// # use noemoji::logging::LogLevel;
/// let toml_str = r#"
///     [log]
///     level = "debug"
/// "#;
///
/// let config = parse_config(toml_str).unwrap();
/// assert_eq!(config.log.level, Some(LogLevel::Debug));
/// ```
pub fn parse_config(toml_str: &str) -> Result<Config, ConfigError> {
    toml::from_str::<Config>(toml_str).map_err(ConfigError::InvalidToml)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_config_default_level_is_none() {
        // While Option::default() returning None is standard Rust behavior,
        // we test it here because None has semantic meaning in our config:
        // "inherit from parent config, or use default if not set anywhere."
        let log = LogConfig::default();
        assert_eq!(log.level, None);
    }

    #[test]
    fn config_default_has_default_log() {
        // See note about defaults in log_config_default_level_is_none test.
        let config = Config::default();
        assert_eq!(config.log, LogConfig::default());
    }
}

// EOF
