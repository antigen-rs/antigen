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

use std::collections::BTreeSet;

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

/// The SCIP symbol relation — **what the SCIP indexer says ABOUT a symbol**, in SCIP's own
/// vocabulary.
///
/// Its own relation rather than fields on [`NodeFact`], for two reasons the substrate must not
/// paper over:
/// - **[`NodeFact`] is a syn-sourced tuple** — it carries an `ItemTarget` kind and a measured
///   `shape_digest`. SCIP measures neither, so a SCIP-sourced `NodeFact` would have to FABRICATE
///   them.
/// - **The vocabularies do not nest.** SCIP separates `EnumMember` from `Struct`, and `TraitMethod`
///   from `Method`; `ItemTarget` carries neither distinction. Mapping SCIP's kind into it would
///   discard precisely the evidence that makes a second instrument worth having (`syn` structurally
///   cannot tell `Some(x)` from a call; SCIP knows it is an `EnumMember`).
///
/// So each instrument's observations are held in that instrument's own terms, and any organ that
/// wants a unified answer JOINS them — the join is a projection, and projections belong to organs.
#[salsa::input]
pub struct SymbolFacts {
    /// What SCIP observed about each symbol it describes, in this snapshot.
    pub symbols: Vec<SymbolFact>,
}

/// One SCIP symbol observation — its identity, SCIP's classification of it, and its docs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolFact {
    /// The raw SCIP symbol string — the instrument's own identity for this thing, kept verbatim so
    /// a join can be re-derived at any fidelity (bare-name today, per-symbol later).
    pub symbol: String,
    /// SCIP's classification, un-mapped ([`ScipKind`]).
    pub kind: ScipKind,
    /// The doc-comment text SCIP carries for the symbol (empty when it carries none).
    pub documentation: Vec<String>,
    /// Symbols this one is an implementation OF (`Relationship.is_implementation`) — the trait→impl
    /// dispatch map. MEASURED EMPTY in a `rust-analyzer 1.96 scip` index (0 of 19141 symbols carry
    /// any relationship), so the slot is held open rather than assumed: an absent relationship is
    /// this instrument's silence, never evidence that no implementation exists.
    pub is_implementation_of: Vec<String>,
}

/// SCIP's own symbol classification — held UN-MAPPED (never collapsed into `ItemTarget`).
///
/// `Other` keeps unrecognized kinds addressable instead of silently dropping them: a kind we have
/// no name for is still an observation, and discarding it would be a pre-classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ScipKind {
    /// A free function.
    Function,
    /// An inherent method.
    Method,
    /// A trait method (declared on a trait).
    TraitMethod,
    /// An enum variant — a `Foo::Bar(x)` "call" is a CONSTRUCTION, not a call. The distinction
    /// `syn` structurally cannot make.
    EnumMember,
    /// A struct — a tuple-struct "call" is likewise a construction.
    Struct,
    /// An enum type.
    Enum,
    /// A trait type.
    Trait,
    /// A macro.
    Macro,
    /// A struct/enum field.
    Field,
    /// A generic type parameter.
    TypeParameter,
    /// A variable / binding.
    Variable,
    /// A static/associated method (no receiver).
    StaticMethod,
    /// A module.
    Module,
    /// A `const` item.
    Constant,
    /// A type alias.
    TypeAlias,
    /// An attribute (proc-macro / derive).
    Attribute,
    /// A kind this vocabulary does not name yet — held with its raw SCIP discriminant.
    ///
    /// The escape hatch is deliberate: a kind we have no name for is still an OBSERVATION, and
    /// dropping it would be a pre-classification. It stays addressable until someone names it.
    Other(i32),
}

