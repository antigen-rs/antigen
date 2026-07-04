//! ATK — Three-valued logic at every audit trust boundary
//! (forward/three-valued-logic-api-boundary-layer, v03-vision-buildout).
//!
//! ## The problem
//!
//! A substrate-witness predicate that contains supply-chain leaves cannot be
//! fully evaluated by the standard `audit()` path — supply-chain leaves are
//! deliberately deferred (`evaluated: false`) and must be driven by
//! `audit_supply_chain()` instead. The `CompositeVerdict` enum correctly
//! represents this as `Indeterminate` (not `Failed`), and the evaluator maps
//! it to `AuditHint::DisciplinePredicateDeferred`. So far, so good.
//!
//! The gap is one layer up: `immune_audit_is_substrate_gap()` in
//! `compute_presentation_verdicts` checks `evaluated_predicate.is_some() &&
//! witness_tier == WitnessTier::None`. This predicate is true for BOTH
//! `DisciplinePredicateFailed` (genuinely checked and failed) AND
//! `DisciplinePredicateDeferred` (not yet checked). So a site with a
//! supply-chain-leaf predicate gets flagged as `SubstrateGap` — the same
//! verdict as a site whose predicate explicitly FAILED — instead of a new
//! `Indeterminate` verdict.
//!
//! ## Three logical states at every trust boundary
//!
//!  Present/absent/malformed:
//!   - `present` — immunity or predicate is declared
//!   - `absent`  — no immunity declared (undefended)
//!   - `malformed` — declared but structurally invalid (`MalformedRequiresPredicate`)
//!
//!  Match/no-match/indeterminate:
//!   - `match`          — predicate evaluated and passed
//!   - `no-match`       — predicate evaluated and failed  → `SubstrateGap`
//!   - `indeterminate`  — predicate not yet evaluable here → needs separate audit
//!
//! Collapsing `indeterminate` → `SubstrateGap` conflates "we tried and it's broken"
//! with "we haven't tried yet." This produces false alarms on any site that
//! deliberately uses supply-chain leaves in its substrate predicate.
//!
//! ## What the tests verify
//!
//! ATK-3V-1: supply-chain-only predicate → `DisciplinePredicateDeferred`, NOT failed.
//! ATK-3V-2: standard-leaf predicate that fails → `DisciplinePredicateFailed`.
//! ATK-3V-3: mixed predicate (supply-chain + failing standard leaf) → `Failed`
//!   because the standard leaf provides a definitive answer in the standard path.
//! ATK-3V-4 (THE GAP TEST): a site with a purely-deferred predicate reports
//!   `ImmuneVerdict::Undefended` or `SubstrateGap` — but the correct answer is
//!   neither: the predicate was not evaluated (deferred). The two-valued
//!   `SubstrateGap` verdict does not represent this state faithfully.
//!   CURRENTLY: `compute_presentation_verdicts` maps deferred → `SubstrateGap`.
//!   DESIRED:   deferred should NOT produce `SubstrateGap` (the gap is in the
//!              verdict logic of `compute_presentation_verdicts`).
//!
//! ATK-3V-4 is the FAILING test. ATK-3V-1, ATK-3V-2, ATK-3V-3 document the
//! current evaluator behavior and must remain passing.

use std::collections::BTreeMap;
use std::path::Path;

use antigen_attestation::{
    evaluate::{EvaluationContext, evaluate_predicate},
    predicate::{Leaf, Predicate},
    schema::ItemRatification,
};
use chrono::NaiveDate;

/// Minimal in-memory context: no docs, no oracles, no trailers.
struct EmptyContext;

impl EvaluationContext for EmptyContext {
    fn today(&self) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()
    }

    fn read_doc(&self, _: &Path) -> Option<String> {
        None
    }

    fn read_oracle(&self, _: &Path) -> Option<String> {
        None
    }

    fn read_git_trailers(&self, _: &Path, _: &str) -> Vec<String> {
        vec![]
    }
}

fn empty_item() -> ItemRatification {
    ItemRatification {
        item_path: "test_fn".to_string(),
        current_fingerprint: "fp-test".to_string(),
        doc_ref: None,
        signers: vec![],
        oracles: vec![],
        fresh_through: None,
        extensions: BTreeMap::new(),
    }
}

