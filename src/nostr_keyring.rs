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

use async_utility::task;
use keyring::Entry;
use nostr::key::{Keys, SecretKey};

use crate::error::{N34Error, N34Result};

/// Keyring service name of n34
const N34_KEYRING_SERVICE_NAME: &str = "n34";
/// Keyring entry name of the n34 keypair
const N34_KEY_PAIR_ENTRY: &str = "n34_keypair";
/// Keyring entry name of the user secret key
const USER_KEY_PAIR_ENTRY: &str = "user_keypair";

/// Retrieves the keypair entry for `n34`.
#[inline]
fn n34_entry() -> keyring::Result<Entry> {
    Entry::new(N34_KEYRING_SERVICE_NAME, N34_KEY_PAIR_ENTRY)
}

/// Retrieves the keypair entry for the user.
#[inline]
fn user_entry() -> keyring::Result<Entry> {
    Entry::new(N34_KEYRING_SERVICE_NAME, USER_KEY_PAIR_ENTRY)
}

pub mod user {
    use super::*;

    /// Stores the user's keypair in the system.
    pub async fn set(key: &Keys) -> N34Result<()> {
        let private_key = key.secret_key().to_secret_bytes();

        task::spawn_blocking(move || {
            user_entry()?
                .set_secret(&private_key)
                .map_err(N34Error::from)
        })
        .await?
    }

    /// Retrieves the user's keypair from the system.
    pub async fn get() -> N34Result<Keys> {
        task::spawn_blocking(move || {
            Ok(Keys::new(SecretKey::from_slice(
                &user_entry()?.get_secret()?,
            )?))
        })
        .await?
    }

    /// Delete the user's keypair from the system.
    pub async fn delete() -> N34Result<()> {
        task::spawn_blocking(move || user_entry()?.delete_credential().map_err(N34Error::from))
            .await?
    }
}

pub mod n34 {
    use super::*;

    /// Stores the `n34` client's keypair in the system.
    pub async fn set(key: &Keys) -> N34Result<()> {
        let private_key = key.secret_key().to_secret_bytes();

        task::spawn_blocking(move || {
            n34_entry()?
                .set_secret(&private_key)
                .map_err(N34Error::from)
        })
        .await?
    }

    /// Retrieves the `n34` client's keypair from the system.
    pub async fn get() -> N34Result<Keys> {
        task::spawn_blocking(move || {
            Ok(Keys::new(SecretKey::from_slice(
                &n34_entry()?.get_secret()?,
            )?))
        })
        .await?
    }
}
