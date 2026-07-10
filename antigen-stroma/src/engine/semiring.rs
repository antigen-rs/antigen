//! The four semirings + the compile-gate that makes the non-idempotent (counting) path
//! unconstructible on a raw cyclic graph (ADR-068 R2/R3).
//!
//! A [`Semiring`] supplies the `(plus, times, zero, one)` monoid pair plus an `edge_weight` that maps
//! an edge's [`ResolutionTier`] into the semiring's weight. The reachability closure ([`run_closure`])
//! is generic over the semiring, so ONE closure computes detection / conductance / provenance by
//! swapping the dial (R1). The blast (counting) semiring is NON-idempotent — on a cyclic graph its
//! all-pairs closure diverges (~100000x blowup, measured) — so it is barred from [`run_closure`] and
//! admitted ONLY through [`run_condensed`], whose [`Condensed`] argument is a proof that an
//! SCC-condensation pre-pass ran (R3/R4).

use std::collections::HashMap;

use super::closure::ReachabilityRelation;
use crate::read::ResolutionTier;

/// A semiring: the `(plus, times, zero, one)` dial the reachability closure folds per node-pair.
///
/// `IDEMPOTENT` records whether `plus` is idempotent (`a ⊕ a == a`). An idempotent semiring's
/// all-pairs closure converges on a cyclic graph; a non-idempotent one (counting) does not, and is
/// gated behind [`Condensed`] via the sealed [`Idempotent`] marker (see the module note on the
/// stable realization of R3).
pub trait Semiring {
    /// The weight folded per node-pair (a `bool` for detection, a hop-count for conductance, a tier
    /// for provenance, a path-count for blast).
    type Weight: Clone;

    /// Whether `plus` is idempotent. `true` ⇒ closure-safe on cycles; `false` ⇒ condensation-only.
    const IDEMPOTENT: bool;

    /// The additive identity (`a ⊕ 0 == a`) — the "no path" weight.
    fn zero() -> Self::Weight;
    /// The multiplicative identity (`a ⊗ 1 == a`) — the "empty path" weight.
    fn one() -> Self::Weight;
    /// The additive fold — combine two alternative paths' weights (`min`, `max`, `||`, `+`).
    fn plus(a: Self::Weight, b: Self::Weight) -> Self::Weight;
    /// The multiplicative fold — combine two consecutive hops' weights along one path.
    fn times(a: Self::Weight, b: Self::Weight) -> Self::Weight;
    /// The weight a single edge contributes, given the tier it was reconstructed at.
    fn edge_weight(tier: ResolutionTier) -> Self::Weight;
}

/// A sealed marker for semirings whose `plus` is idempotent (closure-safe on cyclic graphs).
///
/// This is the STABLE realization of the R3 compile-gate: [`run_closure`] is bounded `S: Idempotent`,
/// so `run_closure::<Blast>` fails to compile (`Blast` does not implement `Idempotent`). The
/// non-idempotent counting path is thus unconstructible via the raw-graph closure — it must go through
/// [`run_condensed`]. (R3 literally names `where Assert<{ S::IDEMPOTENT }>: IsTrue`, which needs
/// nightly `generic_const_exprs` for a generic `S`; the marker trait binds the same invariant on
/// stable. The [`Assert`]/[`IsTrue`] types below enforce const/marker coherence.)
pub trait Idempotent: Semiring + sealed::Sealed {}

mod sealed {
    /// Sealed so only this module can declare a semiring idempotent (the marker is a load-bearing
    /// safety claim, not an open extension point).
    pub trait Sealed {}
}

/// A compile-time boolean assertion carrier (ADR-070 idiom): `Assert<true>` implements [`IsTrue`],
/// `Assert<false>` does not.
pub struct Assert<const B: bool>;

/// Implemented only by `Assert<true>` — a bound `Assert<{ C }>: IsTrue` holds iff `C` is `true`.
pub trait IsTrue {}
impl IsTrue for Assert<true> {}

/// A witness that an SCC-condensation pre-pass ran (R4): the ONLY door to a non-idempotent semiring.
///
/// [`run_condensed`] consumes a `Condensed<S>`, so a counting closure cannot be invoked without first
/// producing this proof (via the [`condense`](mod@super::condense) pre-pass). The raw-graph counting path is
/// unconstructible.
pub struct Condensed<S: Semiring>(pub S);

