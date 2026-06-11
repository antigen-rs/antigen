//! ATK-047 — property/invariant tests for the GATE-G near-miss primitive
//! (ADR-047). The algebra no single example can cover: claims that must hold for
//! ALL inputs, where a generator hunts the counterexample.
//!
//! The §Q9 unit tests in `self_tolerance.rs` pin the *named* near-miss cases
//! (ATK-047-1..4, N4, the positive control). This file defends the **universally-
//! quantified** claims those examples instantiate — the invariants whose violation
//! would be a silent hole no hand-picked fixture happens to land on.
//!
//! Each invariant is stated as a property over generated `Fingerprint`s /
//! `syn::Item`s; `proptest` shrinks any counterexample to its minimal form. These
//! test the PUBLIC contract (`is_near_miss`, `corpus_witnesses_draft`,
//! `promote_if_safe`) the ADRs lock — NOT the gate's internal verdict tuning (that
//! is the keystone's own moving target, defended by the unit tests).
//!
//! # Negative-control record (the teeth-check that shaped these invariants)
//!
//! A green property test proves nothing unless it can go RED on the mutation it
//! defends. Each invariant here was run against a deliberate mutation of the gate
//! (in an isolated git worktree) to confirm it has teeth. The sweep (2026-06-11)
//! and what it CAUGHT:
//!
//! | mutation (in `self_tolerance.rs`)            | claim broken          | caught by |
//! |----------------------------------------------|-----------------------|-----------|
//! | defeat the `len >= 2` near-miss guard        | empty-drop vacuity (N4)| unit `single_conjunct_..._empty_drop` + invariant 1 |
//! | `has_discriminating_conjunct` → always-true  | (A)-binary refusal     | unit `bare_structural_..._autoimmune` + **invariant 5** |
//! | `corpus_witnesses_draft` → always-true       | route-to-human         | unit `near_miss_verdict_invariant_to_corpus_item_class` |
//! | `evaluate` → always `Spared` (defeat spare-clean) | autoimmunity ships | unit `rejects_the_naive_autoimmune_draft` + **invariant 4** |
//!
//! **Two invariants were green-but-TOOTHLESS in their first form and the sweep
//! caught it** (the value of the negative control made concrete):
//! - **Invariant 4** generated only random drafts and PASSED the gravest mutation
//!   (defeat spare-clean) — the random generator never produced an autoimmune draft
//!   that also promotes. FIX: the corpus-derived [`clean_binding_draft`] generator
//!   guarantees a clean-binding draft reaches the gate, so the autoimmune-promotion
//!   path is reliably exercised. NOW catches the mutation.
//! - **Invariant 5** gated its own assertion on `has_discriminating_conjunct` (the
//!   very function under test) — the mutation that made it always-true made the
//!   `if` never fire, so the invariant passed VACUOUSLY. FIX: an INDEPENDENT
//!   structural oracle [`is_structurally_bare`]. NOW catches the mutation.
//!
//! Re-run the sweep with the same worktree-isolated procedure when the gate logic
//! changes; the table is the registry of which test defends which break.

use antigen::learn::self_tolerance::{
    ToleranceVerdict, corpus_witnesses_draft, is_near_miss, promote_if_safe,
};
use antigen_fingerprint::{Constraint, Fingerprint, ItemKind};
use proptest::prelude::*;

// ── generators ──────────────────────────────────────────────────────────────

/// A small pool of constraint leaves spanning the (A)-binary partition: identity
/// anchors (`Item`, `ImplOfTrait`) and discriminating signals (`BodyCalls`,
/// `BodyContainsMacro`). Enough variety to build degenerate, precise, and
/// disjunction-bearing drafts.
fn any_constraint() -> impl Strategy<Value = Constraint> {
    prop_oneof![
        Just(Constraint::Item(ItemKind::Impl)),
        Just(Constraint::Item(ItemKind::Fn)),
        Just(Constraint::ImplOfTrait("Drop".into())),
        "[a-z]{3,6}".prop_map(Constraint::BodyCalls),
        "[a-z]{3,6}".prop_map(Constraint::BodyContainsMacro),
    ]
}

/// A draft of 0..=5 constraints — deliberately includes the empty and
/// single-conjunct edge cases (the len < 2 boundary the near-miss guard keys on).
fn any_draft() -> impl Strategy<Value = Fingerprint> {
    prop::collection::vec(any_constraint(), 0..=5)
        .prop_map(|constraints| Fingerprint { constraints })
}

