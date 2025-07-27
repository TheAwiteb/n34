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

use std::{collections::HashSet, fs, path::PathBuf};

use nostr::{
    nips::{nip19::Nip19Coordinate, nip46::NostrConnectURI},
    types::RelayUrl,
};

use crate::error::{N34Error, N34Result};

/// Errors that can occur when working with configuration files.
#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error(
        "Could not determine the default config path: both `$XDG_CONFIG_HOME` and `$HOME` \
         environment variables are missing or unset."
    )]
    CanNotFindConfigPath,
    #[error("Couldn't read the config file: {0}")]
    ReadFile(std::io::Error),
    #[error("Couldn't write in the config file: {0}")]
    WriteFile(std::io::Error),
    #[error("Couldn't serialize the config. This is a bug, please report it: {0}")]
    Serialize(toml::ser::Error),
    #[error("Failed to parse the config file: {0}")]
    ParseFile(toml::de::Error),
    #[error("Duplicate configuration set name detected: '{0}'. Each set  must have a unique name.")]
    SetDuplicateName(String),
    #[error("No set with the given name `{0}`")]
    SetNotFound(String),
    #[error("You can't create an new empty set.")]
    NewEmptySet,
}

/// Configuration for the command-line interface.
#[derive(serde::Serialize, serde::Deserialize, Clone, Default, Debug)]
pub struct CliConfig {
    /// Path to the configuration file (not serialized)
    #[serde(skip)]
    path:                   PathBuf,
    /// Groups of repositories and relays.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sets:               Vec<RepoRelaySet>,
    /// The default PoW difficulty
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pow:                Option<u8>,
    /// List of fallback relays used if no fallback relays was provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_relays:    Option<Vec<RelayUrl>>,
    /// Default Nostr bunker URL used for signing events.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::parsers::de_bunker_url",
        serialize_with = "super::parsers::ser_bunker_url"
    )]
    pub bunker_url:         Option<NostrConnectURI>,
    /// Whether to use the system keyring to store the secret key.
    #[serde(default)]
    pub keyring_secret_key: bool,
}

/// A named group of repositories and relays.
#[derive(serde::Serialize, serde::Deserialize, Default, Clone, Debug)]
pub struct RepoRelaySet {
    /// Unique identifier for this group.
    pub name:   String,
    /// Repository addresses in this group.
    #[serde(
        default,
        skip_serializing_if = "HashSet::is_empty",
        serialize_with = "super::parsers::ser_naddrs",
        deserialize_with = "super::parsers::de_naddrs"
    )]
    pub naddrs: HashSet<Nip19Coordinate>,
    /// Relay URLs in this group.
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub relays: HashSet<RelayUrl>,
}

impl CliConfig {
    /// Reads and parse a TOML config file from the given path, creating it if
    /// missing.
    pub fn load(file_path: PathBuf) -> N34Result<Self> {
        tracing::info!(path = %file_path.display(), "Loading configuration from file");
        // Make sure the file is exist
        if let Some(parent) = file_path.parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent)?;
        }

        let _ = fs::File::create_new(&file_path);

        let mut config: Self =
            toml::from_str(&fs::read_to_string(&file_path).map_err(ConfigError::ReadFile)?)
                .map_err(ConfigError::ParseFile)?;
        config.path = file_path;

        config.post_sets()?;

        Ok(config)
    }

    /// Dump the config as toml in a file
    pub fn dump(mut self) -> N34Result<()> {
        tracing::debug!(config = ?self, "Writing configuration to {}", self.path.display());
        self.post_sets()?;

        fs::write(
            &self.path,
            toml::to_string_pretty(&self).map_err(ConfigError::Serialize)?,
        )
        .map_err(ConfigError::WriteFile)?;

        Ok(())
    }

    /// Performs post-processing validation on the sets after loading or before
    /// dumping.
    fn post_sets(&mut self) -> N34Result<()> {
        self.sets.as_slice().ensure_names()?;
        self.sets.dedup_naddrs();

        Ok(())
    }
}

impl RepoRelaySet {
    /// Create a new [`RepoRelaySet`]
    pub fn new(
        name: impl Into<String>,
        naddrs: impl IntoIterator<Item = Nip19Coordinate>,
        relays: impl IntoIterator<Item = RelayUrl>,
    ) -> Self {
        Self {
            name:   name.into(),
            naddrs: HashSet::from_iter(naddrs),
            relays: HashSet::from_iter(relays),
        }
    }

    /// Removes duplicate repository addresses by comparing their coordinates,
    /// ignoring embedded relays.
    pub fn dedup_naddrs(&mut self) {
        let mut seen = HashSet::new();
        self.naddrs.retain(|n| seen.insert(n.coordinate.clone()));
    }
}

