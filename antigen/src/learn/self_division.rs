//! SELF-DIVISION — the frontier-native three-verdict organ (split / leave-whole /
//! poised).
//!
//! # The question this organ answers
//!
//! A learned failure-class is anti-unified from a *cluster* of marked sites. Sometimes
//! that cluster is secretly **two** defect-classes wearing one coat: kept whole, a
//! single fingerprint must cover both, so it either over-generalizes (matches a clean
//! sibling — a **precision** collapse) or under-covers (drops a member — a **recall**
//! drop). Split into two lineages, each fingerprint tightly binds its own sub-class AND
//! spares clean code. The organ reads the cluster's [`Affinity`] geometry and returns
//! one of three verdicts:
//!
//! - [`SelfDivision::Split`] — no single Pareto-optimal point is both **tight**
//!   (recall→1) AND **complete** (precision→1), but a partition into two disjoint
//!   lineages **jointly** attains the tight-and-complete corner no single point reaches.
//! - [`SelfDivision::LeaveWhole`] — the corner IS reachable by one point
//!   ([`WholeReason::SinglePointReachable`]), OR the two candidate sub-lineages have
//!   **overlapping support** that cannot separate ([`WholeReason::OverlappingSupport`]),
//!   OR the split simply does not help ([`WholeReason::SplitDoesNotHelp`]).
//! - [`SelfDivision::Poised`] — the split closes the gap only within tolerance ε (the
//!   bifurcation boundary). Per Ashby's imported failure mode (eager escalation =
//!   instability, nb 026 §1), poised **routes to a human** ([`Disposition::RouteToHuman`])
//!   and **never auto-splits**.
//!
//! # Decided purely from 2-D Pareto geometry (the three assumption-flags)
//!
//! Every threshold in this module is a [`Affinity::partial_cmp`] against a **corner
//! target** `(1-ε, 1-ε)` — a Pareto dominance test, never a scalar. That is not a
//! stylistic choice; it is how the three correctness invariants are kept *structural*:
//!
//! - **A-SPLIT-1 (no total order).** The decision never collapses `(recall, precision)`
//!   into a single number (no `recall·w + precision·(1-w)`). [`reaches_corner`] asks
//!   whether a point Pareto-dominates-or-equals the corner — a high recall can **never**
//!   buy back a low precision, because dominance requires *both* axes. The type has no
//!   [`Ord`] and this organ adds none.
//! - **A-SPLIT-2 (overlap ≠ phase).** "Entangled / cannot separate" is the **measured**
//!   [`jaccard`] overlap of the two sub-lineages' member supports — a real quantity, not
//!   an unconstructed phase θ.
//! - **A-SPLIT-3 (ε at the bifurcation → human).** The poised case has an explicit
//!   tolerance ε ([`Classifier::epsilon`]) and defaults to [`Disposition::RouteToHuman`];
//!   [`Disposition::AutoSplit`] is reachable from [`SelfDivision::Split`] **alone**, so
//!   the "never auto-split a poised cluster" property is enforced by the type, not a
//!   comment.
//!
//! # The joint capability is the Pareto MEET, not a blend
//!
//! Running two fingerprints, the two-lineage system's guaranteed affinity is bounded by
//! the **weaker** lineage on each axis: the component-wise minimum
//! `(min recall, min precision)` — the [`meet`] in the affinity product lattice.
//! The meet is a Pareto operation (it lets no axis compensate for the other), so using
//! it honors A-SPLIT-1. It is the *conservative* joint estimate the split must clear:
//! if even the weaker-of-both point reaches the corner, the real joint system does at
//! least as well.

use std::cmp::Ordering;
use std::collections::BTreeSet;

use crate::learn::affinity::Affinity;

