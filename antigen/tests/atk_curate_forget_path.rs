//! ATK-CURATE-FORGET-PATH — adversarial attack on CURATE's moral-center forget-gate.
//!
//! **STATUS: TEETH-TESTS (the gate held) + ONE LIVE BUG FOUND (double-retire).**
//!
//! The claim to refute: `Forget` (the one irreversible action — it retires a
//! failure-class) is reachable ONLY when the class verdict is `Obsolete`. The cost
//! of breaking this is the worst thing antigen can do: **forget a defense that is
//! still needed** (a well-defended, evading, dormant, or indeterminate class silently
//! retired).
//!
//! # Attack surface (five dimensions)
//!
//! 1. **Verdict-space** — is there ANY `ClassVerdict` value for which
//!    `is_auto_forgettable()` returns `true` besides `Obsolete`? (The "closed-alphabet /
//!    a new variant silently becomes forgettable" trap.)
//! 2. **The sensor seam** — can the discriminator PRODUCE `Obsolete` for a class that
//!    is actually evading, dormant, or defended? Probe degenerate sensor combinations.
//! 3. **`apply(Forget)` idempotency** — can a second `Forget` on an already-retired
//!    record corrupt the autobiography without detection? (LIVE BUG FOUND — see
//!    ATK-CURATE-3 below.)
//! 4. **Tombstone integrity** — is the `Retired` tombstone genuinely readable in
//!    history after Forget? Can Forget drop prior history?
//! 5. **End-to-end** — wire real sensor inputs through classify → curate → apply and
//!    verify that no path through the full pipeline forgets a class that should be kept.
//!
//! # ATK-CURATE-3 — LIVE BUG: `apply(Forget)` is not idempotent
//!
//! `apply(CurationAction::Forget, &mut record)` does not check whether the record is
//! already retired before appending another `Retired` event. Calling it twice appends
//! two `Retired` events to the autobiography. `is_retired()` returns true both times
//! (it uses `any()`, which is correct), but the raw event stream now contains two
//! deaths for one class — a fact that corrupts any cold-reader doing event-counting or
//! audit-history traversal (`events().iter().filter(Retired).count() == 2`).
//!
//! The fix: `apply(Forget, record)` should be idempotent — if the record is already
//! retired, a second `Forget` appends nothing and returns `None` (same as the
//! no-event actions).
//!
//! The test ATK-CURATE-3-double-retire is born-RED until this guard exists.

use antigen::learn::curate::{apply, curate, CurationAction};
use antigen::learn::discriminator::{classify, ClassVerdict};
use antigen::learn::life_record::{LifeEvent, LifeRecord};
use antigen::learn::reader::SilentStatus;

// ---------------------------------------------------------------------------
// ATK-CURATE-1 — Verdict-space exhaustiveness: `Forget` is reachable from
// `Obsolete` ONLY across the full enum.
//
// This re-expresses the inline curate::tests::forget_is_reachable_from_obsolete_only
// test at the EXTERNAL ATK level, with the additional adversarial framing.
// ---------------------------------------------------------------------------

#[test]
fn atk_curate1_forget_unreachable_from_evaded() {
    assert_ne!(
        curate(ClassVerdict::Evaded),
        CurationAction::Forget,
        "ATK-CURATE-1a: Evaded must NOT produce Forget — an evading class is still \
         alive (near-miss appeared); forgetting it discards a working immunity. \
         The moral-center gate must block this path.",
    );
}

#[test]
fn atk_curate1_forget_unreachable_from_well_defended() {
    assert_ne!(
        curate(ClassVerdict::WellDefended),
        CurationAction::Forget,
        "ATK-CURATE-1b: WellDefended must NOT produce Forget — a live witness holds \
         this class; the witness is plausibly why the shape is gone. Forgetting it \
         discards a working immunity and reintroduces antigen's own \
         RoutingTableStale nightmare.",
    );
}

#[test]
fn atk_curate1_forget_unreachable_from_dormant() {
    assert_ne!(
        curate(ClassVerdict::Dormant),
        CurationAction::Forget,
        "ATK-CURATE-1c: Dormant must NOT produce Forget — the shape is still present \
         in live code (no near-miss yet). Forgetting it drops a defense that may fire \
         when the shape recurs in a triggering form.",
    );
}

