//! The **affinity 2-vector** (v0.6, ADR-061) — MATURE's gradient, as a type.
//!
//! This is the **TYPE** the maturation organ climbs, NOT the engine that climbs it
//! (the targeted mutator / gradient walk is a later unit — P3.5). It is the missing
//! primitive the v0.4 charter assumed: a *height* a maturing draft can be ordered by.
//!
//! # Why a 2-vector, not a scalar (the locked design)
//!
//! The germinal-center metaphor (ADR-003, biology is load-bearing) says "affinity"
//! as if it were a single number (one antibody's `Kd` for one antigen). A code
//! fingerprint is not one binder against one target — it must **bind a defect
//! cluster** AND **spare clean siblings**, and those are *two objectives that trade
//! off*:
//!
//! - **BIND-TIGHT** ([`recall`](Affinity::recall)) — how much of the defect cluster
//!   the draft matches. Adding conjuncts that catch a missed member RAISES recall.
//! - **SPARE-CLEAN** ([`precision`](Affinity::precision)) — how much of the clean
//!   corpus the draft correctly spares. Over-fitting to the cluster (a "photocopy"
//!   draft, maximally specific) LOWERS precision against *novel* clean siblings.
//!
//! A single scalar "affinity" climbing to a "ceiling" silently picks a point on this
//! recall↔precision tradeoff and **hides the choice** — exactly the two
//! residual failure poles (`overfits-homogeneous-clone-pair` = over-matured photocopy;
//! the clustering-recall seam = under-matured), which are the two *ends of the same
//! unnamed axis*. Naming the axis is a free design move now; retrofitting a 2-objective
//! model after callers depend on a scalar `affinity` field is expensive (ADR-007,
//! structurally-guaranteed need). So MATURE optimizes and **reports** the 2-vector
//! (ADR-042 legibility — the human ratifying sees the tradeoff, not an opaque score),
//! and [`Objective`] names which pole a point favors.
//!
//! # The "ceiling" is the Pareto frontier, not a magic number
//!
//! Real affinity maturation does not climb forever to a hardcoded threshold — it
//! **stops when antigen is depleted** (less antigen remains to compete for; B-cells
//! stop being selected). The code cognate: keep maturing while a mutation still
//! **moves the frontier** (binds a previously-missed member OR spares a previously-
//! bound clean sibling *without worsening the other axis*); STOP when no
//! Pareto-improving mutation remains — the draft has reached the **Pareto frontier**
//! it can no longer leave. The non-arbitrary stopping signal is "the improvement-room
//! is depleted," expressed as [`Affinity::pareto_improves_on`] returning `false` for
//! every reachable mutation. This dissolves "who set the ceiling?": nobody — the
//! frontier is structural.
//!
//! # The 2-vector is itself an anti-Goodhart surface
//!
//! A scalar affinity is a Goodhart target: an optimizer climbs it by over-fitting
//! (more conjuncts → higher "affinity"), and the over-fit is *hidden* in the single
//! number. As a 2-vector, that same gaming is **visible**: an optimizer chasing
//! recall (more conjuncts) shows up as **precision-collapse** on the reported vector.
//! The tradeoff that a scalar conceals, the vector exposes (ADR-056 OQ2 /
//! endogenous-Goodhart lineage).
//!
//! # Composes, does not compete (ADR-002)
//!
//! Both axes are computed from machinery antigen already ships — no new matcher:
//! - recall reuses [`Fingerprint::matches`] over the cluster members;
//! - precision reuses the SAME spare-clean predicate the B-gate
//!   ([`evaluate`](crate::learn::self_tolerance::evaluate)) already runs, counted
//!   rather than short-circuited.
//!
//! So [`Affinity::measure`] is a *reader* over present substrate (the spine's
//! READ-LATENT tier), not a new generator.

use antigen_fingerprint::Fingerprint;

