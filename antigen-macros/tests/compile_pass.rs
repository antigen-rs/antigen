//! Compile-success fixture harness.
//!
//! Each `.rs` file under `tests/ui_pass/` is expected to compile cleanly. Used
//! for DX guarantees that are about *absence of errors/warnings* rather than
//! error wording — e.g. DX finding 1: a `#[antigen]` marker struct must not
//! trip `dead_code` in a binary-like crate under `#![deny(dead_code)]`.
//!
//! Kept separate from `compile_fail.rs` so the `tests/ui/*.rs` glob (all
//! expected-to-fail) and the `tests/ui_pass/*.rs` glob (all expected-to-pass)
//! never overlap.

#[test]
fn compile_pass_fixtures() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui_pass/*.rs");
}
