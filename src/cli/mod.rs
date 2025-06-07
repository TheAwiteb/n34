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

/// Commands module
pub mod commands;
/// The CLI config
pub mod config;
/// Default lazy values for CLI arguments
pub mod defaults;
/// CLI arguments parsers
pub mod parsers;
/// CLI traits
pub mod traits;
/// Common helper types used throughout the CLI.
pub mod types;

use clap::Parser;
use clap_verbosity_flag::Verbosity;

pub use self::commands::*;
pub use self::config::*;
use self::traits::CommandRunner;
use crate::error::N34Result;

/// Header message, used in the help message
const HEADER: &str = r#"Copyright (C) 2025 Awiteb <a@4rs.nl>
License GNU GPL-3.0-or-later <https://gnu.org/licenses/gpl-3.0.html>
This is free software: you are free to change and redistribute it.
There is NO WARRANTY, to the extent permitted by law.

Git repository: https://git.4rs.nl/awiteb/n34.git"#;

/// Footer message, used in the help message
const FOOTER: &str = r#"Please report bugs to <naddr1qqpkuve5qgsqqqqqq9g9uljgjfcyd6dm4fegk8em2yfz0c3qp3tc6mntkrrhawgrqsqqqauesksc39>."#;

/// Name of the file storing the repository address
pub const NOSTR_ADDRESS_FILE: &str = "nostr-address";

#[derive(Parser, Debug)]
#[command(about, version, before_long_help = HEADER, after_long_help = FOOTER)]
/// A command-line interface for interacting with NIP-34 and other Nostr
/// code-related stuff.
pub struct Cli {
    #[command(flatten)]
    pub options:   commands::CliOptions,
    /// Controls the verbosity level of output
    #[command(flatten)]
    pub verbosity: Verbosity,
    /// The subcommand to execute
    #[command(subcommand)]
    pub command:   commands::Commands,
}


impl Cli {
    /// Executes the command
    pub async fn run(self) -> N34Result<()> {
        self.command.run(self.options).await
    }
}

/// Processes the CLI configuration and returns it if successful.
pub fn post_cli(cli: Cli) -> N34Result<Cli> {
    // TODO
    Ok(cli)
}
