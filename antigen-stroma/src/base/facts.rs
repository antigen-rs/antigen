//! STEP 3 — the fact tables (ADR-068; relational-as-base).
//!
//! ONE `#[salsa::input]` per base RELATION, NOT one mega-input. salsa's revision/invalidation is
//! per-input-field: splitting by relation gives the genome's RELATIONAL TUPLE-INVALIDATION for free
//! — an edit that changes call-resolution invalidates `EdgeFacts` WITHOUT touching `NodeFacts`/
//! `ContractFacts`. One mega-input would invalidate everything on any change (defeating incrementality).
//!
//! NO differential-dataflow (genome three-lineage convergence: DD is salsa-hostile, zero prior art,
//! refuted by the point-wise query-census). Do NOT scaffold DD.

// The three relations are salsa `#[input]`s (ADR-070 §4.6: ONE input per relation, so an edit that
// changes call-resolution invalidates `EdgeFacts` WITHOUT touching `NodeFacts`/`ContractFacts`). The
// salsa-generated `new`/accessors can't carry doc comments — the allow covers only that surface.
#![allow(missing_docs)]

use crate::node::node::Contract;
use crate::node::{IdentityDigest, StromaNodeId};
use crate::read::ResolutionTier;

/// The node relation — the `#[salsa::input]` table of node TUPLES (not the per-entity [`crate::node::node::Node`]
/// handle; that handle is the in-place-mutable identity layer, this is the relation the closure reads).
#[salsa::input]
pub struct NodeFacts {
    /// The node tuples in this snapshot, each keyed by its stable qualified-path identity.
    pub nodes: Vec<NodeFact>,
}

/// The edge relation. Populated from TWO feeders (syn syntactic + SCIP resolved), each tier-stamped;
/// the ladder (ADR-069) lives in `EdgeFact.tier`. salsa re-runs the closure ONLY when this changes.
#[salsa::input]
pub struct EdgeFacts {
    /// The edge tuples in this snapshot, each tier-stamped at the tier it was reconstructed at.
    pub edges: Vec<EdgeFact>,
}

/// The local-contract relation (provides/requires + provenance, ADR-067 §A.3).
#[salsa::input]
pub struct ContractFacts {
    /// The per-node local contracts (provides/requires + provenance) in this snapshot.
    pub contracts: Vec<ContractFact>,
}

/// One node tuple in the relation — the node's identity + its kind + the two digests.
///
/// The plain value-tuple form the closure iterates, distinct from the salsa [`crate::node::node::Node`]
/// handle (which is the in-place-mutable per-entity storage the maintenance pass `set_*`s).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeFact {
    /// The fully-qualified, collision-free, cfg-aware identity (ADR-067 §A.1).
    pub id: StromaNodeId,
    /// The node-kind discriminant (REUSED `ItemTarget` — the matching vocabulary, NOT the identity).
    pub kind: antigen::scan::ItemTarget,
    /// The collision-resistant signing digest (BLAKE3) — changes on any edit (the danger signal).
    pub identity_digest: IdentityDigest,
    /// The name-insensitive shape digest (FNV) — the clustering / near-miss / backdate key.
    /// Changes on a MEANING edit (body/signature change) but NOT on a rename.
    pub shape_digest: crate::node::digest::ShapeDigest,
}

/// One contract tuple in the relation — a node's local provides/requires + provenance (ADR-067 §A.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractFact {
    /// The node this contract belongs to.
    pub node: StromaNodeId,
    /// The local contract payload (provides/requires + provenance-tier).
    pub contract: Contract,
}

/// One edge tuple — tier-stamped (the resolution ladder lives HERE).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeFact {
    /// The source node — the edge points FROM here.
    pub src: StromaNodeId,
    /// The destination node — the edge points TO here.
    pub dst: StromaNodeId,
    /// The edge-kind discriminant (call/import/type-use/… — the OPEN registry).
    pub kind: EdgeKind,
    /// The tier this edge was reconstructed at — syntactic (syn) or resolved (SCIP). A syntactic
    /// edge can never corroborate up; the lattice-JOIN provenance semiring (ENGINE) raises tier only
    /// across fresh-independent sources.
    pub tier: ResolutionTier,
}