/// A draft's **affinity** to a defect cluster, as the (recall, precision) 2-vector
/// the maturation organ optimizes and reports (ADR-061).
///
/// Both fields are rates in `[0.0, 1.0]`. This is the *height* primitive the v0.4
/// charter assumed-away (it had the maturation LOOP and the STOP-test but no gradient
/// to climb): two drafts are comparable by [Pareto dominance](Affinity::dominates),
/// and the maturation "ceiling" is the Pareto frontier they can no longer leave.
///
/// **Not totally ordered.** Two affinities where one wins on recall and the other on
/// precision are genuinely *incomparable* — that incomparability is the whole point
/// (it is the tradeoff a scalar would hide). [`PartialOrd`] reflects this: it returns
/// `None` for incomparable points. There is deliberately **no `Ord`** — collapsing
/// the frontier to a total order is exactly the scalar mistake this type exists to
/// refuse.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct Affinity {
    /// **BIND-TIGHT** — the fraction of defect-cluster members the draft matches,
    /// in `[0.0, 1.0]`. `1.0` = the draft binds every cluster member (the recall
    /// ceiling). Rises as conjuncts are added to catch missed members.
    ///
    /// By construction of the anti-unifier a draft binds every site it was
    /// generalized from, so a freshly anti-unified draft starts at `recall == 1.0`;
    /// recall becomes informative once maturation mutates the draft (a mutation that
    /// tightens precision may *drop* a cluster member — the recall cost the human
    /// must see).
    pub recall: f64,
    /// **SPARE-CLEAN** — the fraction of clean-corpus items the draft correctly
    /// spares (does NOT match), in `[0.0, 1.0]`. `1.0` = the draft binds no clean
    /// item (the B-gate's `Spared` verdict — the safety floor). Falls as the draft
    /// over-fits toward a photocopy that matches a novel clean sibling.
    ///
    /// This is the *measured* form of the same spare-clean signal the B-gate refuses
    /// on: `precision < 1.0` means at least one clean item is bound — the autoimmune
    /// condition. The gate is the binary cliff; this is the continuous slope toward it.
    pub precision: f64,
}

/// **Deserialize enforces the clamp invariant at the type boundary** (ADR-065
/// harden, the moral-center P0 fix).
///
/// The `recall`/`precision` clamp (`clamp_rate`: `NaN → 0.0`, `±∞ → [0,1]`) is the
/// type's documented standing invariant — the "honest-labeling-at-the-default"
/// posture [`new`](Affinity::new) names. But a *derived* `Deserialize` populates the
/// `pub` fields RAW, so a persisted life-record carrying a non-finite float (e.g. a
/// hand-edited or format-permissive `Scored(Affinity)` event) would deserialize an
/// UNCLAMPED affinity — and a non-finite value is lethal downstream: `+∞` clears the
/// drift detector's finite `ε_cut`, fabricating a confident `Drift` that auto-forgets
/// a class (the conservatism-JOIN guards `UnderPowered`/`Indeterminate`, NOT a `Drift`
/// synthesized from garbage). Routing deserialization through [`new`](Affinity::new)
/// makes the documented invariant CODE-TRUE for every construction path, not just the
/// in-process constructors — a claimed invariant that the load path violates is
/// precisely the failure-class antigen exists to catch.
impl<'de> serde::Deserialize<'de> for Affinity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        // Deserialize the wire fields RAW, then route through `new` so the same clamp
        // every other construction path uses is applied on load. The shadow struct
        // matches the derived `Serialize` shape exactly (a `{recall, precision}` map),
        // so the serialize→deserialize round-trip is preserved for in-range values and
        // SANITIZED (never rejected) for out-of-range/non-finite ones.
        #[derive(serde::Deserialize)]
        struct RawAffinity {
            recall: f64,
            precision: f64,
        }
        let raw = RawAffinity::deserialize(deserializer)?;
        Ok(Self::new(raw.recall, raw.precision))
    }
}

