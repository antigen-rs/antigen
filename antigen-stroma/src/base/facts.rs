//! The fact tables (ADR-068; relational-as-base).
//!
//! ONE `#[salsa::input]` per base RELATION, not one mega-input. salsa's revision/invalidation is
//! per-input-field: splitting by relation gives relational tuple-invalidation for free — an edit
//! that changes call-resolution invalidates `EdgeFacts` without touching `NodeFacts`/`ContractFacts`.
//! One mega-input would invalidate everything on any change, defeating incrementality.
//!
//! The base uses salsa for incremental memoization, not differential-dataflow.

// The three relations are salsa `#[input]`s (ADR-070 §4.6: ONE input per relation, so an edit that
// changes call-resolution invalidates `EdgeFacts` WITHOUT touching `NodeFacts`/`ContractFacts`). The
// salsa-generated `new`/accessors can't carry doc comments — the allow covers only that surface.
#![allow(missing_docs)]

use crate::node::StromaNodeId;
use crate::node::node::Contract;
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
    /// The fully-qualified, collision-free, cfg-aware identity (ADR-067 §A.1). The signing
    /// `identity_digest` lives HERE (inside `id`) — the SINGLE source of truth; read it via
    /// `node.id.identity_digest`. (The separate salsa `Node` handle carries the digest as a
    /// top-level mutable field because it is `set_*`-mutated in place; a `NodeFact` is a value
    /// tuple, so it holds the digest once.)
    pub id: StromaNodeId,
    /// The node-kind discriminant (REUSED `ItemTarget` — the matching vocabulary, NOT the identity).
    pub kind: antigen::scan::ItemTarget,
    /// The name-insensitive shape digest (FNV) — the clustering / near-miss / backdate key. A SIBLING
    /// of identity (NOT part of the `id` 3-tuple): it changes on a MEANING edit (body/signature) but
    /// NOT on a rename, and two different-named same-shape items SHARE it. So it lives top-level here.
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
/// receptor a read can select for via [`crate::read::Perspective`]. The registry holds the kinds the
/// syntactic + resolved feeders populate; other lenses (data-flow, co-change) accrete their own kinds.
///
/// `#[non_exhaustive]` so a downstream match must keep a wildcard arm — adding a kind is a
/// pure-accretion, never a breaking change (ADR-067 §B4 accrete-never-migrate at the type level).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum EdgeKind {
    /// A call edge: `src` invokes `dst`. Syntactic feeder (name-edge, `dread`) refined by SCIP
    /// (enclosure-reconstructed, `resolved`). The primary detection relation.
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
    /// A co-change edge: `src` and `dst` historically change together (a temporal-tier signal). A
    /// history/lifecycle lens populates it; the kind is present so the relational schema stays open.
    CoChange,
    /// A data-flow edge: a value flows from `src` to `dst`. A data-flow lens (ADR-067 §E.11)
    /// populates it; the kind is present so the relational schema stays open.
    DataFlow,
    /// A lineage edge: an AUTHORED structural-projection link (rename/split/merge re-homing). Unlike
    /// the others this is a SovereignMerge-class authored edge, NOT a recomputable derivation
    /// (ADR-067 §F.14a) — the lifecycle layer authors it; the base never re-derives it.
    Lineage,
}

// The reachability closure reads these fact tables; its shape (shown so the relational contract is
// legible from here):
//
//   #[salsa::tracked]
//   fn reachability(db: &dyn Db) -> ReachabilityRelation {
//       let edges = EdgeFacts::get(db).edges(db);
//       ascent::ascent_run! { ... }   // the semiring-datalog closure
//   }
//
// Four semirings run over one query (detection/conductance/provenance/blast) via a `Semiring` trait
// with `const IDEMPOTENT: bool`, plus SCC-condensation (Tarjan) for the blast tier. These fact tables
// define the relational shape the closure queries; they do not contain the closure itself.
//
// The non-idempotent (counting) semiring must be gated at COMPILE time, not with a runtime
// `if !IDEMPOTENT { panic!() }`: a counting semiring on a cyclic graph without condensation is a
// silent-slowdown, so the type system forbids writing it. The idiom is a sealed trait + a const bound:
//
//   trait Semiring { type Weight; const IDEMPOTENT: bool; /* plus/times/zero/one */ }
//   struct Condensed<S: Semiring>(S);           // proves a condensation pre-pass ran
//   fn run_closure<S: Semiring>(facts: &EdgeFacts) -> ... where Assert<{ S::IDEMPOTENT }>: IsTrue {}
//   fn run_closure_condensed<S: Semiring>(c: Condensed<S>, ...) {}  // the only door for non-idempotent
//
// The non-condensed counting path is unconstructible at compile time — the slow cyclic-graph case
// cannot be written.
