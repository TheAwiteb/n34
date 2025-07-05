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

/// `patch apply` suubcommand
mod apply;
/// `patch close` subcommand
mod close;
/// `patch draft` subcommand
mod draft;
/// `patch fetch` subcommand
mod fetch;
/// `patch list` subcommand
mod list;
/// `patch merge` subcommand
mod merge;
/// `patch reopen` subcommand
mod reopen;
/// `patch send` subcommand
mod send;

use std::{
    fmt,
    path::{Path, PathBuf},
    str::FromStr,
    sync::LazyLock,
};

use clap::Subcommand;
use nostr::event::Kind;
use regex::Regex;

use self::apply::ApplyArgs;
use self::close::CloseArgs;
use self::draft::DraftArgs;
use self::fetch::FetchArgs;
use self::list::ListArgs;
use self::merge::MergeArgs;
use self::reopen::ReopenArgs;
use self::send::SendArgs;
use super::{CliOptions, CommandRunner};
use crate::error::{N34Error, N34Result};

/// Regular expression for extracting the patch subject.
static SUBJECT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^Subject: (.*(?:\n .*)*)").unwrap());

/// Regular expression for extracting the patch body.
static BODY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\n\n((?:.|\n)*?)(?:\n--[ -]|\z)").unwrap());

/// Regular expiration for extracting the patch version and number
static PATCH_VERSION_NUMBER_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[PATCH\s+(?:v(?<version>\d+)\s*)?(?<number>\d+)/(?:\d+)").unwrap()
});

#[derive(Subcommand, Debug)]
pub enum PatchSubcommands {
    /// Send one or more patches to a repository.
    Send(SendArgs),
    /// Fetches a patch by its id.
    Fetch(FetchArgs),
    /// Closes an open or drafted patch.
    Close(CloseArgs),
    /// Converts the closed or open patch to draft state.
    Draft(DraftArgs),
    /// Reopens a closed or drafted patch.
    Reopen(ReopenArgs),
    /// Set an open patch status to applied.
    Apply(ApplyArgs),
    /// Set an open patch status to merged.
    Merge(MergeArgs),
    /// List the repositories patches.
    List(ListArgs),
}

/// Represents a git patch
#[derive(Clone, Debug)]
pub struct GitPatch {
    /// Full content of the patch file
    pub inner:   String,
    /// Short description of the patch changes
    pub subject: String,
    /// Detailed explanation of the patch changes
    pub body:    String,
}

#[derive(Debug)]
pub enum PatchStatus {
    /// The patch is currently open
    Open,
    /// The patch has been merged/applied
    MergedApplied,
    /// The patch has been closed
    Closed,
    /// A patch that has been drafted but not yet applied.
    Draft,
}

impl PatchStatus {
    /// Maps the issue status to its corresponding Nostr kind.
    #[inline]
    pub fn kind(&self) -> Kind {
        match self {
            Self::Open => Kind::GitStatusOpen,
            Self::MergedApplied => Kind::GitStatusApplied,
            Self::Closed => Kind::GitStatusClosed,
            Self::Draft => Kind::GitStatusDraft,
        }
    }

    /// Returns the string representation of the patch status.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::MergedApplied => "Merged/Applied",
            Self::Closed => "Closed",
            Self::Draft => "Draft",
        }
    }

    /// Check if the patch is open.
    #[inline]
    pub fn is_open(&self) -> bool {
        matches!(self, Self::Open)
    }

    /// Check if the patch is merged/applied.
    #[inline]
    pub fn is_merged_or_applied(&self) -> bool {
        matches!(self, Self::MergedApplied)
    }

    /// Check if the patch is closed.
    #[inline]
    pub fn is_closed(&self) -> bool {
        matches!(self, Self::Closed)
    }

    /// Check if the patch is drafted
    #[inline]
    pub fn is_drafted(&self) -> bool {
        matches!(self, Self::Draft)
    }
}

