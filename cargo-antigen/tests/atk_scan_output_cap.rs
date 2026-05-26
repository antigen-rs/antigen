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
