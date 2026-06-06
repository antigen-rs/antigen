//! ATK — Stacked same-antigen same-item #[immune] entries masking substrate gap
//! (forward/immune-stacked-same-item-substrate-gap-mask, v03-vision-buildout).
//!
//! ## The find()-first-match masking bug
//!
//! In `compute_presentation_verdicts` (audit.rs, the deprecated `#[immune]` channel),
//! `immune_audit` is selected via:
//!
//!   `immunity_audits.iter().find(|a| antigen_type match + file match + item_target match)`
//!
//! `find()` returns the FIRST matching entry. If two `#[immune(X)]` declarations
//! sit on the same item (stacked same-antigen same-item, deprecated interface),
//! and the FIRST has no `requires=` (or a passing predicate) while the SECOND has
//! a FAILING `requires=`, then:
//!
//!   - `find()` returns the FIRST entry (no requires = → Defended)
//!   - The SECOND entry's failing predicate is never consulted
//!   - The presentation verdict reports Defended at some tier
//!   - The substrate gap from the second entry is INVISIBLE
//!
//! This is a silent-wrong-verdict: the code returns a confident "Defended" when
//! the correct answer is "`SubstrateGap`" (one of the stacked immunity declarations
//! has a broken substrate predicate).
//!
//! ## Scope
//!
//! This bug only affects the DEPRECATED `#[immune]` interface with stacked
//! same-antigen same-item declarations. The modern `#[presents(X, requires=)]`
//! path (ADR-029 R5) doesn't have this issue — there is only one presentation
//! site per item for a given antigen. The deprecated path is the attack surface.
//!
//! The fix options are:
//!   (a) Remove the deprecated `#[immune]` interface entirely (v0.3 option).
//!   (b) Change `find()` to `filter()` + `all()` check: if ANY stacked immunity
//!       has a substrate gap, surface the gap regardless of other entries.
//!
//! ## Tests
//!
//! ATK-IS-1: Single `#[immune]` with no requires → Defended (baseline).
//! ATK-IS-2: Single `#[immune]` with failing requires → `SubstrateGap` (baseline).
//! ATK-IS-3 (THE FAILING TEST): Two stacked `#[immune]` entries on the same item,
//!   first has no requires (→ Defended), second has failing requires. `find()`
//!   returns the first, second's gap is masked. DESIRED: `SubstrateGap`.
//!   CURRENT: Defended (wrong verdict, substrate gap masked).

use std::path::PathBuf;

use antigen::audit::{ImmuneVerdict, audit};
use antigen::scan::{Immunity, ItemTarget, MatchKind, Presentation, ScanReport};

fn presentation(antigen_type: &str) -> Presentation {
    Presentation {
        antigen_type: antigen_type.to_string(),
        file: PathBuf::from("src/lib.rs"),
        line: 10,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Fn("guarded_fn".to_string()),
        match_kind: MatchKind::ExplicitMarker,
        canonical_path: None,
        inherited_from: None,
        structural_fingerprint: String::new(),
        requires_predicate: None,
        proof: None,
    }
}

fn immunity_no_requires(antigen_type: &str) -> Immunity {
    // A deprecated #[immune(X)] with no requires= predicate.
    // This will have witness_status = Missing (no witness fn), tier = None.
    // But it has NO substrate predicate, so immune_audit_is_substrate_gap returns false.
    Immunity {
        antigen_type: antigen_type.to_string(),
        witness: "my_test_fn".to_string(),
        requires_predicate: None, // no substrate predicate
        file: PathBuf::from("src/lib.rs"),
        line: 15,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Fn("guarded_fn".to_string()),
        canonical_path: None,
        structural_fingerprint: String::new(),
    }
}

fn immunity_with_failing_requires(antigen_type: &str) -> Immunity {
    // A deprecated #[immune(X, requires = ...)] where the predicate will FAIL
    // because there's no sidecar on disk (sidecar_missing → tier=None, substrate gap).
    // The predicate JSON is a valid predicate; there's just no sidecar to satisfy it.
    let valid_predicate_json = r#"{"kind":"leaf","leaf":{"name":"signers","required":["alice"]}}"#;
    Immunity {
        antigen_type: antigen_type.to_string(),
        witness: String::new(),
        requires_predicate: Some(valid_predicate_json.to_string()),
        file: PathBuf::from("src/lib.rs"),
        line: 20, // different line, same item_target
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Fn("guarded_fn".to_string()), // SAME item
        canonical_path: None,
        structural_fingerprint: String::new(),
    }
}

// ============================================================================
// ATK-IS-1: Baseline — single immunity with no requires
// ============================================================================
//
// When there's only ONE immunity entry for an item and it has no requires=,
// the audit should produce Reachability or None tier (witness resolution),
// not SubstrateGap. This baseline confirms single-entry behavior is correct.
#[test]
fn atk_is1_single_immunity_no_requires_does_not_produce_substrate_gap() {
    let mut report = ScanReport::default();
    report.presentations.push(presentation("TestClass"));
    report.immunities.push(immunity_no_requires("TestClass"));

    let audit_report = audit(&report, std::path::Path::new("."));

    let verdict = audit_report
        .presentation_verdicts
        .iter()
        .find(|v| v.antigen_type == "TestClass")
        .expect("ATK-IS-1: must have a presentation verdict for TestClass");

    // A single no-requires immunity with an unresolved witness → Undefended,
    // NOT SubstrateGap (no substrate intent was declared).
    assert!(
        !matches!(verdict.verdict, ImmuneVerdict::SubstrateGap),
        "ATK-IS-1: a single no-requires immunity must NOT produce SubstrateGap \
         (no substrate intent was declared). Got: {:?}",
        verdict.verdict
    );
}

