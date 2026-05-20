//! Adversarial precision test for the three-axis tier-honest reporting system.
//!
//! ## What this test guards
//!
//! ADR-019 §Decision commits to per-`EvidenceKind` tier ceilings:
//!
//! - `SubstrateState` → max `Execution`; cannot reach `FormalProof`
//! - `Behavioral` → max `Execution`
//! - `TypeSystemProof` → max `FormalProof`
//! - `None` → max `None`
//!
//! These ceilings are load-bearing promises to callers. If a future refactor
//! accidentally raises `SubstrateState`'s ceiling to `FormalProof`, callers
//! relying on the `SubstrateState ≤ Execution` invariant would silently
//! overclaim. This test file catches that regression at the exact boundary.
//!
//! The `can_reach` helper is tested alongside `max_tier` — a regression in
//! either without the other would still be caught.
//!
//! ## The epistemic distinction
//!
//! Two audit results can both report `WitnessTier::Execution` while meaning
//! different things:
//! - `EvidenceKind::Behavioral + Execution` — a test function ran
//! - `EvidenceKind::SubstrateState + Execution` — a signed substrate is current
//!
//! The ceiling contract enforces that this distinction can never be erased by
//! substrate-witness predicate logic returning `FormalProof`. A phantom-type
//! witness (compile-time proof) IS allowed to return `FormalProof`; the
//! substrate-witness evaluator is NOT.

use antigen_attestation::{EvidenceKind, WitnessTier};

// --- Ceiling contract (const assertions where possible) ---

#[test]
fn evidence_kind_substrate_state_ceiling_is_execution() {
    assert_eq!(EvidenceKind::SubstrateState.max_tier(), WitnessTier::Execution);
}

#[test]
fn evidence_kind_behavioral_ceiling_is_execution() {
    assert_eq!(EvidenceKind::Behavioral.max_tier(), WitnessTier::Execution);
}

#[test]
fn evidence_kind_type_system_proof_ceiling_is_formal_proof() {
    assert_eq!(
        EvidenceKind::TypeSystemProof.max_tier(),
        WitnessTier::FormalProof
    );
}

#[test]
fn evidence_kind_none_ceiling_is_none() {
    assert_eq!(EvidenceKind::None.max_tier(), WitnessTier::None);
}

// --- SubstrateState hard exclusion ---

#[test]
fn substrate_state_cannot_reach_formal_proof() {
    assert!(
        !EvidenceKind::SubstrateState.can_reach(WitnessTier::FormalProof),
        "SubstrateState evidence cannot reach FormalProof — ADR-019 §Decision ceiling"
    );
}

#[test]
fn substrate_state_can_reach_execution() {
    assert!(EvidenceKind::SubstrateState.can_reach(WitnessTier::Execution));
}

#[test]
fn substrate_state_can_reach_reachability() {
    assert!(EvidenceKind::SubstrateState.can_reach(WitnessTier::Reachability));
}

#[test]
fn substrate_state_can_reach_none() {
    assert!(EvidenceKind::SubstrateState.can_reach(WitnessTier::None));
}

// --- TypeSystemProof positive ceiling ---

#[test]
fn type_system_proof_can_reach_formal_proof() {
    assert!(EvidenceKind::TypeSystemProof.can_reach(WitnessTier::FormalProof));
}

#[test]
fn type_system_proof_can_reach_execution() {
    assert!(EvidenceKind::TypeSystemProof.can_reach(WitnessTier::Execution));
}

// --- WitnessTier ordinal monotonicity ---
//
// The tier ordering is relied on by `can_reach` (which uses `<=`) and by
// callers that gate CI on minimum tiers. An accidental discriminant swap
// would silently corrupt these comparisons.

#[test]
fn witness_tier_ordinal_monotonic_none_lt_reachability() {
    assert!(WitnessTier::None < WitnessTier::Reachability);
}

#[test]
fn witness_tier_ordinal_monotonic_reachability_lt_execution() {
    assert!(WitnessTier::Reachability < WitnessTier::Execution);
}

#[test]
fn witness_tier_ordinal_monotonic_execution_lt_formal_proof() {
    assert!(WitnessTier::Execution < WitnessTier::FormalProof);
}

// BehavioralAlignment = 3 is reserved; FormalProof = 4. If someone adds
// BehavioralAlignment and accidentally swaps discriminants, this catches it.
#[test]
fn witness_tier_formal_proof_discriminant_is_four() {
    assert_eq!(WitnessTier::FormalProof as u32, 4);
}

#[test]
fn witness_tier_execution_discriminant_is_two() {
    assert_eq!(WitnessTier::Execution as u32, 2);
}

// --- Behavioral ceiling mirrors SubstrateState ---
//
// Both Behavioral and SubstrateState cap at Execution. Behavioral witnesses
// could in theory evolve to FormalProof (e.g., a model-checker integration
// that produces a proof certificate); that would require a new EvidenceKind,
// not relaxing Behavioral's ceiling. This test ensures the two ceilings stay
// equal at Execution until such an amendment lands.

#[test]
fn behavioral_and_substrate_state_have_same_ceiling() {
    assert_eq!(
        EvidenceKind::Behavioral.max_tier(),
        EvidenceKind::SubstrateState.max_tier()
    );
}

// --- Serde round-trip for EvidenceKind ---
//
// ADR-019 §M5 lists EvidenceKind values in the three-axis audit output.
// The kebab-case serialization is load-bearing for JSON compatibility.

#[test]
fn evidence_kind_substrate_state_serializes_snake_case() {
    let s = serde_json::to_string(&EvidenceKind::SubstrateState).unwrap();
    assert_eq!(s, "\"substrate_state\"");
}

#[test]
fn evidence_kind_type_system_proof_serializes_snake_case() {
    let s = serde_json::to_string(&EvidenceKind::TypeSystemProof).unwrap();
    assert_eq!(s, "\"type_system_proof\"");
}

#[test]
fn evidence_kind_behavioral_serializes_snake_case() {
    let s = serde_json::to_string(&EvidenceKind::Behavioral).unwrap();
    assert_eq!(s, "\"behavioral\"");
}

#[test]
fn evidence_kind_none_serializes_snake_case() {
    let s = serde_json::to_string(&EvidenceKind::None).unwrap();
    assert_eq!(s, "\"none\"");
}
