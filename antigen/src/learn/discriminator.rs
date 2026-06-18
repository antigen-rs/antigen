//! The DISCRIMINATOR — the shared classifier spine (P3a) that fuses the two
//! streamless sensors into one curation verdict per failure-class.
//!
//! # The build-once share is at the CLASSIFIER, not the sensors
//!
//! The v0.6 obsolete/dormant/evaded/well-defended classification was once imagined
//! as "one change-detector feeding one discriminator." The converge-adversarial
//! corrected that: the *sensors* read DIFFERENT inputs with DIFFERENT mechanisms and
//! do NOT collapse — only the **classifier** is shared. This module IS that shared
//! classifier. It consumes:
//!
//! - **INPUT 1 — the SOURCE-AST axis** ([`SilentStatus`], from
//!   [`silent_status`](crate::learn::reader::silent_status)): is the failure-shape
//!   gone / present / mutated-just-past / undecidable in live code? STREAMLESS.
//! - **INPUT 2 — the WITNESS axis** (a `defended` bool, from P2's
//!   [`is_class_defended`](crate::audit::AuditReport::is_class_defended)): does the
//!   class still carry a live (`tier > None`) witness at its sites?
//!
//! A third axis — INPUT 3, the LOUD-minority rate-stream (ADWIN over the affinity
//! trajectory) — is intentionally NOT consumed here: it is a separate sensor
//! (different math, gated on the streaming change-detector) that, when built, refines
//! the EVADED/DORMANT split for the loud classes. This classifier is the silent-core
//! spine the four efferent loops hang off; the loud refinement layers on later.
//!
//! # Why the WITNESS axis OVERRIDES "shape gone"
//!
//! The single most load-bearing cell: a class whose **shape is gone** (the
//! SOURCE-AST axis would call it OBSOLETE) but which still carries a **live witness**
//! is **WELL-DEFENDED, not obsolete** — the witness is the plausible *reason* the
//! shape is gone (the guard held). Forgetting it would discard a working immunity
//! (antigen's own `RoutingTableStale` nightmare, re-introduced by the very organ
//! meant to fight noise). On the silence axis alone these are identical; the witness
//! axis is what keeps WELL-DEFENDED distinct from OBSOLETE (the same split P2' draws).

use crate::learn::reader::SilentStatus;

/// The per-class curation verdict — what the efferent loops (CURATE / forgetting /
/// red-queen) act on. Derived purely from the two streamless sensor outputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ClassVerdict {
    /// The failure-shape mutated to just past the fingerprint (a near-miss appeared).
    /// **Re-arm / broaden** — the red-queen signal. (From [`SilentStatus::Evading`];
    /// a near-miss is active evasion regardless of witness state.)
    Evaded,
    /// The shape is gone from live code AND a live witness still guards the sites —
    /// the witness is plausibly *why* the shape is gone. **Keep; do NOT forget** (the
    /// witness-OVERRIDE of "shape gone"). Distinct from [`Obsolete`](Self::Obsolete)
    /// only on the witness axis.
    WellDefended,
    /// The shape is gone AND no live witness holds it — nothing keeps the failure
    /// from recurring, and it isn't currently in the code. **Forgettable** (the only
    /// verdict CURATE may auto-forget on).
    Obsolete,
    /// The shape is present in live code but no instance currently trips it (no
    /// near-miss). **Keep** — it may fire when the shape recurs in a triggering form.
    Dormant,
    /// The sensors cannot decide gone-vs-evaded (a single-conjunct class whose shape
    /// is absent — [`SilentStatus::Indeterminate`]). **Route-to-human, never
    /// auto-forget** (ADR-057 conservative-default). The conservative verdict the
    /// efferent loops must treat as "do nothing irreversible."
    RouteToHuman,
}

/// Fuse the two streamless sensor outputs into a per-class [`ClassVerdict`] (P3a).
///
/// `silent` is INPUT 1 ([`silent_status`](crate::learn::reader::silent_status));
/// `defended` is INPUT 2 (`tier > None` per-class, from
/// [`is_class_defended`](crate::audit::AuditReport::is_class_defended)).
///
/// The fusion, in precedence order:
/// 1. [`SilentStatus::Evading`] → [`ClassVerdict::Evaded`] (a near-miss is active
///    evasion — the witness axis does not override an act-now red-queen signal).
/// 2. [`SilentStatus::Indeterminate`] → [`ClassVerdict::RouteToHuman`] (undecidable,
///    conservative).
/// 3. [`SilentStatus::Dormant`] → [`ClassVerdict::Dormant`] (shape alive).
/// 4. [`SilentStatus::Obsolete`] → **the witness axis decides**:
///    [`ClassVerdict::WellDefended`] if `defended` (a live witness holds it), else
///    [`ClassVerdict::Obsolete`] (truly gone — the only auto-forgettable cell).
#[must_use]
pub const fn classify(silent: SilentStatus, defended: bool) -> ClassVerdict {
    match silent {
        SilentStatus::Evading => ClassVerdict::Evaded,
        SilentStatus::Indeterminate => ClassVerdict::RouteToHuman,
        SilentStatus::Dormant => ClassVerdict::Dormant,
        SilentStatus::Obsolete => {
            if defended {
                ClassVerdict::WellDefended
            } else {
                ClassVerdict::Obsolete
            }
        },
    }
}

