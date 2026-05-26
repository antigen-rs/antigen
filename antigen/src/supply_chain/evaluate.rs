//! Substrate-witness evaluation for Supply-Chain Defense Family (ADR-025).
//!
//! Drives the witness-leaf state types from [`super::witness`] against a
//! workspace root + the recorded sidecars under `.attest/supply-chain/`.

use std::path::{Path, PathBuf};

use super::manifest::{read_manifest_deps, DepEntry};
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
#[must_use]
pub fn dep_attest_path(workspace_root: &Path, crate_name: &str, version: &str) -> PathBuf {
    supply_chain_root(workspace_root)
        .join("dep-attest")
        .join(format!("{crate_name}@{version}.json"))
}

/// Path to a [`ContentHashRecord`] sidecar for `<crate>@<version>`.
#[must_use]
pub fn content_hash_path(workspace_root: &Path, crate_name: &str, version: &str) -> PathBuf {
    supply_chain_root(workspace_root)
        .join("content-hash")
        .join(format!("{crate_name}@{version}.json"))
}

/// Path to a [`MaintainerSnapshot`] sidecar for `<crate>`.
#[must_use]
pub fn maintainer_path(workspace_root: &Path, crate_name: &str) -> PathBuf {
    supply_chain_root(workspace_root)
        .join("maintainer")
        .join(format!("{crate_name}.json"))
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
        }
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
    use super::*;
    use crate::supply_chain::schema::ReviewScope;
    use tempfile::TempDir;

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
            }
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
            }
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
}
