//! ADWIN — the honest-blind batch drift-detector (ADR-065), the **loud-class** half
//! of CURATE's automatic decay-trigger.
//!
//! # What this organ is
//!
//! antigen needs to decide WHEN a learned class has gone obsolete (or is being
//! evaded) by watching its per-class affinity-trajectory for a downward change-point.
//! ADWIN (Bifet & Gavaldà 2007, the field-standard streaming concept-drift detector)
//! is the canonical mechanism. This module is antigen's **batch-pure** build of it —
//! NOT the crate's streaming `&mut self` struct (a forbidden second state-store that
//! desyncs from the append-only life-record; ADR-064). The detector is a PURE
//! derivation over the already-materialized
//! [`score_trajectory`](crate::learn::life_record::LifeRecord::score_trajectory).
//!
//! # `UnderPowered` is the SPINE (the Aristotelian move — ADR-065 T1)
//!
//! A change on a stream is detectable only above a statistical-power threshold;
//! below it, detection is mathematically impossible. At antigen's CURRENT scale
//! (classes have matured n≈4-8 times) the bound is DEAD: `2·ε_cut > 1.0` = the max
//! observable signal, so a *correct* detector CANNOT fire. Therefore
//! [`DriftVerdict::UnderPowered`] is not a corner case — it is the DEFAULT verdict
//! for every class today. The organ's entire v0.6 value is that it HONESTLY says
//! "I cannot yet see drift for this class, and here is exactly when I will be able to
//! (`n_star`, computed from the bound, no real data needed)." A detector that fires
//! zero and says-so is the correct, valuable v0.6 organ — and it is the SAME organ
//! that fires correctly once trajectories lengthen, with NO code change.
//!
//! **INV-ADWIN-1: `UnderPowered` is never suppressed.** No wildcard arm in
//! [`DriftVerdict`] processing may collapse it into `NoDrift`. Silence has two causes
//! — no-drift vs can't-see — and they are DISTINCT verdicts (a bare `bool` collapsing
//! them is the silent-miscalibration antigen exists to catch).
//!
//! # The floor→full regime-switch (ADR-065 T1+T5+A6)
//!
//! It is ONE [`DriftVerdict`] type, two regimes that [`detect`] dispatches between:
//!
//! - the **FLOOR** (rigorous ADWIN0, [`eps_cut_floor`]) — `δ'=δ/n`, all-n splits,
//!   Theorem-3.1-rigorous, returns `UnderPowered` while blind. Governs below the
//!   sample-count the normal-approximation needs.
//! - the **FULL** (variance-aware ADWIN2, [`eps_cut_full`] / [`ExpHistogram`]) — Eq
//!   3.1, `δ'=δ/ln n`, O(log n) bucket-cuts, normal-approximation. Governs once a
//!   class has accumulated enough maturations.
//!
//! The floor's `UnderPowered` verdict already carries `eps_cut` and `max_observable`;
//! the moment `eps_cut < max_observable` persistently, the class has crossed its
//! power threshold `n*` — the SAME signal the seam reads. The floor's power-guard IS
//! the seam trigger; no separate length-counter.
//!
//! **INV-ADWIN-2: the floor and full `δ'` are NOT interchangeable.** `δ/n` in the
//! full detector over-corrects (loses the sensitivity the EH structure buys); `δ/ln n`
//! in the floor under-corrects (a forgetting-storm). Each regime uses its own; a
//! born-red test asserts it.
//!
//! # The source-verified math (ADR-065 — VERBATIM, do NOT reconstruct from memory)
//!
//! Transcribed verbatim from the Bifet-Gavaldà 2007 PDF (§3.1/3.2 ADWIN0 rigorous
//! form Theorem 3.1, Eq. 3.1 variance-aware ADWIN2, §3.3 exponential-histogram
//! bucket-merge). The constants are load-bearing — getting one wrong is a silent
//! miscalibration. See [`eps_cut_floor`] / [`eps_cut_full`] / [`ExpHistogram`] for
//! the per-formula citations.

use crate::learn::affinity::Affinity;

/// The confidence parameter `δ` the synthetic-fixture suite uses (ADR-065).
///
/// Lower = more conservative (fires less). The detector takes `δ` as an argument so
/// callers can tighten it; this is the default the born-red fixtures pin.
pub const DEFAULT_DELTA: f64 = 0.05;