// ============================================================================
// ATK-3V-1: Pure supply-chain predicate → Indeterminate (not Failed)
// ============================================================================
//
// A predicate that contains only supply-chain leaves (dep_pinned) cannot be
// evaluated by the standard path. The evaluator must return Indeterminate
// rather than Failed — the distinction is "not yet checked" vs "checked and
// failed."
//
// CURRENTLY PASSING: the EvalNode / CompositeVerdict layer is correct.
// This test documents that the evaluator itself is not the problem.
#[test]
fn atk_3v1_supply_chain_leaf_evaluates_as_indeterminate_not_failed() {
    // A single supply-chain leaf in a predicate — standard eval cannot evaluate it.
    // The evaluator returns `evaluated: false` for supply-chain leaves, which means
    // the leaf outcome is Indeterminate (deferred), not Failed.
    let pred = Predicate::leaf(Leaf::DepPinned { crate_name: None });
    let item = empty_item();
    let ctx = EmptyContext;

    let result = evaluate_predicate(&pred, &item, "fp-test", Path::new("src/lib.rs"), &ctx)
        .expect("evaluate_predicate must not return Err for supply-chain leaf");

    // The leaf is deferred — the evaluator correctly does NOT mark it as failed.
    // It emits DisciplinePredicateDeferred, not DisciplinePredicateFailed.
    assert_eq!(
        result.audit_hint,
        antigen_attestation::AuditHint::DisciplinePredicateDeferred,
        "ATK-3V-1: supply-chain-only predicate must produce DisciplinePredicateDeferred, \
         not DisciplinePredicateFailed. Collapsing deferred → failed violates \
         three-valued-logic at the trust boundary. Got: {:?}",
        result.audit_hint
    );

    assert_eq!(
        result.witness_tier,
        antigen_attestation::WitnessTier::None,
        "ATK-3V-1: deferred predicate produces WitnessTier::None (not-evaluated)"
    );
}

// ============================================================================
// ATK-3V-2: Failing standard-leaf predicate → Failed (not Indeterminate)
// ============================================================================
//
// A standard leaf that actually fails (FreshWithinDays with no signers) must
// produce DisciplinePredicateFailed, not DisciplinePredicateDeferred. These
// two states must be distinguishable.
#[test]
fn atk_3v2_failing_standard_leaf_is_failed_not_indeterminate() {
    // FreshWithinDays with no signers and no fresh_through → no date → fails.
    let pred = Predicate::leaf(Leaf::FreshWithinDays { days: 30 });
    let item = empty_item(); // no signers, no fresh_through
    let ctx = EmptyContext;

    let result = evaluate_predicate(&pred, &item, "fp-test", Path::new("src/lib.rs"), &ctx)
        .expect("evaluate_predicate returns Ok");

    assert_eq!(
        result.audit_hint,
        antigen_attestation::AuditHint::DisciplinePredicateFailed,
        "ATK-3V-2: failing standard leaf must produce DisciplinePredicateFailed, \
         not DisciplinePredicateDeferred. The three states must be distinct. \
         Got: {:?}",
        result.audit_hint
    );
}

// ============================================================================
// ATK-3V-3: Mixed all_of — deferred + failing standard leaf → Failed
// ============================================================================
//
// `all_of([dep_pinned(…), fresh_within_days(1)])` where `fresh_within_days`
// fails: the standard leaf provides a definitive "no" answer, so the verdict
// must be Failed, not Indeterminate.
//
// Biology analogy: T-cell activation requires BOTH antigen signal AND
// co-stimulation. If the antigen signal is absent (standard leaf fails),
// activation does NOT occur regardless of the co-stimulation state
// (supply-chain leaf). The T-cell doesn't enter "indeterminate" state —
// it fails the activation gate.
#[test]
fn atk_3v3_all_of_with_failing_standard_leaf_is_failed_not_indeterminate() {
    let pred = Predicate::all_of(vec![
        Predicate::leaf(Leaf::DepPinned { crate_name: None }), // deferred
        Predicate::leaf(Leaf::FreshWithinDays { days: 1 }),    // fails (no signers)
    ])
    .expect("all_of with two children is valid");
    let item = empty_item();
    let ctx = EmptyContext;

    let result = evaluate_predicate(&pred, &item, "fp-test", Path::new("src/lib.rs"), &ctx)
        .expect("evaluate_predicate returns Ok");

    assert_eq!(
        result.audit_hint,
        antigen_attestation::AuditHint::DisciplinePredicateFailed,
        "ATK-3V-3: all_of with a failing standard leaf must produce DisciplinePredicateFailed, \
         not DisciplinePredicateDeferred. A definitive-no from one leaf overrides deferral of \
         a sibling. Got: {:?}",
        result.audit_hint
    );
}

