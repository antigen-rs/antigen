//! `cargo antigen fingerprint`: print the structural fingerprint of scanned
//! immune/presents sites.
//!
//! The same digest `attest scaffold`/`sign` need and `scan --format json`
//! surfaces, exposed as a first-class verb so an operator can obtain a
//! fingerprint WITHOUT scaffolding first — for the `sign` step, for hand-editing
//! a sidecar, or for scripting (`FP=$(cargo antigen fingerprint --antigen X
//! --item-path y --format json | jq -r '.[0].structural_fingerprint')`).
//!
//! Exit-code contract these tests pin:
//!   0 = printed at least one match, OR no filter was given (empty workspace is
//!       not an error).
//!   1 = a filter (`--antigen` / `--item-path`) was given but matched nothing —
//!       so scripts can't silently capture an empty fingerprint.
//!   2 = root does not exist / scan failed.

use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cargo-antigen"))
}

fn fingerprint(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(bin())
        .arg("antigen")
        .arg("fingerprint")
        .args(args)
        .output()
        .expect("failed to run cargo-antigen");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// Stage a single-file crate with one `#[immune]` site and one `#[presents]`
/// site on a structurally-matching function (so the fingerprint scan finds
/// both). Returns `(tempdir, crate_root)`.
fn staged_crate() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let src_dir = tmp.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    let lib = r##"
use antigen::{antigen, immune, presents};

#[antigen(
    name = "demo-discipline",
    family = "forgotten-lesson",
    fingerprint = r#"item = fn"#,
    summary = "demo antigen for the fingerprint subcommand tests"
)]
pub struct DemoDiscipline;

#[immune(DemoDiscipline, witness = test(demo_witness))]
pub fn guarded_fn(x: i32) -> i32 {
    x + 1
}

#[presents(DemoDiscipline)]
pub fn other_fn(x: i32) -> i32 {
    x - 1
}
"##;
    std::fs::write(src_dir.join("lib.rs"), lib).unwrap();
    (tmp, src_dir)
}

#[test]
fn fingerprint_reports_immune_site_filtered_by_antigen() {
    let (_tmp, root) = staged_crate();
    let (code, stdout, stderr) = fingerprint(&[
        "--root",
        root.to_str().unwrap(),
        "--antigen",
        "DemoDiscipline",
    ]);
    assert_eq!(code, 0, "filtered match must exit 0: {stderr}");
    assert!(
        stdout.contains("fnv1a64:"),
        "human output must include the structural fingerprint: {stdout}"
    );
    assert!(
        stdout.contains("guarded_fn") && stdout.contains("[immune]"),
        "must report the immune site by item label + kind: {stdout}"
    );
}

#[test]
fn fingerprint_json_is_scriptable() {
    let (_tmp, root) = staged_crate();
    let (code, stdout, stderr) = fingerprint(&[
        "--root",
        root.to_str().unwrap(),
        "--antigen",
        "DemoDiscipline",
        "--item-path",
        "guarded_fn",
        "--format",
        "json",
    ]);
    assert_eq!(code, 0, "json match must exit 0: {stderr}");

    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON array");
    let arr = v.as_array().expect("top-level array");
    assert!(
        !arr.is_empty(),
        "item-path filter must match the named site"
    );
    // Every reported row is the named item. A single `#[immune]` site also
    // surfaces a same-line `presents` row (the immune declaration implies a
    // presentation, and the `item = fn` pattern fingerprint-matches it), so
    // there may be more than one row — but they all describe `guarded_fn` and
    // share its digest.
    for m in arr {
        assert_eq!(
            m["item_path"], "guarded_fn",
            "filter must narrow to the named item"
        );
        let fp = m["structural_fingerprint"].as_str().unwrap();
        assert!(
            fp.starts_with("fnv1a64:"),
            "fingerprint is the FNV digest: {fp}"
        );
    }
    assert!(
        arr.iter().any(|m| m["site_kind"] == "immune"),
        "the immune site must be reported: {arr:?}"
    );
}

#[test]
fn fingerprint_unfiltered_reports_all_sites() {
    let (_tmp, root) = staged_crate();
    let (code, stdout, _stderr) =
        fingerprint(&["--root", root.to_str().unwrap(), "--format", "json"]);
    assert_eq!(code, 0);
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let arr = v.as_array().unwrap();
    // The immune site (always fingerprinted) plus the presents site (a
    // fingerprint-match against the `item = fn` pattern) both appear.
    let kinds: Vec<&str> = arr
        .iter()
        .map(|m| m["site_kind"].as_str().unwrap())
        .collect();
    assert!(
        kinds.contains(&"immune"),
        "unfiltered output must include the immune site: {kinds:?}"
    );
    assert!(
        kinds.contains(&"presents"),
        "unfiltered output must include the presents site: {kinds:?}"
    );
}

#[test]
fn fingerprint_filtered_no_match_exits_1() {
    let (_tmp, root) = staged_crate();
    let (code, stdout, stderr) = fingerprint(&[
        "--root",
        root.to_str().unwrap(),
        "--antigen",
        "NonexistentDiscipline",
    ]);
    assert_eq!(
        code, 1,
        "a filter that matches nothing must exit 1 so scripts don't capture empty: \
         stdout={stdout} stderr={stderr}"
    );
    assert!(
        stdout.is_empty(),
        "no-match must print nothing to stdout (the script-captured stream): {stdout}"
    );
    assert!(
        stderr.contains("no immune/presents site"),
        "no-match must explain on stderr: {stderr}"
    );
}

#[test]
fn fingerprint_missing_root_exits_2() {
    let ghost = Path::new("definitely/not/a/real/path/xyzzy");
    let (code, _stdout, stderr) = fingerprint(&["--root", ghost.to_str().unwrap()]);
    assert_eq!(code, 2, "nonexistent root must exit 2: {stderr}");
}
