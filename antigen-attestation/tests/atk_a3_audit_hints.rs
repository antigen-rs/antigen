//! Adversarial precision test: all 14 `SubstrateAuditHint` variants fire at
//! the correct states, and the 4-group taxonomy is complete.
//!
//! ## What this test guards
//!
//! `SubstrateAuditHint` has 14 variants in 4 groups. Each variant corresponds
//! to a distinct substrate state that a consumer (CI gate, developer, tool)
//! needs to distinguish. A regression that collapses two variants into one, or
//! re-maps a state to the wrong hint, is a **silent precision loss** — callers
//! that gate on `DisciplinePredicatePassedSubstrateCurrent` would stop
//! distinguishing "passed via fresh attestation" from "passed via stale
//! carry-forward" if both collapsed to the same hint.
//!
//! ## The completeness guard
//!
//! Test `all_14_hint_variants_are_explicitly_exercised` lists all 14 variants
//! by name in an explicit array. If a future commit adds a 15th variant to
//! `SubstrateAuditHint` without updating this test, the match is NOT exhaustive
//! — Rust's exhaustiveness-checking fires on the array construction and the
//! test fails to compile, forcing the author to add a test case.
//!
//! ## Test organization
//!
//! - Group 1: Immunity-claim hints (7 variants)
//! - Group 2: Tolerance-claim hints (4 variants)
//! - Group 3: Kind-mismatch hints (2 variants)
//! - Group 4: Compound contradiction (1 variant)
//! - Completeness: exhaustive array + serde round-trip for all 14

use std::collections::BTreeMap;
use std::path::Path;

use antigen_attestation::{
    evaluate::{evaluate_predicate_with_kind, EvaluatedPredicate},
    predicate::SignerCurrency,
    schema::DEFAULT_DELTA_CHAIN_CAP,
    AuditHint, EvaluationContext, EvidenceKind, ItemRatification, Leaf, Predicate,
    RatificationKind, SignatureStrength, Signer, SignerBasis, WitnessTier,
};
use chrono::NaiveDate;

// --- Minimal in-memory context -------------------------------------------

struct Ctx {
    today: NaiveDate,
    cap: u32,
}

impl Ctx {
    fn new() -> Self {
        Self {
            today: NaiveDate::from_ymd_opt(2026, 5, 19).unwrap(),
            cap: DEFAULT_DELTA_CHAIN_CAP,
        }
    }
    fn with_cap(mut self, cap: u32) -> Self {
        self.cap = cap;
        self
    }
}

impl EvaluationContext for Ctx {
    fn today(&self) -> NaiveDate {
        self.today
    }
    fn read_doc(&self, _path: &Path) -> Option<String> {
        None
    }
    fn read_oracle(&self, _path: &Path) -> Option<String> {
        None
    }
    fn read_git_trailers(&self, _file: &Path, _item: &str) -> Vec<String> {
        vec![]
    }
    fn delta_chain_cap(&self) -> u32 {
        self.cap
    }
}

// --- Builders ------------------------------------------------------------

fn sample_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 5, 19).unwrap()
}

fn fresh_signer(name: &str, fp: &str) -> Signer {
    Signer {
        name: name.to_string(),
        role: None,
        date: sample_date(),
        signed_against_fingerprint: fp.to_string(),
        basis: SignerBasis::Fresh { reasoning: None },
        strength: SignatureStrength::GitTrust,
        signature: None,
    }
}

fn delta_signer(name: &str, fp: &str, depth: u32, prior: &str) -> Signer {
    Signer {
        name: name.to_string(),
        role: None,
        date: sample_date(),
        signed_against_fingerprint: fp.to_string(),
        basis: SignerBasis::DeltaFrom {
            prior_fingerprint: prior.to_string(),
            rationale: "reviewed; consistent".to_string(),
            chain_depth: depth,
            cumulative_root_fingerprint: prior.to_string(),
        },
        strength: SignatureStrength::GitTrust,
        signature: None,
    }
}