#[test]
fn atk_curate1_forget_unreachable_from_route_to_human() {
    assert_ne!(
        curate(ClassVerdict::RouteToHuman),
        CurationAction::Forget,
        "ATK-CURATE-1d: RouteToHuman must NOT produce Forget — the sensors could not \
         decide gone-vs-evaded. Auto-forgetting an undecidable verdict is the ADR-057 \
         lethal corner: irreversible action under uncertainty.",
    );
}

#[test]
fn atk_curate1_forget_reachable_from_obsolete_and_only_obsolete() {
    // The full sweep: only Obsolete should Forget.
    let all = [
        ClassVerdict::Evaded,
        ClassVerdict::WellDefended,
        ClassVerdict::Obsolete,
        ClassVerdict::Dormant,
        ClassVerdict::RouteToHuman,
    ];
    for v in all {
        let action = curate(v);
        let forgets = action == CurationAction::Forget;
        let should_forget = v == ClassVerdict::Obsolete;
        assert_eq!(
            forgets, should_forget,
            "ATK-CURATE-1e: Forget reachable from Obsolete ONLY — {v:?} produced \
             Forget={forgets} (expected {should_forget}). A wrong-forget discards a \
             still-needed defense (the RoutingTableStale reintroduced by the very \
             organ that was built to fight it).",
        );
    }
}

// ---------------------------------------------------------------------------
// ATK-CURATE-2 — Sensor seam: can the discriminator produce `Obsolete` for a
// class that is actually evading, dormant, or defended? Probe all degenerate
// sensor input combinations that could silently route to Forget.
// ---------------------------------------------------------------------------

/// Shape-gone + live witness → WellDefended, NOT Obsolete.
/// The witness-override is the load-bearing discriminator cell: if this cell
/// flipped, a defended class would be forgotten.
#[test]
fn atk_curate2_shape_gone_with_live_witness_never_reaches_forget() {
    let verdict = classify(SilentStatus::Obsolete, /* defended */ true);
    assert_eq!(
        verdict,
        ClassVerdict::WellDefended,
        "ATK-CURATE-2a: shape-gone + live witness must classify as WellDefended, \
         NOT Obsolete. The witness is the plausible reason the shape is gone (the \
         guard held). Producing Obsolete here opens a path to Forget a working \
         immunity.",
    );
    let action = curate(verdict);
    assert_ne!(
        action,
        CurationAction::Forget,
        "ATK-CURATE-2a (action): WellDefended must not produce Forget.",
    );
}

/// Shape-present (Dormant) — regardless of witness state — must never reach Forget.
/// Dormant means the fingerprint matches a live corpus item; the class is alive.
#[test]
fn atk_curate2_shape_present_dormant_never_reaches_forget() {
    for defended in [true, false] {
        let verdict = classify(SilentStatus::Dormant, defended);
        assert_eq!(
            verdict,
            ClassVerdict::Dormant,
            "ATK-CURATE-2b: Dormant + defended={defended} must classify as Dormant, \
             not Obsolete. The shape is alive in the corpus — forgetting it drops a \
             live defense.",
        );
        assert_ne!(
            curate(verdict),
            CurationAction::Forget,
            "ATK-CURATE-2b (action): Dormant must not produce Forget.",
        );
    }
}

/// Near-miss (Evading) — regardless of witness state — must never reach Forget.
/// Evading means the defect mutated just past the fingerprint; the red-queen signal.
#[test]
fn atk_curate2_evading_never_reaches_forget() {
    for defended in [true, false] {
        let verdict = classify(SilentStatus::Evading, defended);
        assert_eq!(
            verdict,
            ClassVerdict::Evaded,
            "ATK-CURATE-2c: Evading + defended={defended} must classify as Evaded \
             regardless of witness state. The near-miss is the act-now red-queen \
             signal — the witness axis does not override an active evasion.",
        );
        assert_ne!(
            curate(verdict),
            CurationAction::Forget,
            "ATK-CURATE-2c (action): Evaded must not produce Forget — the defect \
             mutated past the fingerprint; forgetting it removes the only defense \
             just as the shape re-arms.",
        );
    }
}

/// Indeterminate (single-conjunct, shape absent) — regardless of witness state —
/// must never reach Forget. The sensors cannot decide gone-vs-evaded; the
/// conservative ADR-057 verdict is always RouteToHuman.
#[test]
fn atk_curate2_indeterminate_never_reaches_forget() {
    for defended in [true, false] {
        let verdict = classify(SilentStatus::Indeterminate, defended);
        assert_eq!(
            verdict,
            ClassVerdict::RouteToHuman,
            "ATK-CURATE-2d: Indeterminate + defended={defended} must classify as \
             RouteToHuman — the sensors cannot distinguish gone from evaded for a \
             single-conjunct class. Auto-forgetting here is the ADR-057 lethal \
             corner: irreversible action under uncertainty.",
        );
        assert_ne!(
            curate(verdict),
            CurationAction::Forget,
            "ATK-CURATE-2d (action): RouteToHuman must not produce Forget.",
        );
    }
}

