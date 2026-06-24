//! `cargo antigen pre-tag` — the **pre-tag release checkpoint** (the enforcement
//! arm of the non-cascade guarantee).
//!
//! # Why this exists
//!
//! antigen's CI gate (`cargo +nightly fmt --check`, `clippy -D warnings`,
//! `cargo test`, `cargo doc`, MSRV `cargo check`) runs on **push** — *after* the
//! release tag is already created and pushed. But the git tag is the **one
//! non-idempotent, human-recovery-required step** in the release: once
//! `v<version>` exists on the remote, undoing it is a manual, error-prone
//! intervention (force-delete the remote tag, hope no consumer already fetched
//! it). Everything else in the release (a bad bump, a stale CHANGELOG, missing
//! publish metadata) is re-runnable; the tag is not.
//!
//! `pre-tag` runs the consistency + tag-safety checks the CI gate runs **too
//! late** to catch — *before* the tag is cut. It is a **releng gate**, not a
//! user-facing scan: it answers one question — "is it safe to tag `v<version>`
//! right now?" — and exits non-zero if the answer is no.
//!
//! This command is the runnable defense for the
//! [`NonIdempotentReleaseStep`](antigen::stdlib::release_engineering::NonIdempotentReleaseStep)
//! stdlib antigen: the failure-class is "a release step not safe to re-run", and
//! the checks below ARE its immune memory made executable.
//!
//! # The check-set (grounded in how v0.6.0 actually released)
//!
//! Each check is a self-contained function returning a [`CheckResult`]. The
//! check-set is deliberately **not** a superset of CI — it is the complement:
//! the things CI cannot catch in time.
//!
//! 0. **Workspace members** — every `[workspace] members` crate is covered by
//!    `MEMBER_CRATES`. A new publishable member nobody hard-listed is invisible
//!    to every per-crate check below, so this runs first: it bounds their reach.
//! 1. **Version coherence** — the workspace `version`, every crate's internal
//!    `=<version>` path-dep pin, and the version-to-tag all agree. Manifests are
//!    PARSED (`toml_edit`), so formatting (no-space inline tables, dotted
//!    sub-tables, a whitespace-bearing `"= X"` pin) cannot smuggle a pin past.
//! 2. **No tag exists yet** — neither `git tag -l` nor `git ls-remote --tags
//!    origin` shows `v<version>`. THE headline check: the tag is the
//!    non-recoverable step, so an existing tag blocks hard.
//! 3. **CHANGELOG entry** — `CHANGELOG.md` has a real `## [<version>]` section
//!    (not an `UNRELEASED` placeholder) WITH a non-empty, non-`TODO`,
//!    non-comment body.
//! 4. **Publish metadata** — each crate resolves `description` / `license` /
//!    `repository` / `readme` to a NON-EMPTY value (a genuine
//!    `key.workspace = true` inheritance counts; `= false` and `= ""` do not).
//! 5. **README presence** — each crate has a `README.md` on disk.
//! 6. **Cargo.lock freshness** — the lock is fresh (`cargo metadata --locked`)
//!    AND committed (`git status --porcelain -- Cargo.lock` empty).
//! 7. **Local gate reminder** — print the CI gate to run by hand before tagging
//!    (this command does NOT reimplement CI; it points at it).
//!
//! # Subprocess discipline
//!
//! Git and cargo are invoked as **fixed-arg** subprocesses
//! (`std::process::Command::new("git")` / `"cargo"`), never user-interpolated
//! shells — the ADR-019 §4 bright-line (a named tool, fixed args, no user-code
//! exec). The git/cargo I/O lives here in the CLI, never in the `antigen` lib
//! (the `vcs_witness` pattern, ADR-002).

use std::path::Path;

/// The version-to-tag was not supplied, so `pre-tag` derives it from the
/// workspace `Cargo.toml`'s `[workspace.package] version`. This is the version
/// that would be tagged.
///
/// Read as the **source of truth** for the version-coherence check: every other
/// version string (the crate pins, the CHANGELOG header, the tag) must agree
/// with this one.
pub const WORKSPACE_TOML: &str = "Cargo.toml";

/// The five workspace member crates whose pins + publish-metadata are checked.
/// Hard-listed (not globbed) so adding a crate is a conscious pre-tag decision.
/// The hard-list is RECONCILED against the live `[workspace] members` by the
/// `workspace-members` check ([`workspace_members_absent_from_const`]): a new
/// publishable member that nobody added here FAILS the gate rather than being
/// silently skipped.
pub const MEMBER_CRATES: &[&str] = &[
    "antigen",
    "antigen-attestation",
    "antigen-fingerprint",
    "antigen-macros",
    "cargo-antigen",
];

/// The four publish-blocking metadata keys crates.io requires (or
/// workspace-inherits). A missing one is a `cargo publish` hard-blocker — the
/// failure-class [`CratesIoPublishBlockerMissingMetadata`](antigen::stdlib::release_engineering::CratesIoPublishBlockerMissingMetadata).
pub const REQUIRED_METADATA_KEYS: &[&str] = &["description", "license", "repository", "readme"];

/// One pre-tag check's verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckResult {
    /// Short stable check name (e.g. `"version-coherence"`), for the PASS/FAIL line.
    pub name: &'static str,
    /// `true` = PASS, `false` = FAIL (a FAIL makes the whole gate exit non-zero).
    pub passed: bool,
    /// Human-readable one-or-more lines explaining the verdict (what was checked,
    /// what was found). Always present — a PASS says what it verified, a FAIL says
    /// what to fix.
    pub detail: String,
}

impl CheckResult {
    /// A passing check.
    fn pass(name: &'static str, detail: impl Into<String>) -> Self {
        Self {
            name,
            passed: true,
            detail: detail.into(),
        }
    }

    /// A failing check.
    fn fail(name: &'static str, detail: impl Into<String>) -> Self {
        Self {
            name,
            passed: false,
            detail: detail.into(),
        }
    }
}

// ============================================================================
// Pure helpers (no I/O) — the testable core of each check.
// ============================================================================

/// Extract the `[workspace.package] version = "X"` from a workspace `Cargo.toml`
/// body. Returns `None` if the workspace-package version line is absent.
///
/// Intentionally a small hand-parser (not a full TOML parse): it reads the
/// `[workspace.package]` table and the FIRST bare `version = "..."` inside it.
/// The workspace root has exactly one such table; this avoids a TOML-deps churn
/// for a single-field read. (A crate's own `version.workspace = true` is NOT a
/// `version = "..."` literal, so it is never mistaken for the source.)
#[must_use]
pub fn parse_workspace_version(toml_body: &str) -> Option<String> {
    let mut in_workspace_package = false;
    for line in toml_body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            // A new table header. We are inside [workspace.package] only while
            // its header is the active one.
            in_workspace_package = trimmed == "[workspace.package]";
            continue;
        }
        if in_workspace_package {
            if let Some(v) = parse_version_literal(trimmed) {
                return Some(v);
            }
        }
    }
    None
}

/// Parse a `version = "X"` line into `X`. Returns `None` for any other line
/// (including `version.workspace = true`, which carries no literal).
fn parse_version_literal(line: &str) -> Option<String> {
    let rest = line.strip_prefix("version")?.trim_start();
    let rest = rest.strip_prefix('=')?.trim();
    let inner = rest.strip_prefix('"')?;
    let end = inner.find('"')?;
    Some(inner[..end].to_string())
}