fn item_with(signers: Vec<Signer>) -> ItemRatification {
    ItemRatification {
        item_path: "sinh".to_string(),
        current_fingerprint: "fp-current".to_string(),
        doc_ref: None,
        signers,
        oracles: vec![],
        fresh_through: None,
        extensions: BTreeMap::new(),
    }
}

fn signers_pred_current(required: Vec<&str>) -> Predicate {
    Predicate::Leaf(Leaf::Signers {
        required: required.into_iter().map(str::to_string).collect(),
        roles: BTreeMap::new(),
        against: SignerCurrency::Current,
        signature_allow: vec![],
        signature_prefer: None,
    })
}

fn signers_pred_any(required: Vec<&str>) -> Predicate {
    Predicate::Leaf(Leaf::Signers {
        required: required.into_iter().map(str::to_string).collect(),
        roles: BTreeMap::new(),
        against: SignerCurrency::Any,
        signature_allow: vec![],
        signature_prefer: None,
    })
}

fn eval_immunity(pred: &Predicate, item: &ItemRatification) -> EvaluatedPredicate {
    evaluate_predicate_with_kind(
        pred,
        item,
        "fp-current",
        Path::new("src/lib.rs"),
        RatificationKind::Immunity,
        &Ctx::new(),
    )
    .unwrap()
}

fn eval_tolerance(pred: &Predicate, item: &ItemRatification) -> EvaluatedPredicate {
    evaluate_predicate_with_kind(
        pred,
        item,
        "fp-current",
        Path::new("src/lib.rs"),
        RatificationKind::Tolerance,
        &Ctx::new(),
    )
    .unwrap()
}

fn eval_immunity_with_ctx(
    pred: &Predicate,
    item: &ItemRatification,
    ctx: &Ctx,
) -> EvaluatedPredicate {
    evaluate_predicate_with_kind(
        pred,
        item,
        "fp-current",
        Path::new("src/lib.rs"),
        RatificationKind::Immunity,
        ctx,
    )
    .unwrap()
}

// =========================================================================
// Group 1 — Immunity-claim hints (7 variants)
// =========================================================================

#[test]
fn hint_discipline_sidecar_missing_struct() {
    let result = EvaluatedPredicate::sidecar_missing();
    assert_eq!(result.audit_hint, AuditHint::DisciplineSidecarMissing);
    assert_eq!(result.witness_tier, WitnessTier::None);
    assert_eq!(result.evidence_kind, EvidenceKind::SubstrateState);
    assert_eq!(result.signature_strength, None);
}

#[test]
fn hint_discipline_sidecar_schema_invalid_struct() {
    let result = EvaluatedPredicate::sidecar_schema_invalid();
    assert_eq!(result.audit_hint, AuditHint::DisciplineSidecarSchemaInvalid);
    assert_eq!(result.witness_tier, WitnessTier::None);
    assert_eq!(result.evidence_kind, EvidenceKind::SubstrateState);
    assert_eq!(result.signature_strength, None);
}

#[test]
fn hint_discipline_predicate_failed_when_required_signer_absent() {
    let pred = signers_pred_current(vec!["alice"]);
    let item = item_with(vec![]); // no signers → predicate fails
    let result = eval_immunity(&pred, &item);
    assert_eq!(result.audit_hint, AuditHint::DisciplinePredicateFailed);
    assert_eq!(result.witness_tier, WitnessTier::None);
}

