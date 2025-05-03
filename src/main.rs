// n34 - A CLI to interact with NIP-34 and other stuff related to codes in nostr
// Copyright (C) 2025 Awiteb <a@4rs.nl>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://gnu.org/licenses/gpl-3.0.html>.

/// Command line interface module
mod cli;
/// N34 errors
mod error;
/// Nostr utils module
mod nostr_utils;

use std::process::ExitCode;

use clap::Parser;
use clap_verbosity_flag::Verbosity;

use self::cli::Cli;

/// Configures the logging level based on the provided verbosity.
///
/// When verbosity is set to TRACE, includes file and line numbers in logs.
fn set_log_level(verbosity: Verbosity) {
    let is_trace = verbosity
        .tracing_level()
        .is_some_and(|l| l == tracing::Level::TRACE);

    let subscriber = tracing_subscriber::fmt()
        .with_file(is_trace)
        .with_line_number(is_trace)
        .without_time()
        .with_max_level(verbosity)
        .finish();
    tracing::subscriber::set_global_default(subscriber).ok();
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    set_log_level(cli.verbosity);

    if let Err(err) = cli.run().await {
        tracing::error!("{err}");
        return err.exit_code();
    }

    ExitCode::SUCCESS
}
