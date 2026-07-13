//! GLMY **path homology** H₀/H₁ over the tier-filtered reachability graph.
//!
//! The afferent *confidence-graded topology* organ (Lens A, charter
//! `jbd/charters/confidence-graded-topology/idea.md`; research notebook 022 §11.9).
//!
//! # What this computes
//!
//! Given a directed edge slice (a call/reach graph), this module computes Grigor'yan–Lin–Muranov–Yau
//! **path homology** (arXiv 2404.01007), NOT the directed flag complex and NOT the undirected complex.
//! The complex-choice is the whole design decision: path homology's *trivial* motifs (the bigon `a⇄b`,
//! the transitive triangle with its shortcut chord, the boundary square = reconciling diamond) coincide
//! with the code's *benign* cycles (mutual recursion, helper+shortcut, reconciling routes); its
//! *generators* (the directed cycle — including the 3-cycle — and the non-boundary square/polygon) are
//! exactly genuine multi-route entanglement with no reconciling shortcut. See the ℚ-verified taxonomy in
//! notebook 022 §11.9, dogfooded as the born-red tests at the foot of this file.
//!
//! # The 3-level tier filtration → a confidence barcode
//!
//! [`ResolutionTier`] is a total order `Syntactic < Resolved < T3Mir`, and every edge already carries
//! its tier. Threshold the edge set downward from the top — `G₃` (tier ≥ `T3Mir`) ⊆ `G₂` (tier ≥
//! `Resolved`) ⊆ `G₁` (all edges) — a bona-fide filtration on the graph, indexed by a knob already in
//! the type. Thresholding never touches orientation, so **chirality is preserved at every level** (the
//! chirality-preserving persistence the Vietoris–Rips objection said was impossible). [`TierBarcode`]
//! reads the H₀/H₁ dimension at each level; a class surviving down to `G₃` is proof-grade, a bar that
//! appears only at `Syntactic` is a weak-evidence artifact.
//!
//! # The math (regular path homology, over ℚ)
//!
//! For a digraph with vertex set `V` and edge set `E` (regular reduction: `e_{ii} = 0`, self-loops
//! dropped):
//! - `Ω₀` = span of vertices; `Ω₁` = span of edges (the allowed 1-paths), `dim = m`.
//! - An allowed 2-path `e_{ijk}` (edges `i→j`, `j→k`) has `∂e_{ijk} = e_{jk} − e_{ik} + e_{ij}`. It is
//!   ∂-invariant (lies in `Ω₂`) individually iff `i == k` (then `e_{ii} = 0`) or `i→k ∈ E` (the term
//!   `e_{ik}` is itself an allowed 1-path). Otherwise only *differences* of two routes `i→j→k`,
//!   `i→l→k` sharing the same non-allowed endpoint pair `(i,k)` lie in `Ω₂` (the `e_{ik}` cancels).
//! - `rank(∂₂)` is computed over ℚ by exact rational Gaussian elimination (a hand-rolled `Frac`, no
//!   heavy dependency).
//! - `dim H₀ = c` (weakly-connected components); `dim H₁ = (m − n + c) − rank(∂₂)`, where `m − n + c`
//!   is the cycle-space dimension of the underlying (simple) graph.
//!
//! # Reconciliation notes (this is an MVP — honest scope)
//!
//! - **Induced vertex set.** `n` and the component count `c` are taken over the vertices *touched* by
//!   the level's active edges (the induced subgraph), not a single fixed global vertex set. This is
//!   what lets `H₀` *rise* as a lower tier introduces a new disconnected edge-cluster (the notebook's
//!   worked barcode `H₀ = 1 → 2 → 1`); `H₁ = m − n + c − rank(∂₂)` is invariant to the isolated-vertex
//!   convention either way.
//! - **Dimension barcode, not generator pairing.** [`TierBarcode`] reports the H₀/H₁ *dimension* at each
//!   of the three levels. Full persistence pairing (tracking each generator's birth/death tier through
//!   the induced maps `H₁(Gᵢ) → H₁(Gᵢ₋₁)`) is deferred; the dimension barcode is what the §11.9 worked
//!   example pins and is sufficient to read proof-grade-persistence vs weak-evidence bars off the three
//!   levels.
//! - **H₂+ not computed.** The charter scopes H₀/H₁ only (the tractable `O((mn)^ω)` regime).
//! - [`undirected_flag_h1`] is provided as the *contrast* invariant (the taxonomy's "flag-H₁" column):
//!   the undirected flag (clique) complex that fills triangles. It exists to witness chirality (the
//!   directed 3-cycle is a path-H₁ generator but flag-trivial), not as an organ in its own right.

