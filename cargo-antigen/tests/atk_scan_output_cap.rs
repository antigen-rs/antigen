//! `cargo antigen scan` human output must CAP per-antigen fingerprint matches.
//!
//! ## The failure this guards (outsider + observer, 2026-05-26)
//!
//! `print_fingerprint_matches` printed EVERY fingerprint match, one per line,
//! unbounded. Observer measured 18,436 such lines on antigen's own tree. A
//! newcomer's first `cargo antigen scan` produced an 18K-line wall of
//! "[fingerprint match]" advisories — each implicitly reading as a TODO — even
//! though the design (glossary filter/proof split) says fingerprint matches are
//! EXPECTED NOISE the witness layer refines, not action items. A tool that
//! floods a first-time user teaches them the tool is noise: the opposite of
//! onboarding.
//!
//! The fix: human output groups matches by antigen type and shows at most
//! `MAX_FINGERPRINT_MATCHES_PER_ANTIGEN` per group, then a "+N more" summary
//! pointing at `--format json` for full enumeration (which CI gates use).
//! `--format json` stays exhaustive.
//!
//! These tests pin the bound (the human stream is bounded regardless of match
//! count) and the invariant that JSON is NOT capped.

use std::fmt::Write as _;
use std::path::PathBuf;
use std::process::Command;

/// Must match `MAX_FINGERPRINT_MATCHES_PER_ANTIGEN` in main.rs. Kept as a local
/// const so the test reads as a contract, not a magic number.
const CAP: usize = 10;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cargo-antigen"))
}

/// Stage a crate with one antigen whose `item = struct` fingerprint matches
/// `n_structs` distinct structs — so scan produces `n_structs` fingerprint
/// matches for a single antigen type. Returns `(tempdir, crate_root)`.
fn staged_many_matches(n_structs: usize) -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let src_dir = tmp.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();

    let mut lib = String::from(
        r##"use antigen::antigen;

#[antigen(
    name = "every-struct",
    family = "forgotten-lesson",
    fingerprint = r#"item = struct"#,
    summary = "matches every struct — used to flood the scanner in cap tests"
)]
pub struct EveryStruct;

"##,
    );
    for i in 0..n_structs {
        // Each struct structurally matches `item = struct`, producing one
        // fingerprint-match presentation.
        writeln!(lib, "pub struct Flood{i};").unwrap();
    }

    std::fs::write(src_dir.join("lib.rs"), lib).unwrap();
    (tmp, src_dir)
}

