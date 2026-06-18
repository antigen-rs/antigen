//! CURATE — the **moral center** (the do-now efferent decision-layer).
//!
//! # Why this is the moral center
//!
//! The whole-voyage naturalist finding: **"CURATE is the moral center — the
//! forgetting IS the trust."** Every other organ *senses* and *classifies*; CURATE is
//! the first that *acts* — and one of its actions ([`CurationAction::Forget`])
//! discards a failure-class. A wrong forget here re-introduces antigen's own
//! `RoutingTableStale` nightmare *from inside the organ built to fight noise*:
//! forgetting a defense that is still needed. Getting CURATE wrong IS how antigen
//! becomes the noise it exists to cut through. So this module's load-bearing property
//! is not what it forgets — it is what it is *structurally incapable* of forgetting.
//!
//! # The reversible-first ladder (the systems-thinking ruling)
//!
//! CURATE is NOT one balancing loop — it is a **router over several** balancing loops,
//! each protecting a *different* scarce stock (trust-budget, attention, immunity-
//! coverage, detection-recall), and its exits form a **ladder from reversible to
//! irreversible** (the island `loops/curate-is-a-multi-stock-router`). The do-now
//! scope is that ladder's *minimal* decision-layer — the routing rule + the safety
//! gate — with the full confusion-cube (the multi-stock arbitration sub-gates) left
//! for do-later. The ladder ordering this module honors, reversible → irreversible:
//!
//! 1. [`CurationAction::Keep`] — the null action; nothing spent, nothing changed.
//! 2. [`CurationAction::Hold`] — keep an alive-but-unwalked class (DORMANT-archive);
//!    reversible, discards nothing.
//! 3. [`CurationAction::RouteToHuman`] — escalate an undecidable to a human; the
//!    conservative default (ADR-057), never auto-acting irreversibly.
//! 4. [`CurationAction::ReArm`] — the red-queen response to active evasion; broadens /
//!    re-arms, still discards nothing (it records a drift, not a death).
//! 5. [`CurationAction::Forget`] — the ONLY irreversible, discarding action. Last rung.
//!    Reachable from [`ClassVerdict::Obsolete`] **and nothing else**, structurally.
//!
//! # The safety gate (the load-bearing proof)
//!
//! [`curate`] emits [`CurationAction::Forget`] **only** when
//! [`discriminator::is_auto_forgettable`](crate::learn::discriminator::ClassVerdict::is_auto_forgettable)
//! is `true` — and that contract is `true` for [`ClassVerdict::Obsolete`] alone. The
//! gate is expressed in code (an `if is_auto_forgettable { Forget } else { … }`), so a
//! future edit that tried to Forget a `WellDefended`/`Dormant`/`Evaded`/`RouteToHuman`
//! verdict would have to *delete the gate* to do it — the unsafe path does not exist
//! to be reached by accident. The exhaustive `forget_is_reachable_from_obsolete_only`
//! test pins it the same way the discriminator pins its own auto-forget contract.

use crate::learn::discriminator::ClassVerdict;
use crate::learn::life_record::{LifeEvent, LifeRecord};

/// What CURATE decides to *do* to a failure-class, given its [`ClassVerdict`].
///
/// The variants are the **reversible-first ladder** (see module docs): every action
/// but [`Forget`](Self::Forget) is reversible and discards nothing; `Forget` is the
/// single irreversible exit and is reachable from [`ClassVerdict::Obsolete`] alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CurationAction {
    /// **Keep** — the null action. The class is doing its job ([`ClassVerdict::WellDefended`]):
    /// keep it as-is, spend no resource, change nothing.
    Keep,
    /// **Hold** — keep an *alive-but-not-currently-firing* class
    /// ([`ClassVerdict::Dormant`]): the shape is present in live code but no instance
    /// trips it. Reversible, discards nothing — the DORMANT-archive rung. Forgetting a
    /// Dormant class would drop a defense that may fire when the shape recurs.
    Hold,
    /// **Re-arm** — the red-queen response to active evasion ([`ClassVerdict::Evaded`]):
    /// a near-miss appeared (the defect mutated just past the fingerprint). Broaden /
    /// re-arm the class; records a drift, discards nothing.
    ReArm,
    /// **Route-to-human** — escalate an undecidable verdict ([`ClassVerdict::RouteToHuman`])
    /// to a human ratifier. The conservative default (ADR-057): never auto-act
    /// irreversibly on a class the sensors could not decide. Discards nothing.
    RouteToHuman,
    /// **Forget** — retire the class (append a tombstone). The **only** irreversible,
    /// discarding action and the last rung of the ladder. Emitted **only** for
    /// [`ClassVerdict::Obsolete`] (shape gone AND no live witness holds it), gated
    /// through
    /// [`is_auto_forgettable`](crate::learn::discriminator::ClassVerdict::is_auto_forgettable).
    Forget,
}

