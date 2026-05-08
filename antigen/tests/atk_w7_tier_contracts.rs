//! ATK-W7 contracts for the four-tier `WitnessTier` + `AuditHint` model.
//!
//! Per ADR-005 Amendment 3 (ratified 2026-05-08, commit 817afd0): `WitnessTier`
//! has exactly four variants — `None | Reachability | Execution | FormalProof`
//! — with discriminant 3 reserved for `BehavioralAlignment`. Per-case
//! verification disambiguation lives on a parallel `AuditHint` axis. The
//! tier reports only work the audit *actually performed*; the hint carries
//! the diagnostic detail that the ordinal cannot.
//!
//! These tests pin the W7 implementation against the Amendment 3 mapping
//! table (lines 920–928 of docs/decisions.md). They are the regression
//! shield for the five silent deviations aristotle's Phase 1-8 surfaced in
//! scout's pre-amendment design draft.

use antigen::audit::{audit, AuditHint, ImmunityAudit, WitnessKind, WitnessStatus, WitnessTier};
use antigen::scan::{scan_workspace, Immunity, ItemTarget, ScanReport};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

/// Synthesize a `ScanReport` with a single `Immunity` for unit-style tier tests.
fn synthetic_immunity(witness: &str) -> ScanReport {
    let immunity = Immunity {
        antigen_type: "PanickingInDrop".to_string(),
        witness: witness.to_string(),
        file: PathBuf::from("src/lib.rs"),
        line: 10,
        item_kind: "impl".to_string(),
        item_target: ItemTarget::Unknown { line: 10 },
    };
    let mut report = ScanReport::default();
    report.immunities.push(immunity);
    report
}

/// Run a single-immunity audit and return the one `ImmunityAudit`.
fn audit_single(witness: &str) -> ImmunityAudit {
    let report = synthetic_immunity(witness);
    let root = fixture("atk_a2_003_empty_witness");
    let report = audit(&report, &root);
    assert_eq!(report.audits.len(), 1);
    report.audits.into_iter().next().unwrap()
}

// ============================================================================
// ATK-W7-A: Tier ordering — discriminants per ADR-005 Amendment 3
//
// The strict invariant from Amendment 3: `None < Reachability < Execution <
// FormalProof`, with discriminant 3 reserved for `BehavioralAlignment`.
// Discriminants are stable: 0, 1, 2, 4 (skip 3).
// ============================================================================

#[test]
fn atk_w7_a_tier_ordering_is_total_and_strict() {
    assert!(WitnessTier::None < WitnessTier::Reachability);
    assert!(WitnessTier::Reachability < WitnessTier::Execution);
    assert!(WitnessTier::Execution < WitnessTier::FormalProof);
}

#[test]
fn atk_w7_a_tier_discriminants_are_stable() {
    // Per Amendment 3: stable integer values for serde + binary representation.
    // 3 is reserved for BehavioralAlignment.
    assert_eq!(WitnessTier::None as u8, 0);
    assert_eq!(WitnessTier::Reachability as u8, 1);
    assert_eq!(WitnessTier::Execution as u8, 2);
    assert_eq!(WitnessTier::FormalProof as u8, 4);
}

// ============================================================================
// ATK-W7-B: Status → (Tier, Hint) mapping table per Amendment 3
//
// The mapping is the binding spec from ADR-005 Amendment 3 §Mechanics §2.
// Each row in the Phase 6 table is one assertion here.
// ============================================================================

#[test]
fn atk_w7_b_missing_status_is_none_tier() {
    let a = audit_single("");
    assert!(matches!(a.witness_status, WitnessStatus::Missing));
    assert_eq!(a.witness_tier, WitnessTier::None);
    assert_eq!(a.audit_hint, AuditHint::NoneApplicable);
    assert!(!a.is_well_formed());
}

#[test]
fn atk_w7_b_not_found_is_none_tier() {
    let a = audit_single("absolutely_no_such_function_anywhere_4242");
    assert!(matches!(a.witness_status, WitnessStatus::NotFound { .. }));
    assert_eq!(a.witness_tier, WitnessTier::None);
    assert_eq!(a.audit_hint, AuditHint::NoneApplicable);
}

#[test]
fn atk_w7_b_external_clippy_is_reachability_with_prefix_hint() {
    let a = audit_single("clippy::nonexistent_lint_for_test");
    assert!(matches!(a.witness_status, WitnessStatus::External { .. }));
    assert_eq!(a.witness_tier, WitnessTier::Reachability);
    assert_eq!(a.audit_hint, AuditHint::ExternalToolPrefixRecognized);
    assert!(!a.is_well_formed());
}

