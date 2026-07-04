//! Substrate-witness evaluation for Supply-Chain Defense Family (ADR-025).
//!
//! Drives the witness-leaf state types from [`super::witness`] against a
//! workspace root + the recorded sidecars under `.attest/supply-chain/`.

use std::path::{Path, PathBuf};

use antigen_macros::presents;

use super::manifest::{DepEntry, read_manifest_deps};
use super::schema::{ContentHashRecord, DepAttestation, MaintainerSnapshot, SandboxKind};
use super::witness::{
    ContentHashState, DepAttestedState, DepPinnedState, MaintainerState, SandboxState,
};

// ============================================================================
// Sidecar paths
// ============================================================================

/// Workspace-root-relative path to the supply-chain sidecar root.
#[must_use]
pub fn supply_chain_root(workspace_root: &Path) -> PathBuf {
    workspace_root.join(".attest").join("supply-chain")
}

/// Path to a [`DepAttestation`] sidecar for `<crate>@<version>`.
///
/// Defense-in-depth (sub-clause F): `crate_name` is validated against the
/// safe character set HERE, at the path-building primitive, not only at the
/// call sites. An invalid crate name (path-traversal sequences, separators)
/// resolves to the dep-attest directory itself rather than being joined as a
/// path component, so it cannot escape `supply_chain_root` — a subsequent file
/// read finds no sidecar (the safe failure) instead of probing an attacker-
/// chosen path. This guards the `pub fn` primitive against callers (present or
/// future) that reach it without pre-validating — `evaluate_dep_attested` /
/// `load_content_hash_record` did exactly that.
///
/// Defends [`crate::stdlib::dogfood::PathTraversalViaUnvalidatedComponent`].
#[must_use]
// ADR-029 migration: this path-builder `#[presents]` PathTraversalViaUnvalidatedComponent
// (a `pub fn` reachable by callers that don't pre-validate). The test
// `path_builders_reject_traversal_crate_name` declares it defends the class via
// `#[defended_by]`; the audit cross-references and observes the verdict.
#[presents(PathTraversalViaUnvalidatedComponent)]
pub fn dep_attest_path(workspace_root: &Path, crate_name: &str, version: &str) -> PathBuf {
    let dir = supply_chain_root(workspace_root).join("dep-attest");
    if !is_valid_crate_name(crate_name) || !is_valid_version(version) {
        return dir;
    }
    dir.join(format!("{crate_name}@{version}.json"))
}

/// Path to a [`ContentHashRecord`] sidecar for `<crate>@<version>`.
///
/// Same defense-in-depth as [`dep_attest_path`]: invalid `crate_name`/`version`
/// resolves to the content-hash directory (in-root, non-escaping), never an
/// attacker-controlled traversal.
#[must_use]
// ADR-029 migration: this path-builder `#[presents]` PathTraversalViaUnvalidatedComponent
// (a `pub fn` reachable by callers that don't pre-validate). The test
// `path_builders_reject_traversal_crate_name` declares it defends the class via
// `#[defended_by]`; the audit cross-references and observes the verdict.
#[presents(PathTraversalViaUnvalidatedComponent)]
pub fn content_hash_path(workspace_root: &Path, crate_name: &str, version: &str) -> PathBuf {
    let dir = supply_chain_root(workspace_root).join("content-hash");
    if !is_valid_crate_name(crate_name) || !is_valid_version(version) {
        return dir;
    }
    dir.join(format!("{crate_name}@{version}.json"))
}

/// Path to a [`MaintainerSnapshot`] sidecar for `<crate>`.
///
/// Same defense-in-depth as [`dep_attest_path`]: invalid `crate_name` resolves
/// to the maintainer directory (in-root, non-escaping).
#[must_use]
// ADR-029 migration: this path-builder `#[presents]` PathTraversalViaUnvalidatedComponent
// (a `pub fn` reachable by callers that don't pre-validate). The test
// `path_builders_reject_traversal_crate_name` declares it defends the class via
// `#[defended_by]`; the audit cross-references and observes the verdict.
#[presents(PathTraversalViaUnvalidatedComponent)]
pub fn maintainer_path(workspace_root: &Path, crate_name: &str) -> PathBuf {
    let dir = supply_chain_root(workspace_root).join("maintainer");
    if !is_valid_crate_name(crate_name) {
        return dir;
    }
    dir.join(format!("{crate_name}.json"))
}

