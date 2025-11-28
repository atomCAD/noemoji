// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at <https://mozilla.org/MPL/2.0/>.

//! Command-line interface parsing and help/version display

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::{ExitCode, Termination},
};

use thiserror::Error;

use crate::check::InputSource;

/// Error type for command line argument parsing
#[derive(Debug, Error)]
pub enum CliError {
    /// Unknown option was provided
    #[error("unknown option '{0}'")]
    UnknownOption(String),

    /// Option requires a value but none was provided
    #[error("option '{0}' requires a value")]
    MissingOptionValue(String),

    /// Unexpected positional argument found
    #[error("unexpected argument '{}'", .0.to_string_lossy())]
    UnexpectedArgument(OsString),

    /// Option had a value when none was expected
    #[error("option '{option}' doesn't take a value")]
    UnexpectedValue {
        /// The option that received an unexpected value
        option: String,
        /// The unexpected value that was provided
        value: OsString,
    },

    /// Invalid UTF-8 in argument value
    #[error("invalid UTF-8 in argument: {}", .0.to_string_lossy())]
    InvalidUtf8Value(OsString),

    /// No files were specified but files are required
    #[error("no files specified")]
    NoFilesSpecified,

    /// Internal error that should not occur in normal usage
    #[error("internal error (please report this bug): {0}")]
    InternalError(lexopt::Error),
}

impl From<lexopt::Error> for CliError {
    fn from(err: lexopt::Error) -> Self {
        match err {
            lexopt::Error::UnexpectedOption(option) => CliError::UnknownOption(option),
            lexopt::Error::MissingValue {
                option: Some(option),
                ..
            } => CliError::MissingOptionValue(option),
            lexopt::Error::UnexpectedArgument(arg) => CliError::UnexpectedArgument(arg),
            lexopt::Error::UnexpectedValue { option, value } => {
                CliError::UnexpectedValue { option, value }
            }
            lexopt::Error::NonUnicodeValue(value) => CliError::InvalidUtf8Value(value),
            // These should never occur in our usage:
            // - MissingValue with None: indicates unknown option (internal error)
            // - ParsingFailed: only from ValueExt methods (we don't use those)
            // - Custom: only if we create custom errors (we don't)
            lexopt::Error::MissingValue { option: None, .. }
            | lexopt::Error::ParsingFailed { .. }
            | lexopt::Error::Custom(_) => CliError::InternalError(err),
        }
    }
}

/// CLI command structure
#[derive(Debug, PartialEq, Clone)]
pub enum CliCommand {
    /// Show help information
    Help,
    /// Show version information
    Version,
    /// Process inputs for Unicode compliance checking
    Check {
        /// Input sources to check, in order of processing
        inputs: Vec<InputSource>,
    },
}

/// Parse command line arguments using lexopt
pub fn parse_args(args: &[String]) -> Result<CliCommand, CliError> {
    use lexopt::prelude::*;

    let mut parser = lexopt::Parser::from_args(args.iter().map(|s| s.as_str()));
    let mut inputs = Vec::with_capacity(args.len());

    loop {
        let arg = match parser.next() {
            Ok(Some(arg)) => arg,
            Ok(None) => break,
            Err(err) => return Err(err.into()),
        };
        match arg {
            Short('h') | Long("help") => return Ok(CliCommand::Help),
            Short('V') | Long("version") => return Ok(CliCommand::Version),
            Value(val) => {
                inputs.push(InputSource::File(PathBuf::from(val)));
            }
            _ => return Err(arg.unexpected().into()),
        }
    }

    if inputs.is_empty() {
        return Err(CliError::NoFilesSpecified);
    }

    Ok(CliCommand::Check { inputs })
}

/// Print version information
pub fn print_version() {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}

/// Extract program name from `argv[0]`, falling back to package name
///
/// # Examples
///
/// ```
/// use noemoji::cli::program_name;
///
/// assert_eq!(program_name("/usr/bin/noemoji"), "noemoji");
/// assert_eq!(program_name("./target/debug/noemoji"), "noemoji");
/// assert_eq!(program_name("noemoji"), "noemoji");
/// ```
pub fn program_name(arg0: &str) -> &str {
    Path::new(arg0)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(env!("CARGO_PKG_NAME"))
}

