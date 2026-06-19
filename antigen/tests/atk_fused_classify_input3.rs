//! ATK-FUSED-CLASSIFY — the INPUT-3 wiring: the canonical three-axis fused-classify.
//!
//! `discriminator::fused_classify(trajectory, silent, defended, delta)` is the
//! production-shaped entry the curation pipeline consumes — it consults the ADWIN LOUD
//! rate-stream (INPUT 3) alongside the two streamless sensors, composing
//! `score_trajectory → detect → fuse_channels` into one `ClassVerdict`.
//!
//! These tests drive the REAL composed path on REAL `LifeRecord` trajectories +
//! `SilentStatus` inputs (not a pre-built `DriftVerdict` — that is what the fusion
//! contract `atk_adwin_fusion_conservatism_join.rs` covers). This file's job is to
//! prove the *wiring* — that a trajectory actually flows through `detect` into
//! `fuse_channels` and lands the right verdict end-to-end.
//!
//! # Honest scope (the lead's guard)
//!
//! The curation pipeline (`…→fused_classify→curate→apply`) has NO production CLI
//! caller yet — the binary wires only `propose`. `fused_classify` is the
//! wired-and-ready LIBRARY seam (the canonical fused-classify the pipeline will call),
//! NOT a fabricated CLI caller. These tests defend the library seam.
//!
//! # The contested cell is NOT asserted here
//!
//! `recall-Drift + SilentStatus::Dormant` is under a live design-Q (camp 2767bd73): the
//! merged fusion contract maps it `⇒ Dormant` (virtual-drift KEEP); the INPUT-3 wiring
//! note + the discriminator docstring map it `⇒ Evaded` (loud evasion). Until ruled,
//! this file asserts only the SAFETY FLOOR both readings agree on (NOT `Obsolete`) for
//! that cell, plus the un-contested cells in full.

use antigen::learn::adwin::DEFAULT_DELTA;
use antigen::learn::affinity::Affinity;
use antigen::learn::discriminator::{ClassVerdict, fused_classify};
use antigen::learn::life_record::{LifeEvent, LifeRecord};
use antigen::learn::reader::SilentStatus;

/// Build a `LifeRecord` whose `score_trajectory()` is the given recall sequence
/// (precision held flat at a clean 0.85), the loud-axis input `fused_classify` reads.
fn record_with_recall(class: &str, recalls: &[f64]) -> LifeRecord {
    let mut rec = LifeRecord::new(class);
    for &r in recalls {
        rec.append(LifeEvent::Scored(Affinity::new(r, 0.85)));
    }
    rec
}

/// A long abrupt recall drop (0.9→0.4) — the loud axis FIRES a recall `Drift`.
fn long_recall_drop() -> Vec<f64> {
    let mut v = vec![0.9; 200];
    v.extend(vec![0.4; 200]);
    v
}

// ============================================================================
// THE SPINE — a short trajectory is loud-axis-blind ⇒ abstain (RouteToHuman)
// ============================================================================

#[test]
fn atk_fused_short_trajectory_abstains_via_underpowered() {
    // At antigen's CURRENT scale (n≈8) the loud axis is structurally blind
    // (UnderPowered). The conservatism-JOIN routes a blind loud axis to RouteToHuman,
    // regardless of the streamless read — the honest "ADWIN sees nothing yet" abstain.
    let rec = record_with_recall("short-class", &[0.9, 0.9, 0.9, 0.9, 0.4, 0.4, 0.4, 0.4]);
    let traj = rec.score_trajectory();

    // Even paired with the single forgettable streamless cell (shape-gone, undefended),
    // the blind loud axis HOLDS — never Obsolete (the moral center is un-bypassable).
    let verdict = fused_classify(
        &traj,
        SilentStatus::Obsolete,
        /* defended */ false,
        DEFAULT_DELTA,
    );
    assert_eq!(
        verdict,
        ClassVerdict::RouteToHuman,
        "n=8 loud-axis-blind + shape-gone-undefended ⇒ RouteToHuman (abstain), NOT Obsolete \
         — the conservatism-JOIN never forgets on a blind channel. Got {verdict:?}."
    );
}