use std::collections::{BTreeMap, BTreeSet};

use super::closure::ReachabilityRelation;
use crate::read::ResolutionTier;

// --- exact rationals for the ℚ-rank ----------------------------------------------------------------

/// A reduced rational `num/den` (`den > 0`), the exact scalar the boundary-rank Gaussian elimination
/// folds over. Entries here are tiny integers (`0, ±1, ±2`), so `i128` never overflows for the graph
/// sizes this organ sees; the reduction keeps it exact regardless.
#[derive(Clone, Copy)]
struct Frac {
    num: i128,
    den: i128,
}

impl Frac {
    /// The rational `n / 1`.
    const fn int(n: i128) -> Self {
        Self { num: n, den: 1 }
    }

    /// The additive identity `0`.
    const fn zero() -> Self {
        Self { num: 0, den: 1 }
    }

    /// Build a reduced rational from a numerator/denominator, normalizing the sign onto `num`.
    fn reduce(mut n: i128, mut d: i128) -> Self {
        if d < 0 {
            n = -n;
            d = -d;
        }
        if n == 0 {
            return Self { num: 0, den: 1 };
        }
        let g = gcd(n.abs(), d);
        Self {
            num: n / g,
            den: d / g,
        }
    }

    const fn is_zero(self) -> bool {
        self.num == 0
    }

    fn add(self, o: Self) -> Self {
        Self::reduce(self.num * o.den + o.num * self.den, self.den * o.den)
    }

    fn sub(self, o: Self) -> Self {
        Self::reduce(self.num * o.den - o.num * self.den, self.den * o.den)
    }

    fn mul(self, o: Self) -> Self {
        Self::reduce(self.num * o.num, self.den * o.den)
    }

    fn div(self, o: Self) -> Self {
        Self::reduce(self.num * o.den, self.den * o.num)
    }
}

/// The (positive) greatest common divisor — the reduction kernel for [`Frac`].
fn gcd(a: i128, b: i128) -> i128 {
    let (mut a, mut b) = (a, b);
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a.abs().max(1)
}

/// The rank over ℚ of the matrix whose rows are `rows` (each a dense length-`cols` rational vector),
/// by exact Gaussian elimination. Rank is transpose-invariant, so callers pass boundary *columns* as
/// rows freely.
// The elimination genuinely indexes `rows` by row (swap + the `r != rank` self-skip + the two-row
// read/write of `rows[r]` against `rows[rank]`) and by column — an iterator rewrite cannot express the
// simultaneous mutable/immutable row access, so the range loops are load-bearing here.
#[allow(clippy::needless_range_loop)]
fn rank_q(mut rows: Vec<Vec<Frac>>, cols: usize) -> usize {
    let nrows = rows.len();
    let mut rank = 0;
    for col in 0..cols {
        // Find a pivot row at or below `rank` with a nonzero entry in this column.
        let mut sel = None;
        for r in rank..nrows {
            if !rows[r][col].is_zero() {
                sel = Some(r);
                break;
            }
        }
        let Some(p) = sel else { continue };
        rows.swap(rank, p);
        // Eliminate this column from every other row.
        for r in 0..nrows {
            if r != rank && !rows[r][col].is_zero() {
                let factor = rows[r][col].div(rows[rank][col]);
                for c in col..cols {
                    rows[r][c] = rows[r][c].sub(factor.mul(rows[rank][c]));
                }
            }
        }
        rank += 1;
        if rank == nrows {
            break;
        }
    }
    rank
}

// --- the homology results --------------------------------------------------------------------------

