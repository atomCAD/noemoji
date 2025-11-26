// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at <https://mozilla.org/MPL/2.0/>.

//! Configuration management for noemoji
//!
//! This module provides configuration structures and handling for the noemoji application.

use std::{env, fs, io};

use crate::logging::LogLevel;
use serde::Deserialize;
use thiserror::Error;

/// Configuration parsing and validation errors
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Invalid TOML syntax or structure
    #[error("Invalid TOML configuration: {0}")]
    InvalidToml(#[from] toml::de::Error),
    /// File I/O error during configuration loading
    #[error("I/O error while reading configuration: {0}")]
    IoError(#[from] io::Error),
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

/// Load configuration by searching for .noemoji.toml files hierarchically
///
/// Searches for .noemoji.toml starting from the current working directory and
/// parent directories up to the filesystem root. Returns the first configuration
/// file found, or a default configuration if none is found.
///
/// # Returns
///
/// Returns `Ok(Config)` with either the loaded configuration or default values,
/// or `ConfigError` if a file is found but cannot be read or parsed.
///
/// # Example
///
/// ```rust,no_run
/// # use noemoji::config::load_config;
/// let config = load_config().unwrap();
/// ```
pub fn load_config() -> Result<Config, ConfigError> {
    load_config_from(env::current_dir()?)
}

/// Load configuration by searching for .noemoji.toml files starting from a specific directory
///
/// Searches for .noemoji.toml in the given directory and parent directories
/// up to the filesystem root. Returns the first configuration file found,
/// or a default configuration if none is found.
///
/// # Arguments
///
/// * `start_dir` - The directory to start searching from
///
/// # Returns
///
/// Returns `Ok(Config)` with either the loaded configuration or default values,
/// or `ConfigError` if a file is found but cannot be read or parsed.
///
/// # Example
///
/// ```rust,no_run
/// # use noemoji::config::load_config_from;
/// # use std::path::PathBuf;
/// let config = load_config_from(PathBuf::from("/my/project")).unwrap();
/// ```
pub fn load_config_from(start_dir: std::path::PathBuf) -> Result<Config, ConfigError> {
    let mut current_dir = start_dir;

    loop {
        let config_path = current_dir.join(".noemoji.toml");

        // Check if config file exists
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            return parse_config(&content);
        }

        // Move to parent directory
        match current_dir.parent() {
            Some(parent) => current_dir = parent.to_path_buf(),
            None => break, // Reached filesystem root
        }
    }

    // No configuration file found, return default configuration
    Ok(Config::default())
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
