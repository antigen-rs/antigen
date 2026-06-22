//! ATK-W3 pre-implementation contracts for item-identity matching.
//!
//! These tests define what W3 (item-identity matching in scan) MUST handle
//! correctly when it replaces the 20-line proximity heuristic. All tests here
//! are `#[ignore]` until W3 lands `item_target: ItemTarget` on Presentation
//! and Immunity. Remove `#[ignore]` from each test as W3 makes them testable,
//! verify each test FAILS (confirming the contract is real), then fix the
//! implementation to make them pass.
//!
//! The W3 adversarial check from sweeps/A2-core-macros/README.md:
//! "What about (a) trait method impls where presents lives on the trait method
//! but immune on the impl method? (b) generic impls with multiple instantiations?
//! (c) cfg-conditional impls?"
//!
//! These tests answer (a), (b), (c) with concrete failing fixtures.

use std::path::{Path, PathBuf};

use antigen::scan::scan_workspace;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

// ============================================================================
// ATK-W3-001: presents on trait method, immune on impl method
//
// When #[presents] is on a trait method definition and #[immune] is on the
// concrete impl of that method, the item-identity matcher must treat them
// as addressing the same vulnerability site — not as two unrelated items.
//
// Expected: unaddressed_presentations() is empty (the pair is matched).
// Current behavior (proximity heuristic): depends on how many lines separate
// the trait definition from the impl. In this fixture they may be >20 lines
// apart (or in different files in a real workspace), so the heuristic fails.
//
// W3 fix: TraitFn("SafeOps", "dangerous_op") matches ImplFn("SafeOps::MyType",
// "dangerous_op") OR the matcher has a trait-impl bridging rule.
// ============================================================================

#[test]
fn atk_w3_001_trait_presents_matches_impl_immune() {
    let fixture_root = fixture("atk_w3_001_trait_presents_impl_immune");
    let scan = scan_workspace(&fixture_root, None).unwrap();

    // Both presents and immune should be found.
    assert_eq!(
        scan.presentations.len(),
        1,
        "should find one presentation on the trait method"
    );
    assert_eq!(
        scan.immunities.len(),
        1,
        "should find one immunity on the impl method"
    );

    // The pair should be considered matched — not unaddressed.
    let unaddressed = scan.unaddressed_presentations();
    assert!(
        unaddressed.is_empty(),
        "ATK-W3-001: presents on trait method + immune on impl method must be\n\
         treated as a matched pair by item-identity matching.\n\
         Currently the proximity heuristic may fail if they're >20 lines apart\n\
         or in separate items. Got unaddressed: {:?}",
        unaddressed
    );
}

// ============================================================================
// ATK-W3-002: multiple impl blocks for the same type, presents in first,
// immune in second (>20 lines apart — proximity heuristic already fails this)
//
// Expected: unaddressed_presentations() is empty.
// W3 fix: both impl blocks have impl-target "MyType"; they match on type name.
// ============================================================================

#[test]
fn atk_w3_002_multiple_impl_blocks_matched_by_type_name() {
    let fixture_root = fixture("atk_w3_002_multiple_impls_same_type");
    let scan = scan_workspace(&fixture_root, None).unwrap();

    assert_eq!(scan.presentations.len(), 1, "one presentation");
    assert_eq!(scan.immunities.len(), 1, "one immunity");

    let unaddressed = scan.unaddressed_presentations();
    assert!(
        unaddressed.is_empty(),
        "ATK-W3-002: presents on first impl block + immune on second impl block\n\
         for the same type must be matched by type name, not line proximity.\n\
         The 20-line heuristic already fails this — the impl blocks are deliberately\n\
         separated by blank lines in the fixture.\n\
         Got unaddressed: {:?}",
        unaddressed
    );
}