impl ClassVerdict {
    /// May an efferent loop **auto-forget** a class on this verdict? `true` ONLY for
    /// [`Obsolete`](Self::Obsolete) — the shape is gone AND nothing (witness) holds
    /// it. Every other verdict (including [`RouteToHuman`](Self::RouteToHuman) and
    /// [`WellDefended`](Self::WellDefended)) is NOT auto-forgettable — the safety
    /// contract CURATE reads so a conservative/undecidable verdict never silently
    /// drops a live or unknown class.
    #[must_use]
    pub const fn is_auto_forgettable(self) -> bool {
        matches!(self, Self::Obsolete)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// THE LOAD-BEARING CELL: shape-gone (Obsolete) + a LIVE WITNESS is WELL-DEFENDED,
    /// not Obsolete — the witness axis overrides "shape gone". Forgetting it would
    /// discard a working immunity.
    #[test]
    fn shape_gone_but_defended_is_well_defended_not_obsolete() {
        assert_eq!(
            classify(SilentStatus::Obsolete, true),
            ClassVerdict::WellDefended,
            "a class whose shape is gone but which still carries a live witness is \
             WELL-DEFENDED — the witness is why the shape is gone; do NOT forget it."
        );
        assert!(
            !classify(SilentStatus::Obsolete, true).is_auto_forgettable(),
            "WellDefended must NOT be auto-forgettable."
        );
    }

    /// Shape-gone + NO witness is the ONLY auto-forgettable cell.
    #[test]
    fn shape_gone_and_undefended_is_obsolete_and_forgettable() {
        let v = classify(SilentStatus::Obsolete, false);
        assert_eq!(v, ClassVerdict::Obsolete);
        assert!(
            v.is_auto_forgettable(),
            "Obsolete (shape gone AND no witness) is the only verdict an efferent \
             loop may auto-forget on."
        );
    }

    /// EVADING → Evaded regardless of witness state — a near-miss is an act-now
    /// red-queen signal the witness axis does not override.
    #[test]
    fn evading_is_evaded_regardless_of_witness() {
        assert_eq!(classify(SilentStatus::Evading, true), ClassVerdict::Evaded);
        assert_eq!(classify(SilentStatus::Evading, false), ClassVerdict::Evaded);
        assert!(!ClassVerdict::Evaded.is_auto_forgettable());
    }

    /// INDETERMINATE → `RouteToHuman`, never auto-forget (the ADR-057 conservative
    /// verdict carried through the classifier).
    #[test]
    fn indeterminate_routes_to_human_and_is_not_forgettable() {
        for defended in [true, false] {
            let v = classify(SilentStatus::Indeterminate, defended);
            assert_eq!(
                v,
                ClassVerdict::RouteToHuman,
                "an undecidable sensor verdict must route-to-human, never auto-forget \
                 (ADR-057) — independent of the witness axis."
            );
            assert!(!v.is_auto_forgettable());
        }
    }

    /// DORMANT (shape present, no near-miss) → Dormant, keep, not forgettable.
    #[test]
    fn dormant_stays_dormant_and_is_not_forgettable() {
        for defended in [true, false] {
            let v = classify(SilentStatus::Dormant, defended);
            assert_eq!(v, ClassVerdict::Dormant);
            assert!(!v.is_auto_forgettable());
        }
    }

    /// The auto-forget safety contract end-to-end: across EVERY (silent, defended)
    /// combination, the ONLY auto-forgettable verdict is Obsolete (shape-gone +
    /// undefended). No conservative/undecidable/live cell is ever forgettable.
    #[test]
    fn only_shape_gone_undefended_is_ever_auto_forgettable() {
        let silents = [
            SilentStatus::Obsolete,
            SilentStatus::Dormant,
            SilentStatus::Evading,
            SilentStatus::Indeterminate,
        ];
        for s in silents {
            for defended in [true, false] {
                let forgettable = classify(s, defended).is_auto_forgettable();
                let expected = s == SilentStatus::Obsolete && !defended;
                assert_eq!(
                    forgettable, expected,
                    "auto-forgettable iff shape-gone AND undefended; \
                     ({s:?}, defended={defended}) violated it",
                );
            }
        }
    }
}
