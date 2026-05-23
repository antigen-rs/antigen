//! Substrate-witness leaf state types for Supply-Chain Defense Family (ADR-025).
//!
//! Each type captures what a substrate-witness leaf evaluates to for a
//! specific antigen. These are pure data types; evaluation logic lives in
//! [`crate::supply_chain::evaluate`].
//!
//! ## The five v0.2 witness leaves
//!
//! | Leaf | State type | Backs antigen(s) |
//! |---|---|---|
//! | `dep_pinned(crate?)` | [`DepPinnedState`] | `UnpinnedDependency`, `AutoDependencyChainWithoutPinning` |
//! | `dep_attested(crate, version, exact_version)` | [`DepAttestedState`] | `UnattestedDependencyInclusion`, `DependencyUpgradeWithoutDiffReview` |
//! | `maintainer_unchanged(crate, since_version)` | [`MaintainerState`] | `MaintainerChangeWithoutReattestation` |
//! | `content_hash_matches(crate, version)` | [`ContentHashState`] | `ContentHashMismatch` |
//! | `sandbox_clean(crate, sandbox_kind)` | [`SandboxState`] | `UnsandboxedBuildScript`, `UnsandboxedProcMacro` |

use serde::{Deserialize, Serialize};

use super::schema::{ReviewScope, SandboxKind};

// ============================================================================
// DepPinnedState
// ============================================================================

/// State of a `dep_pinned(crate?)` witness leaf.
///
/// When `crate` is `None`, the leaf asserts that ALL `[dependencies]`
/// entries in the manifest use exact-pin (`=X.Y.Z`) version specifiers.
/// When `crate` is `Some`, the leaf asserts just that one dep is exact-
/// pinned.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum DepPinnedState {
    /// All deps (or the named dep) are exact-pinned. Predicate passes.
    AllPinned,
    /// At least one dep is not exact-pinned (caret, tilde, wildcard, or
    /// `?` form). Predicate fails. The list names the offenders.
    Unpinned {
        /// Names of deps without exact-pin specifiers.
        unpinned_deps: Vec<String>,
    },
    /// The named crate is not present in the manifest. Treated as failure
    /// (cannot attest pinning for a dep that isn't declared).
    NotInManifest {
        /// The crate name the leaf asked about.
        crate_name: String,
    },
}

impl DepPinnedState {
    /// True when the leaf evaluates to predicate-pass.
    #[must_use]
    pub const fn is_pass(&self) -> bool {
        matches!(self, Self::AllPinned)
    }
}

// ============================================================================
// DepAttestedState
// ============================================================================

/// State of a `dep_attested(crate, version, exact_version)` witness leaf.
///
/// Evaluated against a sidecar at
/// `.attest/supply-chain/dep-attest/<crate>@<version>.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum DepAttestedState {
    /// Sidecar exists, is well-formed, has non-empty `reviewable_artifact`,
    /// and (if `exact_version=true` was requested) matches the requested
    /// version. Predicate passes.
    Attested {
        /// Scope of the recorded review.
        review_scope: ReviewScope,
    },
    /// Sidecar exists but `reviewable_artifact` is empty — rubber-stamp.
    /// Audit emits `dep-attest-without-reviewable-artifact`.
    AttestedWithoutReviewableArtifact,
    /// Sidecar is missing entirely. Audit emits
    /// `unattested-dependency-inclusion`.
    SidecarMissing,
    /// Sidecar is malformed (JSON-parse or schema-validation failure).
    SidecarMalformed {
        /// Human-readable parse error.
        error: String,
    },
    /// Sidecar's recorded version doesn't match the requested version AND
    /// `exact_version = true` was requested. The attestation is stale.
    /// Audit emits `dep-attestation-stale`.
    AttestationStale {
        /// Version recorded in the sidecar.
        attested_version: String,
        /// Version that was requested.
        requested_version: String,
    },
}

impl DepAttestedState {
    /// True when the leaf evaluates to predicate-pass.
    #[must_use]
    pub const fn is_pass(&self) -> bool {
        matches!(self, Self::Attested { .. })
    }
}

// ============================================================================
// MaintainerState
// ============================================================================

/// State of a `maintainer_unchanged(crate, since_version)` witness leaf.
///
/// Evaluated against the [`super::schema::MaintainerSnapshot`] sidecar.
/// v0.2 cannot live-query crates.io; the snapshot is the trust anchor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum MaintainerState {
    /// Snapshot exists; recorded owners match the expected set.
    /// Predicate passes.
    Unchanged,
    /// Snapshot exists; recorded owners differ from the prior recorded set
    /// (set comparison). Audit emits
    /// `maintainer-change-without-reattestation`.
    Changed {
        /// Names added since the prior snapshot.
        added: Vec<String>,
        /// Names removed since the prior snapshot.
        removed: Vec<String>,
    },
    /// Snapshot is missing entirely.
    SnapshotMissing,
    /// Live re-query against crates.io failed (v0.2: query is not yet
    /// implemented; this is the persistent v0.2 hint until v0.3+).
    /// Audit emits `crates-io-metadata-query-failed`.
    CratesIoQueryUnavailable,
}

