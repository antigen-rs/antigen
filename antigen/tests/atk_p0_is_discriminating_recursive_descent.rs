//! ATK-P0 — the `is_discriminating` recursive-descent vacuity.
//!
//! **STATUS: P0 LANDED.** These tests were born-red against
//! v0.6-dev and are now GREEN: `is_discriminating` descends into combinator
//! children (`self_tolerance.rs`). The `#[ignore]` markers are dropped; the three
//! degenerate-input asserts + the positive control are the live regression spec.
//!
//! # What this defends (the v0.6 P0 "done" definition)
//!
//! `self_tolerance::is_discriminating` (the (A)-binary partition, ADR-047 OQ3) is
//! the sub-primitive ALL of GATE-G's non-vacuity checks key on
//! (`has_discriminating_conjunct`, `is_near_miss`'s remainder-guard, the
//! C-side `is_degenerate` refusal). At v0.6-dev it was **NON-RECURSIVE**: the three
//! boolean combinators returned `true` UNCONDITIONALLY —
//!
//! ```ignore
//! Constraint::AllOf(_) | Constraint::AnyOf(_) | Constraint::Not(_) => true,
//! ```
//!
//! — "which carry discriminating children" (the comment). But that was an
//! ASSUMPTION, not a check: it never descended to confirm the children actually
//! discriminate. The degenerate input that breaks it is a combinator wrapping ONLY
//! bare-structural anchors:
//!
//! - `Not(Item(Struct))` — matches the entire complement "everything that is not a
//!   struct." A draft whose sole discriminating signal is a NEGATED anchor carries
//!   NO real in-family discrimination — it fabricates against a huge complement
//!   (the armed vacuity: a negation matches the complement, so it OVER-binds, the
//!   self-inflicted-autoimmunity failure-mode). Yet `has_discriminating_conjunct`
//!   reports it as discriminating, so GATE-G's (A)-binary refusal does NOT fire.
//! - `AllOf([Item(Impl), ImplOfTrait("Drop")])` — a wrapped bare-structural
//!   skeleton (binds every `Drop` impl); reported discriminating, refusal skipped.
//! - `AnyOf([Item(Struct), Item(Enum)])` — a disjunction of pure anchors; binds the
//!   whole "struct-or-enum" family; reported discriminating, refusal skipped.
//!
//! # Why this WAS born-red (the vacuity P0 closed)
//!
//! This is the MUST-FIX-BEFORE-ADR-051 carried from v0.5 (briefing.md:75,
//! `safety/gate-g-combinator-anchor-vacuity`). It is LATENT (unreachable) under
//! today's `anti_unify` (which emits a flat top-level `all_of` of leaves — no
//! `Not`, no nested combinator). It becomes LIVE the moment `narrow()` / `persist`
//! / user-`parse` can introduce a `Not` or a nested combinator into a draft — i.e.
//! exactly the ADR-051 narrow/persist surface. Shipping that surface over a
//! non-recursive `is_discriminating` would re-open the vacuity as a SILENT WRONG
//! promote — so the descent had to land first.
//!
//! P0 (LANDED) = `is_discriminating` recursively descends: a combinator is
//! discriminating IFF it (recursively) contains at least one discriminating leaf.
//! `Not(c)` is discriminating iff `c` is; `AllOf`/`AnyOf` iff ANY child is. The
//! asserts below are GREEN and the gate now refuses the wrapped-anchor drafts.
//!
//! The failing test that DEFINED P0's done.

use antigen::learn::self_tolerance::has_discriminating_conjunct;
use antigen_fingerprint::{Constraint, Fingerprint, ItemKind};

