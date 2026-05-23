//! Supply-chain defense family — scan-level correctness tests.
//!
//! These tests verify that `scan_workspace` correctly extracts supply-chain
//! antigen presentations and immunities from source code. They operate at
//! the SCAN level (not audit level) and are compilable with the current
//! code — no new types required.
//!
//! Audit-level tests (`ContentHashMismatch` fires on content difference,
//! dep-attest-without-reviewable-artifact hint, etc.) live in
//! `campsites/.../scientist/tests/` and will be enabled as pathmaker
//! implements the audit-side types (ADR-025 §Enforcement-Surface).
//!
//! ADR-025: Supply-Chain Defense Family.

use antigen::scan::scan_workspace;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

// ============================================================================
// ContentHashMismatch — the NON-NEGOTIABLE antigen (ADR-025 B1-R)
// ============================================================================

#[test]
fn scan_finds_content_hash_mismatch_presentation() {
    // Verify scan_workspace extracts a ContentHashMismatch presentation.
    // This is the chalk/debug/eslint-config attack vector: content replacement
    // at a fixed version. Cargo.lock pins VERSION not CONTENT-HASH.
    let fx = fixture("sc_content_hash_presents_no_immune");
    let scan = scan_workspace(&fx, None).expect("scan completes");

    let found = scan
        .presentations
        .iter()
        .any(|p| p.antigen_type == "ContentHashMismatch");
    assert!(
        found,
        "scan must find ContentHashMismatch presentation; got presentations: {:?}",
        scan.presentations
            .iter()
            .map(|p| &p.antigen_type)
            .collect::<Vec<_>>()
    );
}

#[test]
fn scan_finds_content_hash_mismatch_as_unaddressed() {
    // The presentation exists but has no immunity → must be unaddressed.
    // This is the failing-as-passing form: the test PASSES when the antigen
    // correctly identifies the undefended site.
    let fx = fixture("sc_content_hash_presents_no_immune");
    let scan = scan_workspace(&fx, None).expect("scan completes");

    let unaddressed = scan.unaddressed_presentations();
    let found = unaddressed
        .iter()
        .any(|u| u.presentation.antigen_type == "ContentHashMismatch");
    assert!(
        found,
        "ContentHashMismatch presentation without immunity MUST appear in \
         unaddressed_presentations(); if this fails, the scan is incorrectly \
         matching a non-existent immunity"
    );
}

// content_hash_matches leaf type is NOW recognized in antigen-attestation
// (pathmaker added it to the sealed leaf set). The scanner correctly finds the
// immunity with the requires_predicate captured.

#[test]
fn scan_finds_content_hash_mismatch_immunity_with_requires_predicate() {
    // A site with #[immune(ContentHashMismatch, requires = content_hash_matches(...))]
    // must have its requires_predicate captured by the scanner.
    // This ensures the scanner doesn't silently drop the substrate-witness
    // when it sees `content_hash_matches` as the leaf type (regression guard
    // analogous to the rc.1 substrate-witness-pipeline bug).
    let fx = fixture("sc_content_hash_immune_with_witness");
    let scan = scan_workspace(&fx, None).expect("scan completes");

    // No parse failures — the leaf type is now recognized.
    assert!(
        scan.parse_failures.is_empty(),
        "content_hash_matches is now a recognized leaf type; no parse failures expected. \
         Got: {:?}",
        scan.parse_failures
    );

    let immunity = scan
        .immunities
        .iter()
        .find(|i| i.antigen_type == "ContentHashMismatch");
    assert!(
        immunity.is_some(),
        "scan must find ContentHashMismatch immunity; got immunities: {:?}",
        scan.immunities
            .iter()
            .map(|i| &i.antigen_type)
            .collect::<Vec<_>>()
    );

    let immunity = immunity.unwrap();
    assert!(
        immunity.requires_predicate.is_some(),
        "ContentHashMismatch immunity with requires = content_hash_matches(...) \
         MUST have requires_predicate captured by the scanner. \
         If this fails, the scanner dropped the substrate-witness predicate — \
         this is the rc.1 regression. See atk_a3_substrate_witness_pipeline."
    );

    // The predicate JSON must contain "content_hash_matches" to confirm
    // the scanner read the correct leaf type name.
    let predicate_str = immunity.requires_predicate.as_deref().unwrap_or("");
    assert!(
        predicate_str.contains("content_hash_matches"),
        "requires_predicate must contain 'content_hash_matches'; got: {predicate_str:?}"
    );
}

// ============================================================================
// UnpinnedDependency
// ============================================================================

#[test]
fn scan_finds_unpinned_dependency_presentation() {
    let fx = fixture("sc_unpinned_dep_presents_only");
    let scan = scan_workspace(&fx, None).expect("scan completes");

    let found = scan
        .presentations
        .iter()
        .any(|p| p.antigen_type == "UnpinnedDependency");
    assert!(
        found,
        "scan must find UnpinnedDependency presentation; got: {:?}",
        scan.presentations
            .iter()
            .map(|p| &p.antigen_type)
            .collect::<Vec<_>>()
    );
}

#[test]
fn scan_finds_unpinned_dep_as_unaddressed() {
    // Failing-as-passing: UnpinnedDependency site without immunity is flagged.
    let fx = fixture("sc_unpinned_dep_presents_only");
    let scan = scan_workspace(&fx, None).expect("scan completes");

    let unaddressed = scan.unaddressed_presentations();
    let found = unaddressed
        .iter()
        .any(|u| u.presentation.antigen_type == "UnpinnedDependency");
    assert!(
        found,
        "UnpinnedDependency without immunity MUST be unaddressed. \
         This test passes when the antigen correctly identifies the gap."
    );
}

// ============================================================================
// UnattestedDependencyInclusion / dep_attested rubber-stamp
// ============================================================================

