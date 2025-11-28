// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at <https://mozilla.org/MPL/2.0/>.

use assert_cmd::{Command, cargo};
use predicates::prelude::*;

#[test]
fn no_args_reads_from_stdin() {
    let mut cmd = Command::new(cargo::cargo_bin!("noemoji"));
    cmd.write_stdin("Hello world!")
        .assert()
        .success()
        .stdout(predicates::str::contains("USAGE:").not());
}

#[test]
fn explicit_dash_reads_from_stdin() {
    let mut cmd = Command::new(cargo::cargo_bin!("noemoji"));
    cmd.arg("-")
        .write_stdin("Hello world!")
        .assert()
        .success()
        .stdout(predicates::str::contains("USAGE:").not());
}

#[test]
fn mixing_files_and_stdin() {
    let mut cmd = Command::new(cargo::cargo_bin!("noemoji"));
    cmd.arg("Cargo.toml")
        .arg("-")
        .arg("LICENSE")
        .write_stdin("Hello world from stdin!")
        .assert()
        .success();
}

#[test]
fn stdin_position_in_args_is_respected() {
    // Test that `-` can appear at any position and stdin is processed at that position
    let mut cmd = Command::new(cargo::cargo_bin!("noemoji"));
    cmd.arg("Cargo.toml") // clean file
        .arg("-") // stdin with violation
        .arg("LICENSE") // clean file
        .write_stdin("Has arrow → here")
        .assert()
        .code(1); // Violations found
}

#[test]
fn error_on_first_input_continues_to_second() {
    // Test that an error on one input doesn't stop processing subsequent inputs
    // First file errors, second file also errors - we should see BOTH error messages
    let mut cmd = Command::new(cargo::cargo_bin!("noemoji"));
    cmd.arg("nonexistent_file_AAA.txt") // error (file not found)
        .arg("nonexistent_file_BBB.txt") // error (also not found, should still be processed)
        .assert()
        .failure()
        .code(2)
        .stderr(predicates::str::contains("nonexistent_file_AAA.txt"))
        .stderr(predicates::str::contains("nonexistent_file_BBB.txt")); // Proves second was processed
}

#[test]
fn clean_first_input_then_error_second() {
    // Verify that processing continues from clean file to error file
    // If we see the error message, the second file was processed
    let mut cmd = Command::new(cargo::cargo_bin!("noemoji"));
    cmd.arg("Cargo.toml") // clean file (processed first, no output)
        .arg("nonexistent_file_12345.txt") // error (processed second)
        .assert()
        .failure()
        .code(2)
        .stderr(predicates::str::contains("nonexistent_file_12345.txt")); // Proves second was processed
}

#[test]
fn stdin_with_violations() {
    let mut cmd = Command::new(cargo::cargo_bin!("noemoji"));
    cmd.write_stdin("Hello → world with Unicode arrow!")
        .assert()
        .code(1)
        .stdout(predicates::str::contains("stdin:1:7:"))
        .stdout(predicates::str::contains("→"));
}

#[test]
fn violation_output_shows_line_and_column() {
    let mut cmd = Command::new(cargo::cargo_bin!("noemoji"));
    cmd.write_stdin("line one\nline → two\nline three")
        .assert()
        .code(1)
        .stdout(predicates::str::contains("stdin:2:6:"))
        .stdout(predicates::str::contains("→"));
}

#[test]
fn multiple_violations_all_reported() {
    let mut cmd = Command::new(cargo::cargo_bin!("noemoji"));
    cmd.write_stdin("→ start\nmiddle ←\nend ↑")
        .assert()
        .code(1)
        .stdout(predicates::str::contains("stdin:1:1:").and(predicates::str::contains("→")))
        .stdout(predicates::str::contains("stdin:2:8:").and(predicates::str::contains("←")))
        .stdout(predicates::str::contains("stdin:3:5:").and(predicates::str::contains("↑")));
}

#[test]
fn stdin_with_empty_input() {
    let mut cmd = Command::new(cargo::cargo_bin!("noemoji"));
    cmd.write_stdin("").assert().success(); // Empty input should still succeed
}

#[test]
fn stdin_with_invalid_utf8() {
    let mut cmd = Command::new(cargo::cargo_bin!("noemoji"));
    // Write invalid UTF-8 bytes to stdin
    let invalid_utf8: &[u8] = &[0xFF, 0xFE];
    cmd.write_stdin(invalid_utf8).assert().failure().code(2); // Should return 2 for errors
}

#[test]
fn stdin_integration_with_pipe() {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new(cargo::cargo_bin!("noemoji"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn noemoji");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"test content\n")
        .unwrap();
    let output = child.wait_with_output().expect("Failed to wait on child");

    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn stdin_integration_with_empty_pipe() {
    use std::process::{Command, Stdio};

    let mut child = Command::new(cargo::cargo_bin!("noemoji"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn noemoji");

    drop(child.stdin.take()); // Close stdin immediately (like cat /dev/null)
    let output = child.wait_with_output().expect("Failed to wait on child");

    assert_eq!(output.status.code(), Some(0));
}

// EOF