/// A draft whose ONLY top-level conjunct is `Not(bare-structural-anchor)`.
/// `Not(Item(Struct))` matches the entire "not a struct" complement — no real
/// in-family discrimination, a fabricating over-bind. The (A)-binary partition
/// MUST classify it as NON-discriminating so GATE-G refuses it.
#[test]
fn not_of_bare_anchor_is_not_discriminating() {
    let draft = Fingerprint {
        constraints: vec![Constraint::Not(Box::new(Constraint::Item(
            ItemKind::Struct,
        )))],
    };
    assert!(
        !has_discriminating_conjunct(&draft),
        "Not(Item(Struct)) negates a pure structural anchor — it carries NO \
         discriminating signal (it binds the whole 'not a struct' complement). \
         has_discriminating_conjunct reporting it discriminating means GATE-G's \
         (A)-binary refusal is SKIPPED for a fabricating over-bind. P0: \
         is_discriminating must descend into Not's child."
    );
}

/// A draft whose sole conjunct is `AllOf([Item, ImplOfTrait])` — a wrapped
/// bare-structural skeleton that binds every `Drop` impl. Normalization flattens a
/// top-level `AllOf`, but a nested/wrapped one a future `narrow()` could emit must
/// also be seen through: an `AllOf` of only anchors does not discriminate.
#[test]
fn all_of_only_anchors_is_not_discriminating() {
    let inner = Constraint::AllOf(vec![
        Constraint::Item(ItemKind::Impl),
        Constraint::ImplOfTrait("Drop".into()),
    ]);
    // Wrap once more so it is a genuine combinator-CHILD case, not the top-level
    // AllOf that normalized_top_level flattens away.
    let draft = Fingerprint {
        constraints: vec![Constraint::Not(Box::new(inner))],
    };
    assert!(
        !has_discriminating_conjunct(&draft),
        "Not(AllOf([Item(Impl), ImplOfTrait(Drop)])) wraps ONLY structural anchors \
         — no discriminating leaf anywhere in the tree. P0: is_discriminating must \
         descend recursively; a combinator discriminates IFF a descendant leaf does."
    );
}

/// `AnyOf([Item(Struct), Item(Enum)])` — a disjunction of pure anchors. Binds the
/// whole "struct or enum" family. No discriminating leaf → must be NON-discriminating.
#[test]
fn any_of_only_anchors_is_not_discriminating() {
    let draft = Fingerprint {
        constraints: vec![Constraint::AnyOf(vec![
            Constraint::Item(ItemKind::Struct),
            Constraint::Item(ItemKind::Enum),
        ])],
    };
    assert!(
        !has_discriminating_conjunct(&draft),
        "AnyOf([Item(Struct), Item(Enum)]) is a disjunction of pure structural \
         anchors — it discriminates nothing, it binds the whole struct-or-enum \
         family. P0: is_discriminating must descend into AnyOf's children."
    );
}

/// POSITIVE CONTROL — the fix must NOT over-correct. A combinator that DOES carry a
/// discriminating leaf stays discriminating. `Not(BodyCalls("unwrap"))` negates a
/// real discriminating signal; `AnyOf([Item, BodyCalls])` has one. These stay TRUE
/// after the recursive-descent fix (proving the fix narrows, not flips).
#[test]
fn combinator_with_a_real_discriminating_leaf_stays_discriminating() {
    let not_of_signal = Fingerprint {
        constraints: vec![Constraint::Not(Box::new(Constraint::BodyCalls(
            "unwrap".into(),
        )))],
    };
    assert!(
        has_discriminating_conjunct(&not_of_signal),
        "Not(BodyCalls(unwrap)) negates a REAL discriminating signal — it stays \
         discriminating after P0 (the fix must descend, not blanket-false combinators)."
    );

    let any_of_mixed = Fingerprint {
        constraints: vec![Constraint::AnyOf(vec![
            Constraint::Item(ItemKind::Impl),
            Constraint::BodyCalls("unwrap".into()),
        ])],
    };
    assert!(
        has_discriminating_conjunct(&any_of_mixed),
        "AnyOf([Item(Impl), BodyCalls(unwrap)]) contains a discriminating leaf — \
         stays discriminating after P0."
    );
}
