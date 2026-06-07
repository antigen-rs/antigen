//! E0 — the `--bundled-catalog` CLI contract, end-to-end through the compiled
//! binary. THE PARTIAL-ADOPTER SILENT-MISS REFUTATION (captain's ruling /
//! ADR-043 Amendment 2: an EXPLICIT `--bundled-catalog` ALWAYS injects).
//!
//! The library-level gate (`antigen/tests/e0_bundled_catalog_scan.rs`) proves
//! the `Always` mode catches a partial adopter's flagship footgun. THIS gate is
//! the harder refutation the captain asked for: does the FIX actually CLOSE the
//! silent-miss at the CLI dispatch — i.e. does `cargo antigen scan
//! --bundled-catalog` on a crate that ALREADY declares a local antigen surface
//! the bundled flagship footgun, rather than suppressing the catalog because the
//! crate is "not empty"? Documenting the rule in a comment is not closing it; the
//! binary actually emitting the match is.
//!
//! Observable surface: `--message-format json` (render B / flycheck), the
//! cargo/rustc line-protocol. A bundled-catalog match for the flagship
//! `get-unchecked-without-proof` appears as a `compiler-message` at
//! `level: "warning"` (never `"error"` — antigen does not compile-block; ADR-044
//! claim-scope).

use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cargo-antigen"))
}

/// The shared partial-adopter fixture: ONE local `#[antigen]` declaration AND a
/// real flagship `get_unchecked` footgun the local antigen does NOT cover.
fn partial_adopter_root() -> PathBuf {
    // cargo-antigen/tests → ../../antigen/tests/fixtures/...
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("antigen")
        .join("tests")
        .join("fixtures")
        .join("e0_partial_adopter_one_decl")
}