// --- Const/marker coherence (the Assert/IsTrue idiom, load-bearing) --------------------------------
// Every semiring marked `Idempotent` MUST have `IDEMPOTENT == true`, and the counting semiring MUST
// have `IDEMPOTENT == false`. These are CONCRETE-const args (not generic), so they are stable. If the
// marker trait and the const ever drift, this block fails to compile.
const _: () = {
    const fn assert_is_true<T: IsTrue>() {}
    // Bind each idempotent semiring's const to its marker (concrete-const args ⇒ stable).
    assert_is_true::<Assert<{ Detection::IDEMPOTENT }>>();
    assert_is_true::<Assert<{ Conductance::IDEMPOTENT }>>();
    assert_is_true::<Assert<{ Provenance::IDEMPOTENT }>>();
    // The counting semiring is NON-idempotent by construction.
    assert!(
        !Blast::IDEMPOTENT,
        "Blast (counting) must be non-idempotent — else the condensation gate is vacuous"
    );
};

/// Detection semiring — boolean reachability `(||, &&, false, true)` (R2). Idempotent.
#[derive(Debug, Clone, Copy, Default)]
pub struct Detection;

impl Semiring for Detection {
    type Weight = bool;

    const IDEMPOTENT: bool = true;

    fn zero() -> bool {
        false
    }

    fn one() -> bool {
        true
    }

    fn plus(a: bool, b: bool) -> bool {
        a || b
    }

    fn times(a: bool, b: bool) -> bool {
        a && b
    }

    fn edge_weight(_tier: ResolutionTier) -> bool {
        true
    }
}
impl sealed::Sealed for Detection {}
impl Idempotent for Detection {}

/// Conductance semiring — tropical-min hop-count `(min, +, ∞, 0)` (R2). Every edge weighs 1 hop; `∞`
/// is [`u64::MAX`]. Idempotent (`min` is idempotent).
#[derive(Debug, Clone, Copy, Default)]
pub struct Conductance;

impl Semiring for Conductance {
    type Weight = u64;

    const IDEMPOTENT: bool = true;

    fn zero() -> u64 {
        u64::MAX // ∞ — no path
    }

    fn one() -> u64 {
        0 // empty path — zero hops
    }

    fn plus(a: u64, b: u64) -> u64 {
        a.min(b)
    }

    fn times(a: u64, b: u64) -> u64 {
        a.saturating_add(b) // ∞ + x stays ∞
    }

    fn edge_weight(_tier: ResolutionTier) -> u64 {
        1 // one hop per edge (no schema change — R2)
    }
}
impl sealed::Sealed for Conductance {}
impl Idempotent for Conductance {}

/// Provenance semiring — lattice-JOIN over [`ResolutionTier`] `(max, max, Syntactic, T3Mir)` (R2): the
/// highest tier reachable on any path. Idempotent (`max` is idempotent).
///
/// Per R2 `one == T3Mir` (the lattice TOP). That makes `one` a lattice absorbing element under
/// `times == max`, not a true multiplicative identity, so a naive all-pairs star degenerates; the
/// [`crate::read::query::provenance_of`] query therefore computes the max-tier directly rather than
/// through [`run_closure`] (see the module note). The semiring is defined per spec and its ops are
/// unit-tested; it satisfies the "four semirings" commitment (R2).
#[derive(Debug, Clone, Copy, Default)]
pub struct Provenance;

impl Semiring for Provenance {
    type Weight = ResolutionTier;

    const IDEMPOTENT: bool = true;

    fn zero() -> ResolutionTier {
        ResolutionTier::Syntactic // ⊥ — the join identity
    }

    fn one() -> ResolutionTier {
        ResolutionTier::T3Mir // ⊤ per R2
    }

    fn plus(a: ResolutionTier, b: ResolutionTier) -> ResolutionTier {
        a.max(b)
    }

    fn times(a: ResolutionTier, b: ResolutionTier) -> ResolutionTier {
        a.max(b)
    }

    fn edge_weight(tier: ResolutionTier) -> ResolutionTier {
        tier
    }
}
impl sealed::Sealed for Provenance {}
impl Idempotent for Provenance {}

/// Blast semiring — counting `(+, *, 0, 1)` (R2). NON-idempotent.
///
/// It counts distinct paths, so on a cyclic graph its all-pairs closure diverges. It does NOT implement
/// [`Idempotent`] — hence `run_closure::<Blast>` does not type-check; blast runs only via
/// [`run_condensed`] on a DAG.
#[derive(Debug, Clone, Copy, Default)]
pub struct Blast;

impl Semiring for Blast {
    type Weight = u64;

    const IDEMPOTENT: bool = false;

    fn zero() -> u64 {
        0
    }

    fn one() -> u64 {
        1
    }

