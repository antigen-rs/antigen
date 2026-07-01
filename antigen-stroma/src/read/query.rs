//! The point-wise reachability queries.
//!
//! Each query runs against an already-published snapshot, not an online push: the all-pairs closure
//! runs once as the salsa-gated batch, and these read point-wise from it. Every answer is tier-capped
//! ([`TieredAnswer`]) and bound to the one revision the [`SnapshotHandle`] pins.
//!
//! These signatures are the query contract every organ compiles against. Each query panics when
//! called (see the per-function `# Panics` note): the point-wise answer is read from a reachability
//! closure over the base's [`crate::base::facts::EdgeFacts`], which the query does not compute.

use super::answer::{SnapshotHandle, TieredAnswer};
use crate::node::StromaNodeId;

/// Backward reachability — "what reaches this node?" (single-source-backward detection).
///
/// # Panics
///
/// This function panics when called: it does not compute the backward-reachability closure its answer
/// is read from. The signature is the contract organs compile against.
#[must_use]
pub fn reachable_from(
    _snap: &SnapshotHandle,
    _node: &StromaNodeId,
) -> TieredAnswer<Vec<StromaNodeId>> {
    todo!("single-source-backward over the memoized closure")
}

/// Field-at — "what danger-field reaches this node?" (single-source-forward from a danger marker).
///
/// # Panics
///
/// This function panics when called: it does not compute the forward-field closure its answer is read
/// from. The signature is the contract organs compile against.
#[must_use]
pub fn field_at(_snap: &SnapshotHandle, _node: &StromaNodeId) -> TieredAnswer<Vec<StromaNodeId>> {
    todo!("single-source-forward from danger marker")
}

/// Provenance-of — "how was this edge reconstructed, at what tier?" (per-edge).
///
/// # Panics
///
/// This function panics when called: it does not compute the per-edge provenance semiring its answer
/// is read from. The signature is the contract organs compile against.
#[must_use]
pub fn provenance_of(
    _snap: &SnapshotHandle,
    _node: &StromaNodeId,
) -> TieredAnswer<Vec<StromaNodeId>> {
    todo!("per-edge provenance (lattice-JOIN semiring)")
}

/// Blast-from — "what is downstream-affected?" (source-constrained, offline).
///
/// # Panics
///
/// This function panics when called: it does not compute the source-constrained blast closure its
/// answer is read from (that closure needs SCC-condensation for the cyclic blast tier). The signature
/// is the contract organs compile against.
#[must_use]
pub fn blast_from(_snap: &SnapshotHandle, _node: &StromaNodeId) -> TieredAnswer<Vec<StromaNodeId>> {
    todo!("source-constrained blast (counting semiring, post-condensation)")
}
