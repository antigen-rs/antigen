//! ATK-047 SHAPE-FRAGILITY SEAM — the build-wave adversary's born-red spec for
//! the GATE-G primitives' producer-independence invariant (ADR-047 OQ2 / the
//! `(A)-BINARY-FOREVER` standing invariant; ADR-051 narrow/persist surfaces).
//!
//! # The break (PROVEN BY RUN, `R:/antigen-atk-scratch/src/{shape,bypass}_probe.rs`)
//!
//! The `(A)-binary` safety refusal (`has_discriminating_conjunct`) and the
//! near-miss primitive (`is_near_miss`) operate on the draft's **top-level**
//! `constraints`. Two producers emit two DIFFERENT top-level shapes for the same
//! semantic fingerprint:
//!
//! | producer | top-level shape of `all_of([item=impl, impl_of_trait("Drop")])` |
//! |---|---|
//! | `anti_unify` (the real generator) | **FLAT** — `[Item, ImplOfTrait]` (len 2) |
//! | `Fingerprint::parse("all_of([..])")` | **WRAPPED** — `[AllOf([Item, ImplOfTrait])]` (len 1) |
//!
//! Because `is_discriminating(Constraint::AllOf(_)) == true`, a WRAPPED
//! bare-structural draft makes `has_discriminating_conjunct` return `true` — so the
//! `(A)-binary` SAFETY refusal **does not fire**. The same-semantics FLAT draft is
//! correctly refused. The safety guarantee leaks at the producer seam — antigen's
//! own `ParallelStateTrackersDiverge` at the keystone gate.
//!
//! # Status: GREEN regression guards (the seam is CLOSED — ADR-047 Amendments 1+2)
//!
//! These were born-RED. They are now GREEN: the GATE-G primitives
//! (`has_discriminating_conjunct` / `is_near_miss`) read producer-NORMALIZED top-level
//! conjuncts via `self_tolerance::normalized_top_level`, which **recursively flattens
//! every redundant `AllOf`** (`all_of` is associative + single-child is identity, so
//! the flatten is semantics-preserving and matches the shipped `matcher.rs` algebra).
//! The verdict is now PRODUCER-INDEPENDENT regardless of nesting depth:
//! - a single/double-wrapped `[AllOf([..])]` reads the same as the flat shape
//!   (Amendment 1 + the recursive extension);
//! - a NESTED `[Item, Drop, AllOf([flush, drain, unwrap])]` flattens to the flat
//!   `[Item, Drop, flush, drain, unwrap]` — a single top-level near-miss drop can no
//!   longer strip many discriminators at once (Amendment 2 — the nested-vacuity half);
//! - and the remainder-discriminates rule (Amendment 2 — the flat single-discriminator
//!   half, Hole-I) refuses any drop whose remainder collapses to bare-structural.
//!
//! The LIVE risk these pinned — ADR-051's `narrow()` / `PersistedSpecimen` re-mint
//! re-PARSING a user-edited fingerprint into ARBITRARY nesting — is closed at the
//! primitive, so unit #8 inherits a producer-independent gate.
//!
//! The one residual (genuinely deferred, ADR-047 OQ2, NOT a hole): an `AllOf` *inside
//! an `AnyOf` arm* is semantically necessary (not redundant) — a top-level near-miss
//! drop of the whole `AnyOf` is an under-bind → route-to-human (SAFE, never a
//! fabricated promote). That arm-internal recall-drop is a recall charter, not a
//! safety concern; it is NOT pinned here.

use antigen::learn::self_tolerance::{
    ToleranceVerdict, has_discriminating_conjunct, is_near_miss, promote_if_safe,
};
use antigen_fingerprint::{Constraint, Fingerprint, ItemKind};

/// The bare-structural draft, FLAT — the shape `anti_unify` emits on a
/// body-signal-less collapse. `[Item(Impl), ImplOfTrait("Drop")]`.
fn flat_bare_structural() -> Fingerprint {
    Fingerprint {
        constraints: vec![
            Constraint::Item(ItemKind::Impl),
            Constraint::ImplOfTrait("Drop".into()),
        ],
    }
}

