//! `cargo antigen verify dep-pin --write` — the in-place, format-preserving
//! Cargo.toml rewrite (`infra/verify-dep-pin-in-place-rewrite`,
//! v03-vision-buildout).
//!
//! The mutation half of verify dep-pin: pins each unpinned dep to its resolved
//! `=<version>` from Cargo.lock, IN PLACE, via `toml_edit` (comments + layout +
//! sibling keys preserved). `--write` is opt-in by design (ADR-017-Amd1 posture:
//! gate mutation-safety — rewriting the adopter's manifest is never the default).
//!
//! These tests build a hermetic Cargo.toml + Cargo.lock in a tempdir and assert:
//!   1. string-form + inline-table-form deps both pin to `=<version>`;
//!   2. comments and sibling keys (features, default-features) survive;
//!   3. an already-pinned dep + a path/workspace dep are left untouched;
//!   4. a dep with no Cargo.lock entry is NOT guessed (left untouched + reported);
//!   5. WITHOUT --write, the manifest is not mutated (suggestion-only default).

use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cargo-antigen"))
}

/// Run `cargo-antigen antigen verify dep-pin <args>` in `dir`; return
/// `(exit_code, stdout)`.
fn dep_pin(dir: &Path, args: &[&str]) -> (i32, String) {
    let out = Command::new(bin())
        .arg("antigen")
        .arg("verify")
        .arg("dep-pin")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("failed to run cargo-antigen");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

/// A Cargo.toml with a mix of dep forms; `serde` (string) + `clap` (inline
/// table with features) are unpinned, `chrono` is already pinned, `local` is a
/// path dep, `ghost` is unpinned but absent from the lockfile.
const MANIFEST: &str = r#"[package]
name = "fixture"
version = "0.1.0"
edition = "2021"

[dependencies]
# a string-form dependency (caret requirement — unpinned)
serde = "1.0"
# an inline-table dependency with sibling keys that must survive
clap = { version = "4.5", features = ["derive"], default-features = false }
# already exact-pinned — must NOT change
chrono = "=0.4.38"
# a path dependency — no version, must NOT change
local = { path = "../local" }
# unpinned but absent from Cargo.lock — must NOT be guessed
ghost = "9.9"
"#;

/// A Cargo.lock resolving serde + clap (but NOT ghost).
const LOCKFILE: &str = r#"version = 4

[[package]]
name = "serde"
version = "1.0.219"

[[package]]
name = "clap"
version = "4.5.51"
"#;

fn write_fixture(dir: &Path) {
    std::fs::write(dir.join("Cargo.toml"), MANIFEST).unwrap();
    std::fs::write(dir.join("Cargo.lock"), LOCKFILE).unwrap();
}

#[test]
fn write_pins_string_and_inline_table_deps_preserving_format() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    write_fixture(dir.path());

    let (code, stdout) = dep_pin(dir.path(), &["--write"]);
    assert_eq!(code, 0, "verify dep-pin --write must exit 0: {stdout}");

    let rewritten = std::fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();

    // 1. String-form serde pinned to its resolved version.
    assert!(
        rewritten.contains(r#"serde = "=1.0.219""#),
        "serde must be pinned to its resolved lockfile version: {rewritten}"
    );
    // 2. Inline-table clap: version pinned, sibling keys preserved.
    assert!(
        rewritten.contains(r#"version = "=4.5.51""#),
        "clap's version key must be pinned to its resolved version: {rewritten}"
    );
    assert!(
        rewritten.contains(r#"features = ["derive"]"#)
            && rewritten.contains("default-features = false"),
        "clap's sibling keys (features, default-features) must survive the rewrite: {rewritten}"
    );
    // 3. Comments survive (format-preserving — the whole point of toml_edit).
    assert!(
        rewritten.contains("# a string-form dependency")
            && rewritten.contains("# an inline-table dependency"),
        "comments must be preserved by the format-preserving rewrite: {rewritten}"
    );
    // 4. Already-pinned chrono + path dep `local` untouched.
    assert!(
        rewritten.contains(r#"chrono = "=0.4.38""#),
        "already-pinned chrono must be unchanged: {rewritten}"
    );
    assert!(
        rewritten.contains(r#"local = { path = "../local" }"#),
        "path dep `local` (no version) must be unchanged: {rewritten}"
    );
    // 5. ghost has no lockfile entry → NOT guessed, left as the caret form.
    assert!(
        rewritten.contains(r#"ghost = "9.9""#),
        "ghost (no Cargo.lock entry) must NOT be guessed/pinned: {rewritten}"
    );
    assert!(
        stdout.contains("ghost"),
        "the unresolved ghost dep must be reported, not silently dropped: {stdout}"
    );
}

#[test]
fn without_write_the_manifest_is_not_mutated() {
    // The suggestion-only default: no --write ⇒ Cargo.toml is byte-identical.
    let dir = tempfile::TempDir::new().expect("tempdir");
    write_fixture(dir.path());
    let before = std::fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();

    let (code, stdout) = dep_pin(dir.path(), &[]);
    assert_eq!(code, 0);

    let after = std::fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
    assert_eq!(
        before, after,
        "without --write the manifest must be untouched (suggestion-only default)"
    );
    assert!(
        stdout.contains("--write") && stdout.contains("opt-in"),
        "the suggestion output must point at --write as the opt-in mutation path: {stdout}"
    );
}

#[test]
fn write_is_idempotent_on_an_already_pinned_manifest() {
    // After one --write, every resolvable dep is pinned; a second --write is a
    // no-op (AllPinned) — except ghost which has no lockfile entry and stays a
    // caret form (so the manifest still has one unpinned dep, but it is never
    // guessed). Asserts the rewrite converges and does not thrash.
    let dir = tempfile::TempDir::new().expect("tempdir");
    write_fixture(dir.path());

    let (c1, _) = dep_pin(dir.path(), &["--write"]);
    assert_eq!(c1, 0);
    let after_first = std::fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();

    let (c2, _) = dep_pin(dir.path(), &["--write"]);
    assert_eq!(c2, 0);
    let after_second = std::fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();

    assert_eq!(
        after_first, after_second,
        "a second --write on an already-pinned manifest must be a no-op (idempotent)"
    );
}