impl From<&PatchStatus> for Kind {
    fn from(status: &PatchStatus) -> Self {
        status.kind()
    }
}

impl fmt::Display for PatchStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}


impl TryFrom<Kind> for PatchStatus {
    type Error = N34Error;

    fn try_from(kind: Kind) -> Result<Self, Self::Error> {
        match kind {
            Kind::GitStatusOpen => Ok(Self::Open),
            Kind::GitStatusApplied => Ok(Self::MergedApplied),
            Kind::GitStatusClosed => Ok(Self::Closed),
            Kind::GitStatusDraft => Ok(Self::Draft),
            _ => Err(N34Error::InvalidPatchStatus(kind)),
        }
    }
}

impl GitPatch {
    /// Returns the patch file name from the subject
    pub fn filename(&self, parent: impl AsRef<Path>) -> N34Result<PathBuf> {
        let (patch_version, patch_number) = if self.subject.contains("[PATCH]") {
            (String::new(), "1")
        } else {
            patch_version_and_subject(&self.subject)?
        };

        let patch_name = if patch_number == "0" {
            "cover-letter".to_owned()
        } else {
            patch_file_name(&self.subject)?
        };

        Ok(parent
            .as_ref()
            .join(format!("{patch_version}{patch_number:0>4}-{patch_name}").replace("--", "-"))
            .with_extension("patch"))
    }
}

impl CommandRunner for PatchSubcommands {
    async fn run(self, options: CliOptions) -> N34Result<()> {
        crate::run_command!(self, options, & Send Fetch Close Reopen Draft Apply Merge List)
    }
}

impl FromStr for GitPatch {
    type Err = String;

    fn from_str(patch_content: &str) -> Result<Self, Self::Err> {
        // Regex for subject (handles multi-line subjects)
        let subject = SUBJECT_RE
            .captures(patch_content)
            .and_then(|cap| cap.get(1))
            .ok_or("No subject found")?
            .as_str()
            .trim()
            .replace('\n', "")
            .to_string();

        // Regex for body
        let body = BODY_RE
            .captures(patch_content)
            .and_then(|cap| cap.get(1))
            .ok_or("No body found")?
            .as_str()
            .trim()
            .to_string();
        Ok(Self {
            inner: patch_content.to_owned(),
            subject,
            body,
        })
    }
}

/// Extracts the version prefix and patch number from a patch subject string.
///
/// The version prefix is formatted as "v{version}-" if present, or an empty
/// string. The patch number is mandatory and will cause an error if not found.
fn patch_version_and_subject(subject: &str) -> N34Result<(String, &str)> {
    let captures = PATCH_VERSION_NUMBER_RE.captures(subject).ok_or_else(|| {
        N34Error::InvalidEvent(format!("Can not parse the patch subject `{subject}`"))
    })?;
    Ok((
        captures
            .name("version")
            .map(|m| format!("v{}-", m.as_str()))
            .unwrap_or_default(),
        captures
            .name("number")
            .map(|m| m.as_str())
            .expect("It's not optional, regex will fail if it's not found"),
    ))
}