/// The IDENTICAL semantics, WRAPPED — the shape `Fingerprint::parse` emits, and the
/// shape a `narrow()`/persist reconstruction via the `all_of(...)` surface would
/// produce. `[AllOf([Item(Impl), ImplOfTrait("Drop")])]`.
fn wrapped_bare_structural() -> Fingerprint {
    Fingerprint::parse(r#"all_of([item = impl, impl_of_trait("Drop")])"#).unwrap()
}

/// A clean `Drop` sibling the bare-structural draft over-binds (the autoimmune
/// target the `(A)-binary` refusal exists to protect).
fn clean_drop_sibling() -> syn::Item {
    syn::parse_str("impl Drop for Clean { fn drop(&mut self) { let _ = flush().ok(); } }")
        .expect("clean sibling parses")
}

/// SANITY (passes today): the FLAT bare-structural draft IS refused by `(A)-binary`.
/// Establishes the primitive works on the shape the production generator emits — so
/// the WRAPPED failure below is a producer-seam bug, not a broken predicate.
#[test]
fn flat_bare_structural_is_refused_by_a_binary() {
    assert!(
        !has_discriminating_conjunct(&flat_bare_structural()),
        "a FLAT bare-structural draft carries no discriminating conjunct"
    );
    assert_eq!(
        promote_if_safe(
            flat_bare_structural(),
            std::slice::from_ref(&clean_drop_sibling())
        ),
        Err(ToleranceVerdict::BindsCleanItem { clean_index: None }),
        "FLAT bare-structural → (A)-binary SAFETY refusal"
    );
}

/// GREEN (ADR-047 Amendment 1) — the producer-independence invariant. A WRAPPED
/// bare-structural draft (same semantics as the flat one) MUST ALSO be refused by
/// `(A)-binary`. It now is: `has_discriminating_conjunct` reads producer-normalized
/// top-level conjuncts (a sole top-level `AllOf` is unwrapped), so the outer wrapper
/// no longer reads as a discriminating conjunct and the refusal fires correctly.
// UN-IGNORED (ADR-047 Amendment 1, pathmaker): `has_discriminating_conjunct` /
// `is_near_miss` now normalize a single top-level `AllOf` wrapper, so the verdict is
// producer-independent for the single-wrapper case — a GREEN regression guard now.
// (The NESTED case further below stays born-red — ADR-047 OQ2, genuinely deferred.)
#[test]
fn wrapped_bare_structural_must_also_be_refused_by_a_binary() {
    assert!(
        !has_discriminating_conjunct(&wrapped_bare_structural()),
        "a WRAPPED bare-structural draft must ALSO carry no discriminating conjunct \
         — the (A)-binary verdict must not depend on whether the draft is wrapped"
    );
    assert_eq!(
        promote_if_safe(
            wrapped_bare_structural(),
            std::slice::from_ref(&clean_drop_sibling())
        ),
        Err(ToleranceVerdict::BindsCleanItem { clean_index: None }),
        "WRAPPED bare-structural MUST get the SAME (A)-binary refusal as the flat one \
         (not a NotCorpusWitnessable that only-accidentally-happens-to-also-be-an-Err)"
    );
}

/// GREEN (ADR-047 Amendment 1) — the same invariant stated as the two verdicts must
/// AGREE. The gate's verdict for a draft is a function of the draft's SEMANTICS,
/// never of which producer (`parse` vs `anti_unify`) built it. With the single-
/// wrapper normalization, both now yield `BindsCleanItem{None}` (the (A)-binary
/// refusal), where pre-fix they disagreed (flat → `BindsCleanItem`, wrapped →
/// `NotCorpusWitnessable`).
// UN-IGNORED (ADR-047 Amendment 1, pathmaker): the single-wrapper normalization
// makes the verdict producer-independent — now a GREEN regression guard.
#[test]
fn gate_verdict_is_producer_independent_for_identical_semantics() {
    let clean = clean_drop_sibling();
    let flat_verdict = promote_if_safe(flat_bare_structural(), std::slice::from_ref(&clean));
    let wrapped_verdict = promote_if_safe(wrapped_bare_structural(), std::slice::from_ref(&clean));
    assert_eq!(
        flat_verdict, wrapped_verdict,
        "the gate verdict must depend on the draft's SEMANTICS, not its top-level \
         AllOf-wrapping (parse vs anti_unify) — divergence here is antigen's own \
         ParallelStateTrackersDiverge at the keystone gate"
    );
}

// ============================================================================
// ATK-047 NESTED-VACUITY — the SHARP face of the shape-fragility bug. The
// near-miss primitive drops one TOP-LEVEL conjunct; a nested `all_of` conjunct
// lets ONE top-level drop remove MANY discriminating constraints at once,
// collapsing the draft to its bare structural anchors — so a BARE family member
// (none of the body signals) becomes a spurious "near-miss". This REOPENS the
// trivial-skeleton vacuity (ATK-047-1) that near-miss non-vacuity exists to
// forbid. PROVEN BY RUN: `R:/antigen-atk-scratch/src/nested_anyof_probe2.rs`.
//
// Severity note: this is WORSE than the (A)-binary wrapper bypass, because the
// failure is a silent WRONG *promote* (Ok(PromotedDraft)) — the gate certifies a
// draft whose only corpus witness is a bare structural family member it
// over-near-missed. ADR-047 OQ2 deferred nested as "a future generator's
// concern"; this shows the deferral is unsafe unless the gate DETECTS nesting it
// cannot recurse and routes-to-human, rather than silently mis-near-missing.
// ============================================================================

fn bc(n: &str) -> Constraint {
    Constraint::BodyCalls(n.into())
}

/// The nested draft `[impl, Drop, all_of([flush, drain, unwrap])]` — the inner
/// `all_of` carries the discriminating signal as ONE top-level conjunct.
fn nested_discriminator_draft() -> Fingerprint {
    Fingerprint {
        constraints: vec![
            Constraint::Item(ItemKind::Impl),
            Constraint::ImplOfTrait("Drop".into()),
            Constraint::AllOf(vec![bc("flush"), bc("drain"), bc("unwrap")]),
        ],
    }
}

/// The FLAT-equivalent: `[impl, Drop, flush, drain, unwrap]` — identical semantics,
/// discriminators at the top level.
fn flat_discriminator_draft() -> Fingerprint {
    Fingerprint {
        constraints: vec![
            Constraint::Item(ItemKind::Impl),
            Constraint::ImplOfTrait("Drop".into()),
            bc("flush"),
            bc("drain"),
            bc("unwrap"),
        ],
    }
}

/// A BARE `Drop` impl with NONE of the draft's body signals — three signals from
/// binding. It must NOT be a near-miss (it is not "one constraint away"); treating
/// it as one is the trivial-skeleton vacuity.
fn bare_drop_impl() -> syn::Item {
    syn::parse_str("impl Drop for Far { fn drop(&mut self) { log(); } }").expect("parses")
}

/// GREEN (ADR-047 Amendment 2) — a bare family member must NOT be a near-miss for a
/// NESTED draft. The recursive `normalized_top_level` flatten splices the inner
/// `all_of([flush, drain, unwrap])` into the top level (`all_of` associativity), so
/// the nested draft reads identically to the FLAT one — a single top-level drop can
/// no longer strip many discriminators at once, and the remainder-discriminates rule
/// (Amd2 Hole-I) refuses the bare-structural collapse regardless. Pre-Amd2 this
/// SILENTLY WRONG-near-missed (the live nested-vacuity hole the notary-pass found).
#[test]
fn nested_draft_does_not_spuriously_near_miss_a_bare_family_member() {
    let bare = bare_drop_impl();
    // The FLAT draft correctly rejects the bare member as a near-miss (3 signals away).
    assert!(
        !is_near_miss(&flat_discriminator_draft(), &bare),
        "sanity: a bare Drop impl is 3 signals from the FLAT draft — not a near-miss"
    );
    // The NESTED (semantically-identical) draft must give the SAME answer.
    assert!(
        !is_near_miss(&nested_discriminator_draft(), &bare),
        "a bare Drop impl must NOT be a near-miss for the nested draft either — a \
         single top-level drop of the inner all_of collapses to bare-structural \
         [impl, Drop], reopening the trivial-skeleton vacuity ATK-047-1"
    );
}

/// GREEN (ADR-047 Amendment 2) — the gate must NOT PROMOTE the nested draft against a
/// corpus whose only item is a bare family member. With the recursive flatten + the
/// remainder-discriminates rule, the nested draft now routes-to-human
/// (`NotCorpusWitnessable`) exactly like its FLAT equivalent — no silent wrong-promote
/// of a draft the corpus never genuinely exercised.
#[test]
fn nested_draft_does_not_promote_against_a_bare_family_member_only_corpus() {
    let corpus = [bare_drop_impl()];
    let flat_verdict = promote_if_safe(flat_discriminator_draft(), &corpus);
    let nested_verdict = promote_if_safe(nested_discriminator_draft(), &corpus);
    // The FLAT draft routes-to-human (the bare member doesn't exercise it).
    assert_eq!(
        flat_verdict,
        Err(ToleranceVerdict::NotCorpusWitnessable),
        "sanity: FLAT draft is not corpus-witnessed by a bare family member"
    );
    // The NESTED draft must give the SAME verdict — not a spurious promote.
    assert_eq!(
        nested_verdict, flat_verdict,
        "the nested draft must route-to-human like its flat equivalent — promoting \
         it certifies a draft whose only 'witness' is the bare structural family \
         (the trivial-skeleton vacuity, reopened via nesting)"
    );
}

// ============================================================================
// RETRACTED: the "single-discriminator trivial-skeleton vacuity" I first filed
// here was a FALSE POSITIVE — the ratified design EXPLICITLY rules it intended.
// ADR-047 §Decision (drafts/adr-047-gate-g-soundness.md:53): "a witness must be
// one constraint from binding *the actual draft*, which for `{fn, body_calls(
// "transmute")}` means 'a fn that would bind if it called transmute' — a GENUINE
// discrimination ('B spares non-transmute fns'), not arbitrary unrelated code the
// draft's signal never came near." So a single-discriminator draft `[impl, Drop,
// foo]` near-missing (and promoting against) a bare `Drop` impl that lacks `foo`
// is the gate WORKING: B genuinely spares non-foo Drop impls. Default-to-refuted
// applied to my OWN claim caught it (camp 7fea15c2 → retraction). The two tests
// asserting route-to-human were WRONG and are removed.
//
// What SURVIVES is the producer-independence finding above (the nested case): a
// 3-signal cluster bundled as a nested AllOf gives a DIFFERENT verdict from the
// same 3 signals flat — that divergence is real (same semantics, two verdicts),
// independent of whether the single-discriminator promote is intended.
//
// FORWARD-POINTER (ADR-047 Amendment 2, the live ruling): the retraction above
// deferred to §Decision:53's "single-discriminator promote is intended" prose —
// but Amendment 2 later REFINED that. The remainder-discriminates guard
// (`is_near_miss`, self_tolerance.rs ~352) treats a draft whose sole discriminator
// drops to a bare-structural skeleton as a REAL near-miss hole (the remainder must
// still discriminate). So §Decision:53 is the pre-Amd2 reasoning; do NOT read this
// retraction as the final word on the single-discriminator question — Amd2 is.
// ============================================================================

// ============================================================================
// ATK-047 DOUBLE-WRAP — the RECURSION guard for Fix B (ADR-047 Amendment 2). A
// single-level flatten (Fix A — unwrap ONE outer AllOf) is INSUFFICIENT: a
// `all_of([all_of([..])])` draft re-nests one level deeper and re-opens the
// nested-vacuity. Fix B is RECURSIVE (`flatten_all_of_into` splices every nested
// AllOf, any depth). This guard proves the recursion terminates correctly at
// depth-2 — it would go RED under a non-recursive single-flatten regression.
// REACHABILITY (captain's ruling): NOT generator-only — ADR-051 narrow()/persist
// re-PARSES a user-edited fingerprint (`parse_all_of` does not flatten), so a
// hand-written `all_of([all_of([..])])` reaches the gate. Live by unit #8.
// PROVEN BY RUN: `R:/antigen-atk-scratch/src/doublewrap_probe.rs`.
// ============================================================================

/// A DOUBLE-wrapped draft: the discriminators nested TWO `all_of` levels deep —
/// `[impl, Drop, all_of([all_of([flush, drain, unwrap])])]`. Semantically identical
/// to the flat `[impl, Drop, flush, drain, unwrap]`; the double wrap is what a
/// `narrow()`/persist re-parse of a user-edited fingerprint can reconstruct.
fn double_wrapped_discriminator_draft() -> Fingerprint {
    Fingerprint {
        constraints: vec![
            Constraint::Item(ItemKind::Impl),
            Constraint::ImplOfTrait("Drop".into()),
            Constraint::AllOf(vec![Constraint::AllOf(vec![
                bc("flush"),
                bc("drain"),
                bc("unwrap"),
            ])]),
        ],
    }
}

/// GREEN (Fix B recursion guard) — the DOUBLE-wrapped draft must give the SAME
/// verdict as its flat equivalent: a bare family member is NOT a near-miss, and the
/// gate routes-to-human (not a spurious promote). A single-level flatten (Fix A)
/// would leave `[impl, Drop, all_of([flush,drain,unwrap])]` (still nested one level)
/// and re-open the vacuity — so this is the test that distinguishes recursive Fix B
/// from insufficient Fix A. If this ever REDs, a flatten regressed to non-recursive.
#[test]
fn double_wrapped_draft_is_producer_independent_via_recursive_flatten() {
    let bare = bare_drop_impl();
    let corpus = [bare_drop_impl()];
    // (a) the double-wrap is NOT a near-miss for a bare family member (same as flat).
    assert!(
        !is_near_miss(&double_wrapped_discriminator_draft(), &bare),
        "a bare Drop impl must NOT be a near-miss for the DOUBLE-wrapped draft — the \
         recursive flatten splices both AllOf levels so it reads as the flat \
         [impl, Drop, flush, drain, unwrap], 3 signals from the bare member"
    );
    // (b) the gate verdict is identical to the flat draft's (producer-independent at
    //     nesting depth 2 — the recursion-terminates proof).
    assert_eq!(
        promote_if_safe(double_wrapped_discriminator_draft(), &corpus),
        promote_if_safe(flat_discriminator_draft(), &corpus),
        "the DOUBLE-wrapped draft must route-to-human exactly like its flat \
         equivalent — Fix B's flatten is recursive (any depth), so a deeper wrap \
         cannot re-open the nested vacuity (single-level Fix A would fail here)"
    );
}
