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

use nostr::event::EventId;

use super::CliOptions;
use crate::{cli::types::NostrEvent, error::N34Result};

/// A trait defining the interface for command runners in the CLI.
pub trait CommandRunner {
    /// Whether this command needs the relays option (false by default).
    /// Only applies to commands, not subcommands.
    const NEED_RELAYS: bool = false;
    /// Indicates if this command requires the signer. Defaults to true.
    /// Only applies to commands, not subcommands.
    const NEED_SIGNER: bool = true;

    /// Executes the command and returns a Result indicating success or failure.
    fn run(self, options: CliOptions) -> impl Future<Output = N34Result<()>> + Send;
}

#[easy_ext::ext(VecNostrEventExt)]
impl Vec<NostrEvent> {
    /// Extracts `EventId` from each `NostrEvent` and collects them into a
    /// `Vec<EventId>`.
    pub fn into_event_ids(self) -> Vec<EventId> {
        self.into_iter().map(|e| e.event_id).collect()
    }
}
