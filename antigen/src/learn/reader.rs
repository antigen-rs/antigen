//! The READER — the drift/obsolescence sensor over a class's relationship to the
//! live code (P2). Two facets, split by whether the class emits a temporal signal.
//!
//! # The two facets (the scout's "don't schedule P2 as one block")
//!
//! - **The SILENT-CORE facet (this module's [`silent_status`]) — STREAMLESS.** Antigen's
//!   founding population is *silent* failure-classes (origin.md: the bug nobody
//!   noticed). A silent class emits NO bind-stream — it never fires loudly — so a
//!   temporal change-detector (ADWIN) has a flat-at-zero stream and cannot split the
//!   one cell curation needs most: a no-FIRE class, is it **obsolete** (the shape is
//!   gone — safe to forget), **dormant** (the shape is present but no instance
//!   triggers — keep, it may fire), or **evading** (the shape is present AND a
//!   near-miss appeared — the defect mutated just past the fingerprint, a red-queen
//!   signal)? This facet splits that cell with NO temporal signal, reading two
//!   ALREADY-SHIPPED primitives: [`Fingerprint::matches`] (shape-present?) and
//!   [`is_near_miss`] (near-miss-appeared?). Buildable today — no STOCK dependency.
//! - **The ADWIN facet (LOUD classes) — future.** For classes that DO emit a bind-rate
//!   or affinity trajectory, ADWIN (Bifet & Gavalda 2007) watches the
//!   [`score_trajectory`](crate::learn::life_record::LifeRecord::score_trajectory)
//!   stream for a change-point — the automatic decay-trigger. BUILD-not-wrap (no Rust
//!   ADWIN crate exists). Gated on the STOCK's trajectory stream (now shipped, P1b);
//!   the streaming detector is the next build unit and is intentionally NOT here yet —
//!   this module is the streamless half, honest about its scope.
//!
//! # Why these are SEPARATE verdicts, not one scalar
//!
//! Obsolete and evading are *opposite* curation actions (forget vs. broaden/re-arm),
//! and on the silence axis alone they are identical (both no-FIRE). The discriminator
//! (P3) must not collapse them; this sensor keeps them distinct by reading the
//! second axis — the shape's presence and its near-miss neighbourhood in live code —
//! exactly as the defended-status sensor (P2') keeps WELL-DEFENDED distinct from
//! OBSOLETE by reading witness-liveness. Same shape, different axis.

use antigen_fingerprint::Fingerprint;

use crate::learn::self_tolerance::{is_near_miss, is_near_miss_capable};

/// The silent-core no-FIRE verdict for one class against the live scan corpus — the
/// streamless split the obsolete/well-defended discriminator (P3) reads for classes
/// that carry no temporal signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SilentStatus {
    /// The fingerprint's **shape is gone** — it matches NO item AND no near-miss
    /// exists for it in the live corpus, **and the draft is near-miss-capable** (≥2
    /// conjuncts, so the absence of a near-miss is *trustworthy*). The failure-shape
    /// this class guards no longer exists in any detectable form → an **obsolete**
    /// candidate (safe to forget, subject to the other axes).
    Obsolete,
    /// The shape is **present** (the fingerprint matches a live item) but no
    /// **near-miss** appeared — the guarded shape exists, instances simply don't
    /// currently trip the defect. **Dormant**: keep it; it may fire when the shape
    /// recurs in a triggering form. (NOT obsolete — the shape is alive.)
    Dormant,
    /// A **near-miss appeared** — a live item is one constraint from binding the
    /// draft (the defect mutated to just past the fingerprint), whether or not the
    /// exact shape is still present. **Evading**: the red-queen signal —
    /// broaden/re-arm, do NOT forget. This is the cell ADWIN is blind to for silent
    /// classes.
    Evading,
    /// **Cannot decide gone-vs-evaded** — the shape is absent but the draft is a
    /// *single conjunct* ([`is_near_miss`] is structurally blind there, so evasion
    /// cannot be ruled out). Returning [`Obsolete`](Self::Obsolete) here would let
    /// CURATE forget a class whose defect merely **mutated within its one conjunct's
    /// family** (e.g. `body_calls("unwrap")` → the site now calls `expect()`). The
    /// conservative verdict (ADR-057 conservative-default-under-uncertainty):
    /// **route-to-human, never auto-forget.** A single-conjunct class's absence is
    /// not trustworthy as obsolescence.
    Indeterminate,
}

