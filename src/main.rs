// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at <https://mozilla.org/MPL/2.0/>.

use std::env;

use noemoji::{
    cli::{CliCommand, Outcome, parse_args, print_help, print_version, program_name},
    config::Config,
    logging::init_logger,
};

fn main() -> Outcome {
    let args: Vec<String> = env::args().collect();
    let program = program_name(&args[0]);
    let config = Config::load().unwrap_or_default();
    match init_logger(program, config.log.level.unwrap_or_default()) {
        Ok(()) => log::debug!("logger initialized"),
        Err(_) => log::debug!("logger already initialized"),
    }

    match parse_args(&args[1..]) {
        Ok(CliCommand::Help) => {
            print_help(&args[0]);
            Outcome::Success
        }
        Ok(CliCommand::Version) => {
            print_version();
            Outcome::Success
        }
        Ok(CliCommand::Check { inputs }) => {
            let mut has_violations = false;
            let mut has_errors = false;

            for input in &inputs {
                let name = input.name();

                match input.check(|line, col, ch| {
                    println!("{}:{}:{}: prohibited character '{}'", name, line, col, ch);
                }) {
                    Ok(found) => {
                        if found {
                            has_violations = true;
                        }
                    }
                    Err(err) => {
                        eprintln!("{}: {}", program, err);
                        has_errors = true;
                    }
                }
            }

            if has_errors {
                Outcome::Error
            } else if has_violations {
                Outcome::Violations
            } else {
                Outcome::Success
            }
        }
        Err(err) => {
            eprintln!("{}: {}", program, err);
            eprintln!("Try '{} --help' for more information.", program);
            Outcome::Error
        }
    }
}

// EOF
