//! Compile-state ATK harness — the unsound interleavings that MUST NOT compile.
//!
//! Each `.rs` under `tests/ui/` is asserted to FAIL to compile; its `.stderr` captures the error
//! verbatim (blessed once the type-state lands). Each under `tests/ui_pass/` MUST compile — these are
//! the negative controls proving the SOUND path is still allowed (a type-state that rejected
//! everything would be useless). Same harness as `antigen-macros/tests/compile_fail.rs`.
//!
//! ## The ATKs guarded here (the C3 "law encoded in types, not runtime" family)
//!   - ATK-FRAME-TORN-READ  — a torn read (publish while a read session is live) is a borrow-checker
//!     error (`&mut db` excluded while `&db` is held). `ui_pass`: a correct serialized session compiles.
//!   - ATK-FRAME-TIER-CAP   — a `source=syntactic` read cannot CONSTRUCT a presents-grade verdict
//!     (the `PresentsVerdict`-has-no-public-constructor type-state). `ui_pass`: `corroborate` mints it.
//!   - ATK-FRAME-NONIDEM    — a non-idempotent (counting/blast) semiring on a RAW graph is a compile
//!     error (the `CondensedGraph` type-state). ENGINE-epoch. `ui_pass`: the condensed path compiles.
//!
//! ## Born-red status (the deepest sense)
//! Until the type-states land, the `ui/*.rs` fixtures COMPILE (the cap isn't enforced yet) → trybuild
//! reports "expected `compile_fail` but it succeeded" → the harness is RED. That is correct born-red:
//! the compile-error guarantee does not yet exist. When the builder lands the type-state, the fixture
//! starts failing-to-compile as required, the harness goes GREEN, and the `.stderr` is blessed:
//!   `TRYBUILD=overwrite cargo test -p antigen-stroma --test compile_fail`   (PowerShell: `$env:TRYBUILD="overwrite"`)
//!
//! ## ENGINE-epoch fixtures are gated
//! ATK-FRAME-NONIDEM is engine-epoch. Its fixtures live under `tests/ui_engine/` and are NOT run by
//! the frame-epoch harness below (they reference `CondensedGraph`, an engine-fill type). The engine
//! wave flips them on. The fixture + this note are the born-red PLACEHOLDER (recorded forward, the way
//! ADR-067 names deferred defenses rather than minting dangling ones).

#[test]
fn compile_fail_fixtures() {
    let t = trybuild::TestCases::new();
    // The unsound interleavings — MUST NOT compile (frame epoch: torn-read, tier-cap).
    t.compile_fail("tests/ui/*.rs");
    // The sound paths — MUST compile (the negative controls: a correct session, a corroborated mint).
    t.pass("tests/ui_pass/*.rs");
    // NOTE: tests/ui_engine/*.rs (ATK-FRAME-NONIDEM) is intentionally NOT wired here — engine epoch.
}
