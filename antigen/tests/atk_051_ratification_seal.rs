//! ATK-051 — the ratification-record one-invariant seal (ADR-051 §Q9), BORN-RED.
//!
//! ADR-051's whole surface is **greenfield** (scout-confirmed, HEAD-grep 2026-06-11:
//! `RatificationSpecimen`, `PersistedSpecimen`, `Fate`, `accept`/`reject`/`narrow`,
//! `cargo antigen ratify` all need creating) and sits at build-order steps 7-8 —
//! far from the keystone-spine the pathmaker is on now. This file lands the born-red
//! spec **before** the surface, so the moment it exists the type-invariant is
//! defended and the un-ignore is a one-line act, not a fresh test-design.
//!
//! Per the baton: *"they may be `#[ignore]`'d pending implementation; the bodies are
//! the spec."* Each test below is `#[ignore]`'d with a precise un-ignore condition;
//! its body is the executable spec ADR-051 §Q9 names. When the pathmaker lands the
//! surface, replace the `unimplemented!` shim with the real call and remove the
//! `#[ignore]` — the body already says what "done" means.
//!
//! # The ONE invariant (lead with it — every hole is a corollary)
//!
//! > **THE RATIFICATION RECORD ACCEPTS ONLY A FRESH `PromotedDraft` CAPABILITY
//! > TOKEN, NEVER A BARE `Fingerprint`.**
//!
//! This is ADR-048's capability-token discipline applied to the ratification
//! *surface* — the same trust boundary, one level out (aristotle's recognition). It
//! closes the whole ratification-bypass family BY TYPE, as corollaries not patches:
//! auto-accept-launder (Hole-1), the route-to-human-lie (Hole-2), human-narrow-
//! without-re-gate (Hole-3), the serde-forgery / persistence-launder (Hole-4). The
//! adversarial's residual-probe is the test: *is there ANY path to recorded-as-
//! accepted that does NOT go through a `PromotedDraft`?* If no such path compiles,
//! the one invariant holds.
//!
//! # The compile-fail seal (authored, NOT yet wired — see `the_compile_fail_seal`)
//!
//! The load-bearing test is a trybuild compile-fail: `accept(bare_fingerprint, …)`
//! MUST NOT compile. It cannot be wired today (the `accept` surface does not exist —
//! a fixture referencing it would fail to compile for "unresolved import", a
//! FALSE-RED that proves nothing). The fixtures' bodies are specified inline in
//! `the_compile_fail_seal` below, ready to drop into `tests/ui_ratify/` + a runner
//! line when the surface lands. This is the honest born-red form for a greenfield
//! type-seal: the spec is written, the activation is mechanical.

#![allow(
    unused_imports,
    dead_code,
    unreachable_code,
    clippy::diverging_sub_expression
)]

use antigen::learn::self_tolerance::{PromotedDraft, ToleranceVerdict};
use antigen_fingerprint::Fingerprint;

// ───────────────────────────────────────────────────────────────────────────
// The compile-fail seal — the ONE invariant, authored as a spec to wire on
// surface-landing. (Not executable; documents the trybuild fixtures + the runner.)
// ───────────────────────────────────────────────────────────────────────────

/// ADR-051 §Q9 — `accept_takes_only_a_promoted_draft_not_a_bare_fingerprint`
/// (THE LOAD-BEARING ONE-INVARIANT TEST). The compile-fail suite that proves the
/// whole bypass-family is closed by type. **Authored here; wire on surface-landing.**
///
/// When `RatificationSpecimen::accept` (or the free `accept`) lands, create
/// `tests/ui_ratify/*.rs` with one fixture per candidate bypass — EACH must fail to
/// compile as an `accept` argument:
///
/// 1. `accept_bare_fingerprint.rs` — `accept(a_bare_fingerprint, tier)` → the
///    accept-path is typed on `PromotedDraft`; a bare `Fingerprint` is not accepted.
/// 2. `accept_into_fingerprint_output.rs` — `accept(token.into_fingerprint(), …)` →
///    the downgrade yields a bare `Fingerprint` (ADR-048 §4 class), not acceptable.
/// 3. `accept_narrow_output.rs` — `accept(narrow(specimen, c), …)` → `narrow` yields
///    a bare `Fingerprint` (ADR-051 member of the downgrade class), not acceptable
///    until it re-enters `promote_if_safe`.
/// 4. `accept_deserialized_token.rs` — a serde-deserialized token is impossible
///    (ADR-048 §5: no `Deserialize`), so even attempting it fails to compile.
///
/// PLUS a `tests/ui_ratify_pass/` positive control: `accept(a_real_promoted_draft,
/// tier)` MUST compile (the teeth-check — proves the fails are about the type, not a
/// broken path). Wire all five into a `trybuild::TestCases` runner mirroring
/// `atk_048_promoted_draft_seal.rs`. The `.stderr` snapshots are STABLE-blessed.
#[test]
#[ignore = "born-red: ADR-051 accept surface is greenfield; wire the ui_ratify/ \
            compile-fail suite when RatificationSpecimen::accept lands (steps 7-8)"]