/// The three-verdict self-division read of a candidate cluster + proposed bipartition.
///
/// Decided from the affinity Pareto geometry alone (see the module docs). Route it to
/// the efferent hinge via [`disposition`](Self::disposition) — the poised case is
/// structurally incapable of producing an auto-split.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SelfDivision {
    /// **Split** — keep the cluster as two disjoint lineages. No single point on the
    /// whole-cluster frontier is both tight and complete, the two sub-lineages have
    /// separable support, and their Pareto meet reaches the tight-and-complete corner
    /// the whole frontier could not — a genuine joint improvement, not a marginal one.
    Split,
    /// **Leave whole** — do not divide. Carries the reason the whole cluster is the
    /// right unit (see [`WholeReason`]).
    LeaveWhole(WholeReason),
    /// **Poised** — the split lands on the bifurcation boundary: it closes the gap only
    /// within tolerance ε (either the split barely reaches the corner, or a single whole
    /// point nearly reached it alone). Undecidable from the geometry alone, so it routes
    /// to a human and never auto-splits.
    Poised,
}

/// Why a cluster should be [left whole](SelfDivision::LeaveWhole) rather than divided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WholeReason {
    /// A single point on the whole-cluster frontier already reaches the tight-and-
    /// complete corner within ε — there is no gap for a split to close.
    SinglePointReachable,
    /// The two candidate sub-lineages share too much member support (measured
    /// [`jaccard`] overlap above [`Classifier::max_overlap`]) — they are not two
    /// *disjoint* lineages, and forcing the separation would collapse precision as
    /// shared members are torn between them. The honest cash-value of "entangled".
    OverlappingSupport,
    /// The split's joint meet does not reach (nor come within the bifurcation band of)
    /// the corner — dividing gains nothing over keeping the cluster whole.
    SplitDoesNotHelp,
}

/// The efferent action a [`SelfDivision`] verdict routes to.
///
/// The reversible-first disposition, matching the `RouteToHuman` rung shared with
/// [`CurationAction`](crate::learn::curate::CurationAction) and
/// [`ClassVerdict`](crate::learn::discriminator::ClassVerdict).
///
/// [`AutoSplit`](Self::AutoSplit) is reachable from [`SelfDivision::Split`] **alone**;
/// [`SelfDivision::Poised`] maps to [`RouteToHuman`](Self::RouteToHuman). This makes
/// A-SPLIT-3 ("a poised cluster is never auto-split") a property of the mapping, not a
/// comment that could drift.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Disposition {
    /// Divide the cluster into the two proposed lineages (the only auto-acting exit).
    AutoSplit,
    /// Keep the cluster whole; take no dividing action.
    KeepWhole,
    /// Escalate to a human ratifier — the conservative default at the bifurcation
    /// boundary (ADR-057; Ashby's eager-escalation failure mode, nb 026 §1).
    RouteToHuman,
}

impl SelfDivision {
    /// The efferent [`Disposition`] this verdict routes to.
    ///
    /// The mapping is total and makes A-SPLIT-3 structural: only [`Split`](Self::Split)
    /// yields [`Disposition::AutoSplit`], so a [`Poised`](Self::Poised) cluster can never
    /// be auto-divided by any caller reading this disposition.
    #[must_use]
    pub const fn disposition(self) -> Disposition {
        match self {
            Self::Split => Disposition::AutoSplit,
            Self::LeaveWhole(_) => Disposition::KeepWhole,
            Self::Poised => Disposition::RouteToHuman,
        }
    }

    /// Whether this verdict escalates to a human (the poised bifurcation).
    #[must_use]
    pub const fn routes_to_human(self) -> bool {
        matches!(self.disposition(), Disposition::RouteToHuman)
    }
}

/// One proposed sub-lineage: its best achievable affinity plus the member support
/// (indices into the cluster) it covers.
///
/// `support` is the *measured* member set the overlap test reads (A-SPLIT-2). Indices
/// are opaque cluster-member handles; only their set-overlap matters here.
#[derive(Debug, Clone, PartialEq)]
pub struct SubLineage {
    /// The best `(recall, precision)` point this sub-lineage's fingerprint attains,
    /// measured against its own members and the shared clean corpus.
    pub affinity: Affinity,
    /// The cluster-member indices this sub-lineage's fingerprint covers.
    pub support: BTreeSet<usize>,
}

