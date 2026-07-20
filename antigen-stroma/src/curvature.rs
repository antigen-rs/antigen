//! # `curvature` — discrete graph curvature: the TOPOLOGY-lever (candidate IH perversity).
//!
//! Every other clone in the spine (rough, soft, DS, FCA) is a CONTENT lever — it enriches how a node's
//! evidence is represented. This one is the **topology** lever: it reshapes *which edges may carry evidence*.
//! The claim (charter `curvature-as-perversity`): curvature-bounded transport across bottlenecks plays the
//! role of an intersection-homology **perversity** — a rule bounding how much may cross a singular stratum,
//! keyed to local structure.
//!
//! We use **augmented Forman curvature**, `F(u,v) = 4 − deg(u) − deg(v) + 3·m` where `m` is the number of
//! triangles on edge `(u,v)` (Attali et al., DKE 2025). It is purely combinatorial — it needs only the
//! 2-cells (triangles) the Hodge build already needs, so it adds NO dependency.
//!
//! - **positive** curvature ⇒ the edge sits in a dense neighbourhood with many alternative paths — the
//!   graph-geometric form of *corroboration* (`≥2 independent routes`), safe to diffuse along;
//! - **negative** curvature ⇒ a **bridge** between communities with few alternatives — the over-squashing
//!   locus, and the laundering path where one false verdict reaches a whole community uncorroborated.
//!
//! Nothing here is neural and nothing is learned: curvature is pure graph topology, which the stroma has.

// Degrees and triangle counts are graph-sized (≤ 10⁵ here), far from i64 wrap or f64 mantissa limits.
#![allow(clippy::cast_possible_wrap, clippy::cast_precision_loss)]

use std::collections::{BTreeSet, HashMap, HashSet};

/// A string-node indexing result: the id→index map plus the indexed edge pairs.
pub type IndexedPairs<'a> = (HashMap<&'a str, usize>, Vec<(usize, usize)>);

/// The `A⁺ / A⁻` curvature split: (at-or-above threshold, below threshold).
pub type EdgeSplit = (Vec<(usize, usize)>, Vec<(usize, usize)>);

/// An undirected simple graph over `0..n`, built from (possibly directed, possibly duplicated) edges.
#[derive(Debug, Clone, Default)]
pub struct UndirectedGraph {
    adj: Vec<BTreeSet<usize>>,
}

impl UndirectedGraph {
    /// Build from directed/duplicated pairs; self-loops are dropped (they carry no curvature).
    #[must_use]
    pub fn from_pairs(n: usize, pairs: &[(usize, usize)]) -> Self {
        let mut adj = vec![BTreeSet::new(); n];
        for &(u, v) in pairs {
            if u != v && u < n && v < n {
                adj[u].insert(v);
                adj[v].insert(u);
            }
        }
        Self { adj }
    }

    /// Number of nodes in the graph (the index space `0..n`).
    #[must_use]
    pub const fn node_count(&self) -> usize {
        self.adj.len()
    }

    /// Degree of a node in the undirected simple graph.
    #[must_use]
    pub fn degree(&self, u: usize) -> usize {
        self.adj.get(u).map_or(0, BTreeSet::len)
    }

    /// Every undirected edge once, as `(u, v)` with `u < v`.
    #[must_use]
    pub fn edges(&self) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        for (u, nbrs) in self.adj.iter().enumerate() {
            for &v in nbrs {
                if u < v {
                    out.push((u, v));
                }
            }
        }
        out
    }

    /// Number of triangles containing edge `(u, v)` — i.e. `|N(u) ∩ N(v)|`. These are exactly the 2-cells a
    /// clique-complex fill would add, and the redundant alternative paths that make an edge corroborated.
    #[must_use]
    pub fn triangles_on(&self, u: usize, v: usize) -> usize {
        match (self.adj.get(u), self.adj.get(v)) {
            (Some(a), Some(b)) => a.intersection(b).count(),
            _ => 0,
        }
    }

    /// Augmented Forman curvature of an edge: `4 − deg(u) − deg(v) + 3·triangles(u,v)`.
    #[must_use]
    pub fn forman(&self, u: usize, v: usize) -> i64 {
        4 - self.degree(u) as i64 - self.degree(v) as i64 + 3 * self.triangles_on(u, v) as i64
    }

    /// Curvature of every edge, as `((u, v), curvature)`.
    #[must_use]
    pub fn curvatures(&self) -> Vec<((usize, usize), i64)> {
        self.edges()
            .into_iter()
            .map(|(u, v)| ((u, v), self.forman(u, v)))
            .collect()
    }

    /// The `A⁺ / A⁻` split at a threshold (the paper splits at the mean): edges at-or-above vs below.
    /// `A⁺` is the "safe to diffuse" set; `A⁻` is the bridge/bottleneck set.
    #[must_use]
    pub fn split_at(&self, threshold: i64) -> EdgeSplit {
        let mut pos = Vec::new();
        let mut neg = Vec::new();
        for ((u, v), c) in self.curvatures() {
            if c >= threshold {
                pos.push((u, v));
            } else {
                neg.push((u, v));
            }
        }
        (pos, neg)
    }

    /// **Cut edges** (graph-theoretic bridges): edges whose removal disconnects their component. Tarjan's
    /// low-link algorithm, iterative (no recursion). This is an INDEPENDENT, exact notion of "bottleneck" —
    /// the non-circular ground truth the curvature claim is tested against.
    #[must_use]
    pub fn cut_edges(&self) -> HashSet<(usize, usize)> {
        let n = self.adj.len();
        let nbrs: Vec<Vec<usize>> = self
            .adj
            .iter()
            .map(|s| s.iter().copied().collect())
            .collect();
        let (mut disc, mut low) = (vec![usize::MAX; n], vec![usize::MAX; n]);
        let mut timer = 0usize;
        let mut bridges = HashSet::new();
        for start in 0..n {
            if disc[start] != usize::MAX {
                continue;
            }
            disc[start] = timer;
            low[start] = timer;
            timer += 1;
            // frames: (node, parent, next-child index into nbrs[node])
            let mut stack: Vec<(usize, usize, usize)> = vec![(start, usize::MAX, 0)];
            while let Some(&(u, parent, idx)) = stack.last() {
                if idx < nbrs[u].len() {
                    stack.last_mut().expect("non-empty").2 += 1;
                    let v = nbrs[u][idx];
                    if v == parent {
                        continue; // the edge we entered on (simple graph ⇒ a single parent edge)
                    }
                    if disc[v] == usize::MAX {
                        disc[v] = timer;
                        low[v] = timer;
                        timer += 1;
                        stack.push((v, u, 0));
                    } else {
                        low[u] = low[u].min(disc[v]);
                    }
                } else {
                    stack.pop();
                    if let Some(&(p, ..)) = stack.last() {
                        low[p] = low[p].min(low[u]);
                        if low[u] > disc[p] {
                            bridges.insert(if p < u { (p, u) } else { (u, p) });
                        }
                    }
                }
            }
        }
        bridges
    }
}

