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
    event::{EventBuilder, Tag},
    filter::Filter,
    nips::nip10::Marker,
};

use super::{
    issue::IssueStatus,
    types::{NaddrOrSet, NostrEvent},
};
use crate::{
    cli::CliOptions,
    error::{N34Error, N34Result},
    nostr_utils::traits::ReposUtils,
};
use crate::{
    cli::types::{OptionNaddrOrSetVecExt, RelayOrSetVecExt},
    nostr_utils::{NostrClient, traits::NaddrsUtils, utils},
};

/// Updates an issue's status to `new_status` after validating it with
/// `check_fn`.
pub async fn issue_status_command(
    options: CliOptions,
    issue_id: NostrEvent,
    naddrs: Option<Vec<NaddrOrSet>>,
    new_status: IssueStatus,
    check_fn: impl FnOnce(&IssueStatus) -> N34Result<()>,
) -> N34Result<()> {
    let user_pkey = options.pubkey().await?;
    let naddrs = utils::naddrs_or_file(
        naddrs.flat_naddrs(&options.config.sets)?,
        &utils::nostr_address_path()?,
    )?;
    let relays = options.relays.clone().flat_relays(&options.config.sets)?;
    let client = NostrClient::init(&options, &relays).await;
    client
        .add_relays(&[naddrs.extract_relays(), issue_id.relays].concat())
        .await;

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
            [maintainers.as_slice(), &[issue_event.pubkey]].concat(),
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
        .tags(
            coordinates
                .into_iter()
                .map(|c| Tag::coordinate(c, relay_hint.clone())),
        )
        .dedup_tags()
        .build(user_pkey);

    let event_id = status_event.id.expect("There is an id");
    let user_relays_list = client.user_relays_list(issue_event.pubkey).await?;
    let write_relays = [
        relays,
        naddrs.extract_relays(),
        repos.extract_relays(),
        utils::add_write_relays(user_relays_list.as_ref()),
        client.read_relays_from_user(issue_event.pubkey).await,
        client.read_relays_from_users(&maintainers).await,
    ]
    .concat();

    let success = client
        .send_event_to(status_event, user_relays_list.as_ref(), &write_relays)
        .await?;
    let nevent = utils::new_nevent(event_id, &success)?;
    println!("Issue status created: {nevent}");

    Ok(())
}
