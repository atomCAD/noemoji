// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at <https://mozilla.org/MPL/2.0/>.

//! Configuration management for noemoji
//!
//! This module provides configuration structures and handling for the noemoji application.

use crate::logging::LogLevel;

/// Logger configuration for noemoji.
///
/// Corresponds to the `[log]` section in .noemoji.toml:
/// ```toml
/// [log]
/// level = "debug"
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LogConfig {
    /// Log level setting (None = use default)
    pub level: Option<LogLevel>,
}

/// Configuration settings for noemoji
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct Config {
    /// Log configuration section
    pub log: LogConfig,
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
