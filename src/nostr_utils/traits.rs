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

use std::ops::Deref;

use convert_case::{Case, Casing};
use nostr::hashes::sha1::Hash as Sha1Hash;
use nostr::nips::nip19::FromBech32;
use nostr::{
    event::{Event, EventBuilder, EventId, Kind, Tag, TagKind, TagStandard, Tags},
    key::PublicKey,
    nips::{
        nip01::Coordinate,
        nip19::Nip19Coordinate,
        nip21::Nip21,
        nip34::{GitIssue, GitRepositoryAnnouncement},
    },
    parser::Token,
    types::{RelayUrl, Url},
};

use crate::cli::issue::ISSUE_ALT_PREFIX;
use crate::cli::patch::{
    LEGACY_NGIT_REVISION_ROOT_HASHTAG_CONTENT,
    REVISION_ROOT_HASHTAG_CONTENT,
    ROOT_HASHTAG_CONTENT,
};
use crate::error::{N34Error, N34Result};


/// A trait to add helper instance function to [`Tags`] type
#[easy_ext::ext(TagsExt)]
impl Tags {
    /// Search for the given tag and map it value to a function
    #[inline]
    pub fn map_tag<T>(&self, kind: TagKind, f: impl FnOnce(&TagStandard) -> T) -> Option<T> {
        self.find_standardized(kind).map(f)
    }

    /// Search for the given tag and map it value to a function. If the tag not
    /// found return the default `T`
    #[inline]
    pub fn dmap_tag<T>(&self, kind: TagKind, f: impl FnOnce(&TagStandard) -> T) -> T
    where
        T: Default,
    {
        self.map_tag(kind, f).unwrap_or_default()
    }

    /// Finds the first standard tag of the given kind with the specified
    /// marker, then applies the function to the tag and returns the result.
    #[inline]
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
        force_id: bool,
    ) -> N34Result<EventBuilder> {
        let repo_id = repo_id.trim();
        let kebab_repo_id = repo_id.to_case(Case::Kebab);
        if repo_id.is_empty() || (!force_id && repo_id != kebab_repo_id) {
            if repo_id != kebab_repo_id {
                tracing::error!(
                    "The repo id should be `{kebab_repo_id}` (kebab-case). Use `--force-id` to \
                     override this check"
                );
            }
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
            .dedup_tags()
            .tags(labels.into_iter().map(Tag::hashtag)),
        )
    }

    /// Creates a new [`GitIssue`] event builder with the given
    /// issue details.
    pub fn new_git_issue(
        coordinates: &[Coordinate],
        content: String,
        subject: Option<String>,
        labels: Vec<String>,
    ) -> N34Result<EventBuilder> {
        let mut coordinates = coordinates.iter();
        let first_coordinate = coordinates.next().ok_or(N34Error::EmptyNaddrs)?;

        let mut event_builder = EventBuilder::git_issue(GitIssue {
            repository: first_coordinate.clone(),
            content,
            subject: subject.clone(),
            labels: labels.into_iter().map(|l| l.trim().to_owned()).collect(),
        })
        .map_err(N34Error::from)?
        .tags(
            coordinates
                .clone()
                .map(|c| Tag::coordinate(c.clone(), None)),
        )
        .tags(coordinates.map(|c| Tag::public_key(c.public_key)));

        if let Some(issue_subject) = subject {
            event_builder =
                event_builder.tag(Tag::alt(format!("{ISSUE_ALT_PREFIX}{issue_subject}")))
        }

        Ok(event_builder)
    }
}