/// The ONLY path to Forget through the full sensor pipeline is Obsolete + undefended.
/// All 8 (SilentStatus × defended) combinations, exhaustive sweep.
#[test]
fn atk_curate2_full_sensor_sweep_only_obsolete_undefended_forgets() {
    let statuses = [
        SilentStatus::Obsolete,
        SilentStatus::Dormant,
        SilentStatus::Evading,
        SilentStatus::Indeterminate,
    ];
    for s in statuses {
        for defended in [true, false] {
            let verdict = classify(s, defended);
            let action = curate(verdict);
            let forgets = action == CurationAction::Forget;
            let should_forget = s == SilentStatus::Obsolete && !defended;
            assert_eq!(
                forgets, should_forget,
                "ATK-CURATE-2e: through the full sensor pipeline, Forget is reached \
                 iff (status=Obsolete AND defended=false); ({s:?}, defended={defended}) \
                 produced forgets={forgets} (expected {should_forget}). Any other path \
                 forgets a class that is still needed.",
            );
        }
    }
}

// ---------------------------------------------------------------------------
// ATK-CURATE-3 — LIVE BUG: `apply(Forget)` is NOT idempotent.
//
// A class that is already retired receives a SECOND Retired tombstone if
// apply(Forget, record) is called again. is_retired() stays true (any() fold),
// but the event-count is 2, not 1. This corrupts the autobiography for any
// cold-reader that counts events or audits the history stream.
//
// BORN-RED: this test FAILS on the current implementation because there is no
// idempotency guard in apply(). It becomes green when apply(Forget, record)
// is made idempotent: if record.is_retired(), append nothing, return None.
// ---------------------------------------------------------------------------

#[test]
fn atk_curate3_double_retire_corrupts_autobiography() {
    let mut rec = LifeRecord::new("obsolete-class");
    rec.append(LifeEvent::Born);

    // First Forget — legitimate.
    let first = apply(curate(ClassVerdict::Obsolete), &mut rec);
    assert_eq!(first, Some(LifeEvent::Retired), "first Forget must record Retired");
    assert!(rec.is_retired(), "class is retired after first Forget");

    let events_after_first = rec.events().len();

    // Second Forget — the class is ALREADY retired. A second Forget is a caller bug,
    // but apply() should not silently double-append a tombstone. The autobiography
    // must stay clean: the second call must be idempotent (append nothing, return
    // None — same contract as Keep/Hold/RouteToHuman, none of which append events
    // when the lifecycle state already reflects no change).
    //
    // BORN-RED: this assertion FAILS on the current implementation.
    let second = apply(curate(ClassVerdict::Obsolete), &mut rec);
    assert_eq!(
        second, None,
        "ATK-CURATE-3: apply(Forget) on an already-retired record must be idempotent \
         — it must NOT append a second Retired event. A double-tombstone corrupts the \
         autobiography for cold-readers that count events or audit the history stream. \
         (The fix: if record.is_retired(), return None without appending.)",
    );
    assert_eq!(
        rec.events().len(),
        events_after_first,
        "ATK-CURATE-3: the event-count must not grow on a double-retire. Got {} events, \
         expected {} — a second Retired was silently appended.",
        rec.events().len(),
        events_after_first,
    );
}

// ---------------------------------------------------------------------------
// ATK-CURATE-4 — Tombstone integrity: the Retired event persists in history
// after Forget, and prior history is preserved (Forget is a PUSHED event, not
// an erasure — ADR-059 tombstone-not-silence).
// ---------------------------------------------------------------------------