impl SubLineage {
    /// Construct a sub-lineage from an affinity and the member indices it covers.
    #[must_use]
    pub fn new(affinity: Affinity, support: impl IntoIterator<Item = usize>) -> Self {
        Self {
            affinity,
            support: support.into_iter().collect(),
        }
    }
}

/// The classifier's input: what the whole cluster can achieve, and the two-way split
/// under consideration.
#[derive(Debug, Clone, PartialEq)]
pub struct SplitProposal<'a> {
    /// The `(recall, precision)` points achievable while keeping the cluster **whole**
    /// (its sampled Pareto frontier). The corner is "single-point reachable" iff some
    /// point here reaches it.
    pub whole: &'a [Affinity],
    /// The first proposed sub-lineage.
    pub left: &'a SubLineage,
    /// The second proposed sub-lineage.
    pub right: &'a SubLineage,
}

/// The self-division classifier, parameterized by its two explicit tolerances.
///
/// Both are the correctness knobs the assumption-flags name: [`epsilon`](Self::epsilon)
/// is the corner tolerance ε (A-SPLIT-3) and [`max_overlap`](Self::max_overlap) is the
/// support-separability threshold (A-SPLIT-2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Classifier {
    /// **ε — the corner tolerance.** A point *reaches* the tight-and-complete corner
    /// iff it Pareto-dominates-or-equals `(1-ε, 1-ε)`. The **bifurcation band** is the
    /// annulus between the `ε`-box and the `2ε`-box: a split that reaches the corner
    /// only there (or a whole point that nearly reaches it there) is
    /// [`Poised`](SelfDivision::Poised). Clamped to `(0.0, 0.5]`.
    pub epsilon: f64,
    /// **The support-separability threshold.** Two sub-lineages are separable iff their
    /// [`jaccard`] support overlap is `<= max_overlap`; above it the cluster is
    /// [`OverlappingSupport`](WholeReason::OverlappingSupport). Clamped to `[0.0, 1.0]`.
    pub max_overlap: f64,
}

impl Default for Classifier {
    /// ε = `0.05` (reach within 5 points of the corner on **both** axes), separability
    /// at Jaccard `0.10` (supports must be ≥90% disjoint to count as two lineages).
    /// Deliberately conservative — both defaults bias toward
    /// [`LeaveWhole`](SelfDivision::LeaveWhole) / [`Poised`](SelfDivision::Poised) over
    /// [`Split`](SelfDivision::Split).
    fn default() -> Self {
        Self {
            epsilon: 0.05,
            max_overlap: 0.10,
        }
    }
}

impl Classifier {
    /// Construct a classifier, clamping `epsilon` into `(0.0, 0.5]` and `max_overlap`
    /// into `[0.0, 1.0]` so the corner target and band are always well-formed
    /// (`1-2ε >= 0`). `NaN` on either knob falls back to the [`Default`] value.
    #[must_use]
    pub fn new(epsilon: f64, max_overlap: f64) -> Self {
        let epsilon = if epsilon.is_nan() {
            Self::default().epsilon
        } else {
            epsilon.clamp(f64::MIN_POSITIVE, 0.5)
        };
        let max_overlap = if max_overlap.is_nan() {
            Self::default().max_overlap
        } else {
            max_overlap.clamp(0.0, 1.0)
        };
        Self {
            epsilon,
            max_overlap,
        }
    }