/// Extracts a clean file name from the patch subject by removing version info
/// and special characters. Converts to lowercase and ensures the name only
/// contains alphanumeric, '.', '-', or '_' characters.
fn patch_file_name(subject: &str) -> N34Result<String> {
    Ok(subject
        .split_once("]")
        .ok_or_else(|| {
            N34Error::InvalidEvent(format!(
                "Invalid patch subject. No `[PATCH ...]`: `{subject}`",
            ))
        })?
        .1
        .trim()
        .to_lowercase()
        .replace(
            |c: char| !c.is_ascii_alphanumeric() && !['.', '-', '_'].contains(&c),
            "-",
        )
        .chars()
        .take(60)
        .collect::<String>()
        .trim_matches('-')
        .trim()
        .replace("--", "-"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patch_normal() {
        let patch_content = r#"From 24e8522268ad675996fc3b35209ce23951236bdc Mon Sep 17 00:00:00 2001
From: Awiteb <a@4rs.nl>
Date: Tue, 27 May 2025 19:20:42 +0000
Subject: [PATCH] chore: a to abc

Abc patch
---
 src/nostr_utils/mod.rs            |  1 +
 1files changed, 3 insertions(+), 1 deletions(-)

diff --git a/src/nostr_utils/mod.rs b/src/nostr_utils/mod.rs
index 4120f5a..e68783c 100644
--- a/src/nostr_utils/mod.rs
+++ b/src/nostr_utils/mod.rs
@@ -103,31 +103,9 @@ impl CommandRunner for NewArgs {

- a
+ abc
-- 
2.49.0"#;
        let patch = GitPatch::from_str(patch_content).unwrap();
        assert_eq!(patch.subject, "[PATCH] chore: a to abc");
        assert_eq!(patch.body, "Abc patch");
    }

    #[test]
    fn patch_normal_with_patch_in_content() {
        let patch_content = r#"From 24e8522268ad675996fc3b35209ce23951236bdc Mon Sep 17 00:00:00 2001
From: Awiteb <a@4rs.nl>
Date: Tue, 27 May 2025 19:20:42 +0000
Subject: [PATCH] chore: Subject in subject

A good test patch
---
 src/nostr_utils/mod.rs            |  1 +
 1files changed, 3 insertions(+), 1 deletions(-)

diff --git a/src/nostr_utils/mod.rs b/src/nostr_utils/mod.rs
index 4120f5a..e68783c 100644
--- a/src/nostr_utils/mod.rs
+++ b/src/nostr_utils/mod.rs
@@ -103,31 +103,9 @@ impl CommandRunner for NewArgs {

From: Awiteb <a@4rs.nl>
Date: Tue, 27 May 2025 19:20:42 +0000
Subject: [PATCH] chore: What a subject

hi
---
-- 
2.49.0"#;
        let patch = GitPatch::from_str(patch_content).unwrap();
        assert_eq!(patch.subject, "[PATCH] chore: Subject in subject");
        assert_eq!(patch.body, "A good test patch");
    }

    #[test]
    fn patch_multiline_subject() {
        let patch_content = r#"From 24e8522268ad675996fc3b35209ce23951236bdc Mon Sep 17 00:00:00 2001
From: Awiteb <a@4rs.nl>
Date: Tue, 27 May 2025 19:20:42 +0000
Subject: [PATCH] chore: Some long subject yes so long one Some long subject yes
 so long one

Abc patch
---
 src/nostr_utils/mod.rs            |  1 +
 1files changed, 3 insertions(+), 1 deletions(-)

diff --git a/src/nostr_utils/mod.rs b/src/nostr_utils/mod.rs
index 4120f5a..e68783c 100644
--- a/src/nostr_utils/mod.rs
+++ b/src/nostr_utils/mod.rs
@@ -103,31 +103,9 @@ impl CommandRunner for NewArgs {

- a
+ abc
-- 
2.49.0"#;
        let patch = GitPatch::from_str(patch_content).unwrap();
        assert_eq!(
            patch.subject,
            "[PATCH] chore: Some long subject yes so long one Some long subject yes so long one"
        );
        assert_eq!(patch.body, "Abc patch");
    }

    #[test]
    fn patch_multiline_body() {
        let patch_content = r#"From 24e8522268ad675996fc3b35209ce23951236bdc Mon Sep 17 00:00:00 2001
From: Awiteb <a@4rs.nl>
Date: Tue, 27 May 2025 19:20:42 +0000
Subject: [PATCH] chore: a to abc

Lorem ipsum dolor sit amet. 33 laborum galisum aut fugiat dicta vel accusamus
aliquam vel quisquam fuga in incidunt voluptas a aliquid neque ab iure pariatur.
Et molestiae vero a consectetur laborum et accusantium sequi. Et ratione
atque et molestiae dolorem in asperiores amet id dolor corporis in adipisci
aspernatur.
---
 src/nostr_utils/mod.rs            |  1 +
 1files changed, 3 insertions(+), 1 deletions(-)

diff --git a/src/nostr_utils/mod.rs b/src/nostr_utils/mod.rs
index 4120f5a..e68783c 100644
--- a/src/nostr_utils/mod.rs
+++ b/src/nostr_utils/mod.rs
@@ -103,31 +103,9 @@ impl CommandRunner for NewArgs {

- a
+ abc
-- 
2.49.0"#;
        let patch = GitPatch::from_str(patch_content).unwrap();
        assert_eq!(patch.subject, "[PATCH] chore: a to abc");
        assert_eq!(
            patch.body,
            "Lorem ipsum dolor sit amet. 33 laborum galisum aut fugiat dicta vel accusamus
aliquam vel quisquam fuga in incidunt voluptas a aliquid neque ab iure pariatur.
Et molestiae vero a consectetur laborum et accusantium sequi. Et ratione
atque et molestiae dolorem in asperiores amet id dolor corporis in adipisci
aspernatur."
        );
    }

    #[test]
    fn patch_cover_letter() {
        let patch_content = r#"From 864f3018f62ab2e1265edb670d5493dafe7d2cb2 Mon Sep 17 00:00:00 2001
From: Awiteb <a@4rs.nl>
Date: Tue, 3 Jun 2025 08:41:12 +0000
Subject: [PATCH v2 0/7] feat: Some test just a test

Cover body

Awiteb (1):
  chore: Update `README.md`

 README.md      |  2 +-


base-commit: f670859b92d525874fd621452080c8479964ac6a
-- 
2.49.0"#;
        let patch = GitPatch::from_str(patch_content).unwrap();
        assert_eq!(patch.subject, "[PATCH v2 0/7] feat: Some test just a test");
        assert_eq!(
            patch.body,
            "Cover body

Awiteb (1):
  chore: Update `README.md`

 README.md      |  2 +-


base-commit: f670859b92d525874fd621452080c8479964ac6a"
        );
    }

    #[test]
    fn patch_multiline_cover_subject() {
        let patch_content = r#"From 864f3018f62ab2e1265edb670d5493dafe7d2cb2 Mon Sep 17 00:00:00 2001
From: Awiteb <a@4rs.nl>
Date: Tue, 3 Jun 2025 08:41:12 +0000
Subject: [PATCH v2 0/7] feat: Some test just a test some test just a test some
 test just a test

Cover body

Awiteb (1):
  chore: Update `README.md`

 README.md      |  2 +-


base-commit: f670859b92d525874fd621452080c8479964ac6a
-- 
2.49.0"#;
        let patch = GitPatch::from_str(patch_content).unwrap();
        assert_eq!(
            patch.subject,
            "[PATCH v2 0/7] feat: Some test just a test some test just a test some test just a \
             test"
        );
        assert_eq!(
            patch.body,
            "Cover body

Awiteb (1):
  chore: Update `README.md`

 README.md      |  2 +-


base-commit: f670859b92d525874fd621452080c8479964ac6a"
        );
    }

    #[test]
    fn patch_multiline_cover_body() {
        let patch_content = r#"From 864f3018f62ab2e1265edb670d5493dafe7d2cb2 Mon Sep 17 00:00:00 2001
From: Awiteb <a@4rs.nl>
Date: Tue, 3 Jun 2025 08:41:12 +0000
Subject: [PATCH v2 0/7] feat: Some test just a test some test just a test some
 test just a test

Lorem ipsum dolor sit amet. 33 laborum galisum aut fugiat dicta vel accusamus
aliquam vel quisquam fuga in incidunt voluptas a aliquid neque ab iure pariatur.
Et molestiae vero a consectetur laborum et accusantium sequi. Et ratione
atque et molestiae dolorem in asperiores amet id dolor corporis in adipisci
aspernatur.

Awiteb (1):
  chore: Update `README.md`

 README.md      |  2 +-


base-commit: f670859b92d525874fd621452080c8479964ac6a
-- 
2.49.0"#;
        let patch = GitPatch::from_str(patch_content).unwrap();
        assert_eq!(
            patch.subject,
            "[PATCH v2 0/7] feat: Some test just a test some test just a test some test just a \
             test"
        );
        assert_eq!(
            patch.body,
            "Lorem ipsum dolor sit amet. 33 laborum galisum aut fugiat dicta vel accusamus
aliquam vel quisquam fuga in incidunt voluptas a aliquid neque ab iure pariatur.
Et molestiae vero a consectetur laborum et accusantium sequi. Et ratione
atque et molestiae dolorem in asperiores amet id dolor corporis in adipisci
aspernatur.

Awiteb (1):
  chore: Update `README.md`

 README.md      |  2 +-


base-commit: f670859b92d525874fd621452080c8479964ac6a"
        );
    }

    #[test]
    fn normal_patch_filename() {
        let mut patch = GitPatch {
            inner:   String::new(),
            subject: String::new(),
            body:    String::new(),
        };

        patch.subject = "[PATCH v2 0/3] feat: Some test just a test".to_owned();
        assert_eq!(
            patch.filename("").unwrap(),
            PathBuf::from("v2-0000-cover-letter.patch")
        );
        patch.subject = "[PATCH 0/3] feat: Some test just a test".to_owned();
        assert_eq!(
            patch.filename("").unwrap(),
            PathBuf::from("0000-cover-letter.patch")
        );
        patch.subject = "[PATCH v2 1/3] feat: Some test just a test".to_owned();
        assert_eq!(
            patch.filename("").unwrap(),
            PathBuf::from("v2-0001-feat-some-test-just-a-test.patch")
        );
        patch.subject = "[PATCH v42 1/3] feat: Some test just a test".to_owned();
        assert_eq!(
            patch.filename("").unwrap(),
            PathBuf::from("v42-0001-feat-some-test-just-a-test.patch")
        );
        patch.subject = "[PATCH v42 23/30] feat: Some test just a test".to_owned();
        assert_eq!(
            patch.filename("").unwrap(),
            PathBuf::from("v42-0023-feat-some-test-just-a-test.patch")
        );
        patch.subject = "[PATCH 1/3] feat: Some test just a test".to_owned();
        assert_eq!(
            patch.filename("").unwrap(),
            PathBuf::from("0001-feat-some-test-just-a-test.patch")
        );
        patch.subject = "[PATCH 32/50] feat: Some test just a test".to_owned();
        assert_eq!(
            patch.filename("").unwrap(),
            PathBuf::from("0032-feat-some-test-just-a-test.patch")
        );
        patch.subject = "[PATCH v100 32/50] feat: some long subject some long subject some long \
                         subject some long subject"
            .to_owned();
        assert_eq!(
            patch.filename("").unwrap(),
            PathBuf::from(
                "v100-0032-feat-some-long-subject-some-long-subject-some-long-subject-s.patch"
            )
        );
    }

    #[test]
    fn patch_filename_without_patch() {
        let mut patch = GitPatch {
            inner:   String::new(),
            subject: "[RFC v5 1/2] Something".to_owned(),
            body:    String::new(),
        };

        assert!(patch.filename("").is_err());
        patch.subject = "Something".to_owned();
        assert!(patch.filename("").is_err());
    }

    #[test]
    fn patch_filename_without_number() {
        let mut patch = GitPatch {
            inner:   String::new(),
            subject: "[PATCH v5 /2] Something".to_owned(),
            body:    String::new(),
        };

        assert!(patch.filename("").is_err());
        patch.subject = "[PATCH v5 2/] Something".to_owned();
        assert!(patch.filename("").is_err());
    }

    #[test]
    fn patch_filename_without_version() {
        let patch = GitPatch {
            inner:   String::new(),
            subject: "[PATCH 1/2] Something".to_owned(),
            body:    String::new(),
        };

        assert!(patch.filename("").is_ok());
    }
}
