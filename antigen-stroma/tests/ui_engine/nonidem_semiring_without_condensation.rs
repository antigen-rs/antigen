// ATK-FRAME-NONIDEM (inherited, ENGINE-epoch PLACEHOLDER) — a non-idempotent semiring on a RAW graph
// MUST be a COMPILE error. ADR-068 clause-3 / ADR-070 §4.6 (attack A9).
//
// The failure this closes: a non-idempotent (counting/blast) semiring run on antigen's CYCLIC graph
// WITHOUT a condensation pre-pass is a 100,000x silent-slow blowup. The guard must be COMPILE-TIME
// (the law's "compile-assert enforced"), NOT a runtime `if !IDEMPOTENT { panic!() }`.
//
// THE RECOMMENDED TYPE-STATE (ADR-070 §4.6): a non-idempotent (counting) query is constructible ONLY
// via a `CondensedGraph` (proving a condensation pre-pass ran). The raw-graph counting path is
// UNCONSTRUCTIBLE — its signature requires `CondensedGraph`, so passing a raw `StromaGraph` does not
// type-check.
//
// THIS FIXTURE (engine epoch — NOT wired by the frame-epoch compile_fail harness): call the
// non-idempotent blast query with a RAW graph. MUST NOT compile.
//
// PLACEHOLDER STATUS: this is the born-red obligation RECORDED FORWARD (the way ADR-067 names
// `GlobalConsistencyObstruction` deferred-to-sheaf rather than minting a dangling defense). The engine
// wave builds `CondensedGraph` + the `Semiring` trait + flips this fixture into `tests/ui/`. Until
// then it references engine-fill types and is intentionally not run.

use antigen_stroma::engine::{blast_query, CountingSemiring, StromaGraph};

fn main() {
    let raw: StromaGraph = StromaGraph::default();
    // blast_query is non-idempotent → its signature requires a CondensedGraph. A raw graph MUST NOT
    // type-check (the 100,000x silent-slow path is unconstructible).
    let _ = blast_query::<CountingSemiring>(&raw); // MUST NOT COMPILE — expected CondensedGraph.
}