/// Mean curvature over a set of edge-curvatures (the paper's `μ`, used as the `A⁺/A⁻` threshold).
#[must_use]
pub fn mean_curvature(cs: &[((usize, usize), i64)]) -> f64 {
    if cs.is_empty() {
        return 0.0;
    }
    cs.iter().map(|(_, c)| *c as f64).sum::<f64>() / cs.len() as f64
}

/// Convenience: index arbitrary string node-ids into `0..n`, returning the map and the indexed pairs.
#[must_use]
pub fn index_pairs<'a>(pairs: &[(&'a str, &'a str)]) -> IndexedPairs<'a> {
    let mut idx: HashMap<&str, usize> = HashMap::new();
    let mut out = Vec::with_capacity(pairs.len());
    for &(a, b) in pairs {
        let n = idx.len();
        let u = *idx.entry(a).or_insert(n);
        let n = idx.len();
        let v = *idx.entry(b).or_insert(n);
        out.push((u, v));
    }
    (idx, out)
}

#[cfg(test)]
mod tests {
    use super::{UndirectedGraph, mean_curvature};

    #[test]
    fn a_triangle_has_positive_curvature_on_every_edge() {
        // Each edge: deg 2 + deg 2, one shared triangle → 4 − 2 − 2 + 3 = 3.
        let g = UndirectedGraph::from_pairs(3, &[(0, 1), (1, 2), (2, 0)]);
        for (u, v) in g.edges() {
            assert_eq!(g.triangles_on(u, v), 1);
            assert_eq!(g.forman(u, v), 3, "triangle edges are positively curved");
        }
        assert!(g.cut_edges().is_empty(), "a cycle has no cut edges");
    }

    #[test]
    fn the_barbell_bridge_is_the_most_negative_and_is_a_real_cut_edge() {
        // Two triangles {0,1,2} and {3,4,5} joined by the single bridge (2,3).
        let g = UndirectedGraph::from_pairs(
            6,
            &[(0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3), (2, 3)],
        );
        let bridge = g.forman(2, 3);
        // bridge: deg(2)=3, deg(3)=3, 0 triangles → 4 − 3 − 3 + 0 = −2.
        assert_eq!(bridge, -2);
        assert_eq!(g.triangles_on(2, 3), 0, "a bridge sits on no triangle");
        // every other edge is positively curved and strictly greater than the bridge
        for (u, v) in g.edges() {
            if (u, v) != (2, 3) {
                assert!(g.forman(u, v) > bridge, "community edges beat the bridge");
            }
        }
        // the INDEPENDENT ground truth agrees: it is the unique cut edge.
        let cuts = g.cut_edges();
        assert_eq!(cuts.len(), 1);
        assert!(cuts.contains(&(2, 3)));
    }

    #[test]
    fn a_path_is_all_cut_edges_the_negative_control() {
        // A tree/path has NO redundancy: every edge is a cut edge (the finder must not miss them).
        let g = UndirectedGraph::from_pairs(4, &[(0, 1), (1, 2), (2, 3)]);
        assert_eq!(g.cut_edges().len(), 3, "every edge of a path is a bridge");
        assert!(
            g.curvatures().iter().all(|(_, c)| *c <= 2),
            "no triangles ⇒ no positive boost"
        );
    }

    #[test]
    fn hub_degree_drives_curvature_negative_even_without_a_bridge() {
        // A star: the centre's high degree makes every spoke negatively curved (no triangles to offset).
        let g = UndirectedGraph::from_pairs(5, &[(0, 1), (0, 2), (0, 3), (0, 4)]);
        let cs = g.curvatures();
        assert!(
            mean_curvature(&cs) < 0.0,
            "star spokes are negatively curved"
        );
        assert_eq!(
            g.cut_edges().len(),
            4,
            "and every spoke is a genuine cut edge"
        );
    }
}