/// A handful of real `syn::Item`s — the items a near-miss is hunted against. Mixes
/// impls (with bodies), structs (bodyless), and fns so the corpus spans the
/// item-class axis (ATK-047-3, the Undefined-flip guard).
fn corpus_items() -> Vec<syn::Item> {
    let src = r#"
        impl Drop for A { fn drop(&mut self) { let _ = flush().unwrap(); } }
        impl Drop for B { fn drop(&mut self) { let _ = flush().ok(); } }
        impl Drop for C { fn drop(&mut self) { let _ = work().expect("x"); } }
        struct Bare;
        fn free() { let _ = compute().unwrap(); }
    "#;
    syn::parse_file(src).expect("corpus parses").items
}

// ── invariant 1: the len >= 2 guard is total over ALL items (ATK-047-N4) ──────

proptest! {
    /// INVARIANT (ADR-047 §Mechanics 1, the N4 guard): a draft with FEWER THAN 2
    /// constraints has NO valid near-miss against ANY item — dropping its sole
    /// conjunct yields the empty `all_of` (vacuously `Match`), which must NEVER be
    /// counted as a near-miss. No hand-picked item can prove "for all"; the
    /// generator hunts a counterexample across the whole corpus.
    #[test]
    fn single_or_empty_conjunct_draft_is_never_a_near_miss(
        draft in prop::collection::vec(any_constraint(), 0..=1)
            .prop_map(|constraints| Fingerprint { constraints }),
    ) {
        let corpus = corpus_items();
        for item in &corpus {
            prop_assert!(
                !is_near_miss(&Fingerprint { constraints: draft.constraints.clone() }, item),
                "a <2-conjunct draft must never be a near-miss (ATK-047-N4 empty-drop vacuity): {:?}",
                draft.constraints
            );
        }
        // And the corpus-level rollup agrees: no near-miss anywhere.
        prop_assert!(!corpus_witnesses_draft(&draft, &corpus));
    }
}

// ── invariant 2: is_near_miss is TOTAL — never panics on any constraint vector ─

proptest! {
    /// INVARIANT (totality): the drop-one-conjunct machinery must terminate without
    /// panic for EVERY draft shape against EVERY item — including empty drafts,
    /// nested combinators, and drafts whose dropped-conjunct fingerprint goes
    /// `Undefined` on a bodyless item. A panic here would be antigen's own
    /// ⊥-collapse inside the safety gate.
    #[test]
    fn is_near_miss_is_total_no_panic(draft in any_draft()) {
        let corpus = corpus_items();
        for item in &corpus {
            // The assertion is simply that this returns (does not panic / hang).
            let _ = is_near_miss(&draft, item);
        }
        let _ = corpus_witnesses_draft(&draft, &corpus);
    }
}

// ── invariant 3: a near-miss is SPARED by the whole draft (definitional) ───────

proptest! {
    /// INVARIANT (ADR-047 §Decision): a near-miss item is, BY DEFINITION, one the
    /// whole draft does NOT bind (it fails exactly the one remaining conjunct). So
    /// `is_near_miss(draft, item) == true` IMPLIES `draft.matches(item) == false`.
    /// A "near-miss" the draft actually binds would be a contradiction in the
    /// primitive — the generator hunts any draft/item pair that violates it.
    #[test]
    fn a_near_miss_is_always_spared_by_the_whole_draft(draft in any_draft()) {
        let corpus = corpus_items();
        for item in &corpus {
            if is_near_miss(&draft, item) {
                prop_assert!(
                    !draft.matches(item),
                    "a near-miss MUST be spared by the whole draft (it fails the one \
                     remaining conjunct); draft={:?}",
                    draft.constraints
                );
            }
        }
    }
}

// ── invariant 4: the spare-clean SAFETY invariant on EVERY promoted token ──────
//
// NEGATIVE-CONTROL NOTE (the teeth that made this generator necessary): the first
// form of this invariant generated drafts from `any_draft()` (random constraint
// vectors) and PASSED the mutation that defeats the spare-clean scan (`evaluate`
// always `Spared`) — the random generator essentially never produced a draft that
// (a) clears (A)-binary + near-miss AND (b) binds a clean corpus item, so the
// invariant could not go red on the mutation it exists to defend (a green-but-
// toothless property). The fix is a CORPUS-DERIVED generator: a draft built from a
// clean item's OWN signals is guaranteed to bind that clean item, so the
// autoimmune-promotion path is reliably exercised. This invariant now has teeth
// against the gravest keystone mutation.

