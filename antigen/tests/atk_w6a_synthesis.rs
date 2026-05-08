//! W6a synthesis-pass integration test.
//!
//! After explicit collection, `scan_workspace` runs a second pass that walks
//! every item against every parseable fingerprint and emits synthetic
//! `Presentation { match_kind: FingerprintMatch }` records. Per ADR-001
//! Amendment 1 Change 2 (the 5-state matrix) and ADR-010 Amendment 3
//! Performance Invariant 4 (node-kind dispatch).

use antigen::scan::{scan_workspace, MatchKind};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn w6a_synthesis_emits_fingerprint_match_for_unmarked_site() {
    // The atk_w5_007 fixture has both a free fn and a proptest function with
    // the same name. It also imports antigen::antigen and declares an
    // #[immune(...)] macro — but no #[antigen] declaration in this fixture
    // means there are no fingerprints to match against. We use the basic
    // example as a workspace root: it declares PanickingInDrop with a
    // body_contains_macro fingerprint.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("antigen")
        .join("examples");
    let report = scan_workspace(&root, None).unwrap();

    // basic.rs declares one antigen — PanickingInDropAntigen (the type the
    // macro is applied to is named PanickingInDrop).
    assert!(
        !report.antigens.is_empty(),
        "expected at least one antigen declaration in examples/, got 0",
    );

    // The fingerprint matches `impl` blocks containing panic!/etc. macros;
    // basic.rs has VulnerableType::drop with .unwrap_or() (no panic macro)
    // and SafeType::drop (no panic macro), so neither fires the
    // body_contains_macro check. broken_witness.rs has a `name = matches("*")`
    // fingerprint that fires for everything in the file.
    let fingerprint_matches: Vec<_> = report
        .presentations
        .iter()
        .filter(|p| p.match_kind == MatchKind::FingerprintMatch)
        .collect();
    let explicit_count = report
        .presentations
        .iter()
        .filter(|p| p.match_kind == MatchKind::ExplicitMarker)
        .count();

    eprintln!(
        "W6a synthesis: {} explicit, {} fingerprint matches",
        explicit_count,
        fingerprint_matches.len()
    );
    for p in &fingerprint_matches {
        eprintln!(
            "  fp-match: {}:{}  {} on {}",
            p.file.display(),
            p.line,
            p.antigen_type,
            p.item_kind
        );
    }
    // Substrate-grounded check: at least one fingerprint match across the
    // examples (broken_witness's `name = matches("*")` matches every named
    // top-level item in that file). The exact count depends on what other
    // examples land in the directory; we assert >= 1 so future additions
    // don't break this test.
    assert!(
        !fingerprint_matches.is_empty(),
        "expected at least one synthetic FingerprintMatch presentation; got 0",
    );
}

#[test]
fn w6a_synthesis_dedupes_against_explicit_markers() {
    // When an item has an explicit #[presents(X)] AND would also match X's
    // fingerprint, the synthesis pass must NOT emit a duplicate
    // FingerprintMatch — the explicit marker dominates.
    let _ = fixture; // silence unused warning if this test grows
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("antigen")
        .join("examples");
    let report = scan_workspace(&root, None).unwrap();

    let mut by_target: HashMap<_, Vec<&_>> = HashMap::new();
    for p in &report.presentations {
        by_target
            .entry((
                p.file.clone(),
                p.antigen_type.clone(),
                p.item_target.clone(),
            ))
            .or_default()
            .push(p);
    }
    for (key, ps) in &by_target {
        let has_explicit = ps.iter().any(|p| p.match_kind == MatchKind::ExplicitMarker);
        let has_synthetic = ps
            .iter()
            .any(|p| p.match_kind == MatchKind::FingerprintMatch);
        assert!(
            !(has_explicit && has_synthetic),
            "W6a dedup: same (file, antigen, item_target) {key:?} has BOTH explicit \
             and synthetic presentations — synthesis pass must skip when explicit exists",
        );
    }
}

// ============================================================================
// ATK-W6a-SYN-001: tolerance site gets a spurious FingerprintMatch
//
// `synthesis_pass` deduplication only checks for `MatchKind::ExplicitMarker`.
// A site with `#[antigen_tolerance(X)]` but no `#[presents(X)]` is NOT covered
// by the dedup check — the synthesis pass emits a `FingerprintMatch` for it.
// `unaddressed_presentations` then suppresses it from the "unaddressed" list
// (tolerance check fires), but `report.presentations` contains a spurious
// entry. This inflates `total_declarations()`, produces wrong CLI output for
// the "fingerprint match" state, and violates the invariant that tolerated
// sites are NOT in "unaddressed" OR "fingerprint match" — they are in
// "tolerated" (the 5-state matrix per ADR-001 Amendment 1 Change 2).
//
// STATUS: FAILING — synthesis_pass does not check tolerances in dedup.
// ============================================================================

