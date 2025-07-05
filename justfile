# This justfile is for the contrbutors of this project, not for the end user.
#
# Requirements for this justfile:
# - Linux distribution
# - just (Of course) <https://github.com/casey/just>
# - cargo (For the build and tests) <https://doc.rust-lang.org/cargo/getting-started/installation.html>

set quiet
set unstable
set shell := ["/usr/bin/env", "bash", "-c"]
set script-interpreter := ["/usr/bin/env", "bash"]

JUST_EXECUTABLE := "just -u -f " + justfile()
header := "Available tasks:\n"
# Get the MSRV from the Cargo.toml
tag_change_body := '''{% for group, commits in commits | group_by(attribute="group") %}

{{ group | upper_first }}

{% for commit in commits %}
- {{ commit.message | split(pat="\n") | first | split(pat=":") | slice(start=1) | join(sep=":") | upper_first | trim }} - by {{ commit.author.name}}{% endfor %}{% endfor %}
'''

export TZ := "UTC"

_default:
    @{{JUST_EXECUTABLE}} --list-heading "{{header}}" --list

# Run the CI
ci: && msrv _done_ci
    echo "🔨 Building n34..."
    cargo build -q
    echo "🔍 Checking code formatting..."
    cargo fmt -q -- --check
    RUST_LOG=none taplo fmt --check --config ./.taplo.toml || (echo "❌Toml files is not properly formatted" && exit 1)
    echo "🧹 Running linter checks..."
    cargo clippy -q -- -D warnings
    echo "🧪 Running tests..."
    cargo test -q

# Check that the current MSRV is correct
msrv:
    echo "🔧 Verifying MSRV..."
    cargo-msrv verify
    echo "✅ MSRV verification passed"

_done_ci:
    echo "🎉 CI pipeline completed successfully"

# Update the changelog
[script]
changelog:
    OLD_HASH=$(sha256sum CHANGELOG.md | head -c 64)
    git-cliff > CHANGELOG.md
    NEW_HASH=$(sha256sum CHANGELOG.md | head -c 64)
    if [[ $OLD_HASH != $NEW_HASH ]]; then
        git add CHANGELOG.md
        git commit -m 'chore(changelog): Update the changelog'
        echo 'The changes have been added to the changelog file and committed'
    else
        echo 'No changes have been added to the changelog'
    fi

# Releases a new version of n34. Requires a clean file tree with no uncommitted changes.
[script]
release version:
    set -e
    TAG_MSG="Version {{ version }}$(git-cliff --strip all --unreleased --body '{{ tag_change_body }}')"
    sed -i "s/^version\s*= \".*\"/version = \"{{ version }}\"/" ./Cargo.toml
    taplo fmt --config ./.taplo.toml ./Cargo.toml
    {{ JUST_EXECUTABLE }} ci
    git-cliff -t "v{{ version }}" > CHANGELOG.md
    git add .
    git commit -m 'chore: Bump the version to `v{{ version }}`'
    git tag -s -m "$TAG_MSG" "v{{ version }}"
    git push origin master --tags
    cargo publish
