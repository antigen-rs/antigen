//! ATK-SURVEY-READER-BLIND — the refutation of CURATE's moral center,
//! attacked at the UPSTREAM sensor seam the prior CURATE/discriminator ATKs never
//! reached.
//!
//! **STATUS: born-red on 6aedace; GREEN once `silent_status` uses operational
//! near-miss capability instead of a bare conjunct count.**
//!
//! # Why the prior passes missed this
//!
//! `atk_curate_forget_path.rs` and `discriminator.rs`'s own tests prove the forget
//! gate holds *given a `SilentStatus`/`ClassVerdict` value* — they start from
//! `classify(SilentStatus::Obsolete, …)` and sweep the closed enum. None of them
//! exercise `silent_status` — the function that PRODUCES the `SilentStatus` from a
//! real `Fingerprint` + corpus. The whole forget chain is:
//!
//! ```text
//! silent_status(draft, corpus) -> SilentStatus::Obsolete   <-- THIS STEP, unattacked
//!   -> classify(Obsolete, !defended) -> ClassVerdict::Obsolete
//!   -> curate(Obsolete) -> CurationAction::Forget           (gate proven, holds)
//! ```
//!
//! The gate holds; the WRONG KEY is handed to it. A class that is actively *evading*
//! is read as `Obsolete` and forgotten — antigen's own `RoutingTableStale` nightmare,
//! re-introduced by the very organ built to fight it.
//!
//! # The root cause (`reader.rs` -> `self_tolerance.rs`)
//!
//! `silent_status` trusts an absence as obsolescence iff `is_near_miss_capable(draft)`
//! — which is a pure **COUNT**: `normalized_top_level(draft).len() >= 2`
//! (`self_tolerance.rs::is_near_miss_capable`). But the ACTUAL near-miss detector
//! (`is_near_miss`) requires more than count: a near-miss exists only if dropping ONE
//! conjunct leaves a remainder that still **matches** the item AND still
//! **discriminates** (`has_discriminating_conjunct`, the ADR-047 Amendment 2 fix).
//!
//! So a draft with ≥2 conjuncts but only ONE discriminating conjunct (the rest bare
//! structural/identity anchors — `Item`, `NameMatches`, `ImplOfTrait`) is
//! near-miss-**capable by count** yet near-miss-**BLIND in operation**: the only
//! conjunct-drop that could yield a near-miss (drop the anchor, keep the
//! discriminator) fails the moment the defect mutated past that one discriminator —
//! exactly the evasion case. This is the SAME structural blindness the single-conjunct
//! guard (`Indeterminate`) was built to close, slipping through on a 2-conjunct count.
//!
//! # The fix
//!
//! `silent_status`'s trustworthy-absence test must use *operational* near-miss
//! capability, not a count: a near-miss can be formed only by dropping a NON-essential
//! conjunct and leaving a **discriminating** remainder — so a draft is trustworthy-
//! absent only if at least TWO of its (normalized) top-level conjuncts discriminate.
//! A draft with a single discriminating conjunct (regardless of how many bare anchors
//! pad the count) is as near-miss-blind as a single-conjunct draft and must read
//! `Indeterminate` (route-to-human), never `Obsolete` (forget).

use antigen::learn::adwin::DEFAULT_DELTA;
use antigen::learn::affinity::Affinity;
use antigen::learn::curate::{CurationAction, curate};
use antigen::learn::discriminator::fused_classify;
use antigen::learn::reader::{SilentStatus, silent_status};
use antigen_fingerprint::{Constraint, Fingerprint, GlobPattern, ItemKind};

fn corpus(src: &str) -> Vec<syn::Item> {
    syn::parse_file(src).expect("ATK corpus parses").items
}

// ---------------------------------------------------------------------------
// ATK-SURVEY-READER-1 — the KILLER: a bare anchor DEFEATS the single-conjunct
// evasion guard. The SAME evading defect (unwrap -> expect) reads Indeterminate
// as a single-conjunct draft (correctly protected) but Obsolete (forget!) the
// moment a non-discriminating NAME anchor pads the conjunct count to 2.
// ---------------------------------------------------------------------------