/// The dependency tables a workspace member can carry an internal path-dep in.
/// (A `[target.'cfg(..)'.dependencies]` internal pin is out of v0.6.1 scope —
/// antigen's own crates are never target-conditional on each other.)
const DEP_TABLES: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];

/// Find each internal path-dep pin in a crate's `Cargo.toml` body and check it is
/// `=<expected>`. Returns the list of `(dep_name, found_version)` pins that
/// **disagree** with `expected` (empty = all coherent).
///
/// An internal pin is a dependency whose value is a table carrying a `path` that
/// points inside the workspace (`../...` or `crates/...`) — that `path` is what
/// marks it as an in-workspace pin (vs an external crates.io dep). Only those are
/// checked: external deps have their own versioning.
///
/// Parsed with [`toml_edit`] (not string-matching) so the check is robust to
/// formatting — a no-space inline table `{version="=X",path="../y"}`, a dotted
/// `[dependencies.y]` sub-table, and a whitespace-bearing pin `"= X"` all parse
/// to the same structure. The comparison normalizes whitespace inside the pin so
/// the valid Cargo exact-version `"= 0.6.1"` is recognized as equal to `"=0.6.1"`.
#[must_use]
pub fn internal_pin_mismatches(toml_body: &str, expected: &str) -> Vec<(String, String)> {
    let Ok(doc) = toml_body.parse::<toml_edit::DocumentMut>() else {
        // An unparseable manifest cannot be coherence-cleared; surface it as a
        // single sentinel mismatch rather than silently passing.
        return vec![("<manifest>".to_string(), "<unparseable TOML>".to_string())];
    };

    let want = normalize_pin(&format!("={expected}"));
    let mut mismatches = Vec::new();
    for table_name in DEP_TABLES {
        let Some(table) = doc.get(table_name).and_then(toml_edit::Item::as_table_like) else {
            continue;
        };
        for (dep_name, item) in table.iter() {
            let Some(path) = dep_path(item) else {
                continue; // not a path-dep → external, not coherence-checked
            };
            if !is_internal_path(path) {
                continue; // a path that escapes the workspace is not our pin
            }
            match dep_version(item) {
                Some(found) if normalize_pin(found) == want => {},
                Some(found) => mismatches.push((dep_name.to_string(), found.to_string())),
                None => {
                    // A path-dep with NO version pin at all is itself a mismatch
                    // (an unpinned internal dep can resolve to a stale published
                    // version on a fresh `cargo publish`).
                    mismatches.push((dep_name.to_string(), "<no version pin>".to_string()));
                },
            }
        }
    }
    mismatches
}

/// The `path = "..."` value of a dependency item, if it carries one (an inline
/// table or a dotted sub-table). `None` for a bare-string requirement or a dep
/// with no `path` key.
fn dep_value<'a>(item: &'a toml_edit::Item, key: &str) -> Option<&'a str> {
    item.as_table_like()?.get(key)?.as_str()
}

/// The `path` of a dependency item, if any.
fn dep_path(item: &toml_edit::Item) -> Option<&str> {
    dep_value(item, "path")
}

/// The `version` requirement of a dependency item, if any.
fn dep_version(item: &toml_edit::Item) -> Option<&str> {
    dep_value(item, "version")
}

/// Does a dep `path` point inside the workspace? A sibling-crate pin is `../name`;
/// a nested layout is `crates/name`. An absolute path or one escaping above the
/// workspace root is treated as external (not our coherence concern).
fn is_internal_path(path: &str) -> bool {
    let p = path.replace('\\', "/");
    p.starts_with("../") || (!p.starts_with('/') && !p.contains(':'))
}

/// Normalize a version-requirement string for comparison: strip ALL internal
/// whitespace so the valid Cargo exact-versions `"=0.6.1"` and `"= 0.6.1"` are
/// equal. (Cargo allows whitespace between the comparator and the version.)
fn normalize_pin(pin: &str) -> String {
    pin.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Does `CHANGELOG.md` carry a real `## [<version>]` section header?
///
/// A header of the shape `## [0.6.1]` (optionally followed by ` — <date>`)
/// counts. An `## [Unreleased]` / `## [UNRELEASED]` placeholder does NOT — a
/// release tagged against an Unreleased CHANGELOG ships an undated, unattributed
/// changelog.
#[must_use]
pub fn changelog_has_version_section(changelog_body: &str, version: &str) -> bool {
    let needle = format!("## [{version}]");
    changelog_body.lines().any(|line| {
        let t = line.trim_start();
        t.starts_with(&needle)
    })
}

/// Does the `## [<version>]` CHANGELOG section carry a real BODY — at least one
/// non-empty, non-comment line between its header and the next `## ` header (or
/// EOF)?
///
/// A header with an empty / `TODO`-only / comment-only body is a hollow entry: a
/// release tagged against it ships a changelog section that says nothing. The
/// header-only check ([`changelog_has_version_section`]) is necessary but not
/// sufficient; this is the body half.
#[must_use]
pub fn changelog_section_has_body(changelog_body: &str, version: &str) -> bool {
    let needle = format!("## [{version}]");
    let mut in_section = false;
    for line in changelog_body.lines() {
        let t = line.trim_start();
        if t.starts_with("## ") {
            if in_section {
                // Reached the next `## ` header without finding a real line.
                return false;
            }
            in_section = t.starts_with(&needle);
            continue;
        }
        if in_section && is_real_changelog_line(t) {
            return true;
        }
    }
    // Section ran to EOF — `in_section` true here means no real line was found.
    false
}

/// Is a CHANGELOG body line a *real* content line — not blank, not a `TODO`
/// placeholder, not an HTML comment?
fn is_real_changelog_line(trimmed: &str) -> bool {
    if trimmed.is_empty() {
        return false;
    }
    // A bare TODO placeholder (any case) is not real content.
    let upper = trimmed.to_ascii_uppercase();
    if upper == "TODO" || upper == "TODO." || upper == "TBD" {
        return false;
    }
    // An HTML comment line (`<!-- ... -->`) is scaffolding, not content.
    if trimmed.starts_with("<!--") {
        return false;
    }
    // A Markdown link-reference definition (`[label]: url`) is a hyperlink target
    // some changelog tools auto-emit under the version header — it carries zero
    // release notes, so a section body of only link-refs is hollow (DEFECT-ADV-1).
    if trimmed.starts_with('[') && trimmed.contains("]: ") {
        return false;
    }
    true
}

/// The crate-dir names declared in `[workspace] members` that are **absent** from
/// [`MEMBER_CRATES`]. A non-empty result is a release hazard: a new publishable
/// member that nobody added to the hard-list is invisible to every per-crate
/// check (pin coherence, publish metadata, README presence).
///
/// Each member entry reconciles by its final path component — the crate-DIR name
/// the per-crate checks `root.join(member)` against. A nested `crates/foo` member
/// reconciles as `foo`.
#[must_use]
pub fn workspace_members_absent_from_const(workspace_toml_body: &str) -> Vec<String> {
    let Ok(doc) = workspace_toml_body.parse::<toml_edit::DocumentMut>() else {
        return vec!["<unparseable workspace Cargo.toml>".to_string()];
    };
    let Some(members) = doc
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(toml_edit::Item::as_array)
    else {
        return Vec::new();
    };
    members
        .iter()
        .filter_map(toml_edit::Value::as_str)
        .filter_map(|m| m.rsplit(['/', '\\']).next())
        .filter(|name| !MEMBER_CRATES.contains(name))
        .map(str::to_string)
        .collect()
}

/// Given the version-to-tag and the full set of tag names known locally + on the
/// remote, does the `v<version>` tag already exist anywhere?
///
/// Pure over the tag set so the headline tag-exists check is unit-testable
/// without a live git remote.
#[must_use]
pub fn tag_already_exists<'a>(version: &str, tags: impl IntoIterator<Item = &'a str>) -> bool {
    let needle = format!("v{version}");
    tags.into_iter().any(|t| t.trim() == needle)
}

