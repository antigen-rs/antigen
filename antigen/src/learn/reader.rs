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

use crate::learn::self_tolerance::is_near_miss;

/// The silent-core no-FIRE verdict for one class against the live scan corpus — the
/// streamless split the obsolete/well-defended discriminator (P3) reads for classes
/// that carry no temporal signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SilentStatus {
    /// The fingerprint's **shape is gone** — it matches NO item in the live corpus.
    /// The failure-shape this class guards no longer exists in the code → an
    /// **obsolete** candidate (safe to forget, subject to the other axes).
    Obsolete,
    /// The shape is **present** (the fingerprint matches a live item) but no
    /// **near-miss** appeared — the guarded shape exists, instances simply don't
    /// currently trip the defect. **Dormant**: keep it; it may fire when the shape
    /// recurs in a triggering form. (NOT obsolete — the shape is alive.)
    Dormant,
    /// The shape is present AND a **near-miss appeared** — a live item is one
    /// constraint from binding the draft (the defect mutated to just past the
    /// fingerprint). **Evading**: the red-queen signal — broaden/re-arm, do NOT
    /// forget. This is the cell ADWIN is blind to for silent classes.
    Evading,
}

/// Classify a silent class's no-FIRE state against the live scan `corpus` (P2,
/// bit-3 / silent-core facet) — STREAMLESS, reading only shipped primitives.
///
/// - **[`SilentStatus::Obsolete`]** iff the draft matches NO corpus item (shape gone).
/// - **[`SilentStatus::Evading`]** iff the shape is present AND some corpus item is a
///   [`is_near_miss`] (the defect mutated one constraint past the fingerprint).
/// - **[`SilentStatus::Dormant`]** otherwise (shape present, no near-miss).
///
/// Reads [`Fingerprint::matches`] (shape-present) + [`is_near_miss`] (near-miss) over
/// the corpus — no temporal signal, no STOCK. The two axes are read in this order so
/// EVADING (the act-now red-queen case) is never masked by DORMANT: a present shape
/// with a near-miss is evading, not dormant.
#[must_use]
pub fn silent_status(draft: &Fingerprint, corpus: &[syn::Item]) -> SilentStatus {
    let shape_present = corpus.iter().any(|item| draft.matches(item));
    if !shape_present {
        return SilentStatus::Obsolete;
    }
    let near_miss_appeared = corpus.iter().any(|item| is_near_miss(draft, item));
    if near_miss_appeared {
        SilentStatus::Evading
    } else {
        SilentStatus::Dormant
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
}