/// The exponential-histogram bucket-count parameter `M` (Bifet-Gavaldà §3.3).
///
/// "The paper's validated default." Keep ≤ `M` buckets of each size `2^i`; on the
/// `M+1`-th, merge the two oldest. The paper's worked trace uses `M=2`; the shipped
/// default is `M=5`.
pub const M_BUCKETS: usize = 5;

/// Which affinity axis a drift-verdict concerns.
///
/// The detector runs PER-AXIS and ORs the alarms (ADR-065): a scalarization (F1 /
/// mean) would hide a drift where one axis craters while the other compensates — the
/// exact interior-crater blindness.
/// WHICH axis drifted is decision-relevant: recall-drop routes to the red-queen
/// (evasion), precision-drop to the autoimmunity effector (over-broad binding).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Axis {
    /// **BIND-TIGHT** ([`Affinity::recall`]) — a downward change-point here is the
    /// red-queen / evasion signal (the class stopped catching its cluster).
    Recall,
    /// **SPARE-CLEAN** ([`Affinity::precision`]) — a downward change-point here is the
    /// autoimmunity signal (the class began binding clean code, over-broadening).
    Precision,
}

impl Axis {
    /// Read this axis's scalar out of an [`Affinity`] 2-vector.
    #[must_use]
    pub const fn of(self, a: &Affinity) -> f64 {
        match self {
            Self::Recall => a.recall,
            Self::Precision => a.precision,
        }
    }

    /// Both axes, in the order the per-axis-OR scans them.
    #[must_use]
    pub const fn both() -> [Self; 2] {
        [Self::Recall, Self::Precision]
    }
}

/// The outcome of a change-point test over an affinity-trajectory (ADR-065's sealed
/// verdict — the spine).
///
/// `UnderPowered` is the default at antigen's scale. Silence has two causes —
/// no-drift vs can't-see — and they are DISTINCT verdicts (INV-ADWIN-1). A bare
/// `bool` collapsing them is the silent-miscalibration antigen exists to catch.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum DriftVerdict {
    /// A change-point was found: the mean BEFORE `cut_index` differs from the mean
    /// AFTER by `observed_diff ≥ eps_cut` on `axis`. The automatic decay-trigger.
    ///
    /// `commit_sha` is the **herd-drift hook** (ADR-065 do-now): the change-point's
    /// commit-sha (not just the index) so the future cross-class herd-correlator
    /// (v0.7+) has a shared commit-time-axis to align N classes' change-points on.
    /// `None` at v0.6 — the `Scored` trajectory points do not yet carry shas (the
    /// honest-scope: the axis is RESERVED, populated by the caller when the
    /// life-record events gain commit-identity). See [`detect`]'s caller-mapping note.
    Drift {
        /// The index in the trajectory at which the older window ends and the newer
        /// begins (the detected change-point).
        cut_index: usize,
        /// Which affinity axis drifted (the per-axis-OR winner).
        axis: Axis,
        /// The observed `|μ_before − μ_after|` that cleared the bound.
        observed_diff: f64,
        /// The `ε_cut` the observed difference cleared (the bound at the firing split).
        eps_cut: f64,
        /// The change-point's commit-sha (the herd-drift hook) — `None` at v0.6.
        commit_sha: Option<String>,
    },
    /// No split cleared its bound: the trajectory is stationary within statistical
    /// power. `tightest_margin` = the smallest `eps_cut − observed_diff` over all
    /// tested splits (how close the closest split came to firing — a `NoDrift` that
    /// nearly fired is worth surfacing).
    NoDrift {
        /// The smallest `eps_cut − observed_diff` across tested splits (≥ 0).
        tightest_margin: f64,
    },
    /// **Structurally blind** — `eps_cut ≥ max_observable` (INV-ADWIN-1): the bound
    /// exceeds the maximum signal the trajectory could possibly show, so a correct
    /// detector CANNOT fire. SAYS SO rather than masquerading as `NoDrift`. Carries
    /// `n_star` = how many MORE maturations until the class becomes drift-observable
    /// (a real, actionable number computed from the bound — no real data needed).
    UnderPowered {
        /// The `ε_cut` at the most-powerful (balanced) split of the current window.
        eps_cut: f64,
        /// The maximum observable `|μ_before − μ_after|` (1.0 for a rate in [0,1]).
        max_observable: f64,
        /// `n*`: the trajectory length at which this axis first becomes
        /// drift-observable (`2·ε_cut ≤ max_observable` at the balanced split). The
        /// self-announcement reads this — "class X reaches power at maturation n*".
        n_star: usize,
    },
}

