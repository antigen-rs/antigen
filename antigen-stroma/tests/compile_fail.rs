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
//!     error: `run_closure` is bounded `S: Idempotent` and `Blast` does not implement it, so
//!     `run_closure::<Blast>` does not type-check (`tests/ui/nonidem_run_closure_blast_forbidden.rs`).
//!     `ui_pass`: `run_closure::<Detection>` (the sound idempotent path) compiles.
//!   - ATK-SIGNAL-B2PRIME   — the signal-algebra tier-wall (ADR-071 B2′, `src/signal.rs`): an organ
//!     cannot write at ≤ the tier it reads (`write_back` yields `Up<R>`), and a signed `EfferentChannel`
//!     cannot enter the afferent write door (the channel-split). `ui_pass`: the sound tower-climb compiles.
//!
//! ## Born-red status (the deepest sense)
//! Until the type-states land, the `ui/*.rs` fixtures COMPILE (the cap isn't enforced yet) → trybuild
//! reports "expected `compile_fail` but it succeeded" → the harness is RED. That is correct born-red:
//! the compile-error guarantee does not yet exist. When the builder lands the type-state, the fixture
//! starts failing-to-compile as required, the harness goes GREEN, and the `.stderr` is blessed:
//!   `TRYBUILD=overwrite cargo test -p antigen-stroma --test compile_fail`   (PowerShell: `$env:TRYBUILD="overwrite"`)
//!
//! ## The engine epoch has landed
//! ATK-FRAME-NONIDEM was an engine-epoch placeholder under `tests/ui_engine/`; the read-engine now
//! exists, so the fixture is realized against the real API (`run_closure::<Blast>` — the `Idempotent`
//! marker gate) and lives under `tests/ui/`, wired below alongside the frame-epoch fixtures.

#[test]
fn compile_fail_fixtures() {
    let t = trybuild::TestCases::new();
    // The unsound interleavings — MUST NOT compile (torn-read, tier-cap, non-idempotent-on-raw-graph).
    t.compile_fail("tests/ui/*.rs");
    // The sound paths — MUST compile (the negative controls: correct session, corroborated mint,
    // idempotent-semiring closure).
    t.pass("tests/ui_pass/*.rs");
}
