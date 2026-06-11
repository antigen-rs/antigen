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
//! # Why this is born-red (and `#[ignore]`'d), not a passing test
//!
//! Today the PRODUCTION `propose` path is FLAT end-to-end (`anti_unify` →
//! `is_degenerate` → `promote_if_safe`) and the serde-TREE round-trip preserves
//! flat — so no LIVE bypass exists in v0.5-as-built, and the in-module unit tests
//! pass by using a `flat()` test helper that unwraps the wrapper. **That helper
//! papers the seam at the test level only.** The PRIMITIVE remains shape-fragile.
//!
//! The LIVE risk is ADR-051's `narrow()` + `PersistedSpecimen` re-mint (NOT YET
//! BUILT): if either reconstructs a fingerprint through the `all_of(...)` SURFACE
//! (a `parse` call, or a hand-built `Constraint::AllOf`) rather than the serde
//! tree, the re-gate through `promote_if_safe` silently passes a bare-structural
//! draft. This file pins the invariant BEFORE 051 is built.
//!
//! # The fix this spec defines (un-ignore when either lands)
//!
//! Make the gate producer-INDEPENDENT: normalize a single top-level `AllOf` wrapper
//! inside `has_discriminating_conjunct` / `is_near_miss` (flatten before reading),
//! so the safety verdict cannot depend on which producer built the draft. (The
//! alternative — a locked "no wrapped draft ever reaches `promote_if_safe`"
//! invariant enforced on the narrow/persist path — is weaker: it leaves the
//! primitive a loaded gun for the next caller.) Once the primitive normalizes,
//! delete the `#[ignore]`; these go green and STAY the regression guard.

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

/// BORN-RED — a bare family member must NOT be a near-miss for a NESTED draft. Today
/// `is_near_miss(nested, bare_drop) == true`: dropping the single inner-`all_of`
/// top-level conjunct collapses the draft to `[impl, Drop]`, which binds every Drop
/// impl — so a bare `Drop` impl spuriously "near-misses". The FLAT equivalent
/// correctly returns `false`. Un-ignore when the near-miss primitive recurses into
/// (or refuses) nested conjuncts.
#[test]
#[ignore = "born-red: nested all_of reopens trivial-skeleton vacuity (ATK-047-1) — a bare family member spuriously near-misses; un-ignore when is_near_miss recurses-or-refuses nested conjuncts (ADR-047 OQ2)"]
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

/// BORN-RED — the gate must NOT PROMOTE the nested draft against a corpus whose only
/// item is a bare family member. Today it does (`Ok(PromotedDraft)`) while the FLAT
/// equivalent correctly routes-to-human (`NotCorpusWitnessable`). A promote here
/// certifies a draft the corpus never genuinely exercised.
#[test]
#[ignore = "born-red: nested draft promotes on a bare-family-member-only corpus (vacuity); un-ignore with the nested-near-miss fix"]
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
