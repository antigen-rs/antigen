//! ATK — MATURE degenerate-input stress tests.
//!
//! The `mature()` engine is the gradient-climb organ: it takes a rough draft and
//! widens it toward the Pareto frontier of (recall, precision). These tests attack the
//! degenerate and boundary inputs that the happy-path tests leave uncovered.
//!
//! ## Attacks
//!
//! | ID | Input | Expected behavior |
//! |----|-------|-------------------|
//! | ATK-MATURE-1 | empty clean corpus | precision=0.0 throughout; engine greedily maximizes recall to 1.0 with no autoimmunity check; documents the footgun |
//! | ATK-MATURE-2 | empty cluster | recall=0.0 for all drafts; no `pareto_improves_on`; halts at round 0; trajectory has exactly 1 entry |
//! | ATK-MATURE-3 | `max_rounds=0` | zero loop iterations; trajectory has exactly 1 entry; starting draft returned unchanged |
//! | ATK-MATURE-4 | coldspot draft (all framework, no CDR) | no mutation candidates; halts after 1 round; trajectory has exactly 1 entry |
//! | ATK-MATURE-5 | trajectory non-empty invariant | holds in all degenerate cases |
//! | ATK-MATURE-6 | `max_budget=0` passed to `mutation_budget` | `clamp(1, 0)` panics (min > max) — a genuine bug |

// The f64 comparisons below are exact-by-construction (integer ratios: 0/n = 0.0,
// n/n = 1.0, rate(0, 0) = 0.0 by the zero-guard) — same justification as affinity.rs's
// own tests. The allow covers all tests in this file.
#![allow(clippy::float_cmp, clippy::type_complexity)]

use antigen::learn::life_record::{LifeEvent, LifeRecord};
use antigen::learn::maturation::mature;
use antigen_fingerprint::Fingerprint;

fn fp(src: &str) -> Fingerprint {
    Fingerprint::parse(src).expect("test fingerprint parses")
}

fn defect_item() -> syn::Item {
    syn::parse_quote! {
        impl Drop for Defect { fn drop(&mut self) { self.0.flush(); self.1.unwrap(); } }
    }
}

fn clean_item() -> syn::Item {
    syn::parse_quote! {
        impl Drop for Clean { fn drop(&mut self) { self.0.ok(); } }
    }
}

// A draft with two CDR conjuncts (body_calls) and a framework anchor (impl_of_trait).
// One CDR conjunct is the differentiator between defect and clean.
fn two_cdr_draft() -> Fingerprint {
    fp(
        "all_of([item = impl, impl_of_trait(\"Drop\"), body_calls(\"flush\"), body_calls(\"unwrap\")])",
    )
}

// ---------------------------------------------------------------------------
// ATK-MATURE-1 — Empty clean corpus: precision=0.0 → unconstrained recall climb
//
// When clean_corpus is empty, Affinity::measure produces precision=0.0 for EVERY
// draft (rate(spared, 0) = 0.0 regardless of how many items were spared). A CDR drop
// that widens recall (without worsening the locked-at-0.0 precision) IS a
// pareto_improves_on — so the engine greedily climbs until recall=1.0 or no more
// drops exist, with NO autoimmunity check at all.
//
// This is a footgun: the caller must provide a non-empty clean corpus OR run the
// result through spare_clean before promotion. The engine alone cannot detect
// autoimmunity with an empty corpus.
// ---------------------------------------------------------------------------

/// ATK-MATURE-1a: with an empty clean corpus, maturation maximizes recall to 1.0
/// and sets precision to 0.0, widening the draft without any autoimmunity check.
#[test]
fn atk_mature1a_empty_clean_corpus_greedily_widens_recall() {
    let cluster = [defect_item()];
    let clean: &[syn::Item] = &[]; // empty — no autoimmunity check possible

    // Over-specific draft: requires BOTH flush AND unwrap. The cluster member has both,
    // so starting recall=1.0, but the clean corpus is empty so precision=0.0 throughout.
    let draft = two_cdr_draft();

    let mut record = LifeRecord::new("empty-corpus-test");
    let matured = mature(draft, &cluster, clean, &mut record, 8, 16);

    // Precision is 0.0 because rate(spared, 0) = 0.0 — the denominator is zero.
    // This is NOT "perfect precision" — it is "undefined precision, coerced to 0.0".
    assert_eq!(
        matured.affinity.precision, 0.0,
        "ATK-MATURE-1a: empty corpus forces precision=0.0 (rate(_, 0) = 0.0) — this is \
         undefined/meaningless precision, NOT a safety certificate. Got {:?}",
        matured.affinity
    );

    // The trajectory must be non-empty (ATK-MATURE-5 invariant).
    assert!(
        !matured.trajectory.is_empty(),
        "ATK-MATURE-1a: trajectory must be non-empty even with empty corpus"
    );
}

