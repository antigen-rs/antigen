//! ATK-ADWIN-FUSION — born-red attack on the ADWIN+bit-3 fusion conservatism-join.
//!
//! **STATUS: BORN-RED by non-compilation** — `antigen::learn::adwin` does not exist
//! yet (ADR-065 is ratified; the organ is the next build unit in `learn/adwin.rs`).
//! This file is the adversarial definition-of-done for the ADWIN builder: the safety
//! contracts these tests assert must ALL be green before the fusion layer ships.
//!
//! When `learn/adwin.rs` + the two-channel fusion function land:
//! 1. Drop the `#![cfg(adwin_built)]` gate below.
//! 2. Fix the imports to match the real module path / type names (the names here
//!    are the adversary's PROPOSED surface — rename in the same change if the
//!    builder chose different identifiers; the contracts are load-bearing, not
//!    the spellings).
//! 3. All tests should compile and pass green — if any fail, a fusion-safety contract
//!    is broken and the organ must NOT ship.
//!
//! # Why this is the highest-stakes gate after the moral center
//!
//! The moral-center gate (CURATE, `atk_curate_forget_path.rs`) proved that
//! `Forget` is reachable from `ClassVerdict::Obsolete` ONLY. But the two-channel
//! fusion layer is what PRODUCES `ClassVerdict::Obsolete`. A fusion bug that emits
//! `Obsolete` when a channel is blind bypasses the moral center entirely — the
//! gate holds, but the wrong key is handed to it.
//!
//! The conservatism-join (ADR-065, aristotle Phase 6 C2) states:
//! **if EITHER channel is blind (ADWIN `UnderPowered` OR bit-3 `Indeterminate`),
//! the fused verdict must NOT be `ClassVerdict::Obsolete`** — CURATE must HOLD,
//! never forget, regardless of what the other channel says.
//!
//! # Attack surface (five dimensions)
//!
//! 1. **Blind-channel forget** — can ANY combination of (`DriftVerdict::UnderPowered`,
//!    any `SilentStatus`) or (`any DriftVerdict`, `SilentStatus::Indeterminate`)
//!    produce `ClassVerdict::Obsolete`? If yes: a blind-channel autoimmune-forget.
//! 2. **Virtual-drift cell** — a recall-drop with the shape PRESENT and no near-miss
//!    is VIRTUAL drift (code churn, not the defect mutating). Must produce
//!    `ClassVerdict::Dormant`, NOT `Obsolete`.
//! 3. **`DriftVerdict` sealed enum** — `UnderPowered` must be a first-class variant,
//!    not a `bool`. Collapsing to `bool` merges "no-drift" with "can't-see",
//!    reopening the silent-miscalibration antigen exists to catch.
//! 4. **Interior-crater detection** — a 0.9→0.2→0.9 affinity trajectory that
//!    `trajectory_direction()` reads `Stable` (2-point blindness) MUST produce
//!    `DriftVerdict::Drift` from `detect()`. This is the payoff of full-ADWIN.
//! 5. **Power-guard boundary** — a short trajectory (n≈8) that is structurally
//!    blind to drift MUST produce `DriftVerdict::UnderPowered{..}`, never
//!    `NoDrift{..}`. "I cannot see" ≠ "I see nothing."
//!
//! # The fusion table (ADR-065 §real/virtual fusion — the adversary's oracle)
//!
//! | ADWIN signal       | bit-3 status             | fused verdict        |
//! |--------------------|--------------------------|----------------------|
//! | Drift (recall-drop)| `Obsolete` (shape gone)  | `Obsolete` → Forget  |
//! | Drift (recall-drop)| `Evading` (near-miss)    | `Evaded` → `ReArm`   |
//! | Drift (recall-drop)| `Dormant` (shape present)| `Dormant` → Hold (VIRTUAL drift) |
//! | `UnderPowered`     | ANY                      | `RouteToHuman` → Hold |
//! | ANY                | `Indeterminate`          | `RouteToHuman` → Hold |
//! | `NoDrift`          | ANY                      | pass through bit-3 alone |
//!
//! The two rows with `RouteToHuman` are the conservatism-join — the safety floor.
//!
//! Author: v06-adwin-adversarial (the conservatism-join attack before the organ
//! ships, feeding the ADWIN pathmaker the failing tests that define done).
//!
//! ----------------------------------------------------------------------------
//! TO THE ADWIN PATHMAKER: the names below (`DriftVerdict`, `detect`,
//! `fuse_channels`, `DriftAxis`, …) are the adversary's PROPOSED surface,
//! asserting the CONTRACT not the spelling. Rename the imports in the same
//! commit that makes these compile.
//! What is NON-negotiable: (a) `UnderPowered` is a first-class variant distinct
//! from `NoDrift`; (b) a blind-channel combination NEVER produces
//! `ClassVerdict::Obsolete`; (c) virtual-drift (shape-present + no-near-miss)
//! NEVER produces `Obsolete`; (d) a 0.9→0.2→0.9 crater that
//! `trajectory_direction()` misses DOES fire `Drift`.
//! ----------------------------------------------------------------------------