#[test]
fn atk_w6a_syn_001_tolerated_site_does_not_get_spurious_fingerprint_match() {
    // Create a temp workspace with one .rs file containing:
    //   - an #[antigen] struct with a `name = matches("Vulnerable")` fingerprint
    //   - a struct named `Vulnerable` with #[antigen_tolerance(TestAntigen, ...)]
    //     but no #[presents(TestAntigen)]
    //
    // After scan, `presentations` must NOT contain a FingerprintMatch for
    // `Vulnerable` — the tolerance acknowledges the match without triggering
    // the synthesis presentation path.
    let dir = tempfile::TempDir::new().expect("tempdir");
    let src_path = dir.path().join("lib.rs");
    let mut f = std::fs::File::create(&src_path).expect("create lib.rs");
    write!(
        f,
        r#"
use antigen::antigen;
use antigen::antigen_tolerance;

#[antigen(
    name = "test-antigen",
    fingerprint = "item = struct, name = matches(\"Vulnerable\")",
    summary = "test antigen for syn-001",
)]
struct TestAntigen;

#[antigen_tolerance(
    TestAntigen,
    rationale = "deliberate: this struct is the controlled test specimen",
)]
struct Vulnerable;
"#
    )
    .expect("write lib.rs");
    drop(f);

    let report = scan_workspace(dir.path(), None).unwrap();

    // Antigen must be discovered.
    assert_eq!(report.antigens.len(), 1, "expected one antigen declaration");

    // The tolerance must be discovered.
    assert_eq!(report.tolerances.len(), 1, "expected one tolerance");

    // There must be NO presentations at all — neither ExplicitMarker
    // (no #[presents]) nor FingerprintMatch (synthesis must skip tolerated sites).
    let fp_matches: Vec<_> = report
        .presentations
        .iter()
        .filter(|p| p.match_kind == MatchKind::FingerprintMatch && p.antigen_type == "TestAntigen")
        .collect();
    assert!(
        fp_matches.is_empty(),
        "ATK-W6a-SYN-001: synthesis_pass emitted {} spurious FingerprintMatch \
         presentation(s) for a tolerated site — synthesis dedup must also check \
         tolerances, not only ExplicitMarker presentations.\n\
         Spurious entries: {fp_matches:#?}",
        fp_matches.len(),
    );
}

// ============================================================================
// ATK-W6a-SYN-004: immunity on impl-method does not address synthesis match
//                   on the enclosing impl block
//
// `synthesis_pass` only walks top-level `parsed.items`. For `syn::Item::Impl`,
// it assigns `ItemTarget::Impl { trait_path, target_type }`. The visitor
// descends into impl blocks and assigns `ItemTarget::ImplFn` for methods.
// `ItemTarget::addresses()` returns false for heterogeneous variants — so an
// `#[immune(X)]` on a method inside `impl Foo` does NOT address a synthetic
// `FingerprintMatch` on the `impl Foo` block itself.
//
// This is a known scope limitation: synthesis sees the impl block as the
// matched unit; the user must put `#[immune]` on the impl block (not a
// method inside it) to suppress the synthesis match. The test documents the
// behavior so it's an explicit invariant rather than a silent surprise.
//
// STATUS: PASSING — documents existing behavior as an explicit contract.
// ============================================================================

#[test]
fn atk_w6a_syn_004_immunity_on_impl_method_does_not_address_synthesis_match_on_impl_block() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let src_path = dir.path().join("lib.rs");
    let mut f = std::fs::File::create(&src_path).expect("create lib.rs");
    // Fingerprint matches `impl` blocks (no further constraint).
    // #[immune] is on the `drop` method INSIDE the impl, not on the impl block.
    // Per the scope limit above, this must NOT suppress the unaddressed presentation.
    write!(
        f,
        r#"
use antigen::antigen;
use antigen::immune;

#[antigen(
    name = "impl-antigen",
    fingerprint = "item = impl",
    summary = "matches all impl blocks",
)]
struct ImplAntigen;

struct MyType;

impl MyType {{
    #[immune(ImplAntigen, witness = my_test)]
    fn drop(&mut self) {{}}
}}

#[test]
fn my_test() {{}}
"#
    )
    .expect("write lib.rs");
    drop(f);

    let report = scan_workspace(dir.path(), None).unwrap();

    // The synthesis pass must emit a FingerprintMatch for `impl MyType`.
    let fp_match_count = report
        .presentations
        .iter()
        .filter(|p| p.match_kind == MatchKind::FingerprintMatch && p.antigen_type == "ImplAntigen")
        .count();
    assert_eq!(
        fp_match_count, 1,
        "expected one FingerprintMatch for `impl MyType`; got {}",
        fp_match_count
    );

    // The #[immune] is on the method (ImplFn target), not the impl block (Impl target).
    // addresses() returns false for heterogeneous variants — so the match is unaddressed.
    let unaddressed = report.unaddressed_presentations();
    let unaddressed_impl_antigen: Vec<_> = unaddressed
        .iter()
        .filter(|u| u.presentation.antigen_type == "ImplAntigen")
        .collect();
    assert_eq!(
        unaddressed_impl_antigen.len(),
        1,
        "ATK-W6a-SYN-004: #[immune] on an impl METHOD must NOT address a \
         synthesis match on the enclosing IMPL BLOCK — immunity must be \
         placed on the impl block itself. Got {} unaddressed (expected 1).\n\
         This is a known scope invariant, not a bug. Users must write \
         #[immune] on `impl MyType {{` not on a method inside it.",
        unaddressed_impl_antigen.len()
    );
}

