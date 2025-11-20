// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at <https://mozilla.org/MPL/2.0/>.

//! Command-line interface parsing and help/version display

use std::path::Path;

/// Print help information for the program
pub fn print_help(args0: &str) {
    let program = Path::new(&args0)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(env!("CARGO_PKG_NAME"));
    println!(
        "Check files for problematic Unicode characters that should use ASCII equivalents

USAGE:
    {program} <FILE>...

ARGS:
    <FILE>...    One or more files to check for Unicode compliance

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

// EOF
