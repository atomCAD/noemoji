// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at <https://mozilla.org/MPL/2.0/>.

//! Integration tests for logging infrastructure

#[test]
fn logger_actually_works() {
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

#[test]
fn log_level_debug_produces_output() {
    use assert_cmd::Command;
    use predicates::prelude::*;

    // With NOEMOJI_LOG=debug, we should see debug output
    Command::new(assert_cmd::cargo::cargo_bin!("noemoji"))
        .arg("--help")
        .env("NOEMOJI_LOG", "debug")
        .assert()
        .success()
        .stderr(predicate::str::contains("[debug]"));
}

#[test]
fn log_level_off_produces_no_output() {
    use assert_cmd::Command;
    use predicates::prelude::*;

    // With NOEMOJI_LOG=off, we should see no log output
    Command::new(assert_cmd::cargo::cargo_bin!("noemoji"))
        .arg("--help")
        .env("NOEMOJI_LOG", "off")
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn log_level_default_produces_no_output() {
    use assert_cmd::Command;
    use predicates::prelude::*;

    // Without NOEMOJI_LOG, default is off, no log output
    Command::new(assert_cmd::cargo::cargo_bin!("noemoji"))
        .arg("--help")
        .env_remove("NOEMOJI_LOG")
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

#[test]
fn noemoji_log_supports_module_filter_syntax() {
    use assert_cmd::Command;
    use predicates::prelude::*;

    // NOEMOJI_LOG supports env_logger's module filter syntax
    Command::new(assert_cmd::cargo::cargo_bin!("noemoji"))
        .arg("--help")
        .env("NOEMOJI_LOG", "warn,noemoji=debug")
        .assert()
        .success()
        .stderr(predicate::str::contains("[debug]"));
}

#[test]
fn rust_log_fallback_when_noemoji_log_unset() {
    use assert_cmd::Command;
    use predicates::prelude::*;

    // RUST_LOG is used when NOEMOJI_LOG is not set
    Command::new(assert_cmd::cargo::cargo_bin!("noemoji"))
        .arg("--help")
        .env_remove("NOEMOJI_LOG")
        .env("RUST_LOG", "debug")
        .assert()
        .success()
        .stderr(predicate::str::contains("[debug]"));
}

#[test]
fn noemoji_log_takes_precedence_over_rust_log() {
    use assert_cmd::Command;
    use predicates::prelude::*;

    // NOEMOJI_LOG takes precedence over RUST_LOG
    Command::new(assert_cmd::cargo::cargo_bin!("noemoji"))
        .arg("--help")
        .env("NOEMOJI_LOG", "off")
        .env("RUST_LOG", "debug")
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

// EOF
