// n34 - A CLI to interact with NIP-34 and other stuff related to code in Nostr
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
        traits::{CommandRunner, VecNostrEventExt},
        types::{EntityType, NaddrOrSet, NostrEvent, PatchPrStatus},
    },
    error::{N34Error, N34Result},
};

#[derive(Debug, Args)]
pub struct MergeArgs {
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
    naddrs:         Option<Vec<NaddrOrSet>>,
    /// The open patch id to merge it. Must be orignal root patch or
    /// revision root
    patch_id:       NostrEvent,
    /// Patches that have been merged. Use this when only some patches have been
    /// merged, not all.
    #[arg(long = "patches", value_name = "PATCH-EVENT-ID")]
    merged_patches: Vec<NostrEvent>,
    /// The merge commit id
    merge_commit:   Sha1Hash,
}

impl CommandRunner for MergeArgs {
    async fn run(self, options: CliOptions) -> N34Result<()> {
        crate::cli::common_commands::patch_pr_status_command::<{ EntityType::Patch as u8 }>(
            options,
            self.patch_id,
            self.naddrs,
            PatchPrStatus::MergedApplied,
            Some(either::Either::Left(self.merge_commit)),
            self.merged_patches.into_event_ids(),
            |patch_status| {
                if patch_status.is_merged_or_applied() {
                    return Err(N34Error::InvalidStatus(
                        "You can't merge an already merged/applied patch".to_owned(),
                    ));
                }

                if patch_status.is_closed() {
                    return Err(N34Error::InvalidStatus(
                        "You can't merge a closed patch".to_owned(),
                    ));
                }

                if patch_status.is_drafted() {
                    return Err(N34Error::InvalidStatus(
                        "You can't merge a draft patch".to_owned(),
                    ));
                }

                Ok(())
            },
        )
        .await
    }
}
