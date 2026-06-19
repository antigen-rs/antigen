//! ATK-DEEPCOMB-FUSE — degenerate-input attack on the conservatism-JOIN's input
//! boundary (Path B Tier-1 relitigation; the ±∞-Forget class closed at its final layer).
//!
//! **STATUS: born-red on `ec357db` — a non-finite `Drift` forgot a class.**
//!
//! `fuse_channels` is the producer of `ClassVerdict::Obsolete` (the one auto-forgettable
//! cell). It is a `pub const fn` taking a `DriftVerdict`. Its `Drift` match-arms key on
//! `axis` ONLY — they never check `observed_diff.is_finite()`. So a `Drift{axis: Recall,
//! observed_diff: ±∞ or NaN}` + `(SilentStatus::Obsolete, defended=false)` fell through
//! the `_ => bit3` arm to `ClassVerdict::Obsolete` → CURATE `Forget` — a non-finite
//! GARBAGE verdict drove an irreversible forget.
//!
//! This is the SAME ±∞-Forget class the ADR-065 harden pass closed — re-opened one layer
//! up. Harden sanitized `detect`'s OUTPUT (so `detect` never emits a non-finite `Drift`);
//! this closes `fuse_channels`'s INPUT (a hand-constructed verdict). The conservatism-
//! JOIN's whole contract is "a blind/garbage channel cannot endorse an irreversible
//! forget" — a non-finite `observed_diff` IS garbage, so it must route to a human, the
//! same response the JOIN already gives `UnderPowered`.
//!
//! Honest scope: the production caller (`fused_classify`) always feeds `detect`'s
//! sanitized output, and a non-finite `Drift` does not survive a serde round-trip
//! (serde_json renders non-finite floats as `null`, which fails to deserialize into an
//! `f64`), so this is reachable only by a caller hand-constructing the garbage verdict
//! and calling the `pub fn`. Guarded anyway: the moral center must defend its own input
//! boundary (same posture as ADR-065 harden's defense-in-depth at `detect`).
//!
//! FIX: `fuse_channels` half-1b — a `Drift` with non-finite `observed_diff` ⇒
//! `RouteToHuman` (garbage ⇒ blind ⇒ HOLD), right after the `UnderPowered` half-1 guard.

// Test-ergonomics exemption (the workspace exempts test ergonomics, as the sibling
// `atk_adwin_fusion_conservatism_join.rs` does): the contract prose names `serde_json`
// without backticks (doc_markdown).
#![allow(clippy::doc_markdown)]

use antigen::learn::adwin::{DriftAxis, DriftVerdict, fuse_channels};
use antigen::learn::curate::{CurationAction, curate};
use antigen::learn::discriminator::ClassVerdict;
use antigen::learn::reader::SilentStatus;

/// A `Drift` on `axis` with the given `observed_diff` (the rest of the fields fixed —
/// the contract seals the variant at `{cut_index, axis, observed_diff, eps_cut}`).
const fn drift(axis: DriftAxis, observed_diff: f64) -> DriftVerdict {
    DriftVerdict::Drift {
        cut_index: 0,
        axis,
        observed_diff,
        eps_cut: 0.0,
    }
}

// ---------------------------------------------------------------------------
// ATK-DEEPCOMB-FUSE-1 — a non-finite `Drift.observed_diff` must NOT reach Obsolete
// (→ Forget). The garbage channel routes to a human. All three non-finite values.
// ---------------------------------------------------------------------------

#[test]
fn atk_deepcomb_fuse1_nonfinite_drift_never_forgets() {
    for bad in [f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
        for axis in [DriftAxis::Recall, DriftAxis::Precision] {
            // The most-dangerous bit-3 cell: shape-gone-undefended (the only forgettable
            // streamless verdict). A non-finite Drift must NOT let it forget.
            let fused = fuse_channels(drift(axis, bad), SilentStatus::Obsolete, false);
            assert_ne!(
                fused,
                ClassVerdict::Obsolete,
                "ATK-DEEPCOMB-FUSE-1: a Drift with non-finite observed_diff={bad} \
                 (axis={axis:?}) + (Obsolete, undefended) must NOT fuse to Obsolete — a \
                 garbage channel cannot endorse a forget. Got {fused:?}.",
            );
            assert_eq!(
                fused,
                ClassVerdict::RouteToHuman,
                "ATK-DEEPCOMB-FUSE-1: a non-finite Drift is a blind/garbage channel — it \
                 must RouteToHuman (the conservatism-JOIN's response to a blind channel), \
                 not silently fall through to the bit-3 verdict. Got {fused:?}.",
            );
            // And the moral center must never see Forget from this path.
            assert_ne!(
                curate(fused),
                CurationAction::Forget,
                "ATK-DEEPCOMB-FUSE-1 (moral center): a non-finite Drift must never reach \
                 CurationAction::Forget end-to-end.",
            );
        }
    }
}

