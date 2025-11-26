// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at <https://mozilla.org/MPL/2.0/>.

use std::env;

use noemoji::{
    cli::{CliCommand, CliError, Outcome, parse_args, print_help, print_version, program_name},
    config::load_config,
    logging::init_logger,
};

fn main() -> Outcome {
    let args: Vec<String> = env::args().collect();
    let program = program_name(&args[0]);
    let config = load_config().unwrap_or_default();
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
        Ok(CliCommand::Check { .. }) => {
            // TODO: Implement actual file checking logic
            Outcome::Success
        }
        Err(CliError::NoFilesSpecified) => {
            print_help(&args[0]);
            Outcome::Error
        }
        Err(err) => {
            eprintln!("{}: {}", program, err);
            eprintln!("Try '{} --help' for more information.", program);
            Outcome::Error
        }
    }
}

// EOF
