# This justfile is for the contrbutors of this project, not for the end user.
#
# Requirements for this justfile:
# - Linux distribution
# - just (Of course) <https://github.com/casey/just>
# - cargo (For the build and tests) <https://doc.rust-lang.org/cargo/getting-started/installation.html>

set quiet
set unstable
set shell := ["/usr/bin/env", "bash", "-c"]
set script-interpreter := ["/usr/bin/env", "bash", "-c"]

JUST_EXECUTABLE := "just -u -f " + justfile()
header := "Available tasks:\n"
# Get the MSRV from the Cargo.toml
msrv := `cat Cargo.toml | grep "rust-version" | sed 's/.*"\(.*\)".*/\1/'`


_default:
    @{{JUST_EXECUTABLE}} --list-heading "{{header}}" --list

# Run the CI
ci: && msrv
    cargo build -q
    cargo fmt -- --check
    cargo clippy -- -D warnings

# Check that the current MSRV is correct
msrv:
    rustup toolchain install {{msrv}}
    echo "Checking MSRV ({{msrv}})"
    cargo +{{msrv}} check -q
    echo "MSRV is correct"

# Update the changelog
[script]
change-log:
    OLD_HASH=$(sha256sum CHANGELOG.md | head -c 64)
    git-cliff > CHANGELOG.md
    NEW_HASH=$(sha256sum CHANGELOG.md | head -c 64)
    if [[ $OLD_HASH != $NEW_HASH ]]; then
        TZ=UTC git add CHANGELOG.md
        TZ=UTC git commit -m 'chore(changelog): Update the changelog'
        echo 'The changes have been added to the changelog file and committed'
    else
        echo 'No changes have been added to the changelog'
    fi

alias cl := change-log
