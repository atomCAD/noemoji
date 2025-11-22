// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at <https://mozilla.org/MPL/2.0/>.

use assert_cmd::{cargo, prelude::*};
use predicates::prelude::*;
use std::process::Command;

#[test]
fn no_args_shows_usage() {
    let mut cmd = Command::new(cargo::cargo_bin!("noemoji"));
    cmd.assert()
        .failure()
        .code(2)
        .stdout(predicates::str::contains("USAGE:"))
        .stdout(predicates::str::contains("<FILE>..."));
}

#[test]
fn help_flag_shows_usage() {
    let mut cmd = Command::new(cargo::cargo_bin!("noemoji"));
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("USAGE:"))
        .stdout(predicates::str::contains("OPTIONS:"));
}

#[test]
fn help_short_flag_shows_usage() {
    let mut cmd = Command::new(cargo::cargo_bin!("noemoji"));
    cmd.arg("-h")
        .assert()
        .success()
        .stdout(predicates::str::contains("USAGE:"))
        .stdout(predicates::str::contains("OPTIONS:"));
}

#[test]
fn version_flag_shows_version() {
    let expected_version = env!("CARGO_PKG_VERSION");
    let mut cmd = Command::new(cargo::cargo_bin!("noemoji"));
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicates::str::contains("noemoji "))
        .stdout(predicates::str::contains(expected_version));
}

#[test]
fn version_short_flag_shows_version() {
    let expected_version = env!("CARGO_PKG_VERSION");
    let mut cmd = Command::new(cargo::cargo_bin!("noemoji"));
    cmd.arg("-V")
        .assert()
        .success()
        .stdout(predicates::str::contains("noemoji "))
        .stdout(predicates::str::contains(expected_version));
}

#[test]
fn with_args_exits_success() {
    let mut cmd = Command::new(cargo::cargo_bin!("noemoji"));
    // Use Cargo.toml as a test file since we know it exists
    cmd.arg("Cargo.toml")
        .assert()
        .success()
        .stdout(predicates::str::contains("USAGE:").not());
}

#[test]
fn multiple_file_arguments_succeed() {
    let mut cmd = Command::new(cargo::cargo_bin!("noemoji"));
    cmd.arg("Cargo.toml").arg("LICENSE").assert().success();
}

#[test]
fn invalid_flag_shows_error() {
    let mut cmd = Command::new(cargo::cargo_bin!("noemoji"));
    cmd.arg("--invalid-flag")
        .assert()
        .failure()
        .code(2)
        .stderr(predicates::str::contains("--invalid-flag"));
}

#[test]
fn invalid_short_flag_shows_error() {
    let mut cmd = Command::new(cargo::cargo_bin!("noemoji"));
    cmd.arg("-x")
        .assert()
        .failure()
        .code(2)
        .stderr(predicates::str::contains("-x"));
}

#[test]
fn error_message_suggests_help() {
    let mut cmd = Command::new(cargo::cargo_bin!("noemoji"));
    cmd.arg("--invalid-option")
        .assert()
        .failure()
        .code(2)
        .stderr(predicates::str::contains("-h"));
}

// EOF
