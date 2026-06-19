//! ATK — HARDEN re-attack on the ADWIN detector + the conservatism-JOIN, fresh eyes.
//!
//! The shipped `atk_adwin_fusion_conservatism_join.rs` proves the JOIN holds when fed
//! a *manually-constructed* `UnderPowered`. It NEVER drives a degenerate trajectory
//! through the real `detect()` and asks: does `detect()` HONESTLY report blindness, or
//! can a degenerate input make it return a CONFIDENT `NoDrift` while structurally blind?
//! INV-ADWIN-1 ("UnderPowered is never suppressed") is only as strong as `detect()`'s
//! honesty on the inputs it actually receives. The conservatism-JOIN cannot save a
//! class if the channel lies "I see no drift" instead of "I am blind".
//!
//! Attack axis: NaN / +∞ / −∞ in the affinity trajectory. `Affinity`'s fields are
//! `pub` and `serde(derive)`'d — `Affinity::new`/`measure` clamp, but a directly-
//! constructed or deserialized `Affinity` is NOT clamped, so a poisoned value CAN
//! reach `detect()`. We drive the FULL `fused_classify` end-to-end with the lethal
//! bit-3 combination (`Obsolete` + undefended) and assert the moral-center invariant:
//! a structurally-blind trajectory must NEVER auto-forget.
//!
//! ID-chain: ATK-HARDEN-ADWIN-NN.

use antigen::learn::adwin::{DriftVerdict, detect};
use antigen::learn::affinity::Affinity;
use antigen::learn::discriminator::{ClassVerdict, fused_classify};
use antigen::learn::reader::SilentStatus;

/// Build a trajectory directly from (recall, precision) pairs WITHOUT clamping —
/// the unclamped path a `serde`-deserialized life-record or a direct struct literal
/// takes (the `pub` fields + derived `Deserialize` bypass `Affinity::new`'s clamp).
fn raw_traj(pairs: &[(f64, f64)]) -> Vec<Affinity> {
    pairs
        .iter()
        .map(|&(recall, precision)| Affinity { recall, precision })
        .collect()
}

// ── ATK-HARDEN-ADWIN-01: a long ALL-NaN trajectory must NOT be a confident NoDrift ─
//
// detect_floor_axis computes eps_cut from COUNTS only (not values), so on a long
// trajectory it judges itself "powered" (2·eps_cut < 1.0). It then scans splits;
// every observed_diff = (mean(w0) - mean(w1)).abs() is NaN; evidence = NaN - eps is
// NaN; `evidence >= 0.0` is FALSE so nothing fires; `decide` falls to the NoDrift arm
// with tightest_margin (-NaN).max(0.0) = 0.0. Net: detect() == NoDrift{0.0}. That is
// a CONFIDENT "no drift" on a trajectory that carries ZERO usable signal — a silent
// miscalibration. The honest verdict is UnderPowered (the channel cannot see).
#[test]
fn atk_harden_adwin_01_all_nan_trajectory_is_not_confident_no_drift() {
    let n = 64; // long enough that count-based eps_cut judges the window "powered"
    let traj = raw_traj(&vec![(f64::NAN, f64::NAN); n]);
    let verdict = detect(&traj, 0.05);
    assert!(
        !matches!(verdict, DriftVerdict::NoDrift { .. }),
        "ATK-HARDEN-ADWIN-01: a length-{n} ALL-NaN trajectory carries no usable signal, \
         yet detect() returned a CONFIDENT NoDrift — INV-ADWIN-1 says blindness must \
         never masquerade as no-drift. A NaN-poisoned channel must report UnderPowered \
         (cannot see), not NoDrift (saw nothing). Got: {verdict:?}",
    );
}

