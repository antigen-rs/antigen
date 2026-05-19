//! Three-axis tier-honest reporting (ADR-019 §Decision + M5).
//!
//! Extends ADR-005 Amendment 3's `WitnessTier × AuditHint` two-axis
//! reporting to three axes by adding [`EvidenceKind`] as a parallel field
//! on each per-immunity audit result. The third axis is orthogonal and
//! additive: existing reporting is unaffected unless callers opt in to
//! read the new field.
//!
//! Per-`EvidenceKind` ceiling (ADR-019 §Decision):
//! - [`EvidenceKind::TypeSystemProof`] → reaches [`WitnessTier::FormalProof`]
//! - [`EvidenceKind::Behavioral`] → reaches [`WitnessTier::Execution`]
//! - [`EvidenceKind::SubstrateState`] → reaches [`WitnessTier::Execution`];
//!   **cannot reach** [`WitnessTier::FormalProof`]
//!
//! These types are intentionally defined here in `antigen-attestation`
//! (not in `antigen::audit`) so the substrate-witness evaluator can
//! return them directly without an `antigen` crate dependency. The
//! `antigen::audit` module re-exports them so its public surface remains
//! the single integration point.

use serde::{Deserialize, Serialize};

/// The strength of evidence a witness provides for an immunity (or
/// tolerance) claim.
///
/// Re-defined here (in lock-step with `antigen::audit::WitnessTier`) so
/// `antigen-attestation` can return it without a circular dep. The two
/// definitions MUST stay structurally identical; the integration in
/// `antigen::audit` re-exports this one and the public crate-surface
/// remains stable.
///
/// Discriminants are stable per ADR-005 Amendment 3 (room for
/// `BehavioralAlignment` at 3 reserved for a future ADR).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum WitnessTier {
    /// No witness or unresolved witness. Immunity asserted without
    /// evidence. Sidecar-missing, predicate-failed, and schema-invalid
    /// all land here for substrate-witnesses.
    None = 0,
    /// Witness identifier resolves but no execution-level verification
    /// happened. Substrate-witness equivalent: predicate passes but
    /// some signature is stale (against = "current") and not refreshed.
    Reachability = 1,
    /// Witness was executed (code-witness) OR substrate predicate passes
    /// and all currency holds (substrate-witness). Per-`EvidenceKind`
    /// ceiling for `SubstrateState` (cannot upgrade to `FormalProof`).
    Execution = 2,
    // BehavioralAlignment = 3, reserved per ADR-005 Amendment 3 OQ
    /// Compile-time proof — phantom-type construction whose construction
    /// is the proof. `SubstrateState` cannot reach this tier; only
    /// `TypeSystemProof` can.
    FormalProof = 4,
}

/// What kind of evidence the witness produces.
///
/// Third axis added by ADR-019 §M5 alongside `WitnessTier` and
/// `AuditHint`. Each `ImmunityAudit` result carries an `EvidenceKind`
/// so callers can distinguish "Execution-tier because the test ran"
/// from "Execution-tier because the signed substrate is current" —
/// these are substantively different epistemic claims even when the
/// tier ordinal matches.
///
/// The variants are exhaustive at v0.1 ratification time; new evidence
/// kinds would be a v0.2+ amendment to ADR-019.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    /// No evidence kind applies. Used for the `tolerance-vibes-grade`
    /// state where no sidecar opt-in exists and no other-axis evidence
    /// has been claimed. Distinct from `None` witness tier: a witness
    /// can have `EvidenceKind::Behavioral` + `WitnessTier::None` if a
    /// test was named but not found.
    None,
    /// Compile-time / type-system proof. Phantom-type witnesses (per
    /// ADR-013) emit this. Reaches `WitnessTier::FormalProof`.
    TypeSystemProof,
    /// Runtime behavioral evidence. Test functions, proptest harnesses,
    /// external-tool delegation (clippy / kani / prusti). Reaches
    /// `WitnessTier::Execution` (after harness invocation; pre-A3
    /// stops at Reachability).
    Behavioral,
    /// On-disk substrate state. Ratified docs, signed sidecars, oracle-
    /// completion markers, git-trailer signatures. Reaches
    /// `WitnessTier::Execution` (when predicate passes + currency
    /// holds). Cannot reach `WitnessTier::FormalProof`.
    SubstrateState,
}

