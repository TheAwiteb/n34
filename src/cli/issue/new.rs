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
use nostr::{
    event::{EventBuilder, Tag, TagStandard},
    nips::nip19::Nip19Coordinate,
    parser::NostrParser,
};

use crate::{
    cli::{CliOptions, CommandRunner, parsers},
    error::N34Result,
    nostr_utils::{
        NostrClient,
        traits::{NewGitRepositoryAnnouncement, TokenUtils},
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
    /// Repository address
    #[arg(short, long, value_parser = parsers::repo_naddr)]
    naddr:   Nip19Coordinate,
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
        let file_content = utils::read_editor(".md")?;
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
    async fn run(mut self, options: CliOptions) -> N34Result<()> {
        let client = NostrClient::init(&options).await;
        let user_pubk = options.pubkey().await?;
        let relays_list = client.user_relays_list(user_pubk).await?;
        let mut write_relays =
            utils::add_write_relays(options.relays.clone(), relays_list.as_ref());
        client.add_relays(&options.relays).await;
        client.add_relays(&self.naddr.relays).await;
        write_relays.extend(client.fetch_repo(&self.naddr).await?.relays);

        let (subject, content) = self.issue_content()?;
        let tokens = NostrParser::new().parse(&content).collect::<Vec<_>>();
        let mut p_tagged_users = tokens
            .iter()
            .filter_map(TokenUtils::extract_public_key)
            .collect::<Vec<_>>();
        let quote_events = tokens
            .iter()
            .filter_map(TokenUtils::extract_event_id)
            .collect::<Vec<_>>();

        self.label.extend(
            tokens
                .iter()
                .filter_map(TokenUtils::extract_hashtag)
                .collect::<Vec<_>>(),
        );

        // Add the p-tagged users read relays to our write relays. And relays with the
        // nprofile/nevent to our discovery relays
        for (user, relays) in &p_tagged_users {
            client.add_relays(relays).await;
            write_relays =
                utils::add_read_relays(write_relays, client.user_relays_list(*user).await?.as_ref())
        }
        for (event_id, relays) in &quote_events {
            client.add_relays(relays).await;
            // Add the note author to the p-tagged users
            if let Some(author) = client.event_author(*event_id).await? {
                p_tagged_users.push((author, Vec::new()));
                write_relays = utils::add_read_relays(
                    write_relays,
                    client.user_relays_list(author).await?.as_ref(),
                );
            }
        }

        let event = EventBuilder::new_git_issue(
            self.naddr.coordinate.clone(),
            content,
            subject,
            self.label,
        )?
        .pow(options.pow)
        .tags(p_tagged_users.into_iter().map(|(p, _)| Tag::public_key(p)))
        .tags(quote_events.into_iter().map(|(e, r)| {
            Tag::from_standardized(TagStandard::Quote {
                event_id:   e,
                relay_url:  r.first().cloned(),
                public_key: None,
            })
        }))
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
