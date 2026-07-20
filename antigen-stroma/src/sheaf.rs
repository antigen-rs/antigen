//! # `sheaf` — cellular sheaf cohomology over the stroma: H⁰ = the instruments agree, H¹ = they conflict.
//!
//! nb 028 ratified that the stroma is a **sheaf**; this is the machinery. A cellular sheaf on a graph is a
//! vector space (*stalk*) on every vertex and every edge, plus a *restriction map* `F(v) → F(e)` for each
//! incidence. The coboundary `δ⁰` sends a vertex-assignment to its per-edge disagreement:
//!
//! ```text
//! (δ⁰x)_e = R[head(e), e]·x_head − R[tail(e), e]·x_tail
//! H⁰ = ker δ⁰   (assignments every incident edge agrees with — the global sections)
//! H¹ = coker δ⁰ (edge-data no vertex-assignment can explain — the irreducible obstruction)
//! ```
//!
//! **Two disciplines are baked in, both learned the hard way (2026-07-19):**
//! 1. **The graded residual is PRIMARY, the integer dimensions are secondary.** Over floating-point evidence
//!    `rank` is a *thresholding decision*, so reporting `dim H¹ = 3` without a tolerance is an over-claim.
//!    The per-edge residual also *localizes* the conflict to a specific edge, which is what an organ needs to
//!    report — a dimension says conflict exists but not where.
//! 2. **Split by connected component first.** `δ⁰` is block-diagonal over components, so ranks and dimensions
//!    simply add. Exact, free, and the single biggest scaling win.
//!
//! **Trap (Hansen–Ghrist Thm 4.2/4.3):** do NOT apply Kron reduction / graph sparsification to shrink this.
//! It is valid only when vertex stalks have dimension ≤ 1 and fails for sheaves in general — the
//! graph-Laplacian intuition transfers and the sheaf-Laplacian fact does not.

// Stalk/graph dimensions are ≤ 10⁶ here — far from i64 wrap or f64 mantissa limits.
#![allow(clippy::cast_precision_loss, clippy::cast_possible_wrap)]

/// A cellular sheaf of real vector spaces over a graph, with uniform stalk dimensions.
///
/// `restrictions[e] = (R_tail, R_head)`, each an `edge_dim × vertex_dim` row-major matrix.
#[derive(Debug, Clone)]
pub struct Sheaf {
    n_nodes: usize,
    edges: Vec<(usize, usize)>,
    vertex_dim: usize,
    edge_dim: usize,
    restrictions: Vec<(Vec<f64>, Vec<f64>)>,
}

impl Sheaf {
    /// Build a sheaf. Every edge needs a `(R_tail, R_head)` pair of `edge_dim × vertex_dim` matrices.
    ///
    /// # Panics
    /// If any restriction matrix has the wrong length.
    #[must_use]
    pub fn new(
        n_nodes: usize,
        edges: Vec<(usize, usize)>,
        vertex_dim: usize,
        edge_dim: usize,
        restrictions: Vec<(Vec<f64>, Vec<f64>)>,
    ) -> Self {
        assert_eq!(
            edges.len(),
            restrictions.len(),
            "one restriction pair per edge"
        );
        let want = edge_dim * vertex_dim;
        for (t, h) in &restrictions {
            assert_eq!(
                t.len(),
                want,
                "tail restriction must be edge_dim x vertex_dim"
            );
            assert_eq!(
                h.len(),
                want,
                "head restriction must be edge_dim x vertex_dim"
            );
        }
        Self {
            n_nodes,
            edges,
            vertex_dim,
            edge_dim,
            restrictions,
        }
    }

    /// The **constant sheaf**: stalk `ℝ^d` everywhere, every restriction the identity. Its `δ⁰` is the
    /// ordinary incidence matrix, so `dim H¹ = β₁·d` — the trivial baseline to compare a real sheaf against.
    #[must_use]
    pub fn constant(n_nodes: usize, edges: Vec<(usize, usize)>, d: usize) -> Self {
        let mut id = vec![0.0; d * d];
        for i in 0..d {
            id[i * d + i] = 1.0;
        }
        let restrictions = vec![(id.clone(), id); edges.len()];
        Self::new(n_nodes, edges, d, d, restrictions)
    }