fn the_compile_fail_seal_accept_only_promoted_draft() {
    // This test BODY is intentionally a directive, not an assertion: the real
    // defense is the trybuild fixtures documented above. Un-ignoring this without
    // wiring them is a no-op — the doc comment IS the spec. (Kept as a registry
    // anchor so `cargo test -- --ignored` lists the undefended claim by name.)
    panic!("WIRE the tests/ui_ratify/ compile-fail suite — see this test's doc comment");
}

// ───────────────────────────────────────────────────────────────────────────
// Runtime born-red specs — bodies = the ADR-051 §Q9 spec, ready to un-ignore.
// Each calls a NAMED future surface via a shim; replace the shim with the real
// call + drop the #[ignore] when the surface lands.
// ───────────────────────────────────────────────────────────────────────────

/// ADR-051 §Q9 — `not_corpus_witnessable_draft_lands_as_pending_specimen`. The
/// GATE-G route-to-human handoff (Hole-2 made concrete): a `NotCorpusWitnessable`
/// draft has NO token (it is structurally a `Fingerprint`), so it CANNOT be recorded
/// as accepted; it lands in the pending list flagged needs-human-generalization-
/// judgment. The route-to-human verdict is wired, not dropped.
#[test]
#[ignore = "born-red: ADR-051 RatificationSpecimen surface greenfield; un-ignore \
            when the pending-list + gate_verdict field land"]
fn not_corpus_witnessable_draft_lands_as_pending_specimen() {
    // SPEC (un-ignore + wire when the surface lands):
    //   let single = Fingerprint { constraints: vec![Constraint::BodyCalls("unwrap".into())] };
    //   let clean = /* a corpus with no near-miss for `single` */;
    //   // The gate routes it to human (no token minted):
    //   let verdict = promote_if_safe(single.clone(), &clean);
    //   assert_eq!(verdict, Err(ToleranceVerdict::NotCorpusWitnessable));
    //   // It lands as a PENDING specimen flagged needs-human-judgment, and it
    //   // CANNOT be accepted (no PromotedDraft exists for it — Hole-2 by type):
    //   let specimen = RatificationSpecimen::pending_from_route_to_human(single, &clean);
    //   assert_eq!(specimen.fate, Fate::Pending);
    //   assert_eq!(specimen.gate_verdict, ToleranceVerdict::NotCorpusWitnessable);
    //   // `accept(specimen, ..)` is type-impossible (no token) — the compile-fail seal.
    unimplemented!("ADR-051 RatificationSpecimen + pending list not yet built");
}

/// ADR-051 §Q9 — `persisted_specimen_round_trips_as_bare_fingerprint_and_re_mints_on_load`
/// (the adversarial's must-fix-before-baton, the FIFTH surface). The in-memory
/// `RatificationSpecimen` holds the live `PromotedDraft` and is NOT serde; the
/// on-disk `PersistedSpecimen` holds a bare `Fingerprint` and IS serde; load
/// re-mints via `promote_if_safe` against the PERSISTED `spared` corpus (a
/// provenance re-check, NO live scan — the pruner-creep guard).
#[test]
#[ignore = "born-red: ADR-051 PersistedSpecimen surface greenfield; un-ignore when \
            the persist/load round-trip lands"]
