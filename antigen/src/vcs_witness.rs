//! Substrate-witness evaluation for the VCS-Information-Loss Family
//! (ADR-026 §M6 + Amendment 3 + Amendment 4).
//!
//! These are the git-substrate witness leaves referenced by the
//! VCS-info-loss stdlib antigens.
//!
//! Per the WITNESS-LAYER-INDEPENDENCE principle: the witness layer reads git
//! SUBSTRATE (commit trailers, branch state, remote configuration) and is
//! UNCHANGED by Amendment 4's correction to the hook detection trigger.
//! Amendment 4 changes *when* a witness fires (the hook's detection rule),
//! not *what* it evaluates once invoked.
//!
//! ## Library purity (ADR-002)
//!
//! The `antigen` library does NOT shell out to `git`. These evaluators take
//! already-read substrate as input — a commit's parsed trailers, a branch's
//! state, a remote's config flags — as plain data. The `cargo antigen vcs`
//! CLI in the `cargo-antigen` crate performs the actual `git` subprocess
//! reads and feeds the results here. This keeps the lib subprocess-free,
//! deterministically testable, and dependency-light (same discipline as
//! `supply_chain::manifest` reading `Cargo.toml` without a full toml parser).
//!
//! ## The four witness leaves (ADR-026 §M6)
//!
//! | Leaf | State type | Backs antigen(s) |
//! |---|---|---|
//! | `vcs_trailer_present(trailer_name)` | [`TrailerState`](crate::vcs_witness::TrailerState) | most family members (attestation-trailer presence) |
//! | `vcs_rollback_triage_chain(commit)` | [`RollbackTriageState`](crate::vcs_witness::RollbackTriageState) | `RollbackWithoutTriageCommit` |
//! | `vcs_attest_branch_deletion(branch, by_role)` | [`BranchAttestState`](crate::vcs_witness::BranchAttestState) | `BranchDeletionWithoutAttestation` |
//! | `vcs_server_side_enforcement_active(repo, antigen)` | [`ServerEnforcementState`](crate::vcs_witness::ServerEnforcementState) | structural-mode antigens (v0.2.1+) |

use serde::{Deserialize, Serialize};

use crate::vcs::TriageDecision;

// ============================================================================
// TrailerState — vcs_trailer_present(trailer_name)
// ============================================================================

/// State of a `vcs_trailer_present(trailer_name)` witness leaf.
///
/// Evaluates whether a named git commit-message trailer (e.g.,
/// `Triage-Decision:`, `Force-Push-Attestation:`, `Cherry-Picked-From:`)
/// is present on a commit. The trailer-name → value map is read by the CLI
/// via `git interpret-trailers --parse` (or equivalent) and passed here.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum TrailerState {
    /// The named trailer is present with a non-empty value. Predicate passes.
    Present {
        /// The trailer's value.
        value: String,
    },
    /// The named trailer is present but its value is empty/whitespace.
    /// A trailer key with no value is a rubber-stamp — treated as absent
    /// for discipline purposes.
    PresentButEmpty,
    /// The named trailer is absent from the commit message.
    Absent,
}

impl TrailerState {
    /// Evaluate against a commit's parsed trailers (name → value).
    #[must_use]
    pub fn evaluate(trailers: &[(String, String)], trailer_name: &str) -> Self {
        for (name, value) in trailers {
            if name.eq_ignore_ascii_case(trailer_name) {
                return if value.trim().is_empty() {
                    Self::PresentButEmpty
                } else {
                    Self::Present {
                        value: value.clone(),
                    }
                };
            }
        }
        Self::Absent
    }

    /// True when the leaf evaluates to predicate-pass (present + non-empty).
    #[must_use]
    pub const fn is_pass(&self) -> bool {
        matches!(self, Self::Present { .. })
    }
}

// ============================================================================
// RollbackTriageState — vcs_rollback_triage_chain(commit)
// ============================================================================