/// The edge-kind vocabulary — an **OPEN registry** (ADR-070 §4.6).
///
/// The discriminant accretes new kinds as new lenses are added (the open/closed cut — `EdgeKind` is
/// OPEN-extensible, the `provenance_tier` on [`EdgeFact`] is CLOSED). Each kind is a distinct
/// receptor a read can select for via [`crate::read::Perspective`]. The frame seeds the kinds the
/// syntactic + resolved feeders can populate today; later lenses (data-flow, co-change) accrete here.
///
/// `#[non_exhaustive]` so a downstream match must keep a wildcard arm — adding a kind is a
/// pure-accretion, never a breaking change (ADR-067 §B4 accrete-never-migrate at the type level).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum EdgeKind {
    /// A call edge: `src` invokes `dst`. Syntactic feeder (name-edge, `dread`) refined by SCIP
    /// (enclosure-reconstructed, `resolved`). The genome's primary detection relation.
    Call,
    /// An import / `use` edge: `src`'s module brings `dst` into scope.
    Import,
    /// A type-use edge: `src` mentions `dst` in a type position (field, param, return, bound).
    TypeUse,
    /// A trait-impl edge: `src` (an impl) implements trait `dst` (or `dst` is the impl's subject).
    TraitImpl,
    /// A proc-macro-use edge: `src` is annotated/expanded by macro `dst`. The degenerate input for
    /// SCIP enclosure-reconstruction (macro expansion breaks lexical enclosure — see [`crate::scip`]).
    ProcMacroUse,
    /// A co-change edge: `src` and `dst` historically change together (a temporal-tier signal).
    /// Populated by a future history/lifecycle lens; the kind is reserved so the schema is open now.
    CoChange,
    /// A data-flow edge: a value flows from `src` to `dst`. The IFDS/field-maths follow-on populates
    /// it (named-deferred, ADR-067 §E.11); the kind ships so the relational schema stays open.
    DataFlow,
    /// A lineage edge: an AUTHORED structural-projection link (rename/split/merge re-homing). Unlike
    /// the others this is a SovereignMerge-class authored edge, NOT a recomputable derivation
    /// (ADR-067 §F.14a) — the lifecycle layer authors it; the base never re-derives it.
    Lineage,
}

// THE CLOSURE (ENGINE epoch — NOT built here, signature shown so the frame's shape is legible):
//
//   #[salsa::tracked]
//   fn reachability(db: &dyn Db) -> ReachabilityRelation {
//       let edges = EdgeFacts::get(db).edges(db);
//       ascent::ascent_run! { ... }   // the semiring-datalog closure (genome: 33,882 pairs / 2.6ms)
//   }
//
// The 4 semirings over ONE query (detection/conductance/provenance/blast), the Semiring trait with
// `const IDEMPOTENT: bool`, the born-red NonIdempotentSemiringWithoutCondensation compile-assert, and
// SCC-condensation (Tarjan, for blast; IFDS is the field-maths follow-on, named-deferred) are ALL
// engine-epoch fills. The frame defines the relational SHAPE the engine queries; it does NOT scaffold
// the closure. (observer ledger G14 -> Type-A: frame = fact tables + API stubs; engine = closure.)
//
// IDEMPOTENT GATE SHAPE (adversarial: builder will wrongly implement as a runtime panic). The
// NonIdempotentSemiringWithoutCondensation guard must be COMPILE-TIME, not runtime. The pattern is a
// sealed-trait + a const bound, NOT a `if !IDEMPOTENT { panic!() }`:
//
//   trait Semiring { type Weight; const IDEMPOTENT: bool; /* plus/times/zero/one */ }
//   // A counting semiring (IDEMPOTENT=false) is constructible ONLY via a Condensed<S> wrapper:
//   struct Condensed<S: Semiring>(S);           // proves a condensation pre-pass ran
//   fn run_closure<S: Semiring>(facts: &EdgeFacts) -> ... where Assert<{ S::IDEMPOTENT }>: IsTrue {}
//   fn run_closure_condensed<S: Semiring>(c: Condensed<S>, ...) {}  // the only door for non-idempotent
//
// The non-condensed counting path is UNCONSTRUCTIBLE at compile time (the 100,000x silent-slow
// failure on antigen's cyclic graph cannot be written). This is ENGINE epoch, but the SHAPE is named
// here so the builder doesn't reach for a runtime assert.