fn persisted_specimen_round_trips_as_bare_fingerprint_and_re_mints_on_load() {
    // SPEC (un-ignore + wire when the surface lands):
    //   (a) PersistedSpecimen is Serialize+Deserialize; RatificationSpecimen is NOT
    //       (a compile/trait-bound assertion — the token never round-trips serde).
    //   (b) an HONEST round-trip re-mints successfully:
    //       let saved = specimen.persist();              // → PersistedSpecimen (bare fp)
    //       let json = serde_json::to_string(&saved).unwrap();
    //       let loaded: PersistedSpecimen = serde_json::from_str(&json).unwrap();
    //       let live = loaded.re_mint().expect("honest record re-mints");
    //   (c) a TAMPERED persisted draft FAILS the re-gate (forgery-detection):
    //       let mut tampered = saved.clone();
    //       tampered.draft = an_autoimmune_fingerprint;
    //       assert!(tampered.re_mint().is_err());        // the gate refuses the forgery
    //   (d) re_mint re-gates against the PERSISTED `spared` corpus, NOT the live
    //       codebase (the pruner-creep guard — assert no filesystem scan occurs).
    unimplemented!("ADR-051 PersistedSpecimen round-trip not yet built");
}

/// ADR-051 §Q9 — `narrow_produces_a_fingerprint_that_must_regate`. `narrow` is a
/// MEMBER of the ADR-048 downgrade-then-re-gate class (co-named with
/// `into_fingerprint`): it yields a bare `Fingerprint`, NOT a silently re-tokened
/// `PromotedDraft`. The narrowed fingerprint must re-enter `promote_if_safe` to
/// re-acquire a token before it is recordable-as-accepted.
#[test]
#[ignore = "born-red: ADR-051 narrow surface greenfield; un-ignore when narrow lands"]
fn narrow_produces_a_fingerprint_that_must_regate() {
    // SPEC (un-ignore + wire when the surface lands):
    //   let specimen = /* a real RatificationSpecimen holding a token */;
    //   let narrowed: Fingerprint = narrow(&specimen, an_added_constraint);
    //   //                ^^^^^^^^^ NOT a PromotedDraft — a bare Fingerprint.
    //   // It is NOT acceptable-by-type until it re-gates:
    //   let re_token = promote_if_safe(narrowed, &specimen.spared_corpus());
    //   // only `re_token` (an Ok PromotedDraft) is recordable-as-accepted.
    unimplemented!("ADR-051 narrow not yet built");
}

/// ADR-051 §Q9 — `every_ratify_act_writes_a_fate`. accept/reject/narrow each write a
/// `Fate`; a specimen cannot leave the loop with `Fate::Pending` silently (the
/// fate-hook is exercised — the L4-staleness precondition, the maturation feedback).
#[test]
#[ignore = "born-red: ADR-051 Fate + fate-hook greenfield; un-ignore when the \
            three verbs + fate-hook land"]
fn every_ratify_act_writes_a_fate() {
    // SPEC (un-ignore + wire when the surface lands):
    //   // accept → Fate::Accepted{tier}; reject → Fate::Rejected{reason};
    //   // narrow-then-re-gate-then-accept → Fate::Narrowed{new_fingerprint}.
    //   let accepted = accept(specimen_a, tier);
    //   assert!(matches!(accepted.fate, Fate::Accepted { .. }));
    //   let rejected = reject(specimen_b, "autoimmune".into());
    //   assert!(matches!(rejected.fate, Fate::Rejected { .. }));
    //   // No verb leaves the specimen Pending — the fate-hook always fires.
    unimplemented!("ADR-051 Fate + three verbs not yet built");
}

/// ADR-051 §Q9 — `human_and_agent_render_the_same_record`. The CLI rendering and the
/// structured-query rendering read the SAME `RatificationSpecimen` — no parallel
/// ratification state (the co-native invariant; `ParallelStateTrackersDiverge` at
/// the ratification boundary is the failure this defends).
#[test]
#[ignore = "born-red: ADR-051 two-rendering surface greenfield; un-ignore when the \
            record + CLI land (agent rendering is charter — assert the SHAPE is one)"]
fn human_and_agent_render_the_same_record() {
    // SPEC (un-ignore + wire when the surface lands):
    //   // Both renderings take a `&RatificationSpecimen` — one record, two views.
    //   // Assert the CLI list and the (do-now stub of the) structured query read
    //   // the identical specimen instance / the identical schema — never a second
    //   // ratification-state struct (the co-native one-record invariant).
    unimplemented!("ADR-051 two-rendering record not yet built");
}

