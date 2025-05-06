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

/// Extension traits for nostr types.
pub mod traits;
/// Utility functions for nostr.
pub mod utils;

use std::time::Duration;

use nostr::{
    event::{Event, EventId, Kind, UnsignedEvent},
    filter::Filter,
    key::{Keys, PublicKey},
    nips::{nip19::Nip19Coordinate, nip34::GitRepositoryAnnouncement},
    types::RelayUrl,
};
use nostr_sdk::{Client, pool::Output};

use crate::{
    cli::CliOptions,
    error::{N34Error, N34Result},
};

/// Timeout duration for the clinet.
const CLIENT_TIMEOUT: Duration = Duration::from_millis(1500);

/// A client for interacting with the Nostr relays
pub struct NostrClient {
    /// The underlying Nostr client implementation
    client: Client,
}

impl NostrClient {
    /// Creates a new [`NostrClient`] with the given client and options.
    const fn new(client: Client) -> Self {
        Self { client }
    }

    /// Initializes a new [`NostrClient`] instance and connects to the specified
    /// relays.
    pub async fn init(options: &CliOptions) -> Self {
        let client = Self::new(
            Client::builder()
                .signer(Keys::new(
                    options
                        .secret_key
                        .as_ref()
                        .expect("This the only method for now")
                        .clone(),
                ))
                .build(),
        );

        client.add_read_relays(&options.relays).await;
        client
    }

    /// Add read relays and connect to them
    pub async fn add_read_relays(&self, relays: &[RelayUrl]) {
        for relay in relays {
            self.client
                .add_read_relay(relay)
                .await
                .expect("It's a valid relay url");
            if let Err(err) = self.client.try_connect_relay(relay, CLIENT_TIMEOUT).await {
                tracing::error!("Failed to connect to relay '{relay}': {err}");
            }
        }
    }

    /// Sends an event to the specified relays.
    pub async fn send_event_to(
        &self,
        event: UnsignedEvent,
        relays_list: Option<&Event>,
        mut relays: Vec<RelayUrl>,
    ) -> N34Result<Output<EventId>> {
        relays.sort_unstable();
        relays.dedup();
        for relay in &relays {
            let _ = self.client.add_write_relay(relay).await;
        }

        if let Some(event) = relays_list {
            let _ = self.client.send_event_to(&relays, event).await;
        }
        self.client
            .send_event_to(relays, &event.sign(&self.client.signer().await?).await?)
            .await
            .map_err(N34Error::from)
    }

    /// Try to fetch a repository and returns it
    pub async fn fetch_repo(
        &self,
        repo_naddr: &Nip19Coordinate,
    ) -> N34Result<GitRepositoryAnnouncement> {
        let filter = Filter::new()
            .author(repo_naddr.public_key)
            .kind(Kind::GitRepoAnnouncement)
            .identifier(&repo_naddr.identifier);
        let events = self
            .client
            .fetch_events(filter, CLIENT_TIMEOUT)
            .await
            .map_err(|_| N34Error::NotFoundRepo)?;


        Ok(utils::event_into_repo(
            events.first_owned().ok_or(N34Error::NotFoundRepo)?,
            &repo_naddr.identifier,
        ))
    }

    /// Fetches the relay list (kind 10002) for the given user. Returns None if
    /// no relays are found.
    pub async fn user_relays_list(&self, user: PublicKey) -> N34Result<Option<Event>> {
        Ok(self
            .client
            .fetch_events(
                Filter::new().author(user).kind(Kind::RelayList),
                CLIENT_TIMEOUT,
            )
            .await?
            .first_owned())
    }
}