#[test]
fn atk_w7_b_external_kani_is_reachability_with_prefix_hint() {
    let a = audit_single("kani::my_proof");
    assert!(matches!(a.witness_status, WitnessStatus::External { .. }));
    assert_eq!(a.witness_tier, WitnessTier::Reachability);
    assert_eq!(a.audit_hint, AuditHint::ExternalToolPrefixRecognized);
}

#[test]
fn atk_w7_b_phantom_type_shape_is_formal_proof_with_recognized_hint() {
    // Witness shape: `PolarityProof::<FrameTranslation>::verified` —
    // turbofish present, so detect_phantom_type_witness fires before the
    // function-index lookup.
    let a = audit_single("PolarityProof::<FrameTranslation>::verified");
    if let WitnessStatus::Resolved {
        witness_kind: WitnessKind::PhantomType {
            ref proof_type,
            ref type_params,
            ref constructor,
        },
        ..
    } = a.witness_status
    {
        assert_eq!(proof_type, "PolarityProof");
        assert_eq!(type_params, &vec!["FrameTranslation".to_string()]);
        assert_eq!(constructor.as_deref(), Some("verified"));
    } else {
        panic!(
            "expected Resolved {{ PhantomType }}; got {:?}",
            a.witness_status
        );
    }
    assert_eq!(a.witness_tier, WitnessTier::FormalProof);
    assert_eq!(a.audit_hint, AuditHint::PhantomTypeShapeRecognized);
    assert!(a.is_well_formed());
    assert!(a.meets_tier(WitnessTier::FormalProof));
}

#[test]
fn atk_w7_b_phantom_type_no_constructor_still_recognized() {
    // Witness shape without constructor: `PolarityProof::<FrameTranslation>`.
    let a = audit_single("PolarityProof::<FrameTranslation>");
    if let WitnessStatus::Resolved {
        witness_kind:
            WitnessKind::PhantomType {
                ref constructor, ..
            },
        ..
    } = a.witness_status
    {
        assert_eq!(constructor.as_deref(), None);
    } else {
        panic!(
            "expected Resolved {{ PhantomType }}; got {:?}",
            a.witness_status
        );
    }
    assert_eq!(a.witness_tier, WitnessTier::FormalProof);
}

#[test]
fn atk_w7_b_phantom_type_with_multiple_params() {
    let a = audit_single("Witnessed::<FrameTranslation, MyAntibody>::new");
    if let WitnessStatus::Resolved {
        witness_kind: WitnessKind::PhantomType { ref type_params, .. },
        ..
    } = a.witness_status
    {
        assert_eq!(
            type_params,
            &vec![
                "FrameTranslation".to_string(),
                "MyAntibody".to_string(),
            ]
        );
    } else {
        panic!("expected Resolved {{ PhantomType }}");
    }
}

// ============================================================================
// ATK-W7-C: Test classification — #[test] vs #[test] #[ignore]
//
// detect_kind discriminates the two cases per ATK-A2-012. v0.1 maps both to
// Reachability tier (audit doesn't invoke cargo test); the audit_hint axis
// distinguishes them so reports remain informative.
// ============================================================================

#[test]
fn atk_w7_c_ignored_test_distinguished_by_hint_axis() {
    // Direct test: `not_yet_ready_witness` is #[test] #[ignore] in
    // fixtures/atk_a2_012_ignored_test_witness/. We run the audit with that
    // fixture as the workspace root so the function index sees the ignored
    // test.
    let report = synthetic_immunity("not_yet_ready_witness");
    let root = fixture("atk_a2_012_ignored_test_witness");
    let r = audit(&report, &root);
    let ignored = r.audits.into_iter().next().unwrap();

    if let WitnessStatus::Resolved { ref witness_kind, .. } = ignored.witness_status {
        assert_eq!(*witness_kind, WitnessKind::IgnoredTest);
    } else {
        panic!(
            "expected Resolved for not_yet_ready_witness; got {:?}",
            ignored.witness_status
        );
    }
    assert_eq!(ignored.witness_tier, WitnessTier::Reachability);
    assert_eq!(
        ignored.audit_hint,
        AuditHint::TestAttributePresentIgnoreSkipped,
    );
}

// ============================================================================
// ATK-W7-D: External-tool detection wins over phantom-type and function-index
//
// Resolution priority per audit.rs `validate_witness`:
//   1. empty → Missing
//   2. external-tool prefix → External (Reachability + ExternalToolPrefixRecognized)
//   3. turbofish present → PhantomType (FormalProof + PhantomTypeShapeRecognized)
//   4. function-index → Resolved / Ambiguous / NotFound
//
// A witness like `kani::SomeProof::<T>::ctor` matches BOTH external-tool
// (kani:: prefix) and phantom-type-shape (turbofish present). External wins
// because the developer explicitly opted into the kani toolchain by writing
// the prefix; the audit reports Reachability + ExternalToolPrefixRecognized
// until A3+ runs the tool.
// ============================================================================