/// Which objective an affinity 2-vector **favors** — the legibility label MATURE
/// reports alongside the vector (ADR-042) so a human ratifying sees the tradeoff a
/// scalar would have hidden.
///
/// This is a *read* of the vector's shape, never a hidden collapse to a scalar: it
/// names where on the recall↔precision tradeoff a draft sits, leaving the comparison
/// itself partial ([`Affinity::dominates`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Objective {
    /// Recall strictly exceeds precision — the draft leans toward **catching**
    /// (binds the cluster broadly, at some clean-sparing cost). The high-recall pole
    /// the unsafe core wants (feedback-homeostasis tissue-locality: leaky-toward-
    /// catching where a miss is expensive).
    Recall,
    /// Precision strictly exceeds recall — the draft leans toward **sparing** (quiet,
    /// over-specific). The high-precision pole generated/clean-leaning code wants.
    Precision,
    /// Recall and precision are equal — the draft sits on the diagonal, favoring
    /// neither objective.
    Balanced,
}

impl Affinity {
    /// The recall ceiling: binds the whole cluster AND spares the whole clean corpus
    /// (`recall == precision == 1.0`). The ideal a draft maturing toward the frontier
    /// approaches — a draft AT this point dominates every other affinity.
    pub const PERFECT: Self = Self {
        recall: 1.0,
        precision: 1.0,
    };

    /// Construct an affinity from two pre-computed rates, **clamping** each into
    /// `[0.0, 1.0]`.
    ///
    /// Clamping (not asserting) keeps the type total over `f64` inputs: a caller that
    /// computes a rate by division can never accidentally mint an out-of-range or
    /// `NaN`-poisoned affinity (a `NaN` clamps to `0.0`, the most-conservative rate —
    /// the honest-labeling-at-the-default invariant, the same posture as
    /// [`Provenance::DEFAULT`](crate::finding::Provenance::DEFAULT)).
    #[must_use]
    pub const fn new(recall: f64, precision: f64) -> Self {
        Self {
            recall: clamp_rate(recall),
            precision: clamp_rate(precision),
        }
    }

    /// **Measure** a draft's affinity by reading present substrate — the READ-LATENT
    /// constructor (ADR-002, compose-don't-compete; ADR-006, recognition-not-design).
    ///
    /// - `recall` = `|cluster members the draft matches| / |cluster|` via
    ///   [`Fingerprint::matches`].
    /// - `precision` = `|clean items the draft spares| / |clean_corpus|`, counting
    ///   the SAME spare-clean signal the B-gate refuses on (a clean item the draft
    ///   matches is an autoimmune bind — a precision loss).
    ///
    /// An **empty** axis yields the conservative `0.0` for that axis (no evidence is
    /// not perfect evidence — the same floor-not-ceiling default as
    /// [`new`](Self::new)): an empty cluster cannot demonstrate recall, an empty clean
    /// corpus cannot demonstrate precision. (A caller measuring a fresh anti-unified
    /// draft against its own non-empty cluster will see `recall == 1.0` by
    /// construction — see [`recall`](Self::recall).)
    #[must_use]
    pub fn measure(draft: &Fingerprint, cluster: &[syn::Item], clean_corpus: &[syn::Item]) -> Self {
        let recall = rate(
            cluster.iter().filter(|item| draft.matches(item)).count(),
            cluster.len(),
        );
        // A clean item the draft MATCHES is bound (autoimmune) → NOT spared. Precision
        // counts the spared (non-matching) items: the measured form of `spare_clean`.
        let spared = clean_corpus
            .iter()
            .filter(|item| !draft.matches(item))
            .count();
        let precision = rate(spared, clean_corpus.len());
        Self { recall, precision }
    }

    /// Pareto **dominance**: does `self` dominate `other`?
    ///
    /// `true` iff `self` is at-least-as-good on BOTH axes and strictly better on at
    /// least one — the standard strict-dominance relation. A draft that dominates
    /// another is unambiguously a better binder (no tradeoff was made to get there);
    /// a maturation step that produces a dominating affinity is a *free* improvement.
    #[must_use]
    pub const fn dominates(&self, other: &Self) -> bool {
        let ge_both = self.recall >= other.recall && self.precision >= other.precision;
        let gt_one = self.recall > other.recall || self.precision > other.precision;
        ge_both && gt_one
    }

