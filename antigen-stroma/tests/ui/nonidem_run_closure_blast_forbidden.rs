// ATK-FRAME-NONIDEM (ADR-068 R3 / ADR-070 §4.6 attack A9) — the non-idempotent (counting/blast)
// semiring on a RAW graph MUST be a COMPILE error, NOT a runtime `if !IDEMPOTENT { panic!() }`.
//
// The failure this closes: a counting semiring run on antigen's CYCLIC graph without an SCC-
// condensation pre-pass is a ~100000x silent-slow blowup. The guard is the type system: `run_closure`
// is bounded `S: Idempotent`, and `Blast` deliberately does NOT implement `Idempotent`, so this call
// cannot type-check. The slow cyclic-graph path is unconstructible — blast is admitted ONLY through
// `run_condensed`, whose `Condensed` argument proves a condensation pre-pass ran.

use antigen_stroma::engine::{run_closure, Blast, ReachabilityRelation};

fn main() {
    let rel = ReachabilityRelation::from_edges(&[]);
    // MUST NOT COMPILE — the trait bound `Blast: Idempotent` is not satisfied.
    let _ = run_closure::<Blast>(&rel);
}
