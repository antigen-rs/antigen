//! ATK — `ScanCoverage` ignorance-frontier adversarial inputs
//! (infra/multi-crate-scan, v03-vision-buildout).
//!
//! `ScanCoverage` tracks which workspace members were enumerated vs actually
//! scanned. The `unscanned_members()` + `is_complete()` logic is the ignorance
//! frontier substrate: members the scan never reached.
//!
//! ## Adversarial inputs
//!
//! **ATK-COV-1: duplicate entry in `enumerated_members`**
//!   - `["a@1", "a@1", "b@1"]` enumerated, `["a@1", "b@1"]` scanned.
//!   - `unscanned_members()` builds a `HashSet` from `scanned_members` and
//!     filters `enumerated_members` by absence. The duplicate `"a@1"` in
//!     `enumerated_members` passes the filter on its SECOND occurrence (because
//!     `"a@1"` IS in `scanned_members`, but the filter sees each occurrence
//!     independently). Wait — `!scanned.contains("a@1")` is `false` for BOTH
//!     occurrences, so neither passes. Duplicates actually CANCEL correctly.
//!   - CORRECT behavior: `["a@1"]` is correctly scanned; duplicate doesn't produce a phantom.
//!   - NOT a gap. This test documents the correct behavior.
//!
//! **ATK-COV-2: scanned member NOT in enumerated — phantom scanned member**
//!   - `["a@1"]` enumerated, `["a@1", "phantom@1"]` scanned.
//!   - `unscanned_members()` returns empty (all enumerated are in scanned).
//!   - `is_complete()` returns true.
//!   - The phantom `"phantom@1"` in `scanned_members` is silently ignored by the API.
//!   - CORRECT behavior: completeness is defined as "all enumerated members were scanned";
//!     extra entries in `scanned_members` are irrelevant.
//!
//! **ATK-COV-3: `scanned_members` ⊄ `enumerated_members` invariant violation**
//!   - If `scanned_members` contains a member that was NOT enumerated, this suggests
//!     a scan was performed outside the enumerated workspace. The API doesn't validate
//!     this invariant — `ScanCoverage` is data, not a builder with guards.
//!   - The question: should this be an invariant violation that the type system or
//!     API enforces? Currently: NO guard.
//!
//! **ATK-COV-4: identical sets — coverage is complete**
//!   - Both vectors contain the same elements (possibly in different order).
//!   - `is_complete()` must return true.
//!
//! **ATK-COV-5: completely empty `unscanned_members` — returns empty Vec, not panics**
//!   - Empty inputs don't panic.

use antigen::scan::ScanCoverage;

fn coverage(enumerated: &[&str], scanned: &[&str]) -> ScanCoverage {
    ScanCoverage {
        enumerated_members: enumerated.iter().map(ToString::to_string).collect(),
        scanned_members: scanned.iter().map(ToString::to_string).collect(),
    }
}

// ============================================================================
// ATK-COV-1: Duplicate in enumerated_members — no false alarm
// ============================================================================
//
// If `enumerated_members` contains a duplicate entry for a member that WAS scanned,
// `unscanned_members()` must NOT report it as missing. The HashSet-based filter
// correctly handles this: `!scanned.contains(dup)` is false for both occurrences.
#[test]
fn atk_cov1_duplicate_enumerated_member_that_was_scanned_does_not_appear_in_frontier() {
    // "a@1" appears twice in enumerated but IS in scanned.
    let c = coverage(&["a@1", "a@1", "b@1"], &["a@1", "b@1"]);

    let frontier = c.unscanned_members();

    assert!(
        frontier.is_empty(),
        "ATK-COV-1: duplicate 'a@1' in enumerated_members where 'a@1' IS scanned — \
         unscanned_members() must return empty (duplicate appears twice but both \
         occurrences are filtered out by HashSet membership check). Got: {:?}",
        frontier
    );

    assert!(
        c.is_complete(),
        "ATK-COV-1: all enumerated members (including the duplicate) are covered by \
         scanned_members — is_complete() must return true"
    );
}