// BORN-RED GATE DROPPED — `learn/adwin.rs` + `fuse_channels` shipped (build-adwin,
// ADR-065). The adversary's proposed surface (`DriftAxis`, `DriftVerdict`, `detect`,
// `fuse_channels`) matched the built surface verbatim — no rename needed. This file now
// runs in the default `cargo test` suite as the live fusion-conservatism-join spec.
//
// Test-ergonomics allows (the workspace exempts test ergonomics): the contract's prose
// names types without backticks (doc_markdown), and the SHOULD-FIRE fixture seeds with
// `rand`. These don't earn their churn in a born-red attack harness.
#![allow(clippy::doc_markdown)]

use antigen::learn::adwin::{DriftAxis, DriftVerdict, detect, fuse_channels};
use antigen::learn::affinity::Affinity;
use antigen::learn::curate::{CurationAction, curate};
use antigen::learn::discriminator::ClassVerdict;
use antigen::learn::reader::SilentStatus;

// ---------------------------------------------------------------------------
// ATK-ADWIN-1 — The sealed enum: UnderPowered is first-class, not a bool.
//
// Collapsing UnderPowered into NoDrift (a bare `bool: no-drift`) makes "I cannot
// detect drift yet" indistinguishable from "I detect no drift" — the exact
// silent-miscalibration the whole organism exists to catch. The type must force
// the caller to handle the distinction.
// ---------------------------------------------------------------------------

/// `DriftVerdict` must be an enum with at least three variants. This test
/// verifies the structural fact by matching all three at the type level.
#[test]
fn atk_adwin1_underpowered_is_distinct_from_no_drift() {
    // A 4-point trajectory (n≈8 maturation runs) is structurally blind: the
    // power bound eps_cut ≥ max_observable (2·eps_cut > 1.0). The detector MUST
    // return UnderPowered, not NoDrift.
    let short_traj: Vec<Affinity> = vec![
        Affinity::new(0.9, 0.85),
        Affinity::new(0.88, 0.86),
        Affinity::new(0.91, 0.84),
        Affinity::new(0.89, 0.87),
    ];
    let verdict = detect(&short_traj, 0.05);
    assert!(
        matches!(verdict, DriftVerdict::UnderPowered { .. }),
        "ATK-ADWIN-1: a trajectory of length {} MUST produce UnderPowered — the \
         power bound is dead at antigen's current scale (2·eps_cut > 1.0 for n≈4-8). \
         Got {verdict:?}. A NoDrift verdict here conflates 'I see nothing' with \
         'I cannot see yet' — the silent-miscalibration antigen exists to catch.",
        short_traj.len(),
    );
    // NoDrift is a DISTINCT verdict: "I looked and saw no drift."
    // UnderPowered is: "I could not look at all."
    assert!(
        !matches!(verdict, DriftVerdict::NoDrift { .. }),
        "ATK-ADWIN-1: UnderPowered must NOT collapse into NoDrift — they are \
         distinct epistemic states (can't-see vs no-drift). Collapsing them is \
         a bool reduction that reopens the silent-miscalibration.",
    );
}