    /// The coboundary `δ⁰` as a dense row-major matrix: `(|E|·edge_dim) × (|V|·vertex_dim)`.
    #[must_use]
    pub fn coboundary(&self) -> Vec<Vec<f64>> {
        let cols = self.n_nodes * self.vertex_dim;
        let mut m = vec![vec![0.0; cols]; self.edges.len() * self.edge_dim];
        for (ei, &(tail, head)) in self.edges.iter().enumerate() {
            let (rt, rh) = &self.restrictions[ei];
            for r in 0..self.edge_dim {
                let row = &mut m[ei * self.edge_dim + r];
                for c in 0..self.vertex_dim {
                    row[head * self.vertex_dim + c] += rh[r * self.vertex_dim + c];
                    row[tail * self.vertex_dim + c] -= rt[r * self.vertex_dim + c];
                }
            }
        }
        m
    }

    /// `(dim H⁰, dim H¹)` at a relative tolerance. **Both are tolerance-dependent — always report `tol`.**
    ///
    /// Uses the rank-nullity identity: `dim H⁰ = |V|·d_v − r`, `dim H¹ = |E|·d_e − r`.
    #[must_use]
    pub fn cohomology_dims(&self, tol: f64) -> (usize, usize) {
        let r = rank(&mut self.coboundary(), tol);
        let v = self.n_nodes * self.vertex_dim;
        let e = self.edges.len() * self.edge_dim;
        (v - r, e - r)
    }

    /// The free self-check that always holds: `dim H⁰ − dim H¹ == |V|·d_v − |E|·d_e` (the Euler identity).
    #[must_use]
    pub const fn euler_check(&self, dims: (usize, usize)) -> bool {
        let lhs = dims.0 as i64 - dims.1 as i64;
        let rhs =
            (self.n_nodes * self.vertex_dim) as i64 - (self.edges.len() * self.edge_dim) as i64;
        lhs == rhs
    }

    /// **The primary output.** Per-edge disagreement `‖R_head·x_head − R_tail·x_tail‖` for an assignment `x`
    /// (a flat `|V|·vertex_dim` vector). Localizes conflict to a specific edge — a dimension cannot.
    #[must_use]
    pub fn edge_residuals(&self, x: &[f64]) -> Vec<f64> {
        self.edges
            .iter()
            .enumerate()
            .map(|(ei, &(tail, head))| {
                let (rt, rh) = &self.restrictions[ei];
                let mut sq = 0.0;
                for r in 0..self.edge_dim {
                    let mut acc = 0.0;
                    for c in 0..self.vertex_dim {
                        acc =
                            rh[r * self.vertex_dim + c].mul_add(x[head * self.vertex_dim + c], acc);
                        acc = (-rt[r * self.vertex_dim + c])
                            .mul_add(x[tail * self.vertex_dim + c], acc);
                    }
                    sq = acc.mul_add(acc, sq);
                }
                sq.sqrt()
            })
            .collect()
    }

    /// Partition nodes into connected components (`δ⁰` is block-diagonal over these; dims ADD).
    #[must_use]
    pub fn components(&self) -> Vec<usize> {
        let mut comp = vec![usize::MAX; self.n_nodes];
        let mut adj = vec![Vec::new(); self.n_nodes];
        for &(u, v) in &self.edges {
            adj[u].push(v);
            adj[v].push(u);
        }
        let mut next = 0;
        for s in 0..self.n_nodes {
            if comp[s] != usize::MAX {
                continue;
            }
            let mut stack = vec![s];
            comp[s] = next;
            while let Some(u) = stack.pop() {
                for &v in &adj[u] {
                    if comp[v] == usize::MAX {
                        comp[v] = next;
                        stack.push(v);
                    }
                }
            }
            next += 1;
        }
        comp
    }

