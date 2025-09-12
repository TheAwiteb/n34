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

/// `pr apply` suubcommand
mod apply;
/// `pr close` subcommand
mod close;
/// `pr draft` subcommand
mod draft;
/// `pr list` subcommand
mod list;
/// `pr merge` subcommand
mod merge;
/// `pr new` subcommand
mod new;
/// `pr reopen` subcommand
mod reopen;
/// `pr update` subcommand
mod update;
/// `pr view` subcommand
mod view;

use clap::Subcommand;

use self::apply::ApplyArgs;
use self::close::CloseArgs;
use self::draft::DraftArgs;
use self::list::ListArgs;
use self::merge::MergeArgs;
use self::new::NewArgs;
use self::reopen::ReopenArgs;
use self::update::UpdateArgs;
use self::view::ViewArgs;
use crate::{
    cli::{CliOptions, traits::CommandRunner},
    error::N34Result,
};

/// The kind of the pull request
pub const PR_KIND: nostr::event::Kind = nostr::event::Kind::Custom(1618);
/// The kind of the pull request update
pub const PR_UPDATE_KIND: nostr::event::Kind = nostr::event::Kind::Custom(1619);


#[derive(Subcommand, Debug)]
pub enum PrSubcommands {
    /// Create a pull request.
    New(NewArgs),
    /// Update a pull request.
    Update(UpdateArgs),
    /// View a pull request.
    View(ViewArgs),
    /// List pull requests.
    List(ListArgs),
    /// Close a pull request.
    Close(CloseArgs),
    /// Convert to draft.
    Draft(DraftArgs),
    /// Reopen pull request.
    Reopen(ReopenArgs),
    /// Mark as applied.
    Apply(ApplyArgs),
    /// Merge a pull request.
    Merge(MergeArgs),
}

impl CommandRunner for PrSubcommands {
    async fn run(self, options: CliOptions) -> N34Result<()> {
        crate::run_command!(self, options, & New Update View List Close Draft Reopen Apply Merge)
    }
}
