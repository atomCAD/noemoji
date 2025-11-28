// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at <https://mozilla.org/MPL/2.0/>.

//! Input processing and Unicode compliance checking

use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
};

use thiserror::Error;

/// Prohibited Unicode characters that should use ASCII equivalents
const PROHIBITED_CHARS: &[char] = &[
    '→', // Use -> instead
    '←', // Use <- instead
    '↑', // Use ^ instead
    '↓', // Use v instead
];

/// Errors that can occur during input processing
#[derive(Debug, Error)]
pub enum CheckError {
    /// Failed to open file
    #[error("{}: {source}", path.display())]
    OpenFile {
        /// Path to the file that could not be opened
        path: PathBuf,
        /// The underlying I/O error
        #[source]
        source: io::Error,
    },

    /// Failed to read line
    #[error("{source}")]
    ReadLine {
        /// The underlying I/O error
        #[source]
        source: io::Error,
    },
}

/// Represents an input source for processing
#[derive(Debug, PartialEq, Clone)]
pub enum InputSource {
    /// Read from a file
    File(PathBuf),
}

impl InputSource {
    /// Returns the path for this input source
    pub fn path(&self) -> &Path {
        match self {
            InputSource::File(path) => path,
        }
    }

    /// Check this input source for Unicode compliance, streaming output.
    ///
    /// Calls `on_violation` for each prohibited character found.
    /// Returns `Ok(true)` if violations were found, `Ok(false)` if clean.
    pub fn check<F>(&self, on_violation: F) -> Result<bool, CheckError>
    where
        F: FnMut(usize, usize, char),
    {
        match self {
            InputSource::File(path) => {
                let file = File::open(path).map_err(|source| CheckError::OpenFile {
                    path: path.clone(),
                    source,
                })?;
                check_reader(BufReader::new(file), on_violation)
            }
        }
    }
}

/// Check a buffered reader for prohibited characters, streaming results.
fn check_reader<R, F>(reader: R, mut on_violation: F) -> Result<bool, CheckError>
where
    R: BufRead,
    F: FnMut(usize, usize, char),
{
    let mut found_violations = false;

    for (line_idx, line_result) in reader.lines().enumerate() {
        let line = line_result.map_err(|source| CheckError::ReadLine { source })?;

        for (col_idx, ch) in line.chars().enumerate() {
            if PROHIBITED_CHARS.contains(&ch) {
                found_violations = true;
                on_violation(line_idx + 1, col_idx + 1, ch);
            }
        }
    }

    Ok(found_violations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn check_reader_finds_single_violation() {
        let input = Cursor::new("text → more");
        let mut violations = Vec::new();

        let result = check_reader(input, |line, col, ch| {
            violations.push((line, col, ch));
        });

        assert!(result.unwrap());
        assert_eq!(violations, vec![(1, 6, '→')]);
    }

    #[test]
    fn check_reader_finds_multiple_violations_same_line() {
        let input = Cursor::new("a → b ← c");
        let mut violations = Vec::new();

        let result = check_reader(input, |line, col, ch| {
            violations.push((line, col, ch));
        });

        assert!(result.unwrap());
        assert_eq!(violations, vec![(1, 3, '→'), (1, 7, '←')]);
    }

    #[test]
    fn check_reader_finds_violations_across_lines() {
        let input = Cursor::new("line one →\nline two ←\nline three");
        let mut violations = Vec::new();

        let result = check_reader(input, |line, col, ch| {
            violations.push((line, col, ch));
        });

        assert!(result.unwrap());
        assert_eq!(violations, vec![(1, 10, '→'), (2, 10, '←')]);
    }

    #[test]
    fn check_reader_clean_input() {
        let input = Cursor::new("clean text with no violations");
        let mut violations = Vec::new();

        let result = check_reader(input, |line, col, ch| {
            violations.push((line, col, ch));
        });

        assert!(!result.unwrap());
        assert!(violations.is_empty());
    }

    #[test]
    fn check_reader_empty_input() {
        let input = Cursor::new("");
        let mut violations = Vec::new();

        let result = check_reader(input, |line, col, ch| {
            violations.push((line, col, ch));
        });

        assert!(!result.unwrap());
        assert!(violations.is_empty());
    }

    #[test]
    fn check_reader_all_prohibited_chars() {
        let input = Cursor::new("→←↑↓");
        let mut violations = Vec::new();

        let result = check_reader(input, |line, col, ch| {
            violations.push((line, col, ch));
        });

        assert!(result.unwrap());
        assert_eq!(
            violations,
            vec![(1, 1, '→'), (1, 2, '←'), (1, 3, '↑'), (1, 4, '↓')]
        );
    }

    #[test]
    fn check_reader_violation_at_line_start() {
        let input = Cursor::new("→ starts with arrow");
        let mut violations = Vec::new();

        let result = check_reader(input, |line, col, ch| {
            violations.push((line, col, ch));
        });

        assert!(result.unwrap());
        assert_eq!(violations, vec![(1, 1, '→')]);
    }

    #[test]
    fn check_reader_violation_at_line_end() {
        let input = Cursor::new("ends with arrow →");
        let mut violations = Vec::new();

        let result = check_reader(input, |line, col, ch| {
            violations.push((line, col, ch));
        });

        assert!(result.unwrap());
        assert_eq!(violations, vec![(1, 17, '→')]);
    }
}

// EOF
