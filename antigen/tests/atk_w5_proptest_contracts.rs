//! ATK-W5 pre-implementation contracts for structural proptest! witness detection.
//!
//! W5 replaces the textual `source.contains("proptest!")` heuristic with
//! structural detection: walk macro invocations, identify `proptest!` bodies,
//! scan for inner function names, register as `WitnessKind::Proptest`.
//!
//! All tests are `#[ignore]` until W5 lands. When W5 ships:
//! 1. Remove #[ignore] from each test
//! 2. Verify it FAILS (confirming the contract is real pre-fix)
//! 3. Fix the implementation to pass
//!
//! Substrate check: `cargo test --package antigen --test atk_w5_proptest_contracts`

use antigen::audit::{audit, WitnessKind, WitnessStatus};
use antigen::scan::{scan_workspace, Immunity, ScanReport};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

// ============================================================================
// ATK-W5-001: Function inside proptest! block must be classified as Proptest
//
// The structural bug: visit_item_fn doesn't descend into macro invocations.
// `real_proptest_fn` is defined inside `proptest::proptest! { ... }` and is
// not a top-level ItemFn — the visitor never sees it.
//
// W5 must add visit_item_macro (or equivalent) that:
//   - Recognizes the `proptest` macro path
//   - Parses the token body for fn declarations
//   - Registers them with WitnessKind::Proptest
//
// Current behavior: `real_proptest_fn` is NotFound (visitor never indexed it)
// Expected after W5: WitnessStatus::Resolved { witness_kind: Proptest }
// ============================================================================

#[test]
fn atk_w5_001_proptest_inner_function_is_detected() {
    let fixture_root = fixture("atk_w5_proptest_detection");
    let scan = scan_workspace(&fixture_root, None).unwrap();
    let audit_report = audit(&scan, &fixture_root);

    assert_eq!(audit_report.audits.len(), 1, "fixture has one immunity");
    let a = &audit_report.audits[0];

    // The witness real_proptest_fn lives inside a proptest! block.
    // W5 must find it and classify it as Proptest kind.
    match &a.witness_status {
        WitnessStatus::Resolved { witness_kind, .. } => {
            assert_eq!(
                *witness_kind,
                WitnessKind::Proptest,
                "ATK-W5-001: real_proptest_fn is inside a proptest! block and must\n\
                 be classified as WitnessKind::Proptest, not {:?}.\n\
                 Current bug: visit_item_fn never sees functions inside macro\n\
                 invocations — the proptest body is opaque to the visitor.\n\
                 Fix (W5): add visit_item_macro that recognizes proptest! bodies\n\
                 and registers inner function names as Proptest kind.",
                witness_kind
            );
        }
        WitnessStatus::NotFound { .. } => {
            panic!(
                "ATK-W5-001: real_proptest_fn was not found at all — the structural\n\
                 detection has not yet been implemented. W5 must walk into proptest!\n\
                 macro bodies to find these function names."
            );
        }
        other => {
            panic!("ATK-W5-001: unexpected status {:?}", other);
        }
    }
}

// ============================================================================
// ATK-W5-002: Multiple functions inside a proptest! block — all must be found
//
// The fixture has two functions inside one proptest! invocation.
// W5 must register BOTH, not just the first one.
// ============================================================================

#[test]
fn atk_w5_002_multiple_proptest_functions_all_detected() {
    let fixture_root = fixture("atk_w5_proptest_detection");

    // Run audit with a manually constructed immunity for the second function.
    // The simplest way to test this is via the function index directly —
    // but FunctionIndex is private. Instead, test by checking that scan +
    // audit with a witness pointing at second_proptest_fn resolves correctly.
    //
    // This test demonstrates the detection reaches ALL functions in the block,
    // not just the first one the parser encounters.
    //
    // For now: assert that second_proptest_fn appears in the workspace
    // function index by constructing a ScanReport with it as a witness.
    let immunity = Immunity {
        antigen_type: "PanickingInDrop".to_string(),
        witness: "second_proptest_fn".to_string(),
        file: PathBuf::from("lib.rs"),
        line: 1,
        item_kind: "impl".to_string(),
        item_target: antigen::scan::ItemTarget::Unknown { line: 0 },
        canonical_path: None,
        requires_predicate: None,
    };
    let mut report = ScanReport::default();
    report.immunities.push(immunity);

    let audit_report2 = audit(&report, &fixture_root);
    assert_eq!(audit_report2.audits.len(), 1);
    let a = &audit_report2.audits[0];

    match &a.witness_status {
        WitnessStatus::Resolved { witness_kind, .. } => {
            assert_eq!(
                *witness_kind,
                WitnessKind::Proptest,
                "ATK-W5-002: second_proptest_fn (second fn in the proptest! block)\n\
                 must be classified as Proptest kind, not {:?}.\n\
                 Fix: the proptest body walker must scan ALL fn declarations in\n\
                 the block, not stop after the first.",
                witness_kind
            );
        }
        WitnessStatus::NotFound { .. } => {
            panic!(
                "ATK-W5-002: second_proptest_fn not found — W5's proptest body\n\
                 walker must register all functions in the block, not just the first."
            );
        }
        other => panic!("ATK-W5-002: unexpected status {:?}", other),
    }
}

// ============================================================================
// ATK-W5-003: Function with "proptest!" only in doc comment is NOT Proptest
//
// The textual heuristic classifies ANY file with "proptest!" in it as having
// proptest witnesses. A plain #[test] function in a file that mentions
// proptest! in documentation must remain WitnessKind::Test after W5.
//
// This is the over-matching regression the TODO explicitly names.
// ============================================================================

