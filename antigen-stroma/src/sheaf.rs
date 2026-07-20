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
//! **Disciplines baked in, learned the hard way:**
//! 1. **`dim H⁰` / `dim H¹` are the primary — and currently the ONLY — available output. The graded per-edge
//!    residual is DEFERRED.** (This inverts an earlier claim in these docs; corrected 2026-07-20.) The
//!    dimensions are functions of the *structure* and need no data. A residual is a function of *data* — an
//!    observed `x` in the stalks — and this two-instrument construction has none, because it spends the
//!    instrument evidence on the RESTRICTION MAPS and leaves the stalks empty. Improvising an `x` for such a
//!    sheaf only re-derives the census the morphisms came from. Still report a tolerance alongside any
//!    dimension: over floats `rank` is a thresholding decision, not a fact.
//! 2. **Split by connected component first.** `δ⁰` is block-diagonal over components, so ranks and dimensions
//!    simply add. Exact, free, and the single biggest scaling win.
//!
//! **Two design laws, general and reusable:**
//! - **An edge is blind to `ker F_{v⊴e}`.** With vertex stalk `ℝ²` and edge stalk `ℝ`, every restriction is
//!   1×2 and *always* has a 1-dimensional kernel — blindness cannot be avoided, only AIMED. A restriction map
//!   earns its keep by HAVING a kernel (that is what makes `H¹` structure-dependent), so the design question
//!   is never "avoid the blind direction" but **"is the blind direction disjoint from the failure-direction
//!   being hunted?"** Worked counterexample from this very construction: `ker[0.5,0.5] = span{(1,-1)}` is
//!   exactly the cross-instrument DISAGREEMENT axis, so `x_u=(1,0)` vs `x_v=(0,1)` — maximal disagreement —
//!   yields a residual of exactly `0.0`, while exclusive rows return `1.0` on the SAME input. Blind to the
//!   intended target on 100% of edges. The sharper defect is that the two readings are **INCOMMENSURATE**:
//!   because different rows answer different questions on identical input, **no aggregate over a
//!   mixed-edge-type residual vector is meaningful** — not a mean, a max, a threshold, or a ranking.
//! - **A conflict visible at ONE vertex is not an obstruction.** Cross-instrument disagreement is a
//!   *within-vertex* property (`s_v` vs `c_v`); sheaf gluing is a *between-vertex* condition. No restriction
//!   row converts one into the other, so no choice of `R_e` could have worked — the kernel accident above was
//!   the symptom, not the disease. **Cohomology measures the locally-consistent-but-globally-inconsistent**;
//!   if you can see the phenomenon by looking at a single point, cohomology is the wrong instrument by
//!   definition rather than by tuning. Check this BEFORE reaching for it.
//! - **Evidence lives in the MORPHISMS or in the STALKS, never both from one source.** (A) morphisms: the
//!   structural sheaf built here — legitimate, needs no `x`. (B) stalks: a Robinson-style observation sheaf,
//!   where the morphisms come from the CODE and `x_v` is per-symbol evidence — the residual belongs to (B),
//!   where it is non-circular by construction.
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

    /// Per-edge disagreement `‖R_head·x_head − R_tail·x_tail‖` for an assignment `x` (a flat
    /// `|V|·vertex_dim` vector).
    ///
    /// **DEFERRED for the two-instrument structural sheaf — do not read this as that construction's output.**
    /// (An earlier revision of this doc called it "the primary output"; that is retracted, see the module
    /// header.) It is well-defined and correct for any sheaf that genuinely HAS stalk data, i.e. a case (B)
    /// observation sheaf. The structural sheaf built over syn/SCIP is case (A): its evidence is spent on the
    /// restriction maps and its stalks are empty, so there is no honest `x` to pass here — and `ker[0.5,0.5]`
    /// is precisely the disagreement axis, so an improvised `x` reports `0.0` on maximal disagreement.
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

    /// Number of vertices in the base graph.
    #[must_use]
    pub const fn node_count(&self) -> usize {
        self.n_nodes
    }

    /// The coboundary `δ⁰` in **sparse** row form — the same matrix `coboundary` builds densely.
    ///
    /// Each row carries at most `2·vertex_dim` nonzeros (an edge touches exactly its two endpoints), so on a
    /// real call graph the density is ~0.05% and the dense form is not runnable.
    #[must_use]
    pub fn sparse_coboundary(&self) -> SparseMatrix {
        let cols = self.n_nodes * self.vertex_dim;
        let mut rows = Vec::with_capacity(self.edges.len() * self.edge_dim);
        for (ei, &(tail, head)) in self.edges.iter().enumerate() {
            let (rt, rh) = &self.restrictions[ei];
            for r in 0..self.edge_dim {
                let mut row = Vec::with_capacity(2 * self.vertex_dim);
                for c in 0..self.vertex_dim {
                    row.push((head * self.vertex_dim + c, rh[r * self.vertex_dim + c]));
                    row.push((tail * self.vertex_dim + c, -rt[r * self.vertex_dim + c]));
                }
                rows.push(row);
            }
        }
        SparseMatrix::new(rows, cols)
    }

    /// Split into one sub-sheaf per connected component, with vertices renumbered from 0.
    ///
    /// `δ⁰` is block-diagonal over components, so ranks and both cohomology dimensions simply ADD over the
    /// returned pieces. Exact, free, and the single biggest scaling win: elimination cost is superlinear in
    /// block size, so many small blocks beat one large one.
    #[must_use]
    pub fn split_components(&self) -> Vec<Self> {
        let comp = self.components();
        let n_comp = comp.iter().copied().max().map_or(0, |m| m + 1);
        // local index of each global vertex inside its component, plus each component's vertex count
        let mut local = vec![0usize; self.n_nodes];
        let mut sizes = vec![0usize; n_comp];
        for (v, &c) in comp.iter().enumerate() {
            local[v] = sizes[c];
            sizes[c] += 1;
        }
        let mut edges: Vec<Vec<(usize, usize)>> = vec![Vec::new(); n_comp];
        let mut restr: Vec<Vec<(Vec<f64>, Vec<f64>)>> = vec![Vec::new(); n_comp];
        for (ei, &(t, h)) in self.edges.iter().enumerate() {
            let c = comp[t];
            edges[c].push((local[t], local[h]));
            restr[c].push(self.restrictions[ei].clone());
        }
        (0..n_comp)
            .map(|c| {
                Self::new(
                    sizes[c],
                    std::mem::take(&mut edges[c]),
                    self.vertex_dim,
                    self.edge_dim,
                    std::mem::take(&mut restr[c]),
                )
            })
            .collect()
    }

    /// `rank δ⁰` via the sparse path, summed over connected components.
    ///
    /// Equivalent to `rank(&mut self.coboundary(), tol)` but without materializing the dense matrix.
    ///
    /// **The tolerance scale is taken over the WHOLE coboundary, never per block.** `tol` is relative to the
    /// largest entry, so deriving it per component would make the effective cutoff depend on how the graph
    /// happens to partition — a component whose weights are uniformly small would get a proportionally
    /// smaller cutoff and retain rank the whole-graph path discards. That desynchronizes `sparse_rank_total`
    /// from `sparse_rank` and from dense [`rank`] whenever `tol` lands between a block's max and the global
    /// max. Splitting is exact only when the threshold is held fixed across blocks.
    #[must_use]
    pub fn sparse_rank_total(&self, tol: f64) -> SparseRankReport {
        let blocks: Vec<SparseMatrix> = self
            .split_components()
            .iter()
            .map(Self::sparse_coboundary)
            .collect();
        let scale = blocks
            .iter()
            .map(SparseMatrix::max_abs)
            .fold(0.0_f64, f64::max)
            .max(1e-300);
        blocks
            .iter()
            .map(|b| sparse_rank_with_cutoff(b, tol * scale))
            .fold(SparseRankReport::default(), SparseRankReport::merge)
    }

    /// `(dim H⁰, dim H¹)` at a relative tolerance, computed sparsely. **Always report `tol`.**
    #[must_use]
    pub fn sparse_cohomology_dims(&self, tol: f64) -> (usize, usize) {
        let r = self.sparse_rank_total(tol).rank;
        let v = self.n_nodes * self.vertex_dim;
        let e = self.edges.len() * self.edge_dim;
        (v - r, e - r)
    }
}

