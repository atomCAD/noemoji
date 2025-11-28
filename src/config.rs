// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at <https://mozilla.org/MPL/2.0/>.

//! Configuration management for noemoji
//!
//! This module provides hierarchical configuration discovery and merging.
//! Configuration files (`.noemoji.toml`) are searched from the current directory
//! up through parent directories, with child configurations overriding parent values.
//! The search stops when a configuration file sets `inherit = false` or when the
//! filesystem root is reached.

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
///
/// Example `.noemoji.toml` file:
/// ```toml
/// # Stop searching parent directories for config files
/// inherit = false
///
/// [log]
/// level = "debug"  # One of: disabled, error, warn, info, debug, trace
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Copy, Deserialize)]
pub struct Config {
    /// Log configuration section
    #[serde(default)]
    pub log: LogConfig,
    /// When false, stops the config file search at this file
    #[serde(default = "default_inherit")]
    pub inherit: bool,
}

fn default_inherit() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Config {
            log: LogConfig::default(),
            inherit: true,
        }
    }
}

impl Config {
    /// Merge two configurations with field-level precedence
    ///
    /// For Option fields, `self` takes precedence if it's Some, otherwise `other`.
    ///
    /// # Arguments
    ///
    /// * `other` - Fallback configuration for missing values
    ///
    /// # Returns
    ///
    /// Merged configuration with `self`'s values taking precedence, falling back to `other`
    pub fn or(self, other: Self) -> Self {
        Config {
            log: LogConfig {
                level: self.log.level.or(other.log.level),
            },
            // inherit indicates whether search continued, so preserve it from fallback
            inherit: other.inherit,
        }
    }

    /// Load configuration from the current working directory
    ///
    /// Searches for .noemoji.toml files starting from the current directory and
    /// continuing up parent directories. Merges configurations from general to
    /// specific (parent to child), where child configurations override parent
    /// values. If any configuration sets inherit = false, stops scanning parent
    /// directories.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Config)` with merged configuration or default if none found,
    /// or `ConfigError` if any file cannot be read or parsed.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use noemoji::config::Config;
    /// # use noemoji::logging::LogLevel;
    /// // Load configuration from current directory and parents
    /// // Returns default config if no .noemoji.toml files are found
    /// let config = Config::load().expect("Failed to load configuration");
    ///
    /// // Use unwrap_or to apply application defaults for unset values
    /// let level = config.log.level.unwrap_or(LogLevel::Warn);
    /// println!("Log level: {:?}", level);
    /// ```
    ///
    /// # Merging Behavior
    ///
    /// For Option fields like `log.level`:
    /// - `None` in a child config inherits the parent's value
    /// - `Some(value)` in a child config overrides any parent value
    /// - If no configs are found, returns `Config::default()` with all fields as defaults
    pub fn load() -> Result<Config, ConfigError> {
        Self::load_from(env::current_dir()?)
    }

    /// Load configuration from a specific directory
    ///
    /// Searches for .noemoji.toml files starting from the given directory and
    /// continuing up parent directories. Merges configurations from general to
    /// specific (parent to child), where child configurations override parent
    /// values. If any configuration sets inherit = false, stops scanning parent
    /// directories.
    ///
    /// # Arguments
    ///
    /// * `start_dir` - The directory to start searching from
    ///
    /// # Returns
    ///
    /// Returns `Ok(Config)` with merged configuration or default if none found,
    /// or `ConfigError` if any file cannot be read or parsed.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use noemoji::config::Config;
    /// # use noemoji::logging::LogLevel;
    /// # use tempfile::TempDir;
    /// # use std::fs;
    /// // Create a temp directory with a config file
    /// let temp_dir = TempDir::new().unwrap();
    /// fs::write(
    ///     temp_dir.path().join(".noemoji.toml"),
    ///     "[log]\nlevel = \"debug\"\n"
    /// ).unwrap();
    ///
    /// // Load config from directory
    /// let config = Config::load_from(temp_dir.path()).unwrap();
    /// assert_eq!(config.log.level, Some(LogLevel::Debug));
    /// ```
    ///
    /// # Directory Structure Example
    ///
    /// Given this directory structure:
    /// ```text
    /// /home/user/.noemoji.toml         # [log] level = "warn"
    /// /home/user/project/.noemoji.toml # [log] level = "debug"
    /// ```
    ///
    /// Calling `Config::load_from("/home/user/project")` returns a config with
    /// `log.level = Some(Debug)` because child configs override parent values.
    pub fn load_from<P: AsRef<std::path::Path>>(start_dir: P) -> Result<Config, ConfigError> {
        let mut current_dir = Some(start_dir.as_ref().to_path_buf());
        let mut result = Config::default();

        while let Some(dir) = current_dir {
            let config_path = dir.join(".noemoji.toml");

            // Attempt to read the file directly, handling NotFound gracefully
            match fs::read_to_string(&config_path) {
                Ok(content) => {
                    let config = parse_config(&content)?;

                    // Merge: child configs override parent configs
                    // result.or(config) means result (child) takes precedence, config (parent) is fallback
                    result = result.or(config);

                    // If this config has inherit = false, stop scanning for parent configs
                    if !config.inherit {
                        break;
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    // File doesn't exist, continue to parent directory
                }
                Err(e) => {
                    // Other I/O error (permission denied, etc.)
                    return Err(ConfigError::IoError(e));
                }
            }

            current_dir = dir.parent().map(|p| p.to_path_buf());
        }

        Ok(result)
    }
}

/// Parse a TOML configuration string into a Config struct
fn parse_config(toml_str: &str) -> Result<Config, ConfigError> {
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

    #[test]
    fn config_default_inherit_is_true() {
        let config = Config::default();
        assert!(config.inherit);
    }

    #[test]
    fn parse_config_with_inherit_false() {
        let toml_str = r#"
inherit = false

[log]
level = "error"
"#;

        let config = parse_config(toml_str).unwrap();
        assert_eq!(config.log.level, Some(crate::logging::LogLevel::Error));
        assert!(!config.inherit);
    }
}

// EOF
