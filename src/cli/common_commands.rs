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

use std::{iter, str::FromStr, sync::Arc};

use either::Either;
use futures::future;
use nostr::{
    event::{Event, EventBuilder, EventId, Kind, Tag, TagKind, TagStandard},
    filter::{Alphabet, Filter, SingleLetterTag},
    hashes::sha1::Hash as Sha1Hash,
    nips::{nip10::Marker, nip19::ToBech32},
    types::RelayUrl,
};

use super::{
    issue::IssueStatus,
    types::{NaddrOrSet, NostrEvent},
};
use crate::{
    cli::{CliOptions, patch::GitPatch, types::PatchPrStatus},
    error::{N34Error, N34Result},
    nostr_utils::traits::{GitIssuePrMetadata, GitPatchUtils, ReposUtils},
};
use crate::{
    cli::{
        traits::{OptionNaddrOrSetVecExt, RelayOrSetVecExt},
        types::EntityType,
    },
    nostr_utils::{NostrClient, traits::NaddrsUtils, utils},
};

/// Updates the issue's status to `new_status` after validating it with
/// `check_fn`.
pub async fn issue_status_command(
    options: CliOptions,
    issue_id: NostrEvent,
    naddrs: Option<Vec<NaddrOrSet>>,
    new_status: IssueStatus,
    check_fn: impl FnOnce(&IssueStatus) -> N34Result<()>,
) -> N34Result<()> {
    let naddrs = utils::naddrs_or_file(
        naddrs.flat_naddrs(&options.config.sets)?,
        &utils::nostr_address_path()?,
    )?;
    let relays = options.relays.clone().flat_relays(&options.config.sets)?;
    let client = NostrClient::init(&options, &relays).await;
    let user_pubk = client.pubkey().await?;
    client
        .add_relays(&[naddrs.extract_relays(), issue_id.relays].concat())
        .await;

    let owners = naddrs.extract_owners();
    let coordinates = naddrs.clone().into_coordinates();
    let repos = client.fetch_repos(&coordinates).await?;
    let maintainers = repos.extract_maintainers();
    let relay_hint = repos.extract_relays().first().cloned();
    client.add_relays(&repos.extract_relays()).await;

    let issue_event = client
        .fetch_event(Filter::new().id(issue_id.event_id))
        .await?
        .ok_or(N34Error::CanNotFoundIssue)?;

    let issue_status = client
        .fetch_issue_status(
            issue_id.event_id,
            [maintainers.as_slice(), &[issue_event.pubkey], &owners].concat(),
        )
        .await?;

    check_fn(&issue_status)?;

    let status_event = EventBuilder::new(new_status.kind(), "")
        .pow(options.pow.unwrap_or_default())
        .tag(utils::event_reply_tag(
            &issue_id.event_id,
            relay_hint.as_ref(),
            Marker::Root,
        ))
        .tag(Tag::public_key(issue_event.pubkey))
        .tags(maintainers.iter().map(|p| Tag::public_key(*p)))
        .tags(owners.iter().map(|p| Tag::public_key(*p)))
        .tags(
            coordinates
                .into_iter()
                .map(|c| Tag::coordinate(c, relay_hint.clone())),
        )
        .dedup_tags()
        .build(user_pubk);

    let event_id = status_event.id.expect("There is an id");
    let user_relays_list = client.user_relays_list(user_pubk).await?;
    let write_relays = [
        relays,
        naddrs.extract_relays(),
        repos.extract_relays(),
        utils::add_write_relays(user_relays_list.as_ref()),
        client.read_relays_from_user(issue_event.pubkey).await,
        client
            .read_relays_from_users(&[maintainers, owners].concat())
            .await,
    ]
    .concat();

    let success = client
        .send_event_to(status_event, user_relays_list.as_ref(), &write_relays)
        .await?;
    let nevent = utils::new_nevent(event_id, &success)?;
    println!("Issue status created: {nevent}");

    Ok(())
}

