//! `attest scaffold` fingerprint auto-fill: when `--fingerprint` is omitted,
//! scaffold scans the source file's crate subtree and fills the matching
//! item's `structural_fingerprint` automatically.
//!
//! ## Why this exists
//!
//! Before auto-fill, an operator running `attest scaffold` without a
//! fingerprint got an empty `current_fingerprint` placeholder plus a note
//! telling them to run `scan --format json | jq '... .structural_fingerprint'`
//! and hand-edit the JSON. That manual round-trip is exactly the friction the
//! tool exists to remove — and an empty fingerprint silently breaks any
//! `against = "current"` / `fresh_within_days` predicate at audit time.
//!
//! Auto-fill composes with the existing scan machinery (it does not re-derive
//! the item-walk): scaffold scans the source file's directory, matches the
//! immunity/presentation the operator named, and uses its computed digest.
//!
//! These tests drive the compiled binary against a temp crate so they assert
//! the real observable behavior (sidecar contents, stderr, exit codes).

use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cargo-antigen"))
}

fn attest(args: &[&str]) -> (i32, String) {
    let out = Command::new(bin())
        .arg("antigen")
        .arg("attest")
        .args(args)
        .output()
        .expect("failed to run cargo-antigen");
    let exit = out.status.code().unwrap_or(-1);
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    (exit, stderr)
}

/// Read a sidecar's first item's `current_fingerprint`.
fn current_fingerprint(sidecar: &Path) -> String {
    let content = std::fs::read_to_string(sidecar).expect("read sidecar");
    let v: serde_json::Value = serde_json::from_str(&content).expect("parse sidecar JSON");
    v["items"][0]["current_fingerprint"]
        .as_str()
        .expect("current_fingerprint is a string")
        .to_string()
}

/// Stage a single-file crate whose source carries one `#[immune]` site, and
/// return `(tempdir, source_file_path, expected_sidecar_path)`.
///
/// The `#[immune]` predicate is deliberately trivial — the auto-fill only needs
/// scan to find the immunity and compute its structural fingerprint; the
/// predicate content is irrelevant to fingerprinting.
fn staged_immune_crate() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let src_dir = tmp.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();

    // A minimal crate the scanner can walk. The antigen + immune declarations
    // are what scan keys on; the bodies are inert.
    let lib = r##"
use antigen::{antigen, immune};

#[antigen(
    name = "demo-discipline",
    family = "forgotten-lesson",
    fingerprint = r#"item = fn"#,
    summary = "demo antigen used to exercise scaffold fingerprint auto-fill in tests"
)]
pub struct DemoDiscipline;

#[immune(DemoDiscipline, witness = test(demo_witness_test))]
pub fn guarded_fn(x: i32) -> i32 {
    x + 1
}
"##;
    let source_file = src_dir.join("lib.rs");
    std::fs::write(&source_file, lib).unwrap();

    let sidecar = src_dir.join(".attest").join("DemoDiscipline.json");
    (tmp, source_file, sidecar)
}

/// Capture the fingerprint scan computes for the staged crate's immune site, so
/// the auto-fill assertion compares against the real value rather than a
/// hard-coded digest (which would go stale if the digest algorithm changes).
fn scanned_fingerprint(source_file: &Path) -> String {
    let scan_root = source_file.parent().unwrap();
    let out = Command::new(bin())
        .args([
            "antigen",
            "scan",
            "--root",
            scan_root.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .expect("run scan");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let doc: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("scan JSON parse ({e}):\n{stdout}"));
    let imms = doc["report"]["immunities"].as_array().unwrap();
    imms.iter()
        .find(|i| i["antigen_type"] == "DemoDiscipline")
        .and_then(|i| i["structural_fingerprint"].as_str())
        .unwrap_or_else(|| panic!("scan must find DemoDiscipline immunity:\n{stdout}"))
        .to_string()
}

// ============================================================================
// Happy path: omitting --fingerprint auto-fills from scan.
// ============================================================================

#[test]
fn scaffold_autofills_fingerprint_when_omitted() {
    let (_tmp, source_file, sidecar) = staged_immune_crate();
    let expected = scanned_fingerprint(&source_file);
    assert!(
        expected.contains(':'),
        "sanity: scanned fingerprint should be `<algo>:<hex>`, got `{expected}`"
    );

    let (code, stderr) = attest(&[
        "scaffold",
        "--antigen",
        "DemoDiscipline",
        "--source-file",
        source_file.to_str().unwrap(),
        "--item-path",
        "guarded_fn",
        // NB: no --fingerprint.
    ]);
    assert_eq!(code, 0, "scaffold must succeed: {stderr}");
    assert!(
        stderr.contains("auto-filled"),
        "scaffold should report it auto-filled the fingerprint: {stderr}"
    );

    assert_eq!(
        current_fingerprint(&sidecar),
        expected,
        "the sidecar's current_fingerprint must be the scanned digest, not an \
         empty placeholder"
    );
}

// ============================================================================
// Explicit --fingerprint always wins (no scan, value passed through verbatim).
// ============================================================================

#[test]
fn scaffold_explicit_fingerprint_is_not_overridden_by_autofill() {
    let (_tmp, source_file, sidecar) = staged_immune_crate();

    let (code, _stderr) = attest(&[
        "scaffold",
        "--antigen",
        "DemoDiscipline",
        "--source-file",
        source_file.to_str().unwrap(),
        "--item-path",
        "guarded_fn",
        "--fingerprint",
        "fnv1a64:deadbeefdeadbeef",
    ]);
    assert_eq!(code, 0);
    assert_eq!(
        current_fingerprint(&sidecar),
        "fnv1a64:deadbeefdeadbeef",
        "an explicit --fingerprint must be written verbatim, never replaced by \
         auto-fill"
    );
}

// ============================================================================
// No match: auto-fill falls back to the empty placeholder (never a hard fail).
// ============================================================================

#[test]
fn scaffold_falls_back_to_placeholder_when_no_scan_match() {
    let (_tmp, source_file, _demo_sidecar) = staged_immune_crate();

    // Scaffold for an antigen that does NOT appear in the source — auto-fill
    // finds nothing and must fall back to the empty placeholder, still exit 0.
    let (code, stderr) = attest(&[
        "scaffold",
        "--antigen",
        "NonexistentDiscipline",
        "--source-file",
        source_file.to_str().unwrap(),
        "--item-path",
        "guarded_fn",
    ]);
    assert_eq!(
        code, 0,
        "no-match auto-fill must fall back gracefully (exit 0), not fail: {stderr}"
    );
    assert!(
        stderr.contains("fingerprint is empty"),
        "fallback must surface the empty-fingerprint guidance: {stderr}"
    );

    // The sidecar for THIS antigen (its own stem) carries the empty placeholder.
    let sidecar = source_file
        .parent()
        .unwrap()
        .join(".attest")
        .join("NonexistentDiscipline.json");
    assert_eq!(
        current_fingerprint(&sidecar),
        "",
        "no-match scaffold leaves an empty placeholder for the operator to fill"
    );
}