    /// Does maturing FROM `prev` TO `self` move the Pareto frontier?
    ///
    /// `true` iff `self` improves at least one axis **without worsening the other** —
    /// the maturation organ's STOPPING RULE inverted: keep maturing while some
    /// reachable mutation `pareto_improves_on` the current draft; STOP when none does
    /// (the **antigen-depletion** analog — the improvement-room is depleted and the
    /// draft has reached a frontier it can no longer leave).
    ///
    /// Distinct from [`dominates`](Self::dominates) only in direction-of-call: this is
    /// the gradient-step predicate (does this *move* help?), dominance is the
    /// comparison (is this point *better*?). They share the no-axis-worsened core; a
    /// step that `dominates` the previous draft trivially `pareto_improves_on` it.
    #[must_use]
    pub const fn pareto_improves_on(&self, prev: &Self) -> bool {
        self.dominates(prev)
    }

    /// The objective this 2-vector **favors** (ADR-042 legibility) — `Recall` if it
    /// leans toward catching, `Precision` if toward sparing, `Balanced` on the
    /// diagonal. A *read* of the vector's shape, never a scalar collapse.
    #[must_use]
    pub const fn favors(&self) -> Objective {
        if self.recall > self.precision {
            Objective::Recall
        } else if self.precision > self.recall {
            Objective::Precision
        } else {
            Objective::Balanced
        }
    }
}

/// Pareto partial order over affinities (deliberately partial — incomparable points
/// return `None`).
///
/// `a <= b` iff `b` dominates-or-equals `a` on BOTH axes; two points where each wins
/// one axis are **incomparable** (`None`) — the tradeoff a scalar would hide, made a
/// first-class "these are not orderable" answer. There is intentionally no [`Ord`]:
/// a total order over the frontier is the scalar mistake this type refuses.
impl PartialOrd for Affinity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering;
        let r = self.recall.partial_cmp(&other.recall)?;
        let p = self.precision.partial_cmp(&other.precision)?;
        match (r, p) {
            (Ordering::Equal, Ordering::Equal) => Some(Ordering::Equal),
            // Both axes agree in direction (or one ties) → comparable that way.
            (Ordering::Less | Ordering::Equal, Ordering::Less | Ordering::Equal) => {
                Some(Ordering::Less)
            },
            (Ordering::Greater | Ordering::Equal, Ordering::Greater | Ordering::Equal) => {
                Some(Ordering::Greater)
            },
            // One axis up, the other down → genuinely incomparable (the frontier).
            _ => None,
        }
    }
}

/// Clamp an `f64` rate into `[0.0, 1.0]`, mapping `NaN` to the conservative `0.0`.
const fn clamp_rate(x: f64) -> f64 {
    if x.is_nan() { 0.0 } else { x.clamp(0.0, 1.0) }
}

