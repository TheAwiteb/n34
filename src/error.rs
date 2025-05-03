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

use nostr_sdk::client::Error as ClientError;

pub type N34Result<T> = Result<T, N34Error>;

/// N34 errors
#[derive(Debug, thiserror::Error)]
pub enum N34Error {
    #[error("Client Error: {0}")]
    Client(#[from] ClientError),
    #[error("Unable to locate the repository. The repository may not exists in the given relays")]
    NotFoundRepo,
}

impl N34Error {
    /// Returns the exit code associated with this error
    pub fn exit_code(&self) -> ExitCode {
        // TODO: More specific exit code
        ExitCode::FAILURE
    }
}
