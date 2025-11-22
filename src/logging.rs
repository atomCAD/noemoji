// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at <https://mozilla.org/MPL/2.0/>.

//! Logging infrastructure for noemoji CLI

use std::io::Write;

/// Initialize the logger with environment-based configuration
///
/// Uses env_logger configured via the RUST_LOG environment variable.
/// If RUST_LOG is not set, defaults to "info" level.
///
/// This function is idempotent. The first call initializes the global logger;
/// subsequent calls return `Err(SetLoggerError)` which is typically ignored
/// since the logger is already configured.
///
/// # Arguments
///
/// * `program_name` - The program name to display in log messages, typically
///   obtained from `crate::program_name(&args[0])`
///
/// # Returns
///
/// Returns `Ok(())` on successful initialization or `Err(SetLoggerError)`
/// if initialization fails (most commonly because the logger was already set).
///
/// # Examples
///
/// ```
/// use noemoji::{cli::program_name, logging};
///
/// let args: Vec<String> = std::env::args().collect();
/// let program = program_name(&args[0]);
///
/// // Initialize logger and handle result
/// match logging::init_logger(program) {
///     Ok(()) => println!("Logger initialized"),
///     Err(e) => println!("Logger already initialized: {}", e),
/// }
///
/// // Or ignore the result if idempotency is expected
/// let _ = logging::init_logger(program);
/// ```
///
/// Use the `RUST_LOG` environment variable to control log levels:
///
/// ```bash
/// RUST_LOG=debug noemoji file.rs
/// ```
pub fn init_logger(program_name: &str) -> Result<(), log::SetLoggerError> {
    let program_name = program_name.to_string();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
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

// EOF
