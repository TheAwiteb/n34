# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- New command `repo state` - by Awiteb on dd05e05
- Support pull requests - by Awiteb on 4360aa1
- logs: Log to stderr and a file - by Awiteb on 95a46a7
- Accept patches from stdin in `patch send` command - by Awiteb on ee17c21
- Write patches to the stdout in `patch fetch` command - by Awiteb on d2adff7
- repo-announce: Support `u` tag for forks - by Awiteb on 041a2e6
- Enable Tor proxy for onion-service relay connections - by Awiteb on 06c75d2

### Breaking Change

- issue: Make subject mandatory and optional content - by Awiteb on ed1bfaf

### Dependencies

- Add `io-std` feature to `tokio` - by Awiteb on df54a53
- Remove `async-utility` - by Awiteb on 8688c2a

### Documentation

- Update commands help messages - by Awiteb on c308dc1
- Update commands titles and descriptions - by Awiteb on 4bcf104
- Remove PR from pull requests - by Awiteb on d3ac52b
- sets-update: Refrence `passing repositories` section - by Awiteb on 6072dc6

### Fixed

- Check the nip07 signer in `CliOptions::ensure_signer` - by Awiteb on ec68e97
- n34-patch-apply: Required the `applied_commits` - by Awiteb on 089b62a

### Refactor

- Remove `dbus` C dep - by Awiteb on 463010c

## [0.4.0] - 2025-08-08

### Added

- Support signing using NIP-46 bunker - by Awiteb on 4e0ecdc
- Keyring the secret key `n34 config keyring --enable` - by Awiteb on 03d5c80
- New flag to `patch apply and merge` to mention patches - by Awiteb on 67e25da
- Sign using NIP-07 - by Awiteb on 904d140

### Dependencies

- Add `keyring`, `nostr-connect`, `nostr-keyring` and `url` to the dependencies - by Awiteb on f0c20c3
- Remove `url` from `n34` dependencies - by Awiteb on bc8c6f3
- Upgrade to `nostr@0.43.0` - by Awiteb on 1d83e44

### Documentation

- N34 book - by Awiteb on bd3ba1b
- Fix status command docs - by Awiteb on 78113c7

### Refactor

- Move the trait extensions to `traits.rs` - by Awiteb on e17e75e
- Move `root` and `root-revision` to constants - by Awiteb on 8bb4cf0

## [0.3.0] - 2025-07-05

### Added

- New `patch send` command to send patches - by Awiteb on ef8d6c1
- Add `alt` tag to the git issue - by Awiteb on 494cced
- Add `description` tag to the patch - by Awiteb on 812a06a
- New `config pow` command to set the default PoW difficulty - by Awiteb on 51bd239
- New `config relays` command to set the default fallbacks relays - by Awiteb on 5dc8e31
- New `issue view` command to view an issue - by Awiteb on da96985
- New `patch fetch` command to fetch patches - by Awiteb on 364356a
- New `issue {reopen,close,resolve}` commands to manage issue status - by Awiteb on a9a2cb2
- New `patch` subcommands apply,close,draft,merge and reopen to manage the patch status - by Awiteb on 8b09cff
- View the repo maintainers as `npub` - by Awiteb on da284a0
- New `patch list` commands to list the repo patches - by Awiteb on 387dd32
- New `issue list` commands to list the repo issues - by Awiteb on 05b4ae3
- Improve exit codes and make them more specific - by Awiteb on 3510b59

### Dependencies

- Add `either@1.15.0` to the dependencies - by Awiteb on 93136fb
- Update `nostr` to `0.42.2` - by Awiteb on a38a811
- Remove `bitcoin_hashes` and use `nostr::hashes` re-export - by Awiteb on 55e5d86

### Fixed

- Not to return an error if `nostr-address` file does not exist - by Awiteb on 1651fd5
- Spelling in help content - by DanConwayDev on 7c589f1
- Fix a typo in `EmptySetRelays` error message - by Awiteb on 7d70060
- Require a repo in `repo view` command - by Awiteb on 4cc0166

### Refactor

- Store the config in `CliOptions` instead of its path - by Awiteb on a6a61ae
- Improve commands running and make the signer optional for some commands - by Awiteb on b1027b3

### Removed

- Remove the `--repo` option and make the repo an argument - by Awiteb on 45ea7d2
- Remove the `--to` flag from `reply` command and make it an argument - by Awiteb on 6467bc3
- Remove `--euc` flag from `patch send` command and use the repo euc - by Awiteb on 2874ba8

## [0.2.0] - 2025-06-01

### Added

- Add `--force-id` flag to bypass case validation in `repo announce` - by Awiteb on 06374fe
- Add `--address-file` flag to `repo announce` command - by Awiteb on 57b48c7
- Read the `nostr-address` file in `repo view` command - by Awiteb on 8ca8880
- Read the `nostr-address` file in `issue new` command - by Awiteb on 226909e
- Read the `nostr-address` file in `reply` command - by Awiteb on 6fdf0db
- A `--quote-to` flag to quote the replied to content in the editor - by Awiteb on 02070c2
- Enter repository as nip5 - by Awiteb on de68d61
- Make the relays list optional - by Awiteb on ddea502
- Events and naddrs can starts with `nostr:` - by Awiteb on 1abb8e3
- Support relays and naddrs sets - by Awiteb on 4c6578c

### Dependencies

- Add `chrono@0.4.41` to the dependencies - by Awiteb on 998ef8f
- Enable `nip05` feature of `nostr` crate - by Awiteb on f7e837e
- Add `serde@1.0.219`, `dirs@6.0.0` and `toml@0.8.22` - by Awiteb on 84bfafa

### Fixed

- utils-repo_naddr: Create a valid naddr string - by Awiteb on 55a4868

### Refactor

- Support more than one naddr instead of one - by Awiteb on 37cf601

## [0.1.0] - 2025-05-21

### Added

- Setup the CLI and create `repo view` command - by Awiteb on d962732
- Add `repo announce` command - by Awiteb on b444aeb
- nip13: Support PoW - by Awiteb on c0a5e47
- Add `issue new` command - by Awiteb on 54f1c7e
- New `reply` command - by Awiteb on 9444fc1

### Dependencies

- Add nedded dependencies - by Awiteb on d37c696
- Add `easy_ext@1.0.2` to the dependencies - by Awiteb on fb77a8c
- Add `convert_case@0.8.0` to the dependencies - by Awiteb on 8a553f4
- Add `tempfile@3.19.1` to the dependencies - by Awiteb on ce58f29
- Add `futures@0.3.31` to the dependencies - by Awiteb on bd08653
- Bump `nostr` and `nostr_sdk` to `0.42.0` - by Awiteb on 724e270

## [0.0.0] - 2025-05-01

### Added

- Initialize the project - by Awiteb on c3594c6

[0.4.0]: https://git.4rs.nl/awiteb/n34.git/tag/?h=v0.4.0
[0.3.0]: https://git.4rs.nl/awiteb/n34.git/tag/?h=v0.3.0
[0.2.0]: https://git.4rs.nl/awiteb/n34.git/tag/?h=v0.2.0
[0.1.0]: https://git.4rs.nl/awiteb/n34.git/tag/?h=v0.1.0
[0.0.0]: https://git.4rs.nl/awiteb/n34.git/tag/?h=v0.0.0

<!-- generated by git-cliff -->