/// Decide the [`CurationAction`] for a class from its [`ClassVerdict`] — the do-now
/// routing rule on the reversible-first ladder.
///
/// The mapping (verdict → action), and the stock each exit protects:
/// - [`ClassVerdict::WellDefended`] → [`CurationAction::Keep`] (null action).
/// - [`ClassVerdict::Dormant`] → [`CurationAction::Hold`] (immunity-coverage; reversible).
/// - [`ClassVerdict::Evaded`] → [`CurationAction::ReArm`] (detection-recall; red-queen).
/// - [`ClassVerdict::RouteToHuman`] → [`CurationAction::RouteToHuman`] (conservative,
///   ADR-057 — never auto-act on the undecidable).
/// - [`ClassVerdict::Obsolete`] → [`CurationAction::Forget`] (attention/scan-cost) —
///   **but only through the [`is_auto_forgettable`] gate**.
///
/// # The safety gate
///
/// `Forget` is produced **only** when
/// [`ClassVerdict::is_auto_forgettable`] is `true`. That contract is `true` for
/// `Obsolete` alone, so it is *structurally impossible* for this function to Forget any
/// other verdict — the gate, not a comment, is what enforces it. The `else` arm of the
/// gate can never be reached for `Obsolete` (the contract guarantees it), but it is
/// written as the conservative [`CurationAction::RouteToHuman`]: if a future edit ever
/// loosened `is_auto_forgettable` to return `false` for `Obsolete`, CURATE would
/// escalate rather than silently mis-handle it. Defense in depth around the one exit
/// that discards.
///
/// [`is_auto_forgettable`]: crate::learn::discriminator::ClassVerdict::is_auto_forgettable
#[must_use]
pub const fn curate(verdict: ClassVerdict) -> CurationAction {
    match verdict {
        ClassVerdict::WellDefended => CurationAction::Keep,
        ClassVerdict::Dormant => CurationAction::Hold,
        ClassVerdict::Evaded => CurationAction::ReArm,
        ClassVerdict::RouteToHuman => CurationAction::RouteToHuman,
        // The single discarding exit — gated. `is_auto_forgettable` is true ONLY for
        // Obsolete, so reaching `Forget` requires the verdict to BE auto-forgettable;
        // the unsafe path (Forget on a non-Obsolete verdict) is not expressible here.
        ClassVerdict::Obsolete => {
            if verdict.is_auto_forgettable() {
                CurationAction::Forget
            } else {
                // Unreachable while the contract holds; conservative if it ever doesn't.
                CurationAction::RouteToHuman
            }
        },
    }
}

