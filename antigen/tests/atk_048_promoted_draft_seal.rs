//! ATK-048 — the `PromotedDraft` capability-token seal (ADR-048 §Q9, the
//! LOAD-BEARING test).
//!
//! `PromotedDraft` (ADR-048) is antigen's own *parse-don't-validate* capability
//! token: the ONLY assertable generalization, mintable SOLELY by the B-gate
//! (`promote_if_safe` / `propose`). Possession of one is the structural proof that
//! all three of ADR-047's gate checks held. The whole discipline rests on the seal
//! being *unforgeable from outside the gate's module* — and that is a **compile-time
//! property**, not a runtime one, so a unit test cannot assert it. This trybuild
//! harness is the test-class that defends it: each `tests/ui_promoted_draft/*.rs`
//! fixture is a born-RED bypass attempt that **MUST FAIL to compile**, and its
//! `.stderr` snapshot pins the exact compiler refusal.
//!
//! # Why this is the load-bearing test of the keystone
//!
//! ADR-048 §Enforcement-Surface: "the newtype's privacy IS the enforcement." If a
//! caller could construct a `PromotedDraft` *without* running the gate — by the
//! tuple constructor, a `::new`, a `pub` field, a `From<Fingerprint>`, a `Default`,
//! or a serde `Deserialize` from hand-written JSON — the autoimmune draft the whole
//! GATE-G work (ADR-047) exists to refuse would be one ordinary expression away from
//! being asserted, and the type system would say nothing. Each fixture below is one
//! of those bypasses; the seal holds iff **every one fails to compile**.
//!
//! The five surfaces the capability-token must hold across ("attack the
//! boundary, not the type" — a guarantee stated for a type IN ISOLATION leaks at the
//! seam where it is reduced / serialized / embedded / re-axis-read):
//!
//! | surface       | fixture                                  | ADR        |
//! |---------------|------------------------------------------|------------|
//! | promotion     | tuple ctor / `::new` / `pub` field       | 048 §Q9    |
//! | construction  | `From<Fingerprint>` / `Default`          | 048 §Q2    |
//! | serde         | `Deserialize`-from-JSON forgery          | 048 §5     |
//! | accept        | (ADR-051 — born-red when the surface lands) | 051 §Q9 |
//! | persistence   | (ADR-051 `PersistedSpecimen` round-trip — runtime, in `atk_051`) | 051 §Q9 |
//!
//! # The negative control built IN (the teeth-check)
//!
//! A compile-fail suite that can't go GREEN-when-it-should is as much a liar as one
//! that can't go red. `tests/ui_promoted_draft_pass/` holds the *positive* control:
//! the gate's own public read-surface (`fingerprint()`, `tier()`, `as_ref()`,
//! `into_fingerprint()`) MUST compile — proving the fixtures fail because the
//! *constructor* is sealed, NOT because the type is unreachable or the import is
//! broken. Without the pass-control a typo'd import would make every compile-fail
//! "pass" for the wrong reason (the canonical trybuild false-green).
//!
//! # Regenerating the STABLE-blessed snapshots
//!
//! The `.stderr` files are stable-blessed (ADR: `release.yml` builds on stable; the
//! crate ships to stable users). Regenerate ONLY on stable, after an intentional
//! message change, and review the diff:
//!   PowerShell:  `$env:TRYBUILD = "overwrite"; cargo +stable test -p antigen --test atk_048_promoted_draft_seal`
//!   bash:        `TRYBUILD=overwrite cargo +stable test -p antigen --test atk_048_promoted_draft_seal`

/// ADR-048 §Q9 — `promoted_draft_has_no_public_constructor`. The seal: every
/// out-of-module construction path is a compile error. Each fixture is one bypass.
#[test]
fn promoted_draft_capability_token_is_unforgeable_outside_the_gate() {
    let t = trybuild::TestCases::new();
    // Born-RED: each of these is a bypass attempt that MUST NOT compile.
    t.compile_fail("tests/ui_promoted_draft/*.rs");
    // The teeth-check positive control: the gate's PUBLIC read-surface MUST compile.
    // (Proves the compile-fails fail because the constructor is sealed, not because
    // the path is wrong — without this, a broken import false-greens the whole suite.)
    t.pass("tests/ui_promoted_draft_pass/*.rs");
}
