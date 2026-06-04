//! Marked-Unknown markers — scan-read tests (ADR-041).
//!
//! The scanner reads `#[aura]` / `#[dread]` / `#[red_flag]` from the source-walk
//! and surfaces them on `ScanReport::marked_unknowns` (the scan-time half of
//! ADR-039's `Finding`). These tests assert the read, the fixed plane corners,
//! the required-trigger discipline mirrored at scan time, and the
//! existence-certainty-is-first-class property (ADR-041 §What-done-well-means (e)).

use antigen::scan::scan_workspace;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn scan_surfaces_all_three_markers_with_their_fixed_corners() {
    let scan = scan_workspace(&fixture("marked_unknown_markers"), None).expect("scan completes");
    let mus = &scan.marked_unknowns;
    assert_eq!(mus.len(), 3, "three markers; got: {mus:?}");

    let by = |marker: &str| {
        mus.iter()
            .find(|m| m.marker == marker)
            .unwrap_or_else(|| panic!("missing {marker}; got: {mus:?}"))
    };

    // #[aura] → low magnitude (aura), unsure.
    let aura = by("aura");
    assert_eq!(aura.magnitude, "aura");
    assert_eq!(aura.existence_certainty, "unsure");
    assert!(aura.trigger.contains("jitter"));

    // #[dread] → high magnitude (dread), unsure (scared-but-unsure).
    let dread = by("dread");
    assert_eq!(dread.magnitude, "dread");
    assert_eq!(dread.existence_certainty, "unsure");
    assert!(dread.trigger.contains("teardown"));

    // #[red_flag] → high existence-certainty (sure) — the auto-escalate corner.
    let red_flag = by("red-flag");
    assert_eq!(red_flag.existence_certainty, "sure");
    assert!(red_flag.trigger.contains("auth"));
}

#[test]
fn existence_certainty_is_first_class_distinct_from_magnitude() {
    // ADR-041 §What-done-well-means (e): existence_certainty must be a distinct
    // queryable field from magnitude — folding them would mis-rank a high-certainty
    // red_flag (sure) as a low-priority aura. The red_flag and the dread share the
    // SAME magnitude ("dread") but DIFFER on existence_certainty (sure vs unsure):
    // proof the field carries information magnitude alone does not.
    let scan = scan_workspace(&fixture("marked_unknown_markers"), None).expect("scan completes");
    let dread = scan
        .marked_unknowns
        .iter()
        .find(|m| m.marker == "dread")
        .expect("dread present");
    let red_flag = scan
        .marked_unknowns
        .iter()
        .find(|m| m.marker == "red-flag")
        .expect("red-flag present");
    assert_eq!(
        dread.magnitude, red_flag.magnitude,
        "dread and red_flag share magnitude (both high)"
    );
    assert_ne!(
        dread.existence_certainty, red_flag.existence_certainty,
        "but they DIFFER on existence_certainty (unsure vs sure) — the field is first-class, \
         not folded into magnitude; folding would mis-rank the high-certainty red_flag"
    );
}

#[test]
fn every_surfaced_marker_carries_a_nonempty_trigger() {
    // Guard 3 mirrored at scan: a surfaced marker always has a non-empty trigger
    // (a triggerless marker is a parse-failure, never a surfaced marked-unknown).
    let scan = scan_workspace(&fixture("marked_unknown_markers"), None).expect("scan completes");
    for m in &scan.marked_unknowns {
        assert!(
            !m.trigger.trim().is_empty(),
            "a surfaced marked-unknown must carry a non-empty trigger: {m:?}"
        );
    }
}
