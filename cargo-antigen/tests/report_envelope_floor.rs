//! The report-as-live-projection floor: every machine-readable report carries
//! a provenance ENVELOPE, and `--output <file>` saves a *render of the run*.
//!
//! ## What this floor is (and is not)
//!
//! antigen's own thesis forbids a stored, release-anchored report: the instant
//! a report is committed it becomes a `ParallelStateTrackersDiverge` instance —
//! a second copy of the truth that can drift from the code it describes. So the
//! report is NEVER stored-and-read-back. `scan` / `audit` recompute it from the
//! current code every run (like clippy reflecting current source every
//! invocation); the report is just their output.
//!
//! The envelope makes each render self-describing — `antigen_version`,
//! `git_sha`, `generated_at`, `report_schema_version` — so a saved render
//! (`--output`, a SARIF-style dump) carries the provenance to interpret it
//! later WITHOUT antigen reading it back as authoritative. Running `audit` at a
//! tagged commit *is* that tag's defense-posture SBOM, regenerable any time.
//!
//! ## What these tests pin
//!
//! 1. The envelope EXTENDS the stabilized scan-json / audit-json (does not
//!    fork): the four provenance keys are additive siblings, and the existing
//!    payload keys (`report`, `scan`, `audit`, …) stay where consumers read
//!    them — byte-compatible for key-navigating consumers.
//! 2. `git_sha` is present inside a git repo and ABSENT (not a fake value, not
//!    an error) outside one — tier-honest, matching `read_commit_trailers`.
//! 3. `--output <file>` writes the enveloped render and overwrites it each run
//!    (recomputed, never accumulated).
//! 4. The render is a recomputation, not a read-back: two runs produce two
//!    fresh `generated_at` stamps.

use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cargo-antigen"))
}

/// Stage a minimal single-crate workspace with one antigen declaration and one
/// presents-site, so scan/audit have something to report. Returns
/// `(tempdir, crate_root)`. NOT initialized as a git repo — git-state tests opt
/// in explicitly.
fn staged_crate() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let src = tmp.path().join("src");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(
        tmp.path().join("Cargo.toml"),
        "[package]\nname = \"staged\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\
         [lib]\npath = \"src/lib.rs\"\n",
    )
    .unwrap();
    std::fs::write(
        src.join("lib.rs"),
        r##"use antigen::antigen;

#[antigen(
    name = "sample-class",
    family = "forgotten-lesson",
    fingerprint = r#"item = struct"#,
    summary = "a sample failure-class for the report-envelope floor tests"
)]
pub struct SampleClass;

pub struct Defended;
"##,
    )
    .unwrap();
    let root = tmp.path().to_path_buf();
    (tmp, root)
}

/// Initialize `dir` as a git repo with one commit, returning the HEAD SHA.
/// `git` failures fail the test loudly — these are the tests that REQUIRE a repo.
fn init_repo_with_commit(dir: &Path) -> String {
    let run = |args: &[&str]| {
        let out = Command::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .unwrap_or_else(|e| panic!("git {args:?} failed to spawn: {e}"));
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        out
    };
    run(&["init"]);
    run(&["config", "user.email", "test@antigen.test"]);
    run(&["config", "user.name", "report-floor-test"]);
    run(&["add", "-A"]);
    run(&["commit", "-m", "stage", "--no-gpg-sign"]);
    let head = run(&["rev-parse", "HEAD"]);
    String::from_utf8_lossy(&head.stdout).trim().to_owned()
}

fn scan_json(root: &Path) -> serde_json::Value {
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
    serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).expect("scan JSON parses")
}

