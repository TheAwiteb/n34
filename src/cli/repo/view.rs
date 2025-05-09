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

use std::fmt;

use clap::Args;
use nostr::nips::nip19::Nip19Coordinate;

use crate::{
    cli::{CliOptions, CommandRunner, parsers},
    error::N34Result,
    nostr_utils::NostrClient,
};

/// Arguments for the `repo view` command
#[derive(Args, Debug)]
pub struct ViewArgs {
    /// Nostr repository address
    #[arg(short, long, value_parser = parsers::repo_naddr)]
    naddr: Nip19Coordinate,
}

impl CommandRunner for ViewArgs {
    async fn run(self, options: CliOptions) -> N34Result<()> {
        let client = NostrClient::init(&options).await;
        if !self.naddr.relays.is_empty() {
            client.add_relays(&self.naddr.relays).await;
        }

        let repo = client.fetch_repo(&self.naddr.coordinate).await?;
        let mut msg = format!("ID: {}", repo.id);

        if let Some(name) = repo.name {
            msg.push_str(&format!("\nName: {name}"));
        }
        if let Some(desc) = repo.description {
            msg.push_str(&format!("\nDescription: {desc}"));
        }
        if !repo.web.is_empty() {
            msg.push_str(&format!("\nWebpages:\n{}", format_list(repo.web)));
        }
        if !repo.clone.is_empty() {
            msg.push_str(&format!("\nClone urls:\n{}", format_list(repo.clone)));
        }
        if !repo.relays.is_empty() {
            msg.push_str(&format!("\nRelays:\n{}", format_list(repo.relays)));
        }
        if let Some(euc) = repo.euc {
            msg.push_str(&format!("\nEarliest unique commit: {euc}"));
        }
        if !repo.maintainers.is_empty() {
            msg.push_str(&format!(
                "\nMaintainers:\n{}",
                format_list(repo.maintainers)
            ));
        }

        println!("{msg}");
        Ok(())
    }
}

/// Format a vector to print it
fn format_list<T>(vector: Vec<T>) -> String
where
    T: fmt::Display,
{
    vector
        .into_iter()
        .map(|t| format!(" - {t}"))
        .collect::<Vec<String>>()
        .join("\n")
}
