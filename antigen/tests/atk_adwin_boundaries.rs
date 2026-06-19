//! ATK-ADWIN-BOUNDARIES — the edges the fusion/detector contracts didn't pin.
//!
//! Hardening teeth found by dogfooding the ADWIN organ against its own edges (the
//! "checking-checking-checking" pass after the seam merged). Each pins a behavior at a
//! boundary where a SILENT MISCALIBRATION could hide — exactly the failure-class antigen
//! exists to surface. Grouped by the edge:
//!
//! - **δ-range safety** — δ is the false-positive guarantee; out-of-range δ must NOT
//!   silently break it (the load-bearing safety finding of this pass).
//! - **degenerate trajectories** — NaN / empty / single / two-point / all-equal.
//! - **oscillation ≠ drift** — a high-frequency sawtooth is stationary-in-the-mean, NOT
//!   a sustained change-point (a naive reader might expect N drifts; it's NoDrift).
//! - **the CLT / floor→full boundary** — monotone, honest across n≈30.
//! - **M=5 exp-histogram cascade** — conservation under long insert chains.

// Test-ergonomics allows (the workspace exempts test ergonomics): the fixtures name
// verdict types in prose without backticks, use a tiny non-const helper, and do hand
// arithmetic in the noise generator.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_lossless,
    clippy::float_cmp,
    clippy::doc_markdown,
    clippy::missing_const_for_fn,
    clippy::suboptimal_flops,
    clippy::default_trait_access
)]

use antigen::learn::adwin::{
    DEFAULT_DELTA, DriftAxis, DriftVerdict, ExpHistogram, detect, eps_cut_floor, eps_cut_full,
    power_threshold_n,
};
use antigen::learn::affinity::Affinity;

fn a(r: f64) -> Affinity {
    Affinity::new(r, 0.85)
}

/// A clear abrupt 0.9→0.4 recall step of total length `n` (the canonical fire fixture).
fn step(n: usize) -> Vec<Affinity> {
    let half = n / 2;
    (0..half)
        .map(|_| a(0.9))
        .chain((0..n - half).map(|_| a(0.4)))
        .collect()
}

// ============================================================================
// δ-RANGE SAFETY — an out-of-range confidence must NOT silently miscalibrate
// ============================================================================

/// **The safety finding:** δ ≥ 1 (invalid confidence) must NOT make the detector
/// over-fire. Un-clamped, δ=2.0 loosens the bound and fires a FALSE `Drift` on a
/// trajectory whose true δ-bounded verdict the clamp preserves — a false drift feeds a
/// wrongful forget-candidate (the dangerous direction for a decay-trigger). The clamp
/// caps δ at 0.5, so an absurd δ can never silently break the false-positive guarantee.
#[test]
fn atk_boundary_delta_ge_one_does_not_silently_over_fire() {
    // A STATIONARY noisy stream (no change-point). With a sane δ it is NoDrift/UnderPowered.
    // An un-clamped δ≥1 would loosen ε_cut enough to false-fire on the noise.
    let mut seed = 0x5151u64;
    let mut next = || {
        seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        ((seed >> 11) as f64) / ((1u64 << 53) as f64)
    };
    let noise: Vec<Affinity> = (0..400)
        .map(|_| a((0.9 + (next() * 2.0 - 1.0) * 0.05).clamp(0.0, 1.0)))
        .collect();
    // δ=2.0 is clamped to 0.5 internally — the detector must NOT fire on stationary noise.
    assert!(
        !matches!(detect(&noise, 2.0), DriftVerdict::Drift { .. }),
        "δ=2.0 (clamped to ≤0.5) must NOT over-fire on stationary noise — an out-of-range \
         confidence silently breaking the FP guarantee is the miscalibration antigen catches"
    );
}