    /// Classify a [`SplitProposal`] into one of the three verdicts.
    ///
    /// The decision, in order (every reach test is a Pareto dominance against a corner
    /// target — A-SPLIT-1):
    ///
    /// 1. Some whole point reaches the `ε`-corner → [`SinglePointReachable`].
    /// 2. The supports overlap beyond `max_overlap` → [`OverlappingSupport`] (A-SPLIT-2).
    /// 3. The split meet reaches the `ε`-corner:
    ///    - a whole point *nearly* reached it alone (within the `2ε` band) → the gain is
    ///      marginal → [`Poised`](SelfDivision::Poised);
    ///    - otherwise the split jointly attains a corner no single point reached →
    ///      [`Split`](SelfDivision::Split).
    /// 4. The split meet reaches only the `2ε`-corner (the bifurcation band) →
    ///    [`Poised`](SelfDivision::Poised) (A-SPLIT-3 → human).
    /// 5. Otherwise the split does not help → [`SplitDoesNotHelp`].
    ///
    /// [`SinglePointReachable`]: WholeReason::SinglePointReachable
    /// [`OverlappingSupport`]: WholeReason::OverlappingSupport
    /// [`SplitDoesNotHelp`]: WholeReason::SplitDoesNotHelp
    #[must_use]
    pub fn classify(&self, proposal: &SplitProposal<'_>) -> SelfDivision {
        let eps = self.epsilon;
        let band = 2.0 * eps;

        // (1) The whole cluster already reaches the corner with one point → no gap.
        if proposal
            .whole
            .iter()
            .any(|point| reaches_corner(*point, eps))
        {
            return SelfDivision::LeaveWhole(WholeReason::SinglePointReachable);
        }

        // (2) The two candidate lineages are not actually disjoint (A-SPLIT-2). This
        // gates the split regardless of the meet: sharing members means the "two
        // disjoint lineages" premise is false, and separating them would tear shared
        // members between the halves (a precision collapse).
        if jaccard(&proposal.left.support, &proposal.right.support) > self.max_overlap {
            return SelfDivision::LeaveWhole(WholeReason::OverlappingSupport);
        }

        // The conservative joint capability of the two-lineage system: the Pareto meet
        // (weaker-of-both on each axis). Reaching the corner here means BOTH sub-lineages
        // are individually tight AND complete.
        let joint = meet(proposal.left.affinity, proposal.right.affinity);

        if reaches_corner(joint, eps) {
            // The split reaches the corner. Was the whole cluster already almost there
            // with a single point? If so the split's improvement is only marginal (within
            // the band) → bifurcation boundary → human.
            let whole_nearly = proposal
                .whole
                .iter()
                .any(|point| reaches_corner(*point, band));
            if whole_nearly {
                SelfDivision::Poised
            } else {
                SelfDivision::Split
            }
        } else if reaches_corner(joint, band) {
            // The split reaches only the 2ε-corner — it lands in the bifurcation band,
            // neither clearly reaching nor clearly failing (A-SPLIT-3 → human).
            SelfDivision::Poised
        } else {
            // The split's joint meet is nowhere near the corner — dividing gains nothing.
            SelfDivision::LeaveWhole(WholeReason::SplitDoesNotHelp)
        }
    }
}

/// Does `point` reach the tight-and-complete corner at tolerance `eps` — i.e. does it
/// **Pareto-dominate-or-equal** the target `(1-eps, 1-eps)`?
///
/// This is the load-bearing A-SPLIT-1 primitive: it asks the affinity's own partial
/// order whether the point is at-least-`corner` on **both** axes. Because dominance
/// requires both, a high recall can never compensate for a low precision — there is no
/// scalar collapse anywhere in the decision. An incomparable result (one axis above,
/// one below) is *not* reaching.
#[must_use]
pub fn reaches_corner(point: Affinity, eps: f64) -> bool {
    let corner = Affinity::new(1.0 - eps, 1.0 - eps);
    matches!(
        point.partial_cmp(&corner),
        Some(Ordering::Greater | Ordering::Equal)
    )
}

/// The Pareto **meet** of two affinities: the component-wise minimum
/// `(min recall, min precision)`.
///
/// The conservative joint capability of running both lineages — bounded by the weaker
/// on each axis. A lattice meet, not a blend: no axis compensates for the other, so this
/// honors A-SPLIT-1.
#[must_use]
pub const fn meet(a: Affinity, b: Affinity) -> Affinity {
    Affinity::new(a.recall.min(b.recall), a.precision.min(b.precision))
}

