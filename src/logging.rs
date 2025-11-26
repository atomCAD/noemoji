// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at <https://mozilla.org/MPL/2.0/>.

//! Logging infrastructure for noemoji CLI

use std::io::Write;

/// Log verbosity level
///
/// Used as the baseline filter level when no environment variable overrides it.
/// For finer control (e.g., per-module filtering), use the `RUST_LOG`
/// environment variables which support the full env_logger syntax:
///
/// ```bash
/// # Simple level
/// RUST_LOG=debug noemoji file.rs
///
/// # Per-module filtering
/// RUST_LOG=warn,noemoji::parser=trace noemoji file.rs
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

/// Initialize the logger with optional config-based override
///
/// Logging priority (highest to lowest):
/// 1. `RUST_LOG` environment variable
/// 2. `config_level` parameter
///
/// The `RUST_LOG` environment variables support the standard env_logger
/// filter syntax including module-level filtering:
///
/// ```bash
/// RUST_LOG=debug noemoji file.rs
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
/// * `config_level` - Log level from config file; ignored if `RUST_LOG` is set
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

    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(default_level.as_str()),
    )
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
}

// EOF