/// Updates the patch/pr's status to `new_status` after validating it with
/// `check_fn`. The `ENTITY_TYPE` can only be a pull request or a patch
pub async fn patch_pr_status_command<const ENTITY_TYPE: u8>(
    options: CliOptions,
    patch_pr_id: NostrEvent,
    naddrs: Option<Vec<NaddrOrSet>>,
    new_status: PatchPrStatus,
    merge_or_applied_commits: Option<Either<Sha1Hash, Vec<Sha1Hash>>>,
    merge_or_applied_patches: Vec<EventId>,
    check_fn: impl FnOnce(&PatchPrStatus) -> N34Result<()>,
) -> N34Result<()> {
    EntityType::is_pr_or_patch::<ENTITY_TYPE>();
    let entity_type = EntityType::from_u8::<ENTITY_TYPE>();

    let naddrs = utils::naddrs_or_file(
        naddrs.flat_naddrs(&options.config.sets)?,
        &utils::nostr_address_path()?,
    )?;
    let relays = options.relays.clone().flat_relays(&options.config.sets)?;
    let client = NostrClient::init(&options, &relays).await;
    let user_pubk = client.pubkey().await?;
    client
        .add_relays(&[naddrs.extract_relays(), patch_pr_id.relays].concat())
        .await;

    let owners = naddrs.extract_owners();
    let coordinates = naddrs.clone().into_coordinates();
    let repos = client.fetch_repos(&coordinates).await?;
    let maintainers = repos.extract_maintainers();
    let relay_hint = repos.extract_relays().first().cloned();
    client.add_relays(&repos.extract_relays()).await;

    let event = if entity_type.is_patch() {
        client.fetch_patch(patch_pr_id.event_id).await?
    } else {
        client.fetch_pr(patch_pr_id.event_id).await?
    };
    let authorized_pubkeys = [maintainers.as_slice(), &[event.pubkey], &owners].concat();

    if entity_type.is_patch() && event.is_revision_patch() && !new_status.is_merged_or_applied() {
        return Err(N34Error::InvalidStatus(
            "Invalid action for patch revision. Only 'apply' or 'merge' are allowed, 'open', \
             'close', and 'draft' are not supported."
                .to_owned(),
        ));
    }

    let (root_patch_or_pr, root_revision) = if entity_type.is_patch() {
        get_patch_root_revision(&event)?
    } else {
        (event.id, None)
    };
    let current_status = if entity_type.is_patch() {
        client
            .fetch_patch_status(root_patch_or_pr, root_revision, authorized_pubkeys.clone())
            .await?
    } else {
        client
            .fetch_pr_status(event.id, authorized_pubkeys.clone())
            .await?
    };

    check_fn(&current_status)?;

    let mut status_builder = EventBuilder::new(new_status.kind(), "")
        .pow(options.pow.unwrap_or_default())
        .tag(utils::event_reply_tag(
            &root_patch_or_pr,
            relay_hint.as_ref(),
            Marker::Root,
        ))
        .tags(authorized_pubkeys.iter().map(|p| Tag::public_key(*p)))
        .tags(
            coordinates
                .into_iter()
                .map(|c| Tag::coordinate(c, relay_hint.clone())),
        );

    if new_status.is_merged_or_applied() {
        if let Some(merge_commit) = merge_or_applied_commits
            .as_ref()
            .and_then(|e| e.as_ref().left())
        {
            let commit = merge_commit.to_string();
            status_builder = status_builder
                .tag(Tag::custom(
                    TagKind::custom("merge-commit"),
                    iter::once(&commit),
                ))
                .tag(Tag::reference(commit));
        } else if let Some(applied_commits) = merge_or_applied_commits.and_then(|e| e.right()) {
            let commits = applied_commits
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>();
            status_builder = status_builder
                .tag(Tag::custom(TagKind::custom("applied-as-commits"), &commits))
                .tags(commits.into_iter().map(Tag::reference));
        };

        // Patch only
        if let Some(root_revision) = root_revision {
            status_builder = status_builder.tag(utils::event_reply_tag(
                &root_revision,
                relay_hint.as_ref(),
                Marker::Reply,
            ));
        }

        // Patch only
        if !merge_or_applied_patches.is_empty() {
            status_builder = status_builder.tags(
                build_patches_quote(client.clone(), relay_hint.clone(), merge_or_applied_patches)
                    .await,
            );
        }
    }

    let status_event = status_builder.dedup_tags().build(user_pubk);
    let event_id = status_event.id.expect("There is an id");
    let user_relays_list = client.user_relays_list(user_pubk).await?;
    let write_relays = [
        relays,
        naddrs.extract_relays(),
        repos.extract_relays(),
        utils::add_write_relays(user_relays_list.as_ref()),
        client.read_relays_from_users(&authorized_pubkeys).await,
    ]
    .concat();

    let success = client
        .send_event_to(status_event, user_relays_list.as_ref(), &write_relays)
        .await?;
    let nevent = utils::new_nevent(event_id, &success)?;
    println!("Patch status created: {nevent}");

    Ok(())
}

