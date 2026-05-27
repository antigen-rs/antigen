//! ATK: `cargo antigen audit --strict` exit-code contract.
//!
//! `--strict` gates CI by returning exit code 1 when unaddressed EXPLICIT
//! presentations exist (`match_kind == ExplicitMarker`), and exit code 0
//! otherwise. Three contract invariants:
//!
//! 1. **Without `--strict`**: unaddressed explicit presents-sites → exit 0
//!    (audit is informational only; CI does not fail).
//!
//! 2. **With `--strict`**: unaddressed explicit presents-sites → exit 1.
//!
//! 3. **`FingerprintMatch` sites do NOT trigger `--strict`**: a workspace with
//!    only fingerprint-match (inferred) unaddressed sites exits 0 under
//!    `--strict`. The output may say "N unaddressed" but the exit code is 0.
//!    This prevents the silent mismatch where CI fails with a confusing
//!    message ("All explicit presentations are addressed") while still
//!    exiting 1.
//!
//! The critical invariant: `--strict` gates ONLY on `ExplicitMarker`; fingerprint
//! matches are advisory noise requiring human triage. Violating this creates
//! a CI false-positive: the tool exits 1 but the human-readable output says
//! "nothing to fix" — an undiagnosable CI failure.
//!
//! ATK cases:
//!   `atk_strict_no_flag_unaddressed_exits_zero`           — no --strict → exit 0
//!   `atk_strict_flag_unaddressed_explicit_exits_one`      — --strict + unaddressed → exit 1
//!   `atk_strict_flag_fingerprint_match_only_exits_zero`   — --strict + only FP matches → exit 0
//!   `atk_strict_flag_no_presentations_exits_zero`         — --strict + clean workspace → exit 0

use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cargo-antigen"))
}

/// Run `cargo antigen audit [--strict] --root <root>` and return the exit code.
fn run_audit(root: &std::path::Path, strict: bool) -> i32 {
    let mut cmd = Command::new(bin());
    cmd.arg("antigen").arg("audit");
    if strict {
        cmd.arg("--strict");
    }
    cmd.arg("--root").arg(root);
    let out = cmd.output().expect("failed to run cargo-antigen audit");
    out.status.code().unwrap_or(-1)
}

/// Stage a crate with one antigen declaration and one explicit `#[presents(X)]`
/// with no corresponding `#[defended_by(X)]`. Scan will produce one unaddressed
/// explicit presentation. Returns the tempdir (keep alive) and the crate root.
fn staged_unaddressed_explicit() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let src_dir = tmp.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();

    // The antigen declaration and an unaddressed presents-site.
    // No #[defended_by(UnaddressedClass)] anywhere — audit will find one
    // unaddressed explicit presentation.
    let lib = r#"use antigen::{antigen, presents};

#[antigen(
    name = "unaddressed-class",
    family = "test-family",
    summary = "a test failure class"
)]
pub struct UnaddressedClass;

/// A site that presents UnaddressedClass but has no corresponding witness.
#[presents(UnaddressedClass)]
pub fn vulnerable_fn() -> u32 {
    42
}
"#;

    std::fs::write(src_dir.join("lib.rs"), lib).unwrap();
    (tmp, src_dir)
}

/// Stage a crate with one antigen whose fingerprint matches some structs.
/// No explicit `#[presents]` — all matches are `FingerprintMatch` (inferred).
/// Under `--strict`, these must NOT trigger exit 1.
fn staged_fingerprint_match_only() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let src_dir = tmp.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();

    // Fingerprint `item = struct` is single-quoted (no special chars) so it
    // works inside the outer Rust raw-string literal without delimiter clashes.
    let lib = r#"use antigen::antigen;

#[antigen(
    name = "struct-pattern",
    family = "test-family",
    fingerprint = "item = struct",
    summary = "fingerprint that matches all structs in this file"
)]
pub struct StructPatternAntigen;

// These structs trigger FingerprintMatch presentations (inferred, not explicit).
// They must NOT cause --strict to exit 1.
pub struct MatchedByFingerprint1;
pub struct MatchedByFingerprint2;
"#;

    std::fs::write(src_dir.join("lib.rs"), lib).unwrap();
    (tmp, src_dir)
}

/// Stage a crate with no presentations at all (just an antigen declaration).
/// `--strict` must exit 0 — no unaddressed sites.
fn staged_clean() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let src_dir = tmp.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();

    let lib = r#"use antigen::antigen;

#[antigen(
    name = "clean-class",
    family = "test-family",
    summary = "a class with no vulnerable sites"
)]
pub struct CleanClass;

// No #[presents(CleanClass)] anywhere — no presentations at all.
pub fn safe_fn() -> u32 {
    42
}
"#;

    std::fs::write(src_dir.join("lib.rs"), lib).unwrap();
    (tmp, src_dir)
}

// ATK-STRICT-1: without --strict, an unaddressed explicit presents-site exits 0.
//
// `audit` without `--strict` is informational. The exit code must be 0 regardless
// of unaddressed counts — the gate only activates when the user opts into `--strict`.
// This is the baseline control: audit never auto-fails without `--strict`.
#[test]
fn atk_strict_no_flag_unaddressed_exits_zero() {
    let (_tmp, root) = staged_unaddressed_explicit();
    let code = run_audit(&root, /*strict=*/ false);
    assert_eq!(
        code, 0,
        "ATK-STRICT-1: `cargo antigen audit` without --strict must exit 0 even \
        when explicit unaddressed presentations exist. The gate is opt-in via \
        --strict; informational output must never fail CI silently. Got exit {code}"
    );
}

