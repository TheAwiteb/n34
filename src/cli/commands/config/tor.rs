// n34 - A CLI to interact with NIP-34 and other stuff related to code in Nostr
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

use clap::Args;

use crate::{
    cli::{CliOptions, traits::CommandRunner},
    error::N34Result,
};

#[derive(Args, Debug)]
pub struct TorArgs {
    /// Tor proxy port. Set to 0 to disable tor proxy
    #[arg(default_value = "9050")]
    port: u16,
}

impl CommandRunner for TorArgs {
    const NEED_SIGNER: bool = false;

    async fn run(self, mut options: CliOptions) -> N34Result<()> {
        match self.port {
            0 => options.config.tor = None,
            p => options.config.tor = Some(p),
        }
        options.config.dump()
    }
}