// ============================================================================
// ATK-COV-2: Duplicate in enumerated_members that is NOT scanned — both appear
// ============================================================================
//
// If `enumerated_members` has a duplicate for a member that was NOT scanned,
// `unscanned_members()` must return the member exactly ONCE (deduplication at
// construction time, fixed in infra/scan-dedup-presentation-keys).
//
// Prior behavior (pre-fix): both occurrences of "a@1" passed the filter independently,
// producing a frontier of length 2. The fix deduplicates enumerated_members so each
// unscanned member appears exactly once regardless of input multiplicity.
#[test]
fn atk_cov2_duplicate_unscanned_member_appears_once_in_frontier_after_dedup() {
    // "a@1" appears twice in enumerated and is NOT scanned.
    let c = coverage(&["a@1", "a@1", "b@1"], &["b@1"]);

    let frontier = c.unscanned_members();

    // FIXED BEHAVIOR: deduplication means "a@1" appears exactly once.
    assert_eq!(
        frontier.len(),
        1,
        "ATK-COV-2: after dedup fix, duplicate unscanned 'a@1' must appear exactly ONCE \
         in the frontier. frontier: {:?}",
        frontier
    );
    assert_eq!(
        frontier[0], "a@1",
        "ATK-COV-2: the single frontier entry must be 'a@1'"
    );
}

// ============================================================================
// ATK-COV-3: Phantom scanned member (not in enumerated) — ignored silently
// ============================================================================
//
// A `scanned_members` entry that doesn't appear in `enumerated_members` is silently
// ignored by `unscanned_members()` and `is_complete()`. This is correct: completeness
// is defined relative to enumerated members only.
#[test]
fn atk_cov3_phantom_scanned_member_not_in_enumerated_is_silently_ignored() {
    // "phantom@1" is in scanned but NOT in enumerated.
    let c = coverage(&["a@1"], &["a@1", "phantom@1"]);

    // is_complete() should be true: all enumerated ("a@1") are in scanned.
    assert!(
        c.is_complete(),
        "ATK-COV-3: all enumerated members are scanned — is_complete() must return true \
         even when scanned_members contains extra phantom entries"
    );

    // unscanned_members() should be empty: "a@1" IS in scanned.
    assert!(
        c.unscanned_members().is_empty(),
        "ATK-COV-3: no enumerated members are missing from scanned — frontier must be empty"
    );
}

// ============================================================================
// ATK-COV-4: Order independence — sorted vs unsorted enumerations
// ============================================================================
//
// `enumerated_members` and `scanned_members` may be in different orders.
// `unscanned_members()` must produce the same result regardless of order.
#[test]
fn atk_cov4_order_independent_unscanned_detection() {
    // Enumerated in alphabetical order, scanned in reverse.
    let c = coverage(&["a@1", "b@1", "c@1"], &["c@1", "b@1", "a@1"]);

    assert!(
        c.is_complete(),
        "ATK-COV-4: all members scanned in reverse order — is_complete() must return true"
    );

    // Same set, different order, one member missing.
    let c2 = coverage(&["a@1", "b@1", "c@1"], &["c@1", "a@1"]); // "b@1" not scanned
    let frontier = c2.unscanned_members();
    assert_eq!(
        frontier,
        vec!["b@1"],
        "ATK-COV-4: 'b@1' was not scanned — must appear in frontier regardless of order. \
         Got: {:?}",
        frontier
    );
}

// ============================================================================
// ATK-COV-5: Empty inputs — no panic
// ============================================================================
#[test]
fn atk_cov5_empty_inputs_no_panic() {
    let c = coverage(&[], &[]);
    assert!(c.is_complete());
    assert!(c.unscanned_members().is_empty());

    let c2 = coverage(&["a@1"], &[]);
    assert!(!c2.is_complete());
    assert_eq!(c2.unscanned_members(), vec!["a@1"]);

    let c3 = coverage(&[], &["a@1"]);
    assert!(c3.is_complete()); // vacuously: no enumerated members to be unscanned
    assert!(c3.unscanned_members().is_empty());
}

// ============================================================================
// ATK-COV-6: Case-sensitivity of member names
// ============================================================================
//
// Canonical paths are case-sensitive ("A@1" != "a@1"). A member with differing
// case between enumerated and scanned would falsely appear as unscanned.
#[test]
fn atk_cov6_member_names_are_case_sensitive() {
    // "A@1" (uppercase) in enumerated, "a@1" (lowercase) in scanned.
    // These must NOT match — they're different canonical paths.
    let c = coverage(&["A@1"], &["a@1"]);

    // "A@1" is not in scanned (case-sensitive), so it appears as unscanned.
    // This is CORRECT — crate names that differ by case are different crates.
    let frontier = c.unscanned_members();
    assert_eq!(
        frontier,
        vec!["A@1"],
        "ATK-COV-6: case-sensitive member names — 'A@1' != 'a@1'; the uppercase \
         member must appear as unscanned. Got: {:?}",
        frontier
    );
}