#[easy_ext::ext(MutRepoRelaySetsExt)]
impl Vec<RepoRelaySet> {
    /// Removes duplicate repository addresses from each set.
    ///
    /// Relays are automatically deduplicated by the HashSet, but
    /// repository addresses may appear duplicated if relays are sorted
    /// differently or when relay counts vary. This compares addresses by
    /// their coordinates, ignoring any embedded relay details.
    pub fn dedup_naddrs(&mut self) {
        self.iter_mut().for_each(RepoRelaySet::dedup_naddrs);
    }

    /// Finds and returns a mutable reference a set with the given name. Returns
    /// an error if no set with this name exists.
    pub fn get_mut_set(&mut self, name: impl AsRef<str>) -> N34Result<&mut RepoRelaySet> {
        let name = name.as_ref();
        let set = self
            .iter_mut()
            .find(|set| set.name == name)
            .ok_or_else(|| N34Error::from(ConfigError::SetNotFound(name.to_owned())))?;

        tracing::trace!(
            name = %name, set = ?set,
            "Successfully located a set with the giving name"
        );

        Ok(set)
    }

    /// Creates and pushes a new set with the given name.
    ///
    /// Returns an error if a set with the same name already exists.
    pub fn push_set(
        &mut self,
        name: impl Into<String>,
        repos: impl IntoIterator<Item = Nip19Coordinate>,
        relays: impl IntoIterator<Item = RelayUrl>,
    ) -> N34Result<()> {
        let set_name: String = name.into();
        tracing::trace!(sets = ?self, "Pushing set '{set_name}' to sets collection");

        if self.as_slice().exists(&set_name) {
            return Err(ConfigError::SetDuplicateName(set_name).into());
        }

        self.push(RepoRelaySet::new(set_name, repos, relays));

        Ok(())
    }

    /// Removes the set with the given name if it exists. Returns an error if
    /// the set is not found.
    pub fn remove_set(&mut self, name: impl Into<String>) -> N34Result<()> {
        let set_name: String = name.into();
        tracing::trace!(set_name, sets = ?self, "Removing set '{set_name}' from sets collection");

        if !self.as_slice().exists(&set_name) {
            return Err(ConfigError::SetNotFound(set_name).into());
        }

        self.retain(|s| s.name != set_name);

        Ok(())
    }

    /// Removes the given relays from the specified set.
    pub fn remove_relays(
        &mut self,
        name: impl Into<String>,
        relays: impl Iterator<Item = RelayUrl>,
    ) -> N34Result<()> {
        let relays = Vec::from_iter(relays);
        let set = self.get_mut_set(name.into())?;

        set.relays.retain(|r| !relays.contains(r));

        Ok(())
    }

    /// Removes the given naddrs from the specified set.
    pub fn remove_naddrs(
        &mut self,
        name: impl Into<String>,
        naddrs: impl Iterator<Item = Nip19Coordinate>,
    ) -> N34Result<()> {
        let coordinates = Vec::from_iter(naddrs.map(|n| n.coordinate));
        let set = self.get_mut_set(name.into())?;

        set.naddrs.retain(|n| !coordinates.contains(&n.coordinate));

        Ok(())
    }
}

#[easy_ext::ext(RepoRelaySetsExt)]
impl &[RepoRelaySet] {
    /// Checks for duplicate set names. Returns an error if any duplicates are
    /// found.
    pub fn ensure_names(&self) -> N34Result<()> {
        let mut names = Vec::with_capacity(self.len());
        names.extend(self.iter().map(|s| s.name.to_owned()));

        names.sort_unstable();

        if let Some(duplicate) = duplicate_in_sorted(&names) {
            return Err(ConfigError::SetDuplicateName(duplicate.clone()).into());
        }
        Ok(())
    }

    /// Check if a set with the given name exists.
    pub fn exists(&self, set_name: &str) -> bool {
        self.iter().any(|set| set.name == set_name)
    }

    /// Finds and returns a reference a set with the given name. Returns an
    /// error if no set with this name exists.
    pub fn get_set(&self, name: impl AsRef<str>) -> N34Result<&RepoRelaySet> {
        let name = name.as_ref();
        let set = self
            .iter()
            .find(|set| set.name == name)
            .ok_or_else(|| N34Error::from(ConfigError::SetNotFound(name.to_owned())))?;
        tracing::trace!(
            name = %name, set = ?set,
            "Successfully located a set with the giving name"
        );
        Ok(set)
    }
}

/// Helper function that checks for duplicates in a sorted slice
fn duplicate_in_sorted<T: PartialEq + Clone>(items: &[T]) -> Option<&T> {
    items.windows(2).find(|w| w[0] == w[1]).map(|w| &w[0])
}