/// ATK-MATURE-1b: an empty clean corpus allows widening past a draft that WOULD bind
/// a clean item if one existed. The engine accepts the autoimmune widening because it
/// has no clean items to check against. Documents the missing safety check.
#[test]
fn atk_mature1b_empty_corpus_accepts_autoimmune_widening() {
    // Cluster: one member that calls unwrap.
    let m1: syn::Item = syn::parse_quote! {
        impl Drop for Defect { fn drop(&mut self) { self.1.unwrap(); } }
    };
    let cluster = [m1];
    let clean: &[syn::Item] = &[]; // no clean corpus — autoimmunity check absent

    // A draft that requires BOTH flush AND unwrap, but the cluster member only has unwrap.
    // Starting recall = 0.0 (flush not present), precision = 0.0 (empty corpus).
    // Dropping flush → recall = 1.0, precision = 0.0. That IS a pareto_improves_on
    // (recall went up, precision stayed the same). The engine accepts it even though
    // the resulting wider draft might bind clean code if a real corpus existed.
    let draft = fp(
        "all_of([item = impl, impl_of_trait(\"Drop\"), body_calls(\"flush\"), body_calls(\"unwrap\")])",
    );

    let start_recall = {
        use antigen::learn::affinity::Affinity;
        Affinity::measure(&draft, &cluster, clean).recall
    };
    // Starting recall < 1.0 because flush is required but the cluster member lacks it.
    assert_eq!(
        start_recall, 0.0,
        "ATK-MATURE-1b: the over-specific draft misses the cluster member (no flush)"
    );

    let mut record = LifeRecord::new("autoimmune-widening");
    let matured = mature(draft, &cluster, clean, &mut record, 8, 16);

    // The engine ACCEPTED a widening step (Matured milestone present) even though
    // it cannot certify the widened draft is autoimmune-safe.
    let has_matured_event = record.events().contains(&LifeEvent::Matured);
    assert!(
        has_matured_event,
        "ATK-MATURE-1b: with empty corpus the engine accepts a CDR drop (cannot reject \
         for autoimmunity what it cannot see). This widening is unguarded — the caller \
         MUST run spare_clean before promotion. Got final affinity: {:?}",
        matured.affinity
    );
}

// ---------------------------------------------------------------------------
// ATK-MATURE-2 — Empty cluster: recall=0.0 throughout, no improvement possible
//
// When cluster is empty, Affinity::measure produces recall=0.0 for ALL drafts
// (rate(0, 0) = 0.0). A CDR drop can only change precision (by widening to match
// potential clean items). If precision starts at some value, a widening could LOWER
// precision — and never raise recall. So pareto_improves_on can never fire: recall
// would need to increase (it can't, cluster is empty) or precision would need to
// increase (widening lowers or maintains precision). The engine halts at round 1.
// ---------------------------------------------------------------------------

/// ATK-MATURE-2: with an empty cluster, recall=0.0 for all drafts; no Pareto-
/// improvement is possible; the engine halts immediately; trajectory has exactly 1 entry.
#[test]
fn atk_mature2_empty_cluster_halts_immediately() {
    let cluster: &[syn::Item] = &[]; // empty — no defects to match
    let clean = [clean_item()];
    let draft = two_cdr_draft();

    let mut record = LifeRecord::new("empty-cluster");
    let matured = mature(draft.clone(), cluster, &clean, &mut record, 8, 16);

    assert_eq!(
        matured.affinity.recall, 0.0,
        "ATK-MATURE-2: empty cluster forces recall=0.0 — rate(0, 0) = 0.0"
    );
    assert_eq!(
        matured.trajectory.len(),
        1,
        "ATK-MATURE-2: no Pareto-improvement possible — engine halts at round 0; \
         trajectory has exactly 1 entry (the starting score). Got {:?}",
        matured.trajectory
    );
    assert_eq!(
        matured.draft, draft,
        "ATK-MATURE-2: starting draft returned unchanged (no accepted mutation)"
    );
    assert!(
        !record.events().contains(&LifeEvent::Matured),
        "ATK-MATURE-2: no Matured milestone when nothing improves"
    );
}

// ---------------------------------------------------------------------------
// ATK-MATURE-3 — max_rounds=0: zero loop iterations, starting draft returned
// ---------------------------------------------------------------------------

/// ATK-MATURE-3: `max_rounds=0` means the engine runs zero loop iterations and returns
/// the starting draft with a trajectory of exactly 1 entry (the initial score).
#[test]
fn atk_mature3_zero_max_rounds_returns_starting_draft() {
    let cluster = [defect_item()];
    let clean = [clean_item()];
    let draft = two_cdr_draft();

    let mut record = LifeRecord::new("zero-rounds");
    let matured = mature(draft.clone(), &cluster, &clean, &mut record, 8, 0);

    assert_eq!(
        matured.trajectory.len(),
        1,
        "ATK-MATURE-3: max_rounds=0 → zero loop iterations → trajectory has exactly 1 \
         entry. Got {:?}",
        matured.trajectory
    );
    assert_eq!(
        matured.draft, draft,
        "ATK-MATURE-3: starting draft returned unchanged when max_rounds=0"
    );
    assert!(
        !record.events().contains(&LifeEvent::Matured),
        "ATK-MATURE-3: no Matured milestone when zero rounds run"
    );
}