// ============================================================================
// ATK-W6a-SYN-002: `item = mod` fingerprint — dead code in dispatch map
//
// `synthesis_pass`'s node-kind dispatch maps `syn::Item::Mod` →
// `Some(ItemKind::Mod)`, but `item_kind_and_target` returns `None` for mod
// items — so the synthesis loop `continue`s before reaching the dispatch map
// for every mod item. A fingerprint with `item = mod` never fires against any
// item.
//
// This is a known implementation gap (mod items are not yet modeled in
// `ItemTarget`). The test is `#[ignore]` as a pre-impl contract: once mod
// items are added to `item_kind_and_target`, this test must be un-ignored
// and the dead code arm removed.
//
// STATUS: IGNORED — pre-impl contract for when `item = mod` support lands.
// ============================================================================

#[test]
#[ignore = "pre-impl: item = mod not yet supported in synthesis_pass (mod arm unreachable in dispatch)"]
fn atk_w6a_syn_002_item_mod_fingerprint_fires_against_mod_items() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let src_path = dir.path().join("lib.rs");
    let mut f = std::fs::File::create(&src_path).expect("create lib.rs");
    write!(
        f,
        r#"
use antigen::antigen;

#[antigen(
    name = "mod-antigen",
    fingerprint = "item = mod, name = matches(\"inner\")",
    summary = "matches mod items named inner",
)]
struct ModAntigen;

mod inner {{
    pub fn something() {{}}
}}
"#
    )
    .expect("write lib.rs");
    drop(f);

    let report = scan_workspace(dir.path(), None).unwrap();
    assert_eq!(report.antigens.len(), 1, "expected one antigen declaration");

    // When mod support lands, synthesis must emit a FingerprintMatch for `mod inner`.
    let fp_matches: Vec<_> = report
        .presentations
        .iter()
        .filter(|p| p.match_kind == MatchKind::FingerprintMatch)
        .collect();
    assert_eq!(
        fp_matches.len(),
        1,
        "ATK-W6a-SYN-002: expected one FingerprintMatch for `mod inner` \
         but got {}.\n\
         Root cause: synthesis_pass has `syn::Item::Mod` in the dispatch map \
         but `item_kind_and_target` returns None for mod items — the loop \
         hits `continue` before dispatch. Both must be updated together.",
        fp_matches.len(),
    );
}

// ============================================================================
// ATK-W6a-SYN-003: malformed fingerprint in antigen decl → parse_failures,
//                   other fingerprints still evaluated
//
// A malformed fingerprint string in `#[antigen(fingerprint = "MALFORMED")]`
// must:
//   (a) appear in `report.parse_failures` with a message naming the antigen
//   (b) NOT prevent other antigens with valid fingerprints from matching
//
// STATUS: PASSING — verifies existing non-fatal error handling.
// ============================================================================

#[test]
fn atk_w6a_syn_003_malformed_fingerprint_is_nonfatal_and_other_antigens_still_match() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let src_path = dir.path().join("lib.rs");
    let mut f = std::fs::File::create(&src_path).expect("create lib.rs");
    write!(
        f,
        r#"
use antigen::antigen;

#[antigen(
    name = "bad-antigen",
    fingerprint = "this is not valid DSL !!!",
    summary = "malformed fingerprint",
)]
struct BadAntigen;

#[antigen(
    name = "good-antigen",
    fingerprint = "item = struct, name = matches(\"TargetStruct\")",
    summary = "valid fingerprint",
)]
struct GoodAntigen;

struct TargetStruct;
"#
    )
    .expect("write lib.rs");
    drop(f);

    let report = scan_workspace(dir.path(), None).unwrap();

    // Both antigens must be discovered.
    assert_eq!(
        report.antigens.len(),
        2,
        "expected two antigen declarations; got {}",
        report.antigens.len()
    );

    // The malformed fingerprint must surface in parse_failures.
    assert!(
        !report.parse_failures.is_empty(),
        "ATK-W6a-SYN-003: malformed fingerprint did not produce a parse_failure entry"
    );
    let bad_antigen_failure = report.parse_failures.iter().any(|pf| {
        pf.error.contains("BadAntigen")
            || pf.error.contains("bad-antigen")
            || pf.error.contains("fingerprint")
    });
    assert!(
        bad_antigen_failure,
        "ATK-W6a-SYN-003: parse_failures entry does not name the antigen or fingerprint.\n\
         Got: {:?}",
        report.parse_failures
    );

    // GoodAntigen's fingerprint must still fire — TargetStruct matches.
    let fp_matches: Vec<_> = report
        .presentations
        .iter()
        .filter(|p| p.match_kind == MatchKind::FingerprintMatch && p.antigen_type == "GoodAntigen")
        .collect();
    assert_eq!(
        fp_matches.len(),
        1,
        "ATK-W6a-SYN-003: malformed BadAntigen fingerprint must not prevent \
         GoodAntigen from matching TargetStruct. Got {} fp-matches.",
        fp_matches.len()
    );
}
