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

use std::borrow::Cow;

use clap::Args;
use nostr::event::{Kind, Tag, TagKind};
use nostr::{event::EventBuilder, hashes::sha1::Hash as Sha1Hash};

use crate::nostr_utils::traits::ReposUtils;
use crate::{
    cli::{
        CliOptions,
        CommandRunner,
        parsers,
        traits::{OptionNaddrOrSetVecExt, RelayOrSetVecExt},
        types::NaddrOrSet,
    },
    error::N34Result,
    nostr_utils::{NostrClient, traits::NaddrsUtils, utils},
};

/// Prefix for branch references in Git.
const HEADS_REFS: &str = "refs/heads/";

/// Prefix for tag references in Git.
const TAGS_REFS: &str = "refs/tags/";

/// Repository state announcements kind
const REPO_STATE_KIND: Kind = Kind::Custom(30618);

/// `HEAD` tag kind
const HEAD_TAG_KIND: TagKind = TagKind::Custom(Cow::Borrowed("HEAD"));

/// Arguments for the `repo state` command
#[derive(Args, Debug)]
pub struct StateArgs {
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
    naddrs:   Option<Vec<NaddrOrSet>>,
    /// Tags to announce a state for, in the format `<tag-name>=<commit-id>`.
    /// Separated by comma.
    ///
    /// Example: `v0.4.0=9aa3b62de02a63aa6a0d49efa7c484aa550cef56`.
    #[arg(long, value_delimiter = ',', value_parser = parsers::name_and_sha1)]
    tags:     Vec<(String, Sha1Hash)>,
    /// Branches to announce a state for, in the format
    /// `<branch-name>=<commit-id>`. Separated by comma.
    ///
    /// Example: `master=9aa3b62de02a63aa6a0d49efa7c484aa550cef56`.
    #[arg(long, value_delimiter = ',', value_parser = parsers::name_and_sha1)]
    branches: Vec<(String, Sha1Hash)>,
    /// Name of the repository's primary branch, such as 'master' or 'main'.
    head:     String,
}

impl CommandRunner for StateArgs {
    async fn run(self, options: CliOptions) -> N34Result<()> {
        let naddrs = utils::check_empty_naddrs(utils::naddrs_or_file(
            self.naddrs.flat_naddrs(&options.config.sets)?,
            &utils::nostr_address_path()?,
        )?)?;
        let relays = options.relays.clone().flat_relays(&options.config.sets)?;
        let client = NostrClient::init(&options, &relays).await;
        let user_pubk = client.pubkey().await?;
        client.add_relays(&naddrs.extract_relays()).await;

        let repos = client
            .fetch_repos(&naddrs.clone().into_coordinates())
            .await?;
        let repos_id = repos
            .first()
            .cloned()
            .expect("It's not empty, checked above")
            .id;

        let mut event_builder = EventBuilder::new(REPO_STATE_KIND, "")
            .dedup_tags()
            .pow(options.pow.unwrap_or_default())
            .tag(Tag::identifier(&repos_id))
            .tag(Tag::custom(
                HEAD_TAG_KIND,
                &[format!("ref: {HEADS_REFS}{}", self.head)],
            ));

        if !self.branches.is_empty() {
            event_builder = event_builder.tags(refs_tags::<true>(self.branches));
        }

        if !self.tags.is_empty() {
            event_builder = event_builder.tags(refs_tags::<false>(self.tags));
        }

        let event = event_builder.build(user_pubk);
        let event_id = event.id.expect("There is an id");
        let user_relays_list = client.user_relays_list(user_pubk).await?;
        let write_relays = [
            relays,
            naddrs.extract_relays(),
            repos.extract_relays(),
            utils::add_write_relays(user_relays_list.as_ref()),
            // Include read relays for each maintainer (if found)
            client
                .read_relays_from_users(&repos.extract_maintainers())
                .await,
        ]
        .concat();

        tracing::trace!(relays = ?write_relays, "Write relays list");
        let success = client
            .send_event_to(event, user_relays_list.as_ref(), &write_relays)
            .await?;

        let nevent = utils::new_nevent(event_id, &success)?;
        let naddr = utils::repo_naddr(repos_id, user_pubk, &success)?;
        println!("Event created: {nevent}");
        println!("State address: {naddr}");

        Ok(())
    }
}

/// Build the refs tags
#[inline]
fn refs_tags<const IS_HEADS: bool>(refs: Vec<(String, Sha1Hash)>) -> impl IntoIterator<Item = Tag> {
    refs.into_iter().map(|(tag, commit)| {
        Tag::parse(&[
            format!("{}{tag}", if IS_HEADS { HEADS_REFS } else { TAGS_REFS }),
            commit.to_string(),
        ])
        .expect("Not an empty tag")
    })
}