/// Returns `true` iff `name` is a safe crate name: only ASCII alphanumeric, `_`, or `-`.
/// Rejects path traversal sequences (`..`, `/`, `\`) and other shell-special characters.
#[must_use]
pub fn is_valid_crate_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
}

/// Returns `true` iff `version` is safe for path composition.
///
/// Allows ASCII alphanumeric plus `.`, `-`, `+` (the `SemVer` pre-release/build
/// set); rejects path-traversal sequences and separators. Mirrors the version
/// character-set the CLI's `parse_crate_at_version` enforces, applied here at
/// the path-building primitive for defense-in-depth.
#[must_use]
pub fn is_valid_version(version: &str) -> bool {
    !version.is_empty()
        && version
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '+'))
}

// ============================================================================
// dep_pinned
// ============================================================================

/// Evaluate `dep_pinned(crate?)` against the workspace's `Cargo.toml`.
///
/// When `crate_name` is `None`, all deps in the manifest must be exact-
/// pinned for the leaf to pass.
#[must_use]
pub fn evaluate_dep_pinned(workspace_root: &Path, crate_name: Option<&str>) -> DepPinnedState {
    let manifest = workspace_root.join("Cargo.toml");
    let entries = read_manifest_deps(&manifest);
    evaluate_dep_pinned_against(&entries, crate_name)
}

/// Evaluate `dep_pinned(crate?)` against a pre-collected slice of
/// dep entries. Exposed for testing.
#[must_use]
pub fn evaluate_dep_pinned_against(
    entries: &[DepEntry],
    crate_name: Option<&str>,
) -> DepPinnedState {
    let mut unpinned: Vec<String> = Vec::new();
    let mut saw_named = false;

    for entry in entries {
        if let Some(named) = crate_name {
            if entry.name != named {
                continue;
            }
            saw_named = true;
        }

        if !entry.is_exact_pinned() {
            unpinned.push(entry.name.clone());
        }
    }

    if let Some(named) = crate_name {
        if !saw_named {
            return DepPinnedState::NotInManifest {
                crate_name: named.to_string(),
            };
        }
    }

    if unpinned.is_empty() {
        DepPinnedState::AllPinned
    } else {
        DepPinnedState::Unpinned {
            unpinned_deps: unpinned,
        }
    }
}

// ============================================================================
// dep_attested
// ============================================================================

/// Evaluate `dep_attested(crate, version, exact_version)` against the
/// sidecar at `.attest/supply-chain/dep-attest/<crate>@<version>.json`.
#[must_use]
pub fn evaluate_dep_attested(
    workspace_root: &Path,
    crate_name: &str,
    version: &str,
    exact_version: bool,
) -> DepAttestedState {
    let path = dep_attest_path(workspace_root, crate_name, version);
    let Ok(content) = std::fs::read_to_string(&path) else {
        // Try the floor of the dep-attest directory for any version
        // match if exact_version=false.
        if exact_version {
            return DepAttestedState::SidecarMissing;
        }
        return find_any_attest_for_crate(workspace_root, crate_name)
            .unwrap_or(DepAttestedState::SidecarMissing);
    };

    let att = match serde_json::from_str::<DepAttestation>(&content) {
        Err(e) => {
            return DepAttestedState::SidecarMalformed {
                error: e.to_string(),
            };
        },
        Ok(a) => a,
    };

    if !att.has_reviewable_artifact() {
        return DepAttestedState::AttestedWithoutReviewableArtifact;
    }
    if exact_version && att.version != version {
        return DepAttestedState::AttestationStale {
            attested_version: att.version,
            requested_version: version.to_string(),
        };
    }
    DepAttestedState::Attested {
        review_scope: att.review_scope,
    }
}

