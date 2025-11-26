// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at <https://mozilla.org/MPL/2.0/>.

use std::{env, process::ExitCode};

use noemoji::{
    cli::{CliCommand, CliError, parse_args, print_help, print_version, program_name},
    config::Config,
    logging::init_logger,
};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let program = program_name(&args[0]);
    let config = Config::default();
    match init_logger(program, config.log.level.unwrap_or_default()) {
        Ok(()) => log::debug!("logger initialized"),
        Err(_) => log::debug!("logger already initialized"),
    }

    match parse_args(&args[1..]) {
        Ok(CliCommand::Help) => {
            print_help(&args[0]);
            ExitCode::SUCCESS
        }
        Ok(CliCommand::Version) => {
            print_version();
            ExitCode::SUCCESS
        }
        Ok(CliCommand::Check { .. }) => {
            // TODO: Implement actual file checking logic
            ExitCode::SUCCESS
        }
        Err(err) => {
            match err {
                CliError::NoFilesSpecified => print_help(&args[0]),
                err => {
                    let program = program_name(&args[0]);
                    eprintln!("{}: {}", program, err);
                    eprintln!("Try '{} --help' for more information.", program);
                }
            }
            ExitCode::from(2)
        }
    }
}

// EOF