/// A `matched / total` rate, with an empty denominator yielding the conservative
/// `0.0` (no evidence is not perfect evidence — floor, not ceiling).
///
/// The `usize → f64` casts are precision-lossless in practice: `matched <= total`
/// are cardinalities of a cluster / clean-corpus slice (a handful to thousands of
/// `syn::Item`s), never within astronomical orders of magnitude of `f64`'s 2^52
/// mantissa. The cast cannot lose a bit for any reachable corpus.
#[allow(clippy::cast_precision_loss)]
fn rate(matched: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        clamp_rate(matched as f64 / total as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn aff(recall: f64, precision: f64) -> Affinity {
        Affinity { recall, precision }
    }

    // --- the 2-vector is NOT a scalar: incomparability is first-class ---

    #[test]
    fn trades_off_points_are_incomparable() {
        // High recall / low precision vs low recall / high precision: each wins one
        // axis. The whole reason this is a vector — neither is "better"; the tradeoff
        // a scalar would have hidden is a first-class `None`.
        let catchy = aff(0.9, 0.4);
        let sparing = aff(0.4, 0.9);
        assert!(!catchy.dominates(&sparing));
        assert!(!sparing.dominates(&catchy));
        assert_eq!(catchy.partial_cmp(&sparing), None);
        assert_eq!(sparing.partial_cmp(&catchy), None);
    }

    #[test]
    fn dominance_requires_no_axis_worsened() {
        let base = aff(0.5, 0.5);
        // Strictly better on both → dominates.
        assert!(aff(0.6, 0.6).dominates(&base));
        // Better on one, tied on the other → dominates (free improvement).
        assert!(aff(0.6, 0.5).dominates(&base));
        assert!(aff(0.5, 0.6).dominates(&base));
        // Equal → does NOT strictly dominate.
        assert!(!base.dominates(&base));
        // Better on one but WORSE on the other → does NOT dominate (a tradeoff).
        assert!(!aff(0.9, 0.1).dominates(&base));
    }

    #[test]
    fn perfect_dominates_everything_below() {
        assert!(Affinity::PERFECT.dominates(&aff(0.99, 1.0)));
        assert!(Affinity::PERFECT.dominates(&aff(0.0, 0.0)));
        // PERFECT does not dominate itself (strict).
        assert!(!Affinity::PERFECT.dominates(&Affinity::PERFECT));
    }

    #[test]
    fn partial_ord_orders_dominated_pairs_and_refuses_the_frontier() {
        use std::cmp::Ordering;
        assert_eq!(
            aff(0.5, 0.5).partial_cmp(&aff(0.5, 0.5)),
            Some(Ordering::Equal)
        );
        assert_eq!(
            aff(0.4, 0.4).partial_cmp(&aff(0.6, 0.6)),
            Some(Ordering::Less)
        );
        assert_eq!(
            aff(0.6, 0.6).partial_cmp(&aff(0.4, 0.4)),
            Some(Ordering::Greater)
        );
        // tied-one-axis still comparable
        assert_eq!(
            aff(0.4, 0.5).partial_cmp(&aff(0.6, 0.5)),
            Some(Ordering::Less)
        );
        // the frontier: incomparable
        assert_eq!(aff(0.9, 0.1).partial_cmp(&aff(0.1, 0.9)), None);
    }

    // --- the frontier IS the ceiling (antigen-depletion stopping rule) ---

    #[test]
    fn pareto_improves_is_the_stopping_rule() {
        let draft = aff(0.6, 0.7);
        // A mutation that lifts precision without dropping recall → keep maturing.
        assert!(aff(0.6, 0.8).pareto_improves_on(&draft));
        // A mutation that lifts recall but DROPS precision → does NOT improve (it is a
        // lateral move along the frontier, not a climb); the organism does not take it
        // as a free step.
        assert!(!aff(0.8, 0.5).pareto_improves_on(&draft));
        // No change → no improvement → STOP (frontier reached, antigen depleted).
        assert!(!draft.pareto_improves_on(&draft));
    }

    // --- legibility: which objective does the point favor? (ADR-042) ---

    #[test]
    fn favors_reports_the_pole() {
        assert_eq!(aff(0.9, 0.4).favors(), Objective::Recall);
        assert_eq!(aff(0.4, 0.9).favors(), Objective::Precision);
        assert_eq!(aff(0.7, 0.7).favors(), Objective::Balanced);
    }

    // --- construction is total over f64 (clamp, never panic / poison) ---

    #[test]
    fn new_clamps_out_of_range_and_nan() {
        assert_eq!(Affinity::new(1.5, -0.5), aff(1.0, 0.0));
        assert_eq!(Affinity::new(f64::NAN, 0.5), aff(0.0, 0.5));
        assert_eq!(
            Affinity::new(f64::INFINITY, f64::NEG_INFINITY),
            aff(1.0, 0.0)
        );
    }

    // --- measure() reads present substrate (composes the matcher + spare-clean) ---

    #[test]
    fn measure_a_fresh_draft_binds_its_cluster_and_spares_clean() {
        // The shipped propose fixture: two panic-in-Drop guards + a clean sibling.
        let guard_a: syn::Item = syn::parse_quote! {
            impl Drop for GuardA { fn drop(&mut self) { self.0.take().unwrap(); } }
        };
        let guard_b: syn::Item = syn::parse_quote! {
            impl Drop for GuardB { fn drop(&mut self) { self.0.take().expect("x"); } }
        };
        let clean: syn::Item = syn::parse_quote! {
            impl Drop for CleanGuard { fn drop(&mut self) { self.0.take().ok(); } }
        };
        let cluster = [guard_a, guard_b];
        let clean_corpus = [clean];

        let draft = crate::learn::propose::anti_unify(&cluster)
            .expect("the homogeneous Drop cluster anti-unifies");
        let a = Affinity::measure(&draft, &cluster, &clean_corpus);

        // A fresh anti-unified draft binds every site it was generalized from (recall
        // 1.0) AND spares the clean sibling (precision 1.0 — the any_of arm is NoMatch
        // on `.ok()`). Both axes are exact integer ratios (n/n, 0/n), so the whole
        // 2-vector compares exactly to PERFECT.
        assert_eq!(a, Affinity::PERFECT);
    }

    #[test]
    fn measure_empty_axes_are_conservative_zero() {
        let draft = Fingerprint::parse("item = struct").expect("trivial fingerprint parses");
        let empty: [syn::Item; 0] = [];
        let a = Affinity::measure(&draft, &empty, &empty);
        // No cluster → no recall evidence → 0.0; no clean corpus → no precision
        // evidence → 0.0. Floor, not ceiling.
        assert_eq!(a, aff(0.0, 0.0));
    }

    #[test]
    fn measure_counts_an_autoimmune_bind_as_precision_loss() {
        // A bare-structural draft (`item = struct`) matches every struct — it binds the
        // "clean" structs, the autoimmune condition the precision axis measures.
        let draft = Fingerprint::parse("item = struct").expect("trivial fingerprint parses");
        let bound: syn::Item = syn::parse_quote! { struct Anything; };
        let other: syn::Item = syn::parse_quote! { struct AlsoAnything; };
        let clean_corpus = [bound, other];
        let a = Affinity::measure(&draft, &[], &clean_corpus);
        // Both clean items are bound → spared 0 of 2 → precision 0.0 (and recall 0.0,
        // empty cluster). The whole 2-vector is the conservative floor.
        assert_eq!(a, aff(0.0, 0.0));
    }

    #[test]
    fn serde_roundtrips() {
        let a = aff(0.625, 0.875);
        let json = serde_json::to_string(&a).expect("serialize");
        let back: Affinity = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(a, back);
    }

    /// ATK-HARDEN-AFFINITY-SERDE: `Deserialize` enforces the clamp invariant at the
    /// type boundary (the moral-center P0 root fix). A persisted life-record carrying
    /// a non-finite or out-of-range affinity must deserialize to the CLAMPED value
    /// (`NaN → 0.0`, `±∞ → [0,1]`, out-of-range pinned), NOT the raw poison — a raw
    /// `±∞` reaching the drift detector fabricates a confident `Drift` that
    /// auto-forgets a class. The clamp is the type's documented standing invariant; it
    /// must hold on the LOAD path, not only at `new()`.
    #[test]
    fn deserialize_enforces_the_clamp_invariant() {
        // A format that admits non-finite/out-of-range floats: a serde_json::Value
        // built from raw numbers (JSON proper has no Infinity literal, but the same
        // Deserializer path is exercised by any format that does — and direct numbers
        // out of `[0,1]` are the always-reachable case).
        let raw = serde_json::json!({ "recall": 5.0, "precision": -2.0 });
        let back: Affinity = serde_json::from_value(raw).expect("deserialize clamps, never errors");
        assert_eq!(
            back,
            aff(1.0, 0.0),
            "out-of-range affinity must clamp on deserialize (recall 5.0→1.0, \
             precision -2.0→0.0), not load raw — the invariant is enforced at the \
             type boundary, not just at new().",
        );

        // NaN via a float-permissive in-memory Deserializer (serde_json::Number cannot
        // hold NaN, so route the NaN through `from_str` of a non-JSON-but-serde format
        // is overkill; instead assert the constructor-equivalence the impl guarantees:
        // Deserialize == new(), and new() already clamps NaN→0.0 — covered by
        // `new_clamps_out_of_range_and_nan`. The boundary wiring is what this test
        // pins: the LOAD path routes through `new`, so every clamp `new` does, the
        // load path does too.
        let clamped_via_new = Affinity::new(f64::NAN, f64::INFINITY);
        assert_eq!(clamped_via_new, aff(0.0, 1.0));
    }
}