/// The **Jaccard overlap** of two member-support sets: `|A ∩ B| / |A ∪ B|`, in
/// `[0.0, 1.0]`.
///
/// The measured separability quantity of A-SPLIT-2 (overlap is a real count, not a
/// phase). Two empty supports have no union and are defined to have `0.0` overlap
/// (vacuously separable).
#[must_use]
#[allow(clippy::cast_precision_loss)] // support cardinalities are small (a handful of members)
pub fn jaccard(a: &BTreeSet<usize>, b: &BTreeSet<usize>) -> f64 {
    let intersection = a.intersection(b).count();
    let union = a.union(b).count();
    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn aff(recall: f64, precision: f64) -> Affinity {
        Affinity::new(recall, precision)
    }

    fn lineage(recall: f64, precision: f64, members: &[usize]) -> SubLineage {
        SubLineage::new(aff(recall, precision), members.iter().copied())
    }

    // ---------------------------------------------------------------------------
    // TEETH — one per verdict, born red (written before the module existed), plus a
    // negative control that must NOT split, plus the A-SPLIT-1 no-scalarization proof.
    // ---------------------------------------------------------------------------

    /// SPLIT — the whole frontier trades off (each point tight XOR complete, never
    /// both), the two sub-lineages have disjoint support, and their joint meet reaches
    /// the tight-and-complete corner no single whole point could.
    #[test]
    fn split_when_two_disjoint_lineages_jointly_reach_the_corner() {
        let c = Classifier::default();
        // Whole cluster: one point is tight-but-imprecise, the other precise-but-loose.
        // Neither reaches the corner — the tradeoff a single fingerprint cannot escape.
        let whole = [aff(1.0, 0.55), aff(0.55, 1.0)];
        let left = lineage(0.99, 0.99, &[0, 1, 2]);
        let right = lineage(0.98, 1.0, &[3, 4, 5]);
        let proposal = SplitProposal {
            whole: &whole,
            left: &left,
            right: &right,
        };
        assert_eq!(c.classify(&proposal), SelfDivision::Split);
        assert_eq!(c.classify(&proposal).disposition(), Disposition::AutoSplit);
    }

    /// LEAVE-WHOLE (single-point-reachable) — a single whole point already sits in the
    /// corner, so there is no gap for a split to close.
    #[test]
    fn leave_whole_when_a_single_point_reaches_the_corner() {
        let c = Classifier::default();
        let whole = [aff(0.98, 0.99)];
        // Even a "good-looking" split is moot: the whole already reaches the corner.
        let left = lineage(1.0, 1.0, &[0, 1]);
        let right = lineage(1.0, 1.0, &[2, 3]);
        let proposal = SplitProposal {
            whole: &whole,
            left: &left,
            right: &right,
        };
        assert_eq!(
            c.classify(&proposal),
            SelfDivision::LeaveWhole(WholeReason::SinglePointReachable)
        );
    }

    /// LEAVE-WHOLE (overlapping support) — the two proposed lineages share too many
    /// members to be *disjoint*; separating them would collapse precision (A-SPLIT-2).
    /// Note the meet still reaches the corner — the overlap gate fires anyway.
    #[test]
    fn leave_whole_when_support_overlaps_even_if_meet_reaches_corner() {
        let c = Classifier::default();
        let whole = [aff(1.0, 0.5), aff(0.5, 1.0)]; // no single point reaches
        let left = lineage(1.0, 1.0, &[0, 1, 2, 3]);
        let right = lineage(1.0, 1.0, &[2, 3, 4, 5]); // shares {2,3}: J = 2/6 ≈ 0.33 > 0.10
        let proposal = SplitProposal {
            whole: &whole,
            left: &left,
            right: &right,
        };
        assert_eq!(
            c.classify(&proposal),
            SelfDivision::LeaveWhole(WholeReason::OverlappingSupport)
        );
    }

    /// POISED (bifurcation band) — the split's joint meet reaches only the 2ε-corner,
    /// not the ε-corner: it lands right on the boundary → routes to a human, never
    /// auto-splits (A-SPLIT-3).
    #[test]
    fn poised_when_split_lands_in_the_bifurcation_band() {
        let c = Classifier::default(); // ε = 0.05, band to 0.10
        let whole = [aff(0.5, 0.5)]; // nowhere near the corner
        // meet = (0.92, 0.93): fails the 0.95-corner, clears the 0.90-corner → the band.
        let left = lineage(0.92, 0.96, &[0, 1, 2]);
        let right = lineage(0.94, 0.93, &[3, 4, 5]);
        let proposal = SplitProposal {
            whole: &whole,
            left: &left,
            right: &right,
        };
        assert_eq!(c.classify(&proposal), SelfDivision::Poised);
        assert!(c.classify(&proposal).routes_to_human());
        assert_eq!(
            c.classify(&proposal).disposition(),
            Disposition::RouteToHuman
        );
    }

    /// POISED (marginal improvement) — the split reaches the corner, but a single whole
    /// point *nearly* reached it alone (within the band). Dividing for so little is the
    /// eager-escalation failure mode → human, not an auto-split.
    #[test]
    fn poised_when_a_whole_point_nearly_reached_it_alone() {
        let c = Classifier::default();
        let whole = [aff(0.93, 0.93)]; // clears 0.90-corner, fails 0.95-corner → nearly
        let left = lineage(1.0, 1.0, &[0, 1, 2]);
        let right = lineage(0.99, 0.98, &[3, 4, 5]);
        let proposal = SplitProposal {
            whole: &whole,
            left: &left,
            right: &right,
        };
        assert_eq!(c.classify(&proposal), SelfDivision::Poised);
    }

    /// NEGATIVE CONTROL — a cluster that must NOT split: the whole cannot reach the
    /// corner AND the split's joint meet is nowhere near it either. Dividing gains
    /// nothing. If the classifier ever `Split`s here, the teeth above are toothless.
    #[test]
    fn negative_control_does_not_split_when_the_split_does_not_help() {
        let c = Classifier::default();
        let whole = [aff(0.5, 0.5)];
        // Disjoint supports (overlap gate passes), but both halves are mediocre — the
        // meet (0.6, 0.6) is far from the corner and even from the band.
        let left = lineage(0.7, 0.65, &[0, 1, 2]);
        let right = lineage(0.6, 0.6, &[3, 4, 5]);
        let proposal = SplitProposal {
            whole: &whole,
            left: &left,
            right: &right,
        };
        assert_eq!(
            c.classify(&proposal),
            SelfDivision::LeaveWhole(WholeReason::SplitDoesNotHelp)
        );
        assert_ne!(c.classify(&proposal), SelfDivision::Split);
    }

    /// A-SPLIT-1 — there is no scalarization path. A meet of `(1.0, 0.5)` has a
    /// *maximal* recall, yet must NEVER read as reaching the corner: a weighted-sum
    /// collapse `recall·w + precision·(1-w)` would let the high recall buy back the low
    /// precision, and this asserts it cannot. Reaching is Pareto dominance on BOTH axes.
    #[test]
    fn a_split_1_high_recall_never_compensates_low_precision() {
        // The primitive: a maxed recall with a low precision does not reach any corner,
        // no matter how loose (short of ε ≥ 0.5, out of the clamped range).
        assert!(!reaches_corner(aff(1.0, 0.5), 0.05));
        assert!(!reaches_corner(aff(1.0, 0.5), 0.4));
        // ...and it is genuinely *incomparable* to the corner (frontier), not "below".
        let corner = aff(0.95, 0.95);
        assert_eq!(aff(1.0, 0.5).partial_cmp(&corner), None);

        // End-to-end: a split whose joint meet leans hard on recall must not Split — the
        // precision axis is not buyable. Whole trades off; supports disjoint.
        let c = Classifier::default();
        let whole = [aff(1.0, 0.4), aff(0.4, 1.0)];
        let left = lineage(1.0, 0.5, &[0, 1, 2]); // huge recall, weak precision
        let right = lineage(1.0, 0.5, &[3, 4, 5]);
        let proposal = SplitProposal {
            whole: &whole,
            left: &left,
            right: &right,
        };
        assert_ne!(c.classify(&proposal), SelfDivision::Split);
    }

    // ---------------------------------------------------------------------------
    // Supporting properties.
    // ---------------------------------------------------------------------------

    #[test]
    fn reaches_corner_and_meet_agree_with_pareto_dominance() {
        // reaches_corner ⟺ the point dominates-or-equals the corner target.
        let corner = aff(0.9, 0.9);
        assert!(reaches_corner(aff(0.95, 0.9), 0.1)); // ge on both (tie on precision)
        assert!(reaches_corner(aff(0.9, 0.9), 0.1)); // exactly on the corner
        assert!(!reaches_corner(aff(0.95, 0.85), 0.1)); // precision below → incomparable
        assert_eq!(aff(0.95, 0.85).partial_cmp(&corner), None);

        // meet is the component-wise minimum.
        assert_eq!(meet(aff(0.8, 0.9), aff(0.7, 0.95)), aff(0.7, 0.9));
    }

    #[test]
    fn jaccard_measures_support_overlap() {
        let a: BTreeSet<usize> = [0, 1, 2, 3].into_iter().collect();
        let b: BTreeSet<usize> = [2, 3, 4, 5].into_iter().collect();
        assert!((jaccard(&a, &b) - 2.0 / 6.0).abs() < 1e-12);

        let disjoint: BTreeSet<usize> = [10, 11].into_iter().collect();
        assert!(jaccard(&a, &disjoint).abs() < 1e-12);

        let empty = BTreeSet::new();
        assert!(jaccard(&empty, &empty).abs() < 1e-12); // vacuously separable
    }

    #[test]
    fn new_clamps_degenerate_tolerances() {
        // ε clamps into (0, 0.5]; NaN falls back to the default.
        assert!((Classifier::new(1.5, 0.2).epsilon - 0.5).abs() < 1e-12);
        assert!(Classifier::new(-1.0, 0.2).epsilon > 0.0);
        assert!(
            (Classifier::new(f64::NAN, 0.2).epsilon - Classifier::default().epsilon).abs() < 1e-12
        );
        // max_overlap clamps into [0, 1].
        assert!((Classifier::new(0.05, 2.0).max_overlap - 1.0).abs() < 1e-12);
        assert!((Classifier::new(0.05, -1.0).max_overlap).abs() < 1e-12);
    }

    #[test]
    fn disposition_auto_split_is_reachable_from_split_alone() {
        // The A-SPLIT-3 structural property: only Split auto-acts.
        assert_eq!(SelfDivision::Split.disposition(), Disposition::AutoSplit);
        assert_eq!(
            SelfDivision::LeaveWhole(WholeReason::SinglePointReachable).disposition(),
            Disposition::KeepWhole
        );
        assert_eq!(
            SelfDivision::LeaveWhole(WholeReason::OverlappingSupport).disposition(),
            Disposition::KeepWhole
        );
        assert_eq!(
            SelfDivision::LeaveWhole(WholeReason::SplitDoesNotHelp).disposition(),
            Disposition::KeepWhole
        );
        assert_eq!(
            SelfDivision::Poised.disposition(),
            Disposition::RouteToHuman
        );
        assert!(!SelfDivision::Split.routes_to_human());
        assert!(SelfDivision::Poised.routes_to_human());
    }

    #[test]
    fn empty_whole_frontier_never_reports_single_point_reachable() {
        let c = Classifier::default();
        let whole: [Affinity; 0] = [];
        let left = lineage(1.0, 1.0, &[0, 1]);
        let right = lineage(1.0, 1.0, &[2, 3]);
        let proposal = SplitProposal {
            whole: &whole,
            left: &left,
            right: &right,
        };
        // No whole evidence of reachability → not SinglePointReachable; the disjoint,
        // corner-reaching split stands as a genuine Split.
        assert_eq!(c.classify(&proposal), SelfDivision::Split);
    }
}