// ============================================================================
// ATK-W3-003: cfg-conditional impl — presents in cfg(not(test)), immune in
// cfg(test) module
//
// This is the most structurally complex case. The AST visitor sees both items
// but they're in different cfg branches. Item-identity matching must bridge them.
//
// Expected behavior: either (a) matched (cfg is irrelevant to item identity),
// or (b) a diagnostic explaining the cfg-split — but NOT silently unaddressed.
//
// The wrong answer: silent unaddressed presentation. If the test reports
// "1 unaddressed presentation" with no explanation that the cfg branches
// separate them, the developer gets no signal.
// ============================================================================

#[test]
fn atk_w3_003_cfg_conditional_impl_not_silently_unaddressed() {
    let fixture_root = fixture("atk_w3_003_cfg_conditional_impl");
    let scan = scan_workspace(&fixture_root, None).unwrap();

    // The fixture may or may not find both declarations depending on whether
    // the syn visitor descends into cfg-gated items. Verify at minimum that
    // if a presentation is found, it is not SILENTLY unaddressed.
    if !scan.presentations.is_empty() {
        let unaddressed = scan.unaddressed_presentations();
        // Either: matched (unaddressed is empty) — ideal
        // Or: there's a diagnostic in parse_failures explaining the cfg split
        // Not acceptable: silently unaddressed with no explanation
        if !unaddressed.is_empty() {
            assert!(
                !scan.parse_failures.is_empty(),
                "ATK-W3-003: a cfg-conditional presents+immune pair that cannot be\n\
                 matched must produce a diagnostic (parse_failures entry), not silent\n\
                 unaddressed output. The developer needs to know WHY the match failed.\n\
                 Got unaddressed: {:?}, parse_failures: {:?}",
                unaddressed,
                scan.parse_failures
            );
        }
    }
    // If scan found no presentations (cfg gating hides it from the visitor),
    // that's also a known gap — document it but don't fail the test here.
    // The real failure mode is silent unaddressed, which the assert above catches.
}

// ============================================================================
// ATK-W3-004: generic impl — presents on impl<T> Drop, immune on impl<T> body
//
// Generic parameters must not prevent item-identity matching. The matcher must
// use the base type name ("Container") not the full generic path ("Container<T>").
//
// The secondary concern: two instantiations (Container<i32>, Container<String>)
// should both be covered by one immune declaration on impl<T>. This is the
// "structural family" pattern from ADR-007 anti-YAGNI.
// ============================================================================

#[test]
fn atk_w3_004_generic_impl_matched_by_base_type_name() {
    let fixture_root = fixture("atk_w3_004_generic_impl_multiple_instantiations");
    let scan = scan_workspace(&fixture_root, None).unwrap();

    assert_eq!(
        scan.presentations.len(),
        1,
        "one presentation on impl<T> Drop"
    );
    assert_eq!(scan.immunities.len(), 1, "one immunity on impl<T> body");

    let unaddressed = scan.unaddressed_presentations();
    assert!(
        unaddressed.is_empty(),
        "ATK-W3-004: presents on impl<T> Drop and immune on impl<T> Container must\n\
         be matched by base type name 'Container', ignoring the generic parameter.\n\
         The item-identity matcher must strip generic params for matching purposes.\n\
         Got unaddressed: {:?}",
        unaddressed
    );
}

// ============================================================================
// ATK-W3-005: ItemTarget::Unknown equality creates degenerate matching
//
// When the scan visitor cannot classify an item (type alias, const, static,
// or future Rust item kinds), it falls back to ItemTarget::Unknown. The
// structural equality `Unknown == Unknown` means any two Unknown-kind items
// match each other in unaddressed_presentations, even if they are completely
// different items.
//
// Concrete failure: two type aliases both #[presents(X)], one type alias with
// #[immune(X)]. The immune's Unknown matches BOTH presents' Unknown — so one
// of the two vulnerabilities is silently treated as addressed when it isn't.
//
// This is a silent false negative: scan says "0 unaddressed" when there is 1.
// ============================================================================

