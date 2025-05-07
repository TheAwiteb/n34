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
use nostr::{
    event::EventBuilder,
    key::PublicKey,
    nips::nip65::{self, RelayMetadata},
    types::Url,
};

use crate::{
    cli::{CliOptions, CommandRunner},
    error::N34Result,
    nostr_utils::{NostrClient, traits::NewGitRepositoryAnnouncement, utils},
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
        let user_pubk = options.pubkey().await?;
        let relays_list = client.user_relays_list(user_pubk).await?;
        let mut write_relays = options.relays.clone();
        let mut maintainers = vec![user_pubk];
        maintainers.extend(self.maintainers);

        if let Some(event) = relays_list.clone() {
            write_relays.extend(
                nip65::extract_owned_relay_list(event)
                    .filter_map(|(r, m)| m.is_none_or(|m| m == RelayMetadata::Write).then_some(r)),
            );
        }

        let event = EventBuilder::new_git_repo(
            self.repo_id,
            self.name.map(utils::str_trim),
            self.description.map(utils::str_trim),
            self.web,
            self.clone,
            options.relays.clone(),
            maintainers,
            self.labels.into_iter().map(utils::str_trim).collect(),
        )?
        .pow(options.pow)
        .build(user_pubk);

        let nevent = utils::new_nevent(event.id.expect("There is an id"), &write_relays)?;
        let naddr = utils::repo_naddr(user_pubk, &options.relays)?;
        client
            .send_event_to(event, relays_list.as_ref(), &write_relays)
            .await?;

        println!("Event: {nevent}",);
        println!("Repo Address: {naddr}",);

        Ok(())
    }
}
