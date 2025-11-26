// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at <https://mozilla.org/MPL/2.0/>.

//! Logging infrastructure for noemoji CLI

use std::{io::Write, str::FromStr};

use serde::{Deserialize, Deserializer};
use thiserror::Error;

/// Log verbosity level
///
/// Used as the baseline filter level when no environment variable overrides it.
/// For finer control (e.g., per-module filtering), use the `NOEMOJI_LOG` or
/// `RUST_LOG` environment variables which support the full env_logger syntax:
///
/// ```bash
/// # Simple level
/// NOEMOJI_LOG=debug noemoji file.rs
///
/// # Per-module filtering
/// NOEMOJI_LOG=warn,noemoji::parser=trace noemoji file.rs
/// ```
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// No logging output
    #[default]
    Disabled,
    /// Log errors only
    Error,
    /// Log warnings and above
    Warn,
    /// Log info and above
    Info,
    /// Log debug and above
    Debug,
    /// Log everything
    Trace,
}

impl LogLevel {
    /// Mapping table for FromStr implementation
    const FROM_STR_MAPPINGS: &[(&[&str], LogLevel)] = &[
        (&["off", "disabled", "none"], LogLevel::Disabled),
        (&["error"], LogLevel::Error),
        (&["warn", "warning"], LogLevel::Warn),
        (&["info"], LogLevel::Info),
        (&["debug"], LogLevel::Debug),
        (&["trace"], LogLevel::Trace),
    ];

    /// Convert to log::LevelFilter for use with logging infrastructure
    pub const fn to_level_filter(self) -> log::LevelFilter {
        match self {
            Self::Disabled => log::LevelFilter::Off,
            Self::Error => log::LevelFilter::Error,
            Self::Warn => log::LevelFilter::Warn,
            Self::Info => log::LevelFilter::Info,
            Self::Debug => log::LevelFilter::Debug,
            Self::Trace => log::LevelFilter::Trace,
        }
    }
}

/// Error returned when parsing an invalid log level string
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error(
    "invalid log level '{value}', expected: off/disabled/none, error, warn(ing), info, debug, or trace"
)]
pub struct ParseLogLevelError {
    /// The invalid value that was provided
    pub value: String,
}

impl std::str::FromStr for LogLevel {
    type Err = ParseLogLevelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for (aliases, level) in Self::FROM_STR_MAPPINGS {
            if aliases.iter().any(|a| s.eq_ignore_ascii_case(a)) {
                return Ok(*level);
            }
        }

        Err(ParseLogLevelError {
            value: s.to_owned(),
        })
    }
}