fn scan_human_stdout(root: &std::path::Path) -> String {
    let out = Command::new(bin())
        .args(["antigen", "scan", "--root", root.to_str().unwrap()])
        .output()
        .expect("run scan");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn scan_human_output_caps_fingerprint_matches_per_antigen() {
    let n = 40; // well over the cap
    let (_tmp, root) = staged_many_matches(n);
    let stdout = scan_human_stdout(&root);

    // Count the per-site match lines. Each capped detail line carries the
    // "[fingerprint match]" marker; the summary "+N more" line does not.
    let detail_lines = stdout.matches("[fingerprint match]").count();
    assert!(
        detail_lines <= CAP,
        "human scan output must cap per-antigen fingerprint detail lines at {CAP}, \
         got {detail_lines} for {n} matches. The unbounded wall-of-output is the \
         exact onboarding failure this guards.\n---\n{stdout}"
    );

    // The truncation must be SIGNALED, not silent — the newcomer needs to know
    // there are more and where to see them.
    assert!(
        stdout.contains("more") && stdout.contains("--format json"),
        "capped output must point at `--format json` for the full list: {stdout}"
    );
}

#[test]
fn scan_human_output_reframes_matches_as_expected_candidates() {
    let (_tmp, root) = staged_many_matches(3);
    let stdout = scan_human_stdout(&root);

    // The framing must NOT read as a mandatory TODO list. Per the glossary
    // filter/proof split, fingerprint matches are candidate sites the witness
    // layer refines — "candidate" + "expected" carry that intent.
    assert!(
        stdout.contains("candidate") || stdout.contains("expected"),
        "fingerprint-match output should frame matches as expected candidate \
         sites, not a TODO list: {stdout}"
    );
}

#[test]
fn scan_json_output_is_not_capped() {
    let n = 40;
    let (_tmp, root) = staged_many_matches(n);
    let out = Command::new(bin())
        .args([
            "antigen",
            "scan",
            "--root",
            root.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .expect("run scan json");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let doc: serde_json::Value = serde_json::from_str(&stdout).expect("scan JSON parses");

    let fp_matches = doc["report"]["presentations"]
        .as_array()
        .expect("presentations array")
        .iter()
        .filter(|p| p["match_kind"] == "fingerprint_match")
        .count();
    assert!(
        fp_matches >= n,
        "JSON output must enumerate ALL fingerprint matches (the cap is human-only \
         — CI gates rely on full JSON enumeration). Expected >= {n}, got {fp_matches}."
    );
}

// ============================================================================
// ATK-orphaned-lineage-delivery-arm: orphaned lineage edges are computed but
// never rendered in scan output (forward/descended-from-structural-verification).
//
// `ScanReport::orphaned_lineage_edges()` and `dangling_child_lineage_edges()`
// correctly detect orphaned `#[descended_from]` declarations — but
// cargo-antigen's scan subcommand NEVER calls these methods and never includes
// their results in either human or JSON output.
//
// An author who writes `#[descended_from(NonExistentParent)]` after accidentally
// removing or renaming the parent antigen gets ZERO warning from `cargo antigen
// scan`. The orphan is silently present in `lineage_edges` (the raw edge is
// recorded) but the orphan status is invisible.
//
// This test documents the delivery arm gap:
//   1. Stage a workspace with an orphaned #[descended_from] edge.
//   2. Run cargo antigen scan --format json.
//   3. Verify the lineage_edges array contains the edge (scan recorded it).
//   4. Verify the JSON output has NO "orphaned" or similar diagnostic field
//      — the gap is the ABSENCE of orphan diagnostic in the output.
//
// FIX DIRECTION:
// Wire ScanReport::orphaned_lineage_edges() result into the scan output render
// path — either as a separate JSON field (`orphaned_lineage_edges: [...]`) or
// as entries in `parse_failures` with a diagnostic message. The human output
// should warn "N orphaned lineage edges detected: ...".
// ============================================================================

#[test]
fn atk_orphaned_lineage_edge_delivered_in_scan_json() {
    // Stage a source directory with:
    //   - ChildClass: declares #[descended_from(ParentClass)] but ParentClass is never declared
    //   - This creates an orphaned lineage edge in the scan report.
    let tmp = tempfile::tempdir().expect("tempdir");
    let src_dir = tmp.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();

    std::fs::write(
        src_dir.join("lib.rs"),
        r#"use antigen::{antigen, descended_from};

#[antigen(
    name = "child-class",
    summary = "A child antigen that descends from a non-existent parent."
)]
#[descended_from(ParentClass)]
pub struct ChildClass;

// ParentClass is deliberately NOT declared — creating an orphaned lineage edge.
"#,
    )
    .unwrap();

    let out = Command::new(bin())
        .args([
            "antigen",
            "scan",
            "--root",
            src_dir.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .expect("run scan json");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let doc: serde_json::Value = serde_json::from_str(&stdout).expect("scan JSON parses");

    // (3): lineage_edges contains the edge — scan recorded it.
    let lineage_edges = doc["report"]["lineage_edges"]
        .as_array()
        .expect("lineage_edges must be an array");
    let child_edge = lineage_edges
        .iter()
        .find(|e| e["child"].as_str() == Some("ChildClass"));
    assert!(
        child_edge.is_some(),
        "ATK-orphaned-lineage: lineage_edges must contain the ChildClass->ParentClass edge \
         (scan records it); got {lineage_edges:?}"
    );
    assert_eq!(
        child_edge.unwrap()["parent"].as_str(),
        Some("ParentClass"),
        "ATK-orphaned-lineage: edge parent must be ParentClass"
    );

    // (4): DELIVERY ARM FIXED (bf60e5d+). The scan JSON now DELIVERS
    // the computed orphaned_lineage_edges() verdict as a top-level field
    // (sibling to `unaddressed` — the same computed-then-attached pattern; the
    // query is derived state, not stored on ScanReport). An author running
    // `cargo antigen scan --format json` now sees the broken lineage.
    let orphan_field = doc
        .get("orphaned_lineage_edges")
        .and_then(|v| v.as_array())
        .expect("scan JSON must DELIVER orphaned_lineage_edges (top-level array)");
    assert!(
        orphan_field
            .iter()
            .any(|e| e["child"].as_str() == Some("ChildClass")
                && e["parent"].as_str() == Some("ParentClass")),
        "ATK-orphaned-lineage: orphaned_lineage_edges must contain the \
         ChildClass⟶ParentClass edge (parent not declared); got {orphan_field:?}"
    );
    let _ = child_edge; // edge recorded in lineage_edges AND surfaced as orphaned.
}

#[test]
fn atk_orphaned_lineage_human_output_warns() {
    // Same scenario as above but checks human-readable output.
    // The human output also carries no orphan warning — an author running
    // `cargo antigen scan` without --format json sees nothing about the orphan.
    let tmp = tempfile::tempdir().expect("tempdir");
    let src_dir = tmp.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();

    std::fs::write(
        src_dir.join("lib.rs"),
        r#"use antigen::{antigen, descended_from};

#[antigen(
    name = "child-class",
    summary = "Orphan test."
)]
#[descended_from(NonExistentParent)]
pub struct ChildClass;
"#,
    )
    .unwrap();

    let out = Command::new(bin())
        .args(["antigen", "scan", "--root", src_dir.to_str().unwrap()])
        .output()
        .expect("run scan");
    let stdout = String::from_utf8_lossy(&out.stdout);

    // DELIVERY ARM FIXED: the human output now WARNS about the
    // orphaned lineage edge and names the missing parent, so an author running
    // `cargo antigen scan` (no --format json) sees their broken lineage.
    let mentions_orphan =
        stdout.to_lowercase().contains("orphan") && stdout.contains("NonExistentParent");

    assert!(
        mentions_orphan,
        "ATK-orphaned-lineage-human: human output must WARN about the orphaned \
         lineage edge and name the missing parent (NonExistentParent). \
         Current output:\n{stdout}"
    );
}