/// The GLMY path homology of one directed edge slice: `H₀ = c` and `H₁`, with the intermediate ranks
/// exposed so a reader can see *why* a cycle counted (or didn't).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathHomology {
    /// The number of vertices touched by the slice's edges (the induced vertex set).
    pub vertices: usize,
    /// The number of directed edges (deduplicated, self-loops dropped) — `m = dim Ω₁`.
    pub edges: usize,
    /// The number of weakly-connected components — `c`, equal to `dim H₀`.
    pub components: usize,
    /// The cycle-space dimension of the underlying simple graph, `m − n + c` (≥ 0 always).
    pub cycle_space_dim: usize,
    /// `rank(∂₂)` over ℚ — how many independent 2-cells the allowed 2-paths fill.
    pub rank_boundary2: usize,
    /// `dim H₀` — the number of weakly-connected components.
    pub h0: usize,
    /// `dim H₁ = (m − n + c) − rank(∂₂)` — the non-boundary directed cycles (genuine entanglement).
    pub h1: usize,
}

/// Accumulate `coeff` onto the edge `e`'s coordinate of a boundary row.
fn add_coeff(
    row: &mut [Frac],
    index: &BTreeMap<(usize, usize), usize>,
    e: (usize, usize),
    coeff: i128,
) {
    let i = index[&e];
    row[i] = row[i].add(Frac::int(coeff));
}

/// Union–find root with path halving.
fn uf_find(parent: &mut [usize], x: usize) -> usize {
    let mut x = x;
    while parent[x] != x {
        parent[x] = parent[parent[x]];
        x = parent[x];
    }
    x
}

/// The number of weakly-connected components of a digraph on the dense vertex set `0..n` (edges read
/// undirected), via union–find.
fn weakly_connected_components(n: usize, edges: &[(usize, usize)]) -> usize {
    if n == 0 {
        return 0;
    }
    let mut parent: Vec<usize> = (0..n).collect();
    for &(u, v) in edges {
        let (a, b) = (uf_find(&mut parent, u), uf_find(&mut parent, v));
        if a != b {
            parent[a] = b;
        }
    }
    let mut roots: BTreeSet<usize> = BTreeSet::new();
    for i in 0..n {
        let r = uf_find(&mut parent, i);
        roots.insert(r);
    }
    roots.len()
}

/// Deduplicate + drop self-loops, then reindex to a dense touched-vertex set `0..n`. Returns the dense
/// edges and `n`.
fn normalize(raw_edges: &[(usize, usize)]) -> (Vec<(usize, usize)>, usize) {
    let mut simple: BTreeSet<(usize, usize)> = BTreeSet::new();
    for &(u, v) in raw_edges {
        if u != v {
            simple.insert((u, v));
        }
    }
    let mut verts: BTreeSet<usize> = BTreeSet::new();
    for &(u, v) in &simple {
        verts.insert(u);
        verts.insert(v);
    }
    let idx: BTreeMap<usize, usize> = verts.iter().enumerate().map(|(i, &v)| (v, i)).collect();
    let edges: Vec<(usize, usize)> = simple.iter().map(|&(u, v)| (idx[&u], idx[&v])).collect();
    (edges, verts.len())
}