impl ScipKind {
    /// Decode a raw SCIP `symbol_information::Kind` discriminant, keeping unknowns addressable.
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            4 => Self::Attribute,
            8 => Self::Constant,
            11 => Self::Enum,
            12 => Self::EnumMember,
            15 => Self::Field,
            17 => Self::Function,
            25 => Self::Macro,
            26 => Self::Method,
            29 => Self::Module,
            49 => Self::Struct,
            53 => Self::Trait,
            55 => Self::TypeAlias,
            58 => Self::TypeParameter,
            61 => Self::Variable,
            70 => Self::TraitMethod,
            80 => Self::StaticMethod,
            other => Self::Other(other),
        }
    }

    /// Whether a "call" to this symbol is really a CONSTRUCTION (enum variant / tuple struct).
    ///
    /// Reported, never enforced at ingest: the substrate holds the observation, and an organ decides
    /// what to do with an edge whose destination is a constructor.
    #[must_use]
    pub const fn is_constructor(self) -> bool {
        matches!(self, Self::EnumMember | Self::Struct | Self::Enum)
    }
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
    ///
    /// **A PROJECTION of [`Self::witnesses`], not an independent fact** — derive it with
    /// [`Self::new`] rather than stamping it, so the tier can never disagree with the evidence.
    pub tier: ResolutionTier,
    /// **Which instruments witnessed this edge** — the evidence the substrate HOLDS.
    ///
    /// The tier above answers *"how well-resolved?"*; this answers *"who saw it, and who didn't?"* —
    /// and those are different facts. `{Syn, ScipResolved}` (both instruments agree) and
    /// `{ScipResolved}` (resolution saw what parsing cannot) both project to `Resolved`, yet they
    /// are **not the same evidence**: the first is corroborated across independent instruments, the
    /// second is single-sourced. Stamping only the tier annihilates that distinction at the door —
    /// a pre-classification the substrate is not entitled to make (hold-all: sensors hold, organs
    /// project).
    ///
    /// This does NOT license grade-lifting: *lower-never-corroborates-up* still governs what an
    /// organ may CONCLUDE from a witness set (see [`crate::read::tier::corroborate`]). The substrate
    /// holds the pattern; the law constrains the projection.
    pub witnesses: BTreeSet<WitnessSource>,
}

/// An instrument that can witness an edge — WHO saw it, kept distinct from how well-resolved it is.
///
/// Each instrument has its own blind spots, and the blind spots are the point: `syn` parses every
/// `cfg` branch but cannot resolve dispatch; a SCIP index resolves dispatch but describes exactly
/// one configuration. Their disagreement is a signal neither produces alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WitnessSource {
    /// The `syn` syntactic walk — parses EVERY `cfg` branch; cannot resolve dispatch or trait
    /// methods, and cannot distinguish a call from a tuple-struct/enum-variant construction.
    Syn,
    /// SCIP resolved the call cleanly — symbol-level identity, one configuration.
    ScipResolved,
    /// SCIP saw the reference but could not trust the enclosure (a `Generated`/macro-expansion
    /// site) — form-only, never `presents`-grade.
    ScipAmbiguous,
}

impl EdgeFact {
    /// Build an edge from its witness set, **deriving** the tier rather than stamping it.
    ///
    /// The projection: any `ScipResolved` witness earns `Resolved`; otherwise the edge is
    /// `Syntactic` — a macro-generated SCIP enclosure (`ScipAmbiguous`) is form-only and degrades,
    /// and `syn` alone is syntactic by construction. An empty witness set is `Syntactic` (the floor:
    /// an edge nobody witnessed cannot claim resolution).
    #[must_use]
    pub fn new(
        src: StromaNodeId,
        dst: StromaNodeId,
        kind: EdgeKind,
        witnesses: BTreeSet<WitnessSource>,
    ) -> Self {
        let tier = if witnesses.contains(&WitnessSource::ScipResolved) {
            ResolutionTier::Resolved
        } else {
            ResolutionTier::Syntactic
        };
        Self {
            src,
            dst,
            kind,
            tier,
            witnesses,
        }
    }