/// A sparse matrix in row-list form: `rows[i]` is a column-sorted list of `(col, value)` nonzeros.
#[derive(Debug, Clone, Default)]
pub struct SparseMatrix {
    rows: Vec<Vec<(usize, f64)>>,
    cols: usize,
}

impl SparseMatrix {
    /// Build from raw rows. Entries are sorted by column and duplicate columns are summed, so a caller may
    /// emit the same column twice (a self-loop hits its endpoint from both sides). Entries that are EXACTLY
    /// zero after summing are dropped — a restriction row like `[0.0, 1.0]` is genuinely absent from the
    /// coordinate it ignores, and counting those slots as nonzeros would inflate every sparsity statistic.
    #[must_use]
    pub fn new(rows: Vec<Vec<(usize, f64)>>, cols: usize) -> Self {
        let rows = rows
            .into_iter()
            .map(|mut r| {
                r.sort_unstable_by_key(|&(c, _)| c);
                let mut out: Vec<(usize, f64)> = Vec::with_capacity(r.len());
                for (c, v) in r {
                    match out.last_mut() {
                        Some(last) if last.0 == c => last.1 += v,
                        _ => out.push((c, v)),
                    }
                }
                out.retain(|&(_, v)| v != 0.0);
                out
            })
            .collect();
        Self { rows, cols }
    }