/// Compute GLMY **path homology** H₀/H₁ of a directed edge slice.
///
/// Edges are given as `(src, dst)` node-index pairs (any integer indices; they are deduplicated,
/// self-loops are dropped, and the touched vertices are densely reindexed). See the module docs for the
/// construction of `Ω₂` and the `dim H₁ = m − n + c − rank(∂₂)` formula.
#[must_use]
pub fn path_homology(raw_edges: &[(usize, usize)]) -> PathHomology {
    let (edges, n) = normalize(raw_edges);
    let m = edges.len();

    let edge_index: BTreeMap<(usize, usize), usize> =
        edges.iter().enumerate().map(|(i, &e)| (e, i)).collect();
    let edge_set: BTreeSet<(usize, usize)> = edges.iter().copied().collect();
    let mut out: Vec<Vec<usize>> = vec![Vec::new(); n];
    for &(u, v) in &edges {
        out[u].push(v);
    }

    let c = weakly_connected_components(n, &edges);

    // Build a basis of Ω₂ as boundary rows (each row is the ∂ of one Ω₂ generator, in the edge basis).
    let mut rows: Vec<Vec<Frac>> = Vec::new();
    // Bad endpoint-pairs (i,k) with i≠k, i→k ∉ E: collect their length-2 routes for the difference basis.
    let mut bad_routes: BTreeMap<(usize, usize), Vec<usize>> = BTreeMap::new();

    for (i, neigh) in out.iter().enumerate() {
        for &j in neigh {
            for &k in &out[j] {
                // 2-path i→j→k (regular: i≠j and j≠k hold since edges have distinct endpoints).
                let individually_invariant = i == k || edge_set.contains(&(i, k));
                if individually_invariant {
                    // ∂e_{ijk} = e_{ij} + e_{jk} − e_{ik}; the e_{ik} term vanishes when i == k.
                    let mut row = vec![Frac::zero(); m];
                    add_coeff(&mut row, &edge_index, (i, j), 1);
                    add_coeff(&mut row, &edge_index, (j, k), 1);
                    if i != k {
                        add_coeff(&mut row, &edge_index, (i, k), -1);
                    }
                    rows.push(row);
                } else {
                    bad_routes.entry((i, k)).or_default().push(j);
                }
            }
        }
    }

    // Per bad pair, the sum-zero (route difference) subspace has dimension (routes − 1): the e_{ik}
    // term cancels in each difference, leaving a genuine 1-path cycle.
    for (&(i, k), routes) in &bad_routes {
        let j0 = routes[0];
        for &jt in &routes[1..] {
            let mut row = vec![Frac::zero(); m];
            add_coeff(&mut row, &edge_index, (i, jt), 1);
            add_coeff(&mut row, &edge_index, (jt, k), 1);
            add_coeff(&mut row, &edge_index, (i, j0), -1);
            add_coeff(&mut row, &edge_index, (j0, k), -1);
            rows.push(row);
        }
    }

    let rank_d2 = rank_q(rows, m);
    // m − n + c ≥ 0 always (a spanning forest has n − c ≤ m edges), so this usize subtraction is safe.
    let cycle = (m + c) - n;
    let h1 = cycle.saturating_sub(rank_d2);

    PathHomology {
        vertices: n,
        edges: m,
        components: c,
        cycle_space_dim: cycle,
        rank_boundary2: rank_d2,
        h0: c,
        h1,
    }
}

/// The **undirected flag (clique) complex** H₁ of the same edge slice — the taxonomy's "flag-H₁"
/// contrast column.
///
/// Read *undirected*, triangles (3-cliques) are filled by 2-simplices, so a filled triangle carries no
/// H₁. Provided to witness chirality (a directed 3-cycle is a path-H₁ generator yet flag-trivial; a
/// boundary square is path-trivial yet flag-nontrivial) — the two opposite-signed witnesses that pin
/// the path-homology complex-choice.
#[must_use]
pub fn undirected_flag_h1(raw_edges: &[(usize, usize)]) -> usize {
    // Undirected simple edges, stored as (min, max).
    let mut uedges: BTreeSet<(usize, usize)> = BTreeSet::new();
    for &(a, b) in raw_edges {
        if a != b {
            uedges.insert(if a < b { (a, b) } else { (b, a) });
        }
    }
    let mut verts: BTreeSet<usize> = BTreeSet::new();
    for &(a, b) in &uedges {
        verts.insert(a);
        verts.insert(b);
    }
    let idx: BTreeMap<usize, usize> = verts.iter().enumerate().map(|(i, &v)| (v, i)).collect();
    let n = verts.len();
    let edges: Vec<(usize, usize)> = uedges.iter().map(|&(a, b)| (idx[&a], idx[&b])).collect();
    let m = edges.len();
    let edge_index: BTreeMap<(usize, usize), usize> =
        edges.iter().enumerate().map(|(i, &e)| (e, i)).collect();
    let eset: BTreeSet<(usize, usize)> = edges.iter().copied().collect();
    let c = weakly_connected_components(n, &edges);

    // ∂₂ over the 3-cliques i<j<k: ∂ = e_{jk} − e_{ik} + e_{ij} (oriented by ascending vertex index).
    let mut rows: Vec<Vec<Frac>> = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            if !eset.contains(&(i, j)) {
                continue;
            }
            for k in (j + 1)..n {
                if eset.contains(&(i, k)) && eset.contains(&(j, k)) {
                    let mut row = vec![Frac::zero(); m];
                    add_coeff(&mut row, &edge_index, (j, k), 1);
                    add_coeff(&mut row, &edge_index, (i, k), -1);
                    add_coeff(&mut row, &edge_index, (i, j), 1);
                    rows.push(row);
                }
            }
        }
    }
    let rank = rank_q(rows, m);
    let cycle = (m + c) - n;
    cycle.saturating_sub(rank)
}