// ============================================================================
// ATK-IS-2: Baseline — single immunity with failing requires = SubstrateGap
// ============================================================================
//
// A single immunity with a `requires=` predicate that fails (no sidecar on
// disk) must produce SubstrateGap. This confirms the basic substrate-gap
// detection works.
#[test]
fn atk_is2_single_immunity_failing_requires_produces_substrate_gap() {
    let mut report = ScanReport::default();
    report.presentations.push(presentation("TestClass"));
    report
        .immunities
        .push(immunity_with_failing_requires("TestClass"));

    let audit_report = audit(&report, std::path::Path::new("."));

    let verdict = audit_report
        .presentation_verdicts
        .iter()
        .find(|v| v.antigen_type == "TestClass")
        .expect("ATK-IS-2: must have a presentation verdict for TestClass");

    assert!(
        matches!(verdict.verdict, ImmuneVerdict::SubstrateGap),
        "ATK-IS-2: a single immunity with failing requires= must produce SubstrateGap \
         (substrate intent declared but sidecar missing/predicate failing). Got: {:?}",
        verdict.verdict
    );
}

// ============================================================================
// ATK-IS-3 (THE FAILING TEST): Two stacked immunities — find() masks gap
// ============================================================================
//
// Two #[immune(TestClass)] entries on the same item:
//   Entry 1 (line 15): no requires= → immune_audit with no evaluated_predicate
//   Entry 2 (line 20): failing requires= → immune_audit with gap (sidecar_missing)
//
// `compute_presentation_verdicts` calls:
//   `immunity_audits.iter().find(|a| antigen_type == && file == && item_target ==)`
//
// Since both entries match (same antigen_type, file, item_target), `find()` returns
// Entry 1. Entry 2's substrate gap is NEVER CONSULTED.
//
// Result: verdict = Undefended (Entry 1 has no witness, no substrate → Undefended,
//         not SubstrateGap). The gap from Entry 2 is masked.
//
// DESIRED behavior: if ANY stacked immunity for the same item has a substrate gap,
// the verdict must surface the gap (SubstrateGap or similar).
//
// CURRENT behavior: Undefended (Entry 1 governs; Entry 2's gap is invisible).
// DESIRED behavior: SubstrateGap (Entry 2's failing predicate must surface).
//
// THE FIX: change `find()` to `filter()` + check all matches:
//   if any matched entry is a substrate gap, report SubstrateGap.
//   Otherwise proceed with the first matching entry for tier classification.
#[test]
fn atk_is3_stacked_immunity_second_has_failing_requires_gap_is_masked_by_find() {
    let mut report = ScanReport::default();
    report.presentations.push(presentation("TestClass"));
    // Entry 1: no requires=
    report.immunities.push(immunity_no_requires("TestClass"));
    // Entry 2: failing requires= (same item as Entry 1)
    report
        .immunities
        .push(immunity_with_failing_requires("TestClass"));

    let audit_report = audit(&report, std::path::Path::new("."));

    let verdict = audit_report
        .presentation_verdicts
        .iter()
        .find(|v| v.antigen_type == "TestClass")
        .expect("ATK-IS-3: must have a presentation verdict for TestClass");

    // FAILING ASSERTION: the verdict should be SubstrateGap because Entry 2
    // declares substrate intent (requires=) that fails. Entry 1's gap-free
    // status does NOT discharge Entry 2's broken substrate claim.
    //
    // CURRENT: Undefended (find() returns Entry 1; Entry 2 is never consulted)
    // DESIRED: SubstrateGap (any stacked entry with a failing requires= must surface)
    //
    // Note: the verdict may be Undefended (no code witness found for the first
    // entry, no substrate predicate on the first entry) or could be SubstrateGap
    // if the find()-first-match issue is fixed. The fix makes SubstrateGap happen.
    assert!(
        matches!(verdict.verdict, ImmuneVerdict::SubstrateGap),
        "ATK-IS-3 (FAILING): stacked same-antigen same-item immunities where the \
         SECOND entry has a failing requires= predicate: find() returns Entry 1 \
         (no requires=) and Entry 2's substrate gap is MASKED. \
         Current verdict: {:?} (Entry 2's gap is invisible). \
         Desired verdict: SubstrateGap (Entry 2's failing requires= must surface \
         regardless of Entry 1's state). \
         Fix: change find() to filter()+any() check: if ANY stacked immunity entry \
         for the same item is a substrate gap, report SubstrateGap. \
         This is the forward/immune-stacked-same-item-substrate-gap-mask bug.",
        verdict.verdict
    );
}