// ============================================================================
// The verified bounds (Bifet-Gavaldà 2007 — VERBATIM, the load-bearing constants)
// ============================================================================

/// Harmonic mean of the two window sizes — `m = 1/(1/n0 + 1/n1)` (Bifet-Gavaldà §3.2).
///
/// Algebraically identical to `n0·n1/(n0+n1)`. Returns `None` for an empty sub-window
/// (no split to test).
#[must_use]
#[allow(clippy::cast_precision_loss)] // window sizes are small (≤ trajectory length)
fn harmonic_m(n0: usize, n1: usize) -> Option<f64> {
    if n0 == 0 || n1 == 0 {
        return None;
    }
    let (n0, n1) = (n0 as f64, n1 as f64);
    Some(1.0 / (1.0 / n0 + 1.0 / n1))
}

/// **FLOOR — the rigorous ADWIN0 `ε_cut` (Bifet-Gavaldà §3.1/3.2, Theorem 3.1).**
///
/// `ε_cut = sqrt( (1/(2m)) · ln(4/δ') )`, with `δ' = δ/n` (the Bonferroni-style
/// correction over the O(n) split-points). **The constant inside `ln` is 4** (NOT 2 —
/// that is the full form's constant; INV-ADWIN-2). The guaranteed-DETECTABLE shift is
/// `2·ε_cut` (Theorem 3.1.2) — this factor-2 is why n≈8 is dead.
///
/// This is the FULLY-rigorous, distribution-free, verifiable bound (Theorem 3.1 holds
/// unconditionally) — the correct floor where antigen lacks the ~30 samples the
/// variance form's normal-approximation needs.
#[must_use]
#[allow(clippy::cast_precision_loss)] // n is a trajectory length, well within f64
pub fn eps_cut_floor(n0: usize, n1: usize, n: usize, delta: f64) -> Option<f64> {
    let m = harmonic_m(n0, n1)?;
    let delta_prime = delta / (n as f64); // δ' = δ/n — all-n splits (INV-ADWIN-2)
    Some(((1.0 / (2.0 * m)) * (4.0 / delta_prime).ln()).sqrt())
}

/// **FULL — the variance-aware ADWIN2 `ε_cut` (Bifet-Gavaldà Eq 3.1).**
///
/// `ε_cut = sqrt( (2/m)·σ²_W·ln(2/δ') ) + (2/(3m))·ln(2/δ')`, with `δ' = δ/ln(n)`
/// (only O(log n) bucket-boundary cutpoints are checked; INV-ADWIN-2). **The constant
/// inside `ln` is 2** (NOT 4 — do not copy the floor's). The additive Bernstein term
/// `(2/(3m))·ln(2/δ')` is **NOT optional** — it protects small windows (the normal
/// approximation fails there) and dropping it under-fires in exactly antigen's regime.
///
/// `sigma_sq_w` = the observed sample variance of the per-axis scalar values in W.
///
/// **Rigor caveat (ADR-065):** this is the NORMAL-APPROXIMATION form — "perfectly
/// valid in practice" but "not 100% rigorous," valid only above the sample-count the
/// CLT needs (~30, partially relaxed by the Bernstein term). Below that the floor's
/// rigorous bound governs.
#[must_use]
#[allow(clippy::cast_precision_loss)] // n is a trajectory length, well within f64
pub fn eps_cut_full(n0: usize, n1: usize, n: usize, sigma_sq_w: f64, delta: f64) -> Option<f64> {
    let m = harmonic_m(n0, n1)?;
    // δ' = δ/ln(n) — O(log n) cutpoints (INV-ADWIN-2). ln(n) is undefined/≤0 for
    // n < 2; the full regime is never entered there (the floor governs), but guard.
    let ln_n = (n as f64).ln();
    if ln_n <= 0.0 {
        return None;
    }
    let delta_prime = delta / ln_n;
    let variance_term = ((2.0 / m) * sigma_sq_w * (2.0 / delta_prime).ln()).sqrt();
    let bernstein_term = (2.0 / (3.0 * m)) * (2.0 / delta_prime).ln(); // NOT optional
    Some(variance_term + bernstein_term)
}

/// The maximum observable `|μ_before − μ_after|` for an affinity axis.
///
/// Both recall and precision are rates in `[0,1]`, so the mean-difference is bounded by
/// `1.0`. This is the signal `ε_cut` is compared against for the power-guard.
pub const MAX_OBSERVABLE: f64 = 1.0;

