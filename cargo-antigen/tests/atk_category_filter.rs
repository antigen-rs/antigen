//! CLI contract lock-down for the `--category` filter (ADR-028 §CLI integration,
//! G3 deliverable).
//!
//! Tests invoke the compiled `cargo-antigen` binary directly and assert the
//! observable contract of `scan --category` / `audit --category`:
//!   - an unrecognized category value exits 2 with a diagnostic
//!   - a valid category filters the audit's category report (the
//!     `antigen/examples` tree has absent-category declarations that default to
//!     functional-correctness, so the functional-correctness filter surfaces
//!     the defaulted-migration hint while the substrate-alignment filter does
//!     not)

use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cargo-antigen"))
}

/// Run `cargo-antigen antigen <args>` and return `(exit_code, stdout, stderr)`.
fn run(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(bin())
        .arg("antigen")
        .args(args)
        .output()
        .expect("failed to run cargo-antigen");
    let exit = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    (exit, stdout, stderr)
}

/// Workspace examples dir (relative to the cargo-antigen crate at test time).
fn examples_root() -> String {
    // CARGO_MANIFEST_DIR is .../cargo-antigen; examples live at ../antigen/examples.
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .unwrap()
        .join("antigen")
        .join("examples")
        .to_string_lossy()
        .into_owned()
}

#[test]
fn category_filter_unrecognized_value_exits_2() {
    let (exit, _stdout, stderr) =
        run(&["audit", "--root", &examples_root(), "--category", "bogus"]);
    assert_eq!(exit, 2, "unrecognized --category must exit 2");
    assert!(
        stderr.contains("unrecognized --category"),
        "stderr should name the unrecognized category; got: {stderr:?}"
    );
}

#[test]
fn category_filter_scan_unrecognized_value_exits_2() {
    let (exit, _stdout, stderr) = run(&["scan", "--root", &examples_root(), "--category", "nope"]);
    assert_eq!(exit, 2, "unrecognized --category on scan must exit 2");
    assert!(stderr.contains("unrecognized --category"));
}

#[test]
fn category_filter_functional_correctness_surfaces_defaulted() {
    // The examples tree declares antigens without an explicit category; they
    // default to FunctionalCorrectness and emit the migration hint. Filtering
    // to functional-correctness keeps them.
    let (exit, stdout, _stderr) = run(&[
        "audit",
        "--root",
        &examples_root(),
        "--category",
        "functional-correctness",
    ]);
    assert!(
        exit == 0 || exit == 1,
        "audit exit should be 0/1, got {exit}"
    );
    assert!(
        stdout.contains("antigen-category-defaulted-implicit-functional"),
        "functional-correctness filter should retain the defaulted declarations"
    );
}

#[test]
fn category_filter_substrate_alignment_excludes_defaulted() {
    // Absent-category declarations default to FunctionalCorrectness, so the
    // substrate-alignment filter must NOT surface the defaulted-migration hint.
    let (exit, stdout, _stderr) = run(&[
        "audit",
        "--root",
        &examples_root(),
        "--category",
        "substrate-alignment",
    ]);
    assert!(
        exit == 0 || exit == 1,
        "audit exit should be 0/1, got {exit}"
    );
    assert!(
        !stdout.contains("antigen-category-defaulted-implicit-functional"),
        "substrate-alignment filter must exclude functional-correctness-defaulted declarations"
    );
}
