//! SCIP ingestion — the `EdgeReconstruction` type (tier-honesty in the contract).
//!
//! The honesty rule: a macro-call-site MUST be `Ambiguous`/`Unreconstructible`, NEVER
//! silently-`Resolved`; a plain call-site resolves cleanly. Because the type distinguishes these
//! outcomes, the contract cannot represent a silently-resolved macro edge — the tier-honesty is in
//! the type, not left to the ingestion code to remember.

/// The reconstruction outcome of a single SCIP edge — the tier-honesty in the contract.
#[derive(Debug, Clone)]
pub enum EdgeReconstruction {
    /// Cleanly resolved (a plain call-site). Carries `presents`-grade eligibility.
    Resolved(ResolvedEdge),
    /// Ambiguous — multiple candidates (e.g. a macro expansion site). NEVER `presents`-grade.
    Ambiguous(Vec<ResolvedEdge>),
    /// Unreconstructible — the symbol could not be resolved. `dread`-grade floor at most.
    Unreconstructible,
}

/// A resolved SCIP edge — its src/dst SCIP symbols and edge-kind.
///
/// Defined so the reconstruction contract is tier-honest independent of the ingestion code that
/// produces the edges.
#[derive(Debug, Clone)]
pub struct ResolvedEdge {
    /// The source SCIP symbol string (becomes the src node's resolved [`crate::node::id::FqPath`]).
    pub src_symbol: String,
    /// The destination SCIP symbol string (becomes the dst node's resolved path).
    pub dst_symbol: String,
    /// The reconstructed edge-kind (call / import / type-use / …).
    pub kind: crate::base::facts::EdgeKind,
}

/// Ingest SCIP edges into the base (the resolved feeder).
///
/// # Panics
///
/// This function panics when called: it does not run SCIP to produce the reconstruction outcomes.
/// The [`EdgeReconstruction`] outcome type is defined so the reconstruction contract is tier-honest
/// independent of the run that produces those outcomes.
pub fn ingest_scip(_scip_index_path: &std::path::Path) -> Vec<EdgeReconstruction> {
    todo!("run SCIP to produce the EdgeReconstruction outcomes")
}

/// A node id built from a SCIP symbol at a given tier (the resolved feeder's node-construction rule).
///
/// The symbol string becomes the [`crate::node::id::FqPath`] (carrying its tier); the identity digest
/// is BLAKE3 over the symbol bytes; cfg-set is empty (SCIP symbols are cfg-resolved upstream). A single
/// deterministic function so the same symbol always yields the same node identity.
fn node_from_symbol(symbol: &str, tier: crate::read::ResolutionTier) -> crate::node::StromaNodeId {
    crate::node::StromaNodeId {
        fq_path: crate::node::id::FqPath {
            path: symbol.to_string(),
            tier,
        },
        identity_digest: crate::node::digest::IdentityDigest::of_tokens(symbol.as_bytes()),
        cfg_set: crate::node::cfg::CfgSet::default(),
    }
}

/// Populate the base's [`crate::base::facts::EdgeFacts`] from a batch of SCIP reconstruction outcomes
/// (R9 — the resolved-feeder population path), then publish the input handle onto the db.
///
/// The tier-honesty rule is applied here, driven by the reconstruction outcome type (never guessed):
/// - [`EdgeReconstruction::Resolved`] → a `Resolved`-tier edge (presents-grade eligible);
/// - [`EdgeReconstruction::Ambiguous`] → each candidate DEGRADED to `Syntactic` tier (a macro-site's
///   guessed enclosure can never corroborate up — it is form-only);
/// - [`EdgeReconstruction::Unreconstructible`] → SKIPPED (no edge; the symbol could not be resolved).
pub fn populate_from_reconstructions(db: &mut crate::db::StromaDb, recons: &[EdgeReconstruction]) {
    use crate::base::facts::EdgeFact;
    use crate::read::ResolutionTier;

    let mut edges: Vec<EdgeFact> = Vec::new();
    for recon in recons {
        match recon {
            EdgeReconstruction::Resolved(edge) => edges.push(EdgeFact {
                src: node_from_symbol(&edge.src_symbol, ResolutionTier::Resolved),
                dst: node_from_symbol(&edge.dst_symbol, ResolutionTier::Resolved),
                kind: edge.kind,
                tier: ResolutionTier::Resolved,
            }),
            EdgeReconstruction::Ambiguous(candidates) => {
                for edge in candidates {
                    edges.push(EdgeFact {
                        src: node_from_symbol(&edge.src_symbol, ResolutionTier::Syntactic),
                        dst: node_from_symbol(&edge.dst_symbol, ResolutionTier::Syntactic),
                        kind: edge.kind,
                        tier: ResolutionTier::Syntactic, // degraded — never presents-grade
                    });
                }
            },
            EdgeReconstruction::Unreconstructible => {}, // no edge — the symbol did not resolve
        }
    }

    let edge_facts = crate::base::facts::EdgeFacts::new(&*db, edges);
    db.set_edge_facts(edge_facts);
}