/// Sample mean of a slice (0.0 for an empty slice — only called on non-empty windows).
#[must_use]
#[allow(clippy::cast_precision_loss)] // slice len is a trajectory length
fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    xs.iter().sum::<f64>() / (xs.len() as f64)
}

/// Observed sample variance of a slice (population variance, the paper's `σ²_W`).
#[must_use]
#[allow(clippy::cast_precision_loss)] // slice len is a trajectory length
fn variance(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    let mu = mean(xs);
    xs.iter().map(|x| (x - mu).powi(2)).sum::<f64>() / (xs.len() as f64)
}

/// `n*` — how many points the trajectory needs before this axis becomes drift-observable.
///
/// The smallest `n` whose balanced split (`n0=n1=n/2`, the most-powerful split)
/// satisfies `2·ε_cut_floor ≤ MAX_OBSERVABLE`. Computed from the rigorous floor bound,
/// no real data needed (ADR-065's "self-announce power at n*").
///
/// Returns the **absolute** length (not the remaining count); the caller subtracts the
/// current length for "N more maturations". Bounded search (the bound is monotone
/// decreasing in n, so the first crossing is the answer); capped to avoid a runaway.
#[must_use]
pub fn power_threshold_n(delta: f64) -> usize {
    // The bound is monotone decreasing in n; scan even n (balanced split) upward to
    // the first n where 2·ε_cut ≤ MAX_OBSERVABLE. Cap at a generous ceiling — if the
    // bound never clears by then, δ is pathologically small and the class is blind.
    const CEILING: usize = 1_000_000;
    let mut n = 2usize;
    while n <= CEILING {
        if let Some(eps) = eps_cut_floor(n / 2, n / 2, n, delta) {
            if 2.0 * eps <= MAX_OBSERVABLE {
                return n;
            }
        }
        n += 2;
    }
    CEILING
}

// ============================================================================
// The floor detector (rigorous ADWIN0 over a single batch window) + power-guard
// ============================================================================

/// Run the FLOOR detector (rigorous ADWIN0) over one axis's scalar stream. Scans all
/// O(n) splits; fires on the first (the oldest cut, the most history to discard) whose
/// `|μ_W0 − μ_W1| ≥ ε_cut`. Returns the per-axis verdict — including the
/// `UnderPowered` power-guard when the bound exceeds the max observable signal.
///
/// The power-guard (INV-ADWIN-1): if EVERY split is under-powered
/// (`ε_cut ≥ MAX_OBSERVABLE`), the detector is structurally blind and returns
/// [`DriftVerdict::UnderPowered`] with `n*` — it does NOT silently return `NoDrift`.
/// The most-powerful split is the balanced one, so we report its `ε_cut`.
#[must_use]
fn detect_floor_axis(stream: &[f64], axis: Axis, delta: f64) -> DriftVerdict {
    let n = stream.len();
    // The balanced split's ε_cut is the tightest (most-powerful) — the power-guard reads it.
    let balanced_eps = eps_cut_floor(n / 2, n - n / 2, n, delta);

    // POWER-GUARD (INV-ADWIN-1): if even the most-powerful split can't observe the max
    // signal, the detector is blind — say so, with n*. Fewer than 2 points = no split.
    let powered = matches!(balanced_eps, Some(eps) if 2.0 * eps <= MAX_OBSERVABLE);
    if !powered {
        let eps_cut = balanced_eps.unwrap_or(f64::INFINITY);
        return DriftVerdict::UnderPowered {
            eps_cut,
            max_observable: MAX_OBSERVABLE,
            n_star: power_threshold_n(delta),
        };
    }

    // Powered: scan all splits. Among the splits that CLEAR their bound, fire on the
    // one with the strongest EVIDENCE (max `observed_diff − eps_cut`) — the
    // best-localized change-point.
    //
    // # Why max-evidence, not the oldest-clearing cut
    //
    // The paper's streaming ADWIN drops the tail W0 on *any* clearing cut (window
    // adaptation, not localization) and re-tests progressively. For antigen's BATCH
    // use the `cut_index` is a LOCALIZED output (it dates the decay candidate and
    // anchors the commit-sha hook), so the right cut is the one with the most
    // evidence: for a clean step at index k, an early cut blends the post-change tail
    // into W1 (a smaller diff that still clears the tiny large-n bound), which would
    // mis-localize the change. The max-evidence cut lands at the true change-point.
    // This is the standard batch change-point read; it preserves the FP guarantee
    // (we still only fire when SOME cut clears its δ-bounded ε_cut) while localizing.
    let best = best_split(stream, n, delta, eps_cut_floor_for);
    decide(best, axis)
}