    /// Whether independent instruments agree on this edge (`syn` AND SCIP both saw it).
    ///
    /// Reported, never auto-promoted: *lower-never-corroborates-up* governs whether an organ may
    /// lift a grade from this. The substrate only says that it is so.
    #[must_use]
    pub fn is_cross_witnessed(&self) -> bool {
        self.witnesses.contains(&WitnessSource::Syn)
            && (self.witnesses.contains(&WitnessSource::ScipResolved)
                || self.witnesses.contains(&WitnessSource::ScipAmbiguous))
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::cfg::CfgSet;
    use crate::node::digest::IdentityDigest;
    use crate::node::id::FqPath;

    fn node(name: &str) -> StromaNodeId {
        StromaNodeId {
            fq_path: FqPath {
                path: name.to_string(),
                tier: ResolutionTier::Syntactic,
            },
            identity_digest: IdentityDigest::of_tokens(name.as_bytes()),
            cfg_set: CfgSet::default(),
        }
    }

    fn with(w: &[WitnessSource]) -> EdgeFact {
        EdgeFact::new(
            node("a"),
            node("b"),
            EdgeKind::Call,
            w.iter().copied().collect(),
        )
    }

    /// Both instruments agreeing and SCIP alone PROJECT to the same tier — but the substrate must
    /// still hold them as DIFFERENT evidence. Stamping only the tier annihilates the distinction at
    /// the door; that is the pre-classification hold-all forbids.
    #[test]
    fn agreement_is_distinguishable_from_single_sourcing() {
        let both = with(&[WitnessSource::Syn, WitnessSource::ScipResolved]);
        let scip_only = with(&[WitnessSource::ScipResolved]);

        assert_eq!(both.tier, scip_only.tier, "both project to the same tier");
        assert_ne!(
            both.witnesses, scip_only.witnesses,
            "yet the held evidence differs — the agreement signal survives ingest"
        );
        assert!(both.is_cross_witnessed());
        assert!(!scip_only.is_cross_witnessed());
    }

    /// The tier is DERIVED from the witness set, never independently stamped: a clean SCIP
    /// resolution earns `Resolved`; syn alone and a macro-generated (form-only) SCIP enclosure do
    /// not. `{Syn}` and `{ScipAmbiguous}` share a tier but remain distinct evidence.
    #[test]
    fn tier_is_a_projection_of_the_witness_set() {
        assert_eq!(
            with(&[WitnessSource::ScipResolved]).tier,
            ResolutionTier::Resolved
        );
        assert_eq!(with(&[WitnessSource::Syn]).tier, ResolutionTier::Syntactic);
        assert_eq!(
            with(&[WitnessSource::ScipAmbiguous]).tier,
            ResolutionTier::Syntactic,
            "a macro-site's guessed enclosure is form-only — it degrades by construction"
        );
        assert_ne!(
            with(&[WitnessSource::Syn]).witnesses,
            with(&[WitnessSource::ScipAmbiguous]).witnesses,
            "same tier, different evidence — held, not collapsed"
        );
    }

    /// An edge nobody witnessed cannot claim resolution — the honest floor.
    #[test]
    fn an_unwitnessed_edge_sits_at_the_floor() {
        let e = with(&[]);
        assert_eq!(e.tier, ResolutionTier::Syntactic);
        assert!(!e.is_cross_witnessed());
    }

    /// SCIP's kind carries distinctions `ItemTarget` structurally cannot express — which is the
    /// whole reason it is held un-mapped. If these ever collapsed onto one variant, the second
    /// instrument would stop being worth having.
    #[test]
    fn scip_kind_keeps_distinctions_item_target_cannot_express() {
        // EnumMember vs Struct: BOTH are constructions, but they are not the same thing.
        assert_ne!(ScipKind::from_raw(12), ScipKind::from_raw(49));
        // Method vs TraitMethod: dispatch-relevant, and `ItemTarget::Fn` erases it.
        assert_ne!(ScipKind::from_raw(26), ScipKind::from_raw(70));
        assert_eq!(ScipKind::from_raw(17), ScipKind::Function);
    }

    /// A "call" to a constructor is a CONSTRUCTION — the distinction `syn` structurally cannot make
    /// (`Some(x)` and `foo(x)` are the same shape), and the one SCIP hands us for free.
    #[test]
    fn constructors_are_recognizable_where_syn_is_blind() {
        assert!(
            ScipKind::EnumMember.is_constructor(),
            "Some(x) is a construction"
        );
        assert!(
            ScipKind::Struct.is_constructor(),
            "a tuple-struct call constructs"
        );
        assert!(!ScipKind::Function.is_constructor());
        assert!(!ScipKind::Method.is_constructor());
    }

    /// An unrecognized kind stays ADDRESSABLE rather than being dropped: a kind we cannot name is
    /// still an observation, and discarding it would be a pre-classification (hold-all).
    #[test]
    fn an_unknown_kind_is_held_not_dropped() {
        assert_eq!(ScipKind::from_raw(9999), ScipKind::Other(9999));
        assert_ne!(ScipKind::from_raw(9999), ScipKind::from_raw(9998));
    }
}
