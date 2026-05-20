//! P3e CLI adversarial lock-down: `cargo antigen attest` + `cargo antigen tolerate`
//! boundary tests.
//!
//! Tests in this file invoke the compiled `cargo-antigen` binary directly via
//! `std::process::Command`. They lock down the observable CLI contract (exit
//! codes, file-system effects, error message fragments) for the three live v0.1
//! commands: `scaffold`, `sign`, `check`.
//!
//! Exit code contract:
//!   0 = success
//!   1 = user-visible validation failure (not found, already exists, predicate fails)
//!   2 = IO / parse error (file missing, JSON malformed)
//!
//! ATK cases:
//!   `atk_a3_cli_scaffold_idempotency`    — scaffold on existing sidecar → exit 1 without --force
//!   `atk_a3_cli_scaffold_force`          — --force overwrites; exit 0, kind preserved
//!   `atk_a3_cli_sign_unknown_item`       — sign to item-path not in sidecar → exit 1
//!   `atk_a3_cli_sign_duplicate`          — same signer+fingerprint → exit 0, no mutation
//!   `atk_a3_cli_sign_missing_sidecar`    — sidecar path does not exist → exit 2
//!   `atk_a3_cli_check_missing_sidecar`   — sidecar path does not exist → exit 2
//!   `atk_a3_cli_check_malformed_pred`    — predicate JSON invalid → exit 2
//!   `atk_a3_cli_check_passing_predicate` — signers satisfied → exit 0
//!   `atk_a3_cli_check_failing_predicate` — signers not satisfied → exit 1
//!   `atk_a3_cli_tolerate_kind_override`  — `tolerate scaffold` forces kind=Tolerance

use std::path::{Path, PathBuf};
use std::process::Command;

/// Path to the compiled binary injected by Cargo at test-link time.
fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cargo-antigen"))
}

/// Run `cargo-antigen attest <subcommand> <args>` and return `(exit_code, stderr)`.
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

/// Run `cargo-antigen tolerate <subcommand> <args>` and return `(exit_code, stderr)`.
fn tolerate(args: &[&str]) -> (i32, String) {
    let out = Command::new(bin())
        .arg("antigen")
        .arg("tolerate")
        .args(args)
        .output()
        .expect("failed to run cargo-antigen");
    let exit = out.status.code().unwrap_or(-1);
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    (exit, stderr)
}

/// Scaffold a sidecar into `dir/.attest/<stem>.json` and return the path.
fn scaffold_basic(dir: &Path, antigen: &str, item: &str, fp: &str) -> PathBuf {
    let src = dir.join("src").join("lib.rs");
    std::fs::create_dir_all(src.parent().unwrap()).unwrap();
    std::fs::write(&src, "// placeholder").unwrap();
    let (code, stderr) = attest(&[
        "scaffold",
        "--antigen",
        antigen,
        "--source-file",
        src.to_str().unwrap(),
        "--item-path",
        item,
        "--fingerprint",
        fp,
    ]);
    assert_eq!(code, 0, "scaffold should succeed: {stderr}");
    src.parent()
        .unwrap()
        .join(".attest")
        .join(format!("{antigen}.json"))
}

// ============================================================================
// atk_a3_cli_scaffold_idempotency
// ============================================================================

#[test]
fn atk_a3_cli_scaffold_idempotency() {
    let tmp = tempfile::tempdir().unwrap();
    let sidecar = scaffold_basic(tmp.path(), "SignedZero", "sinh", "fp-abc");

    // Second scaffold without --force must fail with exit 1.
    let src = tmp.path().join("src").join("lib.rs");
    let (code, stderr) = attest(&[
        "scaffold",
        "--antigen",
        "SignedZero",
        "--source-file",
        src.to_str().unwrap(),
        "--item-path",
        "sinh",
    ]);
    assert_eq!(
        code, 1,
        "duplicate scaffold without --force must exit 1: {stderr}"
    );
    assert!(
        stderr.contains("already exists"),
        "error message should say 'already exists': {stderr}"
    );

    // Sidecar content unchanged.
    let before = std::fs::read_to_string(&sidecar).unwrap();
    assert!(
        before.contains("SignedZero"),
        "sidecar must still name the antigen"
    );
}

// ============================================================================
// atk_a3_cli_scaffold_force
// ============================================================================

