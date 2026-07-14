//! Minimum vertex-cut over the stroma reachability graph (Menger / max-flow reduction).
//!
//! Answers: **"what is the smallest set of nodes whose removal disconnects every directed path from
//! the source set to the target node?"** -- the chokepoint finder that the blast-complete-answer and
//! root-cause charters name as their tier-2 topology organ.
//!
//! # Reduction -- vertex-split to edge-capacity max-flow
//!
//! The standard Menger reduction:
//! - Each node `v` is split into `v_in` (index `2v`) and `v_out` (index `2v+1`) with a
//!   **unit-capacity** edge `v_in -> v_out`.  Source and target super-nodes get **infinite** capacity
//!   instead so they are never themselves cut (we cut the *interior* bottleneck, not the endpoints).
//! - Each original edge `u -> w` becomes `u_out -> w_in` with **infinite** capacity (edge capacity
//!   is not what we bound; node capacity is).
//! - A synthetic super-source `2n` feeds every source node's `in`-half with an infinite-capacity
//!   edge; a synthetic super-sink `2n+1` is fed by the target node's `out`-half with infinite
//!   capacity.
//! - Max-flow (Edmonds-Karp -- BFS augmenting paths) on the resulting edge-capacity network equals
//!   the min vertex-cut by Menger's theorem.
//! - The cut vertex set is recovered from the residual graph: BFS from the super-source over edges
//!   with remaining capacity; any `v_in` reachable but whose `v_out` is NOT (the saturated split
//!   edge) is an interior cut vertex -- provided it is not itself a source or target.
//!
//! # Complexity
//!
//! O(V * E) per Edmonds-Karp on the expanded graph -- adequate for call-graph scale (thousands of
//! nodes, tens of thousands of edges).  Correctness over cleverness is the design mandate.

// Large but safe: the maximum augmenting bottleneck on any unit-capacity interior graph is bounded
// by the max-flow, so INF is only consumed by source/target/routing edges that are never the
// bottleneck.  INF/2 keeps subtraction safely away from u64::MAX.
const INF: u64 = u64::MAX / 4;

use std::collections::{BTreeSet, VecDeque};

use super::closure::ReachabilityRelation;
use crate::node::StromaNodeId;

/// The result of a minimum vertex-cut query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinVertexCut {
    /// The minimum set of nodes (by dense index in the originating relation) whose removal
    /// disconnects every path from the source set to the target.
    ///
    /// Empty when:
    /// - any source IS the target (already at the target -- trivially connected by zero hops),
    /// - sources and target are disconnected (no cut needed; no path exists to sever),
    /// - or the graph has no nodes.
    pub cut_nodes: BTreeSet<usize>,
    /// The minimum cut size (= `cut_nodes.len()`), for convenience.
    pub cut_size: usize,
}

impl MinVertexCut {
    /// Lift the cut indices back to owned [`StromaNodeId`]s.
    #[must_use]
    pub fn ids(&self, rel: &ReachabilityRelation) -> Vec<StromaNodeId> {
        self.cut_nodes
            .iter()
            .map(|&i| rel.node(i).clone())
            .collect()
    }
}

