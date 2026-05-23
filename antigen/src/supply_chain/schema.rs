//! Sidecar schema for Supply-Chain Defense Family (ADR-025).
//!
//! All sidecars live under `.attest/supply-chain/` at the workspace root.
//! Per-family sub-directories:
//!
//! - `.attest/supply-chain/dep-attest/<crate>@<version>.json` —
//!   [`DepAttestation`]
//! - `.attest/supply-chain/content-hash/<crate>@<version>.json` —
//!   [`ContentHashRecord`]
//! - `.attest/supply-chain/maintainer/<crate>.json` —
//!   [`MaintainerSnapshot`]
//!
//! All schemas are additive (new fields default-on-missing) per ADR-021.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// ============================================================================
// Common
// ============================================================================

/// The granularity of a dependency review captured in a `DepAttestation`.
///
/// Per ADR-025 §Schema-additions. The audit hint
/// `dependency-upgrade-without-diff-review` fires when an upgrade is
/// attested only at `MetadataOnly` scope.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReviewScope {
    /// Full source review of the dep at this version.
    Full,
    /// Diff-only review against the prior version's attestation.
    Diff,
    /// Reviewed only the `build.rs` (sufficient for build-script-only
    /// concerns; insufficient for `DependencyUpgradeWithoutDiffReview`).
    BuildScriptOnly,
    /// Reviewed only the proc-macro source (sufficient for
    /// `UnsandboxedProcMacro` concerns; insufficient otherwise).
    ProcMacroOnly,
    /// Reviewed metadata only (Cargo.toml, README). Lowest tier.
    /// Triggers `dependency-upgrade-without-diff-review` on upgrade.
    MetadataOnly,
}

/// Sandbox-kind discriminator for [`crate::supply_chain::witness::SandboxState`].
///
/// `Build` covers `build.rs` execution at compile time. `ProcMacro` covers
/// proc-macro execution inside the rustc process (higher-risk per ADR-025
/// B3-R).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SandboxKind {
    /// `build.rs` execution at compile time.
    Build,
    /// Proc-macro execution inside the rustc process.
    ProcMacro,
}

impl SandboxKind {
    /// String form for CLI parsing + audit-hint rendering.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Build => "build",
            Self::ProcMacro => "proc-macro",
        }
    }

    /// Parse from the kebab-case form. Returns `None` for unknown kinds.
    ///
    /// Distinct from [`std::str::FromStr::from_str`] — that trait's error
    /// type is `()`; this returns `Option` because the only meaningful
    /// "error" is "not one of the two known kinds."
    #[must_use]
    pub fn parse_kind(s: &str) -> Option<Self> {
        match s {
            "build" => Some(Self::Build),
            "proc-macro" | "proc_macro" => Some(Self::ProcMacro),
            _ => None,
        }
    }
}

// ============================================================================
// DepAttestation
// ============================================================================

/// Sidecar persisted at `.attest/supply-chain/dep-attest/<crate>@<version>.json`.
///
/// Records that a team-member reviewed a dependency at a specific version
/// and attested a `reviewable_artifact` exists that documents the review.
///
/// **REQUIRED `reviewable_artifact`** per ADR-025 §Schema-additions: a
/// `DepAttestation` with an empty `reviewable_artifact` is a rubber-stamp.
/// The audit emits `dep-attest-without-reviewable-artifact` when this
/// field is empty.
///
/// **Named limitation** (per ADR-025 §Known-limitations item 1): the
/// presence of an artifact does not guarantee the artifact actually
/// documents a substantive review. This is rubber-stamp risk; the antigen
/// surfaces structure, not motivation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepAttestation {
    /// Crate name as it appears in `Cargo.toml`.
    pub crate_name: String,
    /// Version string of the crate as recorded in `Cargo.toml`.
    pub version: String,
    /// Whether the attestation applies only to this exact version
    /// (`true`) or to a range starting at this version (`false`).
    /// Default `true` — version-specific attestations are the safer
    /// posture per the AI-pair failure pattern.
    #[serde(default = "default_true")]
    pub exact_version: bool,
    /// Path (workspace-relative) to the document recording the review.
    /// **REQUIRED non-empty** — empty = rubber-stamp, audit-hint fires.
    pub reviewable_artifact: PathBuf,
    /// Review scope. Affects `DependencyUpgradeWithoutDiffReview`
    /// evaluation.
    pub review_scope: ReviewScope,
    /// Signer name (typically `git config user.name` at attest-time).
    pub signed_by: String,
    /// ISO-8601 date the attestation was recorded.
    pub date: String,
    /// Optional free-text rationale.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
}

const fn default_true() -> bool {
    true
}