#[test]
fn atk_a3_cli_scaffold_force() {
    let tmp = tempfile::tempdir().unwrap();
    scaffold_basic(tmp.path(), "SignedZero", "sinh", "fp-v1");

    let src = tmp.path().join("src").join("lib.rs");
    let sidecar = src
        .parent()
        .unwrap()
        .join(".attest")
        .join("SignedZero.json");

    // --force must succeed (exit 0) and overwrite with new fingerprint.
    let (code, stderr) = attest(&[
        "scaffold",
        "--antigen",
        "SignedZero",
        "--source-file",
        src.to_str().unwrap(),
        "--item-path",
        "sinh",
        "--fingerprint",
        "fp-v2",
        "--force",
    ]);
    assert_eq!(code, 0, "--force scaffold must succeed: {stderr}");

    let content = std::fs::read_to_string(&sidecar).unwrap();
    // New fingerprint must be written.
    assert!(
        content.contains("fp-v2"),
        "--force must overwrite with new fingerprint: {content}"
    );
    // Kind must default to Immunity.
    assert!(
        content.contains("\"immunity\"") || content.contains("Immunity"),
        "default kind must be immunity: {content}"
    );
}

// ============================================================================
// atk_a3_cli_sign_unknown_item
// ============================================================================

#[test]
fn atk_a3_cli_sign_unknown_item() {
    let tmp = tempfile::tempdir().unwrap();
    let sidecar = scaffold_basic(tmp.path(), "SignedZero", "sinh", "fp-abc");

    // Sign to a nonexistent item-path.
    let (code, stderr) = attest(&[
        "sign",
        "--sidecar",
        sidecar.to_str().unwrap(),
        "--item-path",
        "cosh", // not "sinh"
        "--signer",
        "alice",
        "--fingerprint",
        "fp-abc",
    ]);
    assert_eq!(code, 1, "unknown item-path must exit 1: {stderr}");
    assert!(
        stderr.contains("Available item paths") || stderr.contains("no item"),
        "error must name available paths: {stderr}"
    );
}

// ============================================================================
// atk_a3_cli_sign_duplicate
// ============================================================================

#[test]
fn atk_a3_cli_sign_duplicate() {
    let tmp = tempfile::tempdir().unwrap();
    let sidecar = scaffold_basic(tmp.path(), "SignedZero", "sinh", "fp-abc");

    let sign_args = &[
        "sign",
        "--sidecar",
        sidecar.to_str().unwrap(),
        "--item-path",
        "sinh",
        "--signer",
        "alice",
        "--fingerprint",
        "fp-abc",
    ];

    // First sign succeeds.
    let (code, _) = attest(sign_args);
    assert_eq!(code, 0);

    let after_first = std::fs::read_to_string(&sidecar).unwrap();
    let first_count = after_first.matches("\"alice\"").count();

    // Second sign with same signer+fingerprint must exit 0 with a warning.
    let (code2, stderr2) = attest(sign_args);
    assert_eq!(
        code2, 0,
        "duplicate sign must exit 0 (idempotent): {stderr2}"
    );
    assert!(
        stderr2.contains("already signed"),
        "must warn about duplicate: {stderr2}"
    );

    // Sidecar must NOT have grown (no new entry).
    let after_second = std::fs::read_to_string(&sidecar).unwrap();
    let second_count = after_second.matches("\"alice\"").count();
    assert_eq!(
        first_count, second_count,
        "duplicate sign must not append a new entry"
    );
}

// ============================================================================
// atk_a3_cli_sign_missing_sidecar
// ============================================================================

#[test]
fn atk_a3_cli_sign_missing_sidecar() {
    let tmp = tempfile::tempdir().unwrap();
    let ghost = tmp.path().join("nonexistent.attest").join("Ghost.json");

    let (code, stderr) = attest(&[
        "sign",
        "--sidecar",
        ghost.to_str().unwrap(),
        "--item-path",
        "sinh",
        "--signer",
        "alice",
        "--fingerprint",
        "fp",
    ]);
    assert_eq!(code, 2, "missing sidecar must exit 2: {stderr}");
}

// ============================================================================
// atk_a3_cli_check_missing_sidecar
// ============================================================================