impl<'de> Deserialize<'de> for LogLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        LogLevel::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// Initialize the logger with optional config-based override
///
/// Logging priority (highest to lowest):
/// 1. `NOEMOJI_LOG` environment variable
/// 2. `RUST_LOG` environment variable
/// 3. `config_level` parameter
///
/// Both environment variables support the standard env_logger filter syntax
/// including module-level filtering:
///
/// ```bash
/// NOEMOJI_LOG=debug noemoji file.rs
/// RUST_LOG=warn,noemoji::parser=trace noemoji file.rs
/// ```
///
/// This function is idempotent. The first call initializes the global logger;
/// subsequent calls return `Err(SetLoggerError)` which is typically ignored
/// since the logger is already configured.
///
/// # Arguments
///
/// * `program_name` - The program name to display in log messages, typically
///   obtained from `crate::program_name(&args[0])`
/// * `config_level` - Log level from config file; ignored if `NOEMOJI_LOG`
///   or `RUST_LOG` is set
///
/// # Returns
///
/// Returns `Ok(())` on successful initialization or `Err(SetLoggerError)`
/// if initialization fails (most commonly because the logger was already set).
///
/// # Examples
///
/// ```
/// use noemoji::{cli::program_name, logging::{self, LogLevel}};
///
/// let args: Vec<String> = std::env::args().collect();
/// let program = program_name(&args[0]);
///
/// // Initialize logger and handle result
/// match logging::init_logger(program, LogLevel::Disabled) {
///     Ok(()) => log::debug!("logger initialized"),
///     Err(_) => log::debug!("logger already initialized"),
/// }
///
/// // Or ignore the result if idempotency is expected
/// let _ = logging::init_logger(program, LogLevel::Info);
/// ```
pub fn init_logger(program_name: &str, config_level: LogLevel) -> Result<(), log::SetLoggerError> {
    let program_name = program_name.to_owned();
    let default_level = config_level.to_level_filter();

    let env = if std::env::var("NOEMOJI_LOG").is_ok() {
        env_logger::Env::new().filter_or("NOEMOJI_LOG", default_level.as_str())
    } else {
        env_logger::Env::default().default_filter_or(default_level.as_str())
    };

    env_logger::Builder::from_env(env)
        .format(move |buf, record| {
            writeln!(
                buf,
                "{}[{}]: {}",
                program_name,
                record.level().to_string().to_lowercase(),
                record.args()
            )
        })
        .try_init()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_level_filter_converts_correctly() {
        assert_eq!(LogLevel::Disabled.to_level_filter(), log::LevelFilter::Off);
        assert_eq!(LogLevel::Error.to_level_filter(), log::LevelFilter::Error);
        assert_eq!(LogLevel::Warn.to_level_filter(), log::LevelFilter::Warn);
        assert_eq!(LogLevel::Info.to_level_filter(), log::LevelFilter::Info);
        assert_eq!(LogLevel::Debug.to_level_filter(), log::LevelFilter::Debug);
        assert_eq!(LogLevel::Trace.to_level_filter(), log::LevelFilter::Trace);
    }

    #[test]
    fn from_str_accepts_canonical_names() {
        assert_eq!("off".parse::<LogLevel>().unwrap(), LogLevel::Disabled);
        assert_eq!("error".parse::<LogLevel>().unwrap(), LogLevel::Error);
        assert_eq!("warn".parse::<LogLevel>().unwrap(), LogLevel::Warn);
        assert_eq!("info".parse::<LogLevel>().unwrap(), LogLevel::Info);
        assert_eq!("debug".parse::<LogLevel>().unwrap(), LogLevel::Debug);
        assert_eq!("trace".parse::<LogLevel>().unwrap(), LogLevel::Trace);
    }

    #[test]
    fn from_str_accepts_aliases() {
        // "disabled" and "none" are aliases for "off"
        assert_eq!("disabled".parse::<LogLevel>().unwrap(), LogLevel::Disabled);
        assert_eq!("none".parse::<LogLevel>().unwrap(), LogLevel::Disabled);
        // "warning" is an alias for "warn"
        assert_eq!("warning".parse::<LogLevel>().unwrap(), LogLevel::Warn);
    }

    #[test]
    fn from_str_is_case_insensitive() {
        assert_eq!("DEBUG".parse::<LogLevel>().unwrap(), LogLevel::Debug);
        assert_eq!("Debug".parse::<LogLevel>().unwrap(), LogLevel::Debug);
        assert_eq!("WARNING".parse::<LogLevel>().unwrap(), LogLevel::Warn);
    }

    #[test]
    fn from_str_rejects_invalid() {
        let invalid = "garbage";
        let err = invalid.parse::<LogLevel>().unwrap_err();
        assert_eq!(err.value, invalid);
        assert!(err.to_string().contains(invalid));
    }

    #[test]
    fn init_logger_is_idempotent() {
        // Verify that init_logger can be called multiple times safely (but may error)
        // This tests the documented guarantee: "This function is idempotent"
        let _ = init_logger("noemoji", LogLevel::Disabled);
        let _ = init_logger("noemoji", LogLevel::Debug);
        let _ = init_logger("noemoji", LogLevel::Info);
    }
}

// EOF