    /// Number of edges in the base graph.
    #[must_use]
    pub const fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

/// Rank of a dense matrix by Gaussian elimination with partial pivoting.
///
/// Consumes (destroys) the input. `tol` is RELATIVE to the largest entry seen — rank over floats is a
/// thresholding decision, not a fact, so the caller must report the tolerance alongside any dimension.
#[must_use]
pub fn rank(m: &mut [Vec<f64>], tol: f64) -> usize {
    let (rows, cols) = (m.len(), m.first().map_or(0, Vec::len));
    let scale = m
        .iter()
        .flat_map(|r| r.iter())
        .fold(0.0_f64, |a, b| a.max(b.abs()))
        .max(1e-300);
    let mut rank = 0;
    for col in 0..cols {
        if rank == rows {
            break;
        }
        let (mut best, mut bestv) = (rank, 0.0_f64);
        for (r, row) in m.iter().enumerate().skip(rank) {
            if row[col].abs() > bestv {
                best = r;
                bestv = row[col].abs();
            }
        }
        if bestv <= tol * scale {
            continue; // column is dependent within tolerance
        }
        m.swap(rank, best);
        let pivot_row = m[rank].clone();
        let pivot = pivot_row[col];
        for row in m.iter_mut().take(rows).skip(rank + 1) {
            let f = row[col] / pivot;
            if f != 0.0 {
                for (target, &p) in row.iter_mut().zip(pivot_row.iter()).skip(col) {
                    *target = (-f).mul_add(p, *target);
                }
            }
        }
        rank += 1;
    }
    rank
}

#[cfg(test)]
mod tests {
    use super::Sheaf;

    const TOL: f64 = 1e-9;

    #[test]
    fn a_single_edge_has_one_global_section_and_no_obstruction() {
        // Two nodes, one edge, identity restrictions: sections are the constants (dim 1); nothing to obstruct.
        let s = Sheaf::constant(2, vec![(0, 1)], 1);
        let d = s.cohomology_dims(TOL);
        assert_eq!(d, (1, 0));
        assert!(s.euler_check(d));
    }

    #[test]
    fn a_tree_has_no_obstruction_but_a_cycle_does_the_constant_baseline() {
        // Path 0-1-2: |V|=3,|E|=2 -> H0=1 (constants), H1=0.
        let path = Sheaf::constant(3, vec![(0, 1), (1, 2)], 1);
        assert_eq!(path.cohomology_dims(TOL), (1, 0));
        // Triangle: |V|=3,|E|=3 -> H0=1, H1=1. For the CONSTANT sheaf dim H1 = beta1 (the trivial baseline).
        let tri = Sheaf::constant(3, vec![(0, 1), (1, 2), (2, 0)], 1);
        let d = tri.cohomology_dims(TOL);
        assert_eq!(d, (1, 1));
        assert!(tri.euler_check(d));
    }

    #[test]
    fn holonomy_around_a_cycle_kills_every_global_section() {
        // THE load-bearing test: restrictions that do not compose to the identity around the triangle admit
        // NO nonzero global section. This is the whole point of a sheaf over the constant sheaf -- the
        // conflict is in the RESTRICTION MAPS, not in the graph's topology.
        let two = (vec![1.0], vec![2.0]); // head reads twice the tail
        let id = (vec![1.0], vec![1.0]);
        let s = Sheaf::new(
            3,
            vec![(0, 1), (1, 2), (2, 0)],
            1,
            1,
            vec![two, id.clone(), id],
        );
        let (h0, h1) = s.cohomology_dims(TOL);
        assert_eq!(h0, 0, "holonomy != 1 around the cycle ⇒ no global sections");
        assert_eq!(h1, 0, "and the obstruction is absorbed: rank is full");
        assert!(s.euler_check((h0, h1)));
    }

    #[test]
    fn edge_residuals_localize_the_disagreement() {
        // A path with a section that agrees on edge 0 and disagrees on edge 1 -- the residual must point
        // at edge 1 specifically. This is the PRIMARY output: a dimension cannot say WHERE.
        let s = Sheaf::constant(3, vec![(0, 1), (1, 2)], 1);
        let r = s.edge_residuals(&[1.0, 1.0, 5.0]);
        assert!((r[0] - 0.0).abs() < TOL, "edge 0 agrees");
        assert!(
            (r[1] - 4.0).abs() < TOL,
            "edge 1 carries the whole disagreement"
        );
    }

    #[test]
    fn components_split_and_dimensions_add() {
        // Two disjoint edges: 2 components, and H0 = 1 + 1 = 2 (a constant per component).
        let s = Sheaf::constant(4, vec![(0, 1), (2, 3)], 1);
        let comp = s.components();
        assert_ne!(comp[0], comp[2], "disjoint edges are separate components");
        assert_eq!(s.cohomology_dims(TOL), (2, 0));
    }
}
