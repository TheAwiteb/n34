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

use std::process::ExitCode;

use nostr::event::{Kind, builder::Error as EventBuilderError};
use nostr_sdk::client::Error as ClientError;

use crate::cli::ConfigError;

pub type N34Result<T> = Result<T, N34Error>;

/// N34 errors
#[derive(Debug, thiserror::Error)]
pub enum N34Error {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Config(#[from] ConfigError),
    #[error("No editor specified in the `EDITOR` environment variable")]
    EditorNotFound,
    #[error("The file you edited is empty. Please save your changes before exiting the editor.")]
    EmptyEditorFile,
    #[error("Client Error: {0}")]
    Client(#[from] ClientError),
    #[error("Unable to locate the repository. The repository may not exists in the given relays")]
    NotFoundRepo,
    #[error("Failed building an event: {0}")]
    EventBuilder(#[from] EventBuilderError),
    #[error("Invalid repository id, it can't be empty and must be kebab-case")]
    InvalidRepoId,
    #[error("Invalid event: {0}")]
    InvalidEvent(String),
    #[error("Bech32 error: {0}")]
    Bech32(#[from] nostr::nips::nip19::Error),
    #[error("Event error: {0}")]
    Event(#[from] nostr::event::Error),
    #[error("Event not found in the specified relays")]
    EventNotFound,
    #[error(
        "Can't reply to this event. Only Git issues, patches, and their comments can be replied \
         to."
    )]
    CanNotReplyToEvent,
    #[error("No repository address given and couldn't read `nostr-address` file: {0}")]
    CanNotReadNostrAddressFile(std::io::Error),
    #[error(
        "The `nostr-address` file is empty.  Please add a valid Nostr repository address (naddr) \
         to the file or provide it manually as a flag."
    )]
    EmptyNostrAddressFile,
    #[error("Invalid `nostr-address` file content: {0}")]
    InvalidNostrAddressFileContent(String),
    #[error("This command requires at least one relay, but none were provided")]
    EmptyRelays,
    #[error("This command requires a signer to sign events")]
    SignerRequired,
    #[error(
        "Invalid repository address. Expected one of these formats:\n- NIP-05 identifier with \
         repository ID: `<user@domain.com>/<repo_id>`\n- Valid NIP-19 naddr string (starts with \
         'naddr1...')\n- Existing set name (merges all repositories in set)\nError: No set named \
         '{0}' exists."
    )]
    InvalidNaddrArg(String),
    #[error(
        "The set '{0}' doesn't contain any addresses. Use 'sets update' to add addresses to it."
    )]
    EmptySetNaddrs(String),
    #[error("The set '{0}' doesn't contain any relays. Use 'sets update' to add addresses to it.")]
    EmptySetRelays(String),
    #[error(
        "Issue not found, make sure it is in the relays and make sure that the ID is an issue ID"
    )]
    CanNotFoundIssue,
    #[error(
        "Patch not found, make sure it is in the relays and make sure that the ID is an patch ID"
    )]
    CanNotFoundPatch,
    #[error(r#"The given patch id is not a root patch. It must contains `["t", "root"]` tag"#)]
    NotRootPatch,
    #[error("This status kind can't be set for an issue: {0}")]
    InvalidIssueStatus(Kind),
    #[error("This status kind can't be set for a patch: {0}")]
    InvalidPatchStatus(Kind),
    #[error("Can't find the root patch of the given patch-revision")]
    RevisionRootNotFound,
    #[error("Invalid status for the issue/patch: {0}")]
    InvalidStatus(String),
}

impl N34Error {
    /// Returns the exit code associated with this error
    pub fn exit_code(&self) -> ExitCode {
        // TODO: More specific exit code
        ExitCode::FAILURE
    }
}