/// A split's evaluation: its cut index, the observed mean-difference, the bound it was
/// tested against, and the evidence `observed_diff − eps_cut` (positive iff it fires).
struct SplitEval {
    cut_index: usize,
    observed_diff: f64,
    eps_cut: f64,
    /// `observed_diff − eps_cut`; the strongest positive value is the best change-point.
    evidence: f64,
}

/// Scan all O(n) interior splits with `bound`, returning the split with the strongest
/// evidence (and the tightest margin among non-firing splits, surfaced via the same
/// `SplitEval` when nothing fires). `None` only if no split is computable (n < 2).
fn best_split(
    stream: &[f64],
    n: usize,
    delta: f64,
    bound: impl Fn(usize, usize, usize, f64) -> Option<f64>,
) -> Option<SplitEval> {
    let mut best: Option<SplitEval> = None;
    for cut in 1..n {
        let (w0, w1) = stream.split_at(cut);
        let Some(eps_cut) = bound(w0.len(), w1.len(), n, delta) else {
            continue;
        };
        let observed_diff = (mean(w0) - mean(w1)).abs();
        let evidence = observed_diff - eps_cut;
        let candidate = SplitEval {
            cut_index: cut,
            observed_diff,
            eps_cut,
            evidence,
        };
        // Keep the split with the greatest evidence (most likely change-point; when
        // none fire, this is the closest-to-firing split — the tightest margin).
        best = Some(match best {
            Some(b) if b.evidence >= candidate.evidence => b,
            _ => candidate,
        });
    }
    best
}

/// Turn the best split into a verdict: `Drift` if it cleared its bound (evidence ≥ 0),
/// else `NoDrift` carrying how close the closest split came (`tightest_margin`).
fn decide(best: Option<SplitEval>, axis: Axis) -> DriftVerdict {
    match best {
        Some(s) if s.evidence >= 0.0 => DriftVerdict::Drift {
            cut_index: s.cut_index,
            axis,
            observed_diff: s.observed_diff,
            eps_cut: s.eps_cut,
            commit_sha: None, // RESERVED — caller maps cut_index → sha (ADR-065)
        },
        Some(s) => DriftVerdict::NoDrift {
            tightest_margin: (-s.evidence).max(0.0),
        },
        None => DriftVerdict::NoDrift {
            tightest_margin: 0.0,
        },
    }
}

/// Floor-bound adapter with the `(n0, n1, n, delta)` signature [`best_split`] expects.
fn eps_cut_floor_for(n0: usize, n1: usize, n: usize, delta: f64) -> Option<f64> {
    eps_cut_floor(n0, n1, n, delta)
}

// ============================================================================
// The exponential-histogram (ADWIN2 §3.3) — the FULL regime's window structure
// ============================================================================

/// One exponential-histogram bucket (Bifet-Gavaldà §3.3).
///
/// A power-of-2 number of elements (`capacity = 2^i`) and their summed content. Buckets
/// are kept newest-first within the histogram; a merge combines the two OLDEST of a
/// given size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bucket {
    /// The number of elements this bucket summarizes (always a power of 2).
    pub capacity: usize,
    /// The summed content (Σ of the elements) this bucket carries.
    pub content: f64,
}

/// The exponential-histogram window (ADWIN2 §3.3).
///
/// Buckets are stored **newest-first** (index 0 = the most recent element). Inserting
/// cascades merges of the two OLDEST buckets of each over-full size upward — the
/// structure that buys O(log n) memory and the O(log n) cutpoint set the full bound uses.
///
/// The bucket-merge is the GOLDEN-FIXTURE-tested core (ADR-065): the paper's own
/// worked trace (`Content 4,2,2,1,1` + new `1` → `4,2,2,2,1` → `4,4,2,1` at M=2) is a
/// born-red test — a wrong merge (newest-not-oldest, no-cascade, capacity-but-not-
/// content) fails it.
#[derive(Debug, Clone, PartialEq)]
pub struct ExpHistogram {
    /// Buckets newest-first (index 0 = most recent).
    buckets: Vec<Bucket>,
    /// `M` — the max bucket-count per size before a merge cascades.
    m: usize,
}

