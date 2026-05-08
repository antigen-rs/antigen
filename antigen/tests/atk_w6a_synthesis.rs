//! W6a synthesis-pass integration test.
//!
//! After explicit collection, `scan_workspace` runs a second pass that walks
//! every item against every parseable fingerprint and emits synthetic
//! `Presentation { match_kind: FingerprintMatch }` records. Per ADR-001
//! Amendment 1 Change 2 (the 5-state matrix) and ADR-010 Amendment 3
//! Performance Invariant 4 (node-kind dispatch).

use antigen::scan::{scan_workspace, MatchKind};
use std::collections::HashMap;
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
            .entry((p.file.clone(), p.antigen_type.clone(), p.item_target.clone()))
            .or_default()
            .push(p);
    }
    for (key, ps) in &by_target {
        let has_explicit = ps.iter().any(|p| p.match_kind == MatchKind::ExplicitMarker);
        let has_synthetic = ps.iter().any(|p| p.match_kind == MatchKind::FingerprintMatch);
        assert!(
            !(has_explicit && has_synthetic),
            "W6a dedup: same (file, antigen, item_target) {key:?} has BOTH explicit \
             and synthetic presentations — synthesis pass must skip when explicit exists",
        );
    }
}
