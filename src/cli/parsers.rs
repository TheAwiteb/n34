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
    nips::nip19::{FromBech32, Nip19Coordinate},
};

/// Parses a Nostr naddr string into a Git repository announcement coordinate.
///
/// # Errors
/// Returns an error if:
/// - The bech32 decoding fails
/// - The naddr doesn't represent a Git repository announcement (kind != 30617)
pub fn repo_naddr(naddr: &str) -> Result<Nip19Coordinate, String> {
    let naddr = Nip19Coordinate::from_bech32(naddr).map_err(|err| err.to_string())?;
    if naddr.kind != Kind::GitRepoAnnouncement {
        return Err("The naddr is not repo announcement address".to_owned());
    }

    if naddr.relays.is_empty() {
        tracing::warn!("The repository naddr does not contain any relay hints");
    }

    Ok(naddr)
}
