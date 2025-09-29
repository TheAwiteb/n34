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


use clap::{ArgGroup, Args};
use nostr::event::{EventBuilder, Tag};

use crate::{
    cli::{
        CliOptions,
        CommandRunner,
        traits::{OptionNaddrOrSetVecExt, RelayOrSetVecExt},
        types::NaddrOrSet,
    },
    error::N34Result,
    nostr_utils::{
        NostrClient,
        traits::{NaddrsUtils, NewGitRepositoryAnnouncement, ReposUtils},
        utils,
    },
};


/// Arguments for the `issue new` command
#[derive(Args, Debug)]
#[clap(
    group(
        ArgGroup::new("issue-subject")
            .required(true)
    )
)]
pub struct NewArgs {
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
    naddrs:  Option<Vec<NaddrOrSet>>,
    /// Markdown content for the issue. Cannot be used together with the
    /// `--editor` flag.
    #[arg(short, long, group = "issue-content")]
    content: Option<String>,
    /// Opens the user's default editor to write issue content. The first line
    /// will be used as the issue subject.
    #[arg(short, long, group = "issue-subject", group = "issue-content")]
    editor:  bool,
    /// The issue subject. Cannot be used together with the `--editor` flag.
    #[arg(long, group = "issue-subject")]
    subject: Option<String>,
    /// Labels for the issue. Can be specified as arguments (-l bug) or hashtags
    /// in content (#bug).
    #[arg(short, long)]
    label:   Vec<String>,
}

impl CommandRunner for NewArgs {
    async fn run(self, options: CliOptions) -> N34Result<()> {
        let naddrs = utils::check_empty_naddrs(utils::naddrs_or_file(
            self.naddrs.flat_naddrs(&options.config.sets)?,
            &utils::nostr_address_path()?,
        )?)?;
        let relays = options.relays.clone().flat_relays(&options.config.sets)?;
        let client = NostrClient::init(&options, &relays).await;
        let user_pubk = client.pubkey().await?;
        let coordinates = naddrs.clone().into_coordinates();
        client.add_relays(&naddrs.extract_relays()).await;
        let repos = client.fetch_repos(coordinates.as_slice()).await?;
        let maintainers = repos.extract_maintainers();
        client.add_relays(&repos.extract_relays()).await;
        let relays_list = client.user_relays_list(user_pubk).await?;
        client
            .add_relays(&utils::add_read_relays(relays_list.as_ref()))
            .await;

        let (subject, content) = utils::subject_and_body(self.subject, self.content, ".md")?;

        let content_details = if let Some(content) = &content {
            Some(client.parse_content(content).await)
        } else {
            None
        };

        let event = EventBuilder::new_git_issue(
            coordinates.as_slice(),
            content.unwrap_or_default(),
            Some(subject),
            self.label,
        )?
        .dedup_tags()
        .pow(options.pow.unwrap_or_default())
        .tags(maintainers.iter().map(|p| Tag::public_key(*p)))
        .tags(
            content_details
                .clone()
                .map(|c| c.into_tags())
                .unwrap_or_default(),
        )
        .build(user_pubk);
        let event_id = event.id.expect("There is an id");

        let write_relays = [
            relays,
            naddrs.extract_relays(),
            utils::add_write_relays(relays_list.as_ref()),
            repos.extract_relays(),
            // Include read relays for each maintainer (if found)
            client.read_relays_from_users(&maintainers).await,
            content_details
                .map(|c| c.write_relays.into_iter().collect())
                .unwrap_or_default(),
        ]
        .concat();

        tracing::trace!(relays = ?write_relays, "Write relays list");
        let success = client
            .send_event_to(event, relays_list.as_ref(), &write_relays)
            .await?;

        let nevent = utils::new_nevent(event_id, &success)?;
        println!("Issue created: {nevent}");

        Ok(())
    }
}
