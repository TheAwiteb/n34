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

use clap::Args;
use nostr::hashes::sha1::Hash as Sha1Hash;

use crate::{
    cli::{
        CliOptions,
        traits::CommandRunner,
        types::{EntityType, NaddrOrSet, NostrEvent, PatchPrStatus},
    },
    error::{N34Error, N34Result},
};

#[derive(Debug, Args)]
pub struct ApplyArgs {
    /// Repository addresses
    ///
    /// In `naddr` format (`naddr1...`), NIP-05 format (`4rs.nl/n34` or
    /// `_@4rs.nl/n34`), or a set name like `kernel`, separated by commas.
    ///
    /// If omitted, looks for a `nostr-address` file.
    #[arg(
        value_name = "NADDR-NIP05-OR-SET",
        long = "repo",
        value_delimiter = ','
    )]
    naddrs:          Option<Vec<NaddrOrSet>>,
    /// The open PR id to apply it.
    pr_id:           NostrEvent,
    /// The applied commits.
    #[arg(required = true)]
    applied_commits: Vec<Sha1Hash>,
}

impl CommandRunner for ApplyArgs {
    async fn run(self, options: CliOptions) -> N34Result<()> {
        crate::cli::common_commands::patch_pr_status_command::<{ EntityType::PullRequest as u8 }>(
            options,
            self.pr_id,
            self.naddrs,
            PatchPrStatus::MergedApplied,
            Some(either::Either::Right(self.applied_commits)),
            Vec::new(),
            |pr_status| {
                if pr_status.is_merged_or_applied() {
                    return Err(N34Error::InvalidStatus(
                        "You can't apply an already applied pull request".to_owned(),
                    ));
                }

                if pr_status.is_closed() {
                    return Err(N34Error::InvalidStatus(
                        "You can't apply a closed pull request".to_owned(),
                    ));
                }

                if pr_status.is_drafted() {
                    return Err(N34Error::InvalidStatus(
                        "Cannot apply a draft pull request".to_owned(),
                    ));
                }
                Ok(())
            },
        )
        .await
    }
}