/// Apply a [`CurationAction`] to a class's [`LifeRecord`], recording the action in the
/// organism's autobiography (the STOCK the whole life-record exists to accumulate).
///
/// Only the two actions that **change the class's posture** leave a mark — the
/// append-only autobiography records lifecycle *transitions*, not no-ops:
/// - [`CurationAction::Forget`] → appends [`LifeEvent::Retired`] (tombstone-not-silence:
///   the dead end stays readable in history; ADR-059). The single discarding action.
/// - [`CurationAction::ReArm`] → appends [`LifeEvent::Drifted`] (the rate-stream drift
///   the red-queen response answers — the class's near-miss is that drift).
/// - [`CurationAction::Keep`] / [`Hold`](CurationAction::Hold) /
///   [`RouteToHuman`](CurationAction::RouteToHuman) → **no event**: these are
///   keep-as-is / escalate decisions that alter no lifecycle state. Appending a
///   "nothing changed" event would pollute the autobiography with noise.
///
/// Returns the [`LifeEvent`] appended (so the caller can see *what* was recorded), or
/// `None` for the non-recording actions.
pub fn apply(action: CurationAction, record: &mut LifeRecord) -> Option<LifeEvent> {
    // Idempotency on the one irreversible action: a class is retired exactly once. A
    // second `Forget` on an already-retired record is a no-op (no duplicate tombstone),
    // so a cold-reader counting `Retired` events in the autobiography can never read two
    // deaths for one class. Same contract as the non-recording actions: no event, no
    // lifecycle change. (Found by the forget-path adversarial: ATK-CURATE-3.)
    if matches!(action, CurationAction::Forget) && record.is_retired() {
        return None;
    }
    let event = match action {
        CurationAction::Forget => Some(LifeEvent::Retired),
        CurationAction::ReArm => Some(LifeEvent::Drifted),
        CurationAction::Keep | CurationAction::Hold | CurationAction::RouteToHuman => None,
    };
    if let Some(ref e) = event {
        record.append(e.clone());
    }
    event
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::learn::reader::SilentStatus;

    /// THE MORAL-CENTER SAFETY PROOF (mirrors the discriminator's auto-forget contract
    /// test): across EVERY [`ClassVerdict`], [`CurationAction::Forget`] is reachable
    /// from [`ClassVerdict::Obsolete`] **and nothing else**. A wrong-forget here is
    /// antigen's own `RoutingTableStale` nightmare — this is the load-bearing test that
    /// it is structurally impossible.
    #[test]
    fn forget_is_reachable_from_obsolete_only() {
        let all = [
            ClassVerdict::Evaded,
            ClassVerdict::WellDefended,
            ClassVerdict::Obsolete,
            ClassVerdict::Dormant,
            ClassVerdict::RouteToHuman,
        ];
        for v in all {
            let forgets = curate(v) == CurationAction::Forget;
            let expected = v == ClassVerdict::Obsolete;
            assert_eq!(
                forgets, expected,
                "Forget must be reachable from Obsolete ONLY — {v:?} produced \
                 Forget={forgets} (expected {expected}). A forget from any other \
                 verdict discards a still-needed defense (RoutingTableStale).",
            );
        }
    }

    /// The forget gate is tied to the discriminator's contract, not re-derived: every
    /// verdict CURATE forgets is exactly a verdict the discriminator marks
    /// auto-forgettable, and vice-versa. The two organs agree on the one cell that may
    /// discard — there is no second opinion about what is safe to forget.
    #[test]
    fn forget_agrees_with_the_auto_forget_contract() {
        let all = [
            ClassVerdict::Evaded,
            ClassVerdict::WellDefended,
            ClassVerdict::Obsolete,
            ClassVerdict::Dormant,
            ClassVerdict::RouteToHuman,
        ];
        for v in all {
            assert_eq!(
                curate(v) == CurationAction::Forget,
                v.is_auto_forgettable(),
                "CURATE forgets a verdict iff the discriminator marks it \
                 auto-forgettable; {v:?} broke that agreement.",
            );
        }
    }

    /// The full verdict → action routing rule (the reversible-first ladder).
    #[test]
    fn every_verdict_routes_to_its_ladder_rung() {
        assert_eq!(curate(ClassVerdict::WellDefended), CurationAction::Keep);
        assert_eq!(curate(ClassVerdict::Dormant), CurationAction::Hold);
        assert_eq!(curate(ClassVerdict::Evaded), CurationAction::ReArm);
        assert_eq!(
            curate(ClassVerdict::RouteToHuman),
            CurationAction::RouteToHuman
        );
        assert_eq!(curate(ClassVerdict::Obsolete), CurationAction::Forget);
    }

    /// END-TO-END through the real upstream sensors: the discriminator's own
    /// shape-gone-but-defended cell ([`SilentStatus::Obsolete`] + a live witness) is
    /// WELL-DEFENDED → CURATE **Keep**, NOT Forget. The witness-override survives all
    /// the way to the action — a defended class is never forgotten even though its
    /// shape is gone. (The cell the discriminator's load-bearing test guards, carried
    /// through to the moral-center action.)
    #[test]
    fn shape_gone_but_defended_is_kept_not_forgotten() {
        use crate::learn::discriminator::classify;
        let verdict = classify(SilentStatus::Obsolete, /* defended */ true);
        assert_eq!(verdict, ClassVerdict::WellDefended);
        assert_eq!(
            curate(verdict),
            CurationAction::Keep,
            "a class whose shape is gone but which still carries a live witness must be \
             KEPT — the witness is why the shape is gone; forgetting it discards a \
             working immunity.",
        );
        // And only shape-gone-AND-undefended reaches Forget through the sensors.
        let undefended = classify(SilentStatus::Obsolete, /* defended */ false);
        assert_eq!(undefended, ClassVerdict::Obsolete);
        assert_eq!(curate(undefended), CurationAction::Forget);
    }

    /// Forget records a [`LifeEvent::Retired`] tombstone into the STOCK — the dead end
    /// stays readable in history (tombstone-not-silence, ADR-059), and the derived
    /// retired-state flips.
    #[test]
    fn forget_appends_a_retired_tombstone() {
        let mut rec = LifeRecord::new("obsolete-class");
        rec.append(LifeEvent::Born);
        assert!(!rec.is_retired());

        let appended = apply(curate(ClassVerdict::Obsolete), &mut rec);
        assert_eq!(appended, Some(LifeEvent::Retired));
        assert!(
            rec.is_retired(),
            "after Forget the class is retired — the tombstone persists in history",
        );
        // tombstone-not-silence: the event is still readable in the stream.
        assert!(rec.events().iter().any(|e| matches!(e, LifeEvent::Retired)));
    }

    /// `ReArm` records a [`LifeEvent::Drifted`] (the red-queen drift) — it does NOT
    /// retire the class. Re-arming answers evasion without discarding the lineage.
    #[test]
    fn rearm_records_drift_without_retiring() {
        let mut rec = LifeRecord::new("evading-class");
        rec.append(LifeEvent::Born);

        let appended = apply(curate(ClassVerdict::Evaded), &mut rec);
        assert_eq!(appended, Some(LifeEvent::Drifted));
        assert!(
            !rec.is_retired(),
            "ReArm must NOT retire the class — it broadens/re-arms, discarding nothing",
        );
    }

    /// The reversible non-discarding actions (`Keep` / `Hold` / `RouteToHuman`) record
    /// NO event — they alter no lifecycle state, so the append-only autobiography stays
    /// free of "nothing changed" noise, and crucially they never retire the class.
    #[test]
    fn keep_hold_and_route_record_nothing_and_never_retire() {
        for verdict in [
            ClassVerdict::WellDefended, // → Keep
            ClassVerdict::Dormant,      // → Hold
            ClassVerdict::RouteToHuman, // → RouteToHuman
        ] {
            let mut rec = LifeRecord::new("kept-class");
            rec.append(LifeEvent::Born);
            let before = rec.events().len();

            let appended = apply(curate(verdict), &mut rec);
            assert_eq!(
                appended, None,
                "{verdict:?} is a keep-as-is / escalate action — it records no event",
            );
            assert_eq!(
                rec.events().len(),
                before,
                "{verdict:?} must not grow the autobiography (no lifecycle transition)",
            );
            assert!(
                !rec.is_retired(),
                "{verdict:?} must never retire the class — it is a reversible action",
            );
        }
    }

    /// THE MORAL-CENTER PROOF AT THE ACTION LAYER: across every verdict, the only one
    /// that ever appends a [`LifeEvent::Retired`] (the discarding tombstone) is
    /// [`ClassVerdict::Obsolete`]. No conservative/live/undecidable verdict can retire
    /// a class through CURATE.
    #[test]
    fn only_obsolete_can_ever_retire_a_class() {
        let all = [
            ClassVerdict::Evaded,
            ClassVerdict::WellDefended,
            ClassVerdict::Obsolete,
            ClassVerdict::Dormant,
            ClassVerdict::RouteToHuman,
        ];
        for v in all {
            let mut rec = LifeRecord::new("c");
            apply(curate(v), &mut rec);
            let retired = rec.is_retired();
            let expected = v == ClassVerdict::Obsolete;
            assert_eq!(
                retired, expected,
                "a class is retired through CURATE iff its verdict is Obsolete; \
                 {v:?} retired={retired} (expected {expected})",
            );
        }
    }
}
