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

use nostr::{
    Kind,
    nips::{
        self,
        nip01::Coordinate,
        nip19::{FromBech32, Nip19Coordinate},
    },
};
use tokio::runtime::Handle;

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

/// Parses a Git repository address which can be either:
/// - A bech32-encoded naddr (e.g. "naddr1...") for Git repository announcements
///   (kind 30617)
/// - A NIP-05 identifier with repository ID (e.g. "4rs.nl/n34" or
///   "_@4rs.nl/n34")
///
/// Returns an error for invalid formats, failed bech32 decoding, wrong event
/// kind.
pub fn repo_naddr(repo_address: &str) -> Result<Nip19Coordinate, String> {
    if repo_address.contains("/") {
        let (nip5, repo_id) = repo_address.split_once("/").expect("There is a `/`");
        return parse_nip5_repo(nip5, repo_id);
    }

    let naddr = Nip19Coordinate::from_bech32(repo_address).map_err(|err| err.to_string())?;
    if naddr.kind != Kind::GitRepoAnnouncement {
        return Err("The naddr is not repo announcement address".to_owned());
    }

    if naddr.relays.is_empty() {
        tracing::warn!("The repository naddr does not contain any relay hints");
    }

    Ok(naddr)
}