#[test]
fn atk_survey_reader1_name_anchor_defeats_single_conjunct_evasion_guard() {
    // The shipped guard correctly protects a single-conjunct body-call class whose
    // defect mutated within its conjunct's family (unwrap -> expect): Indeterminate.
    let single = Fingerprint {
        constraints: vec![Constraint::BodyCalls("unwrap".to_string())],
    };
    // The SAME class with a bare NAME anchor bolted on (adds zero discriminating
    // power; `name = matches("handle_*")` only names what the item IS).
    let anchored = Fingerprint {
        constraints: vec![
            Constraint::NameMatches(GlobPattern("handle_*".to_string())),
            Constraint::BodyCalls("unwrap".to_string()),
        ],
    };
    // The defect mutated unwrap -> expect inside the same named fn. The shape is gone
    // (no unwrap call) but the defect is plainly still present in the unwrap-family.
    let c = corpus("fn handle_request() { x.expect(\"boom\"); }");

    // Baseline: the guard works for the single-conjunct draft.
    assert_eq!(
        silent_status(&single, &c),
        SilentStatus::Indeterminate,
        "shipped single-conjunct guard: an unwrap class whose shape is absent reads \
         Indeterminate (route-to-human, never forget) — the defect may have mutated \
         within its conjunct's family.",
    );

    // born-red: the anchored draft is the same evading defect. It must NOT become
    // forgettable just because the conjunct count crossed 2.
    let status = silent_status(&anchored, &c);
    assert_ne!(
        status,
        SilentStatus::Obsolete,
        "ATK-SURVEY-READER-1: adding a bare NAME anchor to body_calls(\"unwrap\") must \
         NOT flip a live evading class (unwrap -> expect) from Indeterminate to \
         Obsolete. `is_near_miss_capable` trusts the >=2 conjunct COUNT, but near-miss \
         detection is structurally BLIND here (the only near-miss-forming drop keeps \
         the one discriminator, which no longer matches the mutated item). Reading \
         Obsolete forgets a still-needed defense — antigen's own RoutingTableStale \
         nightmare, produced by the very organ built to fight it. (status={status:?})",
    );
}

// ---------------------------------------------------------------------------
// ATK-SURVEY-READER-2 — the general form: a draft with exactly ONE discriminating
// conjunct (padded with anchors to any count) is near-miss-blind and must never
// read Obsolete when its single discriminator's shape is absent.
// ---------------------------------------------------------------------------

#[test]
fn atk_survey_reader2_one_discriminator_padded_with_anchors_is_not_obsolete() {
    // Item(Struct) [anchor] + Derives("Serialize") [the ONE discriminator].
    let draft = Fingerprint {
        constraints: vec![
            Constraint::Item(ItemKind::Struct),
            Constraint::Derives("Serialize".to_string()),
        ],
    };
    // The defect mutated within the derive-family: the struct now derives Deserialize.
    // The full draft no longer matches; near-miss is structurally blind (drop the
    // anchor -> [Derives("Serialize")] does not match a Deserialize-deriving item;
    // drop the discriminator -> [Item(Struct)] matches but does not discriminate).
    let c = corpus("#[derive(Deserialize)] struct ConfigThing { x: u8 }");
    let status = silent_status(&draft, &c);
    assert_ne!(
        status,
        SilentStatus::Obsolete,
        "ATK-SURVEY-READER-2: a draft with a single DISCRIMINATING conjunct padded by \
         a bare structural anchor is near-miss-blind (no conjunct-drop can witness an \
         in-family mutation). Its absence is not trustworthy as obsolescence — it must \
         read Indeterminate (route-to-human), not Obsolete (forget). (status={status:?})",
    );
}

// ---------------------------------------------------------------------------
// ATK-SURVEY-READER-3 — the count/operational-capability split, stated directly:
// a draft can be `is_near_miss_capable` (>=2 conjuncts) yet have NO item in the
// universe that `is_near_miss` could ever flag (every conjunct-drop leaves a
// non-discriminating remainder). When that draft's shape is absent it must route
// to human, never forget.
// ---------------------------------------------------------------------------