// ATK-STRICT-2: with --strict, an unaddressed explicit presents-site exits 1.
//
// When the user opts in with `--strict`, an unaddressed `#[presents(X)]`
// site (ExplicitMarker match_kind) with no `#[defended_by(X)]` witness
// must cause exit 1. This is the core CI gate contract.
#[test]
fn atk_strict_flag_unaddressed_explicit_exits_one() {
    let (_tmp, root) = staged_unaddressed_explicit();
    let code = run_audit(&root, /*strict=*/ true);
    assert_eq!(
        code, 1,
        "ATK-STRICT-2: `cargo antigen audit --strict` must exit 1 when \
        unaddressed explicit presents-sites exist. The staged workspace has \
        one #[presents(UnaddressedClass)] with no #[defended_by(UnaddressedClass)]. \
        Got exit {code}"
    );
}

// ATK-STRICT-3: --strict does NOT exit 1 for fingerprint-match-only unaddressed sites.
//
// FingerprintMatch presentations (inferred, not explicitly declared by the author)
// are advisory noise — they require human triage to decide if the site is truly
// vulnerable. Gating on them under --strict creates a CI false-positive: the tool
// exits 1 while the human-readable output says "All explicit presentations addressed"
// — an undiagnosable CI failure for first-time users.
//
// The invariant: `--strict` gates ONLY on ExplicitMarker match_kind.
// FingerprintMatch sites must not trigger exit 1 regardless of their count.
//
// DEGENERATE INPUT: a workspace with only fingerprint-match presentations (no
// explicit #[presents] anywhere). Exit must be 0 under --strict.
#[test]
fn atk_strict_flag_fingerprint_match_only_exits_zero() {
    let (_tmp, root) = staged_fingerprint_match_only();
    let code = run_audit(&root, /*strict=*/ true);
    assert_eq!(
        code, 0,
        "ATK-STRICT-3: `cargo antigen audit --strict` must exit 0 when the only \
        unaddressed presentations are FingerprintMatch (inferred). Fingerprint \
        matches are advisory noise, not CI-gatable defects. Gating on them creates \
        a false-positive: CI fails while the output says 'All explicit presentations \
        addressed.' Got exit {code}. If this fails with exit 1, the --strict filter \
        on ExplicitMarker at cargo-antigen/src/main.rs has been removed or widened."
    );
}

// ATK-STRICT-4: --strict exits 0 for a clean workspace (no presentations at all).
//
// Control case: when audit finds no presentations (no #[presents] and no fingerprint
// matches), --strict must exit 0. An empty-presentations workspace is always clean.
#[test]
fn atk_strict_flag_no_presentations_exits_zero() {
    let (_tmp, root) = staged_clean();
    let code = run_audit(&root, /*strict=*/ true);
    assert_eq!(
        code, 0,
        "ATK-STRICT-4: `cargo antigen audit --strict` must exit 0 for a workspace \
        with no presentations at all. Got exit {code}"
    );
}

// ATK-STRICT-5: --strict exits 1 when orphaned tolerances exist.
//
// An `#[antigen_tolerance(Foo, rationale = "...")]` is "orphaned" when `Foo`
// is not declared as an antigen in the scanned workspace. Orphaned tolerances
// indicate a structural inconsistency — the tolerance references a failure-class
// that doesn't exist (or is no longer declared in scope).
//
// IMPORTANT: a cross-dep antigen tolerance (tolerating a failure-class that lives
// in a dependency, not the workspace itself) will ALWAYS appear orphaned in a
// non-`--include-deps` scan. This means --strict can produce CI false-positives
// for any adopter who:
//   1. Takes on a dep with antigens (via #[antigen(...)]) AND
//   2. Declares a tolerance for that dep's antigen AND
//   3. Runs `cargo antigen audit --strict` WITHOUT --include-deps
//
// The adopter has a VALID tolerance, but --strict exits 1 because the dep's
// antigen is not visible without --include-deps. This is a silent CI failure
// mode: the output says "orphaned tolerance" and the user has to figure out
// they need --include-deps.
//
// This test exercises the basic orphaned-tolerance → exit 1 path. It does NOT
// yet exercise the cross-dep false-positive case (that would require a multi-crate
// workspace fixture, which is out of scope for this test file).
//
// If --strict should NOT gate on orphaned tolerances (they're less severe than
// unaddressed presentations), the assertion sense should be inverted and the
// main.rs `--strict` condition updated.
#[test]
fn atk_strict_flag_orphaned_tolerance_exits_one() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let src_dir = tmp.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();

    // A tolerance for an antigen that is NOT declared in this workspace.
    // scan.rs will find the tolerance but no matching antigen_type == "NonexistentClass".
    // orphaned_tolerances() will return it. --strict must exit 1.
    let lib = r#"use antigen::antigen_tolerance;

/// A tolerance for a failure class that doesn't exist in this workspace.
/// This represents an orphaned tolerance — the referenced antigen was
/// removed or renamed, leaving this declaration dangling.
#[antigen_tolerance(
    NonexistentClass,
    rationale = "deliberate: tolerating a class that no longer exists"
)]
pub fn tolerating_fn() -> u32 {
    42
}
"#;

    std::fs::write(src_dir.join("lib.rs"), lib).unwrap();

    let code = run_audit(&src_dir, /*strict=*/ true);
    assert_eq!(
        code, 1,
        "ATK-STRICT-5: `cargo antigen audit --strict` must exit 1 when orphaned \
        tolerances exist. A tolerance for 'NonexistentClass' is orphaned because \
        no antigen with that type_name exists in the scanned workspace. \
        Got exit {code}"
    );
}