    /// Number of stored nonzeros (before elimination fill-in).
    #[must_use]
    pub fn nnz(&self) -> usize {
        self.rows.iter().map(Vec::len).sum()
    }

    /// Largest entry by magnitude — the scale a RELATIVE tolerance multiplies.
    ///
    /// Exposed because a caller reducing several blocks of one matrix must take this over the WHOLE matrix
    /// and hold it fixed; see [`Sheaf::sparse_rank_total`].
    #[must_use]
    pub fn max_abs(&self) -> f64 {
        self.rows
            .iter()
            .flat_map(|r| r.iter())
            .fold(0.0_f64, |a, &(_, v)| a.max(v.abs()))
    }

    /// Number of rows.
    #[must_use]
    pub const fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Number of columns.
    #[must_use]
    pub const fn col_count(&self) -> usize {
        self.cols
    }

    /// Expand to the dense row-major form — for cross-checking the sparse path against dense [`rank`].
    /// Only sane on small matrices; the whole point of the sparse path is that this does not fit.
    #[must_use]
    pub fn to_dense(&self) -> Vec<Vec<f64>> {
        self.rows
            .iter()
            .map(|r| {
                let mut d = vec![0.0; self.cols];
                for &(c, v) in r {
                    d[c] = v;
                }
                d
            })
            .collect()
    }
}

/// What a sparse rank computation found, including the fill-in it had to carry.
///
/// Fill-in is the thing that decides whether the sparse path is viable at all: `peak_nnz` is the largest
/// number of nonzeros held in the pivot set at once, against `input_nnz` going in.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SparseRankReport {
    /// The rank, at the tolerance passed in.
    pub rank: usize,
    /// Nonzeros in the input matrix.
    pub input_nnz: usize,
    /// Peak nonzeros held across all retained pivot rows (the real memory cost).
    pub peak_nnz: usize,
    /// Nonzeros in the longest single pivot row (the densest row fill-in produced).
    pub widest_pivot_row: usize,
}

impl SparseRankReport {
    /// Combine two block reports — valid because `δ⁰` is block-diagonal over components, so ranks and
    /// nonzero counts add and the peaks are the max over blocks.
    #[must_use]
    pub fn merge(self, other: Self) -> Self {
        Self {
            rank: self.rank + other.rank,
            input_nnz: self.input_nnz + other.input_nnz,
            peak_nnz: self.peak_nnz.max(other.peak_nnz),
            widest_pivot_row: self.widest_pivot_row.max(other.widest_pivot_row),
        }
    }

    /// Fill-in factor: peak retained nonzeros over input nonzeros. `1.0` means no fill-in at all.
    #[must_use]
    pub fn fill_factor(&self) -> f64 {
        if self.input_nnz == 0 {
            1.0
        } else {
            self.peak_nnz as f64 / self.input_nnz as f64
        }
    }
}