#[test]
fn atk_fused_empty_trajectory_abstains() {
    // No trajectory at all ⇒ the loud axis is blind (UnderPowered, n<2) ⇒ RouteToHuman.
    let rec = record_with_recall("no-traj-class", &[]);
    let traj = rec.score_trajectory();
    assert_eq!(
        fused_classify(&traj, SilentStatus::Obsolete, false, DEFAULT_DELTA),
        ClassVerdict::RouteToHuman,
        "an empty trajectory is loud-axis-blind ⇒ abstain (never forget)"
    );
}

// ============================================================================
// THE UN-CONTESTED CELLS — a loud recall-Drift threaded end-to-end
// ============================================================================

#[test]
fn atk_fused_loud_drop_plus_shape_gone_is_obsolete() {
    // A long recall-Drift (loud axis FIRES) + shape-gone-undefended: the loud drop
    // CORROBORATES the static absence ⇒ Obsolete (REAL obsolescence, the one
    // auto-forgettable cell). This proves the trajectory actually fired Drift through
    // the wiring (a blind axis would have abstained to RouteToHuman instead).
    let rec = record_with_recall("obsolete-class", &long_recall_drop());
    let traj = rec.score_trajectory();
    assert_eq!(
        fused_classify(
            &traj,
            SilentStatus::Obsolete,
            /* defended */ false,
            DEFAULT_DELTA
        ),
        ClassVerdict::Obsolete,
        "loud recall-Drift + shape-gone-undefended ⇒ Obsolete (the loud drop corroborates \
         static absence). If this read RouteToHuman, the loud axis didn't fire (wiring bug)."
    );
}

#[test]
fn atk_fused_loud_drop_plus_shape_gone_defended_is_well_defended() {
    // Same loud Drift, but a live witness holds the (gone) shape ⇒ WellDefended (the
    // witness-OVERRIDE), never Obsolete. The loud axis fired; the witness axis decides.
    let rec = record_with_recall("defended-class", &long_recall_drop());
    let traj = rec.score_trajectory();
    assert_eq!(
        fused_classify(
            &traj,
            SilentStatus::Obsolete,
            /* defended */ true,
            DEFAULT_DELTA
        ),
        ClassVerdict::WellDefended,
        "loud recall-Drift + shape-gone-DEFENDED ⇒ WellDefended (witness override)"
    );
}

#[test]
fn atk_fused_loud_drop_plus_evading_is_evaded() {
    // A loud recall-Drift + a near-miss appeared (Evading) ⇒ Evaded (red-queen). The
    // streamless Evading already says Evaded; the loud axis agrees.
    let rec = record_with_recall("evading-class", &long_recall_drop());
    let traj = rec.score_trajectory();
    assert_eq!(
        fused_classify(&traj, SilentStatus::Evading, false, DEFAULT_DELTA),
        ClassVerdict::Evaded,
        "loud recall-Drift + near-miss ⇒ Evaded (red-queen, broaden/re-arm)"
    );
}

// ============================================================================
// THE CONTESTED CELL — only the SAFETY FLOOR both readings agree on
// ============================================================================

#[test]
fn atk_fused_loud_drop_plus_dormant_is_never_obsolete() {
    // recall-Drift + SilentStatus::Dormant (shape present, no near-miss) is UNDER a live
    // design-Q (camp 2767bd73): merged contract ⇒ Dormant (churn KEEP); INPUT-3 note ⇒
    // Evaded (loud evasion). Both AGREE on the safety floor: NEVER Obsolete (a live shape
    // is not forgettable). We assert ONLY that floor here, pending the ruling — so this
    // test survives EITHER resolution without a silent re-bless.
    let rec = record_with_recall("dormant-class", &long_recall_drop());
    let traj = rec.score_trajectory();
    let verdict = fused_classify(&traj, SilentStatus::Dormant, false, DEFAULT_DELTA);
    assert_ne!(
        verdict,
        ClassVerdict::Obsolete,
        "loud recall-Drift + Dormant (live shape) must NEVER be Obsolete — both the \
         churn-KEEP and the loud-evasion readings agree on this floor. Got {verdict:?}."
    );
    // Cross-check the moral center: whichever the ruling, it must not auto-forget.
    assert!(
        !verdict.is_auto_forgettable(),
        "the contested cell must not be auto-forgettable under either reading"
    );
}