/// Run `cargo antigen scan` with the given extra args against `root`, returning
/// `(exit_code, stdout, stderr)`.
fn scan(root: &Path, extra: &[&str]) -> (i32, String, String) {
    let out = Command::new(bin())
        .arg("antigen")
        .arg("scan")
        .arg("--root")
        .arg(root)
        .args(extra)
        .output()
        .expect("failed to run cargo-antigen");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

const FLAGSHIP: &str = "get-unchecked-without-proof";

// ===========================================================================
// (1) THE REFUTATION: explicit --bundled-catalog on a PARTIAL adopter CLOSES the
//     silent-miss — the flagship footgun is emitted, NOT suppressed.
// ===========================================================================

#[test]
fn explicit_bundled_catalog_flag_catches_a_partial_adopters_flagship_footgun() {
    let (code, stdout, stderr) = scan(
        &partial_adopter_root(),
        &["--bundled-catalog", "--message-format", "json"],
    );

    assert_eq!(
        code, 0,
        "scan --bundled-catalog --message-format json should exit 0 (warnings, not \
         errors). stderr = {stderr}"
    );
    assert!(
        stdout.contains(FLAGSHIP),
        "EXPLICIT --bundled-catalog on a partial adopter (one local antigen) MUST \
         surface the bundled flagship `{FLAGSHIP}` footgun — the captain's ruling \
         (always-inject) closes the silent-miss. If this is absent, the catalog was \
         suppressed because the crate declares a local antigen — the exact \
         silent-miss E0 exists to kill. stdout = {stdout}"
    );
    // Claim-scope: the match is a WARNING (a fingerprint match to inspect), never
    // an ERROR (an audited verdict / compile-block). antigen does not ratify.
    assert!(
        stdout.contains("\"level\":\"warning\"") || stdout.contains("\"level\": \"warning\""),
        "the bundled match must be emitted at level=warning (claim-scope: a \
         fingerprint match, never an error/audited verdict). stdout = {stdout}"
    );
    assert!(
        !stdout.contains("\"level\":\"error\"") && !stdout.contains("\"level\": \"error\""),
        "no bundled match may be emitted at level=error — antigen does not \
         compile-block on an unaudited fingerprint match. stdout = {stdout}"
    );
}

// ===========================================================================
// (2) THE OTHER HALF — without the flag, a partial adopter that declares a local
//     antigen stays LOCAL-ONLY (auto-detect injects only on a zero-decl crate).
//     This pins the deliberate two-mode contract so a future change is conscious:
//     the flag means "always", the no-flag default means "auto-detect-when-empty".
// ===========================================================================

#[test]
fn no_flag_on_a_partial_adopter_stays_local_only() {
    let (code, stdout, stderr) = scan(&partial_adopter_root(), &["--message-format", "json"]);

    assert_eq!(code, 0, "plain scan should exit 0. stderr = {stderr}");
    assert!(
        !stdout.contains(FLAGSHIP),
        "WITHOUT --bundled-catalog, a crate that declares its own antigens stays \
         local-only (auto-detect injects the catalog only for a ZERO-declaration \
         newcomer). The flagship `{FLAGSHIP}` (which only the bundled catalog \
         covers) must NOT appear. If it does, the no-flag default is silently \
         always-injecting — the two-mode contract drifted. stdout = {stdout}"
    );
}

// ===========================================================================
// (3) THE FIRST-90-SECONDS ADOPTION GATE (briefing §1 STREAM-ADOPT). A fresh
//     newcomer runs the PLAIN `cargo antigen scan` (no flag) on their
//     zero-declaration crate. The conversion-or-bust invariant: they must get a
//     RECOGNIZED REAL FINDING (not a silent false all-clear), framed honestly as a
//     CANDIDATE (not a failure / audited verdict), and fast. This is the
//     zero-hits-cliff closed at the USER level — the whole point of E0.
// ===========================================================================

#[test]
fn fresh_newcomer_plain_scan_surfaces_a_real_candidate_not_a_false_all_clear() {
    let start = std::time::Instant::now();
    // The newcomer path: NO --bundled-catalog flag, a ZERO-declaration crate.
    let (code, stdout, stderr) = scan(&consumer_zero_decl_root(), &[]);
    let elapsed = start.elapsed();

    assert_eq!(code, 0, "a plain scan exits 0. stderr = {stderr}");

    // (a) A REAL finding is surfaced — the cliff is closed. A zero-decl crate that
    //     DOES present known footguns must NOT read as a silent all-clear.
    assert!(
        stdout.contains(FLAGSHIP) || stdout.contains("panic-in-drop"),
        "the plain newcomer scan must surface a recognized flagship footgun \
         (auto-detect injects the catalog on a zero-decl crate). A zero-decl crate \
         with real footguns reading as all-clear is the false all-clear E0 kills. \
         stdout = {stdout}"
    );

    // (b) HONEST claim-scope framing: the match reads as a CANDIDATE / fingerprint
    //     match, never an audited 'failure' / 'error' verdict.
    assert!(
        stdout.to_lowercase().contains("candidate")
            || stdout.to_lowercase().contains("fingerprint match"),
        "the surfaced finding must be framed as a CANDIDATE / fingerprint match \
         (claim-scope), not an audited verdict. stdout = {stdout}"
    );

    // (c) FAST — the conversion window. 90s is the spec ceiling; a real scan of a
    //     one-file crate is milliseconds. A generous 30s bound catches a
    //     catastrophic perf regression (e.g. the catalog re-parsing on every item)
    //     without flaking on a slow CI box.
    assert!(
        elapsed.as_secs() < 30,
        "the first-scan conversion window: a fresh scan must complete well under \
         the 90s spec ceiling (a one-file crate is milliseconds). took {elapsed:?} \
         — a regression this large means the catalog path is doing pathological \
         work per item"
    );
}

/// The shared zero-declaration consumer fixture (no local antigens, real footguns).
fn consumer_zero_decl_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("antigen")
        .join("tests")
        .join("fixtures")
        .join("e0_consumer_crate_zero_decls")
}