/// Classify a silent class's no-FIRE state against the live scan `corpus` (P2,
/// bit-3 / silent-core facet) — STREAMLESS, reading only shipped primitives.
///
/// Decided in precedence order (EVADING — the act-now red-queen case — first, so it
/// is never masked):
/// 1. **[`SilentStatus::Evading`]** iff ANY corpus item is a [`is_near_miss`] for the
///    draft (the defect mutated one constraint past the fingerprint) — checked
///    *regardless of whether the exact shape is still present*, so a class whose
///    shape mutated AWAY but left a near-miss is caught.
/// 2. **[`SilentStatus::Dormant`]** iff the shape is present (the draft matches a
///    live item) but no near-miss — the shape is alive, keep it.
/// 3. **[`SilentStatus::Obsolete`]** iff the shape is absent, no near-miss, **and the
///    draft is [`is_near_miss_capable`]** (≥2 conjuncts) — the absence is
///    *trustworthy* (had it evaded, a near-miss would have been detectable).
/// 4. **[`SilentStatus::Indeterminate`]** iff the shape is absent and the draft is a
///    *single conjunct* — [`is_near_miss`] is structurally blind, so gone-vs-evaded
///    is undecidable → route-to-human, never auto-forget (ADR-057).
///
/// The single-conjunct guard closes the READER's evasion-blindness (the adversarial
/// find): without it, `silent_status(body_calls("unwrap"), [fn(){ x.expect() }])`
/// returns `Obsolete` (forget) when the defect actually mutated `unwrap → expect`.
/// Reads [`Fingerprint::matches`] + [`is_near_miss`] + [`is_near_miss_capable`] — no
/// temporal signal, no STOCK.
#[must_use]
pub fn silent_status(draft: &Fingerprint, corpus: &[syn::Item]) -> SilentStatus {
    // EVADING first: a near-miss anywhere is the act-now signal, even if the exact
    // shape mutated away (so it must NOT require shape-present).
    if corpus.iter().any(|item| is_near_miss(draft, item)) {
        return SilentStatus::Evading;
    }
    if corpus.iter().any(|item| draft.matches(item)) {
        return SilentStatus::Dormant;
    }
    // Shape absent and no near-miss. Trust "obsolete" ONLY if a near-miss COULD have
    // been detected (≥2 conjuncts). A single-conjunct draft is near-miss-blind, so
    // its absence cannot be distinguished from an in-conjunct-family mutation →
    // conservative route-to-human (ADR-057), never auto-forget.
    if is_near_miss_capable(draft) {
        SilentStatus::Obsolete
    } else {
        SilentStatus::Indeterminate
    }
}

#[cfg(test)]
mod tests {
    use antigen_fingerprint::Constraint;

    use super::*;

    /// Parse a Rust snippet into the `syn::Item`s a scan corpus would hold.
    fn corpus(src: &str) -> Vec<syn::Item> {
        syn::parse_file(src).expect("test corpus parses").items
    }

    /// A two-conjunct draft of TWO DISCRIMINATING conjuncts: derives `Clone` AND
    /// derives `Debug`. Both are discriminating signals (not bare anchors), so when
    /// `is_near_miss` drops one, the remainder still discriminates (`has_discriminating
    /// _conjunct` holds) — the condition for a valid near-miss after the P0 fix made
    /// `Item`/anchors non-discriminating. (A draft like `[Item(Struct), Derives(Clone)]`
    /// has NO valid near-miss: dropping `Derives` leaves the bare `[Item(Struct)]`
    /// skeleton, which does not discriminate.)
    fn derives_clone_and_debug() -> Fingerprint {
        Fingerprint {
            constraints: vec![
                Constraint::Derives("Clone".into()),
                Constraint::Derives("Debug".into()),
            ],
        }
    }

    /// OBSOLETE: the shape is GONE — the draft matches NO item in the corpus.
    #[test]
    fn shape_gone_is_obsolete() {
        // A corpus with only a plain struct (derives nothing) — the Clone+Debug draft
        // matches nothing.
        let c = corpus("struct Unrelated;");
        assert_eq!(
            silent_status(&derives_clone_and_debug(), &c),
            SilentStatus::Obsolete,
            "a draft whose shape matches NO live item is OBSOLETE (the failure-shape \
             is gone) — the only no-FIRE state that is safe to forget."
        );
    }

    /// DORMANT: the shape is PRESENT (an item deriving both Clone and Debug exists and
    /// the draft binds it) but there is NO near-miss — instances exist, none is
    /// one-constraint-away. The shape is alive → keep it, NOT obsolete.
    #[test]
    fn shape_present_no_near_miss_is_dormant() {
        // An item the draft BINDS (derives Clone AND Debug) and nothing one-away — the
        // bound item is not a near-miss (a near-miss is SPARED, not bound).
        let c = corpus("#[derive(Clone, Debug)] struct Bound;");
        assert_eq!(
            silent_status(&derives_clone_and_debug(), &c),
            SilentStatus::Dormant,
            "shape present (the draft binds a live item) with no near-miss is \
             DORMANT — the shape is alive, keep the class; it is NOT obsolete."
        );
    }