// ── ATK-HARDEN-ADWIN-02: the moral-center — NaN trajectory must NEVER auto-forget ──
//
// The end-to-end consequence: if detect() returns NoDrift on the NaN trajectory, then
// fuse_channels(NoDrift, Obsolete, defended=false) passes through bit-3 = Obsolete,
// which IS auto-forgettable. A poisoned trajectory + the lethal bit-3 combination
// silently auto-forgets a class the channel could not actually evaluate. The
// conservatism-JOIN must hold here too — a blind/poisoned channel cannot endorse an
// irreversible forget.
#[test]
fn atk_harden_adwin_02_nan_trajectory_never_drives_auto_forget() {
    let traj = raw_traj(&vec![(f64::NAN, f64::NAN); 64]);
    let verdict = fused_classify(
        &traj,
        SilentStatus::Obsolete,
        /* defended = */ false,
        0.05,
    );
    assert!(
        !verdict.is_auto_forgettable(),
        "ATK-HARDEN-ADWIN-02: a NaN-poisoned trajectory + (Obsolete, undefended) drove \
         the fused verdict to an AUTO-FORGETTABLE state ({verdict:?}). The moral center \
         must not auto-forget a class whose loud channel carried unusable (NaN) data — \
         a poisoned channel is a blind channel: RouteToHuman, never Obsolete.",
    );
    assert_ne!(
        verdict,
        ClassVerdict::Obsolete,
        "ATK-HARDEN-ADWIN-02: fused verdict reached Obsolete via a NaN trajectory.",
    );
}

// ── ATK-HARDEN-ADWIN-03: +∞ / −∞ in the trajectory ─────────────────────────────────
#[test]
fn atk_harden_adwin_03_infinity_trajectory_never_drives_auto_forget() {
    // A mix of +∞ then finite: mean(w0)=+∞, mean(w1)=finite ⇒ observed_diff=+∞ ⇒
    // evidence = +∞ - eps = +∞ ≥ 0 ⇒ this could FIRE a spurious Drift. A recall-Drift
    // + (Obsolete, undefended) ⇒ Obsolete (the corroborated-obsolescence cell) — an
    // auto-forget driven by a garbage +∞ value. Either way (spurious Drift OR confident
    // NoDrift) the moral center must hold: an ∞-poisoned channel cannot auto-forget.
    let mut pairs = vec![(f64::INFINITY, f64::INFINITY); 32];
    pairs.extend(vec![(0.1_f64, 0.1_f64); 32]);
    let traj = raw_traj(&pairs);
    let verdict = fused_classify(&traj, SilentStatus::Obsolete, false, 0.05);
    assert!(
        !verdict.is_auto_forgettable(),
        "ATK-HARDEN-ADWIN-03: an ∞-poisoned trajectory + (Obsolete, undefended) reached \
         an auto-forgettable verdict ({verdict:?}). A garbage ∞ value must not be read as \
         real drift-evidence that corroborates obsolescence and auto-forgets the class.",
    );
}

// ── ATK-HARDEN-ADWIN-04: detect() on +∞ must not fabricate a confident Drift ───────
//
// Tighter: assert detect() itself does not turn an ∞ step into a Drift verdict. An ∞
// mean-difference always clears any finite eps_cut, so a single ∞ value would
// fabricate a "change-point" the data does not contain.
#[test]
fn atk_harden_adwin_04_infinity_does_not_fabricate_drift() {
    let mut pairs = vec![(f64::INFINITY, 0.5_f64); 16];
    pairs.extend(vec![(0.0_f64, 0.5_f64); 16]);
    let traj = raw_traj(&pairs);
    let verdict = detect(&traj, 0.05);
    assert!(
        !matches!(verdict, DriftVerdict::Drift { .. }),
        "ATK-HARDEN-ADWIN-04: detect() turned a +∞ trajectory value into a CONFIDENT \
         Drift ({verdict:?}). An ∞ mean-difference clears every finite bound, so this is \
         a fabricated change-point from a single garbage value — not real evidence.",
    );
}

// ── ATK-HARDEN-ADWIN-05: a single real recall-crater on a LONG trajectory still fires ─
//
// Negative control / sanity: the detector must NOT be so defanged that it stops seeing
// real drift. A long trajectory with a genuine recall step-down (0.95 → 0.15) must
// FIRE a recall-Drift. (If it doesn't, any hardening I'd propose for the NaN case must
// not break this.)
#[test]
fn atk_harden_adwin_05_real_recall_crater_still_fires() {
    let mut pairs = vec![(0.95_f64, 0.9_f64); 40];
    pairs.extend(vec![(0.15_f64, 0.9_f64); 40]);
    let traj = raw_traj(&pairs);
    let verdict = detect(&traj, 0.05);
    assert!(
        matches!(verdict, DriftVerdict::Drift { .. }),
        "ATK-HARDEN-ADWIN-05 (negative control): a length-80 trajectory with a real \
         recall step-down 0.95→0.15 MUST fire a Drift. Got {verdict:?}. If this is red, \
         the detector is blind to real drift (a different, worse bug).",
    );
}