/// Which of the [`REQUIRED_METADATA_KEYS`] are unresolved for a crate, given its
/// own `Cargo.toml` body. A key counts as resolved only if its `[package]` value
/// is genuinely publishable: a **non-empty** literal (`description = "x"`) OR a
/// genuine workspace inheritance (`license.workspace = true` — `= false` means
/// NOT inherited, so the key resolves to absent).
///
/// Parsed with [`toml_edit`] so a present-but-empty (`description = ""`) value
/// and a `key.workspace = false` opt-out are both correctly read as unresolved —
/// the two false-passes the string-matching version waved through.
///
/// Returns the list of MISSING keys (empty = all four resolve).
#[must_use]
pub fn missing_metadata_keys(crate_toml_body: &str) -> Vec<&'static str> {
    let Ok(doc) = crate_toml_body.parse::<toml_edit::DocumentMut>() else {
        // Unparseable → nothing can be confirmed resolved; report all missing.
        return REQUIRED_METADATA_KEYS.to_vec();
    };
    let package = doc.get("package").and_then(toml_edit::Item::as_table_like);
    REQUIRED_METADATA_KEYS
        .iter()
        .copied()
        .filter(|key| !metadata_key_resolved(package, key))
        .collect()
}

/// Is one metadata `key` resolved in a crate's parsed `[package]` table — either a
/// **non-empty** literal value or a genuine `key.workspace = true` inheritance?
///
/// `toml_edit` exposes `description.workspace = true` as a dotted-key sub-table
/// `{ workspace = true }` under `description`; a plain literal is a string value.
/// An empty string, an all-whitespace string, or `workspace = false` are each
/// **not** resolved.
fn metadata_key_resolved(package: Option<&dyn toml_edit::TableLike>, key: &str) -> bool {
    let Some(item) = package.and_then(|p| p.get(key)) else {
        return false;
    };
    // Workspace inheritance: `key.workspace = true` (and ONLY `= true`).
    if let Some(ws) = item.as_table_like().and_then(|t| t.get("workspace")) {
        return ws.as_bool() == Some(true);
    }
    // A literal value: resolved iff it is a non-empty, non-whitespace string.
    // (A non-string literal like a `readme = false` "no readme" marker is not a
    // publishable value either, so it correctly reads as unresolved.)
    item.as_str().is_some_and(|s| !s.trim().is_empty())
}

// ============================================================================
// I/O-bearing checks (read files / shell git + cargo) — thin wrappers over the
// pure helpers above.
// ============================================================================

/// Run all checks at `root` for the version-to-tag, returning each verdict.
///
/// If `version` is `None`, the version-to-tag is derived from the workspace
/// `Cargo.toml`. A check that cannot even establish the version-to-tag yields a
/// single FAIL (everything downstream needs the version).
#[must_use]
pub fn run_checks(root: &Path, version: Option<&str>) -> Vec<CheckResult> {
    // Resolve the version-to-tag first — every other check is relative to it.
    let workspace_toml_path = root.join(WORKSPACE_TOML);
    let workspace_toml = match std::fs::read_to_string(&workspace_toml_path) {
        Ok(b) => b,
        Err(e) => {
            return vec![CheckResult::fail(
                "version-coherence",
                format!(
                    "cannot read workspace {}: {e} (is --root a workspace root?)",
                    workspace_toml_path.display()
                ),
            )];
        },
    };

    let Some(resolved_version) = version
        .map(str::to_string)
        .or_else(|| parse_workspace_version(&workspace_toml))
    else {
        return vec![CheckResult::fail(
            "version-coherence",
            "could not find [workspace.package] version in Cargo.toml and none was supplied \
             with --version",
        )];
    };

    vec![
        check_workspace_members(&workspace_toml),
        check_version_coherence(root, &workspace_toml, &resolved_version),
        check_no_tag_exists(root, &resolved_version),
        check_changelog(root, &resolved_version),
        check_publish_metadata(root),
        check_readme_presence(root),
        check_lockfile_fresh(root),
        check_local_gate_reminder(),
    ]
}

/// Check 0 — every `[workspace] members` crate is in [`MEMBER_CRATES`]. A new
/// publishable member that nobody hard-listed is invisible to every per-crate
/// check below (pin coherence, publish metadata, README presence) — so it could
/// ship with a stale pin / missing metadata / no README and the gate would never
/// look at it. Runs first: it bounds the coverage of everything that follows.
fn check_workspace_members(workspace_toml: &str) -> CheckResult {
    const NAME: &str = "workspace-members";
    let unlisted = workspace_members_absent_from_const(workspace_toml);
    if unlisted.is_empty() {
        CheckResult::pass(
            NAME,
            format!(
                "all `[workspace] members` are covered by MEMBER_CRATES ({} crates)",
                MEMBER_CRATES.len()
            ),
        )
    } else {
        CheckResult::fail(
            NAME,
            format!(
                "workspace member(s) NOT in pre-tag's MEMBER_CRATES — they are invisible to the \
                 per-crate pin / metadata / README checks (add them to MEMBER_CRATES in \
                 cargo-antigen/src/pre_tag.rs before tagging):\n    - {}",
                unlisted.join("\n    - ")
            ),
        )
    }
}

/// Check 1 — version coherence: workspace version == every internal pin == the
/// version-to-tag.
fn check_version_coherence(root: &Path, workspace_toml: &str, version: &str) -> CheckResult {
    const NAME: &str = "version-coherence";

    // The workspace version itself must agree with the version-to-tag.
    let ws_version = parse_workspace_version(workspace_toml);
    if let Some(ws) = &ws_version {
        if ws != version {
            return CheckResult::fail(
                NAME,
                format!(
                    "workspace version is {ws} but the version-to-tag is {version} — they must agree"
                ),
            );
        }
    }

    // Each crate's internal `=<version>` pins must match.
    let mut all_mismatches: Vec<String> = Vec::new();
    for member in MEMBER_CRATES {
        let path = root.join(member).join("Cargo.toml");
        let Ok(body) = std::fs::read_to_string(&path) else {
            all_mismatches.push(format!("{member}: cannot read {}", path.display()));
            continue;
        };
        for (dep, found) in internal_pin_mismatches(&body, version) {
            all_mismatches.push(format!(
                "{member}: {dep} pinned `{found}`, expected `={version}`"
            ));
        }
    }

    if all_mismatches.is_empty() {
        CheckResult::pass(
            NAME,
            format!(
                "workspace version {version} == all internal `=`-pins across {} crates",
                MEMBER_CRATES.len()
            ),
        )
    } else {
        CheckResult::fail(
            NAME,
            format!(
                "version disagreement(s):\n    - {}",
                all_mismatches.join("\n    - ")
            ),
        )
    }
}