/// State of a `vcs_rollback_triage_chain(commit)` witness leaf
/// (ADR-026 Amendment 3 + Amendment 4).
///
/// Per Amendment 4: the detection signal is the `Triage-Decision:` git
/// TRAILER on the rollback commit itself (commit-intent), NOT a
/// codebase-presence scan for `#[triage_commit]` in source. This leaf
/// validates that a rollback commit carries a well-formed `Triage-Decision:`
/// trailer whose value resolves to a [`TriageDecision`] variant.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum RollbackTriageState {
    /// Rollback commit carries a `Triage-Decision:` trailer resolving to a
    /// valid triage variant. The triage-commit-before-rollback chain holds.
    ChainPresent {
        /// The resolved triage decision.
        decision: TriageDecision,
    },
    /// `Triage-Decision:` trailer present but its value is not a recognized
    /// [`TriageDecision`] variant. Malformed chain — the trailer exists but
    /// doesn't validate.
    ChainMalformed {
        /// The unrecognized trailer value.
        value: String,
    },
    /// No `Triage-Decision:` trailer on the rollback commit. Backs
    /// `RollbackWithoutTriageCommit` (audit hint
    /// `vcs-rollback-without-triage-commit`).
    ChainAbsent,
}

impl RollbackTriageState {
    /// Evaluate against a rollback commit's parsed trailers per Amendment 4
    /// (commit-trailer signal, not codebase-presence).
    #[must_use]
    pub fn evaluate(trailers: &[(String, String)]) -> Self {
        match TrailerState::evaluate(trailers, "Triage-Decision") {
            TrailerState::Present { value } => TriageDecision::parse_decision(&value)
                .map_or(Self::ChainMalformed { value }, |decision| {
                    Self::ChainPresent { decision }
                }),
            TrailerState::PresentButEmpty | TrailerState::Absent => Self::ChainAbsent,
        }
    }

    /// True when the rollback is properly triage-chained.
    #[must_use]
    pub const fn is_pass(&self) -> bool {
        matches!(self, Self::ChainPresent { .. })
    }
}

// ============================================================================
// BranchAttestState — vcs_attest_branch_deletion(branch, by_role)
// ============================================================================

/// State of a `vcs_attest_branch_deletion(branch, by_role)` witness leaf.
///
/// Evaluates whether a `.attest/vcs/branch-archive/<branch>.json` sidecar
/// exists recording who deleted the branch and why. The CLI reads the
/// sidecar (or its absence) and passes the result here.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum BranchAttestState {
    /// Branch-archive attestation sidecar exists with a non-empty
    /// deleting-role + rationale. Predicate passes.
    Attested {
        /// Role/name that attested the deletion.
        by_role: String,
    },
    /// Sidecar exists but the deleting-role field is empty — rubber-stamp.
    AttestedWithoutRole,
    /// No branch-archive sidecar exists. Backs
    /// `BranchDeletionWithoutAttestation`.
    SidecarMissing,
}

impl BranchAttestState {
    /// True when the branch deletion is properly attested.
    #[must_use]
    pub const fn is_pass(&self) -> bool {
        matches!(self, Self::Attested { .. })
    }
}

// ============================================================================
// ServerEnforcementState — vcs_server_side_enforcement_active(repo, antigen)
// ============================================================================

/// State of a `vcs_server_side_enforcement_active(repo, antigen)` witness
/// leaf (ADR-026 Amendment 3 Change 2; v0.2.1+).
///
/// When an antigen declares `ServerSideEnforcementMode::Structural`, the
/// audit MUST verify the remote actually has the server-side hook /
/// `receive.denyNonFastForwards` config active. v0.2 ships friction-only,
/// so this leaf is the v0.2.1+ structural-mode verification surface. The
/// CLI queries the remote config and passes the result here.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ServerEnforcementState {
    /// Remote config confirms structural enforcement is active.
    Active,
    /// `Structural` mode declared but the remote config does NOT confirm
    /// the server-side hook is active. Backs
    /// `vcs-enforcement-structural-mode-declared-but-not-active`; the
    /// antigen demotes to friction-only for audit purposes.
    DeclaredButNotActive,
    /// The remote-config query failed (network, auth, or v0.2 stub).
    /// Distinct from `DeclaredButNotActive` per ADR-026 Amendment 3 — an
    /// attacker (or a flaky network) must not be able to downgrade a
    /// not-active signal into a passing one. Backs
    /// `vcs-server-config-check-failed`.
    CheckFailed {
        /// Human-readable failure reason.
        reason: String,
    },
}