impl ExpHistogram {
    /// A new empty histogram with bucket-count parameter `m`.
    #[must_use]
    pub const fn new(m: usize) -> Self {
        Self {
            buckets: Vec::new(),
            m,
        }
    }

    /// Insert one element (content `x`, capacity 1) at the newest end, then cascade
    /// merges. Each over-full size (`m+1` buckets of size `2^i`) merges its two OLDEST
    /// into one of size `2^{i+1}` (adding capacity AND content), cascading upward.
    pub fn insert(&mut self, x: f64) {
        // New content-1 bucket at the newest end (front).
        self.buckets.insert(
            0,
            Bucket {
                capacity: 1,
                content: x,
            },
        );
        self.cascade_merge();
    }

    /// Cascade the bucket-merge (§3.3): while any size `2^i` has `m+1` buckets, merge
    /// its two OLDEST (the two highest-index buckets of that capacity) into one of size
    /// `2^{i+1}`. Repeats until every size has ≤ `m` buckets.
    fn cascade_merge(&mut self) {
        loop {
            // Find the smallest capacity that is over-full (has > m buckets).
            let mut over_full_cap: Option<usize> = None;
            // Count buckets per capacity in one pass over the (small) bucket list.
            let mut cap = 1usize;
            loop {
                let count = self.buckets.iter().filter(|b| b.capacity == cap).count();
                if count > self.m {
                    over_full_cap = Some(cap);
                    break;
                }
                if count == 0 && cap > self.max_capacity() {
                    break;
                }
                cap = cap.saturating_mul(2);
                if cap > self.max_capacity() && over_full_cap.is_none() {
                    break;
                }
            }
            let Some(cap) = over_full_cap else { break };

            // Merge the two OLDEST buckets of this capacity (oldest = highest index,
            // since buckets are newest-first). Find the two highest indices with `cap`.
            let mut idxs: Vec<usize> = self
                .buckets
                .iter()
                .enumerate()
                .filter(|(_, b)| b.capacity == cap)
                .map(|(i, _)| i)
                .collect();
            // idxs is ascending; the two OLDEST are the two LARGEST indices.
            let oldest = idxs
                .pop()
                .expect("over-full ⇒ ≥ m+1 ≥ 2 buckets of this size");
            let second_oldest = idxs.pop().expect("over-full ⇒ ≥ 2 buckets of this size");
            // Remove the higher index first so the lower index stays valid.
            let (hi, lo) = (oldest.max(second_oldest), oldest.min(second_oldest));
            let b_hi = self.buckets.remove(hi);
            let b_lo = self.buckets.remove(lo);
            // The merged bucket takes the OLDER bucket's slot (the higher original
            // index) — it summarizes the older half of the window. Insert at `lo`
            // (after removing both, `lo` is where the older-of-the-pair sat relative to
            // its newer neighbours; the merged bucket is older than everything that was
            // newer than the pair, so it belongs at the older end of that run).
            let merged = Bucket {
                capacity: b_hi.capacity + b_lo.capacity, // 2^i + 2^i = 2^{i+1}
                content: b_hi.content + b_lo.content,    // +content (NOT capacity-only)
            };
            self.buckets.insert(lo, merged);
        }
    }

    /// The largest capacity currently present (1 if empty) — bounds the cascade scan.
    fn max_capacity(&self) -> usize {
        self.buckets.iter().map(|b| b.capacity).max().unwrap_or(1)
    }

    /// The bucket list, newest-first (index 0 = most recent). The golden-trace fixture
    /// asserts the `content` sequence this returns.
    #[must_use]
    pub fn buckets(&self) -> &[Bucket] {
        &self.buckets
    }

    /// The total element count summarized (Σ capacities) — the window length `n`.
    #[must_use]
    pub fn len(&self) -> usize {
        self.buckets.iter().map(|b| b.capacity).sum()
    }

    /// Whether the histogram summarizes no elements.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.buckets.is_empty()
    }
}

// ============================================================================
// The public detector — per-axis OR, floor→full regime-switch
// ============================================================================

