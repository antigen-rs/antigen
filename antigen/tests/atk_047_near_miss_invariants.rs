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
//! | defeat the `len >= 2` near-miss guard        | empty-drop vacuity (N4)| **subsumed by Amd2** (see note below) — table row kept honest |
//! | `has_discriminating_conjunct` → always-true  | (A)-binary refusal     | unit `bare_structural_..._autoimmune` + **invariant 5** |
//! | `corpus_witnesses_draft` → always-true       | route-to-human         | unit `near_miss_verdict_invariant_to_corpus_item_class` |
//! | `evaluate` → always `Spared` (defeat spare-clean) | autoimmunity ships | unit `rejects_the_naive_autoimmune_draft` + **invariant 4** |
//!
//! **The M1 row (`len >= 2`) is subsumed — and that is itself a finding antigen
//! catches.** When the original sweep ran (2026-06-11), defeating `len >= 2` went
//! RED. It no longer does: ADR-047 **Amendment 2**'s remainder-discriminates guard
//! (`is_near_miss`, `self_tolerance.rs` ~352) closes the empty-drop vacuity (N4)
//! *independently* — a single-conjunct draft drops to an EMPTY remainder, and
//! `has_discriminating_conjunct(empty) == false` rejects it. So Amd2 took over
//! M1's sole correctness job; deleting `len >= 2` now leaves the whole suite green.
//! `len >= 2` is **retained** as a fast-path short-circuit (an empty/1-conjunct
//! draft never reaches the `.any` drop-loop) and as defense-in-depth — NOT removed,
//! because removing it edits the certified GATE-G core (a v0.6 design call). The row
//! is kept and annotated rather than deleted so this teeth-registry stops asserting
//! a mutation→catcher mapping that no longer holds — a stale teeth-table is itself
//! the comment-asserted-property drift-class antigen exists to catch.
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

/// A constraint spanning the (A)-binary partition AND the COMBINATOR depth axis:
/// identity anchors (`Item`, `ImplOfTrait`), discriminating leaves (`BodyCalls`,
/// `BodyContainsMacro`), AND recursively-nested `AllOf`/`AnyOf` combinators.
///
/// The leaf-only generator (a batch-3 catch, 2026-06-11) left
/// invariant 2's "including nested combinators" claim COMMENT-ASSERTED but
/// UNTESTED — and the nested-vacuity hole (a nested `AllOf` conjunct lets ONE
/// top-level drop strip MANY discriminators) lives precisely in the combinator
/// depth a leaf-only generator never reaches. This recursive, depth-limited arm
/// makes a future nested-generator regression TRIP the property, not slip past it.
fn any_constraint() -> impl Strategy<Value = Constraint> {
    let leaf = prop_oneof![
        Just(Constraint::Item(ItemKind::Impl)),
        Just(Constraint::Item(ItemKind::Fn)),
        Just(Constraint::ImplOfTrait("Drop".into())),
        "[a-z]{3,6}".prop_map(Constraint::BodyCalls),
        "[a-z]{3,6}".prop_map(Constraint::BodyContainsMacro),
    ];
    // Depth-limited recursion: up to 2 levels of nesting, 1..=3 children per
    // combinator, ≤8 total nodes — enough to exercise nested `AllOf`/`AnyOf`/`Not`
    // (incl. the double-wrap `AllOf(AllOf(..))` the recursive-canonical-form must
    // flatten) without blowing up the search space. ALL THREE combinators the
    // `Constraint` enum carries are generated — `Not(Box<Self>)` too, so a nested
    // negation can't slip a combinator-shape past the property (the
    // completeness catch: the generator must span every combinator the gate reads).
    leaf.prop_recursive(2, 8, 3, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 1..=3).prop_map(Constraint::AllOf),
            prop::collection::vec(inner.clone(), 1..=3).prop_map(Constraint::AnyOf),
            inner.prop_map(|c| Constraint::Not(Box::new(c))),
        ]
    })
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

/// A LEAF-only constraint (no combinators) — used where the claim is about a
/// *genuinely atomic* conjunct, distinct from a combinator-wrapped one. (The N4
/// guard is about a draft that is single-conjunct AFTER producer-normalization; a
/// `[AllOf([a, b])]` is syntactically one conjunct but normalizes to TWO, so it is
/// NOT an N4 single-conjunct draft — the recursive generator surfaced exactly this
/// distinction, a real subtlety worth pinning, not a gate bug.)
fn any_leaf() -> impl Strategy<Value = Constraint> {
    prop_oneof![
        Just(Constraint::Item(ItemKind::Impl)),
        Just(Constraint::Item(ItemKind::Fn)),
        Just(Constraint::ImplOfTrait("Drop".into())),
        "[a-z]{3,6}".prop_map(Constraint::BodyCalls),
        "[a-z]{3,6}".prop_map(Constraint::BodyContainsMacro),
    ]
}