#[test]
fn atk_survey_reader3_two_anchors_capable_by_count_blind_in_operation() {
    // Two bare anchors: Item(Struct) + NameMatches("Foo*"). No discriminating conjunct
    // at all — but is_near_miss_capable returns true (len == 2). Every conjunct-drop
    // leaves a single bare anchor (non-discriminating), so is_near_miss is structurally
    // false for EVERY possible item. The absence is therefore never verifiable as
    // obsolescence.
    let draft = Fingerprint {
        constraints: vec![
            Constraint::Item(ItemKind::Struct),
            Constraint::NameMatches(GlobPattern("Foo*".to_string())),
        ],
    };
    let c = corpus("struct Bar; enum Foo { A }");
    let status = silent_status(&draft, &c);
    assert_ne!(
        status,
        SilentStatus::Obsolete,
        "ATK-SURVEY-READER-3: a draft on which `is_near_miss` is structurally false for \
         every item (no conjunct-drop ever leaves a discriminating remainder) has an \
         absence that near-miss detection CANNOT verify — it must read Indeterminate, \
         never Obsolete. `is_near_miss_capable` (a pure count) reports it capable; the \
         operational detector is blind. (status={status:?})",
    );
}

// ---------------------------------------------------------------------------
// ATK-SURVEY-READER-4 — END-TO-END through the PRODUCTION pipeline: the bit-3
// blindness slips THROUGH the conservatism-JOIN, because the JOIN only guards
// the LABELLED-blind state (`SilentStatus::Indeterminate`). silent_status
// mislabels the evasion-blind draft as a CONFIDENT `Obsolete`, so the JOIN never
// sees a blind channel and `fused_classify` → curate → Forget retires the live
// evading class. This proves the finding is reachable through the canonical
// fused entry the curation pipeline calls (`fused_classify`), not a reader quirk.
// ---------------------------------------------------------------------------

#[test]
fn atk_survey_reader4_blindness_bypasses_conservatism_join_to_forget() {
    // The evading defect (unwrap -> expect) with a bare name anchor padding the count.
    let anchored = Fingerprint {
        constraints: vec![
            Constraint::NameMatches(GlobPattern("handle_*".to_string())),
            Constraint::BodyCalls("unwrap".to_string()),
        ],
    };
    let c = corpus("fn handle_request() { x.expect(\"boom\"); }");

    // The JOIN's UnderPowered arm masks this bug ONLY while the class is loud-axis-blind
    // (n≈8, antigen's CURRENT scale). The moment a class accumulates a stationary
    // trajectory long enough to be POWERED, detect() returns NoDrift (the honest "I
    // looked, no drift" state) — NOT UnderPowered — and the JOIN passes the bit-3 verdict
    // straight through. Verified empirically: n>=100 stationary reads NoDrift (n=40/60 are
    // still UnderPowered). This is the exact regime ADR-065 says the organ grows into
    // ("the SAME organ fires correctly once trajectories lengthen"), so the forget is a
    // structurally-determined LATENT bug, not a corner — it goes live as the system scales.
    let trajectory: Vec<Affinity> = (0..120)
        .map(|i| {
            let r = if i % 2 == 0 { 0.50 } else { 0.51 }; // tiny noise: variance>0, no change-point
            Affinity::new(r, r)
        })
        .collect();

    let silent = silent_status(&anchored, &c);
    let verdict = fused_classify(
        &trajectory,
        silent,
        /* defended */ false,
        DEFAULT_DELTA,
    );
    let action = curate(verdict);

    // If the bit-3 production mislabels the evasion-blind draft as Obsolete, and ADWIN
    // is NOT UnderPowered (a stationary long trajectory reads NoDrift), the JOIN passes
    // Obsolete through and CURATE forgets a live evading class.
    assert_ne!(
        action,
        CurationAction::Forget,
        "ATK-SURVEY-READER-4: a live evading class (unwrap -> expect, name-anchored) must \
         NOT reach CurationAction::Forget through the production fused_classify pipeline. \
         The conservatism-JOIN only guards the LABELLED-blind bit-3 state \
         (SilentStatus::Indeterminate); silent_status mislabels this evasion-blind draft \
         as a confident Obsolete, so the JOIN never sees a blind channel and the forget \
         fires. (silent={silent:?}, verdict={verdict:?}, action={action:?})",
    );
}
