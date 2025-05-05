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

/// CLI arguments parsers
pub mod parsers;
/// `repo` subcommands
mod repo;
/// CLI traits
mod traits;

use std::fmt;

use clap::{ArgGroup, Args, Parser};
use clap_verbosity_flag::Verbosity;
use nostr::{RelayUrl, SecretKey};

pub use self::repo::RepoSubcommands;
pub use self::traits::CommandRunner;
use crate::error::N34Result;

/// Header message, used in the help message
const HEADER: &str = r#"Copyright (C) 2025 Awiteb <a@4rs.nl>
License GNU GPL-3.0-or-later <https://gnu.org/licenses/gpl-3.0.html>
This is free software: you are free to change and redistribute it.
There is NO WARRANTY, to the extent permitted by law.

Git repository: https://git.4rs.nl/awiteb/n34"#;

/// Footer message, used in the help message
const FOOTER: &str = r#"Please report bugs to <https://git.4rs.nl/awiteb/n34/issues>."#;

/// The command-line interface options
#[derive(Args, Clone)]
#[clap(
    group(
        ArgGroup::new("auth")
            .args(&["secret_key"])
            .required(true)
    )
)]
pub struct CliOptions {
    /// Your Nostr secret key
    #[arg(short, long)]
    pub secret_key: Option<SecretKey>,
    /// Where your relays list. And repository relays if not included in naddr
    #[arg(short, long, required = true)]
    pub relays:     Vec<RelayUrl>,
}

#[derive(Parser, Debug)]
#[command(about, version, before_long_help = HEADER, after_long_help = FOOTER)]
/// A command-line interface for interacting with NIP-34 and other Nostr
/// code-related stuff.
pub struct Cli {
    #[command(flatten)]
    pub options:   CliOptions,
    /// Controls the verbosity level of output
    #[command(flatten)]
    pub verbosity: Verbosity,
    /// The subcommand to execute
    #[command(subcommand)]
    pub command:   Commands,
}

/// N34 commands
#[derive(Parser, Debug)]
pub enum Commands {
    /// Manage repositories
    Repo {
        #[command(subcommand)]
        subcommands: RepoSubcommands,
    },
    // /// Manage issues
    // Issue {
    //     #[command(subcommand)]
    //     subcommands: IssueSubcommands,
    // },
    // /// Manage patches
    // Patch {
    //     #[command(subcommand)]
    //     subcommands: PatchSubcommands,
    // },
}

impl Cli {
    /// Executes the command
    pub async fn run(self) -> N34Result<()> {
        self.command.run(self.options).await
    }
}

impl CommandRunner for Commands {
    async fn run(self, options: CliOptions) -> N34Result<()> {
        tracing::trace!("Options: {options:#?}");
        tracing::trace!("Handling: {self:#?}");
        match self {
            Self::Repo { subcommands } => subcommands.run(options).await,
        }
    }
}

impl fmt::Debug for CliOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CliOptions")
            .field("secret_key", &self.secret_key.as_ref().map(|_| "*******"))
            .field("relays", &self.relays)
            .finish()
    }
}