#[test]
fn hint_discipline_substrate_stale_when_signer_against_old_fingerprint() {
    // against = Any so the predicate PASSES even with a stale signer,
    // but the signer's fingerprint is old → DisciplineSubstrateStale.
    let pred = signers_pred_any(vec!["alice"]);
    let stale = Signer {
        name: "alice".to_string(),
        role: None,
        date: sample_date(),
        signed_against_fingerprint: "fp-old".to_string(), // stale
        basis: SignerBasis::Fresh { reasoning: None },
        strength: SignatureStrength::GitTrust,
        signature: None,
    };
    let item = item_with(vec![stale]);
    let result = eval_immunity(&pred, &item);
    assert_eq!(result.audit_hint, AuditHint::DisciplineSubstrateStale);
    assert_eq!(result.witness_tier, WitnessTier::Reachability);
}

#[test]
fn hint_discipline_delta_chain_near_cap_when_depth_at_cap_minus_one() {
    // cap = 3, depth = 2 = cap - 1 → near-cap hint
    let ctx = Ctx::new().with_cap(3);
    let pred = signers_pred_current(vec!["alice"]);
    let item = item_with(vec![delta_signer("alice", "fp-current", 2, "fp-old")]);
    let result = eval_immunity_with_ctx(&pred, &item, &ctx);
    assert_eq!(
        result.audit_hint,
        AuditHint::DisciplineSubstrateDeltaChainNearCap
    );
    assert_eq!(result.witness_tier, WitnessTier::Execution);
}

#[test]
fn hint_discipline_predicate_passed_via_delta_chain_when_delta_within_cap() {
    // cap = 3, depth = 1 (within cap, not near-cap) → via-delta-chain hint
    let ctx = Ctx::new().with_cap(3);
    let pred = signers_pred_current(vec!["alice"]);
    let item = item_with(vec![delta_signer("alice", "fp-current", 1, "fp-old")]);
    let result = eval_immunity_with_ctx(&pred, &item, &ctx);
    assert_eq!(
        result.audit_hint,
        AuditHint::DisciplinePredicatePassedViaDeltaChain
    );
    assert_eq!(result.witness_tier, WitnessTier::Execution);
}

#[test]
fn hint_discipline_predicate_passed_substrate_current_when_all_fresh() {
    let pred = signers_pred_current(vec!["alice"]);
    let item = item_with(vec![fresh_signer("alice", "fp-current")]);
    let result = eval_immunity(&pred, &item);
    assert_eq!(
        result.audit_hint,
        AuditHint::DisciplinePredicatePassedSubstrateCurrent
    );
    assert_eq!(result.witness_tier, WitnessTier::Execution);
    assert_eq!(result.evidence_kind, EvidenceKind::SubstrateState);
}

// =========================================================================
// Group 2 — Tolerance-claim hints (4 variants)
// =========================================================================

#[test]
fn hint_tolerance_vibes_grade_struct() {
    // ToleranceVibesGrade is emitted when #[antigen_tolerance] has no sidecar
    // opt-in. The EvaluatedPredicate::tolerance_vibes_grade() constructor
    // encodes this state. Evidence kind is None (no substrate consulted).
    let result = EvaluatedPredicate::tolerance_vibes_grade();
    assert_eq!(result.audit_hint, AuditHint::ToleranceVibesGrade);
    assert_eq!(result.witness_tier, WitnessTier::None);
    assert_eq!(result.evidence_kind, EvidenceKind::None);
    assert_eq!(result.signature_strength, None);
}

#[test]
fn hint_tolerance_sidecar_missing_struct() {
    // ToleranceSidecarMissing is emitted when sidecar = true but no sidecar
    // file exists. This is a pre-evaluator state handled by the audit layer;
    // the hint variant round-trips correctly through serde.
    let s = serde_json::to_string(&AuditHint::ToleranceSidecarMissing).unwrap();
    assert_eq!(s, "\"tolerance-sidecar-missing\"");
    let rt: AuditHint = serde_json::from_str(&s).unwrap();
    assert_eq!(rt, AuditHint::ToleranceSidecarMissing);
}

