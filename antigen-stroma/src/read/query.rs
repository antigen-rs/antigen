//! STEP 6 — the point-wise query SIGNATURES (bodies are ENGINE epoch `todo!()`).
//!
//! NO consumer needs all-pairs-online-push (the genome's CLOSED engine-decider). The one all-pairs
//! closure runs ONCE as the salsa-gated batch; these are POINT-WISE queries against the already-
//! computed snapshot. The frame ships the signatures (tier-capped, snapshot-bound); the engine fills
//! the bodies over the ascent closure.

use super::answer::{SnapshotHandle, TieredAnswer};
use crate::node::StromaNodeId;

/// Backward reachability — "what reaches this node?" (detection: single-source-backward).
/// **STUB body — ENGINE epoch.**
#[must_use]
pub fn reachable_from(
    _snap: &SnapshotHandle,
    _node: &StromaNodeId,
) -> TieredAnswer<Vec<StromaNodeId>> {
    todo!("engine epoch: single-source-backward over the memoized closure")
}

/// Field-at — "what danger-field reaches this node?" (single-source-forward from a danger marker).
/// **STUB body — ENGINE epoch.**
#[must_use]
pub fn field_at(_snap: &SnapshotHandle, _node: &StromaNodeId) -> TieredAnswer<Vec<StromaNodeId>> {
    todo!("engine epoch: single-source-forward from danger marker")
}

/// Provenance-of — "how was this edge reconstructed, at what tier?" (per-edge).
/// **STUB body — ENGINE epoch.**
#[must_use]
pub fn provenance_of(
    _snap: &SnapshotHandle,
    _node: &StromaNodeId,
) -> TieredAnswer<Vec<StromaNodeId>> {
    todo!("engine epoch: per-edge provenance (lattice-JOIN semiring)")
}

/// Blast-from — "what is downstream-affected?" (source-constrained, offline).
/// **STUB body — ENGINE epoch (needs SCC-condensation for the cyclic blast tier).**
#[must_use]
pub fn blast_from(_snap: &SnapshotHandle, _node: &StromaNodeId) -> TieredAnswer<Vec<StromaNodeId>> {
    todo!("engine epoch: source-constrained blast (counting semiring, post-condensation)")
}