/// Fetches and displays pull requests, patches, and issues for specified
/// repositories. The `limit` parameter sets the maximum number of items to
/// retrieve.
///
/// The `ENTITY_TYPE` const is `[EntityType]` enum as u8.
pub async fn list_pr_patches_and_issues<const ENTITY_TYPE: u8>(
    options: CliOptions,
    naddrs: Option<Vec<NaddrOrSet>>,
    limit: usize,
) -> N34Result<()> {
    let naddrs = utils::check_empty_naddrs(utils::naddrs_or_file(
        naddrs.flat_naddrs(&options.config.sets)?,
        &utils::nostr_address_path()?,
    )?)?;

    let entity_type = EntityType::from_u8::<ENTITY_TYPE>();
    let relays = options.relays.clone().flat_relays(&options.config.sets)?;
    let client = NostrClient::init(&options, &relays).await;
    client.add_relays(&naddrs.extract_relays()).await;

    let coordinates = naddrs.clone().into_coordinates();
    let repos = client.fetch_repos(&coordinates).await?;
    let authorized_pubkeys = [naddrs.extract_owners(), repos.extract_maintainers()].concat();
    client.add_relays(&repos.extract_relays()).await;
    // This helps discover issues and their status.
    client
        .add_relays(&client.read_relays_from_users(&authorized_pubkeys).await)
        .await;

    let mut filter = Filter::new()
        .coordinates(coordinates.iter())
        .kind(entity_type.kind())
        .limit(limit);

    if entity_type.is_patch() {
        filter = filter.hashtag("root");
    }

    let arc_client = Arc::new(client);
    // Events are sorted by kind in ascending order:
    // 1630 (Open), 1631 (Resolved/Applied), 1632 (Closed), 1633 (Draft)
    let events = utils::sort_by_key(
        future::join_all(
            arc_client
                .fetch_events(filter)
                .await?
                .take(limit)
                .map(|event| {
                    let c = arc_client.clone();
                    let keys = authorized_pubkeys.clone();
                    async move {
                        let status = match entity_type {
                            EntityType::PullRequest => {
                                c.fetch_pr_status(
                                    event.id,
                                    [keys.as_slice(), &[event.pubkey]].concat(),
                                )
                                .await
                                .map(|s| (s.as_str(), s.kind().as_u16()))?
                            }
                            EntityType::Patch => {
                                let (root, root_revision) = get_patch_root_revision(&event)?;
                                c.fetch_patch_status(
                                    root,
                                    root_revision,
                                    [keys.as_slice(), &[event.pubkey]].concat(),
                                )
                                .await
                                .map(|s| (s.as_str(), s.kind().as_u16()))?
                            }
                            EntityType::Issue => {
                                c.fetch_issue_status(
                                    event.id,
                                    [keys.as_slice(), &[event.pubkey]].concat(),
                                )
                                .await
                                .map(|s| (s.as_str(), s.kind().as_u16()))?
                            }
                        };

                        N34Result::Ok((event, status))
                    }
                }),
        )
        .await
        .into_iter()
        .filter_map(|r| r.ok()),
        |(_, (_, k))| *k,
    );

    let lines = events
        .map(|(event, (status, _))| format_entity::<ENTITY_TYPE>(&event, status))
        .collect::<Vec<String>>();

    let max_width = lines
        .iter()
        .map(|s| s.split_once('\n').map_or(85, |(l, _)| l.chars().count()))
        .max()
        .unwrap_or(85)
        .max(67); // length of the event id

    println!("{}", lines.join(&format!("{}\n", "-".repeat(max_width))));

    Ok(())
}

/// Returns a tuple of (root_id, patch_id) if this is a valid root or revision
/// patch.
fn get_patch_root_revision(patch_event: &Event) -> N34Result<(EventId, Option<EventId>)> {
    if patch_event.is_revision_patch() {
        Ok((
            patch_event.root_patch_from_revision()?,
            Some(patch_event.id),
        ))
    } else if patch_event.is_root_patch() {
        Ok((patch_event.id, None))
    } else {
        Err(N34Error::NotRootPatch)
    }
}