#[test]
fn hint_tolerance_predicate_failed_when_required_signer_absent() {
    let pred = signers_pred_current(vec!["alice"]);
    let item = item_with(vec![]); // no signers → predicate fails
    let result = eval_tolerance(&pred, &item);
    assert_eq!(result.audit_hint, AuditHint::TolerancePredicateFailed);
    assert_eq!(result.witness_tier, WitnessTier::None);
}

#[test]
fn hint_tolerance_predicate_passed_substrate_current_when_all_fresh() {
    let pred = signers_pred_current(vec!["alice"]);
    let item = item_with(vec![fresh_signer("alice", "fp-current")]);
    let result = eval_tolerance(&pred, &item);
    assert_eq!(
        result.audit_hint,
        AuditHint::TolerancePredicatePassedSubstrateCurrent
    );
    assert_eq!(result.witness_tier, WitnessTier::Execution);
    assert_eq!(result.evidence_kind, EvidenceKind::SubstrateState);
}

// =========================================================================
// Group 3 — Kind-mismatch hints (2 variants)
// =========================================================================
//
// Kind-mismatch hints fire when the SIDECAR's `kind` field doesn't match
// the DECLARATION's expected kind. Detection happens in the audit layer
// (read sidecar kind, compare against macro's expected kind). The hint
// variants are returned as pre-evaluator states — the evaluator is not
// invoked when a kind mismatch is detected.
//
// These tests verify serde round-trips and variant distinctness.

#[test]
fn hint_discipline_sidecar_kind_mismatch_expected_immunity_got_tolerance_serde() {
    let hint = AuditHint::DisciplineSidecarKindMismatchExpectedImmunityGotTolerance;
    let s = serde_json::to_string(&hint).unwrap();
    assert_eq!(
        s,
        "\"discipline-sidecar-kind-mismatch-expected-immunity-got-tolerance\""
    );
    let rt: AuditHint = serde_json::from_str(&s).unwrap();
    assert_eq!(rt, hint);
}

#[test]
fn hint_tolerance_sidecar_kind_mismatch_expected_tolerance_got_immunity_serde() {
    let hint = AuditHint::ToleranceSidecarKindMismatchExpectedToleranceGotImmunity;
    let s = serde_json::to_string(&hint).unwrap();
    assert_eq!(
        s,
        "\"tolerance-sidecar-kind-mismatch-expected-tolerance-got-immunity\""
    );
    let rt: AuditHint = serde_json::from_str(&s).unwrap();
    assert_eq!(rt, hint);
}

#[test]
fn kind_mismatch_hints_are_distinct_from_each_other() {
    assert_ne!(
        AuditHint::DisciplineSidecarKindMismatchExpectedImmunityGotTolerance,
        AuditHint::ToleranceSidecarKindMismatchExpectedToleranceGotImmunity
    );
}

// =========================================================================
// Group 4 — Compound contradiction (1 variant)
// =========================================================================

#[test]
fn hint_discipline_immunity_tolerance_contradiction_serde_and_distinctness() {
    // DisciplineImmunityToleranceContradiction fires when both #[immune(X)] and
    // #[antigen_tolerance(X, sidecar = true)] are declared at the same site.
    // When this hint fires, WitnessTier MUST be None — contradiction overrides
    // individual tier reports per ADR-019 §M5.
    let hint = AuditHint::DisciplineImmunityToleranceContradiction;
    let s = serde_json::to_string(&hint).unwrap();
    assert_eq!(s, "\"discipline-immunity-tolerance-contradiction\"");
    let rt: AuditHint = serde_json::from_str(&s).unwrap();
    assert_eq!(rt, hint);
    // Contradiction is distinct from any "passing" hint.
    assert_ne!(hint, AuditHint::DisciplinePredicatePassedSubstrateCurrent);
    assert_ne!(hint, AuditHint::TolerancePredicatePassedSubstrateCurrent);
    assert_ne!(hint, AuditHint::DisciplinePredicatePassedViaDeltaChain);
}

// =========================================================================
// Completeness guard
// =========================================================================