/// **The drift detector** (ADR-065) — a batch-pure change-point test over a trajectory.
///
/// Runs PER-AXIS (recall, precision) with `δ_axis = δ/2`
/// (Bonferroni over the two axes) and ORs the alarms — returning the FIRST axis that
/// fires (recall checked first, the red-queen signal never masked). If no axis fires
/// but ANY axis is under-powered, the OR returns `UnderPowered` (INV-ADWIN-1: a blind
/// axis is never collapsed into a confident `NoDrift`).
///
/// # Floor→full regime-switch
///
/// Per axis, [`detect`] dispatches by the trajectory length against the axis's power
/// threshold `n*`: below `n*` the rigorous FLOOR governs (and returns `UnderPowered`);
/// at/above `n*` the variance-aware FULL bound governs (sharper, normal-approximation,
/// valid because n is now large enough). The CALLER sees one function; only the
/// regime-switch lives inside.
///
/// # Caller-mapping note (the commit-sha hook)
///
/// `detect` reads `&[Affinity]`, which carries no commit-identity, so the returned
/// `Drift.commit_sha` is `None`. The herd-drift hook (ADR-065) is closed at the
/// CALLER: when the trajectory was assembled from a [`LifeRecord`]'s `Scored` events,
/// the caller knows which event (and thus which commit, once events carry shas) each
/// index maps to, and populates `commit_sha` before appending `LifeEvent::Drifted`.
/// The pure detector stays sha-free (no time-axis threaded through the math).
///
/// [`LifeRecord`]: crate::learn::life_record::LifeRecord
#[must_use]
pub fn detect(trajectory: &[Affinity], delta: f64) -> DriftVerdict {
    let delta_axis = delta / 2.0; // Bonferroni over the two axes (ADR-065)
    let mut under_powered: Option<DriftVerdict> = None;
    let mut tightest_no_drift = f64::INFINITY;

    for axis in Axis::both() {
        let stream: Vec<f64> = trajectory.iter().map(|a| axis.of(a)).collect();
        match detect_axis(&stream, axis, delta_axis) {
            v @ DriftVerdict::Drift { .. } => return v, // OR: first axis to fire wins
            DriftVerdict::UnderPowered { .. } if under_powered.is_none() => {
                under_powered = Some(detect_axis(&stream, axis, delta_axis));
            },
            DriftVerdict::UnderPowered { .. } => {},
            DriftVerdict::NoDrift { tightest_margin } => {
                tightest_no_drift = tightest_no_drift.min(tightest_margin);
            },
        }
    }

    // No axis fired. INV-ADWIN-1: a blind axis ⇒ UnderPowered (never a confident NoDrift).
    if let Some(up) = under_powered {
        return up;
    }
    DriftVerdict::NoDrift {
        tightest_margin: tightest_no_drift.max(0.0),
    }
}

/// One axis's detector with the floor→full regime-switch. Below the axis's power
/// threshold `n*` the FLOOR governs (rigorous, returns `UnderPowered`); at/above it the
/// FULL variance-aware bound governs.
#[must_use]
fn detect_axis(stream: &[f64], axis: Axis, delta: f64) -> DriftVerdict {
    let n = stream.len();
    let n_star = power_threshold_n(delta);
    if n < n_star {
        // FLOOR regime — rigorous, honest-blind. (Also handles n < 2: no split.)
        return detect_floor_axis(stream, axis, delta);
    }
    // FULL regime — variance-aware, normal-approximation valid (n ≥ n* ≳ 30).
    detect_full_axis(stream, axis, delta)
}

/// Run the FULL detector (variance-aware ADWIN2) over one axis's scalar stream. Same
/// best-evidence split read as the floor (see [`best_split`]), but with the
/// variance-aware [`eps_cut_full`] and `δ'=δ/ln n` (INV-ADWIN-2). Only entered at
/// `n ≥ n*`, where the normal-approximation is valid and the bound CAN fire — so no
/// power-guard arm.
#[must_use]
fn detect_full_axis(stream: &[f64], axis: Axis, delta: f64) -> DriftVerdict {
    let n = stream.len();
    let sigma_sq_w = variance(stream);
    // The variance bound carries σ²_W; close over it to fit the `best_split` signature.
    let bound =
        |n0: usize, n1: usize, n: usize, delta: f64| eps_cut_full(n0, n1, n, sigma_sq_w, delta);
    decide(best_split(stream, n, delta, bound), axis)
}

// ============================================================================
// The real/virtual fusion (the two-channel discriminator — INV-ADWIN-3)
// ============================================================================