/// Compute the min vertex-cut separating `sources` from `target` in the forward direction.
///
/// Sources and target are given as dense indices into `rel`.  The returned [`MinVertexCut`] names
/// the interior vertices that must be removed to sever every forward path.
///
/// If any source equals `target`, the cut is **empty** (the target is already a source; no finite
/// cut severs a zero-hop path).
// One cohesive max-flow unit: vertex-split graph construction, Edmonds-Karp augmentation, and
// residual-side cut recovery share the residual/adjacency state and read most clearly together.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn min_vertex_cut(
    rel: &ReachabilityRelation,
    sources: &[usize],
    target: usize,
) -> MinVertexCut {
    // Degenerate: any source == target => empty cut.
    if sources.contains(&target) {
        return MinVertexCut {
            cut_nodes: BTreeSet::new(),
            cut_size: 0,
        };
    }

    let n = rel.len();
    if n == 0 || sources.is_empty() {
        return MinVertexCut {
            cut_nodes: BTreeSet::new(),
            cut_size: 0,
        };
    }

    // ---- Build the flow network -----------------------------------------------
    //
    // Node layout in the expanded graph (2n + 2 nodes):
    //   v_in  = 2*v
    //   v_out = 2*v + 1
    //   super_source = 2*n
    //   super_sink   = 2*n + 1
    //
    // Edges are stored in pairs: forward edge at even index fwd, reverse edge at fwd^1.
    // We track RESIDUAL CAPACITY directly (residual[fwd] + residual[rev] is invariant under
    // augmentation), so no separate flow array is needed.
    let super_src = 2 * n;
    let super_snk = 2 * n + 1;
    let total = 2 * n + 2;

    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); total];
    let mut edge_to: Vec<usize> = Vec::new();
    let mut residual: Vec<u64> = Vec::new();

    // Push a directed edge (from -> to) with given capacity, plus its zero-capacity reverse.
    let push_edge = |adj: &mut Vec<Vec<usize>>,
                     edge_to: &mut Vec<usize>,
                     residual: &mut Vec<u64>,
                     from: usize,
                     to: usize,
                     cap: u64| {
        let fwd = residual.len(); // always even
        adj[from].push(fwd);
        adj[to].push(fwd ^ 1); // reverse edge at fwd^1
        edge_to.push(to);
        residual.push(cap);
        edge_to.push(from); // reverse edge
        residual.push(0); // reverse starts with zero residual (no flow yet to cancel)
    };

    let source_set: BTreeSet<usize> = sources.iter().copied().collect();

    // Node-split edges: v_in -> v_out.
    //   - Sources and target get INF (they are never the bottleneck to cut).
    //   - All other interior nodes get capacity 1 (each can pass at most one unit of flow).
    for v in 0..n {
        let cap = if source_set.contains(&v) || v == target {
            INF
        } else {
            1
        };
        push_edge(&mut adj, &mut edge_to, &mut residual, 2 * v, 2 * v + 1, cap);
    }

    // Original graph edges: u_out -> w_in with INF capacity (routing edges, not the bottleneck).
    for u in 0..n {
        for &(w, _tier) in rel.forward_adj(u) {
            push_edge(&mut adj, &mut edge_to, &mut residual, 2 * u + 1, 2 * w, INF);
        }
    }

    // Super-source -> each source's v_in (INF).
    for &s in &source_set {
        push_edge(&mut adj, &mut edge_to, &mut residual, super_src, 2 * s, INF);
    }

    // Target's v_out -> super-sink (INF).
    push_edge(
        &mut adj,
        &mut edge_to,
        &mut residual,
        2 * target + 1,
        super_snk,
        INF,
    );

    // ---- Edmonds-Karp (BFS augmenting paths on residual graph) ----------------

    loop {
        // BFS: find a shortest augmenting path from super_src to super_snk.
        let mut parent_edge: Vec<Option<usize>> = vec![None; total];
        parent_edge[super_src] = Some(usize::MAX); // sentinel: super_src has no parent edge
        let mut queue = VecDeque::new();
        queue.push_back(super_src);

        'bfs: while let Some(u) = queue.pop_front() {
            for &eidx in &adj[u] {
                let v = edge_to[eidx];
                if parent_edge[v].is_none() && residual[eidx] > 0 {
                    parent_edge[v] = Some(eidx);
                    if v == super_snk {
                        break 'bfs;
                    }
                    queue.push_back(v);
                }
            }
        }

        // No augmenting path -- max-flow reached.
        if parent_edge[super_snk].is_none() {
            break;
        }

        // Find the bottleneck along the augmenting path.
        let mut bottleneck = u64::MAX;
        let mut cur = super_snk;
        while cur != super_src {
            let eidx = parent_edge[cur].unwrap();
            bottleneck = bottleneck.min(residual[eidx]);
            cur = edge_to[eidx ^ 1]; // reverse edge points back to the parent node
        }

        // Augment: decrease forward residual, increase reverse residual.
        let mut cur = super_snk;
        while cur != super_src {
            let eidx = parent_edge[cur].unwrap();
            residual[eidx] -= bottleneck; // forward edge: capacity consumed
            residual[eidx ^ 1] += bottleneck; // reverse edge: cancellation capacity added
            cur = edge_to[eidx ^ 1];
        }
    }

    // ---- Recover the cut set from the residual graph --------------------------
    //
    // BFS from super_src over edges with residual > 0.
    let mut reachable = vec![false; total];
    reachable[super_src] = true;
    let mut queue = VecDeque::new();
    queue.push_back(super_src);
    while let Some(u) = queue.pop_front() {
        for &eidx in &adj[u] {
            let v = edge_to[eidx];
            if !reachable[v] && residual[eidx] > 0 {
                reachable[v] = true;
                queue.push_back(v);
            }
        }
    }

    // Cut vertices: interior nodes v where v_in is reachable but v_out is NOT (the unit-capacity
    // split edge is saturated).  Sources and target are excluded (their splits have INF capacity).
    let mut cut_nodes = BTreeSet::new();
    for v in 0..n {
        if !source_set.contains(&v) && v != target && reachable[2 * v] && !reachable[2 * v + 1] {
            cut_nodes.insert(v);
        }
    }

    let cut_size = cut_nodes.len();
    MinVertexCut {
        cut_nodes,
        cut_size,
    }
}