#[test]
fn atk_w7_d_external_prefix_wins_over_turbofish_shape() {
    let a = audit_single("kani::SomeProof::<MyAntigen>::verified");
    assert!(
        matches!(a.witness_status, WitnessStatus::External { .. }),
        "expected External (kani:: wins); got {:?}",
        a.witness_status
    );
    assert_eq!(a.witness_tier, WitnessTier::Reachability);
    assert_eq!(a.audit_hint, AuditHint::ExternalToolPrefixRecognized);
}

// ============================================================================
// ATK-W7-E: Audit report aggregate counts
//
// AuditReport carries per-category counts including the new `ambiguous_count`
// and `all_meet_tier` predicate. The CI gate `all_valid()` requires all
// claims to meet Execution; `all_meet_tier(Reachability)` is the looser
// gate ("at least the witness identifier exists somewhere").
// ============================================================================

#[test]
fn atk_w7_e_all_valid_requires_execution_tier() {
    let report = synthetic_immunity("clippy::some_lint");
    let r = audit(&report, &fixture("atk_a2_003_empty_witness"));
    // External lint = Reachability; below Execution.
    assert!(!r.all_valid());
    assert!(r.all_meet_tier(WitnessTier::Reachability));
    assert!(!r.all_meet_tier(WitnessTier::Execution));
}

#[test]
fn atk_w7_e_phantom_type_witness_passes_strict_gate() {
    let report = synthetic_immunity("Proof::<X>::verified");
    let r = audit(&report, &fixture("atk_a2_003_empty_witness"));
    // FormalProof tier passes Execution and FormalProof gates.
    assert!(r.all_valid());
    assert!(r.all_meet_tier(WitnessTier::Execution));
    assert!(r.all_meet_tier(WitnessTier::FormalProof));
}

// ============================================================================
// ATK-W7-F: Ambiguous resolution — full audit pipeline against a real fixture
//
// This is the structural-collision check from ATK-A2-005, restated as a W7
// tier-aware contract: the witness `verify_boundary` resolves to two
// candidates, the audit emits Ambiguous, the tier is None.
// ============================================================================

#[test]
fn atk_w7_f_ambiguous_collision_lands_at_none_tier() {
    let fixture_root = fixture("atk_a2_005_scope_cross_reactive");
    let scan = scan_workspace(&fixture_root, None).unwrap();
    let r = audit(&scan, &fixture_root);
    assert_eq!(r.audits.len(), 1);
    let a = &r.audits[0];
    assert!(matches!(a.witness_status, WitnessStatus::Ambiguous { .. }));
    assert_eq!(a.witness_tier, WitnessTier::None);
    assert_eq!(a.audit_hint, AuditHint::AmbiguousResolution);
    assert!(!a.is_well_formed());
}

// ============================================================================
// ATK-W7-G: serde round-trip for tier and hint
//
// Both fields appear in the JSON output per ADR-005 Amendment 3 §Enforcement.
// The serde-renamed forms are snake_case for tiers and kebab-case for hints
// (matches the rest of the antigen vocabulary).
// ============================================================================

#[test]
fn atk_w7_g_tier_serializes_snake_case() {
    let tier = WitnessTier::Reachability;
    let s = serde_json::to_string(&tier).unwrap();
    assert_eq!(s, "\"reachability\"");
    let back: WitnessTier = serde_json::from_str(&s).unwrap();
    assert_eq!(back, WitnessTier::Reachability);
}

#[test]
fn atk_w7_g_hint_serializes_kebab_case() {
    let hint = AuditHint::TestAttributePresentNotInvoked;
    let s = serde_json::to_string(&hint).unwrap();
    assert_eq!(s, "\"test-attribute-present-not-invoked\"");
    let back: AuditHint = serde_json::from_str(&s).unwrap();
    assert_eq!(back, AuditHint::TestAttributePresentNotInvoked);
}

#[test]
fn atk_w7_g_immunity_audit_round_trips() {
    let a = audit_single("clippy::some_lint");
    let json = serde_json::to_string(&a).unwrap();
    assert!(
        json.contains("\"reachability\""),
        "JSON must contain witness_tier; got: {json}",
    );
    assert!(
        json.contains("\"external-tool-prefix-recognized\""),
        "JSON must contain audit_hint; got: {json}",
    );
}
