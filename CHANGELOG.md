# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- New command `repo state` - by Awiteb
- Support pull requests - by Awiteb
- Add `--personal-fork` flag to `repo announce` command - by Awiteb
- Log to stderr and a file - by Awiteb
- Accept patches from stdin in `patch send` command - by Awiteb

### Breaking Change

- Make subject mandatory and optional content - by Awiteb

### Dependencies

- Add `io-std` feature to `tokio` - by Awiteb

### Documentation

- Update commands help messages - by Awiteb
- Update commands titles and descriptions - by Awiteb

### Fixed

- Check the nip07 signer in `CliOptions::ensure_signer` - by Awiteb
- Required the `applied_commits` - by Awiteb

## [0.4.0] - 2025-08-08

### Added

- Support signing using NIP-46 bunker - by Awiteb
- Keyring the secret key `n34 config keyring --enable` - by Awiteb
- New flag to `patch apply and merge` to mention patches - by Awiteb
- Sign using NIP-07 - by Awiteb

### Dependencies

- Add `keyring`, `nostr-connect`, `nostr-keyring` and `url` to the dependencies - by Awiteb
- Remove `url` from `n34` dependencies - by Awiteb
- Upgrade to `nostr@0.43.0` - by Awiteb

### Documentation

- N34 book - by Awiteb
- Fix status command docs - by Awiteb

### Refactor

- Move the trait extensions to `traits.rs` - by Awiteb
- Move `root` and `root-revision` to constants - by Awiteb

## [0.3.0] - 2025-07-05

### Added

- New `patch send` command to send patches - by Awiteb
- Add `alt` tag to the git issue - by Awiteb
- Add `description` tag to the patch - by Awiteb
- New `config pow` command to set the default PoW difficulty - by Awiteb
- New `config relays` command to set the default fallbacks relays - by Awiteb
- New `issue view` command to view an issue - by Awiteb
- New `patch fetch` command to fetch patches - by Awiteb
- New `issue {reopen,close,resolve}` commands to manage issue status - by Awiteb
- New `patch` subcommands apply,close,draft,merge and reopen to manage the patch status - by Awiteb
- View the repo maintainers as `npub` - by Awiteb
- New `patch list` commands to list the repo patches - by Awiteb
- New `issue list` commands to list the repo issues - by Awiteb
- Improve exit codes and make them more specific - by Awiteb

### Dependencies

- Add `either@1.15.0` to the dependencies - by Awiteb
- Update `nostr` to `0.42.2` - by Awiteb
- Remove `bitcoin_hashes` and use `nostr::hashes` re-export - by Awiteb

### Fixed

- Not to return an error if `nostr-address` file does not exist - by Awiteb
- Spelling in help content - by DanConwayDev
- Fix a typo in `EmptySetRelays` error message - by Awiteb
- Require a repo in `repo view` command - by Awiteb

### Refactor

- Store the config in `CliOptions` instead of its path - by Awiteb
- Improve commands running and make the signer optional for some commands - by Awiteb

### Removed

- Remove the `--repo` option and make the repo an argument - by Awiteb
- Remove the `--to` flag from `reply` command and make it an argument - by Awiteb
- Remove `--euc` flag from `patch send` command and use the repo euc - by Awiteb

## [0.2.0] - 2025-06-01

### Added

- Add `--force-id` flag to bypass case validation in `repo announce` - by Awiteb
- Add `--address-file` flag to `repo announce` command - by Awiteb
- Read the `nostr-address` file in `repo view` command - by Awiteb
- Read the `nostr-address` file in `issue new` command - by Awiteb
- Read the `nostr-address` file in `reply` command - by Awiteb
- A `--quote-to` flag to quote the replied to content in the editor - by Awiteb
- Enter repository as nip5 - by Awiteb
- Make the relays list optional - by Awiteb
- Events and naddrs can starts with `nostr:` - by Awiteb
- Support relays and naddrs sets - by Awiteb

### Dependencies

- Add `chrono@0.4.41` to the dependencies - by Awiteb
- Enable `nip05` feature of `nostr` crate - by Awiteb
- Add `serde@1.0.219`, `dirs@6.0.0` and `toml@0.8.22` - by Awiteb

### Fixed

- Create a valid naddr string - by Awiteb

### Refactor

- Support more than one naddr instead of one - by Awiteb

## [0.1.0] - 2025-05-21

### Added

- Setup the CLI and create `repo view` command - by Awiteb
- Add `repo announce` command - by Awiteb
- Support PoW - by Awiteb
- Add `issue new` command - by Awiteb
- New `reply` command - by Awiteb

### Dependencies

- Add nedded dependencies - by Awiteb
- Add `easy_ext@1.0.2` to the dependencies - by Awiteb
- Add `convert_case@0.8.0` to the dependencies - by Awiteb
- Add `tempfile@3.19.1` to the dependencies - by Awiteb
- Add `futures@0.3.31` to the dependencies - by Awiteb
- Bump `nostr` and `nostr_sdk` to `0.42.0` - by Awiteb

## [0.0.0] - 2025-05-01

### Added

- Initialize the project - by Awiteb

[0.4.0]: https://git.4rs.nl/awiteb/n34.git/tag/?h=v0.4.0
[0.3.0]: https://git.4rs.nl/awiteb/n34.git/tag/?h=v0.3.0
[0.2.0]: https://git.4rs.nl/awiteb/n34.git/tag/?h=v0.2.0
[0.1.0]: https://git.4rs.nl/awiteb/n34.git/tag/?h=v0.1.0
[0.0.0]: https://git.4rs.nl/awiteb/n34.git/tag/?h=v0.0.0

<!-- generated by git-cliff -->
