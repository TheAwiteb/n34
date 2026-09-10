# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.1] - 2026-09-10

No functional changes or new features in this release.

## [0.5.0] - 2026-09-10

### Added

- New command `repo state` - by Awiteb in dd05e05
- Support pull requests - by Awiteb in 4360aa1
- logs: Log to stderr and a file - by Awiteb in 95a46a7
- Accept patches from stdin in `patch send` command - by Awiteb in ee17c21
- Write patches to the stdout in `patch fetch` command - by Awiteb in d2adff7
- repo-announce: Support `u` tag for forks - by Awiteb in 041a2e6
- Enable Tor proxy for onion-service relay connections - by Awiteb in 10d05cc
- Send nip07 request after user open the proxy page - by Awiteb in 2c7cc88

### Breaking Change

- issue: Make subject mandatory and optional content - by Awiteb in ed1bfaf

### Dependencies

- Add `io-std` feature to `tokio` - by Awiteb in df54a53
- Remove `async-utility` - by Awiteb in 8688c2a
- Update nostr crates to v0.44 - by Awiteb in 2961900

### Documentation

- Update commands help messages - by Awiteb in c308dc1
- Update commands titles and descriptions - by Awiteb in 4bcf104
- Remove PR from pull requests - by Awiteb in d3ac52b
- sets-update: Refrence `passing repositories` section - by Awiteb in 6072dc6

### Fixed

- Check the nip07 signer in `CliOptions::ensure_signer` - by Awiteb in ec68e97
- n34-patch-apply: Required the `applied_commits` - by Awiteb in 089b62a

### Refactor

- Remove `dbus` C dep - by Awiteb in 463010c

## [0.4.0] - 2025-08-08

### Added

- Support signing using NIP-46 bunker - by Awiteb in 4e0ecdc
- Keyring the secret key `n34 config keyring --enable` - by Awiteb in 03d5c80
- New flag to `patch apply and merge` to mention patches - by Awiteb in 67e25da
- Sign using NIP-07 - by Awiteb in 904d140

### Dependencies

- Add `keyring`, `nostr-connect`, `nostr-keyring` and `url` to the dependencies - by Awiteb in f0c20c3
- Remove `url` from `n34` dependencies - by Awiteb in bc8c6f3
- Upgrade to `nostr@0.43.0` - by Awiteb in 1d83e44

### Documentation

- N34 book - by Awiteb in bd3ba1b
- Fix status command docs - by Awiteb in 78113c7

### Refactor

- Move the trait extensions to `traits.rs` - by Awiteb in e17e75e
- Move `root` and `root-revision` to constants - by Awiteb in 8bb4cf0

## [0.3.0] - 2025-07-05

### Added

- New `patch send` command to send patches - by Awiteb in ef8d6c1
- Add `alt` tag to the git issue - by Awiteb in 494cced
- Add `description` tag to the patch - by Awiteb in 812a06a
- New `config pow` command to set the default PoW difficulty - by Awiteb in 51bd239
- New `config relays` command to set the default fallbacks relays - by Awiteb in 5dc8e31
- New `issue view` command to view an issue - by Awiteb in da96985
- New `patch fetch` command to fetch patches - by Awiteb in 364356a
- New `issue {reopen,close,resolve}` commands to manage issue status - by Awiteb in a9a2cb2
- New `patch` subcommands apply,close,draft,merge and reopen to manage the patch status - by Awiteb in 8b09cff
- View the repo maintainers as `npub` - by Awiteb in da284a0
- New `patch list` commands to list the repo patches - by Awiteb in 387dd32
- New `issue list` commands to list the repo issues - by Awiteb in 05b4ae3
- Improve exit codes and make them more specific - by Awiteb in 3510b59

### Dependencies

- Add `either@1.15.0` to the dependencies - by Awiteb in 93136fb
- Update `nostr` to `0.42.2` - by Awiteb in a38a811
- Remove `bitcoin_hashes` and use `nostr::hashes` re-export - by Awiteb in 55e5d86

