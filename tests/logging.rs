// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at <https://mozilla.org/MPL/2.0/>.

//! Tests for logging infrastructure

#[test]
fn test_init_logger_idempotent() {
    // Verify that init_logger can be called multiple times safely
    // This tests the documented guarantee: "This function is idempotent"
    use noemoji::logging::{LogLevel, init_logger};

    // First call should succeed
    let result1 = init_logger("noemoji", LogLevel::Disabled);
    assert!(result1.is_ok());

    // Subsequent calls should return error (already initialized) but not panic
    let result2 = init_logger("noemoji", LogLevel::Info);
    assert!(result2.is_err());

    let result3 = init_logger("noemoji", LogLevel::Debug);
    assert!(result3.is_err());
}

#[test]
fn test_logger_actually_works() {
    use assert_cmd::Command;
    use predicates::prelude::*;

    // Run with RUST_LOG=debug and verify we get debug output
    Command::new(assert_cmd::cargo::cargo_bin!("noemoji"))
        .arg("--help")
        .env("RUST_LOG", "debug")
        .assert()
        .success()
        .stderr(predicate::str::contains("[debug]"));
}

// EOF