/// Convenience wrapper: single-source min vertex-cut.
#[must_use]
pub fn min_vertex_cut_single(
    rel: &ReachabilityRelation,
    source: usize,
    target: usize,
) -> MinVertexCut {
    min_vertex_cut(rel, &[source], target)
}

/// Verify that removing `removed` actually disconnects every source from target.
///
/// Returns `true` when no forward path from any source to target exists with `removed` nodes
/// deleted.  Used in tests as a post-condition assertion.
#[must_use]
pub fn cut_disconnects(
    rel: &ReachabilityRelation,
    sources: &[usize],
    target: usize,
    removed: &BTreeSet<usize>,
) -> bool {
    let n = rel.len();
    let mut seen = vec![false; n];
    let mut queue = VecDeque::new();
    for &s in sources {
        if !removed.contains(&s) && !seen[s] {
            seen[s] = true;
            queue.push_back(s);
        }
    }
    while let Some(u) = queue.pop_front() {
        if u == target {
            return false; // target reached -- cut did NOT disconnect
        }
        for &(v, _tier) in rel.forward_adj(u) {
            if !removed.contains(&v) && !seen[v] {
                seen[v] = true;
                queue.push_back(v);
            }
        }
    }
    true // target never reached -- cut DID disconnect (or no path existed)
}

// ---- Test helpers (cfg-gated) -----------------------------------------------

/// Build a [`ReachabilityRelation`] from a simple name-pair adjacency list.
#[cfg(test)]
pub(crate) fn rel_from_edges(pairs: &[(&str, &str)]) -> ReachabilityRelation {
    use crate::base::facts::{EdgeFact, EdgeKind};
    use crate::node::cfg::CfgSet;
    use crate::node::digest::IdentityDigest;
    use crate::node::id::FqPath;
    use crate::read::ResolutionTier;

    let make_node = |name: &str| StromaNodeId {
        fq_path: FqPath {
            path: name.to_string(),
            tier: ResolutionTier::Resolved,
        },
        identity_digest: IdentityDigest::of_tokens(name.as_bytes()),
        cfg_set: CfgSet::default(),
    };
    let edges: Vec<EdgeFact> = pairs
        .iter()
        .map(|(s, d)| EdgeFact {
            src: make_node(s),
            dst: make_node(d),
            kind: EdgeKind::Call,
            tier: ResolutionTier::Resolved,
        })
        .collect();
    ReachabilityRelation::from_edges(&edges)
}

