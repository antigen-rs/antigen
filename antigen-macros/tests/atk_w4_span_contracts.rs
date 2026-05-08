//! ATK-W4 pre-implementation contracts for span-aware error messages.
//!
//! W4 threads token-precise spans through `AntigenArgs`/`ImmuneArgs` `validate()`
//! so error squiggles underline the offending literal, not the whole macro
//! invocation. These tests document what must change.
//!
//! All tests here are `#[ignore]` — they assert against future .stderr content
//! that doesn't exist yet. W4 will regenerate the trybuild fixtures; at that
//! point remove `#[ignore]`, verify each test passes, and the new snapshots are
//! the evidence of correct span threading.
//!
//! The trybuild fixtures in tests/ui/ are the primary W4 test surface.
//! These tests are meta-level contracts on what THOSE fixtures must look like
//! after W4 lands.
//!
//! Key adversarial finding: the `unknown_antigen_field` fixture ALREADY has a
//! precise span (`MetaPair` uses `new_spanned`). W4 must match this quality for
//! the other five fixtures. Inconsistency between them (some precise, some
//! call-site) is a UX regression even if technically "better than nothing."

// ============================================================================
// ATK-W4-001: empty_name error must span the empty string literal ""
//
// Current: spans the whole #[antigen(...)] invocation
// Expected after W4: spans the "" token specifically
//
// The fix requires AntigenArgs to store the Span of the name LitStr during
// parsing, then pass it to validate(). The empty string "" is a real token
// with a real span — it's not call_site.
// ============================================================================

#[test]
#[ignore = "W4 pre-implementation contract — remove when W4 regenerates .stderr fixtures"]
fn atk_w4_001_empty_name_span_points_at_empty_literal() {
    // Contract: after W4, tests/ui/empty_name.stderr must contain:
    //   --> tests/ui/empty_name.rs:7:18  (column 18 is where "" starts)
    //   |
    // 7 | #[antigen(name = "", fingerprint = "x")]
    //   |                  ^^
    //
    // Current (call-site): column 1, underlines the whole macro invocation.
    //
    // Verification: `cat antigen-macros/tests/ui/empty_name.stderr | grep "^7"`
    // should show "7 | #[antigen(name = \"\", ...)" with "^^ " under the "".
    //
    // This test is a documentation anchor. The actual verification is the
    // trybuild fixture regeneration under W4.
    panic!("W4 pre-implementation contract — verify against regenerated .stderr");
}

// ============================================================================
// ATK-W4-002: non_kebab_case_name must span the offending name literal
//
// Current: spans the whole macro invocation
// Expected after W4: spans "FooBar" specifically
// ============================================================================

#[test]
#[ignore = "W4 pre-implementation contract — remove when W4 regenerates .stderr fixtures"]
fn atk_w4_002_kebab_case_error_spans_offending_literal() {
    // Contract: after W4, tests/ui/non_kebab_case_name.stderr must contain:
    //   --> tests/ui/non_kebab_case_name.rs:8:19  (column of "FooBar")
    //   |
    // 8 | #[antigen(name = "FooBar", fingerprint = "x")]
    //   |                   ^^^^^^^
    panic!("W4 pre-implementation contract");
}

// ============================================================================
// ATK-W4-003: missing_fingerprint — the HARD span case
//
// A missing required field has no offending token. The span must point at
// SOMETHING meaningful. Options:
//   (a) The input span (the last token in the arg list) — points near where
//       the missing field should have been
//   (b) The opening paren of the macro — shows "this invocation is incomplete"
//   (c) Keep call_site for this case only — consistent with the "no token"
//       reality, but UX regression vs the others being fixed
//
// The adversarial concern: if W4 fixes empty_name and non_kebab_case but
// leaves missing_fingerprint at call_site, the UX is inconsistent. A user
// comparing two error messages gets different quality spans for similar errors.
//
// Contract: W4 must choose one of (a) or (b) for missing_fingerprint and
// apply it consistently for ALL missing-required-field errors. The choice
// must be documented in a comment in parse.rs.
// ============================================================================

#[test]
#[ignore = "W4 pre-implementation contract — the missing-required-field span case needs an explicit design decision"]
fn atk_w4_003_missing_fingerprint_has_consistent_span_strategy() {
    // Contract: after W4, tests/ui/missing_fingerprint.stderr must NOT use
    // call_site (column 1, full invocation span). It must use either:
    //   (a) The span of the last argument (pointing near the gap), OR
    //   (b) The span of the opening parenthesis (pointing at the whole arglist)
    //
    // The choice between (a) and (b) is a W4 design decision. Either is
    // better than call_site. The decision must be consistent: if (a) is chosen
    // for missing_fingerprint, it must also be used for missing_name (if that
    // fixture existed), and vice versa.
    //
    // ATK: pathmaker may take the easy path and leave this at call_site while
    // fixing the "has a token" cases. That's the adversarial failure mode —
    // partially-threaded spans with inconsistent quality.
    panic!("W4 pre-implementation contract — design decision required");
}

// ============================================================================
// ATK-W4-004: immune_without_witness span
//
// Current: spans the whole #[immune(DummyAntigen)] invocation
// Expected: spans DummyAntigen (the antigen path) since that's the most
// useful location — "this declaration needs a witness"
//
// Alternative: span the closing paren, indicating "expected witness = ... here"
// ============================================================================

#[test]
#[ignore = "W4 pre-implementation contract — remove when W4 regenerates .stderr fixtures"]
fn atk_w4_004_immune_without_witness_spans_antigen_path() {
    // Contract: after W4, tests/ui/immune_without_witness.stderr must contain
    // a span pointing at `DummyAntigen` or the closing `)`, not the whole
    // #[immune(...)] invocation.
    //
    // The antigen path span is stored during ImmuneArgs parsing as
    // `antigen: Path`. Its span is `antigen.span()`. This is the natural
    // token to point at — "this antigen presentation claims immunity but
    // has no proof."
    panic!("W4 pre-implementation contract");
}

// ============================================================================
// ATK-W4-005: unknown_antigen_field already has precise span — regression guard
//
// This fixture ALREADY has a good span (MetaPair uses new_spanned).
// After W4 regenerates snapshots, this fixture must STILL have the precise
// span — W4 must not regress it to call_site while threading spans elsewhere.
// ============================================================================

#[test]
#[ignore = "W4 regression guard — verify after W4 that unknown_antigen_field still shows column-precise span for `bogus`"]
fn atk_w4_005_unknown_field_span_must_not_regress() {
    // Contract: after W4, tests/ui/unknown_antigen_field.stderr must still
    // show the span at `bogus` (column 42), not at column 1.
    //
    //  9 | #[antigen(name = "x", fingerprint = "y", bogus = "z")]
    //    |                                          ^^^^^
    //
    // This was already working before W4. If W4's span refactor accidentally
    // replaces new_spanned() calls with call_site() in the unknown-field path,
    // this guard catches the regression.
    panic!("W4 regression guard — verify against regenerated .stderr after W4");
}
