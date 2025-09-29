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

use std::iter;

use clap::{ArgGroup, Args};
use nostr::{
    event::{EventBuilder, Tag, TagKind, TagStandard, Tags},
    filter::{Alphabet, Filter},
    hashes::sha1::Hash as Sha1Hash,
};

use crate::{
    cli::{
        CliOptions,
        traits::{CommandRunner, OptionNaddrOrSetVecExt, RelayOrSetVecExt},
        types::{NaddrOrSet, NostrEvent},
    },
    error::{N34Error, N34Result},
    nostr_utils::{
        NostrClient,
        traits::{NaddrsUtils, ReposUtils},
        utils,
    },
};

#[derive(Args, Debug)]
#[clap(
    group(
        ArgGroup::new("clone-or-grasp")
        .required(true)
    )
)]
pub struct UpdateArgs {
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
    naddrs:      Option<Vec<NaddrOrSet>>,
    /// Original PR ID
    #[arg(value_name = "EVENT-ID")]
    original_pr: NostrEvent,
    /// Push the pull request update to the repository GRASP server.
    #[arg(long, group = "clone-or-grasp")]
    grasp:       bool,
    /// The SHA-1 hash of the commit at the tip of the PR branch.
    ///
    /// You can get it using `git rev-parse <branch-name>`
    commit:      Sha1Hash,
    /// Repositories to clone for the pull request, separated by commas.
    #[arg(value_delimiter = ',', group = "clone-or-grasp")]
    clones:      Vec<nostr::Url>,
}

impl CommandRunner for UpdateArgs {
    async fn run(self, options: CliOptions) -> N34Result<()> {
        let naddrs = utils::check_empty_naddrs(utils::naddrs_or_file(
            self.naddrs.flat_naddrs(&options.config.sets)?,
            &utils::nostr_address_path()?,
        )?)?;
        let relays = options.relays.clone().flat_relays(&options.config.sets)?;
        let client = NostrClient::init(&options, &relays).await;
        let naddrs_relays = naddrs.extract_relays();
        client.add_relays(&naddrs_relays).await;
        let coordinates = naddrs.into_coordinates();
        let repos = client.fetch_repos(coordinates.as_slice()).await?;
        let maintainers = repos.extract_maintainers();
        let repos_relays = repos.extract_relays();
        client.add_relays(&repos_relays).await;
        let user_pubk = client.pubkey().await?;
        let relays_list = client.user_relays_list(user_pubk).await?;
        client
            .add_relays(&utils::add_read_relays(relays_list.as_ref()))
            .await;

        let Some(orignal_pr) = client
            .fetch_event(
                Filter::new()
                    .id(self.original_pr.event_id)
                    .kind(super::PR_KIND),
            )
            .await?
        else {
            tracing::error!("Can't find the original pull request");
            return Err(N34Error::EventNotFound);
        };

        // TODO: Use `CommentTarget` to mention the orignal PR
        let mut nip22_orignal_pr = Tags::new();
        nip22_orignal_pr.push(Tag::from_standardized_without_cell(TagStandard::Event {
            event_id:   orignal_pr.id,
            relay_url:  None,
            marker:     None,
            public_key: Some(orignal_pr.pubkey),
            uppercase:  true,
        }));
        nip22_orignal_pr.push(Tag::from_standardized_without_cell(
            TagStandard::PublicKey {
                public_key: orignal_pr.pubkey,
                relay_url:  None,
                alias:      None,
                uppercase:  true,
            },
        ));
        nip22_orignal_pr.push(Tag::from_standardized_without_cell(TagStandard::Kind {
            kind:      orignal_pr.kind,
            uppercase: true,
        }));

        let mut event_builder = EventBuilder::new(super::PR_UPDATE_KIND, "")
            .pow(options.pow.unwrap_or_default())
            .tags(nip22_orignal_pr)
            .tags(
                coordinates
                    .into_iter()
                    .map(|c| Tag::coordinate(c, repos_relays.first().cloned())),
            )
            .tags(maintainers.iter().map(|p| Tag::public_key(*p)))
            .tag(Tag::custom(
                TagKind::single_letter(Alphabet::C, false),
                iter::once(self.commit.to_string()),
            ));

        let event = if self.grasp {
            utils::build_grasp_event(&repos, user_pubk, event_builder.clone())?
        } else {
            // Since `grasp` is false, `clones` must be provided
            event_builder = event_builder.tag(Tag::custom(
                TagKind::custom("clone"),
                self.clones.iter().map(ToString::to_string),
            ));

            event_builder.build(user_pubk)
        };

        let event_id = event.id.expect("There is an id");

        let write_relays = [
            relays,
            repos_relays,
            naddrs_relays,
            utils::add_write_relays(relays_list.as_ref()),
            // Include read relays for each maintainer (if found)
            client.read_relays_from_users(&maintainers).await,
        ]
        .concat();

        tracing::trace!(relays = ?write_relays, "Write relays list");
        let success = client
            .send_event_to(event, relays_list.as_ref(), &write_relays)
            .await?;

        let nevent = utils::new_nevent(event_id, &success)?;
        println!("PR update tip created: {nevent}");

        Ok(())
    }
}
