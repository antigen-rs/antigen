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
        canonical_path: None,
        requires_predicate: None,
        structural_fingerprint: String::new(),
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
        witness_kind:
            WitnessKind::PhantomType {
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
        witness_kind: WitnessKind::PhantomType {
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
        witness_kind: WitnessKind::PhantomType {
            ref type_params, ..
        },
        ..
    } = a.witness_status
    {
        assert_eq!(
            type_params,
            &vec!["FrameTranslation".to_string(), "MyAntibody".to_string(),]
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

    if let WitnessStatus::Resolved {
        ref witness_kind, ..
    } = ignored.witness_status
    {
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

// ============================================================================
// ATK-W7-I: stacked #[immune] false-positive — code_witness_sidecar_ignored
//
// When an item has BOTH `#[immune(X, witness = ...)]` AND
// `#[immune(X, requires = ...)]`, audit produces two `ImmunityAudit` records.
// The `requires=` audit legitimately uses the `.attest/X.json` sidecar.
//
// BUG: the `witness=` audit also calls `load_sidecar()` and sets
// `code_witness_sidecar_ignored = true`, even though the sidecar is legitimately
// consumed by the companion `requires=` record. This is a false positive —
// the adopter sees a spurious "sidecar ignored" warning on their correctly-stacked
// hybrid immunity.
//
// STATUS: FAILING — `audit()` at audit.rs:1073 checks for any sidecar for the
// antigen name, regardless of whether a companion `requires=` immunity exists.
//
// Fix: before setting `code_witness_sidecar_ignored = true` for a `witness=`
// immunity, check whether any OTHER immunity in the report for the same
// antigen_type + item_target also uses `requires_predicate = Some(...)`. If so,
// the sidecar is legitimately owned by the `requires=` record and the warning
// is a false positive.
// ============================================================================

#[test]
fn atk_w7_i_stacked_immune_no_false_positive_sidecar_ignored() {
    use std::io::Write;

    // Create a temp workspace directory: src/lib.rs + src/.attest/TestAntigenStacked.json
    let dir = tempfile::TempDir::new().expect("tempdir");
    let src_dir = dir.path().join("src");
    std::fs::create_dir_all(&src_dir).expect("create src");

    // Write a minimal sidecar. load_sidecar() only needs to deserialize Ratification.
    let attest_dir = src_dir.join(".attest");
    std::fs::create_dir_all(&attest_dir).expect("create .attest");
    let sidecar_path = attest_dir.join("TestAntigenStacked.json");
    let mut f = std::fs::File::create(&sidecar_path).expect("create sidecar");
    write!(
        f,
        r#"{{
  "schema_version": "v1",
  "kind": "immunity",
  "antigen": {{ "name": "TestAntigenStacked" }},
  "source_file": "src/lib.rs",
  "items": [
    {{
      "item_path": "stacked_fn",
      "current_fingerprint": "fnv1a64:0000000000000001",
      "signers": [],
      "oracles": []
    }}
  ]
}}"#
    )
    .expect("write sidecar");
    drop(f);

    let src_file = src_dir.join("lib.rs");
    std::fs::write(&src_file, "// placeholder").expect("write lib.rs");

    // Build a ScanReport with two immunity records for the same antigen+item:
    // one witness= and one requires= (stacked on the same item).
    let witness_immunity = Immunity {
        antigen_type: "TestAntigenStacked".to_string(),
        witness: "my_test_fn".to_string(),
        requires_predicate: None,
        file: src_file.clone(),
        line: 5,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Fn("stacked_fn".to_string()),
        canonical_path: None,
        structural_fingerprint: "fnv1a64:0000000000000001".to_string(),
    };
    let requires_immunity = Immunity {
        antigen_type: "TestAntigenStacked".to_string(),
        witness: String::new(),
        // Minimal fresh_within_days predicate JSON
        requires_predicate: Some(
            r#"{"kind":"leaf","name":"fresh_within_days","days":9999}"#.to_string(),
        ),
        file: src_file,
        line: 4,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Fn("stacked_fn".to_string()),
        canonical_path: None,
        structural_fingerprint: "fnv1a64:0000000000000001".to_string(),
    };

    let mut report = ScanReport::default();
    report.immunities.push(witness_immunity);
    report.immunities.push(requires_immunity);

    let audit_report = audit(&report, dir.path());

    // Expect exactly two audit results.
    assert_eq!(
        audit_report.audits.len(),
        2,
        "stacked immune must produce 2 independent audit records; got {}",
        audit_report.audits.len()
    );

    // Find the audit for the witness= record.
    let witness_audit = audit_report
        .audits
        .iter()
        .find(|a| a.immunity.requires_predicate.is_none())
        .expect("must have a witness= audit record");

    // The sidecar IS present and IS legitimately used by the companion requires= record.
    // The witness= audit must NOT falsely flag code_witness_sidecar_ignored = true.
    assert!(
        !witness_audit.code_witness_sidecar_ignored,
        "ATK-W7-I: witness= audit must NOT set code_witness_sidecar_ignored when \
         a companion requires= immunity on the same item legitimately owns the sidecar. \
         The current code calls load_sidecar() unconditionally for every witness= record \
         and flags any present sidecar as 'ignored' — even when the sidecar is correctly \
         owned by a stacked requires= record on the same item. \
         Fix: in audit(), before setting code_witness_sidecar_ignored, check whether \
         any other Immunity in report.immunities uses requires_predicate = Some(_) for \
         the same antigen_type + item_target combination."
    );
}

// ============================================================================
// ATK-W7-H: phantom-type witness end-to-end — scan path produces spaced form
//
// The unit tests in ATK-W7-B call `audit_single("PolarityProof::<T>::verified")`
// with the compact `"::<"` form. But the scan path stores the witness via
// `val.to_token_stream().to_string()` (scan.rs:149), which uses
// `quote::ToTokens` — this renders `PolarityProof::<T>::verified` as
// `"PolarityProof :: < T > :: verified"` (spaces around all tokens).
//
// `detect_phantom_type_witness` checks `trimmed.contains("::<")` — which is
// false for the spaced form. Result: every real-user phantom-type witness is
// silently classified as `NotFound` (falls through to function-index lookup,
// finds no function named `verified`), and the immunity claim lands at
// `WitnessTier::None` instead of `WitnessTier::FormalProof`.
//
// The test exercises the full pipeline: a fixture file with a real
// `#[immune(X, witness = PolarityProof::<FrameTranslation>::verified)]`
// attribute is scanned, then audited. The witness string recorded in the
// scan report is the spaced ToTokens form; `detect_phantom_type_witness`
// must handle it.
//
// STATUS: FIXED in commit 068670d — `detect_phantom_type_witness` now
// normalizes whitespace before the `::<` sentinel check, so the spaced
// ToTokens rendering matches correctly. Test passes.
// ============================================================================

#[test]
fn atk_w7_h_phantom_type_witness_via_scan_path_lands_at_formal_proof() {
    use antigen::scan::scan_workspace;
    use std::io::Write;

    // Create a temp workspace with a phantom-type witness in an #[immune] attr.
    // The scan path will record the witness as the spaced ToTokens form.
    let dir = tempfile::TempDir::new().expect("tempdir");
    let src_path = dir.path().join("lib.rs");
    let mut f = std::fs::File::create(&src_path).expect("create lib.rs");
    write!(
        f,
        "
use antigen::immune;

struct PanickingInDrop;

#[immune(
    PanickingInDrop,
    witness = PolarityProof :: < FrameTranslation > :: verified,
)]
impl Drop for PanickingInDrop {{
    fn drop(&mut self) {{}}
}}
"
    )
    .expect("write lib.rs");
    drop(f);

    let scan_report = scan_workspace(dir.path(), None).unwrap();
    assert_eq!(
        scan_report.immunities.len(),
        1,
        "expected one immunity; got {}",
        scan_report.immunities.len()
    );

    // Confirm what the scan actually stored — it should be the spaced form.
    let recorded_witness = &scan_report.immunities[0].witness;
    eprintln!("ATK-W7-H: recorded witness = {:?}", recorded_witness);
    // The scan path uses quote::ToTokens which inserts spaces around tokens.
    // We document the exact spaced form here for observability.
    // (Do not assert the exact string — ToTokens rendering is an impl detail.)

    let audit_report = audit(&scan_report, dir.path());
    assert_eq!(audit_report.audits.len(), 1);
    let a = &audit_report.audits[0];

    eprintln!("ATK-W7-H: witness_status = {:?}", a.witness_status);
    eprintln!("ATK-W7-H: witness_tier = {:?}", a.witness_tier);

    assert!(
        matches!(
            &a.witness_status,
            WitnessStatus::Resolved {
                witness_kind: WitnessKind::PhantomType { .. },
                ..
            }
        ),
        "ATK-W7-H: phantom-type witness via scan path must resolve to \
         PhantomType kind. Got: {:?}\n\
         Root cause: scan stores witness via quote::ToTokens which inserts \
         spaces around all tokens — `PolarityProof::<T>::verified` becomes \
         `\"PolarityProof :: < T > :: verified\"`. \
         detect_phantom_type_witness checks `contains(\"::<\")` which is false \
         for the spaced form. Fix: normalize whitespace before the sentinel check.",
        a.witness_status
    );
    assert_eq!(
        a.witness_tier,
        WitnessTier::FormalProof,
        "ATK-W7-H: phantom-type witness must be FormalProof tier; got {:?}",
        a.witness_tier
    );
}

