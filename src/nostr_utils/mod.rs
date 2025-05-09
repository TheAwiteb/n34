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

use futures::future;
use nostr::{
    event::{Event, EventId, Kind, UnsignedEvent},
    filter::Filter,
    key::{Keys, PublicKey},
    nips::{nip01::Coordinate, nip34::GitRepositoryAnnouncement},
    types::RelayUrl,
};
use nostr_sdk::Client;

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

        client.add_relays(&options.relays).await;
        client
    }

    /// Add relays and connect to them
    pub async fn add_relays(&self, relays: &[RelayUrl]) {
        let mut tasks = Vec::new();
        for relay in relays {
            let relay = relay.clone();
            let client = self.client.clone();
            tasks.push(tokio::spawn(async move {
                client
                    .add_relay(&relay)
                    .await
                    .expect("It's a valid relay url");
                if let Err(err) = client.try_connect_relay(&relay, CLIENT_TIMEOUT).await {
                    tracing::error!("Failed to connect to relay '{relay}': {err}");
                }
            }));
        }
        future::join_all(tasks).await;
    }

    /// Broadcasts an unsigned event to given relays, optionally broadcast the
    /// relays list event. Returns URLs of relays that successfully received
    /// the event.
    pub async fn send_event_to(
        &self,
        mut event: UnsignedEvent,
        relays_list: Option<&Event>,
        relays: &[RelayUrl],
    ) -> N34Result<Vec<RelayUrl>> {
        event.ensure_id();
        self.add_relays(relays).await;
        let event_id = event.id.expect("It's there");

        if let Some(event) = relays_list {
            let _ = self.client.send_event_to(relays, event).await;
        }

        let result = self
            .client
            .send_event_to(relays, &event.sign(&self.client.signer().await?).await?)
            .await?;

        for relay in &result.success {
            tracing::info!(event_id = %event_id, relay = %relay, "Event sent successfully");
        }
        for (relay, reason) in &result.failed {
            tracing::warn!(event_id = %event_id, relay = %relay, reason = %reason, "Failed to send event");
        }

        Ok(result.success.into_iter().collect())
    }

    /// Fetches the first event matching the given filter, or None if no event
    /// is found.
    pub async fn fetch_event(&self, filter: Filter) -> N34Result<Option<Event>> {
        Ok(self
            .client
            .fetch_events(filter, CLIENT_TIMEOUT)
            .await?
            .first_owned())
    }

    /// Try to fetch a repository and returns it
    pub async fn fetch_repo(
        &self,
        repo_naddr: &Coordinate,
    ) -> N34Result<GitRepositoryAnnouncement> {
        let filter = Filter::new()
            .author(repo_naddr.public_key)
            .kind(Kind::GitRepoAnnouncement)
            .identifier(&repo_naddr.identifier);

        self.fetch_event(filter)
            .await?
            .map(|e| utils::event_into_repo(e, &repo_naddr.identifier))
            .ok_or(N34Error::NotFoundRepo)
    }

    /// Fetches the relay list (kind 10002) for the given user. Returns None if
    /// no relays are found.
    pub async fn user_relays_list(&self, user: PublicKey) -> N34Result<Option<Event>> {
        self.fetch_event(Filter::new().author(user).kind(Kind::RelayList))
            .await
    }

    /// Gets the author of the specified event, if found.
    pub async fn event_author(&self, event_id: EventId) -> N34Result<Option<PublicKey>> {
        Ok(self
            .fetch_event(Filter::new().id(event_id))
            .await?
            .map(|e| e.pubkey))
    }
}