// ---------------------------------------------------------------------------
// ATK-MATURE-4 — Coldspot draft (all framework, no CDR): no candidates
//
// A draft with ONLY framework conjuncts (item, impl_of_trait, name — the frozen
// structural anchors) has no discriminating conjuncts, so mutation_candidates returns
// empty. The engine finds no candidates, best=None, and halts after 1 round.
// ---------------------------------------------------------------------------

/// ATK-MATURE-4: a coldspot draft (all framework anchors, no CDR) produces no mutation
/// candidates; the engine halts after 1 round; trajectory has exactly 1 entry.
#[test]
fn atk_mature4_coldspot_all_framework_no_cdr_halts_immediately() {
    let cluster = [defect_item()];
    let clean = [clean_item()];

    // All framework anchors — no CDR conjuncts.
    let draft = fp("all_of([item = impl, impl_of_trait(\"Drop\")])");

    let mut record = LifeRecord::new("coldspot");
    let matured = mature(draft.clone(), &cluster, &clean, &mut record, 8, 16);

    assert_eq!(
        matured.trajectory.len(),
        1,
        "ATK-MATURE-4: a coldspot draft (all framework, no CDR) has no mutation candidates; \
         engine halts immediately; trajectory has exactly 1 entry. Got {:?}",
        matured.trajectory
    );
    assert_eq!(
        matured.draft, draft,
        "ATK-MATURE-4: coldspot draft returned unchanged"
    );
    assert!(
        !record.events().contains(&LifeEvent::Matured),
        "ATK-MATURE-4: no Matured milestone for a coldspot"
    );
}

// ---------------------------------------------------------------------------
// ATK-MATURE-5 — Trajectory non-empty invariant under all degenerate inputs
//
// The doc says "Always non-empty: index 0 is the starting draft's affinity." This
// must hold for EVERY degenerate input combination.
// ---------------------------------------------------------------------------

/// ATK-MATURE-5: trajectory is always non-empty, regardless of degenerate inputs.
#[test]
fn atk_mature5_trajectory_always_nonempty() {
    let cluster_full = vec![defect_item()];
    let cluster_empty: Vec<syn::Item> = vec![];
    let clean_full = vec![clean_item()];
    let clean_empty: Vec<syn::Item> = vec![];
    let draft = two_cdr_draft();

    let cases: &[(&str, &[syn::Item], &[syn::Item], usize, usize)] = &[
        ("empty cluster", &cluster_empty, &clean_full, 8, 16),
        ("empty clean", &cluster_full, &clean_empty, 8, 16),
        ("both empty", &cluster_empty, &clean_empty, 8, 16),
        ("max_rounds=0", &cluster_full, &clean_full, 8, 0),
        ("max_budget=1", &cluster_full, &clean_full, 1, 16),
    ];

    for (label, cluster, clean, budget, rounds) in cases {
        let mut record = LifeRecord::new("trajectory-invariant");
        let matured = mature(draft.clone(), cluster, clean, &mut record, *budget, *rounds);
        assert!(
            !matured.trajectory.is_empty(),
            "ATK-MATURE-5: trajectory is empty for case '{label}' — invariant broken. \
             Doc says 'Always non-empty: index 0 is the starting draft's affinity.'"
        );
        assert!(
            !record.events().is_empty(),
            "ATK-MATURE-5: record has no events for case '{label}' — at minimum the \
             initial Scored event must be present"
        );
    }
}

// ---------------------------------------------------------------------------
// ATK-MATURE-6 — max_budget=0 panics: clamp(1, 0) with min > max
//
// Inside `mutation_budget`, the final line is:
//   (scaled.round() as usize).clamp(1, max_budget)
// When max_budget=0: clamp(1, 0) panics in Rust because min (1) > max (0).
// The `mature()` function calls `mutation_budget(current_affinity, max_budget)` each
// round. If max_budget=0 is passed, the first round panics.
//
// This is a BORN-RED test — it should fail (panic) until the bug is fixed.
// Fix: add a guard `if max_budget == 0 { return 0; }` before the clamp, and skip the
// candidate loop when budget == 0 (or treat max_budget=0 as "zero exploration" and
// return the starting draft immediately, same as max_rounds=0).
// ---------------------------------------------------------------------------

/// ATK-MATURE-6: `max_budget=0` causes a panic in `mutation_budget` via `clamp(1, 0)`
/// (min > max is a panic in Rust's `clamp`). BORN-RED: this test should FAIL (panic)
/// until the bug is fixed with a `max_budget=0` guard.
#[test]
#[should_panic(expected = "min > max")]
fn atk_mature6_zero_max_budget_panics_in_clamp() {
    let cluster = [defect_item()];
    let clean = [clean_item()];
    let draft = two_cdr_draft();

    let mut record = LifeRecord::new("zero-budget-panic");
    // This panics: mutation_budget(_, 0) calls (scaled as usize).clamp(1, 0)
    // and Rust's clamp panics when min > max.
    let _ = mature(draft, &cluster, &clean, &mut record, 0, 16);
}