#[test]
fn atk_w5_003_doc_comment_proptest_mention_does_not_over_classify() {
    let fixture_root = fixture("atk_w5_proptest_detection");

    // Audit the plain #[test] function that is NOT inside a proptest! block
    // but is in a file that contains proptest! invocations.
    let immunity = Immunity {
        antigen_type: "PanickingInDrop".to_string(),
        witness: "not_proptest_despite_comment".to_string(),
        file: PathBuf::from("lib.rs"),
        line: 1,
        item_kind: "impl".to_string(),
        item_target: antigen::scan::ItemTarget::Unknown { line: 0 },
        canonical_path: None,
        requires_predicate: None,
    };
    let mut report = ScanReport::default();
    report.immunities.push(immunity);

    let audit_report = audit(&report, &fixture_root);
    let a = &audit_report.audits[0];

    match &a.witness_status {
        WitnessStatus::Resolved { witness_kind, .. } => {
            // This function has #[test] but is NOT inside a proptest! block.
            // It must be classified as Test, not Proptest.
            assert_eq!(
                *witness_kind,
                WitnessKind::Test,
                "ATK-W5-003: not_proptest_despite_comment has #[test] and is not\n\
                 inside a proptest! block. It must be WitnessKind::Test, not {:?}.\n\
                 The textual heuristic over-classifies because the file contains\n\
                 proptest! invocations elsewhere. W5's structural detection must\n\
                 classify per-function based on whether that specific function\n\
                 is inside a proptest! body.",
                witness_kind
            );
        }
        other => panic!("ATK-W5-003: unexpected status {:?}", other),
    }
}

// ============================================================================
// ATK-W5-004: plain_test in a file with proptest! must remain WitnessKind::Test
//
// Companion to ATK-W5-003. Any plain #[test] function that is NOT inside
// a proptest! block must remain Test kind even if the file contains
// proptest! invocations.
// ============================================================================

#[test]
fn atk_w5_004_plain_test_in_proptest_file_remains_test_kind() {
    let fixture_root = fixture("atk_w5_proptest_detection");
    let immunity = Immunity {
        antigen_type: "PanickingInDrop".to_string(),
        witness: "plain_test".to_string(),
        file: PathBuf::from("lib.rs"),
        line: 1,
        item_kind: "impl".to_string(),
        item_target: antigen::scan::ItemTarget::Unknown { line: 0 },
        canonical_path: None,
        requires_predicate: None,
    };
    let mut report = ScanReport::default();
    report.immunities.push(immunity);

    let audit_report = audit(&report, &fixture_root);
    let a = &audit_report.audits[0];

    match &a.witness_status {
        WitnessStatus::Resolved { witness_kind, .. } => {
            assert_eq!(
                *witness_kind,
                WitnessKind::Test,
                "ATK-W5-004: plain_test is a plain #[test] not inside proptest!\n\
                 It must be WitnessKind::Test, not {:?}.\n\
                 The textual heuristic currently over-classifies this as Proptest\n\
                 because the file contains proptest! invocations.",
                witness_kind
            );
        }
        other => panic!("ATK-W5-004: unexpected status {:?}", other),
    }
}

// ============================================================================
// ATK-W5-007: Free function with same name as proptest function — W7 resolution
//
// Original W5 framing: precedence ordering. The visit_macro `or_insert_with`
// silently lost to an earlier `visit_item_fn` for the same name, downgrading
// a Proptest witness to Function classification. The pre-W7 fix proposed
// "Proptest always wins."
//
// W7 (ADR-005 Amendment 3) reframes: when two functions share a name, the
// witness is genuinely ambiguous. The user has written `witness =
// shadowed_by_free_fn` against a workspace where that name resolves to two
// distinct definitions. Silently picking Proptest is the same shape of bug
// as silently picking the alphabetically-last file — it lets the audit
// resolution depend on properties the user didn't author intentionally.
//
// Post-W7 correct behavior: report `WitnessStatus::Ambiguous` with both
// candidate locations, witness_tier `None`, audit_hint `AmbiguousResolution`.
// The fix on the user side is to rename one function or qualify the path.
// This is consistent with ATK-A2-005's resolution discipline.
// ============================================================================

#[test]
fn atk_w5_007_proptest_function_collision_with_free_fn_is_ambiguous() {
    use antigen::audit::{AuditHint, WitnessTier};

    let fixture_root = fixture("atk_w5_007_proptest_name_shadow");
    let scan = scan_workspace(&fixture_root, None).unwrap();
    let audit_report = audit(&scan, &fixture_root);

    assert_eq!(audit_report.audits.len(), 1, "fixture has one immunity");
    let a = &audit_report.audits[0];

    // Per W7: same-named free function and proptest function = two distinct
    // candidates. The audit must surface the ambiguity rather than picking one.
    match &a.witness_status {
        WitnessStatus::Ambiguous { candidates } => {
            assert_eq!(
                candidates.len(),
                2,
                "expected two candidates for shadowed_by_free_fn (free fn + proptest fn)",
            );
        }
        other => panic!(
            "ATK-W5-007 (post-W7): expected WitnessStatus::Ambiguous; got {:?}",
            other
        ),
    }
    assert_eq!(a.witness_tier, WitnessTier::None);
    assert_eq!(a.audit_hint, AuditHint::AmbiguousResolution);
    assert!(!a.is_well_formed());
}