// ============================================================================
// ATK-W7-I: external-tool witness via scan path — ToTokens spacing breaks
//            `starts_with("clippy::")` sentinel
//
// Same ToTokens spacing family as ATK-W7-H. `detect_external_tool` checks
// `lower.starts_with("clippy::")`. But a user writes:
//   `witness = clippy::no_panic_in_drop`
// and `quote::ToTokens` renders that as `"clippy :: no_panic_in_drop"`.
// `"clippy :: ...".starts_with("clippy::")` is false — external tool detection
// misses, falls through to function-index lookup, returns NotFound.
//
// `contains("clippy_")` is a secondary guard but requires the user to have
// used an underscore form; the standard `clippy::` path form is the one that
// breaks.
//
// STATUS: FAILING — detect_external_tool uses starts_with("clippy::") which
// does not match the spaced ToTokens rendering.
//
// Fix: normalize whitespace in `validate_witness` before passing to
// `detect_external_tool`, or in `detect_external_tool` itself check both
// `"clippy::"` and `"clippy ::"` (or strip spaces before the check).
// ============================================================================

#[test]
fn atk_w7_i_external_tool_witness_via_scan_path_lands_at_reachability() {
    use antigen::scan::scan_workspace;
    use std::io::Write;

    let dir = tempfile::TempDir::new().expect("tempdir");
    let src_path = dir.path().join("lib.rs");
    let mut f = std::fs::File::create(&src_path).expect("create lib.rs");
    // Write the #[immune] with a clippy:: witness — ToTokens will space it.
    write!(
        f,
        "
use antigen::immune;

struct PanickingInDrop;

#[immune(
    PanickingInDrop,
    witness = clippy :: no_panic_in_drop,
)]
impl Drop for PanickingInDrop {{
    fn drop(&mut self) {{}}
}}
"
    )
    .expect("write lib.rs");
    drop(f);

    let scan_report = scan_workspace(dir.path(), None).unwrap();
    assert_eq!(
        scan_report.immunities.len(),
        1,
        "expected one immunity; got {}",
        scan_report.immunities.len()
    );

    let recorded_witness = &scan_report.immunities[0].witness;
    eprintln!("ATK-W7-I: recorded witness = {:?}", recorded_witness);

    let audit_report = audit(&scan_report, dir.path());
    let a = &audit_report.audits[0];
    eprintln!("ATK-W7-I: witness_status = {:?}", a.witness_status);

    assert!(
        matches!(&a.witness_status, WitnessStatus::External { .. }),
        "ATK-W7-I: clippy:: witness via scan path must resolve to External. \
         Got: {:?}\n\
         Root cause: ToTokens renders `clippy::no_panic_in_drop` as \
         `\"clippy :: no_panic_in_drop\"`. detect_external_tool checks \
         starts_with(\"clippy::\") which is false for the spaced form.",
        a.witness_status
    );
    assert_eq!(
        a.witness_tier,
        WitnessTier::Reachability,
        "ATK-W7-I: external-tool witness must be Reachability tier; got {:?}",
        a.witness_tier
    );
}
