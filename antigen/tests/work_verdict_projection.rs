//! The four-valued prescriptive verdict projection (ADR-033 §Decision 3).
//!
//! `WorkVerdict` is the ADR-029 defense tri-state with the unsatisfied cell
//! temporally split by the frame (the verdict-lattice isomorphism). These tests
//! pin the projection logic — the part that is pure and complete independent of
//! the sidecar-attestation wiring (which the full `audit_prescriptive` adds):
//!
//! 1. The four variants are distinct (ATK-PRES-9).
//! 2. Exactly four variants exist (ATK-PRES-11, exhaustive match).
//! 3. The gem guard: `OutOfFrame` is NEVER `Overdue` — un-evaluable satisfaction
//!    yields `OutOfFrame` regardless of frame (ATK-PRES-8, the load-bearing test).
//! 4. The frame split: unsatisfied-within-frame ⇒ `Pending`; unsatisfied-past
//!    ⇒ `Overdue` (ATK-PRES-5/7).
//! 5. Satisfied ⇒ `Fulfilled` (ATK-PRES-6).
//! 6. `FrameState::classify` reads ISO dates, treats malformed as un-evaluable.

use antigen::audit::{FrameState, WorkVerdict};
use chrono::NaiveDate;

fn day(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
}

#[test]
fn the_four_variants_are_distinct() {
    // ATK-PRES-9.
    let all = [
        WorkVerdict::Pending,
        WorkVerdict::Fulfilled,
        WorkVerdict::Overdue,
        WorkVerdict::OutOfFrame,
    ];
    for (i, a) in all.iter().enumerate() {
        for (j, b) in all.iter().enumerate() {
            assert_eq!(i == j, a == b, "{a:?} vs {b:?} distinctness");
        }
    }
}

#[test]
fn exactly_four_variants() {
    // ATK-PRES-11: exhaustive match — a 5th variant fails to compile here.
    fn exhaustive(v: WorkVerdict) -> &'static str {
        match v {
            WorkVerdict::Pending => "pending",
            WorkVerdict::Fulfilled => "fulfilled",
            WorkVerdict::Overdue => "overdue",
            WorkVerdict::OutOfFrame => "out-of-frame",
        }
    }
    assert_eq!(exhaustive(WorkVerdict::Pending), "pending");
}

#[test]
fn unevaluable_is_out_of_frame_never_overdue() {
    // ATK-PRES-8, THE LOAD-BEARING GUARD. An un-evaluable satisfaction (unknown
    // who-ref) must be OutOfFrame regardless of frame — even past-frame. If this
    // collapsed to Overdue, every unknown-who-ref site would falsely read urgent.
    for frame in [
        FrameState::None,
        FrameState::Within,
        FrameState::Past,
        FrameState::Unparseable,
    ] {
        let v = WorkVerdict::project(false, /* evaluable */ false, frame);
        assert_eq!(
            v,
            WorkVerdict::OutOfFrame,
            "un-evaluable satisfaction with frame {frame:?} must be OutOfFrame, not {v:?}"
        );
        assert_ne!(
            v,
            WorkVerdict::Overdue,
            "the gem guard: OutOfFrame must NEVER collapse to Overdue (frame {frame:?})"
        );
    }
}

#[test]
fn unsatisfied_within_frame_is_pending_past_frame_is_overdue() {
    // ATK-PRES-5 (Pending) + ATK-PRES-7 (Overdue) — the temporal split.
    assert_eq!(
        WorkVerdict::project(false, true, FrameState::Within),
        WorkVerdict::Pending,
        "unsatisfied within frame is Pending (expected, not a failure)"
    );
    assert_eq!(
        WorkVerdict::project(false, true, FrameState::None),
        WorkVerdict::Pending,
        "unsatisfied with no frame is Pending (no deadline to be overdue against)"
    );
    assert_eq!(
        WorkVerdict::project(false, true, FrameState::Past),
        WorkVerdict::Overdue,
        "unsatisfied past frame is Overdue (loud)"
    );
}

#[test]
fn satisfied_is_fulfilled_regardless_of_frame() {
    // ATK-PRES-6.
    for frame in [FrameState::None, FrameState::Within, FrameState::Past] {
        assert_eq!(
            WorkVerdict::project(true, true, frame),
            WorkVerdict::Fulfilled,
            "satisfied is Fulfilled (frame {frame:?})"
        );
    }
}

#[test]
fn only_overdue_is_loud() {
    // ADR-023 loudness isomorphism: Pending is expected (quiet), OutOfFrame is
    // advisory, Fulfilled is clean — only Overdue is the loud verdict.
    assert!(WorkVerdict::Overdue.is_loud());
    assert!(!WorkVerdict::Pending.is_loud());
    assert!(!WorkVerdict::Fulfilled.is_loud());
    assert!(!WorkVerdict::OutOfFrame.is_loud());
}

#[test]
fn frame_state_classify_reads_iso_dates() {
    let today = day("2026-06-01");
    assert_eq!(FrameState::classify(None, today), FrameState::None);
    assert_eq!(
        FrameState::classify(Some("2027-01-01"), today),
        FrameState::Within,
        "a future date is within frame"
    );
    assert_eq!(
        FrameState::classify(Some("2026-06-01"), today),
        FrameState::Within,
        "today is within frame (on-or-before)"
    );
    assert_eq!(
        FrameState::classify(Some("2025-01-01"), today),
        FrameState::Past,
        "a past date is past frame"
    );
    assert_eq!(
        FrameState::classify(Some("not-a-date"), today),
        FrameState::Unparseable,
        "a malformed frame is un-evaluable, never silently within"
    );
}

#[test]
fn unparseable_frame_on_unsatisfied_need_is_out_of_frame() {
    // A garbage deadline cannot make a need Overdue (we can't read the deadline)
    // nor silently Pending — it is un-evaluable ⇒ OutOfFrame.
    assert_eq!(
        WorkVerdict::project(false, true, FrameState::Unparseable),
        WorkVerdict::OutOfFrame
    );
}