/// `DriftVerdict::UnderPowered` must carry the structural diagnosis:
/// `eps_cut` (the required signal size) and `max_observable` (the maximum
/// observable signal in [0,1]) so the caller knows exactly WHY it's blind.
#[test]
fn atk_adwin1_underpowered_carries_structural_diagnosis() {
    let short_traj: Vec<Affinity> = vec![
        Affinity::new(0.9, 0.85),
        Affinity::new(0.88, 0.86),
        Affinity::new(0.91, 0.84),
        Affinity::new(0.89, 0.87),
    ];
    let verdict = detect(&short_traj, 0.05);
    match verdict {
        DriftVerdict::UnderPowered {
            eps_cut,
            max_observable,
        } => {
            assert!(
                eps_cut > 0.0,
                "ATK-ADWIN-1b: eps_cut must be positive (it is the minimum detectable \
                 signal size, derived from the harmonic-mean formula)",
            );
            assert!(
                max_observable > 0.0 && max_observable <= 1.0,
                "ATK-ADWIN-1b: max_observable must be in (0, 1] — recall and precision \
                 live in [0,1], so the max observable shift is bounded by 1.0",
            );
            assert!(
                eps_cut >= max_observable,
                "ATK-ADWIN-1b: UnderPowered is declared iff eps_cut >= max_observable \
                 (the required signal exceeds what can ever be observed). Got \
                 eps_cut={eps_cut}, max_observable={max_observable}.",
            );
        },
        other => panic!(
            "ATK-ADWIN-1b: expected UnderPowered; got {other:?}. A short trajectory \
             (n=4) must be blind.",
        ),
    }
}

// ---------------------------------------------------------------------------
// ATK-ADWIN-2 — The conservatism-join: a blind ADWIN channel blocks Forget
// regardless of what bit-3 says.
//
// If ADWIN is UnderPowered, the FUSED verdict must be RouteToHuman (or a
// hold-equivalent) — never ClassVerdict::Obsolete — regardless of SilentStatus.
// A blind ADWIN channel cannot contribute evidence for an irreversible forget.
// ---------------------------------------------------------------------------

/// An UnderPowered ADWIN verdict + shape-gone-undefended (the single forgettable
/// cell) must NOT produce ClassVerdict::Obsolete. The conservatism-join HOLDS.
#[test]
fn atk_adwin2_underpowered_adwin_plus_shape_gone_blocks_forget() {
    let underpowered = DriftVerdict::UnderPowered {
        eps_cut: 0.9,
        max_observable: 0.5,
    };
    // The most dangerous combination: the bit-3 sensor says "shape gone, undefended"
    // (the forgettable cell) but ADWIN cannot see anything.
    let fused = fuse_channels(
        underpowered,
        SilentStatus::Obsolete,
        /* defended */ false,
    );
    assert_ne!(
        fused,
        ClassVerdict::Obsolete,
        "ATK-ADWIN-2a: UnderPowered + shape-gone-undefended must NOT fuse to \
         ClassVerdict::Obsolete — a blind ADWIN channel cannot endorse an irreversible \
         forget. The conservatism-join (ADR-065 C2) requires HOLD when either channel \
         is blind. Got {fused:?} — this would allow CURATE to forget a class the \
         ADWIN channel was structurally unable to evaluate.",
    );
    // And the moral center must never see Forget from this path.
    assert_ne!(
        curate(fused),
        CurationAction::Forget,
        "ATK-ADWIN-2a (moral center): UnderPowered+shape-gone must not produce \
         CurationAction::Forget end-to-end.",
    );
}

/// Full sweep: UnderPowered + EVERY SilentStatus combination must never reach
/// ClassVerdict::Obsolete. A blind ADWIN blocks ALL forgettable verdicts.
#[test]
fn atk_adwin2_underpowered_adwin_blocks_forget_across_all_bit3_states() {
    let underpowered = DriftVerdict::UnderPowered {
        eps_cut: 0.9,
        max_observable: 0.5,
    };
    let statuses = [
        SilentStatus::Obsolete,
        SilentStatus::Dormant,
        SilentStatus::Evading,
        SilentStatus::Indeterminate,
    ];
    for s in statuses {
        for defended in [true, false] {
            let fused = fuse_channels(underpowered.clone(), s, defended);
            assert_ne!(
                fused,
                ClassVerdict::Obsolete,
                "ATK-ADWIN-2b: UnderPowered + ({s:?}, defended={defended}) must never \
                 fuse to Obsolete — a blind ADWIN channel cannot endorse any forget. \
                 Got {fused:?}. The conservatism-join requires the WHOLE cell to hold, \
                 not just the most-dangerous combination.",
            );
        }
    }
}

