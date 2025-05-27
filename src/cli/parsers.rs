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

use std::{fs, path::Path};

use nostr::{
    Kind,
    nips::{
        self,
        nip01::Coordinate,
        nip19::{FromBech32, Nip19Coordinate},
    },
};
use tokio::runtime::Handle;

use crate::error::{N34Error, N34Result};

fn parse_nip5_repo(nip5: &str, repo_id: &str) -> Result<Nip19Coordinate, String> {
    let (username, domain) = nip5.split_once("@").unwrap_or(("_", nip5));

    let nip5_profile = tokio::task::block_in_place(|| {
        Handle::current().block_on(async {
            nips::nip05::profile(format!("{username}@{domain}"), None)
                .await
                .map_err(|err| err.to_string())
        })
    })?;

    Ok(Nip19Coordinate::new(
        Coordinate::new(Kind::GitRepoAnnouncement, nip5_profile.public_key).identifier(repo_id),
        nip5_profile.relays,
    )
    .expect("The relays is `RelayUrl`"))
}

fn parse_repo_naddr(repo_naddr: &str) -> Result<Nip19Coordinate, String> {
    let naddr = Nip19Coordinate::from_bech32(repo_naddr).map_err(|err| err.to_string())?;
    if naddr.relays.is_empty() {
        tracing::warn!("The repository naddr does not contain any relay hints");
    }

    (naddr.kind == Kind::GitRepoAnnouncement)
        .then_some(naddr)
        .ok_or_else(|| "Invalid naddr: must be of kind 30617 (GitRepoAnnouncement)".to_owned())
}

/// Parses a nostr-address file into a NIP-19 coordinates. Expects the file to
/// contain a repository announcements.
pub fn parse_nostr_address_file(file_path: &Path) -> N34Result<Vec<Nip19Coordinate>> {
    let addresses = fs::read_to_string(file_path)
        .map_err(N34Error::CanNotReadNostrAddressFile)?
        .split("\n")
        .filter_map(|line| {
            (!line.starts_with("#") && !line.trim().is_empty())
                .then_some(parse_repo_naddr(line).map_err(N34Error::InvalidNostrAddressFileContent))
        })
        .collect::<N34Result<Vec<Nip19Coordinate>>>()?;
    if addresses.is_empty() {
        return Err(N34Error::EmptyNostrAddressFile);
    }
    Ok(addresses)
}

/// Parses a Git repository address which can be either:
/// - A bech32-encoded naddr (e.g. "naddr1...") for Git repository announcements
///   (kind 30617)
/// - A NIP-05 identifier with repository ID (e.g. "4rs.nl/n34" or
///   "_@4rs.nl/n34")
///
/// Returns an error for invalid formats, failed bech32 decoding, wrong event
/// kind.
pub fn repo_naddr(repo: &str) -> Result<Nip19Coordinate, String> {
    let repo = repo.trim();

    if repo.contains("/") {
        let (nip5, repo_id) = repo.split_once("/").expect("There is a `/`");
        parse_nip5_repo(nip5, repo_id)
    } else if repo.starts_with("naddr1") {
        parse_repo_naddr(repo)
    } else {
        Err(
            "Invalid repository address format. It can be A NIP-05 identifier with repository ID \
             in format `<nip5>/<repo_id>` or A valid naddr1 string (NIP-19)"
                .to_owned(),
        )
    }
}
