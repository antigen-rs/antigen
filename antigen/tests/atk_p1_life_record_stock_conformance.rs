//! ATK-P1 — the life-record STOCK conformance spec (BORN-RED).
//!
//! **STATUS: BORN-RED. These tests do NOT COMPILE yet** — the life-record STOCK
//! (`antigen::learn::life_record`, ADR-059) is the ONE genuinely-ABSENT organ of
//! v0.6 (the keystone taproot, §3 of `briefing-for-pioneers.md`). This file is the
//! adversarial *definition of done* for P1: the failing test the Pioneer who builds
//! the STOCK makes green. When P1 lands, drop the `#![cfg(...)]` born-red gate (see
//! below), the type emerges to satisfy these asserts, and they become the live
//! regression spec.
//!
//! It is born-red by **non-compilation against an absent API**, the strongest form
//! for an absent organ: the test names the contract surface so precisely that a
//! pathmaker who builds a NON-conformant STOCK (e.g. a `story: String` column) finds
//! these asserts will not compile — the type must be a typed event-stream to satisfy
//! them. That IS the C1 conformance check made executable.
//!
//! # The four load-bearing invariants this DEFINES (each = one #[test])
//!
//! 1. **C1 CONFORMANCE (ADR-058) — the story facet is a TYPED EVENT-STREAM, not
//!    free-text-primary.** The deepest one. The story is constructed by APPENDING
//!    typed lifecycle events (`Born` / `Matured` / `Ratified{who,why,when}` /
//!    `Fired` / `Drifted` / `Retired` / …); prose is a one-way RENDERING of that
//!    stream, NEVER an input. A `story: String` the agent must NL-parse is
//!    NON-CONFORMANT (it reintroduces the reader-seam ADR-058 dissolves). The
//!    `Ratified.why` LEAF-PAYLOAD EXCEPTION holds: a free human-text `why` may be a
//!    value AT a typed leaf (opaque attestation payload, ADR-020), but prose may
//!    NEVER be the STRUCTURE of the record.
//!
//! 2. **APPEND-ONLY.** The record exposes `append`/push of an event and a read of
//!    the stream — there is NO `mutate` / `set` / `delete` / `update` surface. A
//!    forget is a *pushed event*, not an erasure. (ADR-059: antigen's first
//!    persistent mutable substrate is mutable ONLY by append.)
//!
//! 3. **TOMBSTONE-NOT-SILENCE.** Forgetting/retiring a class APPENDS a tombstone
//!    event that REMAINS readable in history. The class's *current* state derives to
//!    "retired", but the event ("we tried this, retired on DATE; if re-proposed,
//!    already walked") is never deleted-to-silence. The cold agent's only signal a
//!    dead end was already walked.
//!
//! 4. **CURRENT-STATE-IS-DERIVED, never stored (ADR-059 identity-amendment).** The
//!    record stores EVENTS, never a claim-about-current-state ("class K is stale").
//!    "Is this class stale/retired?" is a FUNCTION OVER the event stream, not a
//!    stored field that must be kept in sync. Storing events != storing claims; only
//!    the latter drifts. This is what makes the life-record drift-immune like `.git`.
//!
//! # Why born-red and not just a design doc
//!
//! A born-red test is the adversary's backbone for self-close: "done" = THESE asserts
//! pass, not "the happy path renders a nice story." The C1 trap (free-text-primary)
//! is exactly the kind of thing a builder reaches for under flow ("just add a notes
//! String") — this test makes that choice fail to compile. Append-only and
//! tombstone-not-silence are the silent-failure modes: a STOCK that lets you mutate
//! or delete a record looks identical in the happy path and is catastrophic in the
//! cold-agent-inherits-it path.
//!
//! Author: v06-the-maturing-organism--build--adversarial (the failing test that
//! DEFINES P1's done, handed to the STOCK pathmaker).
//!
//! ----------------------------------------------------------------------------
//! STATUS: P1a LANDED (v0.6 Pioneers). The file-level born-red gate is DROPPED —
//! the four P1a asserts below now compile against the real
//! `antigen::learn::life_record` and are GREEN (the live regression spec). The
//! ONE P1b test (`hand_authored_story_diverging_from_the_score_is_flagged`) stays
//! gated behind `feature = "stock-score-born-red"` — it needs MATURE's score type
//! (the `Scored` event + `check_story_coherence`), which is downstream.
//! ----------------------------------------------------------------------------

