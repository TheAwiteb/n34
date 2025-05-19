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

use std::str::FromStr;

use clap::{ArgGroup, Args};
use nostr::{
    event::{EventBuilder, EventId, Kind, Tag},
    filter::Filter,
    nips::nip19::{self, FromBech32, Nip19Coordinate},
    types::RelayUrl,
};

use super::{CliOptions, CommandRunner, parsers};
use crate::{
    error::{N34Error, N34Result},
    nostr_utils::{NostrClient, utils},
};

/// Parses and represents a Nostr `nevent1` or `note1`.
#[derive(Debug, Clone)]
struct NostrEvent {
    /// Unique identifier for the event.
    event_id: EventId,
    /// List of relay URLs associated with the event. Empty if parsing a
    /// `note1`.
    relays:   Vec<RelayUrl>,
}

impl NostrEvent {
    /// Create a new [`NostrEvent`] instance
    fn new(event_id: EventId, relays: Vec<RelayUrl>) -> Self {
        Self { event_id, relays }
    }
}

impl FromStr for NostrEvent {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().starts_with("nevent1") {
            let event = nip19::Nip19Event::from_bech32(s).map_err(|e| e.to_string())?;
            Ok(Self::new(event.event_id, event.relays))
        } else if s.trim().starts_with("note1") {
            Ok(Self::new(
                EventId::from_bech32(s).map_err(|e| e.to_string())?,
                Vec::new(),
            ))
        } else {
            Err("Invalid event id, must starts with `note1` or `nevent1`".to_owned())
        }
    }
}


/// Arguments for the `reply` command
#[derive(Args, Debug)]
#[clap(
    group(
        ArgGroup::new("comment-content")
            .args(["comment", "editor"])
            .required(true)
    )
)]
pub struct ReplyArgs {
    /// The issue, patch, or comment to reply to
    #[arg(long)]
    to:      NostrEvent,
    /// Repository address
    #[arg(short, long, value_parser = parsers::repo_naddr)]
    naddr:   Option<Nip19Coordinate>,
    /// The comment (cannot be used with --editor)
    #[arg(short, long)]
    comment: Option<String>,
    /// Open editor to write comment (cannot be used with --content)
    #[arg(short, long)]
    editor:  bool,
}

impl CommandRunner for ReplyArgs {
    async fn run(self, options: CliOptions) -> N34Result<()> {
        let client = NostrClient::init(&options).await;
        let user_pubk = options.pubkey().await?;
        let relays_list = client.user_relays_list(user_pubk).await?;
        let mut write_relays =
            utils::add_write_relays(options.relays.clone(), relays_list.as_ref());
        client.add_relays(&options.relays).await;
        client.add_relays(&self.to.relays).await;

        if let Some(ref naddr) = self.naddr {
            client.add_relays(&naddr.relays).await;
        }

        let reply_to = client
            .fetch_event(Filter::new().id(self.to.event_id))
            .await?
            .ok_or(N34Error::EventNotFound)?;
        let root = client.find_root(reply_to.clone()).await?;


        let repo_naddr = if let Some(naddr) = self.naddr {
            naddr.coordinate
        } else if let Some(ref root_event) = root {
            root_event
                .tags
                .coordinates()
                .find(|c| c.kind == Kind::GitRepoAnnouncement)
                .ok_or_else(|| {
                    N34Error::InvalidEvent(
                        "The Git issue/patch does not specify a target repository".to_owned(),
                    )
                })?
                .clone()
        } else {
            return Err(N34Error::NotFoundRepo);
        };

        let repo = client.fetch_repo(&repo_naddr).await?;
        write_relays.extend(repo.relays.clone());

        write_relays = client
            .read_relays_from_user(write_relays, reply_to.pubkey)
            .await;
        if let Some(root_event) = &root {
            write_relays = client
                .read_relays_from_user(write_relays, root_event.pubkey)
                .await;
        }

        let content = utils::get_content(self.comment.as_ref(), ".txt")?;
        let content_details = client.parse_content(&content).await;
        write_relays.extend(content_details.write_relays.clone());

        let event = EventBuilder::comment(
            content,
            &reply_to,
            root.as_ref(),
            repo.relays.first().cloned(),
        )
        .tag(Tag::public_key(repo_naddr.public_key))
        .tags(content_details.into_tags())
        .pow(options.pow)
        .build(user_pubk);

        let event_id = event.id.expect("There is an id");
        let author_read_relays = utils::add_read_relays(Vec::new(), relays_list.as_ref());

        tracing::trace!(relays = ?write_relays, "Write relays list");
        let (success, ..) = futures::join!(
            client.send_event_to(event, relays_list.as_ref(), &write_relays),
            client.broadcast(&reply_to, &author_read_relays),
            async {
                if let Some(root_event) = root {
                    let _ = client.broadcast(&root_event, &author_read_relays).await;
                }
            },
        );


        let nevent = utils::new_nevent(event_id, &success?)?;
        println!("Comment created: {nevent}");

        Ok(())
    }
}