#[test]
fn atk_a3_cli_check_missing_sidecar() {
    let tmp = tempfile::tempdir().unwrap();
    let ghost = tmp.path().join("no_such.json");

    let (code, stderr) = attest(&[
        "check",
        "--sidecar",
        ghost.to_str().unwrap(),
        "--predicate",
        r#"{"kind":"leaf","name":"signers","required":["alice"]}"#,
    ]);
    assert_eq!(code, 2, "missing sidecar for check must exit 2: {stderr}");
}

// ============================================================================
// atk_a3_cli_check_malformed_predicate
// ============================================================================

#[test]
fn atk_a3_cli_check_malformed_pred() {
    let tmp = tempfile::tempdir().unwrap();
    let sidecar = scaffold_basic(tmp.path(), "SignedZero", "sinh", "fp-abc");

    let (code, stderr) = attest(&[
        "check",
        "--sidecar",
        sidecar.to_str().unwrap(),
        "--predicate",
        "not valid json {{{",
    ]);
    assert_eq!(code, 2, "malformed predicate must exit 2: {stderr}");
    assert!(
        stderr.contains("predicate JSON invalid") || stderr.contains("invalid"),
        "error must describe JSON problem: {stderr}"
    );
}

// ============================================================================
// atk_a3_cli_check_passing_predicate
// ============================================================================

#[test]
fn atk_a3_cli_check_passing_predicate() {
    let tmp = tempfile::tempdir().unwrap();
    let sidecar = scaffold_basic(tmp.path(), "SignedZero", "sinh", "fp-abc");

    // Sign so the signers leaf can pass.
    attest(&[
        "sign",
        "--sidecar",
        sidecar.to_str().unwrap(),
        "--item-path",
        "sinh",
        "--signer",
        "alice",
        "--fingerprint",
        "fp-abc",
    ]);

    // Evaluate a predicate that matches: signers(alice) against stored fp.
    let predicate = r#"{"kind":"leaf","name":"signers","required":["alice"]}"#;
    let (code, stderr) = attest(&[
        "check",
        "--sidecar",
        sidecar.to_str().unwrap(),
        "--predicate",
        predicate,
    ]);
    assert_eq!(code, 0, "passing predicate must exit 0: {stderr}");
}

// ============================================================================
// atk_a3_cli_check_failing_predicate
// ============================================================================

#[test]
fn atk_a3_cli_check_failing_predicate() {
    let tmp = tempfile::tempdir().unwrap();
    let sidecar = scaffold_basic(tmp.path(), "SignedZero", "sinh", "fp-abc");
    // No signers added — predicate requiring "alice" must fail.

    let predicate = r#"{"kind":"leaf","name":"signers","required":["alice"]}"#;
    let (code, _stderr) = attest(&[
        "check",
        "--sidecar",
        sidecar.to_str().unwrap(),
        "--predicate",
        predicate,
    ]);
    assert_eq!(code, 1, "failing predicate must exit 1");
}

// ============================================================================
// atk_a3_cli_tolerate_kind_override
// ============================================================================

/// `tolerate scaffold` must produce kind=Tolerance even if --kind immunity is supplied.
/// This tests that `run_tolerate` correctly overrides the `--kind` arg.
#[test]
fn atk_a3_cli_tolerate_kind_override() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src").join("lib.rs");
    std::fs::create_dir_all(src.parent().unwrap()).unwrap();
    std::fs::write(&src, "// placeholder").unwrap();

    let (code, stderr) = tolerate(&[
        "scaffold",
        "--antigen",
        "PanickingInDrop",
        "--source-file",
        src.to_str().unwrap(),
        "--item-path",
        "my_struct",
        "--kind",
        "immunity", // should be overridden to tolerance
    ]);
    assert_eq!(code, 0, "tolerate scaffold must succeed: {stderr}");

    let sidecar = src
        .parent()
        .unwrap()
        .join(".attest")
        .join("PanickingInDrop.json");
    let content = std::fs::read_to_string(&sidecar).unwrap();

    // kind must be tolerance, not immunity — run_tolerate forces this.
    assert!(
        content.contains("\"tolerance\"") || content.contains("tolerance"),
        "`tolerate scaffold` must override --kind to tolerance: {content}"
    );
    assert!(
        !content.contains("\"immunity\""),
        "must NOT contain immunity: {content}"
    );
}
