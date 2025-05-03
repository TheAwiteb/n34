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
    event::{Event, TagKind, TagStandard},
    nips::nip34::GitRepositoryAnnouncement,
};

/// Returns the value of the given tag
fn tag_value(tag: &TagStandard) -> String {
    tag.clone().to_vec().remove(1)
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
    GitRepositoryAnnouncement {
        id:          repo_id.into(),
        name:        event.tags.find_standardized(TagKind::Name).map(tag_value),
        description: event
            .tags
            .find_standardized(TagKind::Description)
            .map(tag_value),
        web:         event
            .tags
            .find_standardized(TagKind::Web)
            .map(tag_values)
            .unwrap_or_default(),
        clone:       event
            .tags
            .find_standardized(TagKind::Clone)
            .map(tag_values)
            .unwrap_or_default(),
        relays:      event
            .tags
            .find_standardized(TagKind::Relays)
            .map(tag_values)
            .unwrap_or_default(),
        euc:         None,
        maintainers: event
            .tags
            .find_standardized(TagKind::Maintainers)
            .map(tag_values)
            .unwrap_or_default(),
    }
}