// --- the tier filtration → confidence barcode ------------------------------------------------------

/// Keep only the edges at or above `min` tier (the downward threshold that builds a filtration level).
fn filter_tier(
    edges: &[(usize, usize, ResolutionTier)],
    min: ResolutionTier,
) -> Vec<(usize, usize)> {
    edges
        .iter()
        .filter(|(_, _, t)| *t >= min)
        .map(|&(u, v, _)| (u, v))
        .collect()
}

/// The confidence barcode — path homology at each of the three nested filtration levels.
///
/// `G₃ ⊆ G₂ ⊆ G₁`: `G₃` keeps only `T3Mir` (proof-tier) edges, `G₂` adds `Resolved`, `G₁` adds
/// `Syntactic` (all edges). Reading down the levels: a cycle present already at `G₃` is a proof-grade
/// structural entanglement; one that appears only at `G₁` is a weak-evidence (syntactic-only) artifact.
/// See [`TierBarcode::h0_barcode`]/[`TierBarcode::h1_barcode`] for the dimension barcode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TierBarcode {
    /// `G₃` — edges at tier ≥ `T3Mir` (proof-tier only).
    pub g3: PathHomology,
    /// `G₂` — edges at tier ≥ `Resolved` (proof + resolved).
    pub g2: PathHomology,
    /// `G₁` — all edges (proof + resolved + syntactic).
    pub g1: PathHomology,
}

impl TierBarcode {
    /// Build the barcode from tier-stamped `(src, dst, tier)` edges.
    #[must_use]
    pub fn from_tiered_edges(edges: &[(usize, usize, ResolutionTier)]) -> Self {
        Self {
            g3: path_homology(&filter_tier(edges, ResolutionTier::T3Mir)),
            g2: path_homology(&filter_tier(edges, ResolutionTier::Resolved)),
            g1: path_homology(&filter_tier(edges, ResolutionTier::Syntactic)),
        }
    }

    /// Build the barcode natively from a [`ReachabilityRelation`], reading each edge's stored tier off
    /// the forward adjacency. The relation's dense node indices are used directly.
    #[must_use]
    pub fn from_relation(rel: &ReachabilityRelation) -> Self {
        let mut edges: Vec<(usize, usize, ResolutionTier)> = Vec::new();
        for i in 0..rel.len() {
            for &(j, tier) in rel.forward_adj(i) {
                edges.push((i, j, tier));
            }
        }
        Self::from_tiered_edges(&edges)
    }

    /// The H₀ dimension at each level, `[G₃, G₂, G₁]` — the merge-tier reading (fewer components =
    /// higher-confidence merges).
    #[must_use]
    pub const fn h0_barcode(&self) -> [usize; 3] {
        [self.g3.h0, self.g2.h0, self.g1.h0]
    }