#[test]
fn atk_curate4_forget_is_a_pushed_tombstone_not_an_erasure() {
    let mut rec = LifeRecord::new("obsolete-class");
    rec.append(LifeEvent::Born);
    rec.append(LifeEvent::Matured);
    let events_before = rec.events().len();

    apply(curate(ClassVerdict::Obsolete), &mut rec);

    // Tombstone appended (not silence).
    assert!(
        rec.events().iter().any(|e| matches!(e, LifeEvent::Retired)),
        "ATK-CURATE-4a: Forget must append a Retired tombstone that persists in \
         history — tombstone-not-silence (ADR-059). A cold-reader must be able to \
         see that this dead end was walked.",
    );

    // Prior history NOT erased.
    assert!(
        rec.events().iter().any(|e| matches!(e, LifeEvent::Born)),
        "ATK-CURATE-4b: Forget must NOT erase prior history — Born must still be \
         readable. An erasure-Forget would silence the dead end, hiding that a \
         defense existed and was retired.",
    );
    assert!(
        rec.events().iter().any(|e| matches!(e, LifeEvent::Matured)),
        "ATK-CURATE-4c: Forget must NOT erase prior history — Matured must still be \
         readable. The autobiography grows monotonically (append-only); Forget is one \
         pushed event, not a reset.",
    );

    // Exactly one new event was appended (the Retired tombstone).
    assert_eq!(
        rec.events().len(),
        events_before + 1,
        "ATK-CURATE-4d: Forget appends exactly one event (the Retired tombstone). \
         Got {} events, expected {}.",
        rec.events().len(),
        events_before + 1,
    );
}

#[test]
fn atk_curate4_forget_on_empty_record_is_a_pushable_tombstone_with_no_prior_history() {
    // A class that was never Born can receive a Forget — there is no Born guard in
    // apply(). The resulting record has [Retired] with no [Born] before it.
    // This is a consistency gap (death without birth) worth documenting even if not
    // currently guarded. The test verifies the tombstone lands, and that the event
    // stream is exactly [Retired] — no phantom extra events.
    let mut rec = LifeRecord::new("never-born");
    apply(curate(ClassVerdict::Obsolete), &mut rec);

    assert!(
        rec.is_retired(),
        "ATK-CURATE-4e: even a never-Born record becomes retired after Forget — \
         the tombstone lands.",
    );
    assert_eq!(
        rec.events().len(),
        1,
        "ATK-CURATE-4e: Forget on an empty record appends exactly one event (Retired). \
         Got {} events (expected 1).",
        rec.events().len(),
    );
    assert!(
        matches!(rec.events()[0], LifeEvent::Retired),
        "ATK-CURATE-4e: the sole event is Retired.",
    );
}

// ---------------------------------------------------------------------------
// ATK-CURATE-5 — No fall-through: every ClassVerdict routes to a deterministic
// action; no verdict escapes without hitting either the forget-gate or a safe
// default. Tests the ladder exhaustiveness.
// ---------------------------------------------------------------------------

#[test]
fn atk_curate5_every_verdict_produces_a_deterministic_action() {
    // If a verdict hit a fall-through, curate() would be non-exhaustive and the
    // match would not compile (Rust enforces exhaustive matches). This test
    // additionally verifies the LADDER ORDERING: every verdict hits its expected
    // rung, not some other rung, and certainly not Forget unless it is Obsolete.
    let expected: &[(ClassVerdict, CurationAction)] = &[
        (ClassVerdict::WellDefended, CurationAction::Keep),
        (ClassVerdict::Dormant, CurationAction::Hold),
        (ClassVerdict::Evaded, CurationAction::ReArm),
        (ClassVerdict::RouteToHuman, CurationAction::RouteToHuman),
        (ClassVerdict::Obsolete, CurationAction::Forget),
    ];
    for (verdict, expected_action) in expected {
        let actual = curate(*verdict);
        assert_eq!(
            actual, *expected_action,
            "ATK-CURATE-5: verdict {verdict:?} must route to {expected_action:?} on \
             the reversible-first ladder, but produced {actual:?}. A verdict hitting \
             the wrong rung (especially Forget from a non-Obsolete verdict) means the \
             moral center is routing incorrectly.",
        );
    }
}

/// Keep, Hold, and RouteToHuman must NEVER retire a class — they leave no event
/// and the autobiography is unchanged.
#[test]
fn atk_curate5_reversible_actions_never_retire() {
    for verdict in [
        ClassVerdict::WellDefended,
        ClassVerdict::Dormant,
        ClassVerdict::RouteToHuman,
    ] {
        let mut rec = LifeRecord::new("c");
        rec.append(LifeEvent::Born);
        let before = rec.events().len();

        let appended = apply(curate(verdict), &mut rec);
        assert_eq!(
            appended, None,
            "ATK-CURATE-5a: {verdict:?} must record no event (it is a \
             keep-as-is / escalate action on the reversible rung)",
        );
        assert!(!rec.is_retired(), "ATK-CURATE-5a: {verdict:?} must never retire");
        assert_eq!(
            rec.events().len(),
            before,
            "ATK-CURATE-5a: {verdict:?} must not grow the autobiography"
        );
    }
}