/// δ ≤ 0 must NOT produce a NaN-poisoned verdict. Un-clamped, δ=-0.5 makes
/// `ln(4/δ')` = `ln(negative)` = NaN, and the verdict carries a NaN `eps_cut`. The
/// clamp floors δ at `f64::MIN_POSITIVE` (δ→0 ⇒ blind, the SAFE direction).
#[test]
fn atk_boundary_delta_le_zero_is_not_nan_poisoned() {
    for bad in [-0.5, 0.0, f64::NEG_INFINITY] {
        let v = detect(&step(400), bad);
        // Whatever it returns, no field may be NaN (the silent-miscalibration tell).
        match v {
            DriftVerdict::UnderPowered { eps_cut, .. } => {
                assert!(
                    !eps_cut.is_nan(),
                    "δ={bad}: UnderPowered.eps_cut must not be NaN"
                );
            },
            DriftVerdict::Drift {
                eps_cut,
                observed_diff,
                ..
            } => {
                assert!(
                    !eps_cut.is_nan() && !observed_diff.is_nan(),
                    "δ={bad}: Drift fields must not be NaN"
                );
            },
            DriftVerdict::NoDrift { tightest_margin } => {
                assert!(
                    !tightest_margin.is_nan(),
                    "δ={bad}: NoDrift.margin must not be NaN"
                );
            },
        }
    }
}

/// The clamp is INVISIBLE to valid callers: the canonical δ=0.05 is unchanged.
#[test]
fn atk_boundary_valid_delta_unaffected_by_clamp() {
    // A clear fire at the valid default — the clamp must not perturb in-range δ.
    assert!(
        matches!(
            detect(&step(400), DEFAULT_DELTA),
            DriftVerdict::Drift { .. }
        ),
        "the valid default δ=0.05 must still fire on a clear 0.5 step (clamp is a no-op here)"
    );
}

// ============================================================================
// DEGENERATE TRAJECTORIES — must be honestly blind, never panic, never NaN
// ============================================================================

#[test]
fn atk_boundary_empty_and_single_point_are_underpowered() {
    // No split is possible — the detector is structurally blind and SAYS SO.
    assert!(matches!(
        detect(&[], DEFAULT_DELTA),
        DriftVerdict::UnderPowered { .. }
    ));
    assert!(matches!(
        detect(&[a(0.9)], DEFAULT_DELTA),
        DriftVerdict::UnderPowered { .. }
    ));
}

#[test]
fn atk_boundary_nan_affinity_does_not_panic_or_poison() {
    // Affinity::new clamps NaN → 0.0 (the conservative rate), so a NaN input cannot
    // poison the trajectory. The detector reads a finite stream and is honestly blind
    // at this tiny length — never a panic, never a NaN verdict.
    let v = detect(&[a(f64::NAN), a(0.9), a(0.4)], DEFAULT_DELTA);
    assert!(
        matches!(v, DriftVerdict::UnderPowered { .. }),
        "a NaN-clamped short trajectory is honestly blind, got {v:?}"
    );
}

#[test]
fn atk_boundary_all_equal_is_no_drift_not_blind() {
    // A long flat (zero-variance) stream is STATIONARY — NoDrift (the detector looked
    // and saw no change), NOT UnderPowered (it CAN look at n=400) and NOT Drift.
    let flat: Vec<Affinity> = (0..400).map(|_| a(0.9)).collect();
    assert!(
        matches!(detect(&flat, DEFAULT_DELTA), DriftVerdict::NoDrift { .. }),
        "an all-equal long stream is stationary ⇒ NoDrift (looked, saw nothing)"
    );
}

// ============================================================================
// OSCILLATION ≠ DRIFT — high-frequency sawtooth is stationary-in-the-mean
// ============================================================================

/// A rapidly-oscillating sawtooth (period 10, 200 cycles over n=2000) is NOT sustained
/// drift — it is stationary in the mean. The detector must read NoDrift, NOT a storm of
/// Drifts. (A naive change-point reader that fired per local edge would over-trigger the
/// decay loop on a noisy-but-stable class — the exact forgetting-storm ADWIN's adaptive
/// window exists to avoid.)
#[test]
fn atk_boundary_high_frequency_oscillation_is_not_drift() {
    let saw: Vec<Affinity> = (0..2000)
        .map(|i| a(if (i / 10) % 2 == 0 { 0.9 } else { 0.2 }))
        .collect();
    let v = detect(&saw, DEFAULT_DELTA);
    assert!(
        matches!(v, DriftVerdict::NoDrift { .. }),
        "a high-frequency sawtooth is stationary-in-the-mean ⇒ NoDrift, not a drift-storm; got {v:?}"
    );
}