/// Best-effort scan of the dep-attest directory looking for any version
/// of the named crate. Used by `exact_version=false` callers.
fn find_any_attest_for_crate(workspace_root: &Path, crate_name: &str) -> Option<DepAttestedState> {
    let dir = supply_chain_root(workspace_root).join("dep-attest");
    let entries = std::fs::read_dir(&dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        let stem = path.file_stem()?.to_string_lossy().to_string();
        // file_stem strips `.json` extension; format is `<crate>@<version>`
        if let Some(at) = stem.find('@') {
            let candidate = &stem[..at];
            if candidate == crate_name {
                let content = std::fs::read_to_string(&path).ok()?;
                let att: DepAttestation = serde_json::from_str(&content).ok()?;
                if !att.has_reviewable_artifact() {
                    return Some(DepAttestedState::AttestedWithoutReviewableArtifact);
                }
                return Some(DepAttestedState::Attested {
                    review_scope: att.review_scope,
                });
            }
        }
    }
    None
}

// ============================================================================
// content_hash_matches
// ============================================================================

/// Evaluate `content_hash_matches(crate, version)` against
/// `.attest/supply-chain/content-hash/<crate>@<version>.json` PLUS the
/// current lockfile checksum.
///
/// **v0.2 limitation**: `current_hash_from_lockfile` reads the
/// `[[package]] checksum = "..."` field. Direct crates.io tarball
/// re-fetch is v0.3+.
///
/// **Three failure modes are structurally distinguished** (per ATK-SC-2-A):
/// - `NoAttestation` — sidecar file missing
/// - `SidecarMalformed` — sidecar present but JSON-corrupt
/// - `CrateNotInLockfile` — lockfile missing or crate@version not present
///
/// Collapsing malformed-sidecar into `NoAttestation` would let an attacker
/// convert a high-severity Mismatch into a low-severity `NoAttestation` by
/// corrupting the sidecar JSON. Each state must surface its own hint.
#[must_use]
pub fn evaluate_content_hash_matches(
    workspace_root: &Path,
    crate_name: &str,
    version: &str,
) -> ContentHashState {
    let recorded = match load_content_hash_record(workspace_root, crate_name, version) {
        Ok(Some(r)) => r,
        Ok(None) => return ContentHashState::NoAttestation,
        Err(e) => return ContentHashState::SidecarMalformed { error: e },
    };

    let lockfile = workspace_root.join("Cargo.lock");
    let Some(current) = current_hash_from_lockfile(&lockfile, crate_name, version) else {
        return ContentHashState::CrateNotInLockfile {
            crate_name: crate_name.to_string(),
        };
    };

    if current == recorded.content_hash {
        ContentHashState::Matches
    } else {
        ContentHashState::Mismatch {
            recorded: recorded.content_hash,
            current,
        }
    }
}

/// Pure comparator for the live crates.io content-hash verification.
///
/// The 3-valued core of `infra/live-cratesio-query-and-tarball-sha256`, kept
/// pure (no network) so it is exhaustively unit-testable offline. The network
/// fetch is a thin shell (in the cargo-antigen CLI) that produces `served` and
/// feeds it here: `None` when the registry could not be reached (offline /
/// version absent), `Some(hash)` when a served hash was obtained.
///
/// The three outcomes mirror the gem at the network boundary:
/// - `served == Some(h)` and `h == expected` → [`super::witness::LiveCksumState::Verified`].
/// - `served == Some(h)` and `h != expected` → [`super::witness::LiveCksumState::Mismatch`]
///   (the substitution signal — loud).
/// - `served == None` → [`super::witness::LiveCksumState::Unverifiable`] (⊥ —
///   could-not-evaluate; offline must never read as verified OR failed).
#[must_use]
pub fn compare_live_cksum(served: Option<&str>, expected: &str) -> super::witness::LiveCksumState {
    use super::witness::LiveCksumState;
    match served {
        None => LiveCksumState::Unverifiable {
            reason: "crates.io registry unreachable (offline, network error, or version \
                     absent from the index) — the live hash could not be obtained"
                .to_string(),
        },
        Some(h) if h == expected => LiveCksumState::Verified {
            hash: h.to_string(),
        },
        Some(h) => LiveCksumState::Mismatch {
            expected: expected.to_string(),
            served: h.to_string(),
        },
    }
}