/// Check 2 — the headline check: no `v<version>` tag exists locally OR on the
/// remote. The tag is the non-recoverable step; an existing tag blocks hard.
fn check_no_tag_exists(root: &Path, version: &str) -> CheckResult {
    const NAME: &str = "no-tag-exists";
    let tag = format!("v{version}");

    // Local tags (`git tag -l v<version>` prints the tag iff it exists). The
    // exact-match decision is the pure [`tag_already_exists`] helper (no prefix
    // match — `v0.6.1` must not be blocked by `v0.6.10`).
    let local = git_lines(root, &["tag", "-l", &tag]);
    let local_has = local
        .as_ref()
        .is_some_and(|lines| tag_already_exists(version, lines.iter().map(String::as_str)));

    // Remote tags (`git ls-remote --tags origin v<version>`). The output is
    // `<sha>\trefs/tags/<tag>` lines; reduce each to its `refs/tags/<name>` tail
    // so the same exact-match helper decides. A network failure yields None —
    // surfaced as a soft FAIL (cannot confirm the remote is clean, and "cannot
    // confirm" must not read as "confirmed clean" for the non-recoverable step).
    let remote = git_lines(root, &["ls-remote", "--tags", "origin", &tag]);
    let remote_has = remote.as_ref().is_some_and(|lines| {
        // Keep only genuine `refs/tags/<name>` lines and reduce each to its
        // `<name>` tail; a line without `refs/tags/` is not a tag ref and is
        // dropped (so it can never spuriously satisfy the exact-match helper).
        let names: Vec<&str> = lines
            .iter()
            .filter_map(|l| l.split_once("refs/tags/").map(|(_, name)| name))
            .collect();
        tag_already_exists(version, names.iter().copied())
    });

    match (local_has, remote_has, remote.is_some()) {
        (false, false, true) => CheckResult::pass(
            NAME,
            format!("no `{tag}` tag exists locally or on origin — safe to tag"),
        ),
        (true, ..) => CheckResult::fail(
            NAME,
            format!(
                "`{tag}` ALREADY EXISTS locally — the tag is the non-idempotent step; \
                 delete it (`git tag -d {tag}`) or bump the version before re-running"
            ),
        ),
        (false, true, _) => CheckResult::fail(
            NAME,
            format!(
                "`{tag}` ALREADY EXISTS on origin — recovery requires force-deleting the remote \
                 tag (`git push origin :refs/tags/{tag}`); STOP and coordinate before re-tagging"
            ),
        ),
        (false, false, false) => CheckResult::fail(
            NAME,
            format!(
                "could not query origin for `{tag}` (network/remote error) — cannot confirm the \
                 remote is tag-free; resolve connectivity and re-run rather than tag blind \
                 (the tag is non-recoverable)"
            ),
        ),
    }
}

/// Check 3 — CHANGELOG has a real `## [<version>]` section (not Unreleased).
fn check_changelog(root: &Path, version: &str) -> CheckResult {
    const NAME: &str = "changelog-entry";
    let path = root.join("CHANGELOG.md");
    let Ok(body) = std::fs::read_to_string(&path) else {
        return CheckResult::fail(
            NAME,
            format!(
                "cannot read {} — a release needs a dated CHANGELOG section",
                path.display()
            ),
        );
    };
    if !changelog_has_version_section(&body, version) {
        return CheckResult::fail(
            NAME,
            format!(
                "CHANGELOG.md has no `## [{version}]` section (an `Unreleased` placeholder does \
                 not count) — promote the section to the version + date before tagging"
            ),
        );
    }
    if !changelog_section_has_body(&body, version) {
        return CheckResult::fail(
            NAME,
            format!(
                "CHANGELOG.md `## [{version}]` section is empty (no content before the next \
                 `## ` header — a blank / `TODO` / comment-only body does not count) — write the \
                 release notes before tagging"
            ),
        );
    }
    CheckResult::pass(
        NAME,
        format!("CHANGELOG.md has a `## [{version}]` section with real content"),
    )
}

/// Check 4 — each crate resolves all four publish-blocking metadata keys.
fn check_publish_metadata(root: &Path) -> CheckResult {
    const NAME: &str = "publish-metadata";
    let mut problems: Vec<String> = Vec::new();
    for member in MEMBER_CRATES {
        let path = root.join(member).join("Cargo.toml");
        let Ok(body) = std::fs::read_to_string(&path) else {
            problems.push(format!("{member}: cannot read {}", path.display()));
            continue;
        };
        let missing = missing_metadata_keys(&body);
        if !missing.is_empty() {
            problems.push(format!("{member}: missing {}", missing.join(", ")));
        }
    }
    if problems.is_empty() {
        CheckResult::pass(
            NAME,
            format!(
                "all {} crates resolve description / license / repository / readme",
                MEMBER_CRATES.len()
            ),
        )
    } else {
        CheckResult::fail(
            NAME,
            format!(
                "publish-blocking metadata missing:\n    - {}",
                problems.join("\n    - ")
            ),
        )
    }
}

/// Check 5 — each crate has a `README.md` on disk.
fn check_readme_presence(root: &Path) -> CheckResult {
    const NAME: &str = "readme-presence";
    let mut missing: Vec<String> = Vec::new();
    for member in MEMBER_CRATES {
        let readme = root.join(member).join("README.md");
        if !readme.is_file() {
            missing.push(format!("{member}/README.md"));
        }
    }
    if missing.is_empty() {
        CheckResult::pass(
            NAME,
            format!("all {} crates have a README.md", MEMBER_CRATES.len()),
        )
    } else {
        CheckResult::fail(
            NAME,
            format!(
                "missing README.md (a `readme = \"README.md\"` pointing at a nonexistent file \
                 renders a crates.io \"coming soon\" page):\n    - {}",
                missing.join("\n    - ")
            ),
        )
    }
}

/// Check 6 — `Cargo.lock` is fresh AND committed.
///
/// TWO independent conditions, because they fail differently:
///   - **fresh** — `cargo metadata --locked` succeeds (a stale lock means
///     `--locked` resolution would change it: a dirty release artifact).
///   - **committed** — `git status --porcelain -- Cargo.lock` is empty (a lock
///     that is fresh-on-disk but uncommitted/modified at tag time is a real
///     `NonIdempotentReleaseStep` hazard: the tagged tree omits the lock the
///     release was built against).
///
/// Git unavailability degrades honestly (per the no-tag-exists precedent): a
/// cannot-confirm-committed is a soft FAIL, never read as confirmed-clean — the
/// "committed" half of the claim must be earned, not assumed.
fn check_lockfile_fresh(root: &Path) -> CheckResult {
    const NAME: &str = "lockfile-fresh";

    // --- freshness (cargo) ---
    let out = std::process::Command::new("cargo")
        .current_dir(root)
        .args(["metadata", "--locked", "--format-version", "1", "--no-deps"])
        .output();
    match out {
        Ok(o) if o.status.success() => {},
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            let first = stderr
                .lines()
                .find(|l| !l.trim().is_empty())
                .unwrap_or("")
                .trim();
            return CheckResult::fail(
                NAME,
                format!(
                    "cargo metadata --locked failed — Cargo.lock is stale or uncommitted; run \
                     `cargo update --workspace` and commit the lock. cargo said: {first}"
                ),
            );
        },
        Err(e) => {
            return CheckResult::fail(
                NAME,
                format!("could not run `cargo metadata --locked`: {e}"),
            );
        },
    }

    // --- committed (git) --- fresh-on-disk is not the same as committed.
    // `git status --porcelain -- Cargo.lock` prints one line iff the lock is
    // modified/untracked/staged; empty output means it matches HEAD.
    match git_lines(root, &["status", "--porcelain", "--", "Cargo.lock"]) {
        Some(lines) if lines.iter().all(|l| l.trim().is_empty()) => CheckResult::pass(
            NAME,
            "Cargo.lock is fresh (cargo metadata --locked) AND committed (git status clean)",
        ),
        Some(lines) => {
            let status = lines
                .iter()
                .find(|l| !l.trim().is_empty())
                .map_or("", |l| l.trim());
            CheckResult::fail(
                NAME,
                format!(
                    "Cargo.lock is fresh but UNCOMMITTED (git status: `{status}`) — the tag would \
                     omit the lock the release was built against; stage + commit Cargo.lock \
                     before tagging"
                ),
            )
        },
        None => CheckResult::fail(
            NAME,
            "Cargo.lock is fresh, but git is unavailable / this is not a repo — cannot confirm \
             the lock is COMMITTED; resolve git access and re-run rather than tag on an \
             unverified lock (cannot-confirm is not confirmed-clean)",
        ),
    }
}