/// Rank of a sparse matrix by streaming Gaussian elimination with partial pivoting, dependency-free.
///
/// One row at a time is reduced against a pivot set keyed by pivot column. Because every retained pivot row
/// has its pivot column as its *lowest* column, eliminating only ever creates entries strictly to the right —
/// the leading column advances monotonically, so the reduction terminates. When an incoming row has a larger
/// magnitude in the pivot column than the sitting pivot, the two are swapped (partial pivoting in streaming
/// form), which bounds multiplier growth exactly as the dense path's row swap does.
///
/// `tol` is RELATIVE to the largest entry in the input — rank over floats is a thresholding decision, not a
/// fact, so the caller must report the tolerance alongside any dimension derived from this.
#[must_use]
pub fn sparse_rank(m: &SparseMatrix, tol: f64) -> SparseRankReport {
    sparse_rank_with_cutoff(m, tol * m.max_abs().max(1e-300))
}

/// Rank at an ABSOLUTE cutoff, for callers reducing several blocks of one matrix under a shared threshold.
///
/// [`sparse_rank`] is this with `cutoff = tol · max|entry|`. Use this form when the scale must come from
/// somewhere other than the matrix in hand — the component split in [`Sheaf::sparse_rank_total`] is exactly
/// that case, and deriving the scale per block instead is a real bug (it desynchronizes the split path from
/// the dense one).
#[must_use]
pub fn sparse_rank_with_cutoff(m: &SparseMatrix, cutoff: f64) -> SparseRankReport {
    use std::collections::BTreeMap;

    // pivot column -> that pivot's row, sorted, whose FIRST entry is at the pivot column
    let mut pivots: BTreeMap<usize, Vec<(usize, f64)>> = BTreeMap::new();
    let mut report = SparseRankReport {
        input_nnz: m.nnz(),
        ..SparseRankReport::default()
    };
    let mut live_nnz = 0usize;

    for row in &m.rows {
        let mut cur: BTreeMap<usize, f64> = row.iter().copied().collect();
        loop {
            // advance past entries that are zero within tolerance
            let lead = loop {
                let Some((&c, &v)) = cur.iter().next() else {
                    break None;
                };
                if v.abs() <= cutoff {
                    cur.remove(&c);
                } else {
                    break Some(c);
                }
            };
            let Some(c) = lead else { break }; // row is dependent within tolerance
            let Some(mut prow) = pivots.remove(&c) else {
                // new pivot
                let prow: Vec<(usize, f64)> = cur.into_iter().collect();
                live_nnz += prow.len();
                report.widest_pivot_row = report.widest_pivot_row.max(prow.len());
                report.peak_nnz = report.peak_nnz.max(live_nnz);
                pivots.insert(c, prow);
                report.rank += 1;
                break;
            };
            // partial pivoting: keep the larger magnitude in the pivot seat
            if cur[&c].abs() > prow[0].1.abs() {
                let incoming: Vec<(usize, f64)> = cur.into_iter().collect();
                live_nnz = live_nnz + incoming.len() - prow.len();
                report.widest_pivot_row = report.widest_pivot_row.max(incoming.len());
                report.peak_nnz = report.peak_nnz.max(live_nnz);
                cur = prow.into_iter().collect();
                prow = incoming;
            }
            let f = cur[&c] / prow[0].1;
            for &(pc, pv) in &prow {
                let slot = cur.entry(pc).or_insert(0.0);
                *slot = f.mul_add(-pv, *slot);
            }
            cur.remove(&c); // exactly zero by construction
            pivots.insert(c, prow);
        }
    }
    report
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
    use super::{Sheaf, rank, sparse_rank};

    const TOL: f64 = 1e-9;

    /// Tolerances the cross-check sweeps. Checking only a tiny `tol` hid a real divergence: the split path
    /// and the flat path can only disagree once `tol` is large enough to annihilate some rows but not others,
    /// which never happens near 1e-9. A cross-check that never varies the threshold cannot see threshold bugs.
    const TOL_SWEEP: &[f64] = &[1e-12, 1e-9, 1e-3, 0.4, 0.6, 0.9];

    /// The cross-check every sparse test runs: sparse rank MUST equal dense rank on the same sheaf, at EVERY
    /// tolerance. Disagreement means one of the implementations is wrong, which outranks any measurement.
    fn agree(s: &Sheaf) -> usize {
        for &tol in TOL_SWEEP {
            let dense = rank(&mut s.coboundary(), tol);
            let sparse_flat = sparse_rank(&s.sparse_coboundary(), tol).rank;
            let sparse_split = s.sparse_rank_total(tol).rank;
            assert_eq!(
                dense, sparse_flat,
                "sparse rank disagrees with dense at tol {tol:e}"
            );
            assert_eq!(
                dense, sparse_split,
                "component-split rank disagrees with dense at tol {tol:e}"
            );
            assert_eq!(
                s.cohomology_dims(tol),
                s.sparse_cohomology_dims(tol),
                "sparse cohomology dims disagree with dense at tol {tol:e}"
            );
        }
        rank(&mut s.coboundary(), TOL)
    }

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
        // at edge 1 specifically. This is a CONSTANT sheaf with a supplied x, i.e. the case (B) setting
        // where a residual is meaningful; it is NOT a claim about the two-instrument structural sheaf, whose
        // residual is deferred (see `edge_residuals` and the module header). An earlier revision of this
        // comment called the residual "the PRIMARY output" -- retracted 2026-07-20.
        let s = Sheaf::constant(3, vec![(0, 1), (1, 2)], 1);
        let r = s.edge_residuals(&[1.0, 1.0, 5.0]);
        assert!((r[0] - 0.0).abs() < TOL, "edge 0 agrees");
        assert!(
            (r[1] - 4.0).abs() < TOL,
            "edge 1 carries the whole disagreement"
        );
    }

    /// Makes the module header's blindness law EXECUTABLE instead of merely argued.
    ///
    /// The other `edge_residuals` test runs at `vertex_dim = 1`, where a 1×1 restriction has no nontrivial
    /// kernel and the blindness *cannot occur* — so nothing pinned the actual claim until now. A doc law that
    /// no test can contradict is a comment, not an invariant.
    #[test]
    fn edge_residuals_is_blind_to_cross_instrument_disagreement_at_vertex_dim_2() {
        // `ker[0.5,0.5] = span{(1,-1)}` IS the cross-instrument disagreement axis, so the `Both` row returns
        // exactly 0.0 on MAXIMAL disagreement while the exclusive row returns 1.0 on the SAME input.
        //
        // Neither reading is "wrong" in isolation — the defect is that they are **INCOMMENSURATE**. Because
        // the two rows answer different questions on identical input, NO aggregate over a mixed-edge-type
        // residual vector is meaningful: not a mean, not a max, not a threshold, not a ranking. The scales
        // do not correspond, so there is nothing to compare across edge types.
        let both = (vec![0.5, 0.5], vec![0.5, 0.5]);
        let scip_only = (vec![0.0, 1.0], vec![0.0, 1.0]);
        let s = Sheaf::new(4, vec![(0, 1), (2, 3)], 2, 1, vec![both, scip_only]);
        // v0,v2 = (1,0) "syn asserts, SCIP does not"; v1,v3 = (0,1), the exact opposite.
        let x = [1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0];
        let r = s.edge_residuals(&x);
        assert!(
            r[0].abs() < TOL,
            "the Both row must be BLIND to cross-instrument disagreement; got {}",
            r[0]
        );
        assert!(
            (r[1] - 1.0).abs() < TOL,
            "exclusive row reads 1.0 on the same input -- INCOMMENSURATE with the Both row, so no
             aggregate over mixed edge types is meaningful; got {}",
            r[1]
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

    // ---- the sparse path, mirroring each case above and cross-checked against dense ----

    #[test]
    fn sparse_agrees_with_dense_on_a_single_edge() {
        let s = Sheaf::constant(2, vec![(0, 1)], 1);
        assert_eq!(agree(&s), 1);
        assert_eq!(s.sparse_cohomology_dims(TOL), (1, 0));
    }

    #[test]
    fn sparse_agrees_with_dense_on_a_path_and_a_triangle() {
        let path = Sheaf::constant(3, vec![(0, 1), (1, 2)], 1);
        assert_eq!(agree(&path), 2);
        assert_eq!(path.sparse_cohomology_dims(TOL), (1, 0));
        let tri = Sheaf::constant(3, vec![(0, 1), (1, 2), (2, 0)], 1);
        assert_eq!(agree(&tri), 2, "the triangle's third edge is dependent");
        let d = tri.sparse_cohomology_dims(TOL);
        assert_eq!(d, (1, 1));
        assert!(tri.euler_check(d));
    }

    #[test]
    fn sparse_sees_the_holonomy_that_kills_every_section() {
        let two = (vec![1.0], vec![2.0]);
        let id = (vec![1.0], vec![1.0]);
        let s = Sheaf::new(
            3,
            vec![(0, 1), (1, 2), (2, 0)],
            1,
            1,
            vec![two, id.clone(), id],
        );
        assert_eq!(agree(&s), 3, "holonomy != 1 ⇒ delta^0 has full rank");
        assert_eq!(s.sparse_cohomology_dims(TOL), (0, 0));
    }

    #[test]
    fn sparse_dimensions_add_over_components() {
        let s = Sheaf::constant(4, vec![(0, 1), (2, 3)], 1);
        assert_eq!(s.split_components().len(), 2);
        assert_eq!(agree(&s), 2);
        assert_eq!(s.sparse_cohomology_dims(TOL), (2, 0));
        // An ISOLATED vertex is its own component and contributes a whole stalk to H0.
        let lone = Sheaf::constant(3, vec![(0, 1)], 2);
        assert_eq!(lone.split_components().len(), 2);
        assert_eq!(agree(&lone), 2);
        assert_eq!(lone.sparse_cohomology_dims(TOL), (4, 0));
    }

    #[test]
    fn sparse_agrees_with_dense_on_the_measured_construction() {
        // The shape actually measured on antigen: F(v) = R^2, F(e) = R, the SAME 1x2 row on both endpoints.
        // Three-instrument cases, plus a cycle, so the restriction maps are genuinely heterogeneous.
        let both = (vec![0.5, 0.5], vec![0.5, 0.5]);
        let syn = (vec![1.0, 0.0], vec![1.0, 0.0]);
        let scip = (vec![0.0, 1.0], vec![0.0, 1.0]);
        let s = Sheaf::new(
            4,
            vec![(0, 1), (1, 2), (2, 0), (2, 3)],
            2,
            1,
            vec![both, syn, scip.clone(), scip],
        );
        let r = agree(&s);
        let d = s.sparse_cohomology_dims(TOL);
        assert_eq!(d, (8 - r, 4 - r));
        assert!(s.euler_check(d));
    }

    #[test]
    fn the_component_split_uses_the_global_tolerance_scale_not_a_per_block_one() {
        // REGRESSION (found by adversarial review, 2026-07-20). Two components whose weights differ in
        // magnitude: one carries 0.5 entries, the other 1.0. At tol = 0.6 the global cutoff is 0.6, which
        // annihilates the 0.5 block entirely -> rank 1. Deriving the scale PER COMPONENT gave the 0.5 block
        // its own cutoff of 0.3, so it survived and the split path returned 2 while dense returned 1.
        let half = (vec![0.5, 0.5], vec![0.5, 0.5]);
        let unit = (vec![0.0, 1.0], vec![0.0, 1.0]);
        let s = Sheaf::new(4, vec![(0, 1), (2, 3)], 2, 1, vec![half, unit]);
        assert_eq!(s.split_components().len(), 2, "the blocks must be separate");

        let dense = rank(&mut s.coboundary(), 0.6);
        assert_eq!(dense, 1, "at tol 0.6 the 0.5-weighted block is annihilated");
        assert_eq!(
            s.sparse_rank_total(0.6).rank,
            dense,
            "split path must use the GLOBAL scale, not each block's own"
        );
        assert_eq!(sparse_rank(&s.sparse_coboundary(), 0.6).rank, dense);
        agree(&s);
    }

    #[test]
    fn sparse_rank_reports_its_fill_in() {
        // A star has no fill-in beyond the input; the report must say so rather than guess.
        let star = Sheaf::constant(5, vec![(0, 1), (0, 2), (0, 3), (0, 4)], 1);
        let rep = sparse_rank(&star.sparse_coboundary(), TOL);
        assert_eq!(rep.rank, 4);
        assert_eq!(rep.input_nnz, 8, "4 edges x 2 endpoints x 1 stalk dim");
        assert!(
            rep.fill_factor() <= 1.0,
            "a star eliminates without fill-in, got {}",
            rep.fill_factor()
        );
    }

    #[test]
    fn a_rank_deficient_row_is_dropped_not_counted() {
        // Negative control: an EXACTLY duplicated edge must not raise the rank. If it does, the tolerance
        // path is not actually detecting dependence and every dimension reported is wrong.
        let s = Sheaf::constant(2, vec![(0, 1), (0, 1)], 1);
        assert_eq!(agree(&s), 1);
        assert_eq!(s.sparse_cohomology_dims(TOL), (1, 1));
    }
}