/// Load a content-hash record from disk.
///
/// Returns:
/// - `Ok(Some(record))` — file exists and is well-formed
/// - `Ok(None)` — file is missing (no first-attestation yet)
/// - `Err(parse_error)` — file exists but does NOT deserialize cleanly
///
/// **Per ATK-SC-2-A**: malformed sidecars MUST be distinguishable from
/// missing sidecars. A `Result<Option<_>, _>` separates the two so the
/// caller can surface different audit hints.
///
/// # Errors
///
/// Returns the JSON parse error string if the file exists but fails
/// to deserialize as `ContentHashRecord`.
pub fn load_content_hash_record(
    workspace_root: &Path,
    crate_name: &str,
    version: &str,
) -> Result<Option<ContentHashRecord>, String> {
    let path = content_hash_path(workspace_root, crate_name, version);
    let Ok(content) = std::fs::read_to_string(&path) else {
        return Ok(None);
    };
    serde_json::from_str(&content)
        .map(Some)
        .map_err(|e| e.to_string())
}

/// Persist a content-hash record. Creates the `.attest/supply-chain/
/// content-hash/` directory if missing.
///
/// # Errors
///
/// Returns the IO error from create-dir or write.
pub fn save_content_hash_record(
    workspace_root: &Path,
    record: &ContentHashRecord,
) -> std::io::Result<PathBuf> {
    let path = content_hash_path(workspace_root, &record.crate_name, &record.version);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(record).map_err(std::io::Error::other)?;
    std::fs::write(&path, &json)?;
    Ok(path)
}

/// Extract the `checksum = "..."` value for a `[[package]]` entry whose
/// `name = ...` AND `version = ...` match. v0.2 implementation: a
/// minimal line-based scanner — Cargo.lock is structurally simple.
#[must_use]
pub fn current_hash_from_lockfile(
    lockfile_path: &Path,
    crate_name: &str,
    version: &str,
) -> Option<String> {
    let content = std::fs::read_to_string(lockfile_path).ok()?;
    let mut in_package = false;
    let mut name_match = false;
    let mut version_match = false;
    let mut checksum: Option<String> = None;

    for line in content.lines() {
        let line = line.trim();
        if line == "[[package]]" {
            // commit any pending match
            if in_package && name_match && version_match && checksum.is_some() {
                return checksum;
            }
            in_package = true;
            name_match = false;
            version_match = false;
            checksum = None;
            continue;
        }
        if !in_package {
            continue;
        }
        if let Some(rest) = line.strip_prefix("name = ") {
            if let Some(v) = strip_quotes_simple(rest) {
                name_match = v == crate_name;
            }
        } else if let Some(rest) = line.strip_prefix("version = ") {
            if let Some(v) = strip_quotes_simple(rest) {
                version_match = v == version;
            }
        } else if let Some(rest) = line.strip_prefix("checksum = ") {
            if let Some(v) = strip_quotes_simple(rest) {
                checksum = Some(v.to_string());
            }
        }
    }
    // EOF: commit the last package if it matched.
    if in_package && name_match && version_match && checksum.is_some() {
        return checksum;
    }
    None
}