/// Check 7 — the local-gate reminder. This command does NOT reimplement CI; it
/// points at the exact gate to run by hand before tagging. Always PASSES (it is
/// informational) — the operator runs the gate; pre-tag only ensures the
/// tag-safety + consistency preconditions CI runs too late to catch.
fn check_local_gate_reminder() -> CheckResult {
    const NAME: &str = "local-gate-reminder";
    CheckResult::pass(
        NAME,
        "run the full CI gate locally before tagging (pre-tag does NOT reimplement it):\n    \
         $ cargo +nightly fmt --check\n    \
         $ cargo clippy --workspace --all-targets -- -D warnings\n    \
         $ cargo test --workspace\n    \
         $ cargo doc --workspace --no-deps\n    \
         $ cargo +1.95 check --workspace   # MSRV floor",
    )
}

/// Run a fixed-arg `git` subcommand at `root`, returning its stdout split into
/// trimmed lines on success, or `None` on any failure (git missing / non-zero
/// exit / not a repo). Fixed args only — the ADR-019 §4 bright-line.
fn git_lines(root: &Path, args: &[&str]) -> Option<Vec<String>> {
    let out = std::process::Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(str::to_string)
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- version-coherence pure core ----

    #[test]
    fn parse_workspace_version_reads_the_workspace_package_table() {
        let toml = "\
[workspace]\n\
members = [\"a\"]\n\
\n\
[workspace.package]\n\
version = \"0.6.1\"\n\
edition = \"2021\"\n";
        assert_eq!(parse_workspace_version(toml).as_deref(), Some("0.6.1"));
    }

    #[test]
    fn parse_workspace_version_ignores_a_crate_version_workspace_inherit() {
        // A crate's `version.workspace = true` carries no literal and must never
        // be mistaken for the workspace-package source version.
        let toml = "\
[package]\n\
name = \"antigen\"\n\
version.workspace = true\n";
        assert_eq!(parse_workspace_version(toml), None);
    }

    #[test]
    fn internal_pin_mismatch_is_caught() {
        // A crate pinned to the OLD version while the workspace bumped is the
        // pin-drift half of NonIdempotentReleaseStep — the publish would resolve
        // a stale internal dep.
        let body = "\
[dependencies]\n\
antigen-macros = { version = \"=0.6.0\", path = \"../antigen-macros\" }\n";
        let mismatches = internal_pin_mismatches(body, "0.6.1");
        assert_eq!(mismatches.len(), 1);
        assert_eq!(mismatches[0].0, "antigen-macros");
        assert_eq!(mismatches[0].1, "=0.6.0");
    }

    #[test]
    fn internal_pin_coherent_is_clean() {
        let body = "\
[dependencies]\n\
antigen-fingerprint = { version = \"=0.6.1\", path = \"../antigen-fingerprint\" }\n\
serde = { workspace = true }\n";
        assert!(internal_pin_mismatches(body, "0.6.1").is_empty());
    }

    #[test]
    fn internal_path_dep_without_a_pin_is_a_mismatch() {
        // A path-dep with no version is itself a hazard (resolves a stale
        // published version on a fresh publish).
        let body = "\
[dependencies]\n\
antigen = { path = \"../antigen\" }\n";
        let mismatches = internal_pin_mismatches(body, "0.6.1");
        assert_eq!(mismatches.len(), 1);
        assert_eq!(mismatches[0].1, "<no version pin>");
    }

    #[test]
    fn external_deps_are_not_treated_as_internal_pins() {
        // toml_edit / ureq are crates.io deps with their own versioning; only
        // `path = "../"` internal pins are coherence-checked.
        let body = "\
[dependencies]\n\
toml_edit = \"0.22\"\n\
ureq = \"2\"\n";
        assert!(internal_pin_mismatches(body, "0.6.1").is_empty());
    }

    // ---- tag-exists headline check pure core (born-red discipline) ----

    #[test]
    fn tag_already_exists_detects_the_exact_tag() {
        let tags = ["v0.5.0-beta.1", "v0.6.0"];
        assert!(tag_already_exists("0.6.0", tags));
        assert!(!tag_already_exists("0.6.1", tags));
    }

    #[test]
    fn tag_exists_does_not_prefix_match() {
        // `v0.6.1` must NOT be considered to exist just because `v0.6.10` or
        // `v0.6.1-beta` is in the set — exact match only (an over-eager prefix
        // match would block a legitimate tag).
        let tags = ["v0.6.10", "v0.6.1-beta.1"];
        assert!(!tag_already_exists("0.6.1", tags));
        assert!(tag_already_exists("0.6.10", tags));
    }

    #[test]
    fn tag_exists_trims_whitespace() {
        // `git tag -l` / `ls-remote` lines can carry trailing whitespace.
        let tags = ["v0.6.1  ", "  v0.5.0"];
        assert!(tag_already_exists("0.6.1", tags));
    }

    // ---- changelog check pure core ----

    #[test]
    fn changelog_version_section_is_recognized() {
        let body = "\
# Changelog\n\
\n\
## [0.6.1] — 2026-06-23\n\
\n\
### Added\n";
        assert!(changelog_has_version_section(body, "0.6.1"));
    }

    #[test]
    fn changelog_unreleased_placeholder_does_not_count() {
        let body = "\
# Changelog\n\
\n\
## [Unreleased]\n\
\n\
### Added\n";
        assert!(!changelog_has_version_section(body, "0.6.1"));
    }

    // ---- publish-metadata pure core ----

    #[test]
    fn workspace_inherited_metadata_counts_as_resolved() {
        // The v0.6.0 crates inherit description literally + license/repository
        // via `.workspace = true` + a literal readme — all four resolve.
        let body = "\
[package]\n\
name = \"antigen\"\n\
description = \"Structural memory of failure-classes.\"\n\
license.workspace = true\n\
repository.workspace = true\n\
readme = \"README.md\"\n";
        assert!(missing_metadata_keys(body).is_empty());
    }

    #[test]
    fn missing_metadata_keys_are_reported() {
        let body = "\
[package]\n\
name = \"halfbaked\"\n\
description = \"only a description\"\n";
        let missing = missing_metadata_keys(body);
        assert!(missing.contains(&"license"));
        assert!(missing.contains(&"repository"));
        assert!(missing.contains(&"readme"));
        assert!(!missing.contains(&"description"));
    }

    // ---- DEFECT 1 [FALSE-PASS HIGH]: empty-string metadata -----------------
    // `description = ""` (or license/repository) must NOT count as resolved — an
    // empty literal is present-but-publish-blocking. crates.io rejects an empty
    // description; the gate let it pass because the hand-parser only checked the
    // KEY was present, never that it carried a value.

    #[test]
    fn empty_string_metadata_does_not_count_as_resolved() {
        let body = "\
[package]\n\
name = \"hollow\"\n\
description = \"\"\n\
license = \"\"\n\
repository = \"\"\n\
readme = \"README.md\"\n";
        let missing = missing_metadata_keys(body);
        assert!(
            missing.contains(&"description"),
            "empty description = \"\" must be reported missing, got {missing:?}"
        );
        assert!(missing.contains(&"license"));
        assert!(missing.contains(&"repository"));
        // The non-empty readme literal still resolves.
        assert!(!missing.contains(&"readme"));
    }

    #[test]
    fn whitespace_only_metadata_does_not_count_as_resolved() {
        // A value of only spaces is as empty as "" for publish purposes.
        let body = "\
[package]\n\
description = \"   \"\n";
        assert!(missing_metadata_keys(body).contains(&"description"));
    }

    // ---- DEFECT 2 [FALSE-PASS HIGH]: no-space inline-table pin -------------
    // `{version="=0.6.0",path="../x"}` (no spaces) skipped version-coherence —
    // the hand-parser keyed on the literal `path = "../` (spaces). A real,
    // parsed manifest finds the path-dep regardless of formatting.

    #[test]
    fn no_space_inline_table_pin_is_still_checked() {
        let body = "\
[dependencies]\n\
antigen-macros = {version=\"=0.6.0\",path=\"../antigen-macros\"}\n";
        let mismatches = internal_pin_mismatches(body, "0.6.1");
        assert_eq!(
            mismatches.len(),
            1,
            "no-space inline-table pin must still be coherence-checked, got {mismatches:?}"
        );
        assert_eq!(mismatches[0].0, "antigen-macros");
        assert_eq!(mismatches[0].1, "=0.6.0");
    }

    #[test]
    fn no_space_inline_table_pin_coherent_is_clean() {
        let body = "\
[dependencies]\n\
antigen-fingerprint = {version=\"=0.6.1\",path=\"../antigen-fingerprint\"}\n";
        assert!(internal_pin_mismatches(body, "0.6.1").is_empty());
    }

    #[test]
    fn dotted_subtable_path_dep_is_checked() {
        // `[dependencies.antigen]` dotted-table form is also an internal pin.
        let body = "\
[dependencies.antigen]\n\
version = \"=0.6.0\"\n\
path = \"../antigen\"\n";
        let mismatches = internal_pin_mismatches(body, "0.6.1");
        assert_eq!(mismatches.len(), 1);
        assert_eq!(mismatches[0].0, "antigen");
        assert_eq!(mismatches[0].1, "=0.6.0");
    }

    // ---- DEFECT 4 [FALSE-PASS MED]: `key.workspace = false` ---------------
    // `license.workspace = false` means NOT inherited (the key resolves to
    // absent), yet the substring check `license.workspace` counted it resolved.

    #[test]
    fn workspace_false_inheritance_does_not_count_as_resolved() {
        let body = "\
[package]\n\
name = \"optout\"\n\
description = \"real\"\n\
license.workspace = false\n\
repository.workspace = true\n\
readme = \"README.md\"\n";
        let missing = missing_metadata_keys(body);
        assert!(
            missing.contains(&"license"),
            "license.workspace = false is NOT inherited → license must read missing, got {missing:?}"
        );
        // The genuinely-inherited repository still resolves.
        assert!(!missing.contains(&"repository"));
    }

    // ---- DEFECT 6 [FALSE-BLOCK MED]: `= 0.6.1` (space after =) ------------
    // `version = "= 0.6.1"` is a VALID Cargo exact-version (whitespace after the
    // operator is allowed). The gate wrongly reported a mismatch because it
    // compared the raw string `= 0.6.1` against `=0.6.1`.

    #[test]
    fn pin_with_space_after_equals_is_coherent_not_a_mismatch() {
        let body = "\
[dependencies]\n\
antigen-macros = { version = \"= 0.6.1\", path = \"../antigen-macros\" }\n";
        assert!(
            internal_pin_mismatches(body, "0.6.1").is_empty(),
            "`= 0.6.1` (space after =) is a valid exact pin equal to `=0.6.1`"
        );
    }

    #[test]
    fn pin_with_space_after_equals_still_catches_a_real_drift() {
        // The whitespace-normalization must not mask a genuine version drift.
        let body = "\
[dependencies]\n\
antigen-macros = { version = \"= 0.6.0\", path = \"../antigen-macros\" }\n";
        let mismatches = internal_pin_mismatches(body, "0.6.1");
        assert_eq!(mismatches.len(), 1);
        assert_eq!(mismatches[0].0, "antigen-macros");
    }

    // ---- DEFECT 5 [FALSE-PASS MED]: empty CHANGELOG section ---------------
    // `## [0.6.1]` with an empty / TODO / comment-only body passed the
    // header-only check. The section BODY must carry ≥1 real line.

    #[test]
    fn changelog_section_with_real_body_passes() {
        let body = "\
# Changelog\n\
\n\
## [0.6.1] — 2026-06-23\n\
\n\
### Added\n\
- the self/non-self axis\n\
\n\
## [0.6.0]\n";
        assert!(changelog_section_has_body(body, "0.6.1"));
    }

    #[test]
    fn changelog_empty_section_body_fails() {
        // Header present, body empty until the next `## ` — a hollow entry.
        let body = "\
# Changelog\n\
\n\
## [0.6.1]\n\
\n\
## [0.6.0]\n\
- real content here\n";
        assert!(
            !changelog_section_has_body(body, "0.6.1"),
            "an empty `## [0.6.1]` body must NOT pass"
        );
    }

    #[test]
    fn changelog_todo_only_section_body_fails() {
        let body = "\
## [0.6.1]\n\
TODO\n";
        assert!(
            !changelog_section_has_body(body, "0.6.1"),
            "a `TODO`-only body is not a real changelog section"
        );
    }

    #[test]
    fn changelog_comment_only_section_body_fails() {
        let body = "\
## [0.6.1]\n\
<!-- fill me in -->\n";
        assert!(
            !changelog_section_has_body(body, "0.6.1"),
            "an HTML-comment-only body is not a real changelog section"
        );
    }

    #[test]
    fn changelog_section_at_eof_with_body_passes() {
        // The last section has no following `## ` — body runs to EOF.
        let body = "\
## [0.6.1]\n\
- shipped\n";
        assert!(changelog_section_has_body(body, "0.6.1"));
    }

    // ---- DEFECT 3 [FALSE-PASS HIGH]: new workspace member invisible -------
    // A new publishable member added to `[workspace] members` but NOT to
    // MEMBER_CRATES is invisible to every per-crate check. Reconcile and FAIL.

    #[test]
    fn unlisted_workspace_member_is_detected() {
        let toml = "\
[workspace]\n\
members = [\n\
    \"antigen\",\n\
    \"antigen-attestation\",\n\
    \"antigen-fingerprint\",\n\
    \"antigen-macros\",\n\
    \"cargo-antigen\",\n\
    \"antigen-newcomer\",\n\
]\n";
        let unlisted = workspace_members_absent_from_const(toml);
        assert_eq!(
            unlisted,
            vec!["antigen-newcomer".to_string()],
            "a workspace member not in MEMBER_CRATES must be surfaced"
        );
    }

    #[test]
    fn all_members_listed_is_clean() {
        let toml = "\
[workspace]\n\
members = [\n\
    \"antigen\",\n\
    \"antigen-attestation\",\n\
    \"antigen-fingerprint\",\n\
    \"antigen-macros\",\n\
    \"cargo-antigen\",\n\
]\n";
        assert!(workspace_members_absent_from_const(toml).is_empty());
    }

    #[test]
    fn nested_path_member_uses_its_crate_dir_name() {
        // A member given as a nested path (`crates/foo`) reconciles by its
        // final path component — the crate-dir name the per-crate checks use.
        let toml = "\
[workspace]\n\
members = [\"antigen\", \"crates/antigen-deep\"]\n";
        let unlisted = workspace_members_absent_from_const(toml);
        assert_eq!(unlisted, vec!["antigen-deep".to_string()]);
    }

    // ---- I/O-level wiring (run_checks dispatches to the pure helpers) ------
    // These verify the new checks actually FIRE through run_checks, not just
    // that the pure helpers are correct in isolation.

    use std::io::Write;

    /// Write a minimal workspace root with the given `members` array body and a
    /// version, returning the temp dir (kept alive by the caller).
    fn write_min_workspace(members_lines: &str, version: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        let toml = format!(
            "[workspace]\nmembers = [{members_lines}]\n\n[workspace.package]\nversion = \"{version}\"\n"
        );
        let mut f = std::fs::File::create(dir.path().join("Cargo.toml")).unwrap();
        f.write_all(toml.as_bytes()).unwrap();
        dir
    }

    /// Pull a check by name out of a `run_checks` result vector.
    fn find_check<'a>(results: &'a [CheckResult], name: &str) -> &'a CheckResult {
        results
            .iter()
            .find(|r| r.name == name)
            .unwrap_or_else(|| panic!("no `{name}` check in results"))
    }

    #[test]
    fn run_checks_fails_on_an_unlisted_workspace_member() {
        // DEFECT 3 wiring: a member outside MEMBER_CRATES makes the gate FAIL.
        let dir = write_min_workspace("\"antigen-newcomer\"", "0.6.1");
        let results = run_checks(dir.path(), Some("0.6.1"));
        let wm = find_check(&results, "workspace-members");
        assert!(
            !wm.passed,
            "an unlisted member must FAIL workspace-members: {}",
            wm.detail
        );
        assert!(wm.detail.contains("antigen-newcomer"));
    }

    #[test]
    fn run_checks_passes_workspace_members_when_all_listed() {
        let dir = write_min_workspace(
            "\"antigen\", \"antigen-attestation\", \"antigen-fingerprint\", \
             \"antigen-macros\", \"cargo-antigen\"",
            "0.6.1",
        );
        let results = run_checks(dir.path(), Some("0.6.1"));
        assert!(find_check(&results, "workspace-members").passed);
    }

    #[test]
    fn run_checks_lockfile_check_degrades_honestly_outside_a_repo() {
        // DEFECT 7 wiring: a temp dir is NOT a git repo, so even if the lock were
        // fresh, "committed" cannot be confirmed → the check must NOT pass (it
        // either fails on freshness first, or on the cannot-confirm-committed
        // soft-fail). The load-bearing assertion is that it never PASSES here,
        // since a non-repo can never earn the "committed" half of the claim.
        let dir = write_min_workspace("\"cargo-antigen\"", "0.6.1");
        let results = run_checks(dir.path(), Some("0.6.1"));
        let lf = find_check(&results, "lockfile-fresh");
        assert!(
            !lf.passed,
            "lockfile-fresh must not PASS where committed-ness cannot be confirmed: {}",
            lf.detail
        );
    }

    // ========================================================================
    // ADVERSARIAL PROBES — delta attacks on the v0.6.1 fix
    // ========================================================================

    // ---- version.workspace = true on an internal path-dep ------------------
    // toml_edit sees "version" as a TABLE {workspace=true}, not a string.
    // dep_version(item) = dep_value(item, "version") = item.as_table_like()
    //   ?.get("version")?.as_str()  -> None (TABLE, not str).
    // None arm -> "<no version pin>". CORRECT — no literal pin present.

    #[test]
    fn adv_workspace_inherited_version_on_internal_dep_flagged_as_no_pin() {
        let body = "[dependencies]\nantigen-macros = { version.workspace = true, path = \"../antigen-macros\" }\n";
        let mismatches = internal_pin_mismatches(body, "0.6.1");
        assert!(
            !mismatches.is_empty(),
            "version.workspace=true on internal path-dep must be flagged (no literal pin): {:?}",
            mismatches
        );
        assert_eq!(
            mismatches[0].1, "<no version pin>",
            "should report <no version pin> for workspace-inherited version"
        );
    }

    // ---- optional dep with correct pin — must be clean ---------------------

    #[test]
    fn adv_optional_internal_dep_coherent_is_clean() {
        let body = "[dependencies]\nantigen-macros = { version = \"=0.6.1\", path = \"../antigen-macros\", optional = true }\n";
        assert!(
            internal_pin_mismatches(body, "0.6.1").is_empty(),
            "optional=true must not interfere with pin checking"
        );
    }

    // ---- renamed dep (package = "...") stale pin caught by alias name ------

    #[test]
    fn adv_renamed_dep_stale_pin_caught_by_alias_name() {
        let body = "[dependencies]\nmy-alias = { package = \"antigen-macros\", version = \"=0.6.0\", path = \"../antigen-macros\" }\n";
        let mismatches = internal_pin_mismatches(body, "0.6.1");
        assert!(
            !mismatches.is_empty(),
            "renamed dep with stale pin must be caught"
        );
        assert_eq!(
            mismatches[0].0, "my-alias",
            "should report the alias name, not the package name"
        );
    }

    #[test]
    fn adv_renamed_dep_coherent_pin_is_clean() {
        let body = "[dependencies]\nmy-alias = { package = \"antigen-macros\", version = \"=0.6.1\", path = \"../antigen-macros\" }\n";
        assert!(
            internal_pin_mismatches(body, "0.6.1").is_empty(),
            "renamed dep with correct pin must be clean"
        );
    }

    // ---- caret / tilde / wildcard pins on internal deps must be flagged ----

    #[test]
    fn adv_caret_pin_on_internal_dep_is_mismatch() {
        let body = "[dependencies]\nantigen-macros = { version = \"^0.6.1\", path = \"../antigen-macros\" }\n";
        let mismatches = internal_pin_mismatches(body, "0.6.1");
        assert!(
            !mismatches.is_empty(),
            "caret ^0.6.1 must be flagged (not an exact =pin): {:?}",
            mismatches
        );
        assert_eq!(mismatches[0].1, "^0.6.1");
    }

    #[test]
    fn adv_tilde_pin_on_internal_dep_is_mismatch() {
        let body = "[dependencies]\nantigen-macros = { version = \"~0.6.1\", path = \"../antigen-macros\" }\n";
        let mismatches = internal_pin_mismatches(body, "0.6.1");
        assert!(!mismatches.is_empty(), "tilde ~0.6.1 must be flagged");
    }

    #[test]
    fn adv_wildcard_pin_on_internal_dep_is_mismatch() {
        let body =
            "[dependencies]\nantigen-macros = { version = \"*\", path = \"../antigen-macros\" }\n";
        let mismatches = internal_pin_mismatches(body, "0.6.1");
        assert!(!mismatches.is_empty(), "wildcard pin * must be flagged");
    }

    // ---- DEFECT: [target.'cfg(...)'.dependencies] silently skipped ---------
    // DEP_TABLES = ["dependencies", "dev-dependencies", "build-dependencies"].
    // A target-conditional table is NOT in DEP_TABLES → silently skipped.
    // The source comment acknowledges this as out-of-v0.6.1-scope (antigen's
    // own crates are never target-conditional on each other). Test documents gap.

    #[test]
    fn adv_target_cfg_internal_dep_silently_skipped_known_gap() {
        let body = "[target.'cfg(unix)'.dependencies]\nantigen-macros = { version = \"=0.6.0\", path = \"../antigen-macros\" }\n";
        let mismatches = internal_pin_mismatches(body, "0.6.1");
        // KNOWN GAP: stale pin in a target-conditional table passes through.
        // For antigen's own crates (no such deps), this is acceptable per the comment.
        // This test FAILS if the gap is ever closed (that's the correct update).
        assert!(
            mismatches.is_empty(),
            "KNOWN GAP: target-conditional internal dep silently skipped — if this now fails, the gap is closed (update this test)"
        );
    }

    // ---- DEFECT: glob workspace member produces false-block ----------------
    // rsplit on "crates/*" gives "*" as the final component.
    // MEMBER_CRATES doesn't contain "*" → reported as unlisted.
    // For antigen's own workspace (no globs used), not a live issue.
    // Severity: MEDIUM false-block if workspace ever uses glob members.

    #[test]
    fn adv_glob_workspace_member_produces_false_block_on_star() {
        let toml = "[workspace]\nmembers = [\n    \"antigen\",\n    \"antigen-attestation\",\n    \"antigen-fingerprint\",\n    \"antigen-macros\",\n    \"cargo-antigen\",\n    \"crates/*\",\n]\n";
        let unlisted = workspace_members_absent_from_const(toml);
        // Glob string "crates/*" → final component "*" → reported as unlisted.
        // FAIL here means the glob IS correctly expanded (gap closed — update test).
        assert!(
            unlisted.contains(&"*".to_string()),
            "DEFECT documented: glob member 'crates/*' produces '*' as unlisted (false-block for glob workspaces): {:?}",
            unlisted
        );
    }

    // ---- DEFECT: link-reference definition in changelog body = false-pass --
    // A line "[0.6.1]: https://..." is not blank, not TODO, not <!--
    // → is_real_changelog_line returns true → body check passes.
    // A link-ref is METADATA, not release notes.
    // Severity: LOW (pathological case; real changelogs include actual notes).

    #[test]
    fn adv_changelog_link_ref_only_body_is_false_pass() {
        let body = "## [0.6.1]\n[0.6.1]: https://github.com/foo/bar/compare/v0.6.0...v0.6.1\n\n## [0.6.0]\n- real\n";
        let result = changelog_section_has_body(body, "0.6.1");
        // Currently PASSES (false-pass). This test FAILS until the check
        // filters link-reference definitions from "real content".
        assert!(
            !result,
            "DEFECT: link-reference-only CHANGELOG body is falsely accepted as real content (false-pass)"
        );
    }

    // ---- changelog body robustness -----------------------------------------

    #[test]
    fn adv_changelog_body_whitespace_only_fails() {
        let body = "## [0.6.1]\n   \n\t\n## [0.6.0]\n- real\n";
        assert!(
            !changelog_section_has_body(body, "0.6.1"),
            "whitespace-only body must not pass"
        );
    }

    #[test]
    fn adv_changelog_no_trailing_newline_eof_passes() {
        let body = "## [0.6.1]\n- shipped";
        assert!(
            changelog_section_has_body(body, "0.6.1"),
            "no trailing newline at EOF must still work"
        );
    }

    #[test]
    fn adv_changelog_version_not_present_returns_false_safely() {
        let body = "## [0.6.0]\n- old content\n";
        assert!(
            !changelog_has_version_section(body, "0.6.1"),
            "missing version section must return false without panic"
        );
        assert!(
            !changelog_section_has_body(body, "0.6.1"),
            "missing version body must return false without panic"
        );
    }

    // ---- readme = false (bool) reads as missing ----------------------------

    #[test]
    fn adv_readme_false_bool_reads_as_missing() {
        let body = "[package]\nname = \"noreadme\"\ndescription = \"x\"\nlicense.workspace = true\nrepository.workspace = true\nreadme = false\n";
        let missing = missing_metadata_keys(body);
        assert!(
            missing.contains(&"readme"),
            "readme = false (bool) must be reported missing, got {:?}",
            missing
        );
    }

    // ---- DEFECT: ../../ path dep falsely treated as internal ---------------
    // is_internal_path: p.starts_with("../") → true for "../../outside".
    // An external dep that escapes the workspace root is coherence-checked.
    // If its pin ≠ "=<expected>", it gets flagged — FALSE-BLOCK.
    // Severity: LOW for antigen (no such deps), documented here.

    #[test]
    fn adv_double_dotdot_escape_path_falsely_treated_as_internal() {
        let body = "[dependencies]\noutside-crate = { version = \"^1.0\", path = \"../../outside-workspace\" }\n";
        let mismatches = internal_pin_mismatches(body, "0.6.1");
        // "../../outside-workspace" starts_with("../") → treated as internal.
        // ^1.0 ≠ =0.6.1 → flagged. This is a false-block.
        assert!(
            !mismatches.is_empty(),
            "DEFECT documented: '../../outside' treated as internal; ^1.0 pin falsely flagged (false-block)"
        );
        assert_eq!(mismatches[0].0, "outside-crate");
    }

    // ---- parse_workspace_version does not bleed past [workspace.package] ---

    #[test]
    fn adv_workspace_version_not_bleed_to_subsequent_table() {
        let toml = "[workspace]\nmembers = [\"x\"]\n\n[workspace.package]\nversion = \"0.6.1\"\n\n[profile.dev]\nversion = \"9.9.9\"\n";
        let v = parse_workspace_version(toml);
        assert_eq!(
            v.as_deref(),
            Some("0.6.1"),
            "version must be read from [workspace.package] only, not bleed into [profile.dev]"
        );
    }

    // ---- unparseable TOML degrades conservatively (fails, not passes) ------

    #[test]
    fn adv_unparseable_workspace_toml_fails_conservatively() {
        let bad = "this is not valid [[[ TOML";
        let result = workspace_members_absent_from_const(bad);
        assert!(
            !result.is_empty(),
            "unparseable workspace TOML must fail (non-empty unlisted)"
        );
        assert!(
            result[0].contains("unparseable"),
            "sentinel must mention 'unparseable': {:?}",
            result
        );
    }

    #[test]
    fn adv_unparseable_crate_toml_returns_sentinel_mismatch() {
        let bad = "[[[ not valid TOML";
        let result = internal_pin_mismatches(bad, "0.6.1");
        assert!(
            !result.is_empty(),
            "unparseable crate TOML must return sentinel mismatch"
        );
        assert!(
            result[0].0.contains("<manifest>"),
            "sentinel must name <manifest>: {:?}",
            result
        );
    }

    #[test]
    fn adv_unparseable_metadata_toml_reports_all_missing() {
        let bad = "[[[ not valid";
        let missing = missing_metadata_keys(bad);
        assert_eq!(
            missing.len(),
            4,
            "unparseable metadata TOML must report all 4 keys missing: {:?}",
            missing
        );
    }
}
