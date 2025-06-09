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

/// `issue new` subcommand
mod new;
/// `issue view` subcommand
mod view;

use clap::Subcommand;

use self::new::NewArgs;
use self::view::ViewArgs;
use super::{CliOptions, CommandRunner};
use crate::error::N34Result;

/// Prefix used for git issue alt.
pub const ISSUE_ALT_PREFIX: &str = "git issue: ";

#[derive(Subcommand, Debug)]
pub enum IssueSubcommands {
    /// Create a new repository issue
    New(NewArgs),
    /// View an issue by its ID
    View(ViewArgs),
}

impl CommandRunner for IssueSubcommands {
    async fn run(self, options: CliOptions) -> N34Result<()> {
        crate::run_command!(self, options, & New View)
    }
}
