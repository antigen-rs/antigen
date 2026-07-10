//! The reachability relation — forward/backward adjacency over [`EdgeFact`]s, plus the tier-tracking
//! traversal the point-wise queries read (ADR-068 R1).
//!
//! ONE relation, built once per query from the base's [`crate::base::facts::EdgeFacts`]. The four
//! semirings ([`super::semiring`]) fold over this same relation; the queries also traverse it directly
//! to recover the answer's tier (the MINIMUM edge-tier on the reached sub-graph — R6 tier-honesty —
//! and, for provenance, the MAXIMUM).

use std::collections::{BTreeSet, HashMap, VecDeque};

use crate::base::facts::EdgeFact;
use crate::node::StromaNodeId;
use crate::read::ResolutionTier;

/// The directed reachability relation over the node-set induced by a set of edges.
///
/// Nodes are indexed densely (`0..len`); `forward[i]` / `backward[i]` carry each incident edge's
/// destination/source index paired with the edge's [`ResolutionTier`]. Built from the base's authored
/// [`EdgeFact`]s (R9: the edges are input; this relation is recomputable output).
pub struct ReachabilityRelation {
    nodes: Vec<StromaNodeId>,
    index: HashMap<StromaNodeId, usize>,
    forward: Vec<Vec<(usize, ResolutionTier)>>,
    backward: Vec<Vec<(usize, ResolutionTier)>>,
}

/// The result of a tier-tracking traversal: the reached node-indices (excluding the start) plus the
/// min/max edge-tier over every edge traversed to reach them.
pub struct Reached {
    /// The reached node indices (the start node itself is excluded).
    pub nodes: BTreeSet<usize>,
    /// The minimum edge-tier traversed — the answer's tier-honest floor (R6). `None` iff nothing was
    /// reached (no edge traversed).
    pub min_tier: Option<ResolutionTier>,
    /// The maximum edge-tier traversed — the provenance lattice-JOIN (highest tier on any path).
    pub max_tier: Option<ResolutionTier>,
}

impl ReachabilityRelation {
    /// Build the relation from a set of authored edges, indexing every node that appears as a src/dst.
    #[must_use]
    pub fn from_edges(edges: &[EdgeFact]) -> Self {
        let mut index: HashMap<StromaNodeId, usize> = HashMap::new();
        let mut nodes: Vec<StromaNodeId> = Vec::new();
        let mut intern = |id: &StromaNodeId, nodes: &mut Vec<StromaNodeId>| -> usize {
            if let Some(&i) = index.get(id) {
                i
            } else {
                let i = nodes.len();
                index.insert(id.clone(), i);
                nodes.push(id.clone());
                i
            }
        };
        // First pass: assign indices (stable insertion order).
        let pairs: Vec<(usize, usize, ResolutionTier)> = edges
            .iter()
            .map(|e| {
                let s = intern(&e.src, &mut nodes);
                let d = intern(&e.dst, &mut nodes);
                (s, d, e.tier)
            })
            .collect();
        let n = nodes.len();
        let mut forward = vec![Vec::new(); n];
        let mut backward = vec![Vec::new(); n];
        for (s, d, tier) in pairs {
            forward[s].push((d, tier));
            backward[d].push((s, tier));
        }
        Self {
            nodes,
            index,
            forward,
            backward,
        }
    }

    /// The number of distinct nodes in the relation.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Whether the relation has no nodes.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// The node id at a dense index.
    #[must_use]
    pub fn node(&self, i: usize) -> &StromaNodeId {
        &self.nodes[i]
    }

    /// The dense index of a node id, if present in the relation.
    #[must_use]
    pub fn index_of(&self, id: &StromaNodeId) -> Option<usize> {
        self.index.get(id).copied()
    }

    /// The forward adjacency (outgoing edges) of a node index.
    #[must_use]
    pub fn forward_adj(&self, i: usize) -> &[(usize, ResolutionTier)] {
        &self.forward[i]
    }

    /// The backward adjacency (incoming edges) of a node index.
    #[must_use]
    pub fn backward_adj(&self, i: usize) -> &[(usize, ResolutionTier)] {
        &self.backward[i]
    }

    /// BFS from `start` along forward or backward adjacency, collecting reached nodes and the min/max
    /// edge-tier over the edges that FIRST-DISCOVER them. The visited-set makes this terminate on cyclic
    /// graphs.
    ///
    /// The tier fold runs only when an edge discovers a new node. Back-edges and self-loops (edges to an
    /// already-seen node) are cycle-closing artifacts, NOT reaching-path edges: folding their tier would
    /// let a lower-tier back-edge silently demote an answer whose actual reaching path is fully resolved
    /// (e.g. `A --Resolved--> B`, `B --Syntactic--> B` would report `B` at `Syntactic`, not `Resolved`).
    /// That is the R6 tier-honesty invariant, enforced at the traversal.
    #[must_use]
    pub fn reach(&self, start: usize, forward: bool) -> Reached {
        let mut seen = vec![false; self.nodes.len()];
        let mut queue = VecDeque::new();
        let mut out = BTreeSet::new();
        let mut min_tier: Option<ResolutionTier> = None;
        let mut max_tier: Option<ResolutionTier> = None;
        seen[start] = true;
        queue.push_back(start);
        while let Some(u) = queue.pop_front() {
            let adj = if forward {
                &self.forward[u]
            } else {
                &self.backward[u]
            };
            for &(v, tier) in adj {
                if !seen[v] {
                    seen[v] = true;
                    out.insert(v);
                    queue.push_back(v);
                    // Fold ONLY discovery edges — back-edges/self-loops are not on any reaching path.
                    min_tier = Some(min_tier.map_or(tier, |m| m.min(tier)));
                    max_tier = Some(max_tier.map_or(tier, |m| m.max(tier)));
                }
            }
        }
        // The start node can re-enter `out` via a cycle back to itself; exclude it (reachability of
        // OTHER nodes is the query's contract).
        out.remove(&start);
        Reached {
            nodes: out,
            min_tier,
            max_tier,
        }
    }

    /// Map a set of node indices back to owned node ids (query answer value).
    #[must_use]
    pub fn ids_of(&self, indices: &BTreeSet<usize>) -> Vec<StromaNodeId> {
        indices.iter().map(|&i| self.nodes[i].clone()).collect()
    }
}