// ---------------------------------------------------------------------------
// ATK-DEEPCOMB-FUSE-2 — NEGATIVE CONTROL: a FINITE Drift still flows to its normal
// verdict. The guard must not over-correct (route every Drift to human).
// ---------------------------------------------------------------------------

#[test]
fn atk_deepcomb_fuse2_finite_drift_still_flows_normally() {
    // A finite recall-Drift + shape-gone-undefended is REAL obsolescence corroborated by
    // the loud axis ⇒ Obsolete (the streamless verdict stands; this is the cell the
    // fusion table's recall-Drift + NoDrift/Obsolete row produces). The guard must leave
    // this untouched — a finite diff is every real `detect` output.
    let fused = fuse_channels(
        drift(DriftAxis::Recall, 0.45),
        SilentStatus::Obsolete,
        false,
    );
    assert_eq!(
        fused,
        ClassVerdict::Obsolete,
        "ATK-DEEPCOMB-FUSE-2 (negative control): a FINITE recall-Drift + shape-gone-\
         undefended must still fuse to Obsolete (real obsolescence) — the non-finite \
         guard must not over-correct and route a legitimate finite Drift to human. \
         Got {fused:?}.",
    );

    // A finite recall-Drift + Evading stays Evaded (the streamless verdict). Another
    // normal-flow cell the guard must not disturb.
    let evading = fuse_channels(drift(DriftAxis::Recall, 0.30), SilentStatus::Evading, false);
    assert_eq!(
        evading,
        ClassVerdict::Evaded,
        "ATK-DEEPCOMB-FUSE-2: a finite recall-Drift + Evading must still be Evaded — the \
         guard touches only non-finite diffs. Got {evading:?}.",
    );

    // A finite Drift + WellDefended (shape-gone + live witness) stays WellDefended.
    let defended = fuse_channels(drift(DriftAxis::Recall, 0.45), SilentStatus::Obsolete, true);
    assert_eq!(
        defended,
        ClassVerdict::WellDefended,
        "ATK-DEEPCOMB-FUSE-2: a finite Drift + shape-gone-DEFENDED stays WellDefended \
         (the witness override) — the guard does not disturb it. Got {defended:?}.",
    );
}

// ---------------------------------------------------------------------------
// ATK-DEEPCOMB-FUSE-3 — the guard does not mask the OTHER conservatism cells: a
// non-finite Drift + a bit-3 state that already routes-to-human stays RouteToHuman
// (no behavior change there), and + Dormant (the undecidable cell) stays RouteToHuman.
// Confirms the new guard composes with the existing JOIN, not overrides it.
// ---------------------------------------------------------------------------

#[test]
fn atk_deepcomb_fuse3_nonfinite_composes_with_existing_join() {
    // non-finite Drift + Indeterminate: already RouteToHuman by half-2; stays so.
    assert_eq!(
        fuse_channels(
            drift(DriftAxis::Recall, f64::INFINITY),
            SilentStatus::Indeterminate,
            false
        ),
        ClassVerdict::RouteToHuman,
        "ATK-DEEPCOMB-FUSE-3: non-finite Drift + Indeterminate is RouteToHuman (both the \
         non-finite guard and the bit-3-blind guard agree).",
    );
    // non-finite Drift + Dormant: the finite version is the undecidable recall cell
    // (RouteToHuman); the non-finite version is also RouteToHuman (garbage guard fires
    // first). Either way, never a forget.
    assert_eq!(
        fuse_channels(
            drift(DriftAxis::Recall, f64::NAN),
            SilentStatus::Dormant,
            false
        ),
        ClassVerdict::RouteToHuman,
        "ATK-DEEPCOMB-FUSE-3: non-finite Drift + Dormant routes to human (never forget).",
    );
}