### Fixed

- Not to return an error if `nostr-address` file does not exist - by Awiteb in 1651fd5
- Spelling in help content - by DanConwayDev in 7c589f1
- Fix a typo in `EmptySetRelays` error message - by Awiteb in 7d70060
- Require a repo in `repo view` command - by Awiteb in 4cc0166

### Refactor

- Store the config in `CliOptions` instead of its path - by Awiteb in a6a61ae
- Improve commands running and make the signer optional for some commands - by Awiteb in b1027b3

### Removed

- Remove the `--repo` option and make the repo an argument - by Awiteb in 45ea7d2
- Remove the `--to` flag from `reply` command and make it an argument - by Awiteb in 6467bc3
- Remove `--euc` flag from `patch send` command and use the repo euc - by Awiteb in 2874ba8

## [0.2.0] - 2025-06-01

### Added

- Add `--force-id` flag to bypass case validation in `repo announce` - by Awiteb in 06374fe
- Add `--address-file` flag to `repo announce` command - by Awiteb in 57b48c7
- Read the `nostr-address` file in `repo view` command - by Awiteb in 8ca8880
- Read the `nostr-address` file in `issue new` command - by Awiteb in 226909e
- Read the `nostr-address` file in `reply` command - by Awiteb in 6fdf0db
- A `--quote-to` flag to quote the replied to content in the editor - by Awiteb in 02070c2
- Enter repository as nip5 - by Awiteb in de68d61
- Make the relays list optional - by Awiteb in ddea502
- Events and naddrs can starts with `nostr:` - by Awiteb in 1abb8e3
- Support relays and naddrs sets - by Awiteb in 4c6578c

### Dependencies

- Add `chrono@0.4.41` to the dependencies - by Awiteb in 998ef8f
- Enable `nip05` feature of `nostr` crate - by Awiteb in f7e837e
- Add `serde@1.0.219`, `dirs@6.0.0` and `toml@0.8.22` - by Awiteb in 84bfafa

### Fixed

- utils-repo_naddr: Create a valid naddr string - by Awiteb in 55a4868

### Refactor

- Support more than one naddr instead of one - by Awiteb in 37cf601

## [0.1.0] - 2025-05-21

### Added

- Setup the CLI and create `repo view` command - by Awiteb in d962732
- Add `repo announce` command - by Awiteb in b444aeb
- nip13: Support PoW - by Awiteb in c0a5e47
- Add `issue new` command - by Awiteb in 54f1c7e
- New `reply` command - by Awiteb in 9444fc1

### Dependencies

- Add nedded dependencies - by Awiteb in d37c696
- Add `easy_ext@1.0.2` to the dependencies - by Awiteb in fb77a8c
- Add `convert_case@0.8.0` to the dependencies - by Awiteb in 8a553f4
- Add `tempfile@3.19.1` to the dependencies - by Awiteb in ce58f29
- Add `futures@0.3.31` to the dependencies - by Awiteb in bd08653
- Bump `nostr` and `nostr_sdk` to `0.42.0` - by Awiteb in 724e270

## [0.0.0] - 2025-05-01

### Added

- Initialize the project - by Awiteb in c3594c6

[0.5.1]: https://git.4rs.nl/awiteb/n34.git/tag/?h=v0.5.1
[0.5.0]: https://git.4rs.nl/awiteb/n34.git/tag/?h=v0.5.0
[0.4.0]: https://git.4rs.nl/awiteb/n34.git/tag/?h=v0.4.0
[0.3.0]: https://git.4rs.nl/awiteb/n34.git/tag/?h=v0.3.0
[0.2.0]: https://git.4rs.nl/awiteb/n34.git/tag/?h=v0.2.0
[0.1.0]: https://git.4rs.nl/awiteb/n34.git/tag/?h=v0.1.0
[0.0.0]: https://git.4rs.nl/awiteb/n34.git/tag/?h=v0.0.0

<!-- generated by git-cliff -->