// ── invariant 1: the len >= 2 guard is total over ALL items (ATK-047-N4) ──────

proptest! {
    /// INVARIANT (ADR-047 §Mechanics 1, the N4 guard): a draft of ≤1 *normalized*
    /// conjunct has NO valid near-miss against ANY item — dropping its sole conjunct
    /// yields the empty `all_of` (vacuously `Match`), which must NEVER be counted as a
    /// near-miss. The generator is LEAF-ONLY (`any_leaf`): a `[AllOf([a,b])]` is
    /// syntactically single-conjunct but normalizes to TWO, so it is NOT an N4 case
    /// (the gate's `normalized_top_level` unwraps it first — the recursive generator
    /// surfaced this distinction). No hand-picked item can prove "for all"; the
    /// generator hunts a counterexample across the whole corpus.
    #[test]
    fn single_or_empty_conjunct_draft_is_never_a_near_miss(
        draft in prop::collection::vec(any_leaf(), 0..=1)
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

// ── invariant 6: a near-miss's REMAINDER must not be bare-structural ───────────
//
// REGISTRY #1 — CLOSED 2026-06-11 (ADR-047 Amendment 2, `is_near_miss` now requires
// `has_discriminating_conjunct(&remainder)` on the dropped-conjunct remainder;
// `self_tolerance.rs:352`). This invariant is the UNIVERSALIZATION of the
// confirmed-LIVE ATK-047-1 finding (`atk_047_shape_fragility_seam.rs`
// gave TWO specific examples; this proves the fix for the CLASS — all single-
// discriminator drafts vs all non-binding items). Born RED, fixed, now a PERMANENT
// regression guard (the ATK lifecycle: never deleted).
//
// THE HOLE IT GUARDS: before Amd2, `is_near_miss` counted an item a near-miss if
// dropping ANY ONE conjunct made the draft bind it — including dropping the SOLE
// discriminator, whose remainder `[Item, ImplOfTrait]` is bare-structural and
// matches every family member. That is "near" only by collapsing to the structural
// skeleton — the trivial-skeleton vacuity (ATK-047-1). A near-miss witnessed that
// way is NOT a real in-family discrimination. Amd2's remainder-must-discriminate
// rule closes it; this invariant is the standing proof it stays closed.

/// A draft whose ENTIRE discriminating signal lives in ONE conjunct (a single
/// `body_calls`), on a `Drop` impl. Its only near-miss route is to drop that sole
/// discriminator — leaving the bare-structural `[impl, Drop]` remainder, which Amd2
/// now refuses to count as a near-miss.
fn single_discriminator_draft() -> Fingerprint {
    Fingerprint {
        constraints: vec![
            Constraint::Item(ItemKind::Impl),
            Constraint::ImplOfTrait("Drop".into()),
            Constraint::BodyCalls("a_signal_no_clean_item_makes".into()),
        ],
    }
}

proptest! {
    /// INVARIANT (REGISTRY #1, CLOSED by ADR-047 Amd2 — the standing guard): a
    /// single-discriminator draft must NOT be near-miss-witnessed by an item that
    /// shares NONE of its discriminating signal — such an item is "near" only via the
    /// bare-structural collapse (ATK-047-1). For ALL corpus items the
    /// `single_discriminator_draft` does not bind, `is_near_miss` must be FALSE (the
    /// only near-miss route drops the sole discriminator, leaving a bare-structural
    /// remainder Amd2's `has_discriminating_conjunct(&remainder)` check now rejects).
    /// If this ever goes RED, the remainder-must-discriminate rule regressed.
    #[test]
    fn single_discriminator_near_miss_must_not_be_bare_structural_collapse(
        draft in Just(single_discriminator_draft()),
    ) {
        let corpus = corpus_items();
        for item in &corpus {
            // The draft binds an item ONLY if that item makes the rare signal — none
            // of the corpus items do, so the draft binds none of them. Every one is a
            // non-binding item whose ONLY near-miss route is the bare-structural drop.
            if !draft.matches(item) {
                prop_assert!(
                    !is_near_miss(&draft, item),
                    "an item sharing NONE of the single discriminator must not be a \
                     near-miss — it is 'near' only by collapsing the draft to its bare \
                     structural skeleton (the trivial-skeleton vacuity ATK-047-1); \
                     item-binds-draft={}",
                    draft.matches(item)
                );
            }
        }
    }
}