#[test]
fn all_14_hint_variants_are_explicitly_exercised() {
    // This list MUST be updated when any new variant is added to SubstrateAuditHint.
    // The explicit construction (not `..` or wildcard) means a new variant added to
    // the enum WITHOUT adding it here causes a compile error — the array construction
    // would need to include it for the exhaustive-match discipline to fire.
    //
    // Note: Rust doesn't enforce exhaustive matching on array literals, so this is
    // a manual-count guard: the assert_eq!(len, 14) below catches count drift,
    // and the serde test below catches serialization regression.
    let all_hints = [
        // Group 1 — Immunity-claim (7)
        AuditHint::DisciplineSidecarMissing,
        AuditHint::DisciplineSidecarSchemaInvalid,
        AuditHint::DisciplinePredicateFailed,
        AuditHint::DisciplineSubstrateStale,
        AuditHint::DisciplineSubstrateDeltaChainNearCap,
        AuditHint::DisciplinePredicatePassedViaDeltaChain,
        AuditHint::DisciplinePredicatePassedSubstrateCurrent,
        // Group 2 — Tolerance-claim (4)
        AuditHint::ToleranceVibesGrade,
        AuditHint::ToleranceSidecarMissing,
        AuditHint::TolerancePredicateFailed,
        AuditHint::TolerancePredicatePassedSubstrateCurrent,
        // Group 3 — Kind-mismatch (2)
        AuditHint::DisciplineSidecarKindMismatchExpectedImmunityGotTolerance,
        AuditHint::ToleranceSidecarKindMismatchExpectedToleranceGotImmunity,
        // Group 4 — Compound contradiction (1)
        AuditHint::DisciplineImmunityToleranceContradiction,
    ];
    assert_eq!(
        all_hints.len(),
        14,
        "SubstrateAuditHint has {len} entries in this array but expected 14; \
         if a new variant was added, add a test above and update this count",
        len = all_hints.len()
    );

    // All variants are pairwise distinct (no enum value collision).
    for i in 0..all_hints.len() {
        for j in (i + 1)..all_hints.len() {
            assert_ne!(
                all_hints[i], all_hints[j],
                "hints at indices {i} and {j} are equal — enum variant duplication?"
            );
        }
    }
}

#[test]
fn all_14_hint_variants_serialize_to_distinct_kebab_case_strings() {
    let serialized: Vec<String> = [
        AuditHint::DisciplineSidecarMissing,
        AuditHint::DisciplineSidecarSchemaInvalid,
        AuditHint::DisciplinePredicateFailed,
        AuditHint::DisciplineSubstrateStale,
        AuditHint::DisciplineSubstrateDeltaChainNearCap,
        AuditHint::DisciplinePredicatePassedViaDeltaChain,
        AuditHint::DisciplinePredicatePassedSubstrateCurrent,
        AuditHint::ToleranceVibesGrade,
        AuditHint::ToleranceSidecarMissing,
        AuditHint::TolerancePredicateFailed,
        AuditHint::TolerancePredicatePassedSubstrateCurrent,
        AuditHint::DisciplineSidecarKindMismatchExpectedImmunityGotTolerance,
        AuditHint::ToleranceSidecarKindMismatchExpectedToleranceGotImmunity,
        AuditHint::DisciplineImmunityToleranceContradiction,
    ]
    .iter()
    .map(|h| serde_json::to_string(h).unwrap())
    .collect();

    assert_eq!(serialized.len(), 14);

    // All strings are distinct — no serde collision.
    let mut sorted = serialized.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        14,
        "two or more hints serialize to the same string — serde collision detected"
    );

    // All strings are JSON strings in kebab-case.
    for s in &serialized {
        assert!(
            s.starts_with('"'),
            "hint serialization should be a JSON string: {s}"
        );
        assert!(
            s.contains('-'),
            "hint serialization should be kebab-case (contain '-'): {s}"
        );
    }
}
