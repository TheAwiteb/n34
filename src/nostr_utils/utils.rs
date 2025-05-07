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
    event::{Event, EventId, Kind, TagKind, TagStandard},
    filter::{Alphabet, SingleLetterTag},
    key::PublicKey,
    nips::{
        nip01::Coordinate,
        nip19::{Nip19Coordinate, Nip19Event, ToBech32},
        nip34::GitRepositoryAnnouncement,
    },
    types::RelayUrl,
};

use super::traits::TagsExt;
use crate::error::{N34Error, N34Result};

/// Returns the value of the given tag
fn tag_value(tag: &TagStandard) -> String {
    tag.clone().to_vec().remove(1)
}

/// Parses the tag value into type `T` if possible.
fn parse_value<T: FromStr>(tag: &TagStandard) -> Option<T> {
    tag_value(tag).parse().ok()
}

/// Gets all values from the tag. If any value fails to parse, returns an empty
/// vector.
fn tag_values<T>(tag: &TagStandard) -> Vec<T>
where
    T: FromStr + fmt::Debug,
    <T as FromStr>::Err: fmt::Debug,
{
    tag.clone()
        .to_vec()
        .into_iter()
        .skip(1)
        .map(|t| {
            let result = T::from_str(t.as_str());
            tracing::trace!("Parsing `{t}` result: `{result:?}`");
            result
        })
        .collect::<Result<_, _>>()
        .unwrap_or_default()
}

/// Convert [`Event`] to [`GitRepositoryAnnouncement`]
pub fn event_into_repo(event: Event, repo_id: impl Into<String>) -> GitRepositoryAnnouncement {
    let tags = &event.tags;

    GitRepositoryAnnouncement {
        id:          repo_id.into(),
        name:        tags.map_tag(TagKind::Name, tag_value),
        description: tags.map_tag(TagKind::Description, tag_value),
        euc:         tags
            .map_marker(
                TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::R)),
                "euc",
                parse_value,
            )
            .flatten(),
        web:         tags.dmap_tag(TagKind::Web, tag_values),
        clone:       tags.dmap_tag(TagKind::Clone, tag_values),
        relays:      tags.dmap_tag(TagKind::Relays, tag_values),
        maintainers: tags.dmap_tag(TagKind::Maintainers, tag_values),
    }
}

/// Returns a new string with leading and trailing whitespace removed.
pub fn str_trim(s: String) -> String {
    s.trim().to_owned()
}

/// Returns a vector with duplicate elements removed.
pub fn dedup<I, T>(iter: I) -> Vec<T>
where
    T: std::cmp::Ord,
    I: Iterator<Item = T>,
{
    let mut vector: Vec<T> = iter.collect();
    vector.sort_unstable();
    vector.dedup();
    vector
}

/// Creates a new NIP-19 nevent string from an event ID and up to 3 unique relay
/// URLs.
pub fn new_nevent(event_id: EventId, relays: &[RelayUrl]) -> N34Result<String> {
    Nip19Event::new(event_id)
        .relays(dedup(relays.iter().take(3).cloned()))
        .to_bech32()
        .map_err(N34Error::from)
}

/// Creates a NIP-19 naddr string for a git repository announcement and up to 3
/// unique relay URLs.
pub fn repo_naddr(pubk: PublicKey, relays: &[RelayUrl]) -> N34Result<String> {
    Nip19Coordinate::new(
        Coordinate::new(Kind::GitRepoAnnouncement, pubk),
        dedup(relays.iter().take(3)),
    )
    .expect("Valid relays")
    .to_bech32()
    .map_err(N34Error::from)
}