/// A draft built to BIND a chosen clean corpus item: it carries that item's own
/// item-kind + trait-anchor + a real body-signal it makes, PLUS a discriminating
/// signal (so it clears (A)-binary) and a second-conjunct (so it could be
/// near-miss-witnessed). Such a draft binds the clean item it was built from — the
/// gate MUST refuse to promote it (spare-clean), and if it ever promotes, that is
/// autoimmunity. This deliberately targets the spare-clean failure surface.
fn clean_binding_draft() -> impl Strategy<Value = Fingerprint> {
    // `flush().unwrap()` and `flush().ok()` items in `corpus_items()` both call
    // `flush`; a draft of {impl, Drop, body_calls("flush")} binds them. Add an
    // `unwrap` arm so it discriminates yet still binds the unwrap items.
    Just(Fingerprint {
        constraints: vec![
            Constraint::Item(ItemKind::Impl),
            Constraint::ImplOfTrait("Drop".into()),
            Constraint::BodyCalls("flush".into()),
            Constraint::BodyCalls("unwrap".into()),
        ],
    })
}

proptest! {
    /// THE LOAD-BEARING SAFETY INVARIANT (ADR-047 §Frontier; ADR-048): a token the
    /// gate mints (`Ok(PromotedDraft)`) NEVER binds any item in the clean corpus it
    /// was promoted against. This is the *whole point* of the gate, stated for ALL
    /// drafts and ALL corpora — not the three example fixtures. The generator mixes
    /// random drafts with CORPUS-DERIVED clean-binding drafts (above) so the
    /// autoimmune-promotion surface is reliably hit: if `promote_if_safe` ever
    /// returns `Ok` for a draft that binds a clean item, that is autoimmunity shipped
    /// through the gate — the keystone's defining failure.
    #[test]
    fn a_promoted_token_never_binds_a_clean_corpus_item(
        draft in prop_oneof![any_draft(), clean_binding_draft()],
    ) {
        let corpus = corpus_items();
        if let Ok(token) = promote_if_safe(draft.clone(), &corpus) {
            let fp = token.fingerprint();
            for (i, clean) in corpus.iter().enumerate() {
                prop_assert!(
                    !fp.matches(clean),
                    "a PROMOTED token bound clean-corpus item {i} — autoimmunity shipped \
                     through the gate (the keystone's defining failure); draft={:?}",
                    draft.constraints
                );
            }
        }
    }
}

/// A STRUCTURAL bare-structural check — `true` iff the draft carries ONLY identity
/// anchors (`Item` / `ImplOfTrait` / `NameMatches`), independent of the function
/// under test. Invariant 5 must NOT gate its own assertion on
/// `has_discriminating_conjunct` (the function being defended): a mutation that
/// makes that predicate always-true would make the `if !has_discriminating_conjunct`
/// guard never fire and the invariant pass vacuously (a self-referential teeth-gap
/// the negative-control sweep caught). This independent oracle closes that gap.
fn is_structurally_bare(draft: &Fingerprint) -> bool {
    !draft.constraints.is_empty()
        && draft.constraints.iter().all(|c| {
            matches!(
                c,
                Constraint::Item(_) | Constraint::ImplOfTrait(_) | Constraint::NameMatches(_)
            )
        })
}

proptest! {
    /// INVARIANT (ADR-047 §Mechanics 2, the (A)-binary): a draft carrying ONLY
    /// identity anchors (bare-structural) must NEVER mint a token against a non-empty
    /// corpus — it is refused before the spare-clean scan. The guard uses an
    /// INDEPENDENT structural oracle ([`is_structurally_bare`]), NOT
    /// `has_discriminating_conjunct` — so a mutation defeating the (A)-binary
    /// predicate makes this invariant go RED (it cannot pass vacuously through its
    /// own subject). This is the teeth the negative-control sweep demanded.
    #[test]
    fn bare_structural_draft_never_promotes(draft in any_draft()) {
        let corpus = corpus_items();
        if is_structurally_bare(&draft) {
            let verdict = promote_if_safe(draft.clone(), &corpus);
            prop_assert!(
                matches!(verdict, Err(ToleranceVerdict::BindsCleanItem { clean_index: None })),
                "a bare-structural draft must be refused by the (A)-binary check, got {:?}; \
                 draft={:?}",
                verdict,
                draft.constraints
            );
        }
    }
}