impl EvidenceKind {
    /// The maximum [`WitnessTier`] this evidence kind can reach. Per
    /// ADR-019 §Decision: substrate-state evidence cannot reach
    /// `FormalProof`; only type-system proof can.
    #[must_use]
    pub const fn max_tier(self) -> WitnessTier {
        match self {
            Self::None => WitnessTier::None,
            Self::TypeSystemProof => WitnessTier::FormalProof,
            Self::Behavioral | Self::SubstrateState => WitnessTier::Execution,
        }
    }

    /// `true` if `tier` is reachable under this evidence kind's ceiling.
    /// Used by audit to clamp over-eager tier assignments.
    #[must_use]
    pub fn can_reach(self, tier: WitnessTier) -> bool {
        tier <= self.max_tier()
    }
}

/// Strength of the signer-identity binding on a substrate-witness signer.
///
/// `None` for non-substrate witnesses; `Some(GitTrust)` for v0.1
/// substrate-witnesses (identity from `git config user.name + user.email`,
/// fingerprint pin via `signed_against_fingerprint`); `Some(CryptoSigned)`
/// reserved for v0.4+ DSSE + Sigstore activation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SignatureStrength {
    /// Identity bound to git config + fingerprint pin. v0.1 default.
    GitTrust,
    /// Identity bound cryptographically (DSSE-PAE-encoded; Sigstore
    /// transparency log). v0.4+ activation path; reserved on schema
    /// + tier types so activation does not require incompatible bump.
    CryptoSigned,
}

/// Per-case audit-hint disambiguation for substrate-witness results
/// (ADR-019 §M5 state-mapping tables). Parallel to (and additive to)
/// `antigen::audit::AuditHint`; both can fire on the same audit result.
///
/// Naming follows ADR-019 §M5 + adversarial T6-R (the
/// `discipline-predicate-passed-substrate-current` hint replaces
/// the v2 draft's `discipline-substrate-validated-and-current` overclaim).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum SubstrateAuditHint {
    // --- Immunity-claim substrate hints (per M5 table 1) ---
    /// No `.attest/` directory, or no sidecar for this antigen.
    DisciplineSidecarMissing,
    /// Sidecar exists but did not parse as a valid `Ratification` schema.
    DisciplineSidecarSchemaInvalid,
    /// Sidecar parsed but the substrate-witness predicate failed.
    /// Per-leaf details surface in the audit-output detail field.
    DisciplinePredicateFailed,
    /// Predicate passes but ≥1 signature is stale relative to the current
    /// fingerprint AND the leaf used `against = "current"`.
    DisciplineSubstrateStale,
    /// Predicate passes, all current, but one or more signers' chain
    /// depth is at or near the configured cap (`chain_depth >= cap - 1`).
    /// Informs the next delta will be refused; signer must do a Fresh
    /// re-attestation.
    DisciplineSubstrateDeltaChainNearCap,
    /// Predicate passes, all current, and at least one signer's basis
    /// is `DeltaFrom` (within caps). Surfaces that the attestation is
    /// carry-forward rather than fresh — informational, not a warning.
    DisciplinePredicatePassedViaDeltaChain,
    /// Predicate passes, all current, all signers' bases are `Fresh`.
    /// The strongest substrate-witness state available in v0.1.
    DisciplinePredicatePassedSubstrateCurrent,

    // --- Tolerance-claim substrate hints (per M5 table 2) ---
    /// `#[antigen_tolerance(X)]` declared without `sidecar = true` opt-in.
    /// Surfaces ADR-011's vibes-grade gap; consumers can gate CI on this
    /// hint to enforce attested-tolerance discipline.
    ToleranceVibesGrade,
    /// `#[antigen_tolerance(X, sidecar = true)]` but no sidecar exists
    /// at the expected `.attest/<Antigen>.json` location.
    ToleranceSidecarMissing,
    /// Tolerance sidecar exists but predicate failed.
    TolerancePredicateFailed,
    /// Tolerance sidecar exists, predicate passes, all signers current
    /// and Fresh. The strongest tolerance-attestation state in v0.1.
    TolerancePredicatePassedSubstrateCurrent,

    // --- Kind-mismatch hints (per adversarial TOL-A / TOL-B) ---
    /// `#[immune(X, requires = ...)]` site, sidecar exists with
    /// `kind = Tolerance` instead of expected `Immunity`. Common cause:
    /// site switched from `#[antigen_tolerance]` to `#[immune]` but the
    /// sidecar wasn't regenerated. Audit reports `WitnessTier::None` —
    /// the sidecar IS schema-valid but the kind doesn't match the
    /// declaration; this is a semantic error distinct from `schema-invalid`.
    DisciplineSidecarKindMismatchExpectedImmunityGotTolerance,
    /// `#[antigen_tolerance(X, sidecar = true, requires = ...)]` site,
    /// sidecar exists with `kind = Immunity` instead of expected
    /// `Tolerance`. Symmetric to the immunity-side kind mismatch above.
    ToleranceSidecarKindMismatchExpectedToleranceGotImmunity,

    // --- Compound-claim contradiction (per adversarial T4-A) ---
    /// Site declares BOTH `#[immune(X, ...)]` and
    /// `#[antigen_tolerance(X, sidecar = true, ...)]` for the same antigen.
    /// This is logically incoherent — a site cannot simultaneously be
    /// immune (compliant) and tolerating (non-compliant). Audit emits
    /// this hint at `WitnessTier::None` and the contradiction overrides
    /// the individual tier reports. The proc-macro should ideally
    /// reject this at compile time (TOL-C); the audit hint catches the
    /// case where compile-time guard is bypassed or sidecars are
    /// out-of-sync with declarations.
    DisciplineImmunityToleranceContradiction,
}