/// A GRADUAL monotone decline (0.9→0.4 over the whole window) DOES fire — the adaptive
/// detector catches slow drift, not only abrupt steps (the complement to oscillation).
#[test]
fn atk_boundary_gradual_decline_fires() {
    let grad: Vec<Affinity> = (0..400)
        .map(|i| a(0.9 - (i as f64) * 0.5 / 400.0))
        .collect();
    assert!(
        matches!(
            detect(&grad, DEFAULT_DELTA),
            DriftVerdict::Drift {
                axis: DriftAxis::Recall,
                ..
            }
        ),
        "a gradual monotone recall decline must fire (adaptive window catches slow drift)"
    );
}

// ============================================================================
// THE CLT / FLOOR→FULL BOUNDARY — monotone + honest across n≈30
// ============================================================================

#[test]
fn atk_boundary_clt_threshold_is_monotone_and_honest() {
    // Just below / at / above NORMAL_APPROX_MIN (30) a clear 0.5 step is still blind
    // (n*≈76 at δ_axis for a 0.5 shift), and it stays UnderPowered — the floor→full
    // switch does not spuriously fire at the regime boundary.
    for n in [28usize, 30, 32] {
        assert!(
            matches!(
                detect(&step(n), DEFAULT_DELTA),
                DriftVerdict::UnderPowered { .. }
            ),
            "n={n} (around the CLT boundary) must stay UnderPowered for a 0.5 step (n* is larger)"
        );
    }
    // Well past n* the SAME step fires — the same organ, no code change.
    assert!(matches!(
        detect(&step(400), DEFAULT_DELTA),
        DriftVerdict::Drift { .. }
    ));
}

#[test]
fn atk_boundary_eps_cut_guards_empty_subwindow() {
    // A zero-size sub-window has no harmonic mean — both bounds return None (no split).
    assert_eq!(eps_cut_floor(0, 4, 4, 0.05), None);
    assert_eq!(eps_cut_floor(4, 0, 4, 0.05), None);
    assert_eq!(eps_cut_full(0, 4, 4, 0.04, 0.05), None);
}

#[test]
fn atk_boundary_power_threshold_is_finite_and_sane() {
    // n* at the per-axis δ (0.025) is the ~76 the detector reports; δ→0 caps at the
    // search ceiling (blind, but finite — no infinite loop).
    assert_eq!(power_threshold_n(0.025), 76);
    assert_eq!(
        power_threshold_n(0.0),
        1_000_000,
        "δ=0 ⇒ never powered ⇒ the search ceiling, finite"
    );
}

// ============================================================================
// M=5 EXP-HISTOGRAM CASCADE — conservation under long insert chains
// ============================================================================

#[test]
fn atk_boundary_exp_histogram_m5_conserves_under_long_chain() {
    // 1000 unit inserts at the shipped M=5: content and capacity must be exactly
    // conserved (the merge ADDS, never drops — the capacity-but-not-content bug), every
    // capacity a power of 2, ≤ M=5 buckets per size, and O(M·log(n/M)) total buckets.
    let mut eh = ExpHistogram::new(5);
    for _ in 0..1000 {
        eh.insert(1.0);
    }
    let content: f64 = eh.buckets().iter().map(|b| b.content).sum();
    assert_eq!(
        content, 1000.0,
        "Σcontent == n (merge adds content, never drops)"
    );
    assert_eq!(eh.len(), 1000, "Σcapacity == n");
    let mut per_size: std::collections::BTreeMap<usize, usize> = Default::default();
    for b in eh.buckets() {
        assert!(
            b.capacity.is_power_of_two(),
            "every capacity is a power of 2"
        );
        *per_size.entry(b.capacity).or_default() += 1;
    }
    for (cap, count) in &per_size {
        assert!(count <= &5, "M=5: ≤ 5 buckets of size {cap}, got {count}");
    }
    // O(M·log(n/M)) memory: ~5·log2(200) ≈ 38; assert it's bounded well under n.
    assert!(
        eh.buckets().len() < 50,
        "EH is O(M·log(n/M)) memory, not O(n): {} buckets for n=1000",
        eh.buckets().len()
    );
}