    /// EVADING: the shape is PRESENT and a NEAR-MISS appeared — a live item is one
    /// constraint from binding (derives Clone but NOT Debug). Dropping the `Debug`
    /// conjunct leaves the discriminating `[Derives(Clone)]` remainder, which binds it
    /// → a valid near-miss. The defect mutated just past the fingerprint → re-arm.
    #[test]
    fn shape_present_with_near_miss_is_evading() {
        // One bound item (Clone + Debug) so the shape is present, AND one near-miss
        // (Clone only — fails exactly the Debug conjunct, spared by the whole draft,
        // binds the discriminating remainder when Debug is dropped).
        let c = corpus(
            "#[derive(Clone, Debug)] struct Bound;\n\
             #[derive(Clone)] struct NearMiss;",
        );
        assert_eq!(
            silent_status(&derives_clone_and_debug(), &c),
            SilentStatus::Evading,
            "shape present AND a near-miss appeared (an item one constraint from \
             binding) is EVADING — the red-queen cell ADWIN is blind to for silent \
             classes. It must NOT be read as obsolete (forget) or dormant (ignore)."
        );
    }

    /// ORDER GUARD: EVADING is never masked by DORMANT — a present shape WITH a
    /// near-miss is evading even though the shape is also "present" (the dormant
    /// precondition). The near-miss axis is checked and wins.
    #[test]
    fn evading_is_not_masked_by_dormant() {
        let c = corpus(
            "#[derive(Clone, Debug)] struct Bound;\n\
             #[derive(Clone)] struct NearMiss;",
        );
        let status = silent_status(&derives_clone_and_debug(), &c);
        assert_ne!(
            status,
            SilentStatus::Dormant,
            "a present-shape-WITH-near-miss must not collapse to Dormant — the \
             near-miss (evasion) signal takes precedence over bare presence."
        );
        assert_eq!(status, SilentStatus::Evading);
    }

    /// A single-conjunct body-signal draft: `body_calls("unwrap")`. Single-conjunct
    /// drafts are common (one body signal), not an edge case.
    fn body_calls_unwrap() -> Fingerprint {
        Fingerprint {
            constraints: vec![Constraint::BodyCalls("unwrap".into())],
        }
    }

    /// REGRESSION (adversarial find — the lethal single-conjunct evasion-blindness):
    /// a single-conjunct class whose defect MUTATED within its conjunct's family
    /// (`unwrap` → `expect`) used to read `Obsolete` (= forget) because `is_near_miss`
    /// is structurally blind for single-conjunct drafts (the `len < 2` gate-guard).
    /// It must now read `Indeterminate` (route-to-human, never auto-forget) — a
    /// single-conjunct class's *absence* is not trustworthy as obsolescence.
    #[test]
    fn single_conjunct_shape_absent_is_indeterminate_not_obsolete() {
        // The defect mutated: the site calls `expect()`, not `unwrap()` — so the
        // `body_calls("unwrap")` draft matches nothing AND (single-conjunct) can have
        // no near-miss. Gone-vs-evaded is undecidable here.
        let c = corpus("fn evaded() { x.expect(\"msg\"); }");
        let status = silent_status(&body_calls_unwrap(), &c);
        assert_ne!(
            status,
            SilentStatus::Obsolete,
            "a single-conjunct class whose shape is absent must NOT read Obsolete \
             (forget) — is_near_miss is structurally blind for it, so the defect may \
             have mutated within the conjunct's family (unwrap → expect). Reading \
             Obsolete here would let CURATE forget a still-live evading class."
        );
        assert_eq!(
            status,
            SilentStatus::Indeterminate,
            "the conservative verdict (ADR-057): gone-vs-evaded is undecidable for a \
             single-conjunct draft → route-to-human."
        );
    }

    /// A single-conjunct draft whose shape IS present reads Dormant (no near-miss is
    /// possible, but the shape is alive — keep it). The Indeterminate verdict is ONLY
    /// for the shape-absent single-conjunct case.
    #[test]
    fn single_conjunct_shape_present_is_dormant() {
        let c = corpus("fn live() { x.unwrap(); }");
        assert_eq!(
            silent_status(&body_calls_unwrap(), &c),
            SilentStatus::Dormant,
            "single-conjunct with the shape PRESENT is Dormant (alive, keep) — \
             Indeterminate is only for the undecidable shape-absent case."
        );
    }

    /// EVADING does not require the exact shape to still be present: a MULTI-conjunct
    /// draft whose exact shape mutated AWAY but left a near-miss is still Evading
    /// (the near-miss check runs regardless of shape-presence).
    #[test]
    fn multi_conjunct_shape_mutated_away_with_near_miss_is_evading() {
        // No item binds the full Clone+Debug draft (shape absent), but a Clone-only
        // struct is a near-miss (one constraint — Debug — away).
        let c = corpus("#[derive(Clone)] struct OnlyClone;");
        assert_eq!(
            silent_status(&derives_clone_and_debug(), &c),
            SilentStatus::Evading,
            "a multi-conjunct draft whose exact shape is absent but which has a \
             near-miss in the corpus is EVADING — the near-miss check must not be \
             gated on shape-present, or a mutated-away defect reads obsolete."
        );
    }
}