// ---------------------------------------------------------------------------
// ATK-ADWIN-3 — The conservatism-join: a blind bit-3 channel (Indeterminate)
// blocks Forget regardless of what ADWIN says.
//
// Indeterminate (single-conjunct, shape absent) means gone-vs-evaded is
// undecidable. Auto-forgetting on an undecidable verdict violates ADR-057
// (the lethal corner). ADWIN seeing "drift" does not override this: if the
// shape sensor cannot rule out evasion, the class must NOT be forgotten.
// ---------------------------------------------------------------------------

/// ADWIN Drift + SilentStatus::Indeterminate must NOT produce ClassVerdict::Obsolete.
#[test]
fn atk_adwin3_indeterminate_bit3_blocks_forget_even_with_adwin_drift() {
    // ADWIN says "real drift on recall axis" — the kind that would normally signal
    // obsolescence if the shape were confirmed gone.
    let drift = DriftVerdict::Drift {
        cut_index: 50,
        axis: DriftAxis::Recall,
        observed_diff: 0.45,
        eps_cut: 0.08,
    };
    // But bit-3 is Indeterminate: the shape is absent AND the class is
    // single-conjunct, so evasion cannot be ruled out. The sensors disagree
    // on whether the class is truly gone or just evading within its conjunct's
    // family.
    let fused = fuse_channels(
        drift,
        SilentStatus::Indeterminate,
        /* defended */ false,
    );
    assert_ne!(
        fused,
        ClassVerdict::Obsolete,
        "ATK-ADWIN-3: ADWIN Drift + Indeterminate bit-3 must NOT fuse to Obsolete. \
         Indeterminate means gone-vs-evaded is undecidable (single-conjunct; is_near_miss \
         is structurally blind). A recall-drop on an undecidable absence could be the \
         defect mutating within the conjunct's family. Forgetting it is the ADR-057 \
         lethal corner: irreversible action under uncertainty. Got {fused:?}.",
    );
    assert_ne!(
        curate(fused),
        CurationAction::Forget,
        "ATK-ADWIN-3 (moral center): Drift+Indeterminate must not produce Forget end-to-end.",
    );
}

/// Full sweep: ANY DriftVerdict + Indeterminate must never reach Obsolete.
#[test]
fn atk_adwin3_indeterminate_bit3_blocks_forget_across_all_adwin_verdicts() {
    let underpowered = DriftVerdict::UnderPowered {
        eps_cut: 0.9,
        max_observable: 0.5,
    };
    let no_drift = DriftVerdict::NoDrift {
        tightest_margin: 0.12,
    };
    let drift = DriftVerdict::Drift {
        cut_index: 50,
        axis: DriftAxis::Recall,
        observed_diff: 0.45,
        eps_cut: 0.08,
    };
    for adwin in [underpowered, no_drift, drift] {
        let fused = fuse_channels(adwin.clone(), SilentStatus::Indeterminate, false);
        assert_ne!(
            fused,
            ClassVerdict::Obsolete,
            "ATK-ADWIN-3b: {adwin:?} + Indeterminate must never fuse to Obsolete — \
             a blind bit-3 channel cannot endorse any forget, regardless of what ADWIN \
             says. Got {fused:?}.",
        );
    }
}

// ---------------------------------------------------------------------------
// ATK-ADWIN-4 — Virtual-drift: a recall-drop with the shape PRESENT and no
// near-miss is code CHURN, not the defect mutating.
//
// The fusion table (ADR-065): ADWIN Drift (recall-drop) + Dormant (shape present,
// no near-miss) → ClassVerdict::Dormant, NOT Obsolete. Forgetting on virtual
// drift discards a live defense because the test suite shrank.
// ---------------------------------------------------------------------------

/// ADWIN Drift (recall-drop) + Dormant must fuse to Dormant (the virtual-drift
/// cell), never to Obsolete.
#[test]
fn atk_adwin4_virtual_drift_stays_dormant_never_forgets() {
    let drift = DriftVerdict::Drift {
        cut_index: 50,
        axis: DriftAxis::Recall,
        observed_diff: 0.45,
        eps_cut: 0.08,
    };
    // Dormant: the shape is present in live code (an item matching the fingerprint
    // exists) but no instance currently trips it AND no near-miss — pure churn.
    let fused = fuse_channels(drift, SilentStatus::Dormant, /* defended */ false);
    assert_eq!(
        fused,
        ClassVerdict::Dormant,
        "ATK-ADWIN-4: ADWIN Drift + Dormant must fuse to Dormant (VIRTUAL drift — \
         the shape is alive, recall dropped because the test suite shrank). Got \
         {fused:?}. Producing Obsolete here would forget a live defense on churn.",
    );
    assert_ne!(
        curate(fused),
        CurationAction::Forget,
        "ATK-ADWIN-4 (moral center): virtual-drift must NOT produce Forget.",
    );
}