/// Formats patch, issue or PR. For patches, extracts the
/// subject line from the Git patch format. For issues and PRs, combines the
/// subject with labels. The output includes status and formatted ID.
fn format_entity<const ENTITY_TYPE: u8>(event: &Event, status: &str) -> String {
    let entity_type = EntityType::from_u8::<ENTITY_TYPE>();

    let subject = match entity_type {
        EntityType::Patch => {
            GitPatch::from_str(&event.content)
                .map(|p| p.subject)
                .unwrap_or_else(|_| {
                    event
                        .content
                        .lines()
                        .find(|line| line.trim().starts_with("Subject: "))
                        .unwrap_or_default()
                        .trim()
                        .trim_start_matches("Subject: ")
                        .to_owned()
                })
        }
        _ => {
            // Issues and PRs
            let labels = event.extract_event_labels();
            let subject = event.extract_event_subject();

            if labels.is_empty() {
                subject.to_owned()
            } else {
                format!(r#""{subject}" {labels}"#)
            }
        }
    };

    format!(
        "({status}) {}\nID: {}\n",
        utils::smart_wrap(&subject, 85),
        event.id.to_bech32().expect("Infallible")
    )
}

/// Generates a list of tags for quoting patches in merge/applied status events.
async fn build_patches_quote(
    client: NostrClient,
    relay_hint: Option<RelayUrl>,
    patches: Vec<EventId>,
) -> Vec<Tag> {
    let client = Arc::new(client);
    let relay_hint = Arc::new(relay_hint);

    future::join_all(patches.into_iter().map(|eid| {
        let task_relay = Arc::clone(&relay_hint);
        let task_client = Arc::clone(&client);

        async move {
            Tag::custom(
                TagKind::q(),
                [
                    eid.to_hex(),
                    task_relay
                        .as_ref()
                        .as_ref()
                        .map(|r| r.to_string())
                        .unwrap_or_default(),
                    task_client
                        .event_author(eid)
                        .await
                        .ok()
                        .flatten()
                        .map(|p| p.to_hex())
                        .unwrap_or_default(),
                ],
            )
        }
    }))
    .await
}

pub async fn view_pr_issue<const IS_PR: bool>(
    options: CliOptions,
    naddrs: Option<Vec<NaddrOrSet>>,
    event_id: NostrEvent,
) -> N34Result<()> {
    let naddrs = utils::naddrs_or_file(
        naddrs.flat_naddrs(&options.config.sets)?,
        &utils::nostr_address_path()?,
    )?;
    let relays = options.relays.clone().flat_relays(&options.config.sets)?;
    let client = NostrClient::init(&options, &relays).await;

    client.add_relays(&naddrs.extract_relays()).await;
    client.add_relays(&event_id.relays).await;
    let repos = client.fetch_repos(&naddrs.into_coordinates()).await?;
    client.add_relays(&repos.extract_relays()).await;

    let event = client
        .fetch_event(Filter::new().id(event_id.event_id).kind(
            const {
                if IS_PR {
                    crate::cli::pr::PR_KIND
                } else {
                    Kind::GitIssue
                }
            },
        ))
        .await?
        .ok_or(
            const {
                if IS_PR {
                    N34Error::CanNotFoundPr
                } else {
                    N34Error::CanNotFoundIssue
                }
            },
        )?;
    let authorized_pubkeys = [repos.extract_maintainers().as_slice(), &[event.pubkey]].concat();
    let status = if IS_PR {
        client
            .fetch_pr_status(event.id, authorized_pubkeys)
            .await?
            .to_string()
    } else {
        client
            .fetch_issue_status(event.id, authorized_pubkeys)
            .await?
            .to_string()
    };

    let event_subject = utils::smart_wrap(event.extract_event_subject(), 70);
    let event_author = client.get_username(event.pubkey).await;
    let mut event_labels = utils::smart_wrap(&event.extract_event_labels(), 70);

    if event_labels.is_empty() {
        event_labels = "\n".to_owned();
    } else {
        event_labels = format!("{event_labels}\n\n")
    }

    let pr_data = if IS_PR {
        // Check if there is an update
        let pr_update = client
            .fetch_event(
                Filter::new()
                    .kind(crate::cli::pr::PR_UPDATE_KIND)
                    .custom_tag(SingleLetterTag::uppercase(Alphabet::E), event.id)
                    .author(event.pubkey),
            )
            .await?;

        let commit = pr_update
            .as_ref()
            .unwrap_or(&event)
            .tags
            .find(TagKind::single_letter(Alphabet::C, false))
            .and_then(|t| t.content())
            .unwrap_or("N/A");
        let branch = event
            .tags
            .find(TagKind::custom("branch-name"))
            .and_then(|t| t.content())
            .unwrap_or("N/A");
        let clones = pr_update
            .as_ref()
            .unwrap_or(&event)
            .tags
            .iter()
            .filter_map(|t| t.as_standardized())
            .find_map(|t| {
                match t {
                    TagStandard::GitClone(urls) if !urls.is_empty() => Some(urls.clone()),
                    _ => None,
                }
            })
            .unwrap_or_default();

        format!(
            "\nCommit: {commit}\nBranch: {branch}\nClone:\n{}",
            utils::format_iter(clones)
        )
    } else {
        String::new()
    };

    println!(
        "({status}) {event_subject} - [by {event_author}]\n{event_labels}{}{pr_data}",
        utils::smart_wrap(&event.content, 80)
    );
    Ok(())
}