/// Alias: the canonical [`AuditHint`] for `antigen-attestation` is
/// [`SubstrateAuditHint`]. When integration with `antigen::audit` lands,
/// the integration layer maps these to additive entries on the broader
/// audit-hint surface; for now this alias keeps the crate's public
/// vocabulary aligned with the broader project naming.
pub use SubstrateAuditHint as AuditHint;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn witness_tier_ord_is_monotonic() {
        assert!(WitnessTier::None < WitnessTier::Reachability);
        assert!(WitnessTier::Reachability < WitnessTier::Execution);
        assert!(WitnessTier::Execution < WitnessTier::FormalProof);
    }

    #[test]
    fn evidence_kind_substrate_state_cannot_reach_formal_proof() {
        assert_eq!(
            EvidenceKind::SubstrateState.max_tier(),
            WitnessTier::Execution
        );
        assert!(!EvidenceKind::SubstrateState.can_reach(WitnessTier::FormalProof));
    }

    #[test]
    fn evidence_kind_behavioral_cannot_reach_formal_proof() {
        assert_eq!(EvidenceKind::Behavioral.max_tier(), WitnessTier::Execution);
        assert!(!EvidenceKind::Behavioral.can_reach(WitnessTier::FormalProof));
    }

    #[test]
    fn evidence_kind_type_system_proof_reaches_formal_proof() {
        assert_eq!(
            EvidenceKind::TypeSystemProof.max_tier(),
            WitnessTier::FormalProof
        );
        assert!(EvidenceKind::TypeSystemProof.can_reach(WitnessTier::FormalProof));
    }

    #[test]
    fn evidence_kind_none_max_tier_is_none() {
        assert_eq!(EvidenceKind::None.max_tier(), WitnessTier::None);
        assert!(!EvidenceKind::None.can_reach(WitnessTier::Reachability));
    }

    #[test]
    fn substrate_audit_hint_serializes_kebab_case() {
        let hint = SubstrateAuditHint::DisciplinePredicatePassedSubstrateCurrent;
        let json = serde_json::to_string(&hint).unwrap();
        assert_eq!(json, "\"discipline-predicate-passed-substrate-current\"");
    }

    #[test]
    fn tolerance_vibes_grade_hint_serializes_kebab_case() {
        let hint = SubstrateAuditHint::ToleranceVibesGrade;
        let json = serde_json::to_string(&hint).unwrap();
        assert_eq!(json, "\"tolerance-vibes-grade\"");
    }
}