// NOTE TO THE STOCK PATHMAKER: the names below (LifeRecord, LifeEvent, Actor, …) are
// the adversary's PROPOSED surface, asserting the CONTRACT not the spelling. If you
// name the type `Autobiography` or the event `RatifiedBy`, rename here in the same
// change — the asserts are what's load-bearing, the identifiers are negotiable
// (recognition-not-design). What is NON-negotiable: (a) the story is a typed event
// enum, (b) no mutate/delete surface, (c) a tombstone event persists, (d) current
// state is a fn over the stream.
use antigen::learn::life_record::{Actor, LifeEvent, LifeRecord};

/// A class is keyed by its kebab-case `#[antigen(name = "...")]` identifier.
const CLASS: &str = "parallel-state-trackers-diverge";

// ---------------------------------------------------------------------------
// 1. C1 CONFORMANCE — the story is a TYPED EVENT-STREAM, not free-text-primary.
// ---------------------------------------------------------------------------

/// The story is BUILT by appending typed lifecycle events. This compiles ONLY if
/// `LifeEvent` is a typed enum (`Born`, `Matured`, `Ratified{..}`, …). A pathmaker
/// who models the story as `story: String` cannot satisfy this — there is no
/// `LifeEvent::Born` to push. THAT non-compilation is the C1 enforcement.
#[test]
fn story_is_a_typed_event_stream_not_free_text() {
    let mut rec = LifeRecord::new(CLASS);
    rec.append(LifeEvent::Born);
    rec.append(LifeEvent::Matured);
    rec.append(LifeEvent::Ratified {
        // LEAF-PAYLOAD EXCEPTION: `why` may be free human text AS A LEAF VALUE
        // (opaque attestation, ADR-020). Prose-at-a-leaf is allowed; prose-as-the
        // -STRUCTURE-of-the-record is forbidden. The STRUCTURE here is the typed
        // `Ratified` variant, traversable WITHOUT parsing `why`.
        who: Actor::Human("alice".into()),
        why: "the diverging-tracker bug bit us twice — worth a standing defense".into(),
    });

    // The agent reader consumes the NARRATIVE by matching typed events — NO
    // NL-parsing of a prose blob. This is the C1 test made executable: the reader
    // can answer "was this class ratified, and by whom?" purely structurally.
    let ratified_by = rec.events().iter().find_map(|e| match e {
        LifeEvent::Ratified { who, .. } => Some(who.clone()),
        _ => None,
    });
    assert_eq!(
        ratified_by,
        Some(Actor::Human("alice".into())),
        "the agent must read 'who ratified' from a TYPED event, never by NL-parsing \
         a free-text story blob — that is the ADR-058 C1 conformance criterion."
    );
}

/// Prose is a ONE-WAY RENDERING of the typed stream — a projection, never a source.
/// `render()` is a pure function of the events: the same event sequence renders the
/// same prose, and the prose carries NO information not present in the events. (If a
/// builder lets prose be authored independently, story-vs-struct drift — antigen's
/// own nightmare — reappears inside its life-record; persona-c REQ-4 forbids it.)
#[test]
fn prose_is_a_one_way_rendering_of_the_events() {
    let mut a = LifeRecord::new(CLASS);
    a.append(LifeEvent::Born);
    a.append(LifeEvent::Matured);

    let mut b = LifeRecord::new(CLASS);
    b.append(LifeEvent::Born);
    b.append(LifeEvent::Matured);

    assert_eq!(
        a.render(),
        b.render(),
        "render() must be a pure projection of the typed events — identical event \
         streams render identical prose. Prose is an OUTPUT of the structure, never \
         an independent input (ADR-058 / persona-c REQ-4: no story-vs-struct drift)."
    );
}

// ---------------------------------------------------------------------------
// 2. APPEND-ONLY — no mutate / set / delete surface.
// ---------------------------------------------------------------------------

/// Appending events only grows the stream; earlier events are never altered or
/// removed. (The absence of a `mutate`/`delete` API is enforced structurally — there
/// is nothing to call. This test pins the positive half: history only grows.)
#[test]
fn the_record_is_append_only_history_only_grows() {
    let mut rec = LifeRecord::new(CLASS);
    rec.append(LifeEvent::Born);
    let after_one = rec.events().len();
    rec.append(LifeEvent::Matured);
    let after_two = rec.events().len();

    assert_eq!(after_one, 1, "first append yields exactly one event");
    assert_eq!(
        after_two, 2,
        "second append GROWS the stream — never replaces"
    );
    assert!(
        matches!(rec.events()[0], LifeEvent::Born),
        "the earliest event is never mutated by a later append — append-only."
    );
}

// ---------------------------------------------------------------------------
// 3. TOMBSTONE-NOT-SILENCE — a forget WRITES a record, never deletes-to-silence.
// ---------------------------------------------------------------------------

