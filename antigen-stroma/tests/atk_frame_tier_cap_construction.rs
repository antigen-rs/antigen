//! ATK-FRAME-TIER-CAP-CONSTRUCTION (gap-closing, build-discovered) — the grade is DERIVED from the
//! source tier AT CONSTRUCTION, never caller-supplied. ADR-070 §3.2 (the C3 "law-in-types" form).
//!
//! ## Why this ATK exists (a stronger defense the build revealed)
//! The builder made `TieredAnswer::answered(value, tier)` derive `grade = tier.detection_ceiling()` —
//! so `{ tier: Syntactic, grade: Presents }` is UNCONSTRUCTIBLE, not merely refused at runtime. My
//! original TIER-CAP ATK guards the `PresentsVerdict`-no-ctor door; it does NOT guard the
//! `TieredAnswer` construction-level cap. A future refactor that added a public `TieredAnswer { tier,
//! grade }` literal or a caller-supplied `grade` would silently reopen the false-quiet hole, and
//! nothing in the suite would catch it. This closes that gap — the same "C3 dogfooded" pattern as the
//! fidelity-signature and atomic-publish gems: the law is a type/construction error, not a convention.
//!
//! ## Two guards, two tiers of strength (GREEN-from-birth — the read-CONTRACT is already filled):
//!   - PROPERTY (runtime, GREEN now — `detection_ceiling`/`corroborate` are filled): the derived grade
//!     equals the source-tier ceiling — Syntactic→Dread, Resolved→Presents. NOT born-red: the
//!     construction-level cap was discovered already-built, so these are immediate forever-guards.
//!   - COMPILE-STATE (trybuild, `tests/ui/tieredanswer_grade_not_caller_supplied.rs`): a struct literal
//!     setting `grade` directly does NOT compile (the field is private). Guards the derive-only path.

use antigen_stroma::read::answer::TieredAnswer;
use antigen_stroma::read::tier::{DetectionGrade, ResolutionTier};

// PROPERTY (GREEN now — detection_ceiling is filled): a Syntactic source yields a Dread-grade
// answer BY CONSTRUCTION — the grade is derived, not supplied.
#[test]
fn atk_frame_tier_cap_syntactic_answer_is_dread_by_construction() {
    let answer = TieredAnswer::answered("x", ResolutionTier::Syntactic);
    assert_eq!(
        answer.grade(),
        DetectionGrade::Dread,
        "ATK-FRAME-TIER-CAP-CONSTRUCTION: a Syntactic-sourced answer carried a grade above Dread. \
         The grade must be DERIVED from the source tier (detection_ceiling) at construction — a \
         syntactic source cannot carry a presents-grade answer (the false-quiet cardinal sin)."
    );

    // And reading it at the Presents floor must REFUSE (the route-to-human channel, never silent).
    assert!(
        answer.read_at_least(DetectionGrade::Presents).is_none(),
        "ATK-FRAME-TIER-CAP-CONSTRUCTION: a Dread-grade answer served a value at the Presents floor — \
         a silent tier-downgrade (the false-quiet defense failed)."
    );
}

// NEGATIVE CONTROL (teeth): a Resolved source DOES reach Presents-grade by construction — proving the
// cap is tier-keyed (derived per-tier), not a blanket "everything is Dread".
#[test]
fn nc_frame_tier_cap_resolved_answer_reaches_presents() {
    let answer = TieredAnswer::answered("x", ResolutionTier::Resolved);
    assert_eq!(
        answer.grade(),
        DetectionGrade::Presents,
        "NC: a Resolved-sourced answer did not reach Presents-grade — the cap is a blanket floor, not \
         a tier-keyed derivation. It would refuse every content-grade read (useless)."
    );
    assert!(
        answer.read_at_least(DetectionGrade::Presents).is_some(),
        "NC: a Resolved/Presents answer refused a Presents-floor read — over-refusal."
    );
}

// NEGATIVE CONTROL (teeth, the corroboration door): a syntactic input on either side of corroboration
// must NOT mint a presents-grade answer — `corroborated` returns None (the law-forbidden lift can't
// construct an answer).
#[test]
fn nc_frame_tier_cap_corroborate_with_syntactic_refuses() {
    let forbidden =
        TieredAnswer::corroborated("x", ResolutionTier::Syntactic, ResolutionTier::Resolved);
    assert!(
        forbidden.is_none(),
        "NC: corroborate(Syntactic, Resolved) minted an answer — a syntactic input corroborated UP. \
         Only two fresh-independent resolved-or-higher sources may lift the grade (§3.2)."
    );
}
