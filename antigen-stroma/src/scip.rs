//! STEP 6 — SCIP ingestion. The frame ships the `EdgeReconstruction` TYPE (tier-honesty in the
//! contract); the scip-run WIRING is engine-fill.
//!
//! The honesty rule (born-red ATK): a macro-call-site MUST be `Ambiguous`/`Unreconstructible`,
//! NEVER silently-Resolved. A plain call-site MUST resolve cleanly (the negative control). Shipping
//! the TYPE at frame-time means the contract cannot represent a silently-resolved macro edge.

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

/// A resolved SCIP edge. **STUB — fill (frame epoch type; population is engine):** the src/dst SCIP
/// symbols + edge-kind.
#[derive(Debug, Clone)]
pub struct ResolvedEdge {
    // STUB: src_symbol, dst_symbol, kind
}

/// Ingest SCIP edges into the base (the resolved feeder). **STUB body — ENGINE epoch** (the scip-run
/// wiring). The frame defines the outcome TYPE so the contract is tier-honest; the engine wires the run.
pub fn ingest_scip(_scip_index_path: &std::path::Path) -> Vec<EdgeReconstruction> {
    todo!("engine epoch: wire the SCIP run; frame ships the EdgeReconstruction type only")
}
