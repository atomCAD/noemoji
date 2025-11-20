// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at <https://mozilla.org/MPL/2.0/>.

use std::{env, process::ExitCode};

use noemoji::cli::print_help;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        print_help(&args[0]);
        return ExitCode::from(2);
    }

    ExitCode::SUCCESS
}

// EOF