/// Print help information for the program
pub fn print_help(args0: &str) {
    let program = program_name(args0);
    println!(
        "Check files for problematic Unicode characters that should use ASCII equivalents

USAGE:
    {program} [OPTIONS] <FILE>...

ARGS:
    <FILE>...    One or more files to check for Unicode compliance

OPTIONS:
    -h, --help       Show this help message and exit
    -V, --version    Show version information and exit

EXAMPLES:
    {program} README.md
    {program} src/*.rs
    {program} docs/*.md **/*.rs

EXIT CODES:
    0    All files are compliant (success)
    1    One or more files contain violations (violations)
    2    Error reading or processing files (errors)"
    );
}

/// Outcome of running the linter, following Unix exit code conventions
///
/// The exit codes follow standard Unix conventions:
/// - 0: Success (no issues found)
/// - 1: Violations found (lint failures)
/// - 2: Error (runtime/usage errors)
///
/// # Examples
///
/// ```
/// use noemoji::cli::Outcome;
///
/// fn main() -> Outcome {
///     // ... do work ...
///     Outcome::Success
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// All files are compliant, no violations found
    Success,
    /// One or more files contain violations
    Violations,
    /// Error reading or processing files
    Error,
}

impl Termination for Outcome {
    fn report(self) -> ExitCode {
        match self {
            Outcome::Success => ExitCode::from(0),
            Outcome::Violations => ExitCode::from(1),
            Outcome::Error => ExitCode::from(2),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    #[test]
    fn outcome_success_returns_zero() {
        let code = Outcome::Success.report();
        assert_eq!(code, ExitCode::from(0));
    }

    #[test]
    fn outcome_violations_returns_one() {
        let code = Outcome::Violations.report();
        assert_eq!(code, ExitCode::from(1));
    }

    #[test]
    fn outcome_error_returns_two() {
        let code = Outcome::Error.report();
        assert_eq!(code, ExitCode::from(2));
    }

    #[test]
    fn from_lexopt_unexpected_option() {
        let lexopt_err = lexopt::Error::UnexpectedOption("--bad".to_owned());
        let cli_err: CliError = lexopt_err.into();
        assert!(matches!(cli_err, CliError::UnknownOption(_)));
    }

    #[test]
    fn from_lexopt_missing_value() {
        let lexopt_err = lexopt::Error::MissingValue {
            option: Some("--config".to_owned()),
        };
        let cli_err: CliError = lexopt_err.into();
        assert!(matches!(cli_err, CliError::MissingOptionValue(_)));
    }

    #[test]
    fn from_lexopt_unexpected_argument() {
        let lexopt_err = lexopt::Error::UnexpectedArgument(OsString::from("arg"));
        let cli_err: CliError = lexopt_err.into();
        assert!(matches!(cli_err, CliError::UnexpectedArgument(_)));
    }

    #[test]
    fn from_lexopt_unexpected_value() {
        let lexopt_err = lexopt::Error::UnexpectedValue {
            option: "--help".to_owned(),
            value: OsString::from("val"),
        };
        let cli_err: CliError = lexopt_err.into();
        assert!(matches!(cli_err, CliError::UnexpectedValue { .. }));
    }

    #[test]
    #[cfg(unix)]
    fn from_lexopt_non_unicode_value_unix() {
        use std::os::unix::ffi::OsStringExt;
        let invalid_bytes = vec![0xFF, 0xFE];
        let os_str = OsString::from_vec(invalid_bytes);
        let lexopt_err = lexopt::Error::NonUnicodeValue(os_str);
        let cli_err: CliError = lexopt_err.into();
        assert!(matches!(cli_err, CliError::InvalidUtf8Value(_)));
    }

    #[test]
    #[cfg(not(unix))]
    fn from_lexopt_non_unicode_value_fallback() {
        let lexopt_err = lexopt::Error::NonUnicodeValue(OsString::from("test"));
        let cli_err: CliError = lexopt_err.into();
        assert!(matches!(cli_err, CliError::InvalidUtf8Value(_)));
    }

    #[test]
    fn from_lexopt_custom_becomes_internal_error() {
        let lexopt_err = lexopt::Error::Custom("custom error".into());
        let cli_err: CliError = lexopt_err.into();
        assert!(matches!(cli_err, CliError::InternalError(_)));
    }

    #[test]
    fn from_lexopt_missing_value_none_becomes_internal_error() {
        let lexopt_err = lexopt::Error::MissingValue { option: None };
        let cli_err: CliError = lexopt_err.into();
        assert!(matches!(cli_err, CliError::InternalError(_)));
    }
}

// EOF
