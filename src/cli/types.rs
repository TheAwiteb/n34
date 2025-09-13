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

use std::{fmt, str::FromStr};

use nostr::{
    event::{EventId, Kind},
    nips::{
        nip01::Coordinate,
        nip05::{Nip05Address, Nip05Profile},
        nip19::{self, FromBech32, Nip19Coordinate},
    },
    types::RelayUrl,
    util::BoxedFuture,
};
use nostr_connect::client::AuthUrlHandler;
use tokio::runtime::Handle;

use super::parsers;
use crate::{
    cli::{RepoRelaySet, traits::RepoRelaySetsExt},
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

/// Enum representing the type of entity to handle in common commands.
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum EntityType {
    /// Pull Request
    PullRequest,
    /// Patch
    Patch,
    /// Issue
    Issue,
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

#[derive(Debug)]
pub struct EchoAuthUrl;

impl AuthUrlHandler for EchoAuthUrl {
    fn on_auth_url(
        &self,
        auth_url: nostr::Url,
    ) -> BoxedFuture<'_, Result<(), Box<dyn std::error::Error>>> {
        Box::pin(async move {
            println!("The bunker requires authentication. Please open this URL: {auth_url}");
            Ok(())
        })
    }
}

impl EntityType {
    /// Returns true if the entity is a pull request.
    #[inline]
    pub const fn is_pr(&self) -> bool {
        matches!(self, Self::PullRequest)
    }

    /// Returns true if the entity is a patch.
    #[inline]
    pub const fn is_patch(&self) -> bool {
        matches!(self, Self::Patch)
    }

    /// Returns true if the entity is an issue.
    #[inline]
    pub const fn is_issue(&self) -> bool {
        matches!(self, Self::Issue)
    }

    /// Returns the kind of the entity
    #[inline]
    pub const fn kind(&self) -> Kind {
        match self {
            Self::PullRequest => crate::cli::pr::PR_KIND,
            Self::Patch => Kind::GitPatch,
            Self::Issue => Kind::GitIssue,
        }
    }

    /// Converts a [`u8`] value to the corresponding enum variant.
    #[inline]
    pub const fn from_u8<const NUM: u8>() -> Self {
        const {
            match NUM {
                val if val == Self::PullRequest as u8 => Self::PullRequest,
                val if val == Self::Patch as u8 => Self::Patch,
                val if val == Self::Issue as u8 => Self::Issue,
                _ => {
                    panic!("No enum with the given numeric value")
                }
            }
        }
    }

    /// Ensures the entity is either a pull request or a patch. Compilation
    /// will fail if the entity is neither.
    pub const fn is_pr_or_patch<const ENTITY_TYPE: u8>() {
        const {
            let entity = EntityType::from_u8::<ENTITY_TYPE>();
            if !entity.is_pr() && !entity.is_patch() {
                panic!("The entity should be a pull request or a patch")
            }
        }
    }
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

#[derive(Debug)]
pub enum PatchPrStatus {
    /// The patch/pr is currently open
    Open,
    /// The patch/pr has been merged/applied
    MergedApplied,
    /// The patch/pr has been closed
    Closed,
    /// A patch/pr that has been drafted but not yet applied.
    Draft,
}

impl PatchPrStatus {
    /// Returns all status kinds
    #[inline]
    pub const fn all_kinds() -> [Kind; 4] {
        [
            Self::Open.kind(),
            Self::MergedApplied.kind(),
            Self::Closed.kind(),
            Self::Draft.kind(),
        ]
    }

    /// Maps the patch/pr status to its corresponding Nostr kind.
    #[inline]
    pub const fn kind(&self) -> Kind {
        match self {
            Self::Open => Kind::GitStatusOpen,
            Self::MergedApplied => Kind::GitStatusApplied,
            Self::Closed => Kind::GitStatusClosed,
            Self::Draft => Kind::GitStatusDraft,
        }
    }

    /// Returns the string representation of the patch/pr status.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::MergedApplied => "Merged/Applied",
            Self::Closed => "Closed",
            Self::Draft => "Draft",
        }
    }

    /// Check if the status is open.
    #[inline]
    pub fn is_open(&self) -> bool {
        matches!(self, Self::Open)
    }

    /// Check if the status is merged/applied.
    #[inline]
    pub fn is_merged_or_applied(&self) -> bool {
        matches!(self, Self::MergedApplied)
    }

    /// Check if the status is closed.
    #[inline]
    pub fn is_closed(&self) -> bool {
        matches!(self, Self::Closed)
    }

    /// Check if the status is draft
    #[inline]
    pub fn is_drafted(&self) -> bool {
        matches!(self, Self::Draft)
    }
}

impl From<&PatchPrStatus> for Kind {
    fn from(status: &PatchPrStatus) -> Self {
        status.kind()
    }
}

impl fmt::Display for PatchPrStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}


impl TryFrom<Kind> for PatchPrStatus {
    type Error = N34Error;

    fn try_from(kind: Kind) -> Result<Self, Self::Error> {
        match kind {
            Kind::GitStatusOpen => Ok(Self::Open),
            Kind::GitStatusApplied => Ok(Self::MergedApplied),
            Kind::GitStatusClosed => Ok(Self::Closed),
            Kind::GitStatusDraft => Ok(Self::Draft),
            _ => Err(N34Error::InvalidPatchStatus(kind)),
        }
    }
}

fn parse_nip5_repo(nip5: &str, repo_id: &str) -> Result<NaddrOrSet, String> {
    let (username, domain) = nip5.split_once("@").unwrap_or(("_", nip5));

    let nip5_address =
        Nip05Address::parse(&format!("{username}@{domain}")).map_err(|err| err.to_string())?;

    let nip5_json = tokio::task::block_in_place(|| {
        Handle::current().block_on(async {
            reqwest::get(nip5_address.url().as_str())
                .await
                .map_err(|err| err.to_string())?
                .text()
                .await
                .map_err(|err| err.to_string())
        })
    })?;

    let nip5_profile =
        Nip05Profile::from_raw_json(&nip5_address, &nip5_json).map_err(|err| err.to_string())?;

    Ok(NaddrOrSet::Naddr(Nip19Coordinate::new(
        Coordinate::new(Kind::GitRepoAnnouncement, nip5_profile.public_key).identifier(repo_id),
        nip5_profile.relays,
    )))
}
