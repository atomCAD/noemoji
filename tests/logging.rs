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

// EOF