    fn plus(a: u64, b: u64) -> u64 {
        a.saturating_add(b)
    }

    fn times(a: u64, b: u64) -> u64 {
        a.saturating_mul(b)
    }

    fn edge_weight(_tier: ResolutionTier) -> u64 {
        1
    }
}
impl sealed::Sealed for Blast {}
// NOTE: Blast deliberately does NOT `impl Idempotent` — that omission is the compile-gate (R3).

/// The all-pairs reachability closure over an IDEMPOTENT semiring (R1) — a Floyd–Warshall/Kleene star
/// over the `(plus, times)` dial.
///
/// Returns the closure weight for every ordered node-index pair `(i, j)` (the reachability matrix
/// flattened to a map). Bounded `S: Idempotent`, so the non-idempotent counting path is unconstructible
/// here (`run_closure::<Blast>` fails to compile — R3); use [`run_condensed`] for that.
#[must_use]
pub fn run_closure<S>(rel: &ReachabilityRelation) -> HashMap<(usize, usize), S::Weight>
where S: Semiring + Idempotent {
    let n = rel.len();
    // dist[i][j] — seeded with reflexivity (empty path) + direct edges.
    let mut dist = vec![vec![S::zero(); n]; n];
    for (i, row) in dist.iter_mut().enumerate().take(n) {
        row[i] = S::plus(S::zero(), S::one());
    }
    // Seed direct edges. A range loop (not `iter().enumerate()`): the body indexes `dist[i][j]` at a
    // `j` drawn from `i`'s adjacency, not the outer element.
    #[allow(clippy::needless_range_loop)]
    for i in 0..n {
        for &(j, tier) in rel.forward_adj(i) {
            dist[i][j] = S::plus(dist[i][j].clone(), S::edge_weight(tier));
        }
    }
    // Kleene relaxation (idempotent ⇒ converges in one O(n^3) sweep for a closed semiring).
    for k in 0..n {
        for i in 0..n {
            for j in 0..n {
                let through = S::times(dist[i][k].clone(), dist[k][j].clone());
                dist[i][j] = S::plus(dist[i][j].clone(), through);
            }
        }
    }
    let mut out = HashMap::with_capacity(n * n);
    for (i, row) in dist.into_iter().enumerate() {
        for (j, w) in row.into_iter().enumerate() {
            out.insert((i, j), w);
        }
    }
    out
}

/// Run a semiring closure on a CONDENSED (acyclic) DAG — the ONLY door for a non-idempotent semiring.
///
/// R3/R4. The [`Condensed`] argument is the proof that an SCC-condensation pre-pass ran; `dag` is the
/// super-node adjacency (`dag[i]` = the super-node successors of super-node `i`), which is acyclic, so
/// even the counting semiring terminates.
///
/// Returns the per-super-node-pair closure weight. For [`Blast`] this is the number of distinct DAG
/// paths between super-nodes — finite because the graph is acyclic.
#[must_use]
pub fn run_condensed<S: Semiring>(
    _proof: Condensed<S>,
    dag: &[Vec<usize>],
) -> HashMap<(usize, usize), S::Weight> {
    let n = dag.len();
    let mut out = HashMap::with_capacity(n * n);
    // For each target j, memoize the closure weight of every source i via a DAG recursion. This avoids
    // the reflexive-diagonal double-count a Floyd–Warshall star inflicts on a NON-idempotent `plus`
    // (the whole reason blast must be condensed first). The DAG (no cycles) guarantees termination.
    for j in 0..n {
        let mut memo: Vec<Option<S::Weight>> = vec![None; n];
        for i in 0..n {
            let w = count_to::<S>(dag, i, j, &mut memo);
            out.insert((i, j), w);
        }
    }
    out
}

/// The semiring closure weight from `i` to `j` over an acyclic `dag`: `one` for the empty path when
/// `i == j`, plus the fold over each successor's weight-to-`j` (each super-edge weighs `one`). Memoized
/// per target; terminates because the graph is acyclic.
fn count_to<S: Semiring>(
    dag: &[Vec<usize>],
    i: usize,
    j: usize,
    memo: &mut [Option<S::Weight>],
) -> S::Weight {
    if let Some(w) = &memo[i] {
        return w.clone();
    }
    let mut acc = if i == j { S::one() } else { S::zero() };
    for k in 0..dag[i].len() {
        let s = dag[i][k];
        let sub = count_to::<S>(dag, s, j, memo);
        acc = S::plus(acc, S::times(S::one(), sub));
    }
    memo[i] = Some(acc.clone());
    acc
}
