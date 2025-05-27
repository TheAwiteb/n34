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


use clap::{ArgGroup, Args};
use futures::future;
use nostr::{
    event::{EventBuilder, Tag},
    nips::nip19::Nip19Coordinate,
};

use crate::{
    cli::{CliOptions, CommandRunner, parsers},
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
        ArgGroup::new("issue-content")
            .args(["content", "editor"])
            .required(true)
    ),
    group(
        ArgGroup::new("issue-subject")
            .args(["editor", "subject"])
    )
)]
pub struct NewArgs {
    /// Repository address in `naddr` format or `<nip5>/repo_id`. e.g.
    /// `4rs.nl/n34` and `_@4rs.nl/n34`
    ///
    /// If not provided, `n34` will look for the `nostr-address` file.
    #[arg(value_name = "NADDR-OR-NIP05", long = "repo", value_parser = parsers::repo_naddr)]
    naddrs:  Option<Vec<Nip19Coordinate>>,
    /// Markdown content for the issue. Cannot be used together with the
    /// `--editor` flag.
    #[arg(short, long)]
    content: Option<String>,
    /// Opens the user's default editor to write issue content. The first line
    /// will be used as the issue subject.
    #[arg(short, long)]
    editor:  bool,
    /// The issue subject. Cannot be used together with the `--editor` flag.
    #[arg(long)]
    subject: Option<String>,
    /// Labels for the issue. Can be specified as arguments (-l bug) or hashtags
    /// in content (#bug).
    #[arg(short, long)]
    label:   Vec<String>,
}

impl NewArgs {
    /// Returns the subject and the content of the issue. (subject, content)
    pub fn issue_content(&self) -> N34Result<(Option<String>, String)> {
        if let Some(content) = self.content.as_ref() {
            if let Some(subject) = self.subject.as_ref() {
                return Ok((Some(subject.trim().to_owned()), content.trim().to_owned()));
            }
            return Ok((None, content.trim().to_owned()));
        }
        // If the `self.content` is `None` then the `self.editor` is `true`
        let file_content = utils::read_editor(None, ".md")?;
        if file_content.contains('\n') {
            Ok(file_content
                .split_once('\n')
                .map(|(s, c)| (Some(s.trim().to_owned()), c.trim().to_owned()))
                .expect("There is a new line"))
        } else {
            tracing::info!("File content contains only issue body without a subject line");
            Ok((None, file_content))
        }
    }
}

impl CommandRunner for NewArgs {
    async fn run(self, options: CliOptions) -> N34Result<()> {
        let client = NostrClient::init(&options).await;
        let user_pubk = options.pubkey().await?;
        let naddrs = utils::naddrs_or_file(self.naddrs.clone(), &utils::nostr_address_path()?)?;
        let mut naddrs_iter = naddrs.clone().into_iter();

        client.add_relays(&naddrs.extract_relays()).await;

        let relays_list = client.user_relays_list(user_pubk).await?;
        let mut write_relays = [
            options.relays,
            utils::add_write_relays(relays_list.as_ref()),
            client
                .fetch_repos(&naddrs.into_coordinates())
                .await?
                .extract_relays(),
        ]
        .concat();

        let (subject, content) = self.issue_content()?;
        let content_details = client.parse_content(&content).await;
        write_relays.extend(content_details.write_relays.clone());

        // Include read relays for each repository owner (if found)
        write_relays.extend(
            future::join_all(
                naddrs_iter
                    .clone()
                    .map(|c| client.read_relays_from_user(c.public_key)),
            )
            .await
            .into_iter()
            .flatten(),
        );

        let event = EventBuilder::new_git_issue(
            naddrs_iter
                .next()
                .expect("There is at least one address")
                .coordinate
                .clone(),
            content,
            subject,
            self.label,
        )?
        .dedup_tags()
        .pow(options.pow)
        .tags(content_details.into_tags())
        // p-tag the reset of the reposotoies owners
        .tags(naddrs_iter.clone().map(|n| Tag::public_key(n.public_key)))
        // a-tag the reset of the reposotoies
        .tags(naddrs_iter.map(|n| Tag::coordinate(n.coordinate, n.relays.first().cloned())))
        .build(user_pubk);
        let event_id = event.id.expect("There is an id");

        tracing::trace!(relays = ?write_relays, "Write relays list");
        let success = client
            .send_event_to(event, relays_list.as_ref(), &write_relays)
            .await?;

        let nevent = utils::new_nevent(event_id, &success)?;
        println!("Issue created: {nevent}");

        Ok(())
    }
}
