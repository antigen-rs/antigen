//! Compile-error fixture harness (W2).
//!
//! Each `.rs` file under `tests/ui/` is expected to fail to compile. The
//! corresponding `.stderr` file captures the error message verbatim. trybuild
//! diffs the actual compiler output against the saved `.stderr`, so changes to
//! error wording surface as test failures.
//!
//! Why this matters: error-message quality is the named-observer-stratum
//! (ADR-008) UX of the macros. Errors are how a Rust developer first
//! encounters antigen's discipline. Trybuild fixtures lock the wording so
//! refactors don't silently regress messages — and so W4 (span-aware errors)
//! has a baseline to upgrade rather than a void.
//!
//! Regenerating snapshots after intentional message changes:
//!   `$env:TRYBUILD = "overwrite"; cargo test -p antigen-macros --test compile_fail`
//! On Linux/macOS:
//!   `TRYBUILD=overwrite cargo test -p antigen-macros --test compile_fail`
//!
//! Cross-cutting with W4 (span-aware errors): when W4 lands, several of these
//! fixtures will have refined `.stderr` files (errors anchored to the
//! offending token rather than `Span::call_site()`). Regenerate, review the
//! diff, commit alongside the W4 change. The fixture files themselves stay
//! the same — only the saved error wording moves.

#[test]
fn compile_fail_fixtures() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
    // ui_pass/ holds fixtures that MUST compile successfully — currently the
    // ADR-009 Amendment 1 verify-only case (`#[antigen(name = "x")]` with no
    // fingerprint). Without a `t.pass` harness these would be inert; this makes
    // the "missing fingerprint compiles" contract load-bearing (it replaced the
    // old ui/missing_fingerprint.rs compile_fail fixture).
    t.pass("tests/ui_pass/*.rs");
}
