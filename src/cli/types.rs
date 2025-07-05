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

use nostr::{
    event::{EventId, Kind},
    nips::{
        self,
        nip01::Coordinate,
        nip19::{self, FromBech32, Nip19Coordinate},
    },
    types::RelayUrl,
};
use tokio::runtime::Handle;

use super::{RepoRelaySetsExt, parsers};
use crate::{
    cli::RepoRelaySet,
    error::{N34Error, N34Result},
};

/// Either a NIP-19 coordinate (naddr) or a named set.
#[derive(Debug, Clone)]
pub enum NaddrOrSet {
    /// NIP-19 coordinate.
    Naddr(Nip19Coordinate),
    /// Name of a set (may not exist).
    Set(String),
}

/// Either relay URL or a named set.
#[derive(Debug, Clone)]
pub enum RelayOrSet {
    /// Relay URL.
    Relay(RelayUrl),
    /// Name of a set (may not exist).
    Set(String),
}

/// Parses and represents a Nostr `nevent1` or `note1`.
#[derive(Debug, Clone)]
pub struct NostrEvent {
    /// Unique identifier for the event.
    pub event_id: EventId,
    /// List of relay URLs associated with the event. Empty if parsing a
    /// `note1`.
    pub relays:   Vec<RelayUrl>,
}

impl NaddrOrSet {
    /// Returns the naddr if `Naddr` or try to get the relays from the set.
    /// Returns error if the set naddrs are empty or the set not found.
    pub fn get_naddrs(self, sets: &[RepoRelaySet]) -> N34Result<Vec<Nip19Coordinate>> {
        match self {
            Self::Naddr(nip19_coordinate) => Ok(vec![nip19_coordinate]),
            Self::Set(name) => {
                let set = sets
                    .get_set(&name)
                    .map_err(|_| N34Error::InvalidNaddrArg(name.clone()))?;
                if set.naddrs.is_empty() {
                    Err(N34Error::EmptySetNaddrs(name))
                } else {
                    Ok(Vec::from_iter(set.naddrs.clone()))
                }
            }
        }
    }
}


impl RelayOrSet {
    /// Returns the relay if `Relay` or try to get the relays from the set.
    /// Returns error if the set relays are empty or the set not found
    pub fn get_relays(self, sets: &[RepoRelaySet]) -> N34Result<Vec<RelayUrl>> {
        match self {
            Self::Relay(relay) => Ok(vec![relay]),
            Self::Set(name) => {
                let set = sets
                    .get_set(&name)
                    .map_err(|_| N34Error::InvalidRelaysArg(name.clone()))?;
                if set.relays.is_empty() {
                    Err(N34Error::EmptySetRelays(name))
                } else {
                    Ok(Vec::from_iter(set.relays.clone()))
                }
            }
        }
    }
}

impl NostrEvent {
    /// Create a new [`NostrEvent`] instance
    fn new(event_id: EventId, relays: Vec<RelayUrl>) -> Self {
        Self { event_id, relays }
    }
}

impl FromStr for NaddrOrSet {
    type Err = String;

    /// Parses a Git repository address which can be either:
    /// - A bech32-encoded naddr (e.g. "naddr1...") for Git repository
    ///   announcements (kind 30617)
    /// - A NIP-05 identifier with repository ID (e.g. "4rs.nl/n34" or
    ///   "_@4rs.nl/n34")
    /// - A set name.
    ///
    /// Returns an error for invalid formats, failed bech32 decoding, wrong
    /// event kind.
    fn from_str(naddr_or_set: &str) -> Result<Self, Self::Err> {
        let naddr_or_set = naddr_or_set.trim();

        if naddr_or_set.contains("/") {
            let (nip5, repo_id) = naddr_or_set.split_once("/").expect("There is a `/`");
            parse_nip5_repo(nip5, repo_id)
        } else if naddr_or_set.starts_with("naddr1") || naddr_or_set.starts_with("nostr:naddr1") {
            parsers::parse_repo_naddr(naddr_or_set.trim_start_matches("nostr:")).map(Self::Naddr)
        } else {
            Ok(Self::Set(naddr_or_set.to_owned()))
        }
    }
}

impl FromStr for RelayOrSet {
    type Err = String;

    /// Parse a string into a relay URL or a set name.
    /// If the string is a valid URL (e.g., "wss://example.com"), it's treated
    /// as a relay URL. Otherwise, it's treated as a set name, and its
    /// associated relays will be merged.
    fn from_str(relay_or_set: &str) -> Result<Self, Self::Err> {
        let relay_or_set = relay_or_set.trim();

        if relay_or_set.starts_with("wss://") {
            RelayUrl::from_str(relay_or_set)
                .map_err(|err| err.to_string())
                .map(Self::Relay)
        } else {
            Ok(Self::Set(relay_or_set.to_owned()))
        }
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

#[easy_ext::ext(NaddrOrSetVecExt)]
impl Vec<NaddrOrSet> {
    /// Converts this vector of [`NaddrOrSet`] into a flat vector of
    /// [`Nip19Coordinate`] using the given sets.
    pub fn flat_naddrs(self, sets: &[RepoRelaySet]) -> N34Result<Vec<Nip19Coordinate>> {
        self.into_iter()
            .map(|n| n.get_naddrs(sets))
            .try_fold(Vec::new(), |mut acc, item| {
                acc.extend(item?);
                Ok(acc)
            })
    }
}

#[easy_ext::ext(RelayOrSetVecExt)]
impl Vec<RelayOrSet> {
    /// Converts this vector of [`RelayOrSet`] into a flat vector of
    /// [`RelayUrl`] using the given sets.
    pub fn flat_relays(self, sets: &[RepoRelaySet]) -> N34Result<Vec<RelayUrl>> {
        self.into_iter()
            .map(|n| n.get_relays(sets))
            .try_fold(Vec::new(), |mut acc, item| {
                acc.extend(item?);
                Ok(acc)
            })
    }
}


#[easy_ext::ext(OptionNaddrOrSetVecExt)]
impl Option<Vec<NaddrOrSet>> {
    /// Converts this vector of [`NaddrOrSet`] into a flat vector of
    /// [`Nip19Coordinate`] using the given sets.
    pub fn flat_naddrs(&self, sets: &[RepoRelaySet]) -> N34Result<Option<Vec<Nip19Coordinate>>> {
        // Clones self here to simplify command code
        self.clone()
            .map(|naddrs| naddrs.flat_naddrs(sets))
            .transpose()
    }
}

fn parse_nip5_repo(nip5: &str, repo_id: &str) -> Result<NaddrOrSet, String> {
    let (username, domain) = nip5.split_once("@").unwrap_or(("_", nip5));

    let nip5_profile = tokio::task::block_in_place(|| {
        Handle::current().block_on(async {
            nips::nip05::profile(format!("{username}@{domain}"), None)
                .await
                .map_err(|err| err.to_string())
        })
    })?;

    Ok(NaddrOrSet::Naddr(
        Nip19Coordinate::new(
            Coordinate::new(Kind::GitRepoAnnouncement, nip5_profile.public_key).identifier(repo_id),
            nip5_profile.relays,
        )
        .expect("The relays is `RelayUrl`"),
    ))
}