/// Retiring a class appends a `Retired` tombstone event that REMAINS readable. The
/// dead-end stays visible to the next (cold) agent: "we tried this, retired on DATE."
/// A STOCK that dropped the record on forget would silence the dead end — the exact
/// failure tombstone-not-silence exists to prevent.
#[test]
fn forgetting_writes_a_tombstone_that_persists_in_history() {
    let mut rec = LifeRecord::new(CLASS);
    rec.append(LifeEvent::Born);
    rec.append(LifeEvent::Retired);

    let has_tombstone = rec.events().iter().any(|e| matches!(e, LifeEvent::Retired));
    assert!(
        has_tombstone,
        "retiring a class must APPEND a tombstone event that survives in history — \
         a forget is evidence, NOT a hole (tombstone-not-silence). The cold agent's \
         only signal a dead end was already walked is this persisted record."
    );
    // The Born event still precedes the tombstone — the full arc is legible.
    assert!(
        matches!(rec.events()[0], LifeEvent::Born),
        "the tombstone is appended AFTER the birth — the whole life arc stays readable."
    );
}

// ---------------------------------------------------------------------------
// 4. CURRENT-STATE-IS-DERIVED, never stored (ADR-059 identity-amendment).
// ---------------------------------------------------------------------------

/// "Is this class retired?" is a FUNCTION OVER the event stream, not a stored
/// `is_retired` field. The record stores EVENTS (what happened), and current-state is
/// DERIVED — so there is no claim-about-current-state to drift out of sync. This is
/// what makes the life-record drift-immune like `.git` (events, not claims).
#[test]
fn current_state_derives_from_events_never_stored_as_a_claim() {
    let mut live = LifeRecord::new(CLASS);
    live.append(LifeEvent::Born);
    live.append(LifeEvent::Matured);
    assert!(
        !live.is_retired(),
        "a class with no Retired event derives to NOT-retired — state is read FROM \
         the stream, not a stored flag."
    );

    live.append(LifeEvent::Retired);
    assert!(
        live.is_retired(),
        "after a Retired event is appended, is_retired() DERIVES true by folding the \
         stream — antigen never stores the claim 'class K is retired' (that would \
         drift); it stores the EVENT and derives the state (ADR-059)."
    );
}

// ---------------------------------------------------------------------------
// 5. STORY-STRUCT COHERENCE (persona-c REQ-4) — the hand-authored WRITE-seam.
//    DEFERRED: this case needs the SCORE field (a hand-authored "maturing well"
//    contradicted by a DROPPING score-trajectory) — and the score field is
//    MATURE-gated (P1b, downstream of the affinity-2-vector TYPE), NOT turn-zero.
//    Gated separately so the turn-zero P1a build (events + projection) is not
//    blocked on MATURE. Lands when the score-trajectory field exists.
// ---------------------------------------------------------------------------

/// persona-c REQ-4 (aristotle's criterion #3, the WRITE-seam that survives the READ
/// -seam dissolution): a HAND-AUTHORED story carrying a witness-link must be
/// COHERENCE-CHECKABLE against the struct — a "this class is maturing well" while the
/// score-trajectory DROPS is antigen's own drift-nightmare reproduced inside its
/// life-record, and the coherence-check MUST flag it. (The pure-projection path (a) is
/// covered by `prose_is_a_one_way_rendering_of_the_events`; this pins the hand-authored
/// path (b). Free-text-only with no coherence path is the born-red failure.)
///
/// SEQUENCING: needs the score-trajectory field → MATURE-gated (P1b), not turn-zero.
#[test]
#[cfg(feature = "stock-score-born-red")] // separate gate: MATURE's score type must exist
fn hand_authored_story_diverging_from_the_score_is_flagged() {
    let mut rec = LifeRecord::new(CLASS);
    rec.append(LifeEvent::Born);
    // Two score-points showing a DROPPING affinity trajectory.
    rec.append(LifeEvent::Scored {
        recall: 0.9,
        precision: 0.8,
    });
    rec.append(LifeEvent::Scored {
        recall: 0.5,
        precision: 0.4,
    }); // dropped

    // A hand-authored optimistic note that CONTRADICTS the dropping score.
    let divergence =
        rec.check_story_coherence("this class is maturing well — affinity climbing steadily");
    assert!(
        divergence.is_some(),
        "a hand-authored 'maturing well' story while the score-trajectory DROPS must \
         be FLAGGED by the coherence-check (persona-c REQ-4 / sub-clause-F re-validation). \
         A free-text story with no coherence path against the struct is the born-red \
         failure — story-vs-struct drift is antigen's own nightmare inside its own record."
    );
}