/// The fused real/virtual-drift verdict — what CURATE acts on after the two-channel join.
///
/// Joins the ADWIN (loud, temporal) channel with the bit-3 static-shape channel
/// (ADR-065's safety-critical table). A single stream cannot split REAL from VIRTUAL
/// drift; the two channels together can.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FusedDrift {
    /// recall-drop + shape GONE ⇒ REAL drift, the class is OBSOLETE (forgettable,
    /// subject to ADR-057's reversible ladder).
    RealObsolete,
    /// recall-drop + shape PRESENT + a near-miss ⇒ REAL drift, the class is being
    /// EVADED (the red-queen signal — broaden / re-arm).
    RealEvading,
    /// precision-drop + clean-binds rising ⇒ REAL drift, the class is AUTOIMMUNE
    /// over-broadening.
    RealAutoimmune,
    /// recall-drop + shape PRESENT + no near-miss ⇒ VIRTUAL drift (churn). **KEEP** —
    /// the shape is alive and not being evaded; the drop is noise/churn, not death.
    VirtualKeep,
    /// **The conservatism-JOIN (INV-ADWIN-3 — the safety corner).** EITHER channel is
    /// blind — ADWIN `UnderPowered` OR bit-3 `Indeterminate` — so the fusion CANNOT
    /// safely decide. **CURATE HOLDS, never forgets**, regardless of the other
    /// channel. A virtual-drift cell must never fall through to forget when a channel
    /// is blind.
    Hold,
}

/// The static-shape channel's reading (the bit-3 sensor's contribution to the fusion,
/// reduced to the cells the table needs). This is the SECOND channel ADWIN joins with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaticChannel {
    /// The guarded shape is GONE from live code.
    ShapeGone,
    /// The shape is PRESENT and a near-miss appeared (the defect mutated just past it).
    ShapePresentNearMiss,
    /// The shape is PRESENT and no near-miss — alive, not evaded.
    ShapePresentNoNearMiss,
    /// Clean-binds are rising (the precision-drop's static corroboration — autoimmune).
    CleanBindsRising,
    /// **Blind** — the static channel cannot decide (bit-3 `Indeterminate`). Forces the
    /// conservatism-JOIN to [`FusedDrift::Hold`].
    Indeterminate,
}

/// **Fuse the two channels (INV-ADWIN-3 — the conservatism-JOIN).**
///
/// Joins the ADWIN temporal verdict with the bit-3 static-shape channel per ADR-065's
/// table. The hard constraint: if EITHER channel is blind (`UnderPowered` OR
/// `Indeterminate`), the result is [`FusedDrift::Hold`] — CURATE never forgets on a
/// blind channel.
///
/// This closes the autoimmune-forget cell: a `VirtualKeep` (churn) verdict is only
/// reached when BOTH channels are sighted AND agree the shape is alive-not-evaded.
#[must_use]
pub const fn fuse(adwin: &DriftVerdict, static_channel: StaticChannel) -> FusedDrift {
    // INV-ADWIN-3, half 1: ADWIN blind ⇒ HOLD, regardless of the static channel.
    if matches!(adwin, DriftVerdict::UnderPowered { .. }) {
        return FusedDrift::Hold;
    }
    // INV-ADWIN-3, half 2: static channel blind ⇒ HOLD, regardless of ADWIN.
    if matches!(static_channel, StaticChannel::Indeterminate) {
        return FusedDrift::Hold;
    }

    // Both channels sighted. Join by the table (ADR-065).
    match adwin {
        DriftVerdict::Drift {
            axis: Axis::Recall, ..
        } => match static_channel {
            StaticChannel::ShapeGone => FusedDrift::RealObsolete,
            StaticChannel::ShapePresentNearMiss => FusedDrift::RealEvading,
            // recall-drop + shape-present-no-near-miss is the churn/KEEP cell; recall-drop
            // + clean-binds-rising is not a table row, so the conservative read also KEEPS
            // (no obsolete/evading evidence on the recall axis).
            StaticChannel::ShapePresentNoNearMiss | StaticChannel::CleanBindsRising => {
                FusedDrift::VirtualKeep
            }
            StaticChannel::Indeterminate => FusedDrift::Hold, // unreachable (guarded above)
        },
        DriftVerdict::Drift {
            axis: Axis::Precision,
            ..
        } => match static_channel {
            StaticChannel::CleanBindsRising => FusedDrift::RealAutoimmune,
            // precision-drop without rising clean-binds: conservative keep.
            _ => FusedDrift::VirtualKeep,
        },
        // No drift on either axis (and both channels sighted) ⇒ nothing to act on; the
        // class is alive and stationary — KEEP.
        DriftVerdict::NoDrift { .. } => FusedDrift::VirtualKeep,
        // Guarded above, but the match must be exhaustive.
        DriftVerdict::UnderPowered { .. } => FusedDrift::Hold,
    }
}