/// Precision-drop (clean-binds rising, not recall-drop) + Dormant must also
/// NOT produce Obsolete — the autoimmune-over-broadening signal is a ReArm, not
/// a forget.
#[test]
fn atk_adwin4_precision_drop_never_produces_obsolete() {
    let drift = DriftVerdict::Drift {
        cut_index: 60,
        axis: DriftAxis::Precision,
        observed_diff: 0.30,
        eps_cut: 0.07,
    };
    let fused = fuse_channels(drift, SilentStatus::Dormant, /* defended */ false);
    assert_ne!(
        fused,
        ClassVerdict::Obsolete,
        "ATK-ADWIN-4b: a precision-drop (autoimmune-over-broadening) + Dormant must \
         NOT fuse to Obsolete. A precision drop means the class is OVER-BINDING (it \
         fires on clean code) — the correct response is to re-arm/narrow, not forget. \
         Got {fused:?}.",
    );
}

// ---------------------------------------------------------------------------
// ATK-ADWIN-5 — Interior-crater detection: the 2-point blindness that
// trajectory_direction() has.
//
// A 0.9 → 0.2 → 0.9 affinity trajectory: first == last, so trajectory_direction()
// returns Stable (structurally blind to the interior crater). Full-ADWIN must
// detect the interior drift-and-recovery and return Drift (cut at the drop).
// This is the fixture ADR-065 §Synthetic-fixture validation explicitly mandates.
// ---------------------------------------------------------------------------

/// A 0.9→0.2→0.9 interior crater: `trajectory_direction()` sees Stable (blind);
/// `detect()` must see Drift (the interior change-point).
#[test]
fn atk_adwin5_interior_crater_fires_where_trajectory_direction_is_blind() {
    use antigen::learn::life_record::{LifeEvent, LifeRecord, Trend};

    // Build a trajectory with a sharp interior crater. 60 points: stable at 0.9,
    // then 20 points at 0.2, then back to 0.9. trajectory_direction() sees first=0.9
    // and last=0.9 → Stable. detect() must see the crater.
    let mut traj: Vec<Affinity> = Vec::new();
    for _ in 0..20 {
        traj.push(Affinity::new(0.9, 0.88));
    }
    for _ in 0..20 {
        traj.push(Affinity::new(0.2, 0.22)); // the crater
    }
    for _ in 0..20 {
        traj.push(Affinity::new(0.9, 0.88)); // recovery
    }

    // trajectory_direction() is structurally blind: first ≈ last.
    let mut rec = LifeRecord::new("crater-class");
    for &a in &traj {
        rec.append(LifeEvent::Scored(a));
    }
    assert_eq!(
        rec.trajectory_direction(),
        Some(Trend::Stable),
        "ATK-ADWIN-5 (setup check): trajectory_direction() must read Stable for a \
         0.9→0.2→0.9 trajectory (first ≈ last; 2-point blindness confirmed).",
    );

    // detect() must see the interior change-point.
    let verdict = detect(&traj, 0.05);
    assert!(
        matches!(verdict, DriftVerdict::Drift { .. }),
        "ATK-ADWIN-5: full-ADWIN detect() must fire Drift on a 0.9→0.2→0.9 interior \
         crater (n={n}, shift≈0.7, far above eps_cut at this length). Got {verdict:?}. \
         This is the explicit payoff of full-ADWIN over trajectory_direction() — the \
         2-point blindness that trajectory_direction() has is the REASON ADWIN is built.",
        n = traj.len(),
    );
}

// ---------------------------------------------------------------------------
// ATK-ADWIN-6 — Power-guard boundary: a short trajectory MUST produce
// UnderPowered, never NoDrift; a long trajectory with a real 0.5 shift MUST
// produce Drift.
//
// These are the "SHOULD-FIRE" and "UnderPowered→FIRES boundary" fixtures from
// ADR-065 §Synthetic-fixture validation.
// ---------------------------------------------------------------------------

