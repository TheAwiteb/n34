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

use convert_case::{Case, Casing};
use nostr::{
    event::{EventBuilder, Tag, TagKind, TagStandard, Tags},
    key::PublicKey,
    nips::nip34::GitRepositoryAnnouncement,
    types::{RelayUrl, Url},
};

use crate::error::{N34Error, N34Result};


/// A trait to add helper instance function to [`Tags`] type
#[easy_ext::ext(TagsExt)]
impl Tags {
    /// Search for the given tag and map it value to a function
    pub fn map_tag<T>(&self, kind: TagKind, f: impl FnOnce(&TagStandard) -> T) -> Option<T> {
        self.find_standardized(kind).map(f)
    }

    /// Search for the given tag and map it value to a function. If the tag not
    /// found return the default `T`
    pub fn dmap_tag<T>(&self, kind: TagKind, f: impl FnOnce(&TagStandard) -> T) -> T
    where
        T: Default,
    {
        self.map_tag(kind, f).unwrap_or_default()
    }

    /// Finds the first standard tag of the given kind with the specified
    /// marker, then applies the function to the tag and returns the result.
    pub fn map_marker<T>(
        &self,
        kind: TagKind,
        marker: &str,
        f: impl FnOnce(&TagStandard) -> T,
    ) -> Option<T> {
        self.filter_standardized(kind)
            .find(|t| (*t).clone().to_vec().last().is_some_and(|m| m == marker))
            .map(f)
    }
}

/// Trait for building [`GitRepositoryAnnouncement`] events
#[easy_ext::ext(NewGitRepositoryAnnouncement)]
impl EventBuilder {
    /// Creates a new [`GitRepositoryAnnouncement`] event builder with the given
    /// repository details.
    #[allow(clippy::too_many_arguments)]
    pub fn new_git_repo(
        repo_id: String,
        name: Option<String>,
        description: Option<String>,
        web: Vec<Url>,
        clone: Vec<Url>,
        relays: Vec<RelayUrl>,
        maintainers: Vec<PublicKey>,
        labels: Vec<String>,
    ) -> N34Result<EventBuilder> {
        let repo_id = repo_id.trim();
        if repo_id.is_empty() || repo_id != repo_id.to_case(Case::Kebab) {
            return Err(N34Error::InvalidRepoId);
        }

        Ok(
            EventBuilder::git_repository_announcement(GitRepositoryAnnouncement {
                id: repo_id.to_owned(),
                name,
                description,
                web,
                clone,
                relays,
                euc: None,
                maintainers,
            })?
            .tags(labels.into_iter().map(Tag::hashtag)),
        )
    }
}