impl MaintainerState {
    /// True when the leaf evaluates to predicate-pass.
    #[must_use]
    pub const fn is_pass(&self) -> bool {
        matches!(self, Self::Unchanged)
    }
}

// ============================================================================
// ContentHashState
// ============================================================================

/// State of a `content_hash_matches(crate, version)` witness leaf.
///
/// **THE LOAD-BEARING WITNESS** for the chalk/debug attack class.
/// Evaluated against the recorded [`super::schema::ContentHashRecord`]
/// sidecar PLUS the live `Cargo.lock` checksum (v0.2) or the registry
/// tarball SHA-256 (v0.3+).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ContentHashState {
    /// Record exists; current hash matches recorded hash. Predicate
    /// passes.
    Matches,
    /// Record exists; current hash DIFFERS from recorded hash.
    /// **The chalk/debug-class attack signal.** Audit emits
    /// `content-hash-mismatch`.
    Mismatch {
        /// Hash recorded at first-attestation.
        recorded: String,
        /// Hash sampled now.
        current: String,
    },
    /// No first-attestation record exists for this crate@version. The
    /// antigen cannot fire until the record is created via
    /// `cargo antigen verify content-hash record`. Audit emits
    /// `content-hash-no-attestation`.
    NoAttestation,
    /// Record is missing the version'd subkey, or the crate is not in
    /// `Cargo.lock`. Treated as a configuration error.
    CrateNotInLockfile {
        /// Crate name the leaf asked about.
        crate_name: String,
    },
    /// The `.attest/supply-chain/content-hash/<crate>@<version>.json` file
    /// exists but does not deserialize cleanly. Per ATK-SC-2-A this MUST
    /// NOT silently downgrade to `NoAttestation` — that would let an
    /// attacker convert a Mismatch (high-severity) into a missing-
    /// attestation warning by corrupting the sidecar. Audit emits
    /// `content-hash-sidecar-malformed`.
    SidecarMalformed {
        /// Human-readable parse error for diagnostics.
        error: String,
    },
}

impl ContentHashState {
    /// True when the leaf evaluates to predicate-pass.
    #[must_use]
    pub const fn is_pass(&self) -> bool {
        matches!(self, Self::Matches)
    }
}

// ============================================================================
// SandboxState
// ============================================================================

/// State of a `sandbox_clean(crate, sandbox_kind)` witness leaf.
///
/// **v0.4+ feature**: actual sandbox execution is deferred to tooling-
/// phase 3. v0.2 returns [`SandboxState::ToolingNotYetAvailable`] for
/// all calls — the audit hint surfaces the limitation explicitly per
/// ADR-005 Amendment 2.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum SandboxState {
    /// Sandbox ran build.rs / proc-macro and observed no out-of-bounds
    /// behavior. Predicate passes. (v0.4+)
    Clean {
        /// What kind of sandbox check this was.
        sandbox_kind: SandboxKind,
    },
    /// Sandbox detected out-of-bounds behavior (network, fs writes
    /// outside `OUT_DIR`, env mutations). Predicate fails. (v0.4+)
    Violation {
        /// What kind of sandbox check this was.
        sandbox_kind: SandboxKind,
        /// Human-readable violation summary.
        details: String,
    },
    /// v0.2: tooling not yet available — the audit emits the
    /// `unsandboxed-build-script` / `unsandboxed-proc-macro` hint as an
    /// awareness signal. Per ADR-025 §Enforcement-Surface, sandbox
    /// detection limitations are explicitly named.
    ToolingNotYetAvailable {
        /// What kind of sandbox would have been used.
        sandbox_kind: SandboxKind,
    },
}

impl SandboxState {
    /// True when the leaf evaluates to predicate-pass.
    #[must_use]
    pub const fn is_pass(&self) -> bool {
        matches!(self, Self::Clean { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_state_predicate_logic() {
        assert!(DepPinnedState::AllPinned.is_pass());
        assert!(!DepPinnedState::Unpinned {
            unpinned_deps: vec!["serde".to_string()],
        }
        .is_pass());
        assert!(!DepPinnedState::NotInManifest {
            crate_name: "missing".to_string(),
        }
        .is_pass());
    }

    #[test]
    fn dep_attested_state_predicate_logic() {
        assert!(DepAttestedState::Attested {
            review_scope: ReviewScope::Full,
        }
        .is_pass());
        assert!(!DepAttestedState::AttestedWithoutReviewableArtifact.is_pass());
        assert!(!DepAttestedState::SidecarMissing.is_pass());
    }

    #[test]
    fn content_hash_state_predicate_logic() {
        assert!(ContentHashState::Matches.is_pass());
        assert!(!ContentHashState::Mismatch {
            recorded: "a".to_string(),
            current: "b".to_string(),
        }
        .is_pass());
        assert!(!ContentHashState::NoAttestation.is_pass());
    }

    #[test]
    fn sandbox_state_v02_returns_tooling_unavailable() {
        let s = SandboxState::ToolingNotYetAvailable {
            sandbox_kind: SandboxKind::Build,
        };
        assert!(!s.is_pass());
    }
}