    /// The H₁ dimension at each level, `[G₃, G₂, G₁]` — the entanglement barcode (a class present at
    /// `G₃` is proof-grade; classes appearing only lower are weak-evidence).
    #[must_use]
    pub const fn h1_barcode(&self) -> [usize; 3] {
        [self.g3.h1, self.g2.h1, self.g1.h1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === The ℚ-verified benign-vs-entangled taxonomy (notebook 022 §11.9), dogfooded born-red. ======
    //
    // Each test is a distinct row of the taxonomy table; together they PIN the GLMY path-homology
    // complex-choice against both the directed-flag and the undirected complex.

    /// bigon `a⇄b` (mutual recursion) → path-H₁ = 0. The regular reduction `e_{aa}=0` makes the two
    /// 2-paths `a→b→a`, `b→a→b` individually ∂-invariant, and their boundaries collapse the cycle.
    #[test]
    fn benign_bigon_is_trivial() {
        let g = path_homology(&[(0, 1), (1, 0)]);
        assert_eq!(g.h0, 1);
        assert_eq!(
            g.h1, 0,
            "mutual recursion is benign — path homology sees no H₁"
        );
    }

    /// transitive triangle `a→b→c` + chord `a→c` (helper + shortcut) → path-H₁ = 0 (the 2-path bounds).
    #[test]
    fn benign_transitive_triangle_is_trivial() {
        let g = path_homology(&[(0, 1), (1, 2), (0, 2)]);
        assert_eq!(g.h0, 1);
        assert_eq!(g.h1, 0, "a helper with a shortcut chord is benign");
    }

    /// directed 3-cycle `a→b→c→a` (no chord) → path-H₁ = 1 (a generator: genuine 3-way entanglement).
    #[test]
    fn directed_3cycle_generates_h1() {
        let g = path_homology(&[(0, 1), (1, 2), (2, 0)]);
        assert_eq!(g.h0, 1);
        assert_eq!(
            g.h1, 1,
            "a chord-free directed 3-cycle is genuine entanglement"
        );
    }

    /// boundary square `a→b→d`, `a→c→d` (no `a→d`) → path-H₁ = 0 (the two routes reconcile: the
    /// difference `e_{abd} − e_{acd}` is a filling 2-cell).
    #[test]
    fn boundary_square_is_trivial() {
        // a=0, b=1, c=2, d=3
        let g = path_homology(&[(0, 1), (1, 3), (0, 2), (2, 3)]);
        assert_eq!(g.h0, 1);
        assert_eq!(g.h1, 0, "a reconciling diamond is benign");
    }

    /// non-boundary square → path-H₁ = 1. Realized as the two-source/two-sink square on `{d,e,f,g}`
    /// (`d→e, d→g, f→e, f→g`): the SAME undirected 4-cycle as the boundary square, but reoriented so
    /// there is no length-2 path and hence no filling 2-cell — a genuine multi-route entanglement.
    ///
    /// NOTE (spec ambiguity resolved): the one-shot brief parenthesized this as `d→e→f, d→g→f`, but
    /// that is literally a *diamond* (two 2-paths d⇒f) = a boundary square = H₁ 0, which contradicts
    /// the test's own name and the §11.9 taxonomy. The taxonomy is authoritative ("non-boundary
    /// square/polygon → 1"), so the test uses a genuine non-boundary square.
    #[test]
    fn nonboundary_square_generates_h1() {
        // d=0, e=1, f=2, g=3 — sources {d,f}, sinks {e,g}.
        let g = path_homology(&[(0, 1), (0, 3), (2, 1), (2, 3)]);
        assert_eq!(g.h0, 1);
        assert_eq!(g.h1, 1, "a non-reconciling square is genuine entanglement");
    }

    /// Chirality is preserved with OPPOSITE signs across two witnesses — the pinning pair (§11.9):
    /// the directed 3-cycle is a path-H₁ generator but flag-trivial (1 vs 0); the boundary square is
    /// path-trivial but flag-nontrivial (0 vs 1). No single symmetrization can fake both directions.
    #[test]
    fn chirality_preserved() {
        let three_cycle = [(0, 1), (1, 2), (2, 0)];
        assert_eq!(path_homology(&three_cycle).h1, 1, "3-cycle: path-H₁ = 1");
        assert_eq!(
            undirected_flag_h1(&three_cycle),
            0,
            "3-cycle: undirected flag-H₁ = 0 (the triangle fills)"
        );

        let boundary_square = [(0, 1), (1, 3), (0, 2), (2, 3)];
        assert_eq!(
            path_homology(&boundary_square).h1,
            0,
            "boundary square: path-H₁ = 0"
        );
        assert_eq!(
            undirected_flag_h1(&boundary_square),
            1,
            "boundary square: undirected flag-H₁ = 1 (no triangle to fill the 4-cycle)"
        );
    }

    /// The §11.9 worked barcode: `(H₀, H₁) = (1,1), (2,1), (1,2)` across `G₃, G₂, G₁`.
    ///
    /// Construction — a `T3Mir` directed 3-cycle (proof-grade entanglement), a disjoint `Resolved`
    /// edge (a second component that appears at G₂), and two `Syntactic` edges that both bridge the
    /// two clusters (H₀: 2→1) and close a second independent cycle (H₁: 1→2):
    /// - `T3Mir`:     0→1, 1→2, 2→0   (the proof-grade 3-cycle)
    /// - `Resolved`:  3→4             (a disjoint resolved coupling)
    /// - `Syntactic`: 2→3, 4→0        (weak-evidence bridges closing the outer cycle)
    #[test]
    fn barcode_worked_example_11_9() {
        use ResolutionTier::{Resolved, Syntactic, T3Mir};
        let edges = [
            (0, 1, T3Mir),
            (1, 2, T3Mir),
            (2, 0, T3Mir),
            (3, 4, Resolved),
            (2, 3, Syntactic),
            (4, 0, Syntactic),
        ];
        let bc = TierBarcode::from_tiered_edges(&edges);

        // G₃ (T3Mir only): the 3-cycle alone → 1 component, 1 entanglement class.
        assert_eq!((bc.g3.h0, bc.g3.h1), (1, 1), "G₃ = (H₀,H₁) = (1,1)");
        // G₂ (+Resolved): the disjoint 3→4 edge adds a second component; still one cycle.
        assert_eq!((bc.g2.h0, bc.g2.h1), (2, 1), "G₂ = (H₀,H₁) = (2,1)");
        // G₁ (+Syntactic): the bridges merge to one component and close a second cycle.
        assert_eq!((bc.g1.h0, bc.g1.h1), (1, 2), "G₁ = (H₀,H₁) = (1,2)");

        assert_eq!(bc.h0_barcode(), [1, 2, 1]);
        assert_eq!(bc.h1_barcode(), [1, 1, 2]);
    }

    /// The barcode reads identically when sourced natively from a [`ReachabilityRelation`] — the
    /// integration path an organ actually uses.
    #[test]
    fn barcode_from_reachability_relation() {
        use ResolutionTier::{Resolved, Syntactic, T3Mir};

        use crate::base::facts::{EdgeFact, EdgeKind};
        use crate::node::StromaNodeId;
        use crate::node::cfg::CfgSet;
        use crate::node::digest::IdentityDigest;
        use crate::node::id::FqPath;

        fn node(name: &str) -> StromaNodeId {
            StromaNodeId {
                fq_path: FqPath {
                    path: name.to_string(),
                    tier: ResolutionTier::Resolved,
                },
                identity_digest: IdentityDigest::of_tokens(name.as_bytes()),
                cfg_set: CfgSet::default(),
            }
        }
        fn edge(s: &str, d: &str, tier: ResolutionTier) -> EdgeFact {
            EdgeFact {
                src: node(s),
                dst: node(d),
                kind: EdgeKind::Call,
                tier,
            }
        }

        // Same shape as the worked example, but the relation assigns its own dense indices by first
        // appearance — the barcode is index-labeling-invariant.
        let facts = [
            edge("a", "b", T3Mir),
            edge("b", "c", T3Mir),
            edge("c", "a", T3Mir),
            edge("d", "e", Resolved),
            edge("c", "d", Syntactic),
            edge("e", "a", Syntactic),
        ];
        let rel = ReachabilityRelation::from_edges(&facts);
        let bc = TierBarcode::from_relation(&rel);
        assert_eq!(bc.h0_barcode(), [1, 2, 1]);
        assert_eq!(bc.h1_barcode(), [1, 1, 2]);
    }
}
