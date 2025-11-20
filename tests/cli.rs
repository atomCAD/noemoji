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
fn with_args_exits_success() {
    let mut cmd = Command::new(cargo::cargo_bin!("noemoji"));
    // Use Cargo.toml as a test file since we know it exists
    cmd.arg("Cargo.toml")
        .assert()
        .success()
        .stdout(predicates::str::contains("USAGE:").not());
}

// EOF