fn audit_json(root: &Path) -> serde_json::Value {
    let out = Command::new(bin())
        .args([
            "antigen",
            "audit",
            "--root",
            root.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .expect("run audit json");
    serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).expect("audit JSON parses")
}

/// The envelope's four provenance keys, asserted on any enveloped report. The
/// `expect_sha` arg pins whether `git_sha` should be present (in-repo) or
/// absent (skip-serialized outside a repo).
fn assert_envelope(doc: &serde_json::Value, expect_sha: Option<&str>) {
    assert_eq!(
        doc["antigen_version"].as_str(),
        Some(env!("CARGO_PKG_VERSION")),
        "envelope must stamp the producing antigen version"
    );
    assert!(
        doc["report_schema_version"].as_u64() == Some(1),
        "envelope must carry report_schema_version = 1 (got {:?})",
        doc.get("report_schema_version")
    );
    let ts = doc["generated_at"]
        .as_str()
        .expect("envelope must carry an RFC3339 generated_at timestamp");
    // RFC3339 has the date-time `T` separator and an offset; a cheap shape check
    // that catches an empty or obviously-wrong stamp without pulling in a parser.
    assert!(
        ts.contains('T') && (ts.contains('+') || ts.ends_with('Z')),
        "generated_at must be RFC3339 (got {ts:?})"
    );
    match expect_sha {
        Some(sha) => assert_eq!(
            doc["git_sha"].as_str(),
            Some(sha),
            "in a git repo, git_sha must be the workspace HEAD"
        ),
        None => assert!(
            doc.get("git_sha").is_none(),
            "outside a git repo, git_sha must be ABSENT (skip-serialized), not a \
             fake value: {:?}",
            doc.get("git_sha")
        ),
    }
}

#[test]
fn scan_json_carries_provenance_envelope_extending_not_forking() {
    let (_tmp, root) = staged_crate();
    let doc = scan_json(&root);
    // Not a repo → git_sha absent.
    assert_envelope(&doc, None);
    // EXTEND, not fork: the existing payload key is still at the top level
    // exactly where existing consumers navigate.
    assert!(
        doc.get("report").is_some(),
        "the `report` payload key must remain a top-level sibling of the \
         envelope keys (extend, not fork): {doc}"
    );
    assert!(
        doc["report"].get("antigens").is_some(),
        "the flattened payload must preserve the scan report's own shape \
         (report.antigens): {doc}"
    );
}

#[test]
fn scan_json_git_sha_present_inside_a_repo() {
    let (_tmp, root) = staged_crate();
    let head = init_repo_with_commit(&root);
    let doc = scan_json(&root);
    assert_envelope(&doc, Some(&head));
}

#[test]
fn audit_json_carries_provenance_envelope_extending_not_forking() {
    let (_tmp, root) = staged_crate();
    let doc = audit_json(&root);
    assert_envelope(&doc, None);
    // The audit payload's own top-level keys must survive the flatten.
    for key in ["scan", "audit", "category"] {
        assert!(
            doc.get(key).is_some(),
            "the audit payload key `{key}` must remain a top-level sibling of \
             the envelope keys: {doc}"
        );
    }
}

#[test]
fn output_flag_writes_an_enveloped_render_to_file() {
    let (tmp, root) = staged_crate();
    let out_path = tmp.path().join("render.json");
    let out = Command::new(bin())
        .args([
            "antigen",
            "scan",
            "--root",
            root.to_str().unwrap(),
            "--output",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("run scan --output");
    assert!(out.status.success(), "scan --output should succeed");
    let content = std::fs::read_to_string(&out_path).expect("render file must exist");
    let doc: serde_json::Value =
        serde_json::from_str(&content).expect("the render file must be valid JSON");
    // The file is a full enveloped render regardless of console --format.
    assert_envelope(&doc, None);
    assert!(
        doc.get("report").is_some(),
        "the --output render must carry the full report payload: {doc}"
    );
}

#[test]
fn audit_output_flag_writes_an_enveloped_render_to_file() {
    let (tmp, root) = staged_crate();
    let out_path = tmp.path().join("audit-render.json");
    let out = Command::new(bin())
        .args([
            "antigen",
            "audit",
            "--root",
            root.to_str().unwrap(),
            "--output",
            out_path.to_str().unwrap(),
        ])
        .output()
        .expect("run audit --output");
    assert!(out.status.success(), "audit --output should succeed");
    let content = std::fs::read_to_string(&out_path).expect("audit render file must exist");
    let doc: serde_json::Value =
        serde_json::from_str(&content).expect("the audit render file must be valid JSON");
    assert_envelope(&doc, None);
    assert!(
        doc.get("audit").is_some(),
        "the audit --output render must carry the audit payload: {doc}"
    );
}

#[test]
fn render_is_recomputed_each_run_never_accumulated() {
    // The whole point of "live projection, not stored truth": each run
    // recomputes and re-stamps. The file is overwritten (one render, not a
    // growing log), and a second run produces a fresh timestamp — antigen never
    // reads the prior file back.
    let (tmp, root) = staged_crate();
    let out_path = tmp.path().join("render.json");
    let run_once = || {
        let out = Command::new(bin())
            .args([
                "antigen",
                "scan",
                "--root",
                root.to_str().unwrap(),
                "--output",
                out_path.to_str().unwrap(),
            ])
            .output()
            .expect("run scan --output");
        assert!(out.status.success());
        let content = std::fs::read_to_string(&out_path).unwrap();
        let doc: serde_json::Value = serde_json::from_str(&content).unwrap();
        doc["generated_at"].as_str().unwrap().to_owned()
    };
    let first = run_once();
    // Sleep a hair so the RFC3339 stamp (sub-second precision) is guaranteed to
    // advance; without it two runs inside the same instant could tie.
    std::thread::sleep(std::time::Duration::from_millis(5));
    let second = run_once();
    assert_ne!(
        first, second,
        "each run must re-stamp generated_at (recomputed live, not read back \
         from the prior render)"
    );
    // The file holds exactly one render, not two concatenated — it parses as a
    // single JSON document, which the second run_once() already proved.
}