/// Helper functions for [`Token`] type
#[easy_ext::ext(TokenUtils)]
impl Token<'_> {
    /// Returns `Some((public_key, relays))` from the givin token if it's npub1
    /// or nprofile1
    #[inline]
    pub fn extract_public_key(&self) -> Option<(PublicKey, Vec<RelayUrl>)> {
        match self {
            Token::Nostr(nip21) => {
                match nip21 {
                    Nip21::Pubkey(pkey) => Some((*pkey, Vec::new())),
                    Nip21::Profile(profile) => Some((profile.public_key, profile.relays.clone())),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// Returns `Some((note_id, relays))` from the givin token if it's note1 or
    /// nevent1
    #[inline]
    pub fn extract_event_id(&self) -> Option<(EventId, Vec<RelayUrl>)> {
        match self {
            Token::Nostr(nip21) => {
                match nip21 {
                    Nip21::EventId(event_id) => Some((*event_id, Vec::new())),
                    Nip21::Event(event) => Some((event.event_id, event.relays.clone())),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// Returns `Some(hashtag)` from the givin token if it's a hashtag
    #[inline]
    pub fn extract_hashtag(&self) -> Option<String> {
        match self {
            Token::Hashtag(tag) => Some(tag.trim().to_owned()),
            _ => None,
        }
    }
}

/// Utility functions for working with lists of NIP-19 coordinates
#[easy_ext::ext(NaddrsUtils)]
impl Vec<Nip19Coordinate> {
    /// Converts these coordinate addresses to basic coordinates
    #[inline]
    pub fn into_coordinates(self) -> Vec<Coordinate> {
        self.into_iter().map(|n| n.coordinate).collect()
    }

    /// Returns all repository owners' public keys from these coordinates.
    #[inline]
    pub fn extract_owners(&self) -> Vec<PublicKey> {
        self.iter().map(|n| n.public_key).collect()
    }

    /// Extracts all relay URLs from these coordinates
    #[inline]
    pub fn extract_relays(&self) -> Vec<RelayUrl> {
        self.iter().flat_map(|n| n.relays.clone()).collect()
    }
}

/// Utility functions for working with lists of repository announcement
#[easy_ext::ext(ReposUtils)]
impl Vec<GitRepositoryAnnouncement> {
    /// Extracts all relay URLs from these repositories
    #[inline]
    pub fn extract_relays(&self) -> Vec<RelayUrl> {
        self.iter().flat_map(|n| n.relays.clone()).collect()
    }

    /// Extract the grasp servers.
    #[inline]
    pub fn extract_grasp_servers(&self) -> Vec<&Url> {
        self.iter()
            .flat_map(RepoUtils::extract_grasp_servers)
            .collect()
    }

    /// Extract all clones
    #[inline]
    pub fn extract_clones(&self) -> Vec<&Url> {
        self.iter().flat_map(|r| r.clone.deref()).collect()
    }

    /// Extract all the maintainers from these repositories
    #[inline]
    pub fn extract_maintainers(&self) -> Vec<PublicKey> {
        self.iter().flat_map(|r| r.maintainers.clone()).collect()
    }

    /// Gets the first EUC hash from the reposotoies if it exists.
    #[inline]
    pub fn extract_euc(&self) -> Option<&Sha1Hash> {
        self.iter().find_map(|r| r.euc.as_ref())
    }
}

#[easy_ext::ext(RepoUtils)]
impl GitRepositoryAnnouncement {
    /// Retrieves the GRASP servers from the list of clones based on the relay's
    /// GRASP check.
    pub fn extract_grasp_servers(&self) -> Vec<&Url> {
        self.clone
            .iter()
            .filter(|clone| self.relays.is_grasp(clone))
            .collect()
    }
}

#[easy_ext::ext(RelayUrlsUtils)]
impl Vec<RelayUrl> {
    /// Checks if the provided clone URL is a GRASP clone URL and is present in
    /// the relays.
    pub fn is_grasp(&self, clone_url: &Url) -> bool {
        clone_url.is_grasp_clone()
            && self.iter().any(|relay| {
                clone_url
                    .as_str()
                    .trim_start_matches("https://")
                    .trim_start_matches("http://")
                    .starts_with(
                        relay
                            .as_str_without_trailing_slash()
                            .trim_start_matches("wss://")
                            .trim_start_matches("ws://"),
                    )
            })
    }
}

#[easy_ext::ext(IsGraspClone)]
impl Url {
    /// Checks if the URL is a GRASP clone URL. A GRASP clone URL is identified
    /// by having a `<string>.git` suffix in the last part and a valid public
    /// key in the second last part.
    pub fn is_grasp_clone(&self) -> bool {
        let mut parts = self.as_str().split('/').rev().take(2);
        parts
            .next()
            .and_then(|r| (r.len() > 4 && r.ends_with(".git")).then_some(r))
            .is_some()
            && parts
                .next()
                .and_then(|p| PublicKey::from_bech32(p).ok())
                .is_some()
    }
}

/// Utility functions for working with patch events
#[easy_ext::ext(GitPatchUtils)]
impl Event {
    /// Returns whether the patch is a root or not
    #[inline]
    pub fn is_root_patch(&self) -> bool {
        self.kind == Kind::GitPatch
            && self
                .tags
                .filter(TagKind::t())
                .any(|t| t.content() == Some(ROOT_HASHTAG_CONTENT))
    }

    /// Returns whether the patch is patch-revision or not
    #[inline]
    pub fn is_revision_patch(&self) -> bool {
        self.kind == Kind::GitPatch
            && self.tags.filter(TagKind::t()).any(|t| {
                [
                    Some(REVISION_ROOT_HASHTAG_CONTENT),
                    Some(LEGACY_NGIT_REVISION_ROOT_HASHTAG_CONTENT),
                ]
                .contains(&t.content())
            })
    }

    /// Gets the root patch ID from a patch-revision event by finding the `e`
    /// tag that replies to it. Fails if no such tag is found or if the tag
    /// contains an invalid event ID.
    pub fn root_patch_from_revision(&self) -> N34Result<EventId> {
        self.tags
            .iter()
            .find(|tag| tag.is_reply())
            .ok_or_else(|| {
                N34Error::InvalidEvent(
                    "A patch revision without `e`-reply to the root patch".to_owned(),
                )
            })?
            .content()
            .ok_or_else(|| N34Error::InvalidEvent("`e` tag without an event".to_owned()))?
            .parse()
            .map_err(|err| N34Error::InvalidEvent(format!("Invalid event ID in `e` tag: {err}")))
    }
}

/// Utility functions for working with issue and PR metadata
#[easy_ext::ext(GitIssuePrMetadata)]
impl Event {
    /// Gets the subject line of the issue/pr or "N/A" if none exists
    #[inline]
    pub fn extract_event_subject(&self) -> &str {
        self.tags
            .find(TagKind::Subject)
            .and_then(|t| t.content())
            .unwrap_or("N/A")
    }

    /// Gets all issue/pr labels formatted as comma-separated hashtags (e.g.
    /// "#bug, #feature")
    #[inline]
    pub fn extract_event_labels(&self) -> String {
        self.tags
            .filter(TagKind::t())
            .filter_map(|t| t.content().map(|l| format!("#{l}")))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[easy_ext::ext(KindExt)]
impl Kind {
    /// Checks if the kind is a root kind, such as GitIssue, GitPatch, GitPr or
    /// GitPrUpdate.
    pub fn is_root_kind(&self) -> bool {
        match self {
            val if val == &crate::cli::pr::PR_KIND || val == &crate::cli::pr::PR_UPDATE_KIND => {
                true
            }
            Kind::GitIssue | Kind::GitPatch => true,
            _ => false,
        }
    }

    /// Returns true if this kind supports replies, such as Comment, GitIssue,
    /// GitPatch, GitPr, or GitPrUpdate.
    pub fn can_reply_to(&self) -> bool {
        self == &Kind::Comment || self.is_root_kind()
    }
}