/// Resolve the version `Cargo.lock` recorded for `crate_name` — the version a
/// `verify dep-pin` suggestion should pin to (`=<version>`).
///
/// Returns the first `[[package]]` block whose `name` matches. A crate present
/// at multiple versions (rare for direct deps) yields the first; the suggestion
/// is advisory, and the adopter confirms before applying. Same minimal
/// line-based scanner as [`current_hash_from_lockfile`] — `Cargo.lock` is
/// structurally simple and this avoids a TOML-parser dependency on the
/// substrate-reading path.
#[must_use]
pub fn resolved_version_from_lockfile(lockfile_path: &Path, crate_name: &str) -> Option<String> {
    let content = std::fs::read_to_string(lockfile_path).ok()?;
    let mut in_package = false;
    let mut name_match = false;
    let mut version: Option<String> = None;

    for line in content.lines() {
        let line = line.trim();
        if line == "[[package]]" {
            if in_package && name_match {
                if let Some(v) = version.take() {
                    return Some(v);
                }
            }
            in_package = true;
            name_match = false;
            version = None;
            continue;
        }
        if !in_package {
            continue;
        }
        if let Some(rest) = line.strip_prefix("name = ") {
            if let Some(v) = strip_quotes_simple(rest) {
                name_match = v == crate_name;
            }
        } else if let Some(rest) = line.strip_prefix("version = ") {
            if let Some(v) = strip_quotes_simple(rest) {
                version = Some(v.to_string());
            }
        }
    }
    // EOF: commit the last package if it matched.
    if in_package && name_match {
        return version;
    }
    None
}

fn strip_quotes_simple(s: &str) -> Option<&str> {
    let s = s.trim();
    s.strip_prefix('"').and_then(|s| s.strip_suffix('"'))
}

// ============================================================================
// maintainer_unchanged
// ============================================================================

/// Evaluate `maintainer_unchanged(crate, since_version)` against the
/// maintainer-snapshot sidecar.
///
/// **v0.2**: live crates.io query is not implemented; this evaluator
/// returns `CratesIoQueryUnavailable` if no snapshot is present, and
/// `Unchanged` if the snapshot's `since_version` matches.
#[must_use]
pub fn evaluate_maintainer_unchanged(
    workspace_root: &Path,
    crate_name: &str,
    since_version: &str,
) -> MaintainerState {
    if !is_valid_crate_name(crate_name) {
        return MaintainerState::SnapshotMissing;
    }
    let Some(snap) = load_maintainer_snapshot(workspace_root, crate_name) else {
        return MaintainerState::SnapshotMissing;
    };

    if snap.since_version != since_version {
        // Snapshot version differs from request — we don't have a way
        // to live-query crates.io in v0.2 to detect actual change;
        // surface the unavailable hint.
        return MaintainerState::CratesIoQueryUnavailable;
    }

    // If the snapshot is current we trust it; live re-query is v0.3+.
    MaintainerState::Unchanged
}