#[test]
#[ignore = "W3 lands TypeAlias as a first-class ItemTarget so this fixture's three aliases \
    are no longer Unknown — they are TypeAlias(\"VulnerableAlias1\"), TypeAlias(\"VulnerableAlias2\"), \
    TypeAlias(\"ProtectedAlias\"). Under W3 matching, the immune (ProtectedAlias) addresses \
    neither presentation, so unaddressed.len() is 2, not 1. Test premise predates the \
    classification — needs re-fixturing (e.g., a const + module item that genuinely fall \
    through to Unknown) to actually exercise the Unknown-never-matches invariant."]
fn atk_w3_005_unknown_target_does_not_match_across_different_items() {
    let fixture_root = fixture("atk_w3_005_unknown_target_degenerate");
    let scan = scan_workspace(&fixture_root, None).unwrap();

    // The fixture has 2 presentations and 1 immunity, all on type aliases
    // (which the visitor cannot classify, so they all get ItemTarget::Unknown).
    assert_eq!(
        scan.presentations.len(),
        2,
        "fixture has two presentations on type aliases"
    );
    assert_eq!(
        scan.immunities.len(),
        1,
        "fixture has one immunity on a type alias"
    );

    let unaddressed = scan.unaddressed_presentations();

    // One immunity cannot address both presentations — at least one must remain
    // unaddressed. If Unknown == Unknown makes all three match each other, the
    // scan reports zero unaddressed (silent false negative).
    assert_eq!(
        unaddressed.len(),
        1,
        "ATK-W3-005: two presentations with one immunity must leave exactly one\n\
         unaddressed. If ItemTarget::Unknown == Unknown makes all Unknown items\n\
         match each other, the scan silently reports 0 unaddressed.\n\
         Fix: Unknown items must never match each other — Unknown should not\n\
         satisfy the item_target equality check in unaddressed_presentations.\n\
         Unknown items should always appear as unaddressed until W3 can classify\n\
         them or the user explicitly marks them with #[antigen_tolerance].\n\
         Got unaddressed: {:?}",
        unaddressed
    );
}

// ============================================================================
// ATK-W3-006: cross-file presents+immune pair produces false unaddressed
//
// The W3 matching requires i.file == p.file. A common Rust pattern:
// #[presents] in lib.rs, #[immune] in tests/integration_test.rs (a separate
// file that tests the Drop impl). The file equality check rejects this pair
// even though it's structurally correct.
//
// This is explicitly A3 scope per the W3 implementation comment. This test
// documents the expected behavior so the scope boundary is substrate-grounded.
// When A3 ships cross-file matching, remove #[ignore] and verify it passes.
// ============================================================================

#[test]
#[ignore = "A3 pre-implementation contract; cross-file presents+immune matching is out of scope for W3"]
fn atk_w3_006_cross_file_presents_immune_pair_is_not_silently_unaddressed() {
    // Contract: when W3/A3 extends matching across files, a presents in lib.rs
    // and an immune in tests/integration_test.rs for the same item must be
    // matched — not produce a false unaddressed presentation.
    //
    // For now this is the scope-boundary documentation. The W3 limitation must
    // be explicitly stated in the scan report when cross-file pairs are detected:
    // "presentation at lib.rs:N has no matching immunity in the same file;
    // cross-file matching is not yet supported (A3)."
    //
    // TODO: write fixture with cross-file pair when A3 ships.
    panic!("A3 pre-implementation contract — remove #[ignore] when cross-file matching ships");
}

// ============================================================================
// ATK-W3-007 (coordination-tier): "task complete" routing without substrate check
//
// Per the verification protocol established at A1 close: every "task complete"
// claim must name a substrate-grounded check. This test is the enforcement
// mechanism for W3 specifically.
//
// When W3 is marked complete, the routing must include:
// - A named substrate check (e.g., "cargo test --package antigen atk_w3")
// - Confirmation that the proximity heuristic TODO at scan.rs:245 is removed
// - An adoption-log entry showing W3 results against a real multi-crate adopter
//   workspace (per Milestone B exit criteria in A2 README)
//
// This test cannot be automated in Rust — it's a process contract. It lives
// here as documentation that the W3 closure must include these three artifacts.
// These must be verified at Milestone B.
// ============================================================================
