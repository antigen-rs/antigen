//! `--fingerprint` digest-format guard across the attest verbs.
//!
//! The `--fingerprint` arg holds a structural DIGEST (`fnv1a64:` + 16 hex), not
//! the fingerprint DSL grammar. Before, `sign` only warned on an EMPTY
//! fingerprint; a present-but-malformed digest (a DSL string, an unprefixed
//! hash, a typo) was recorded verbatim as `signed_against_fingerprint` and then
//! silently never matched the item's real digest at audit — a dead-on-arrival
//! signature with no signal at sign time.
//!
//! The guard warns (does not hard-fail) on a non-empty value that isn't a valid
//! `fnv1a64:`-prefixed 16-hex digest, matching the empty-case posture and
//! tolerating a future digest scheme while making the dead signature loud.
//!
//! The guard is run at EVERY digest-accepting attest verb — sign, scaffold,
//! delta, check — closing the cross-site inconsistency that earned the
//! `FingerprintDigestWithoutFormatValidation` class (it was a 1-of-N spread:
//! only `sign` was guarded). These tests pin: malformed → warning, valid → no
//! warning, empty → warning (preserved), at the sign + scaffold + delta sites.
//! The verb still succeeds (exit 0) — the value is the operator's to commit; we
//! only make the consequence visible.

use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cargo-antigen"))
}

/// Stage a sidecar with one item carrying a valid stored digest, so the only
/// variable under test is the `--fingerprint` value passed to `sign`. Returns
/// `(tempdir, sidecar_path)`.
fn staged_sidecar() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let src = tmp.path().join("src");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(src.join("lib.rs"), "// placeholder").unwrap();

    let good = "fnv1a64:beecdcd530c6269c";
    let scaffold = Command::new(bin())
        .args([
            "antigen",
            "attest",
            "scaffold",
            "--antigen",
            "Demo",
            "--source-file",
            src.join("lib.rs").to_str().unwrap(),
            "--item-path",
            "foo",
            "--fingerprint",
            good,
        ])
        .output()
        .expect("scaffold");
    assert!(scaffold.status.success(), "scaffold must succeed");

    (tmp, src.join(".attest").join("Demo.json"))
}

fn sign_with_fingerprint(sidecar: &Path, fp: &str) -> (i32, String) {
    let out = Command::new(bin())
        .args([
            "antigen",
            "attest",
            "sign",
            "--sidecar",
            sidecar.to_str().unwrap(),
            "--item-path",
            "foo",
            "--signer",
            "alice",
            "--fingerprint",
            fp,
        ])
        .output()
        .expect("sign");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn sign_warns_on_malformed_digest() {
    let (_tmp, sidecar) = staged_sidecar();
    let (code, stderr) = sign_with_fingerprint(&sidecar, "garbage-not-a-digest");
    assert_eq!(
        code, 0,
        "sign still succeeds — the value is the operator's to commit"
    );
    assert!(
        stderr.contains("does not look like a structural digest"),
        "a malformed --fingerprint must warn that it can't match at audit: {stderr}"
    );
}

#[test]
fn sign_warns_on_unprefixed_hash() {
    let (_tmp, sidecar) = staged_sidecar();
    // 16 hex chars but missing the `fnv1a64:` version prefix — a plausible
    // copy-paste error that would silently never match.
    let (code, stderr) = sign_with_fingerprint(&sidecar, "beecdcd530c6269c");
    assert_eq!(code, 0);
    assert!(
        stderr.contains("does not look like a structural digest"),
        "an unprefixed hash must warn (the version prefix is load-bearing): {stderr}"
    );
}

#[test]
fn sign_does_not_warn_on_valid_digest() {
    let (_tmp, sidecar) = staged_sidecar();
    // Matches the sidecar's stored current_fingerprint, so neither the format
    // warning nor the stale-mismatch warning should fire.
    let (code, stderr) = sign_with_fingerprint(&sidecar, "fnv1a64:beecdcd530c6269c");
    assert_eq!(code, 0);
    assert!(
        !stderr.contains("does not look like a structural digest"),
        "a valid fnv1a64 digest must NOT trigger the format warning: {stderr}"
    );
}

#[test]
fn sign_warns_on_uppercase_hex() {
    let (_tmp, sidecar) = staged_sidecar();
    // Digests are lowercase-hex; uppercase is malformed (the producer emits
    // `{hash:016x}`). Catches a normalization slip that would never match.
    let (code, stderr) = sign_with_fingerprint(&sidecar, "fnv1a64:BEECDCD530C6269C");
    assert_eq!(code, 0);
    assert!(
        stderr.contains("does not look like a structural digest"),
        "uppercase hex must warn — digests are lowercase: {stderr}"
    );
}

// ============================================================================
// Cross-site: the same guard fires at scaffold and delta, not just sign.
// ============================================================================

#[test]
fn scaffold_warns_on_explicit_malformed_fingerprint() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(src.join("lib.rs"), "// placeholder").unwrap();

    // Explicit malformed --fingerprint (not auto-filled, so the guard applies).
    let out = Command::new(bin())
        .args([
            "antigen",
            "attest",
            "scaffold",
            "--antigen",
            "Demo",
            "--source-file",
            src.join("lib.rs").to_str().unwrap(),
            "--item-path",
            "foo",
            "--fingerprint",
            "garbage-not-a-digest",
        ])
        .output()
        .expect("scaffold");
    assert_eq!(out.status.code(), Some(0), "scaffold still succeeds");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("does not look like a structural digest"),
        "scaffold must warn on a malformed explicit --fingerprint (cross-site \
         consistency with sign): {stderr}"
    );
}

#[test]
fn scaffold_does_not_warn_on_valid_explicit_fingerprint() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(src.join("lib.rs"), "// placeholder").unwrap();

    let out = Command::new(bin())
        .args([
            "antigen",
            "attest",
            "scaffold",
            "--antigen",
            "Demo",
            "--source-file",
            src.join("lib.rs").to_str().unwrap(),
            "--item-path",
            "foo",
            "--fingerprint",
            "fnv1a64:beecdcd530c6269c",
        ])
        .output()
        .expect("scaffold");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("does not look like a structural digest"),
        "a valid digest must not trigger the scaffold format warning: {stderr}"
    );
}

#[test]
fn delta_warns_on_malformed_fingerprint() {
    // Stage a sidecar with a prior fresh signature so delta has a chain root.
    let (_tmp, sidecar) = staged_sidecar();
    let valid = "fnv1a64:beecdcd530c6269c";
    let sign = Command::new(bin())
        .args([
            "antigen",
            "attest",
            "sign",
            "--sidecar",
            sidecar.to_str().unwrap(),
            "--item-path",
            "foo",
            "--signer",
            "alice",
            "--fingerprint",
            valid,
        ])
        .output()
        .expect("seed sign");
    assert_eq!(sign.status.code(), Some(0), "seed sign must succeed");

    // Delta with a malformed new --fingerprint must warn.
    let out = Command::new(bin())
        .args([
            "antigen",
            "attest",
            "delta",
            "--sidecar",
            sidecar.to_str().unwrap(),
            "--item-path",
            "foo",
            "--signer",
            "alice",
            "--fingerprint",
            "garbage-not-a-digest",
            "--prior-fingerprint",
            valid,
            "--rationale",
            "carry-forward after a whitespace-only reformat of the item body",
        ])
        .output()
        .expect("delta");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("does not look like a structural digest"),
        "delta must warn on a malformed --fingerprint (cross-site consistency): {stderr}"
    );
}
