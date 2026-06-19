//! ATK-DEEPCOMB-MATURE — degenerate-input attack on the maturation engine's
//! `max_budget` boundary (Path B deep-comb, the tracked ATK-MATURE-6).
//!
//! **STATUS: born-red on 86cea89 — a LIVE PANIC reachable from the PUBLIC API.**
//!
//! `antigen::learn::maturation::mature` is a `pub fn` taking `max_budget: usize`. Its
//! first act each round is `mutation_budget(current_affinity, max_budget)`, which ends
//! in `(scaled.round() as usize).clamp(1, max_budget)`. For `max_budget == 0` that is
//! `clamp(1, 0)` — and Rust's `Ord::clamp` PANICS when `min > max`
//! (`min > max. min = 1, max = 0` at `maturation.rs:136`). A `usize` of `0` is a
//! perfectly valid typed input; a public API that panics on it is a defect, not a
//! contract the caller could have known to avoid (the docstring says "a budget in
//! `1..=max_budget`" but never states `max_budget >= 1` as a precondition, and the
//! type does not enforce it).
//!
//! Honest scope (the briefing's question — "reachable from a public path on 86cea89?"):
//! `mature` has NO production/CLI caller yet on 86cea89 (only its own tests + the
//! roadmap reference it), so this is NOT CLI-reachable today. But it IS reachable from
//! the **public library API** — any consumer of the `antigen` crate calling
//! `maturation::mature(.., 0, ..)` hits the panic. A `pub fn` is a public path.
//!
//! # The fix (proposed)
//!
//! `max_budget == 0` means "zero exploration width" — the honest semantics is a budget
//! of `0` (explore no candidates → `.take(0)` → no Pareto improvement → the climb halts
//! immediately at the initial draft), NOT a panic and NOT a silent floor of 1 (which
//! would explore a candidate the caller asked to forbid). `mutation_budget` must return
//! `0` when `max_budget == 0`, and `1..=max_budget` otherwise.

use antigen::learn::life_record::{LifeEvent, LifeRecord};
use antigen::learn::maturation::mature;
use antigen_fingerprint::{Constraint, Fingerprint};

fn items(src: &str) -> Vec<syn::Item> {
    syn::parse_file(src).expect("ATK fixture parses").items
}

fn two_call_draft() -> Fingerprint {
    Fingerprint {
        constraints: vec![
            Constraint::BodyCalls("unwrap".into()),
            Constraint::BodyCalls("flush".into()),
        ],
    }
}

// ---------------------------------------------------------------------------
// ATK-DEEPCOMB-MATURE-1 — max_budget == 0 must NOT panic (the live ATK-MATURE-6).
// ---------------------------------------------------------------------------

#[test]
fn atk_deepcomb_mature1_zero_budget_does_not_panic() {
    let cluster = items("fn a(){ x.unwrap(); x.flush(); } fn b(){ y.unwrap(); y.flush(); }");
    let clean = items("fn c(){ z.ok(); }");
    let mut rec = LifeRecord::new("zero-budget");

    // BORN-RED: this currently panics at maturation.rs:136 (clamp(1, 0), min > max).
    let matured = mature(two_call_draft(), &cluster, &clean, &mut rec, 0, 5);

    // With zero exploration budget, the climb must halt at the initial draft (no
    // candidate explored), returning the unchanged draft — never panic.
    assert_eq!(
        matured.draft,
        two_call_draft(),
        "ATK-DEEPCOMB-MATURE-1: max_budget=0 means zero exploration — the climb must \
         halt at the initial draft, returning it unchanged. A panic on a valid usize=0 \
         input is the defect.",
    );
    // The record holds exactly the initial Scored event (no Matured transition).
    assert!(
        rec.events()
            .iter()
            .any(|e| matches!(e, LifeEvent::Scored(_))),
        "ATK-DEEPCOMB-MATURE-1: the initial affinity is still scored into the record.",
    );
    assert!(
        !rec.events().iter().any(|e| matches!(e, LifeEvent::Matured)),
        "ATK-DEEPCOMB-MATURE-1: zero budget explores nothing, so no Matured transition \
         is recorded.",
    );
}

// ---------------------------------------------------------------------------
// ATK-DEEPCOMB-MATURE-2 — max_budget == 1 still works (the boundary above zero):
// a budget of 1 explores exactly one candidate; no off-by-one regression from the fix.
// ---------------------------------------------------------------------------

#[test]
fn atk_deepcomb_mature2_budget_one_still_explores() {
    let cluster = items("fn a(){ x.unwrap(); x.flush(); } fn b(){ y.unwrap(); y.flush(); }");
    // A clean sibling that does NOT call unwrap (so widening the draft can Pareto-improve
    // by dropping a discriminator). The point is only that budget=1 does not panic and
    // returns a valid result.
    let clean = items("fn c(){ z.ok(); z.flush(); }");
    let mut rec = LifeRecord::new("budget-one");

    let matured = mature(two_call_draft(), &cluster, &clean, &mut rec, 1, 5);
    // No panic, a real trajectory recorded — the boundary just above zero is intact.
    assert!(
        !matured.trajectory.is_empty(),
        "ATK-DEEPCOMB-MATURE-2: max_budget=1 must explore (≥1 trajectory point) without \
         panic — the fix for zero must not break the budget=1 boundary.",
    );
}

// ---------------------------------------------------------------------------
// ATK-DEEPCOMB-MATURE-3 — max_rounds == 0 is already safe (the loop never runs the
// budget). Pins that the zero-boundary on the OTHER bound is honest (no panic, just
// the initial score). A control: confirms only the max_budget bound was the hazard.
// ---------------------------------------------------------------------------

#[test]
fn atk_deepcomb_mature3_zero_rounds_is_safe_initial_score_only() {
    let cluster = items("fn a(){ x.unwrap(); x.flush(); } fn b(){ y.unwrap(); y.flush(); }");
    let clean = items("fn c(){ z.ok(); }");
    let mut rec = LifeRecord::new("zero-rounds");

    // max_rounds=0 with a SAFE max_budget: the for-loop body never runs, so
    // mutation_budget is never called — no panic regardless. Initial score only.
    let matured = mature(two_call_draft(), &cluster, &clean, &mut rec, 8, 0);
    assert_eq!(
        matured.draft,
        two_call_draft(),
        "ATK-DEEPCOMB-MATURE-3: max_rounds=0 halts before any round — initial draft \
         unchanged.",
    );
    assert_eq!(
        rec.events()
            .iter()
            .filter(|e| matches!(e, LifeEvent::Scored(_)))
            .count(),
        1,
        "ATK-DEEPCOMB-MATURE-3: exactly the one initial Scored event with zero rounds.",
    );
}