/// Look up a node's dense index by name; panics on miss (test-only).
#[cfg(test)]
pub(crate) fn idx(rel: &ReachabilityRelation, name: &str) -> usize {
    use crate::node::StromaNodeId;
    use crate::node::cfg::CfgSet;
    use crate::node::digest::IdentityDigest;
    use crate::node::id::FqPath;
    use crate::read::ResolutionTier;

    let id = StromaNodeId {
        fq_path: FqPath {
            path: name.to_string(),
            tier: ResolutionTier::Resolved,
        },
        identity_digest: IdentityDigest::of_tokens(name.as_bytes()),
        cfg_set: CfgSet::default(),
    };
    rel.index_of(&id)
        .unwrap_or_else(|| panic!("node '{name}' not in relation"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- BORN-RED TEETH (fail before impl, green after) -----------------------------------------

    /// ATK-MINCUT-001 * Chain -- the single bottleneck.
    ///
    /// A -> B -> C -> D.  One path; min vertex-cut is 1 (either B or C).
    #[test]
    fn chain_bottleneck_cut_size_is_one() {
        let rel = rel_from_edges(&[("A", "B"), ("B", "C"), ("C", "D")]);
        let src = idx(&rel, "A");
        let tgt = idx(&rel, "D");
        let cut = min_vertex_cut_single(&rel, src, tgt);
        assert_eq!(cut.cut_size, 1, "chain: exactly one interior bottleneck");
        assert!(
            cut_disconnects(&rel, &[src], tgt, &cut.cut_nodes),
            "chain: removing the cut vertex must disconnect A from D"
        );
        assert!(!cut.cut_nodes.contains(&src), "cut must not contain source");
        assert!(!cut.cut_nodes.contains(&tgt), "cut must not contain target");
    }

    /// ATK-MINCUT-002 * Diamond / two disjoint paths -- cut size equals path count.
    ///
    /// A->B->D and A->C->D: two vertex-disjoint paths; min vertex-cut is 2.
    #[test]
    fn diamond_two_disjoint_paths_cut_size_is_two() {
        let rel = rel_from_edges(&[("A", "B"), ("A", "C"), ("B", "D"), ("C", "D")]);
        let src = idx(&rel, "A");
        let tgt = idx(&rel, "D");
        let cut = min_vertex_cut_single(&rel, src, tgt);
        assert_eq!(
            cut.cut_size, 2,
            "diamond: two independent paths -> cut size 2"
        );
        assert!(
            cut_disconnects(&rel, &[src], tgt, &cut.cut_nodes),
            "diamond: removing both cut vertices must disconnect A from D"
        );
    }

    /// ATK-MINCUT-003 * Source equals target -- degenerate, empty cut.
    #[test]
    fn source_equals_target_yields_empty_cut() {
        let rel = rel_from_edges(&[("A", "B"), ("B", "C")]);
        let b = idx(&rel, "B");
        let cut = min_vertex_cut_single(&rel, b, b);
        assert_eq!(cut.cut_size, 0, "source == target: no cut needed");
        assert!(cut.cut_nodes.is_empty());
    }

    /// ATK-MINCUT-004 * Disconnected pair -- no path exists, empty cut.
    #[test]
    fn disconnected_pair_yields_empty_cut() {
        let rel = rel_from_edges(&[("A", "B"), ("C", "D")]);
        let src = idx(&rel, "A");
        let tgt = idx(&rel, "C");
        let cut = min_vertex_cut_single(&rel, src, tgt);
        assert_eq!(cut.cut_size, 0, "disconnected: no path -> empty cut");
        assert!(cut.cut_nodes.is_empty());
    }

    /// ATK-MINCUT-005 * Negative control: vertex-cut differs from edge-cut.
    ///
    /// Graph: S->A->B->T, S->C->B->T.
    ///
    /// Min edge-cut S->T: 2 (both source-side or both sink-side edges).
    /// Min vertex-cut S->T: 1 (removing B severs both paths -- B is the shared bottleneck).
    ///
    /// Asserting `cut_size` == 1 proves we built vertex-cut, not edge-cut.
    #[test]
    fn vertex_cut_differs_from_edge_cut_negative_control() {
        let rel = rel_from_edges(&[("S", "A"), ("S", "C"), ("A", "B"), ("C", "B"), ("B", "T")]);
        let src = idx(&rel, "S");
        let tgt = idx(&rel, "T");
        let cut = min_vertex_cut_single(&rel, src, tgt);

        assert_eq!(
            cut.cut_size, 1,
            "vertex-cut is 1 (B is the sole bottleneck); a naive edge-cut would be 2"
        );

        let b = idx(&rel, "B");
        assert!(cut.cut_nodes.contains(&b), "the cut vertex must be B");

        assert!(
            cut_disconnects(&rel, &[src], tgt, &cut.cut_nodes),
            "removing B must disconnect S from T"
        );

        // Confirm minimality: neither A nor C alone disconnects.
        let a = idx(&rel, "A");
        let c = idx(&rel, "C");
        let just_a: BTreeSet<usize> = std::iter::once(a).collect();
        let just_c: BTreeSet<usize> = std::iter::once(c).collect();
        assert!(
            !cut_disconnects(&rel, &[src], tgt, &just_a),
            "removing only A leaves the C->B->T path"
        );
        assert!(
            !cut_disconnects(&rel, &[src], tgt, &just_c),
            "removing only C leaves the A->B->T path"
        );
    }

    // ---- Additional correctness teeth ---------------------------------------------------

    /// A direct edge S->T has no interior nodes; cut is empty.
    #[test]
    fn direct_edge_has_zero_interior_cut() {
        let rel = rel_from_edges(&[("S", "T")]);
        let src = idx(&rel, "S");
        let tgt = idx(&rel, "T");
        let cut = min_vertex_cut_single(&rel, src, tgt);
        assert_eq!(cut.cut_size, 0, "no interior nodes on a direct edge");
    }

    /// A cycle (A->B->C, B->A) queried A->C: the only path A->B->C has one interior node B.
    #[test]
    fn cyclic_graph_cut_is_interior_bottleneck() {
        let rel = rel_from_edges(&[("A", "B"), ("B", "C"), ("B", "A")]);
        let src = idx(&rel, "A");
        let tgt = idx(&rel, "C");
        let cut = min_vertex_cut_single(&rel, src, tgt);
        assert_eq!(
            cut.cut_size, 1,
            "only path A->B->C has one interior node (B)"
        );
        assert!(
            cut_disconnects(&rel, &[src], tgt, &cut.cut_nodes),
            "removing B must disconnect A from C"
        );
    }

    /// Multi-source: {A, B} -> T via independent interior nodes X and Y.
    /// Both paths must be severed -> `cut_size` == 2.
    #[test]
    fn multi_source_cut_severs_all_paths() {
        let rel = rel_from_edges(&[("A", "X"), ("X", "T"), ("B", "Y"), ("Y", "T")]);
        let src_a = idx(&rel, "A");
        let src_b = idx(&rel, "B");
        let tgt = idx(&rel, "T");
        let cut = min_vertex_cut(&rel, &[src_a, src_b], tgt);
        assert_eq!(
            cut.cut_size, 2,
            "two independent streams -> `cut_size` == 2"
        );
        assert!(
            cut_disconnects(&rel, &[src_a, src_b], tgt, &cut.cut_nodes),
            "removing both cut vertices must disconnect both A and B from T"
        );
    }
}