// ───────────────────────────────────────────────────────────────────────────
// The ADR-047-Amd2 ↔ ADR-051 COUPLING — the nested-vacuity fix is a 051-soundness
// PREREQ (captain batch-3 + the adversarial's 051-seal audit). 051's narrow()/
// re-mint RE-PARSE user-edited fingerprints into ARBITRARY nesting — so 051's
// SHAPE-soundness (the reconstructed fingerprint re-gates to the right verdict, and
// forgery-detection holds against a NESTED autoimmune draft) rides the gate's
// recursive canonical-form (Amd2 Hole-II). The original atk_051 seal tested the
// accept-only-PromotedDraft TYPE invariant but NOT the SHAPE of the re-minted
// fingerprint — exactly where the shape-fragility detonates. These close that gap.
// ───────────────────────────────────────────────────────────────────────────

/// ADR-051 ↔ ADR-047 Amd2 — `narrow_output_is_producer_normalized`. A `narrow()` that
/// reconstructs the fingerprint through the `all_of(..)` surface (or a re-parse) can
/// emit a WRAPPED/NESTED shape. The re-gate verdict on a narrowed draft MUST be
/// producer-independent — identical for the flat and the nested form of the same
/// semantic fingerprint — i.e. it rides the gate's recursive canonical-form (Amd2
/// Hole-II). Without it, a narrowed draft's (A)-binary / near-miss verdict depends on
/// whether `narrow` happened to nest — a `ParallelStateTrackersDiverge` at the
/// ratification surface.
#[test]
#[ignore = "born-red: ADR-051 narrow surface greenfield + rides ADR-047 Amd2 \
            recursive-normalize (Hole-II); un-ignore when narrow + the recursive \
            canonical-form both land"]
fn narrow_output_is_producer_normalized() {
    // SPEC (un-ignore when narrow + Amd2 recursive-normalize land):
    //   // The SAME semantic fingerprint, flat vs nested (as narrow/parse might build):
    //   let flat   = Fingerprint { constraints: vec![/* impl, Drop, body_calls("x") */] };
    //   let nested = Fingerprint { constraints: vec![Constraint::AllOf(flat.constraints.clone())] };
    //   // After Amd2's recursive normalize, the gate verdict is IDENTICAL:
    //   assert_eq!(
    //       promote_if_safe(flat,   &corpus).map(|t| t.tier()),
    //       promote_if_safe(nested, &corpus).map(|t| t.tier()),
    //       "a narrowed draft's verdict must be producer-independent (Amd2 Hole-II)"
    //   );
    unimplemented!("ADR-051 narrow + ADR-047 Amd2 recursive-normalize not yet built");
}

/// ADR-051 ↔ ADR-047 Amd2 — `tampered_nested_autoimmune_draft_is_caught_by_re_mint`.
/// The `PersistedSpecimen` forgery-detection (re-mint must `Err` on a tampered draft)
/// FALSE-GREENS against a NESTED autoimmune draft until the recursive canonical-form
/// lands: a forger persists an autoimmune fingerprint WRAPPED in a redundant `AllOf`
/// so the single-level normalize doesn't see through it, and the re-gate spuriously
/// promotes (`Ok`) instead of refusing — the forgery passes. The fix (Amd2 Hole-II
/// recursive flatten) makes the re-mint see the flattened autoimmune shape and `Err`.
/// This is the adversarial's 051-soundness elevation (captain batch-3).
#[test]
#[ignore = "born-red: ADR-051 PersistedSpecimen re-mint greenfield + rides ADR-047 \
            Amd2 recursive-normalize; un-ignore when re-mint + recursive canonical-form land"]
fn tampered_nested_autoimmune_draft_is_caught_by_re_mint() {
    // SPEC (un-ignore when PersistedSpecimen re-mint + Amd2 recursive-normalize land):
    //   // A forger hand-edits a persisted draft to an autoimmune one, WRAPPED in a
    //   // redundant AllOf to evade a single-level normalize:
    //   let autoimmune_nested = Fingerprint {
    //       constraints: vec![Constraint::AllOf(an_autoimmune_fingerprint.constraints)]
    //   };
    //   let tampered = PersistedSpecimen { draft: autoimmune_nested, .. };
    //   // The re-mint (re-gate against the persisted spared corpus) MUST catch it —
    //   // the recursive normalize flattens the wrapper, the gate sees the autoimmune
    //   // shape, spare-clean fails → Err. (Without Amd2 Hole-II: spuriously Ok = the
    //   // forgery FALSE-GREENS.)
    //   assert!(tampered.re_mint().is_err(),
    //       "a nested autoimmune draft must NOT survive re-mint (Amd2 Hole-II closes \
    //        the persistence-launder via nesting)");
    unimplemented!("ADR-051 re-mint + ADR-047 Amd2 recursive-normalize not yet built");
}
