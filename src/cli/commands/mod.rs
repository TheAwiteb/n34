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


/// `issue` subcommands
mod issue;
/// 'reply` command
mod reply;
/// `repo` subcommands
mod repo;


use std::fmt;

use clap::{ArgGroup, Args, Parser};
use nostr::key::{Keys, PublicKey, SecretKey};
use nostr::types::RelayUrl;

use self::issue::IssueSubcommands;
use self::reply::ReplyArgs;
use self::repo::RepoSubcommands;
use super::traits::CommandRunner;
use crate::error::{N34Error, N34Result};

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
    /// Fallbacks relay to write and read from it. Multiple relays can be
    /// passed.
    #[arg(short, long)]
    pub relays:     Vec<RelayUrl>,
    /// Proof of Work difficulty when creatring events
    #[arg(long, default_value_t = 0)]
    pub pow:        u8,
}

/// N34 commands
#[derive(Parser, Debug)]
pub enum Commands {
    /// Manage repositories
    Repo {
        #[command(subcommand)]
        subcommands: RepoSubcommands,
    },
    /// Manage issues
    Issue {
        #[command(subcommand)]
        subcommands: IssueSubcommands,
    },
    // /// Manage patches
    // Patch {
    //     #[command(subcommand)]
    //     subcommands: PatchSubcommands,
    // },
    /// Reply to issues and patches.
    Reply(ReplyArgs),
}


impl CommandRunner for Commands {
    async fn run(self, options: CliOptions) -> N34Result<()> {
        tracing::trace!("Options: {options:#?}");
        tracing::trace!("Handling: {self:#?}");
        match self {
            Self::Repo { subcommands } => subcommands.run(options).await,
            Self::Issue { subcommands } => subcommands.run(options).await,
            Commands::Reply(args) => args.run(options).await,
        }
    }
}

impl CliOptions {
    /// Gets the public key of the user.
    pub async fn pubkey(&self) -> N34Result<PublicKey> {
        if let Some(sk) = &self.secret_key {
            return Ok(Keys::new(sk.clone()).public_key());
        }
        unreachable!("There is no other method until now")
    }

    /// Returns an error if there are no relays.
    pub fn ensure_relays(&self) -> N34Result<()> {
        if self.relays.is_empty() {
            return Err(N34Error::EmptyRelays);
        }
        Ok(())
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
