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

use std::{fs, str::FromStr};

use clap::{ArgGroup, Args};
use futures::future;
use nostr::{
    event::{Event, EventBuilder, EventId, Kind},
    filter::Filter,
    nips::{
        nip01::{Coordinate, Metadata},
        nip19::{self, FromBech32, Nip19Coordinate, ToBech32},
    },
    types::RelayUrl,
};

use super::{CliOptions, CommandRunner};
use crate::{
    cli::parsers,
    error::{N34Error, N34Result},
    nostr_utils::{
        NostrClient,
        traits::{NaddrsUtils, ReposUtils},
        utils,
    },
};

/// Length of a Nostr npub (public key) in characters.
const NPUB_LEN: usize = 63;
/// The max date "9999-01-01 at 00:00 UTC"
const MAX_DATE: i64 = 253370764800;

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
        let str_event = s.trim().trim_start_matches("nostr:");
        if str_event.starts_with("nevent1") {
            let event = nip19::Nip19Event::from_bech32(str_event).map_err(|e| e.to_string())?;
            Ok(Self::new(event.event_id, event.relays))
        } else if str_event.starts_with("note1") {
            Ok(Self::new(
                EventId::from_bech32(str_event).map_err(|e| e.to_string())?,
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
    ),
    group(
        ArgGroup::new("quote-reply-to")
            .args(["comment", "quote_to"])
    )
)]
pub struct ReplyArgs {
    /// The issue, patch, or comment to reply to
    #[arg(long, value_name = "nevent1-or-note1")]
    to:       NostrEvent,
    /// Quote the replied-to event in the editor
    #[arg(long)]
    quote_to: bool,
    /// Repository address in `naddr` format or `<nip5>/repo_id`. e.g.
    /// `4rs.nl/n34` and `_@4rs.nl/n34`
    ///
    /// If not provided, `n34` will look for the `nostr-address` file and if not
    /// found, will get it from the root event if found.
    #[arg(value_name = "NADDR-OR-NIP05", long = "repo", value_parser = parsers::repo_naddr)]
    naddrs:   Option<Vec<Nip19Coordinate>>,
    /// The comment (cannot be used with --editor)
    #[arg(short, long)]
    comment:  Option<String>,
    /// Open editor to write comment (cannot be used with --content)
    #[arg(short, long)]
    editor:   bool,
}

impl CommandRunner for ReplyArgs {
    async fn run(self, options: CliOptions) -> N34Result<()> {
        let nostr_address_path = utils::nostr_address_path()?;
        let client = NostrClient::init(&options).await;
        let user_pubk = options.pubkey().await?;

        let repo_naddrs = if let Some(naddrs) = self.naddrs {
            client.add_relays(&naddrs.extract_relays()).await;
            Some(naddrs)
        } else if fs::exists(&nostr_address_path).is_ok() {
            let naddrs = utils::naddrs_or_file(None, &nostr_address_path)?;
            client.add_relays(&naddrs.extract_relays()).await;
            Some(naddrs)
        } else {
            None
        };

        client.add_relays(&self.to.relays).await;

        let relays_list = client.user_relays_list(user_pubk).await?;
        let mut write_relays =
            utils::add_write_relays(options.relays.clone(), relays_list.as_ref());

        let reply_to = client
            .fetch_event(Filter::new().id(self.to.event_id))
            .await?
            .ok_or(N34Error::EventNotFound)?;
        let root = client.find_root(reply_to.clone()).await?;

        let repos_coordinate = if let Some(naddrs) = repo_naddrs {
            naddrs.into_coordinates()
        } else if let Some(ref root_event) = root {
            coordinates_from_root(root_event)?
        } else {
            return Err(N34Error::NotFoundRepo);
        };

        let repos = client.fetch_repos(&repos_coordinate).await?;
        // Merge repository announcement relays into write relays
        write_relays.extend(repos.extract_relays());
        // Include read relays for each repository owner (if found)
        write_relays.extend(
            future::join_all(
                repos_coordinate
                    .iter()
                    .map(|c| client.read_relays_from_user(Vec::new(), c.public_key)),
            )
            .await
            .into_iter()
            .flatten(),
        );

        write_relays = client
            .read_relays_from_user(write_relays, reply_to.pubkey)
            .await;
        if let Some(root_event) = &root {
            write_relays = client
                .read_relays_from_user(write_relays, root_event.pubkey)
                .await;
        }

        let quoted_content = if self.quote_to {
            Some(quote_reply_to_content(&client, &reply_to).await)
        } else {
            None
        };

        let content = utils::get_content(self.comment.as_ref(), quoted_content.as_ref(), ".txt")?;
        let content_details = client.parse_content(&content).await;
        write_relays.extend(content_details.write_relays.clone());

        let event = EventBuilder::comment(
            content,
            &reply_to,
            root.as_ref(),
            repos.first().and_then(|r| r.relays.first()).cloned(),
        )
        .dedup_tags()
        .pow(options.pow)
        .tags(content_details.into_tags())
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

/// Creates a quoted reply string in the format "On yyyy-mm-dd at hh:mm UTC,
/// <author> wrote:" followed by the event content. Uses display name if
/// available, otherwise falls back to a shortened npub string. Dates are
/// formatted in UTC.
async fn quote_reply_to_content(client: &NostrClient, quoted_event: &Event) -> String {
    let author_name = client
        .fetch_event(
            Filter::new()
                .kind(Kind::Metadata)
                .author(quoted_event.pubkey),
        )
        .await
        .ok()
        .flatten()
        .and_then(|e| Metadata::try_from(&e).ok())
        .and_then(|m| m.display_name.or(m.name))
        .unwrap_or_else(|| {
            let pubkey = quoted_event
                .pubkey
                .to_bech32()
                .expect("The error is `Infallible`");
            format!("{}...{}", &pubkey[..8], &pubkey[NPUB_LEN - 8..])
        });

    let fdate = chrono::DateTime::from_timestamp(
        quoted_event
            .created_at
            .as_u64()
            .try_into()
            .unwrap_or(MAX_DATE),
        0,
    )
    .map(|datetime| datetime.format("On %F at %R UTC, ").to_string())
    .unwrap_or_default();

    format!(
        "{fdate}{author_name} wrote:\n> {}",
        quoted_event.content.trim().replace("\n", "\n> ")
    )
}

/// Gets the repository coordinate from a root Nostr event's tags.
/// The event must contain a coordinate tag with GitRepoAnnouncement kind.
fn coordinates_from_root(root: &Event) -> N34Result<Vec<Coordinate>> {
    let coordinates: Vec<Coordinate> = root
        .tags
        .coordinates()
        .filter(|c| c.kind == Kind::GitRepoAnnouncement)
        .cloned()
        .collect();

    if coordinates.is_empty() {
        return Err(N34Error::InvalidEvent(
            "The Git issue/patch does not specify a target repository".to_owned(),
        ));
    }

    Ok(coordinates)
}
