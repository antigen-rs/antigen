//! The point-wise reachability queries.
//!
//! Each query runs against an already-published snapshot, not an online push: the reachability closure
//! is built once from the base's [`crate::base::facts::EdgeFacts`] (retrieved through the pinned
//! [`SnapshotHandle`]) and each query reads point-wise from it. Every answer is tier-capped
//! ([`TieredAnswer`]) — its tier is the MINIMUM edge-tier traversed (R6 tier-honesty), except
//! [`provenance_of`], which is defined as the MAXIMUM (lattice-JOIN) tier on any path.
//!
//! When the base has no published edge relation (a fresh `StromaDb`) or the queried node is absent, the
//! answer is an empty value at the `Syntactic` floor — the honest "nothing reached" result, never a
//! panic.

use std::collections::{BTreeSet, VecDeque};

use super::answer::{SnapshotHandle, TieredAnswer};
use crate::engine::{ReachabilityRelation, condense};
use crate::node::StromaNodeId;
use crate::read::ResolutionTier;

/// Build the reachability relation from the snapshot's published edge relation (R9: the edges are the
/// authored input; this relation is recomputable output). Empty when nothing has been published.
fn relation(snap: &SnapshotHandle) -> ReachabilityRelation {
    let db = snap.db();
    let edges = db.edge_facts().map_or_else(Vec::new, |ef| ef.edges(db));
    ReachabilityRelation::from_edges(&edges)
}

/// The empty, tier-floor answer (nothing reached / node absent / base unpopulated).
const fn empty() -> TieredAnswer<Vec<StromaNodeId>> {
    TieredAnswer::answered(Vec::new(), ResolutionTier::Syntactic)
}

/// Backward reachability — "what reaches this node?" (single-source-backward detection).
///
/// The set of nodes with a directed path INTO `node`, at the minimum edge-tier traversed (R6). A path
/// crossing any `Syntactic` edge caps the answer at `Dread`-grade.
#[must_use]
pub fn reachable_from(
    snap: &SnapshotHandle,
    node: &StromaNodeId,
) -> TieredAnswer<Vec<StromaNodeId>> {
    let rel = relation(snap);
    let Some(start) = rel.index_of(node) else {
        return empty();
    };
    let reached = rel.reach(start, false);
    let tier = reached.min_tier.unwrap_or(ResolutionTier::Syntactic);
    TieredAnswer::answered(rel.ids_of(&reached.nodes), tier)
}

/// Field-at — "what danger-field reaches this node?" (single-source-forward from a danger marker).
///
/// The set of nodes reachable FROM `node`, at the minimum edge-tier traversed (R6). DESIGN-tier note:
/// the danger-marker semantics (ADR-068) is deferred; the MVP treats the argument node itself as the
/// danger source and returns its forward field.
#[must_use]
pub fn field_at(snap: &SnapshotHandle, node: &StromaNodeId) -> TieredAnswer<Vec<StromaNodeId>> {
    let rel = relation(snap);
    let Some(start) = rel.index_of(node) else {
        return empty();
    };
    let reached = rel.reach(start, true);
    let tier = reached.min_tier.unwrap_or(ResolutionTier::Syntactic);
    TieredAnswer::answered(rel.ids_of(&reached.nodes), tier)
}

/// Provenance-of — "how well-resolved is this node's forward field?" (per-node lattice-JOIN).
///
/// The provenance semiring (`max`-JOIN over [`ResolutionTier`], R2) — the HIGHEST tier on any path out
/// of `node`. Unlike the detection queries, its answer tier is the MAXIMUM edge-tier (the best evidence
/// available), so a node whose immediate edges are `Resolved` earns a `Presents`-grade answer even when
/// deeper edges are syntactic. Value = the forward-reachable set the provenance was computed over.
#[must_use]
pub fn provenance_of(
    snap: &SnapshotHandle,
    node: &StromaNodeId,
) -> TieredAnswer<Vec<StromaNodeId>> {
    let rel = relation(snap);
    let Some(start) = rel.index_of(node) else {
        return empty();
    };
    let reached = rel.reach(start, true);
    let tier = reached.max_tier.unwrap_or(ResolutionTier::Syntactic);
    TieredAnswer::answered(rel.ids_of(&reached.nodes), tier)
}

/// Blast-from — "what is downstream-affected?" (source-constrained, offline).
///
/// The downstream set of `node` computed over the SCC-CONDENSED DAG (R4): SCCs are collapsed to
/// super-nodes (Tarjan), then the forward super-node reachability is expanded back to member nodes. The
/// condensation is why this terminates on a cyclic graph where the counting/blast semiring would
/// otherwise diverge (R3). Tier = minimum edge-tier over the forward-reachable sub-graph (R6).
#[must_use]
pub fn blast_from(snap: &SnapshotHandle, node: &StromaNodeId) -> TieredAnswer<Vec<StromaNodeId>> {
    let rel = relation(snap);
    let Some(start) = rel.index_of(node) else {
        return empty();
    };
    let cond = condense(&rel);
    let start_scc = cond.scc_of[start];

    // Forward reachability over the condensed DAG (acyclic ⇒ terminates), expanded to member nodes.
    let mut downstream: BTreeSet<usize> = BTreeSet::new();
    let mut seen_scc = vec![false; cond.dag_adj.len()];
    let mut queue = VecDeque::new();
    seen_scc[start_scc] = true;
    queue.push_back(start_scc);
    while let Some(s) = queue.pop_front() {
        for &t in &cond.dag_adj[s] {
            for &member in &cond.sccs[t] {
                downstream.insert(member);
            }
            if !seen_scc[t] {
                seen_scc[t] = true;
                queue.push_back(t);
            }
        }
    }
    // Cycle-mates in the start node's own SCC are downstream too (excluding the start node itself).
    for &member in &cond.sccs[start_scc] {
        if member != start {
            downstream.insert(member);
        }
    }

    let tier = rel
        .reach(start, true)
        .min_tier
        .unwrap_or(ResolutionTier::Syntactic);
    TieredAnswer::answered(rel.ids_of(&downstream), tier)
}