// ============================================================================
// ATK-3V-4: Deferred predicate must NOT produce SubstrateGap in the full
// audit pipeline (end-to-end fixture test).
// ============================================================================
//
// This test exercises the FULL audit pipeline:
//   scan_workspace → audit() → compute_presentation_verdicts() → PresentationVerdict
//
// Uses fixture: antigen/tests/fixtures/atk_3v4_deferred_not_substrate_gap/
//   - lib.rs declares #[immune(DeferredPredicateClass, requires = dep_pinned())]
//   - .attest/DeferredPredicateClass.json is a VALID sidecar (no signers, passes
//     schema validation) so load_sidecar() returns Ok — we reach evaluate_predicate.
//   - dep_pinned() is a supply-chain leaf → DEFERRED by the standard audit path.
//   - The correct verdict: NOT SubstrateGap (deferred != failed).
//   - Before the three-valued-logic fix: immune_audit_is_substrate_gap() returned
//     true for DisciplinePredicateDeferred, producing ImmuneVerdict::SubstrateGap.
//   - After the fix: deferred is excluded from the substrate-gap gate.
//
// FALSIFICATION CHECK: removing the `&& a.audit_hint != AuditHint::DisciplinePredicateDeferred`
// guard from immune_audit_is_substrate_gap() causes this test to FAIL (the verdict
// becomes SubstrateGap). Confirmed by observer: git stash (reverts fix) → test fails.
#[test]
fn atk_3v4_deferred_predicate_does_not_produce_substrate_gap_verdict() {
    use std::path::Path;

    use antigen::audit::{ImmuneVerdict, audit};
    use antigen::scan::scan_workspace;

    // Fixture: antigen/tests/fixtures/atk_3v4_deferred_not_substrate_gap/src/
    //   - lib.rs: #[immune(DeferredPredicateClass, requires = dep_pinned())]
    //   - .attest/DeferredPredicateClass.json: valid sidecar (no signers, passes validate())
    //
    // Flow through the audit pipeline:
    //   1. scan_workspace finds the #[immune] with dep_pinned requires_predicate
    //   2. audit() → audit_substrate_witness → load_sidecar → Ok (sidecar exists, valid)
    //   3. evaluate_predicate(dep_pinned) → Indeterminate (supply-chain leaf, DEFERRED)
    //   4. immune_audit_is_substrate_gap: with the fix, returns false (deferred excluded)
    //   5. compute_presentation_verdicts: verdict is NOT SubstrateGap
    let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("atk_3v4_deferred_not_substrate_gap")
        .join("src");

    let scan = scan_workspace(&fixture_root, None).expect("ATK-3V-4: scan of fixture must succeed");

    // Confirm the fixture immunity was captured with the dep_pinned predicate.
    let immunity = scan
        .immunities
        .iter()
        .find(|i| i.antigen_type.contains("DeferredPredicateClass"))
        .expect("ATK-3V-4: fixture must have an immunity for DeferredPredicateClass");
    assert!(
        immunity.requires_predicate.is_some(),
        "ATK-3V-4: fixture immunity must have requires_predicate (dep_pinned)"
    );

    // Run the full audit.
    let report = audit(&scan, &fixture_root);

    // Find the presentation verdict for guarded_fn.
    let verdict = report
        .presentation_verdicts
        .iter()
        .find(|v| v.antigen_type.contains("DeferredPredicateClass"))
        .map(|v| &v.verdict);

    // THE ASSERTION: verdict must NOT be SubstrateGap.
    // A deferred supply-chain predicate means "supply-chain audit needed, not
    // checked in the standard path." Before the three-valued-logic fix,
    // immune_audit_is_substrate_gap() returned true for DisciplinePredicateDeferred,
    // producing SubstrateGap. After the fix, deferred is excluded → not SubstrateGap.
    assert!(
        !matches!(verdict, Some(ImmuneVerdict::SubstrateGap)),
        "ATK-3V-4: a site with a supply-chain-only (dep_pinned) predicate must NOT \
         report SubstrateGap — the predicate is DEFERRED (not evaluated), not failed. \
         Got verdict: {:?}. \
         Fix: immune_audit_is_substrate_gap() must exclude DisciplinePredicateDeferred. \
         If this fails after the fix was applied: the guard was removed or widened.",
        verdict
    );
}
