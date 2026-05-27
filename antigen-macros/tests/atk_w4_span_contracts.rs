//! ATK-W4 span-aware error message contracts.
//!
//! W4 threaded token-precise spans through `AntigenArgs`/`ImmuneArgs` so
//! validation errors point at the offending literal — or, for missing-required-
//! field errors, at the macro's argument list (the closest meaningful anchor
//! when there is no offending token).
//!
//! These tests verify the trybuild .stderr fixtures encode the W4 span
//! discipline. Each contract names a specific column the diagnostic must
//! anchor at; reading the fixture's text (rather than re-running trybuild)
//! keeps the assertion fast and makes the failure message a useful diff.
//!
//! Adversarial finding (preserved from pre-implementation contracts): the
//! `unknown_antigen_field` fixture had column-precise spans before W4 because
//! `MetaPair` already used `new_spanned`. ATK-W4-005 guards against W4's
//! refactor regressing that.

use std::fs;
use std::path::Path;

/// Read a .stderr fixture and return its contents.
fn read_stderr(name: &str) -> String {
    let path = format!("tests/ui/{name}.stderr");
    fs::read_to_string(Path::new(&path))
        .unwrap_or_else(|e| panic!("could not read fixture {path}: {e}"))
}

/// Assert the .stderr contains a `--> path:LINE:COL` anchor at the given line
/// and column. The fixture's relative path is `tests/ui/<name>.rs`.
fn assert_anchor(stderr: &str, name: &str, line: u32, col: u32) {
    let needle = format!("--> tests/ui/{name}.rs:{line}:{col}");
    assert!(
        stderr.contains(&needle),
        "expected `{needle}` in {name}.stderr; got:\n{stderr}",
    );
}

// ============================================================================
// ATK-W4-001: empty_name spans the empty string literal "" (col 18)
// ============================================================================

#[test]
fn atk_w4_001_empty_name_span_points_at_empty_literal() {
    let stderr = read_stderr("empty_name");
    assert_anchor(&stderr, "empty_name", 7, 18);
    assert!(
        stderr.contains("^^"),
        "expected `^^` caret under empty `\"\"`; got:\n{stderr}",
    );
}

// ============================================================================
// ATK-W4-002: non_kebab_case_name spans the offending name literal "FooBar"
// ============================================================================

#[test]
fn atk_w4_002_kebab_case_error_spans_offending_literal() {
    let stderr = read_stderr("non_kebab_case_name");
    assert_anchor(&stderr, "non_kebab_case_name", 8, 18);
    assert!(
        stderr.contains("^^^^^^^^"),
        "expected 8-char caret under `\"FooBar\"`; got:\n{stderr}",
    );
}

// ============================================================================
// ATK-W4-003: REMOVED — missing_fingerprint is no longer a compile error
//
// ADR-009 Amendment 1 (ratified 2026-05-27) makes `fingerprint` optional:
// `#[antigen(name = "x")]` without a fingerprint is now a verify-only antigen
// (no syn-scannable source surface; detection is via `requires=`/witness only).
// The compile-fail fixture (tests/ui/missing_fingerprint.rs) and this test were
// removed because the behavior they pinned no longer exists.
//
// The W4 args_span discipline (anchor at first token of arg list, not call_site)
// is still enforced by ATK-W4-001 (empty_name) which exercises the same span
// path via a different required-field validation.
// ============================================================================

// ============================================================================
// ATK-W4-004: immune_without_witness spans the antigen path
// ============================================================================

#[test]
fn atk_w4_004_immune_without_witness_spans_antigen_path() {
    let stderr = read_stderr("immune_without_witness");
    // Anchor at column 10 — start of `DummyAntigen` inside `#[immune(...)]`.
    assert_anchor(&stderr, "immune_without_witness", 10, 10);
    assert!(
        stderr.contains("^^^^^^^^^^^^"),
        "expected 12-char caret under `DummyAntigen`; got:\n{stderr}",
    );
}

// ============================================================================
// ATK-W4-005: unknown_antigen_field span must not regress
//
// Pre-W4 baseline: `MetaPair` used `new_spanned` on the unknown ident, so
// this fixture already had a column-precise span (col 42, under `bogus`).
// W4 must preserve this — refactoring spans elsewhere must not silently
// downgrade this case to call_site.
// ============================================================================

#[test]
fn atk_w4_005_unknown_field_span_must_not_regress() {
    let stderr = read_stderr("unknown_antigen_field");
    assert_anchor(&stderr, "unknown_antigen_field", 9, 42);
    assert!(
        stderr.contains("^^^^^"),
        "expected 5-char caret under `bogus`; got:\n{stderr}",
    );
    assert!(
        !stderr.contains("unknown_antigen_field.rs:9:1\n"),
        "unknown_antigen_field must not regress to call_site span; got:\n{stderr}",
    );
}

// ============================================================================
// ATK-W4-006: span discipline is *uniform* — none of the W4-affected fixtures
// regress to call_site.
//
// The "consistency" check from the pre-implementation contracts: having some
// spans precise and others at call_site is a UX regression even if the
// precise ones are improvements in isolation. After W4, NO fixture in the
// W4-affected set may show a column-1 anchor.
// ============================================================================

#[test]
fn atk_w4_006_no_w4_fixture_uses_call_site_span() {
    // Note: missing_fingerprint was removed from this set when ADR-009
    // Amendment 1 made fingerprint optional — it is no longer a compile error.
    let fixtures = [
        ("empty_name", 7),
        ("non_kebab_case_name", 8),
        ("immune_without_witness", 10),
        ("unknown_antigen_field", 9),
    ];
    for (name, line) in fixtures {
        let stderr = read_stderr(name);
        let call_site_anchor = format!("--> tests/ui/{name}.rs:{line}:1\n");
        assert!(
            !stderr.contains(&call_site_anchor),
            "{name}.stderr regressed to call_site (col 1); got:\n{stderr}",
        );
    }
}
