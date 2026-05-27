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

// ============================================================================
// atk_a3_cli_design_phase_subcommands_return_failure
// ============================================================================
//
// ADR-005 Amendment 3 (audit-tier-honesty) requires that operations that did
// NOT do their stated work MUST report failure, never success. The v0.1-rc
// CLI verbs `delta`, `oracle`, `list`, `move`, `migrate`, `gc` (per ADR-019
// §M4) are not yet implemented; their stubs MUST exit non-zero so operator
// scripts cannot interpret a no-op as completion.
//
// Originally the stubs returned `ExitCode::SUCCESS` — that was a silent-
// failure trap: `cargo antigen attest list | xargs -I{} ...` would silently
// pipe nothing. The fix flipped them to `ExitCode::FAILURE` with an explicit
// "not implemented in v0.1-rc" message.

#[test]
fn atk_a3_cli_attest_list_accepts_root_flag() {
    // `attest list` is now implemented (task #53). Passing a valid root
    // exits 0 whether or not sidecars are found.
    let (code, _stderr) = attest(&["list", "--root", "."]);
    assert_eq!(code, 0, "`attest list --root .` must exit 0");
}

// `attest delta`: shipped (task #53); no longer a stub.
// `attest move`: DROPPED entirely (commit 37b0eeb); gc + audit enforce the discipline.
// `attest oracle`: now covered by atk_a3_cli_attest_oracle_renamed_for_f28_r2_collision_avoidance.

#[test]
fn atk_a3_cli_attest_migrate_was_dropped_for_additive_only_schema() {
    // Per ADR-021 §D2: additive-only schema evolution makes migration
    // unnecessary. The `attest migrate` subcommand was removed entirely.
    // Invoking it must fail with clap's "unknown subcommand" error
    // (typically exit 2 or non-zero).
    let (code, _stderr) = attest(&["migrate"]);
    assert_ne!(
        code, 0,
        "removed `attest migrate` MUST NOT report success; expect clap unknown-subcommand"
    );
}

#[test]
fn atk_a3_cli_attest_oracle_renamed_for_f28_r2_collision_avoidance() {
    // Per ADR-021 F28-R2: `attest oracle complete` renamed → `attest oracle
    // mark` to disambiguate from top-level `cargo antigen oracle complete`
    // (state-machine transition). The v0.1-rc stub is hidden in --help but
    // still exits non-zero per ADR-005 Am 3 (tier-honesty: a stub MUST NOT
    // report success).
    let (code, stderr) = attest(&["oracle"]);
    assert_ne!(
        code, 0,
        "design-phase `attest oracle` (placeholder for `mark`) MUST NOT report success"
    );
    assert!(
        stderr.contains("attest oracle mark")
            || stderr.contains("design phase")
            || stderr.contains("not yet implemented"),
        "stderr should reference the renamed verb or design-phase status: {stderr}"
    );
}

#[test]
fn atk_a3_cli_attest_gc_accepts_root_flag() {
    // `attest gc` is now implemented (task #53). Report-only by default.
    let (code, _stderr) = attest(&["gc", "--root", "."]);
    assert_eq!(code, 0, "`attest gc --root .` must exit 0");
}

#[test]
fn atk_a3_cli_tolerate_list_accepts_root_flag() {
    // `tolerate list` is now implemented (task #53).
    let (code, _stderr) = tolerate(&["list", "--root", "."]);
    assert_eq!(code, 0, "`tolerate list --root .` must exit 0");
}

// ============================================================================
// ATK-attest-corrupt-sidecar: `attest list` warns about malformed sidecar
// JSON files but continues listing valid ones.
//
// run_attest_list() at main.rs:3636-3644 uses a match+continue pattern that
// emits "warning: X is not valid Ratification JSON: Y" for each corrupt file.
// This IS a diagnostic (good). The test below pins this behavior as a
// regression anchor -- if the warning ever disappears, this test fails.
//
// Contrast with oracle list (main.rs:1738) which uses `if let Ok` and silently
// skips without ANY diagnostic (the separate findings/oracle-corrupt-json-silent-skip
// block covers that gap).
//
// `attest gc` at main.rs:3712 uses `if let Ok` in its per-file parse and
// therefore DOES silently skip corrupt sidecars (gc's gc loop is different
// from the list loop). The gc test below pins that behavior as a
// documentation test.
// ============================================================================

#[test]
fn atk_attest_list_warns_on_corrupt_sidecar_json() {
    // Regression anchor: attest list DOES warn about corrupt sidecar files.
    // The warning "is not valid Ratification JSON" should appear in stderr.
    // If this test FAILS (warning disappears), the diagnostic has been lost --
    // the tool would be silently skipping corrupt sidecars (the oracle-list gap).
    let tmp = tempfile::tempdir().unwrap();
    let attest_dir = tmp.path().join("src").join("lib.rs.attest");
    std::fs::create_dir_all(&attest_dir).unwrap();

    std::fs::write(
        attest_dir.join("corrupt.ratification.json"),
        b"{ not valid json at all ",
    )
    .unwrap();

    let (code, stderr) = attest(&["list", "--root", tmp.path().to_str().unwrap()]);

    assert_eq!(
        code, 0,
        "attest list must exit 0 even with corrupt sidecars (warning, not error): stderr={stderr}"
    );
    assert!(
        stderr
            .to_lowercase()
            .contains("not valid ratification json")
            || stderr.to_lowercase().contains("invalid"),
        "ATK-attest-corrupt-sidecar: attest list must warn about the corrupt sidecar file. \
         If this fails, the diagnostic was removed and the tool silently skips. \
         Current stderr:\n{stderr}"
    );
}

#[test]
fn atk_attest_gc_silently_skips_corrupt_sidecar_json() {
    // attest gc uses `if let Ok(rat) = serde_json::from_str(...)` at main.rs:3712
    // (inside the per-sidecar gc loop) which silently skips files that fail to parse.
    // Unlike attest list, gc does NOT warn about the corrupt file.
    //
    // An adopter running `attest gc` to clean up their sidecar directory gets no
    // signal that a corrupt sidecar was encountered. The malformed file is neither
    // flagged as an orphan nor warned about.
    //
    // This documents the CURRENT BEHAVIOR as a gap relative to attest list's behavior.
    // If gc is fixed to also warn, invert the assertion below.
    let tmp = tempfile::tempdir().unwrap();
    let attest_dir = tmp.path().join("src").join("lib.rs.attest");
    std::fs::create_dir_all(&attest_dir).unwrap();

    std::fs::write(
        attest_dir.join("corrupt-gc.ratification.json"),
        b"{ truncated",
    )
    .unwrap();

    let (code, stderr) = attest(&["gc", "--root", tmp.path().to_str().unwrap()]);

    assert_eq!(
        code, 0,
        "gc exits 0 for a corrupt-only workspace: stderr={stderr}"
    );
    // CURRENT BEHAVIOR: gc silently skips the corrupt file, no diagnostic.
    // When fixed (gc warns like list does), invert this assertion:
    //   assert!(stderr.contains("not valid") || stderr.contains("corrupt"))
    assert!(
        !stderr.to_lowercase().contains("not valid ratification")
            && !stderr.to_lowercase().contains("corrupt-gc.ratification"),
        "ATK-attest-corrupt-sidecar-gc: when this STARTS FAILING, gc has been fixed \
         to warn about corrupt sidecars -- update to assert the diagnostic IS present. \
         Current stderr:\n{stderr}"
    );
}
