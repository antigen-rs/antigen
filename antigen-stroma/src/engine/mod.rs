//! The read-engine (ADR-068 datalog-closure layer) — the reachability closure the point-wise
//! [`crate::read::query`] functions read from.
//!
//! ONE relational base ([`crate::base::facts::EdgeFacts`]), ONE closure, FOUR semirings that differ
//! only in the value folded per node-pair (ADR-068 R1): detection (boolean), conductance (tropical-min
//! hop-count), provenance (lattice-JOIN over [`crate::read::ResolutionTier`]), and blast (counting,
//! NON-idempotent → compile-gated behind SCC-condensation).
//!
//! FABLE ONE-SHOT NOTES (reconciliation index — every assumption/simplification/DESIGN-tier deferral):
//!
//! - SIMPLIFIED (Fable, salsa): the point-wise queries receive only a `SnapshotHandle { db }`, so the
//!   edge relation must be retrievable from `&StromaDb` alone. `base::facts::EdgeFacts` is a plain
//!   `#[salsa::input]` (NOT a salsa singleton — `base/facts.rs` is frozen), so there is no
//!   `EdgeFacts::get(db)`. We therefore hold the created `EdgeFacts` input HANDLE on the db
//!   (`StromaDb::set_edge_facts`/`edge_facts`, added to `db.rs`). Faithful to R9 (SCIP still populates
//!   the real salsa input); the handle just makes it reachable. The closure reads it via the salsa
//!   accessor `edge_facts.edges(db)`.
//!
//! - SIMPLIFIED (Fable, no memoization): the closure is recomputed per query call (build
//!   `ReachabilityRelation`, traverse) rather than wrapped in a `#[salsa::tracked]` memoized fn. R5
//!   permits this fallback explicitly; salsa is still the revision clock / atomic-publish keystone (the
//!   `&db`/`&mut db` split), but the closure memoization is left to the harden wave. n is tiny; this is
//!   correct, just not incremental.
//!
//! - SIMPLIFIED (Fable, compile-gate idiom): R3 names `where Assert<{ S::IDEMPOTENT }>: IsTrue` on the
//!   GENERIC fn. That is a generic-const-expression (the const arg depends on the type param `S`) and
//!   needs nightly `generic_const_exprs`; the crate ships to STABLE. So the generic gate on
//!   [`run_closure`] is a sealed marker trait [`semiring::Idempotent`] instead — SAME invariant: the
//!   non-idempotent counting path is UNCONSTRUCTIBLE via `run_closure` (`run_closure::<Blast>` fails to
//!   compile, `Blast: Idempotent` unsatisfied). The ADR-named `Assert`/`IsTrue` types ARE still defined
//!   and are load-bearing: a `const` coherence block statically asserts every `Idempotent` semiring's
//!   `IDEMPOTENT == true` (concrete-const args are stable), so the marker trait and the const cannot
//!   drift. Non-idempotent semirings run only via [`run_condensed`], which consumes a
//!   [`semiring::Condensed`] proof-of-pre-pass.
//!
//! - DESIGN-tier (untouched, commented only): the λ-temperature unification (ADR-068 Am1) and the
//!   per-`Perspective` sub-closures are OUT of this MVP — the closure runs over `AllEdges` /
//!   `Polarity::Is` (every `EdgeKind`, forward polarity). `field_at`'s danger-marker is DESIGN-tier;
//!   the MVP treats the argument node as the danger source (R-skeleton).
//!
//! - Answer tier honesty (R6): an answer's tier is the MINIMUM edge-tier traversed
//!   (`Syntactic`-floor); provenance is the exception it is defined to be — the MAX (lattice-JOIN)
//!   tier on any path. Tier is handed to [`crate::read::TieredAnswer::answered`], which derives the
//!   grade (a `Syntactic` source ⇒ `Dread`-grade by construction).

pub mod closure;
pub mod condense;
pub mod mincut;
pub mod path_homology;
pub mod semiring;

pub use closure::ReachabilityRelation;
pub use condense::{Condensation, condense};
pub use path_homology::{PathHomology, TierBarcode, path_homology, undirected_flag_h1};
pub use semiring::{
    Assert, Blast, Condensed, Conductance, Detection, Idempotent, IsTrue, Provenance, Semiring,
    run_closure, run_condensed,
};