#[test]
fn scan_finds_unattested_dep_inclusion_presentation() {
    // The dep_attested rubber-stamp fixture presents UnattestedDependencyInclusion.
    let fx = fixture("sc_dep_attested_rubber_stamp");
    let scan = scan_workspace(&fx, None).expect("scan completes");

    let found = scan
        .presentations
        .iter()
        .any(|p| p.antigen_type == "UnattestedDependencyInclusion");
    assert!(
        found,
        "scan must find UnattestedDependencyInclusion presentation; got: {:?}",
        scan.presentations
            .iter()
            .map(|p| &p.antigen_type)
            .collect::<Vec<_>>()
    );
}

// dep_attested leaf type is NOW recognized (pathmaker added to parser).

#[test]
fn scan_finds_dep_attested_rubber_stamp_immunity() {
    // The immunity exists (dep_attested without reviewable_artifact).
    // The scan should find the immunity; the AUDIT will flag the
    // dep-attest-without-reviewable-artifact hint because the dep_attested
    // has no reviewable_artifact argument.
    let fx = fixture("sc_dep_attested_rubber_stamp");
    let scan = scan_workspace(&fx, None).expect("scan completes");

    // No parse failures — dep_attested is now recognized.
    assert!(
        scan.parse_failures.is_empty(),
        "dep_attested is now a recognized leaf type; no parse failures expected. \
         Got: {:?}",
        scan.parse_failures
    );

    let found = scan
        .immunities
        .iter()
        .any(|i| i.antigen_type == "UnattestedDependencyInclusion");
    assert!(
        found,
        "scan must find UnattestedDependencyInclusion immunity with dep_attested witness"
    );
}

// ============================================================================
// All eleven supply-chain antigens coverage test
// ============================================================================

/// Canonical list of ADR-025 supply-chain antigen names (11 total, §Decision).
///
/// These names MUST match the stdlib antigen declarations exactly.
/// If an antigen is renamed or the count changes, this test fails — naming the
/// regression. Anti-YAGNI / structural-guarantee: all 11 are committed to.
const SUPPLY_CHAIN_ANTIGENS: &[&str] = &[
    "ContentHashMismatch",
    "UnsandboxedProcMacro",
    "UnpinnedDependency",
    "UnattestedDependencyInclusion",
    "DependencyUpgradeWithoutDiffReview",
    "SuddenDependencyExpansion",
    "UnsandboxedBuildScript",
    "UnpinnedTransitiveDependency",
    "MaintainerChangeWithoutReattestation",
    "AutoDependencyChainWithoutPinning",
    "PostInstallScriptInDependency",
];

#[test]
fn scan_finds_all_eleven_supply_chain_antigen_declarations() {
    // Anti-YAGNI / ADR-007: all 11 antigens are structurally committed to.
    // This test ensures that when the supply-chain stdlib is implemented,
    // all 11 declared antigens can be FOUND in a scan. The fixture declares
    // all 11 inline so this test works even before the stdlib is shipped.
    let fx = fixture("sc_all_eleven_supply_chain_presents");
    let scan = scan_workspace(&fx, None).expect("scan completes");

    let declared_names: std::collections::HashSet<&str> =
        scan.antigens.iter().map(|a| a.type_name.as_str()).collect();

    let mut missing = Vec::new();
    for &name in SUPPLY_CHAIN_ANTIGENS {
        if !declared_names.contains(name) {
            missing.push(name);
        }
    }
    assert!(
        missing.is_empty(),
        "scan must find all 11 supply-chain antigen declarations. Missing: {missing:?}. \
         ADR-025 §Decision commits to all 11 — ADR-007 anti-YAGNI."
    );
}

#[test]
fn scan_finds_all_eleven_supply_chain_presentations() {
    // All 11 supply-chain antigens have presentations in this fixture.
    let fx = fixture("sc_all_eleven_supply_chain_presents");
    let scan = scan_workspace(&fx, None).expect("scan completes");

    let presented_types: std::collections::HashSet<&str> = scan
        .presentations
        .iter()
        .map(|p| p.antigen_type.as_str())
        .collect();

    let mut missing = Vec::new();
    for &name in SUPPLY_CHAIN_ANTIGENS {
        if !presented_types.contains(name) {
            missing.push(name);
        }
    }
    assert!(
        missing.is_empty(),
        "scan must find presentations for all 11 supply-chain antigens. Missing: {missing:?}"
    );
}

#[test]
fn supply_chain_antigen_names_are_eleven_total() {
    // Locks the count: ADR-025 commits to exactly 11 stdlib antigens.
    // If this count changes (addition or removal), it must be an intentional
    // ADR amendment, not a silent drift.
    assert_eq!(
        SUPPLY_CHAIN_ANTIGENS.len(),
        11,
        "ADR-025 commits to exactly 11 supply-chain stdlib antigens (ADR-007 anti-YAGNI). \
         Changing this count requires an explicit ADR amendment."
    );
}

// ============================================================================
// Known scanner gap: visit_item_mod not implemented (→ Unknown target)
//
// `#[presents(X)]` on a `pub mod` block falls through to `ItemTarget::Unknown`
// because the scanner does not implement `visit_item_mod`.
//
// For supply-chain antigens, module-level annotation is natural: the whole
// module uses a dependency and all its call sites are in scope.
// The supply-chain implementation MUST add `visit_item_mod` + `ItemTarget::Mod`
// to the scanner. When that lands, update the fixtures above to use `mod`
// targets where appropriate and add a test asserting `ItemTarget::Mod` is found.
//
// Phase 2 test to add (after visit_item_mod lands):
//   - scan_mod_target_finds_presentations_on_mod_items
//   - supply_chain_annotate_whole_dep_module (natural use pattern)
// ============================================================================