/// ReArm records a Drifted event but must NOT retire the class.
#[test]
fn atk_curate5_rearm_records_drift_never_retire() {
    let mut rec = LifeRecord::new("evading-class");
    rec.append(LifeEvent::Born);

    let appended = apply(curate(ClassVerdict::Evaded), &mut rec);
    assert_eq!(
        appended,
        Some(LifeEvent::Drifted),
        "ATK-CURATE-5b: Evaded → ReArm must record Drifted",
    );
    assert!(
        !rec.is_retired(),
        "ATK-CURATE-5b: ReArm must NOT retire the class — it broadens/re-arms, \
         discarding nothing. A re-armed class is more alive, not dead.",
    );
    assert!(
        !rec.events().iter().any(|e| matches!(e, LifeEvent::Retired)),
        "ATK-CURATE-5b: no Retired event in the stream after ReArm",
    );
}

// ---------------------------------------------------------------------------
// ATK-CURATE-6 — is_auto_forgettable contract: CURATE forgets a verdict iff
// the discriminator marks it auto-forgettable, and vice-versa. The two organs
// agree on the one cell that may discard.
// ---------------------------------------------------------------------------

#[test]
fn atk_curate6_forget_gate_agrees_with_auto_forgettable_contract() {
    let all = [
        ClassVerdict::Evaded,
        ClassVerdict::WellDefended,
        ClassVerdict::Obsolete,
        ClassVerdict::Dormant,
        ClassVerdict::RouteToHuman,
    ];
    for v in all {
        let curate_forgets = curate(v) == CurationAction::Forget;
        let discriminator_allows = v.is_auto_forgettable();
        assert_eq!(
            curate_forgets, discriminator_allows,
            "ATK-CURATE-6: CURATE and the discriminator must agree on auto-forgettability. \
             For {v:?}: curate forgets={curate_forgets}, is_auto_forgettable={discriminator_allows}. \
             A disagreement means the two organs have diverged on the one cell that \
             may discard — one would permit forgetting the other would forbid.",
        );
    }
}

// ---------------------------------------------------------------------------
// ATK-CURATE-7 — End-to-end pipeline: wire real sensor inputs through the
// full path (SilentStatus + defended → classify → curate → apply) and verify
// that the ONLY path that retires a class in the autobiography is the
// shape-gone-AND-undefended path.
// ---------------------------------------------------------------------------

#[test]
fn atk_curate7_end_to_end_only_shape_gone_undefended_retires() {
    let statuses = [
        SilentStatus::Obsolete,
        SilentStatus::Dormant,
        SilentStatus::Evading,
        SilentStatus::Indeterminate,
    ];
    for s in statuses {
        for defended in [true, false] {
            let mut rec = LifeRecord::new("c");
            rec.append(LifeEvent::Born);

            let verdict = classify(s, defended);
            let action = curate(verdict);
            apply(action, &mut rec);

            let retired = rec.is_retired();
            let should_retire = s == SilentStatus::Obsolete && !defended;
            assert_eq!(
                retired, should_retire,
                "ATK-CURATE-7: through the full pipeline (sensor→classify→curate→apply), \
                 a class is retired iff (status=Obsolete AND defended=false). \
                 ({s:?}, defended={defended}) retired={retired} (expected {should_retire}). \
                 Any other path retires a class that is still needed.",
            );
        }
    }
}

/// The end-to-end path for the load-bearing cell (shape-gone + defended) must
/// produce Keep and leave the autobiography untouched.
#[test]
fn atk_curate7_shape_gone_but_defended_keeps_and_leaves_history_clean() {
    let mut rec = LifeRecord::new("defended-class");
    rec.append(LifeEvent::Born);
    let before = rec.events().len();

    let verdict = classify(SilentStatus::Obsolete, /* defended */ true);
    let action = curate(verdict);
    assert_eq!(
        action,
        CurationAction::Keep,
        "ATK-CURATE-7a: shape-gone + defended must produce Keep end-to-end",
    );

    let appended = apply(action, &mut rec);
    assert_eq!(
        appended, None,
        "ATK-CURATE-7a: Keep records no event — the autobiography is unchanged",
    );
    assert!(!rec.is_retired(), "ATK-CURATE-7a: a defended class is never retired");
    assert_eq!(
        rec.events().len(),
        before,
        "ATK-CURATE-7a: the biography must not grow for a Keep action",
    );
}
