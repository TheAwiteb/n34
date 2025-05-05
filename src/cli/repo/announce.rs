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

use clap::Args;
use convert_case::{Case, Casing};
use nostr::{
    event::{EventBuilder, Kind, Tag},
    key::PublicKey,
    nips::{
        nip01::Coordinate,
        nip19::{Nip19Coordinate, Nip19Event, ToBech32},
        nip34::GitRepositoryAnnouncement,
    },
    types::Url,
};

use crate::{
    cli::{CliOptions, CommandRunner},
    error::{N34Error, N34Result},
    nostr_utils::{NostrClient, traits::NewGitRepositoryAnnouncement},
};


/// Arguments for the `repo announce` command
#[derive(Args, Debug)]
pub struct AnnounceArgs {
    /// Unique identifier for the repository in kebab-case.
    #[arg(long = "id")]
    repo_id:     String,
    /// A name for the repository.
    #[arg(short, long)]
    name:        Option<String>,
    /// A description for the repository.
    #[arg(short, long)]
    description: Option<String>,
    /// Webpage URLs for the repository (if provided by the git server).
    #[arg(short, long)]
    web:         Vec<Url>,
    /// URLs for cloning the repository.
    #[arg(short, long)]
    clone:       Vec<Url>,
    /// Additional maintainers of the repository (besides yourself).
    #[arg(short, long)]
    maintainers: Vec<PublicKey>,
    /// Labels to categorize the repository.
    #[arg(short, long)]
    labels:      Vec<String>,
}

impl CommandRunner for AnnounceArgs {
    async fn run(self, options: CliOptions) -> N34Result<()> {
        let client = NostrClient::init(&options).await;
        let naddr = Nip19Coordinate::new(
            Coordinate::new(Kind::GitRepoAnnouncement, options.pubkey().await),
            options.relays.iter().take(3),
        )
        .expect("Valid relays");

        let mut maintainers = vec![naddr.public_key];
        maintainers.extend(self.maintainers);

        let event_builder = EventBuilder::new_git_repo(
            self.repo_id,
            self.name,
            self.description,
            self.web,
            self.clone,
            options.relays.clone(),
            maintainers,
            self.labels,
        )?;

        let result = client
            .send_builder_to(event_builder, &options.relays)
            .await?;

        for relay in &result.success {
            tracing::info!(relay = %relay, "Event sent successfully");
        }
        for (relay, reason) in &result.failed {
            tracing::warn!(relay = %relay, reason = %reason, "Failed to send event");
        }

        println!(
            "Event: {}",
            Nip19Event::new(result.val)
                .relays(options.relays.into_iter().take(3))
                .to_bech32()?
        );
        println!("Repo Address: {}", naddr.to_bech32()?);

        Ok(())
    }
}