impl DepAttestation {
    /// Returns true if the sidecar identifies a substantive review
    /// (`reviewable_artifact` is non-empty AND not whitespace-only).
    ///
    /// **Whitespace-bypass guard** (per ATK-SC-1-A): a `reviewable_artifact`
    /// containing only spaces, tabs, or newlines is treated as a rubber-stamp
    /// — `is_empty()` alone would let `" "` slip through. The spirit of the
    /// requirement per ADR-025 is "non-empty AND meaningful," and whitespace-
    /// only is structurally indistinguishable from empty for our purposes.
    ///
    /// Audit emits `dep-attest-without-reviewable-artifact` when this is
    /// false.
    #[must_use]
    pub fn has_reviewable_artifact(&self) -> bool {
        // OS-string trim isn't available on all platforms uniformly; round-
        // trip via to_string_lossy for a uniform definition. The cost is
        // negligible (path is short) and the semantics match ASCII paths,
        // which is the only case ADR-025 supply-chain sidecars target.
        let s = self.reviewable_artifact.to_string_lossy();
        !s.trim().is_empty()
    }
}

// ============================================================================
// ContentHashRecord
// ============================================================================

/// Sidecar persisted at
/// `.attest/supply-chain/content-hash/<crate>@<version>.json`.
///
/// Records the first-attestation content-hash of a published crate
/// artifact at a specific version. Subsequent `cargo antigen verify
/// content-hash <crate@version>` runs compare the current resolved hash
/// against this record; divergence emits `content-hash-mismatch`.
///
/// **THE LOAD-BEARING SIDECAR FOR THE CHALK/DEBUG ATTACK CLASS**.
/// Without first-attestation, content-hash verification cannot fire
/// (only `content-hash-no-attestation` surfaces).
///
/// **Named limitation (v0.2)**: `hash_source` records where the hash
/// was sampled from. v0.2 supports `cargo-lock-checksum` only (the
/// `[[package]] checksum = "..."` field). v0.3+ adds `crates-io-tarball`
/// (direct tarball SHA-256) which closes the first-resolve-poisoning
/// gap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentHashRecord {
    /// Crate name.
    pub crate_name: String,
    /// Version string.
    pub version: String,
    /// SHA-256 hex digest of the artifact content (lower-case, no `0x`).
    pub content_hash: String,
    /// Where the hash was sampled from. v0.2: `"cargo-lock-checksum"`.
    /// v0.3+: also `"crates-io-tarball"`.
    pub hash_source: String,
    /// Signer name at attestation time.
    pub signed_by: String,
    /// ISO-8601 date the record was created.
    pub date: String,
}

// ============================================================================
// MaintainerSnapshot
// ============================================================================

/// Sidecar persisted at `.attest/supply-chain/maintainer/<crate>.json`.
///
/// Records the crate's owner/maintainer set at the most recent attested
/// version. `cargo antigen verify maintainer-changes` compares the
/// current crates.io owner set against this snapshot.
///
/// **Named limitation (v0.2)**: snapshot is populated manually or from
/// a local crates.io API query during attestation; live re-query at
/// audit time is v0.3+ (`crates-io-metadata-query-failed` hint covers
/// the v0.2 case).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintainerSnapshot {
    /// Crate name.
    pub crate_name: String,
    /// Version this snapshot was taken against.
    pub since_version: String,
    /// Owner names/teams as reported by crates.io at snapshot time.
    /// Set semantics; order doesn't matter at compare time.
    pub owners: Vec<String>,
    /// Signer name at snapshot time.
    pub signed_by: String,
    /// ISO-8601 date the snapshot was recorded.
    pub date: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dep_attestation_roundtrip() {
        let a = DepAttestation {
            crate_name: "serde".to_string(),
            version: "1.0.197".to_string(),
            exact_version: true,
            reviewable_artifact: PathBuf::from("docs/dep-attest/serde-1.0.197.md"),
            review_scope: ReviewScope::Full,
            signed_by: "alice".to_string(),
            date: "2026-05-22".to_string(),
            rationale: Some("Workspace baseline; reviewed at major version bump.".to_string()),
        };
        let json = serde_json::to_string(&a).unwrap();
        let back: DepAttestation = serde_json::from_str(&json).unwrap();
        assert_eq!(back.crate_name, "serde");
        assert!(back.has_reviewable_artifact());
        assert_eq!(back.review_scope, ReviewScope::Full);
    }

    #[test]
    fn dep_attestation_empty_artifact_flags() {
        let a = DepAttestation {
            crate_name: "serde".to_string(),
            version: "1.0.197".to_string(),
            exact_version: true,
            reviewable_artifact: PathBuf::new(),
            review_scope: ReviewScope::MetadataOnly,
            signed_by: "alice".to_string(),
            date: "2026-05-22".to_string(),
            rationale: None,
        };
        assert!(!a.has_reviewable_artifact());
    }

    #[test]
    fn sandbox_kind_str_roundtrip() {
        assert_eq!(SandboxKind::Build.as_str(), "build");
        assert_eq!(SandboxKind::ProcMacro.as_str(), "proc-macro");
        assert_eq!(SandboxKind::parse_kind("build"), Some(SandboxKind::Build));
        assert_eq!(
            SandboxKind::parse_kind("proc-macro"),
            Some(SandboxKind::ProcMacro)
        );
        // proc_macro underscore form also accepted (per snake-case Rust convention)
        assert_eq!(
            SandboxKind::parse_kind("proc_macro"),
            Some(SandboxKind::ProcMacro)
        );
        assert_eq!(SandboxKind::parse_kind("unknown"), None);
    }
}