/// Load a maintainer snapshot from disk; `None` on missing/malformed.
#[must_use]
pub fn load_maintainer_snapshot(
    workspace_root: &Path,
    crate_name: &str,
) -> Option<MaintainerSnapshot> {
    let path = maintainer_path(workspace_root, crate_name);
    let content = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

// ============================================================================
// sandbox_clean
// ============================================================================

/// Evaluate `sandbox_clean(crate, sandbox_kind)`.
///
/// **v0.2**: returns `ToolingNotYetAvailable` unconditionally — actual
/// sandbox execution is deferred to v0.4+ per ADR-025 tooling-phase 3.
#[must_use]
pub const fn evaluate_sandbox_clean(_crate_name: &str, sandbox_kind: SandboxKind) -> SandboxState {
    SandboxState::ToolingNotYetAvailable { sandbox_kind }
}

#[cfg(test)]
mod tests {
    use antigen_macros::defended_by;
    use tempfile::TempDir;

    use super::*;
    // `#[defended_by]` (ADR-029) is used only on the witness test below; import
    // it here rather than at module scope (where it would be unused).
    use crate::supply_chain::schema::ReviewScope;

    fn write_manifest(dir: &Path, content: &str) {
        std::fs::write(dir.join("Cargo.toml"), content).unwrap();
    }

    #[test]
    fn dep_pinned_all_pinned() {
        let tmp = TempDir::new().unwrap();
        write_manifest(
            tmp.path(),
            r#"
[dependencies]
serde = "=1.0.197"
"#,
        );
        assert_eq!(
            evaluate_dep_pinned(tmp.path(), None),
            DepPinnedState::AllPinned
        );
    }

    #[test]
    fn dep_pinned_flags_caret() {
        let tmp = TempDir::new().unwrap();
        write_manifest(
            tmp.path(),
            r#"
[dependencies]
serde = "1.0"
clap = "=4.0"
"#,
        );
        match evaluate_dep_pinned(tmp.path(), None) {
            DepPinnedState::Unpinned { unpinned_deps } => {
                assert_eq!(unpinned_deps, vec!["serde".to_string()]);
            },
            other => panic!("expected Unpinned, got {other:?}"),
        }
    }

    #[test]
    fn dep_pinned_named_not_in_manifest() {
        let tmp = TempDir::new().unwrap();
        write_manifest(
            tmp.path(),
            r#"
[dependencies]
serde = "=1.0.197"
"#,
        );
        match evaluate_dep_pinned(tmp.path(), Some("missing")) {
            DepPinnedState::NotInManifest { crate_name } => assert_eq!(crate_name, "missing"),
            other => panic!("expected NotInManifest, got {other:?}"),
        }
    }

    #[test]
    fn dep_attested_sidecar_missing() {
        let tmp = TempDir::new().unwrap();
        let state = evaluate_dep_attested(tmp.path(), "serde", "1.0.197", true);
        assert_eq!(state, DepAttestedState::SidecarMissing);
    }

    #[test]
    fn dep_attested_empty_artifact_flags_rubber_stamp() {
        let tmp = TempDir::new().unwrap();
        let att = DepAttestation {
            crate_name: "serde".to_string(),
            version: "1.0.197".to_string(),
            exact_version: true,
            reviewable_artifact: PathBuf::new(),
            review_scope: ReviewScope::MetadataOnly,
            signed_by: "alice".to_string(),
            date: "2026-05-22".to_string(),
            rationale: None,
        };
        let path = dep_attest_path(tmp.path(), "serde", "1.0.197");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, serde_json::to_string(&att).unwrap()).unwrap();

        assert_eq!(
            evaluate_dep_attested(tmp.path(), "serde", "1.0.197", true),
            DepAttestedState::AttestedWithoutReviewableArtifact
        );
    }

    #[test]
    fn content_hash_no_attestation_state() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(
            evaluate_content_hash_matches(tmp.path(), "serde", "1.0.197"),
            ContentHashState::NoAttestation
        );
    }

    #[test]
    fn content_hash_match_path() {
        let tmp = TempDir::new().unwrap();
        // Record
        let record = ContentHashRecord {
            crate_name: "serde".to_string(),
            version: "1.0.197".to_string(),
            content_hash: "abc123".to_string(),
            hash_source: "cargo-lock-checksum".to_string(),
            signed_by: "alice".to_string(),
            date: "2026-05-22".to_string(),
        };
        save_content_hash_record(tmp.path(), &record).unwrap();
        // Cargo.lock with matching checksum
        std::fs::write(
            tmp.path().join("Cargo.lock"),
            r#"
[[package]]
name = "serde"
version = "1.0.197"
checksum = "abc123"
"#,
        )
        .unwrap();
        assert_eq!(
            evaluate_content_hash_matches(tmp.path(), "serde", "1.0.197"),
            ContentHashState::Matches
        );
    }

    #[test]
    fn content_hash_mismatch_path() {
        let tmp = TempDir::new().unwrap();
        let record = ContentHashRecord {
            crate_name: "serde".to_string(),
            version: "1.0.197".to_string(),
            content_hash: "recorded-hash".to_string(),
            hash_source: "cargo-lock-checksum".to_string(),
            signed_by: "alice".to_string(),
            date: "2026-05-22".to_string(),
        };
        save_content_hash_record(tmp.path(), &record).unwrap();
        std::fs::write(
            tmp.path().join("Cargo.lock"),
            r#"
[[package]]
name = "serde"
version = "1.0.197"
checksum = "swapped-hash"
"#,
        )
        .unwrap();
        match evaluate_content_hash_matches(tmp.path(), "serde", "1.0.197") {
            ContentHashState::Mismatch { recorded, current } => {
                assert_eq!(recorded, "recorded-hash");
                assert_eq!(current, "swapped-hash");
            },
            other => panic!("expected Mismatch, got {other:?}"),
        }
    }

    #[test]
    fn content_hash_crate_not_in_lockfile() {
        let tmp = TempDir::new().unwrap();
        let record = ContentHashRecord {
            crate_name: "serde".to_string(),
            version: "1.0.197".to_string(),
            content_hash: "abc".to_string(),
            hash_source: "cargo-lock-checksum".to_string(),
            signed_by: "alice".to_string(),
            date: "2026-05-22".to_string(),
        };
        save_content_hash_record(tmp.path(), &record).unwrap();
        // No Cargo.lock at all.
        match evaluate_content_hash_matches(tmp.path(), "serde", "1.0.197") {
            ContentHashState::CrateNotInLockfile { crate_name } => assert_eq!(crate_name, "serde"),
            other => panic!("expected CrateNotInLockfile, got {other:?}"),
        }
    }

    #[test]
    fn maintainer_missing_snapshot() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(
            evaluate_maintainer_unchanged(tmp.path(), "serde", "1.0.197"),
            MaintainerState::SnapshotMissing
        );
    }

    #[test]
    fn sandbox_v02_returns_tooling_unavailable() {
        let state = evaluate_sandbox_clean("serde", SandboxKind::Build);
        assert_eq!(
            state,
            SandboxState::ToolingNotYetAvailable {
                sandbox_kind: SandboxKind::Build
            }
        );
    }

    // ========================================================================
    // Path-traversal defense-in-depth: the path-builder primitives must refuse
    // to escape supply_chain_root on a malicious crate_name/version, regardless
    // of whether the caller pre-validated. (observer's finding: evaluate_dep_
    // attested + load_content_hash_record reached the path-builders without the
    // is_valid_crate_name check that evaluate_maintainer_unchanged has.)
    // ========================================================================

    /// A path-traversal `crate_name` must NOT produce a path that escapes the
    /// supply-chain root. The guarded path-builder resolves it to the in-root
    /// directory instead of joining the traversal component.
    #[test]
    #[defended_by(PathTraversalViaUnvalidatedComponent)]
    fn path_builders_reject_traversal_crate_name() {
        let root = Path::new("/ws");
        let sc_root = supply_chain_root(root);
        for evil in [
            "../../../etc/passwd",
            "..",
            "foo/bar",
            "foo\\bar",
            "a/../../b",
            "", // empty is also invalid
        ] {
            for built in [
                dep_attest_path(root, evil, "1.0.0"),
                content_hash_path(root, evil, "1.0.0"),
                maintainer_path(root, evil),
            ] {
                assert!(
                    built.starts_with(&sc_root),
                    "path-builder must stay within supply_chain_root for evil \
                     crate_name {evil:?}; got {built:?}"
                );
                // And specifically: it must NOT have joined the traversal — the
                // built path is the bare in-root directory, no escaping component.
                assert!(
                    !built.to_string_lossy().contains(".."),
                    "built path must not carry a `..` traversal component for \
                     {evil:?}; got {built:?}"
                );
            }
        }
    }

    /// A malicious `version` is also blocked (`dep_attest`/`content_hash` compose
    /// it into the filename).
    #[test]
    fn path_builders_reject_traversal_version() {
        let root = Path::new("/ws");
        let sc_root = supply_chain_root(root);
        for evil_version in ["../../../etc", "1.0.0/../../.."] {
            for built in [
                dep_attest_path(root, "serde", evil_version),
                content_hash_path(root, "serde", evil_version),
            ] {
                assert!(built.starts_with(&sc_root));
                assert!(!built.to_string_lossy().contains(".."));
            }
        }
    }

    /// Control: a VALID `crate_name` + version still composes the real sidecar
    /// path (the guard must not over-reject legitimate input).
    #[test]
    fn path_builders_compose_valid_names() {
        let root = Path::new("/ws");
        let p = dep_attest_path(root, "serde_json", "1.0.197");
        assert!(p.ends_with("serde_json@1.0.197.json"));
        let m = maintainer_path(root, "tokio-util");
        assert!(m.ends_with("tokio-util.json"));
    }

    #[test]
    fn resolved_version_reads_the_locked_version() {
        let tmp = TempDir::new().unwrap();
        let lock = tmp.path().join("Cargo.lock");
        std::fs::write(
            &lock,
            r#"# This file is automatically @generated by Cargo.
version = 3

[[package]]
name = "serde"
version = "1.0.197"
checksum = "abc123"

[[package]]
name = "tokio"
version = "1.36.0"
"#,
        )
        .unwrap();
        assert_eq!(
            resolved_version_from_lockfile(&lock, "serde").as_deref(),
            Some("1.0.197")
        );
        // A package with no checksum line (path/git dep) still resolves a version.
        assert_eq!(
            resolved_version_from_lockfile(&lock, "tokio").as_deref(),
            Some("1.36.0")
        );
    }

    #[test]
    fn resolved_version_absent_crate_is_none() {
        let tmp = TempDir::new().unwrap();
        let lock = tmp.path().join("Cargo.lock");
        std::fs::write(
            &lock,
            "[[package]]\nname = \"serde\"\nversion = \"1.0.197\"\n",
        )
        .unwrap();
        assert_eq!(resolved_version_from_lockfile(&lock, "not-a-dep"), None);
    }

    #[test]
    fn resolved_version_missing_lockfile_is_none() {
        let tmp = TempDir::new().unwrap();
        // No Cargo.lock written.
        assert_eq!(
            resolved_version_from_lockfile(&tmp.path().join("Cargo.lock"), "serde"),
            None
        );
    }

    // ------------------------------------------------------------------------
    // compare_live_cksum — the 3-valued live crates.io verification core.
    // Pure (no network), so all three branches are exhaustively unit-testable
    // offline; the network shell (cargo-antigen) only produces the Option.
    // ------------------------------------------------------------------------

    #[test]
    fn live_cksum_matching_served_hash_is_verified() {
        use crate::supply_chain::witness::LiveCksumState;
        let out = compare_live_cksum(Some("abc123"), "abc123");
        assert_eq!(
            out,
            LiveCksumState::Verified {
                hash: "abc123".into()
            }
        );
        assert!(out.is_verified() && !out.is_unverifiable());
    }

    #[test]
    fn live_cksum_differing_served_hash_is_mismatch() {
        use crate::supply_chain::witness::LiveCksumState;
        let out = compare_live_cksum(Some("served-deadbeef"), "expected-abc123");
        assert_eq!(
            out,
            LiveCksumState::Mismatch {
                expected: "expected-abc123".into(),
                served: "served-deadbeef".into(),
            }
        );
        // A mismatch is NOT verified, but it is also NOT unverifiable — it is a
        // genuine, loud, evaluated failure (the substitution signal).
        assert!(!out.is_verified() && !out.is_unverifiable());
    }

    #[test]
    fn live_cksum_offline_is_unverifiable_not_pass_or_fail() {
        use crate::supply_chain::witness::LiveCksumState;
        // The gem at the network boundary: None (offline) is ⊥, distinct from
        // both Verified and Mismatch. It must NOT collapse to either.
        let out = compare_live_cksum(None, "expected-abc123");
        assert!(
            out.is_unverifiable(),
            "offline must be Unverifiable (⊥): {out:?}"
        );
        assert!(
            !out.is_verified(),
            "offline must NOT read as verified (false-green): {out:?}"
        );
        assert!(
            matches!(out, LiveCksumState::Unverifiable { .. }),
            "offline is the third value, never Mismatch (false-alarm): {out:?}"
        );
    }

    #[test]
    fn live_cksum_three_outcomes_are_pairwise_distinct() {
        // The whole point of the 3-valued core: the three outcomes never alias.
        let verified = compare_live_cksum(Some("h"), "h");
        let mismatch = compare_live_cksum(Some("h"), "other");
        let unverifiable = compare_live_cksum(None, "h");
        assert_ne!(verified, mismatch);
        assert_ne!(verified, unverifiable);
        assert_ne!(mismatch, unverifiable);
    }
}
