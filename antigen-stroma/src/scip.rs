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
    // src_symbol / dst_symbol / kind.
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