impl ServerEnforcementState {
    /// True only when structural enforcement is confirmed active. Both
    /// `DeclaredButNotActive` and `CheckFailed` are non-pass — per the
    /// honest-tier-naming discipline, an unverifiable claim is not a pass.
    #[must_use]
    pub const fn is_pass(&self) -> bool {
        matches!(self, Self::Active)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trailers(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect()
    }

    #[test]
    fn trailer_present_with_value() {
        let t = trailers(&[("Triage-Decision", "abc1234")]);
        let s = TrailerState::evaluate(&t, "Triage-Decision");
        assert!(s.is_pass());
        assert_eq!(
            s,
            TrailerState::Present {
                value: "abc1234".into()
            }
        );
    }

    #[test]
    fn trailer_case_insensitive_name_match() {
        let t = trailers(&[("triage-decision", "x")]);
        assert!(TrailerState::evaluate(&t, "Triage-Decision").is_pass());
    }

    #[test]
    fn trailer_empty_value_is_not_pass() {
        let t = trailers(&[("Triage-Decision", "   ")]);
        assert_eq!(
            TrailerState::evaluate(&t, "Triage-Decision"),
            TrailerState::PresentButEmpty
        );
        assert!(!TrailerState::evaluate(&t, "Triage-Decision").is_pass());
    }

    #[test]
    fn trailer_absent() {
        let t = trailers(&[("Some-Other", "x")]);
        assert_eq!(
            TrailerState::evaluate(&t, "Triage-Decision"),
            TrailerState::Absent
        );
    }

    #[test]
    fn rollback_triage_chain_present_resolves_decision() {
        // Amendment 4: commit-trailer signal, not codebase scan.
        let t = trailers(&[("Triage-Decision", "red")]);
        let s = RollbackTriageState::evaluate(&t);
        assert!(s.is_pass());
        assert_eq!(
            s,
            RollbackTriageState::ChainPresent {
                decision: TriageDecision::Red
            }
        );
    }

    #[test]
    fn rollback_triage_chain_malformed_value() {
        let t = trailers(&[("Triage-Decision", "purple")]);
        assert_eq!(
            RollbackTriageState::evaluate(&t),
            RollbackTriageState::ChainMalformed {
                value: "purple".into()
            }
        );
    }

    #[test]
    fn rollback_triage_chain_absent_when_no_trailer() {
        let t = trailers(&[("Reviewed-By", "alice")]);
        assert_eq!(
            RollbackTriageState::evaluate(&t),
            RollbackTriageState::ChainAbsent
        );
        assert!(!RollbackTriageState::evaluate(&t).is_pass());
    }

    #[test]
    fn branch_attest_pass_states() {
        assert!(
            BranchAttestState::Attested {
                by_role: "reviewer".into()
            }
            .is_pass()
        );
        assert!(!BranchAttestState::AttestedWithoutRole.is_pass());
        assert!(!BranchAttestState::SidecarMissing.is_pass());
    }

    #[test]
    fn server_enforcement_only_active_passes() {
        assert!(ServerEnforcementState::Active.is_pass());
        assert!(!ServerEnforcementState::DeclaredButNotActive.is_pass());
        assert!(
            !ServerEnforcementState::CheckFailed {
                reason: "network".into()
            }
            .is_pass()
        );
    }

    #[test]
    fn witness_states_serialize_kebab_tagged() {
        let s = serde_json::to_string(&RollbackTriageState::ChainAbsent).unwrap();
        assert_eq!(s, r#"{"kind":"chain-absent"}"#);
        let s2 = serde_json::to_string(&ServerEnforcementState::DeclaredButNotActive).unwrap();
        assert_eq!(s2, r#"{"kind":"declared-but-not-active"}"#);
    }
}