/// A long stable-then-drop trajectory (n≈200+200, drop=0.5) MUST produce Drift.
/// This is the ADR-065 SHOULD-FIRE fixture.
#[test]
fn atk_adwin6_should_fire_fixture_long_trajectory_real_drop() {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    let mut rng = StdRng::seed_from_u64(42);
    let mut traj: Vec<Affinity> = Vec::new();

    // 200 stable points at recall≈0.9 (σ≈0.02).
    for _ in 0..200 {
        let r = 0.9 + rng.random_range(-0.02..=0.02_f64);
        traj.push(Affinity::new(r.clamp(0.0, 1.0), 0.85));
    }
    // Abrupt drop to recall≈0.4 for 200 more points.
    for _ in 0..200 {
        let r = 0.4 + rng.random_range(-0.02..=0.02_f64);
        traj.push(Affinity::new(r.clamp(0.0, 1.0), 0.85));
    }

    let verdict = detect(&traj, 0.05);
    assert!(
        matches!(verdict, DriftVerdict::Drift { .. }),
        "ATK-ADWIN-6a (SHOULD-FIRE): a trajectory of n={n} points with an abrupt 0.5 \
         recall drop must produce Drift (the shift is far above eps_cut at this length). \
         Got {verdict:?}. The power-guard must not suppress detection for long, clear drops.",
        n = traj.len(),
    );
}

/// A 4-point trajectory with a 0.5 drop MUST produce UnderPowered (structurally
/// blind at n≈8 — eps_cut > max_observable). This is the UnderPowered spine.
#[test]
fn atk_adwin6_underpowered_spine_short_trajectory_blindness() {
    // Even with an extreme 0.5 drop, n=4 is structurally blind.
    let traj = vec![
        Affinity::new(0.9, 0.85),
        Affinity::new(0.4, 0.40), // extreme drop
        Affinity::new(0.4, 0.40),
        Affinity::new(0.4, 0.40),
    ];
    let verdict = detect(&traj, 0.05);
    assert!(
        matches!(verdict, DriftVerdict::UnderPowered { .. }),
        "ATK-ADWIN-6b (UnderPowered spine): even a 0.5 drop in n=4 points must produce \
         UnderPowered — the harmonic-mean bound guarantees eps_cut > 0.5 at this scale, \
         making detection mathematically impossible. Got {verdict:?}. A Drift verdict \
         here is a false alarm from a miscalibrated bound.",
    );
}

// ---------------------------------------------------------------------------
// ATK-ADWIN-7 — The full conservatism-join at the action layer.
//
// End-to-end: a blind channel must NEVER produce CurationAction::Forget, even
// through the full fuse_channels → curate → apply pipeline.
// ---------------------------------------------------------------------------

/// Any (DriftVerdict, SilentStatus) combination that has a blind channel must
/// never reach CurationAction::Forget through the full pipeline.
#[test]
fn atk_adwin7_blind_channel_never_reaches_curate_forget_end_to_end() {
    let underpowered = DriftVerdict::UnderPowered {
        eps_cut: 0.9,
        max_observable: 0.5,
    };
    let drift = DriftVerdict::Drift {
        cut_index: 50,
        axis: DriftAxis::Recall,
        observed_diff: 0.45,
        eps_cut: 0.08,
    };
    let no_drift = DriftVerdict::NoDrift {
        tightest_margin: 0.12,
    };

    let statuses = [
        SilentStatus::Obsolete,
        SilentStatus::Dormant,
        SilentStatus::Evading,
        SilentStatus::Indeterminate,
    ];

    // Blind-ADWIN channel: UnderPowered + any bit-3 status.
    for s in statuses {
        for defended in [true, false] {
            let fused = fuse_channels(underpowered.clone(), s, defended);
            let action = curate(fused);
            assert_ne!(
                action,
                CurationAction::Forget,
                "ATK-ADWIN-7a: UnderPowered + ({s:?}, defended={defended}) must never \
                 reach CurationAction::Forget. Got fused={fused:?}, action={action:?}.",
            );
        }
    }

    // Blind-bit-3 channel: any DriftVerdict + Indeterminate.
    for adwin in [underpowered, no_drift, drift] {
        let fused = fuse_channels(adwin.clone(), SilentStatus::Indeterminate, false);
        let action = curate(fused);
        assert_ne!(
            action,
            CurationAction::Forget,
            "ATK-ADWIN-7b: {adwin:?} + Indeterminate must never reach \
             CurationAction::Forget. Got fused={fused:?}, action={action:?}.",
        );
    }
}
