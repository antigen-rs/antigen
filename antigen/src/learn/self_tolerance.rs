//! B — the self-tolerance / spare-clean gate (v0.4, ADR-045: the one safety-tangle).
//!
//! The SELECTOR half of the affinity-maturation arm — antigen's **thymus**. A
//! proposed (drafted) fingerprint is *promotable* only if it **spares every item
//! in a clean corpus**: a draft that matches clean code would, once promoted,
//! flag that clean code — antigen's own **autoimmunity**. This gate performs
//! negative selection: it rejects any draft that binds a known-clean sibling.
//!
//! # Why this is the safety floor
//!
//! The PROPOSE generator (C) anti-unifies a cluster of structurally-similar
//! defective sites into a draft. The naive generalization (drop the differing
//! leaves) over-generalizes — e.g. a `panic-in-Drop` cluster collapses to "any
//! `Drop` impl", which matches a CLEAN `Drop` sibling. The generator's own output
//! IS the false positive. **C must never promote a draft this gate rejects.**
//! Even a smarter disjunction draft must pass B: B is required regardless of how
//! PROPOSE generalizes (ADR-045 — the C ══ B co-ship; the captain's highest-stakes
//! line: never ship C without B green).
//!
//! # Claim-scope (ADR-044)
//!
//! **What B proves:** the draft does not match any item in *this* clean corpus
//! (a decidable, bounded fact — `Fingerprint::matches` over a finite corpus).
//! **What B does NOT prove:** that the draft is correct, or that it spares ALL
//! clean code everywhere (that is the open-world generalization problem — a
//! corpus-bounded gate, like a knockoffs/FDR control, not a total guarantee). B
//! is a *necessary* safety gate, not a *sufficient* correctness proof. The
//! corpus is the ratifier's responsibility: a richer clean corpus is a stronger
//! gate. B never asserts the draft is a real named class — that stays with the
//! human/incident LABEL.

use antigen_fingerprint::{Constraint, Fingerprint};

use crate::finding::Provenance;

/// The verdict of the spare-clean gate for one draft against one clean corpus.
///
/// Three-valued (ADR-047): the gate decides on two axes — SAFETY (does the draft
/// over-bind clean code?) and GENERALIZATION-QUALITY (can B certify the
/// generalization is corpus-exercised?). The three verdicts name the three
/// distinct corpus-relative promotion-decision provenances.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToleranceVerdict {
    /// The draft spares every clean-corpus item — SAFE to promote. (Used as the
    /// pure-predicate answer of [`evaluate`]; the promotion authority replaces it
    /// with a [`PromotedDraft`] on the `Ok` arm.)
    Spared,
    /// **Autoimmune refusal (SAFETY).** The draft matched at least one clean-corpus
    /// item (promoting it would flag clean code), OR it is **bare-structural
    /// over-general** — it fails the (A)-binary discriminating-conjunct check and
    /// binds the whole structural family. Carries the index of the first clean item
    /// it bound (for the binds-clean case), for a precise diagnostic.
    BindsCleanItem {
        /// The position (in the supplied corpus slice) of the first clean item the
        /// draft matched. `None` when the refusal is the (A)-binary bare-structural
        /// case (no clean item was bound — the draft was refused before the
        /// spare-clean scan because it carries no discriminating signal).
        clean_index: Option<usize>,
    },
    /// **Route-to-human (GENERALIZATION-QUALITY).** The draft is *safe* (it spares
    /// the corpus and carries a discriminating signal) but the corpus contains **no
    /// near-miss** — no item one constraint from binding it — so B cannot certify
    /// the draft's generalization is corpus-exercised. First-class, NOT an error:
    /// "this draft is safe but I can't certify it generalizes — ratify it by hand"
    /// (the handoff into the ratification-interface, ADR-051). B never fakes a
    /// generalization-verdict it cannot make (sub-clause-F-honest, ADR-005).
    NotCorpusWitnessable,
}

impl ToleranceVerdict {
    /// `true` iff the draft is safe to promote (it spared the whole clean corpus).
    #[must_use]
    pub const fn is_safe(&self) -> bool {
        matches!(self, Self::Spared)
    }
}

/// A draft that has passed B's gate against a real clean corpus, **carrying its
/// score** — the ONLY assertable generalization (ADR-048, the capability-token).
///
/// Constructed solely by [`promote_if_safe`] (and, transitively,
/// [`propose`](crate::learn::propose::propose)); **there is no public constructor,
/// no `From<Fingerprint>`, no `Default`, and no `Deserialize`** (the serde-forgery
/// guard, ADR-048 §5 — a `#[derive(Deserialize)]` would forge a token from
/// hand-written JSON, never having passed the gate). Possession of a
/// `PromotedDraft` is the **structural proof** that ALL THREE of ADR-047's gate
/// checks held — **(A)-binary** (carries a discriminating signal),
/// **near-miss non-vacuity** (the corpus exercised it), and **spare-clean** (binds
/// no clean item) — AND that the output is SCORED (ADR-049: the `tier` is the
/// existing [`Provenance`] ordinal, gate-assigned). One token, three invariants:
/// B-gate-ran (ADR-048), gate-checks-held (ADR-047), output-scored (ADR-049).
///
/// `Serialize` IS derived (emitting the token's fingerprint is safe); it is
/// *construction-from-untrusted-bytes* that forges, so only `Deserialize` is
/// withheld. To persist a token, serialize it, store the bare [`Fingerprint`], and
/// re-enter [`promote_if_safe`] on load to re-acquire the token (the
/// *deserialize-downgrades* member of the shared gate-bypass class — same shape as
/// [`PromotedDraft::into_fingerprint`]).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct PromotedDraft {
    /// PRIVATE — no bypass construction. The gated fingerprint.
    fingerprint: Fingerprint,
    /// The score (ADR-049), gate-assigned (ADR-050's routing computes it at
    /// promotion). The EXISTING [`Provenance`] ordinal — not a new type.
    tier: Provenance,
}

impl PromotedDraft {
    /// Read-only access to the gated fingerprint (for `.matches()` / serialize).
    /// Never hands out an owned un-tokened `Fingerprint` that could be
    /// re-promoted-by-forgery; use [`into_fingerprint`](Self::into_fingerprint)
    /// for the explicit one-way downgrade.
    #[must_use]
    pub const fn fingerprint(&self) -> &Fingerprint {
        &self.fingerprint
    }

    /// The gate-assigned score tier (ADR-049/050). The auto-accept policy-lever
    /// reads THIS — a gate-assigned tier, never a C-self-assessed one.
    #[must_use]
    pub const fn tier(&self) -> Provenance {
        self.tier
    }

    /// The owned one-way **capability downgrade** (ADR-048 §4, the shared
    /// gate-bypass class): extract the plain [`Fingerprint`], which is assertable
    /// as nothing. Re-promotion requires re-routing through [`promote_if_safe`].
    /// Safe because the downgrade is explicit and one-way — the same class as
    /// `narrow()` (ADR-051) and deserialize-downgrades (ADR-048 §5).
    #[must_use]
    pub fn into_fingerprint(self) -> Fingerprint {
        self.fingerprint
    }
}

impl AsRef<Fingerprint> for PromotedDraft {
    fn as_ref(&self) -> &Fingerprint {
        &self.fingerprint
    }
}

/// The B-gate: is `draft` safe to promote against this `clean_corpus`?
///
/// Returns [`ToleranceVerdict::Spared`] iff the draft matches NONE of the clean
/// items; otherwise [`ToleranceVerdict::BindsCleanItem`] naming the first clean
/// item it bound. This is negative selection: a draft that binds a clean sibling
/// is rejected because promoting it ships autoimmunity.
///
/// `clean_corpus` is a slice of `syn::Item`s the operator asserts are clean (the
/// known-good siblings the draft must spare). A larger, more representative
/// corpus is a stronger gate (claim-scope: corpus-bounded).
#[must_use]
pub fn evaluate(draft: &Fingerprint, clean_corpus: &[syn::Item]) -> ToleranceVerdict {
    for (i, item) in clean_corpus.iter().enumerate() {
        if draft.matches(item) {
            return ToleranceVerdict::BindsCleanItem {
                clean_index: Some(i),
            };
        }
    }
    ToleranceVerdict::Spared
}

/// The boolean spare-clean predicate (the B-gate contract).
///
/// `true` iff `draft` is SAFE to promote (it spares every clean-corpus item).
/// Thin wrapper over [`evaluate`] for callers that only need the yes/no.
///
/// This is the gate C consults at the PROMOTE step: **C must not promote a draft
/// for which this returns `false`.**
#[must_use]
pub fn spare_clean(draft: &Fingerprint, clean_corpus: &[syn::Item]) -> bool {
    evaluate(draft, clean_corpus).is_safe()
}

/// The draft's **top-level conjuncts, producer-normalized** (ADR-047 Amendment 1 —
/// the shape-fragility seam fix).
///
/// Two producers emit two top-level *shapes* for the SAME semantic fingerprint:
/// `anti_unify` (the real generator) emits a **flat** `Vec<Constraint>` (e.g.
/// `[Item, ImplOfTrait, AnyOf]`), while `Fingerprint::parse("all_of([..])")` (and a
/// `narrow()`/persist reconstruction via the `all_of(..)` surface) emits a single
/// **wrapped** `[AllOf([..])]`. The GATE-G primitives ((A)-binary + near-miss) read
/// *top-level* conjuncts, so without normalization a wrapped bare-structural draft
/// reads its outer `AllOf` as a discriminating conjunct and **evades the (A)-binary
/// refusal** — the safety verdict would depend on which producer built the draft
/// (antigen's own `ParallelStateTrackersDiverge` at the keystone gate). Normalizing
/// a *single* top-level `AllOf` wrapper makes the verdict **producer-independent**.
///
/// Scope (honest): this unwraps a SOLE top-level `AllOf` (one level). It does NOT
/// recurse into a *nested* `AllOf` that sits *alongside* other top-level conjuncts —
/// that is the harder ADR-047 OQ2 nested-vacuity case (a single top-level drop of an
/// inner `AllOf` can still collapse many discriminators at once), still pinned
/// born-red in `atk_047_shape_fragility_seam.rs` and deferred to a future generator
/// that emits nesting. Today `anti_unify` never nests, so this one-level normalize
/// closes the live (parse/narrow/persist-surface) seam.
fn normalized_top_level(draft: &Fingerprint) -> &[Constraint] {
    match draft.constraints.as_slice() {
        // A sole top-level `all_of([..])` wrapper: read its children as the real
        // top-level conjuncts (the flat shape `anti_unify` would have emitted).
        [Constraint::AllOf(inner)] => inner,
        // Already flat (the production shape) — read as-is.
        other => other,
    }
}

/// The **(A)-binary** discriminating-conjunct predicate (ADR-047 §Mechanics 2;
/// shared with C's non-degeneracy guard, ADR-056 — ONE predicate, two call-sites,
/// no `ParallelStateTrackersDiverge`).
///
/// `true` iff `draft.constraints` carries **≥1 conjunct that is not a bare
/// structural anchor** — i.e. it carries an actual discriminating signal that
/// distinguishes a defect from its clean family-sibling. A draft failing this is
/// **bare-structural over-general** (it binds the whole structural family and
/// spares none in-family) — a SAFETY problem B refuses (and C refuses upstream).
///
/// **The partition (locked against the real `Constraint` enum, ADR-047 OQ3 /
/// ADR-056 OQ3), stated as a principle:** an anchor that names *what the item IS*
/// is a **structural/identity anchor** — [`Constraint::Item`] (the item-kind),
/// [`Constraint::ImplOfTrait`] (the trait-impl identity), [`Constraint::NameMatches`]
/// (the item's name). Every other constraint names *what distinguishes a defect*
/// (a body signal, a qualifier, a derive/attr/serde introspection, a range, a
/// boolean combinator) and is **discriminating**. A future generator that anchors
/// a family on some new identity constraint extends the anchor set here — the rule
/// is the principle, not the three-element list.
///
/// **BINARY, never a count** (ADR-047 §Standing invariant): "has-a-discriminating-
/// conjunct? yes/no." A tunable "≥K conjuncts" floor would be a magic number on the
/// *generalization* axis installed inside the *safety* gate — antigen's own
/// `FingerprintGamedNotDefended`/Goodhart class. There is no number to game.
#[must_use]
pub fn has_discriminating_conjunct(draft: &Fingerprint) -> bool {
    // Read producer-normalized top-level conjuncts so the verdict is the same for a
    // flat (anti_unify) and a single-AllOf-wrapped (parse/narrow/persist) draft of
    // identical semantics (ADR-047 Amendment 1).
    normalized_top_level(draft).iter().any(is_discriminating)
}

/// One constraint's side of the (A)-binary partition: `false` for a bare
/// structural/identity anchor, `true` for a discriminating signal. The single
/// source of truth for the partition (ADR-047 OQ3 / ADR-056 OQ3).
const fn is_discriminating(c: &Constraint) -> bool {
    match c {
        // Structural/identity anchors — name *what the item IS*, not what
        // distinguishes a defect. A draft of only these over-binds the family.
        Constraint::Item(_) | Constraint::ImplOfTrait(_) | Constraint::NameMatches(_) => false,
        // Everything else distinguishes a defect from its clean sibling:
        // body signals, qualifiers, attr/derive/serde introspection, ranges,
        // and the boolean combinators (which carry discriminating children).
        Constraint::Variants(_)
        | Constraint::HasMethod(_)
        | Constraint::AttrPresent(_)
        | Constraint::DocContains(_)
        | Constraint::BodyContainsMacro(_)
        | Constraint::BodyCalls(_)
        | Constraint::Qualifier(_)
        | Constraint::Derives(_)
        | Constraint::SerdeArg(_)
        | Constraint::AllOf(_)
        | Constraint::AnyOf(_)
        | Constraint::Not(_) => true,
    }
}

/// Is `item` a **near-miss** for `draft`? (ADR-047 §Mechanics 1 — the GATE-G
/// non-vacuity primitive.)
///
/// `true` iff the draft has **≥2 top-level conjuncts**, the draft does NOT bind the
/// item, and there exists ONE conjunct whose removal makes the draft bind it. A
/// near-miss item matches all-but-one of the draft's conjuncts and is spared by
/// failing exactly that remaining one — the proof that B made a **real in-family
/// discrimination** (it spared an item it *plausibly could have flagged*, not an
/// item it was never near).
///
/// # The `len >= 2` guard closes the empty-`all_of` vacuity (ATK-047-N4)
///
/// Dropping the sole conjunct of a single-conjunct draft yields an *empty*
/// `constraints` vector, which the shipped matcher makes **vacuously `Match`**
/// (`matcher.rs` — "an empty slice is vacuously `Match`"). Without the guard, every
/// non-binding item would be a false "near-miss" and vacuity reopens — and a
/// `narrow()`-minted single *discriminating* conjunct (`[body_calls("unwrap")]`)
/// PASSES (A)-binary yet still empty-drops, so (A)-binary does NOT subsume this; it
/// is a distinct hole. A single-conjunct draft has **no valid near-miss by
/// construction** (there is nothing to be "one constraint away" from when there is
/// only one constraint) → it is not corpus-witnessable (route-to-human).
///
/// **Top-level conjuncts only:** `anti_unify` emits a flat top-level `all_of` (the
/// `any_of` is itself one top-level conjunct, never nested). A future generator
/// emitting a *nested* `any_of` would want a recursive drop — a scope boundary
/// (ADR-047 OQ2), not built now. The dropped-conjunct fingerprint is matched by the
/// standard [`Fingerprint::matches`]; its `Undefined` projects to "doesn't fire"
/// via the shipped Kleene-strong matcher, so there is no separate-skeleton
/// definedness artifact (closing ATK-047-3).
#[must_use]
pub fn is_near_miss(draft: &Fingerprint, item: &syn::Item) -> bool {
    // Read producer-normalized top-level conjuncts (ADR-047 Amendment 1): a single
    // top-level `all_of([..])` wrapper is unwrapped to the flat conjuncts the
    // generator would have emitted, so a wrapped draft drops one REAL conjunct at a
    // time (not the sole outer wrapper, which would always-empty-drop to vacuous).
    let conjuncts = normalized_top_level(draft);
    // A single-conjunct (or empty) draft has no valid near-miss: dropping its sole
    // conjunct yields the empty all_of (vacuously Match), which would make every
    // non-binding item a false near-miss (ATK-047-N4).
    if conjuncts.len() < 2 {
        return false;
    }
    // A near-miss is SPARED by the whole draft (it fails exactly the one conjunct).
    if draft.matches(item) {
        return false;
    }
    // ∃ one conjunct whose removal makes the (still non-empty) draft bind the item.
    (0..conjuncts.len()).any(|i| {
        let mut cs = conjuncts.to_vec();
        cs.remove(i);
        // `cs` is non-empty here (len was ≥ 2), so this is not the vacuous empty match.
        Fingerprint { constraints: cs }.matches(item)
    })
}

/// Does the clean corpus contain a **near-miss** for `draft`? (ADR-047
/// §Mechanics 3 — the near-miss non-vacuity check.)
///
/// `true` iff ≥1 corpus item is one constraint from binding the draft (the draft's
/// generalization is corpus-exercised); `false` (route-to-human) iff no corpus item
/// is — B cannot certify the generalization.
#[must_use]
pub fn corpus_witnesses_draft(draft: &Fingerprint, corpus: &[syn::Item]) -> bool {
    corpus.iter().any(|item| is_near_miss(draft, item))
}

/// Promote `draft` through B's gate against a clean corpus, minting a
/// [`PromotedDraft`] capability-token iff it passes ALL THREE of ADR-047's checks.
///
/// **The sole minter of [`PromotedDraft`]** (ADR-048): on success returns
/// `Ok(PromotedDraft)` — the only assertable generalization; on failure returns
/// `Err(ToleranceVerdict)` naming why. A caller routing its draft through
/// `promote_if_safe` *cannot* obtain a token that failed B — the autoimmune draft
/// is structurally unable to acquire one (unforgettable-by-construction, ADR-048).
///
/// The checks run **in order** (ADR-047 §Mechanics 4):
/// 1. **Empty-corpus refusal** (the gate-G hazard, captain's v0.4 ruling): an empty
///    corpus makes [`spare_clean`] *vacuously* `true` — B would verify NOTHING.
///    Refused as `BindsCleanItem { clean_index: None }` ("cannot certify safety
///    against nothing"). The refusal lives HERE (structural), not per-caller.
/// 2. **(A)-binary SAFETY refusal** ([`has_discriminating_conjunct`]): a
///    bare-structural over-general draft (no discriminating signal) binds the whole
///    family → `BindsCleanItem { clean_index: None }`. (C's non-degeneracy guard,
///    ADR-056, catches this upstream too — defense-in-depth.)
/// 3. **Near-miss GENERALIZATION check** ([`corpus_witnesses_draft`]): no corpus
///    item is one constraint from binding → `NotCorpusWitnessable` (SAFE but B
///    cannot certify the generalization → route-to-human, ADR-051).
/// 4. **Spare-clean SAFETY check** ([`spare_clean`]): the draft binds a clean
///    corpus item → `BindsCleanItem { clean_index: Some(i) }` (autoimmune).
///
/// **Two axes, kept distinct** (ADR-047 rev-3): (A)-binary and spare-clean are
/// SAFETY (does it over-bind?); near-miss is GENERALIZATION-QUALITY (can B certify
/// it extends?). A draft that is *safe but not near-miss-witnessed* (a
/// twins-collapsed draft) routes to a human (`NotCorpusWitnessable`) — B refuses to
/// fake the generalization-verdict it cannot make. `Result` (not `Option`) is
/// load-bearing: it carries the three-valued verdict's route-to-human reason
/// through the type (an `Option` would swallow it).
///
/// # The score tier (ADR-048/049/050)
///
/// The minted token carries a `tier: Provenance` (the score, ADR-049 — mandatory,
/// type-enforced). ADR-050's two-signal routing computes the real tier at
/// promotion; that routing is a *later* build unit (the `incident=` slice). Until
/// it wires in, the gate assigns the **conservative floor** [`Provenance::DEFAULT`]
/// (`Imagined`, the lowest tier — "can never over-claim by omission"). This is the
/// honest default, not a divergence: a gate-assigned token is scored at the floor
/// until the routing earns it a higher tier. (Build-seam for unit 6 — ADR-050:
/// replace the floor with the two-signal routing here.)
pub fn promote_if_safe(
    draft: Fingerprint,
    clean_corpus: &[syn::Item],
) -> Result<PromotedDraft, ToleranceVerdict> {
    // (1) Cannot certify safety against nothing: an empty corpus makes spare_clean
    //     vacuously true, which would promote a draft B never actually checked.
    if clean_corpus.is_empty() {
        return Err(ToleranceVerdict::BindsCleanItem { clean_index: None });
    }
    // (2) (A)-binary SAFETY refusal: a bare-structural draft (no discriminating
    //     signal) over-binds the whole structural family — refuse before scanning.
    if !has_discriminating_conjunct(&draft) {
        return Err(ToleranceVerdict::BindsCleanItem { clean_index: None });
    }
    // (3) Near-miss GENERALIZATION check: with no near-miss in the corpus B cannot
    //     certify the draft generalizes — route to a human (SAFE, not autoimmune).
    if !corpus_witnesses_draft(&draft, clean_corpus) {
        return Err(ToleranceVerdict::NotCorpusWitnessable);
    }
    // (4) Spare-clean SAFETY check: a draft that binds a clean item is autoimmune.
    match evaluate(&draft, clean_corpus) {
        ToleranceVerdict::Spared => Ok(PromotedDraft {
            fingerprint: draft,
            // ADR-050's two-signal routing wires the real tier here (build unit 6);
            // until then, the conservative floor (honest, never over-claims).
            tier: Provenance::DEFAULT,
        }),
        // The draft bound a clean item — autoimmune. (NotCorpusWitnessable is
        // already handled above; evaluate only returns Spared / BindsCleanItem.)
        verdict @ ToleranceVerdict::BindsCleanItem { .. } => Err(verdict),
        // evaluate never returns NotCorpusWitnessable; preserve totality honestly.
        ToleranceVerdict::NotCorpusWitnessable => Err(ToleranceVerdict::NotCorpusWitnessable),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drop_family() -> Vec<syn::Item> {
        let src = r#"
            pub struct GuardA;
            impl Drop for GuardA {
                fn drop(&mut self) { let _ = flush().unwrap(); }
            }
            pub struct GuardB;
            impl Drop for GuardB {
                fn drop(&mut self) { let _ = flush().expect("must"); }
            }
            pub struct CleanGuard;
            impl Drop for CleanGuard {
                fn drop(&mut self) { let _ = flush().ok(); }
            }
        "#;
        syn::parse_file(src).expect("parses").items
    }

    fn impl_drop_for(items: &[syn::Item], ty: &str) -> syn::Item {
        items
            .iter()
            .find(|it| {
                let syn::Item::Impl(i) = it else { return false };
                let Some((_, tp, _)) = &i.trait_ else {
                    return false;
                };
                let is_drop = tp.segments.last().is_some_and(|s| s.ident == "Drop");
                let syn::Type::Path(p) = &*i.self_ty else {
                    return false;
                };
                let on = p.path.segments.last().is_some_and(|s| s.ident == ty);
                is_drop && on
            })
            .expect("found")
            .clone()
    }

    /// Build a FLAT top-level fingerprint from `src`'s single top-level `all_of`,
    /// unwrapping it into the flat `Vec<Constraint>` shape `anti_unify` really emits
    /// (ADR-047 OQ2: the near-miss predicate drops one *top-level* conjunct, and the
    /// real generator output is flat — `parse("all_of([..])")` would wrap everything
    /// in ONE `AllOf` conjunct, which is NOT the shape the predicate sees in prod).
    fn flat(src: &str) -> Fingerprint {
        let parsed = Fingerprint::parse(src).unwrap();
        match parsed.constraints.as_slice() {
            [Constraint::AllOf(inner)] => Fingerprint {
                constraints: inner.clone(),
            },
            _ => parsed,
        }
    }

    /// `[Item(Impl), ImplOfTrait("Drop")]` (flat) — bare-structural (NO
    /// discriminating signal). It binds every `Drop` impl (autoimmune) AND fails
    /// (A)-binary. The shape `anti_unify` emits for a body-signal-less Drop cluster.
    fn naive_draft() -> Fingerprint {
        flat(r#"all_of([item = impl, impl_of_trait("Drop")])"#)
    }

    /// The healthy disjunction draft (flat): carries a discriminating `any_of`,
    /// binds the `{unwrap, expect}` defects, spares the `.ok()` clean sibling. The
    /// flat top-level shape `anti_unify` emits (`Item`, `ImplOfTrait`, `AnyOf` at top).
    fn disjunction_draft() -> Fingerprint {
        flat(
            r#"all_of([item = impl, impl_of_trait("Drop"), any_of([body_calls("unwrap"), body_calls("expect")])])"#,
        )
    }

    #[test]
    fn rejects_the_naive_autoimmune_draft() {
        let items = drop_family();
        let clean = vec![impl_drop_for(&items, "CleanGuard")];
        // The naive draft binds the clean sibling → the spare-clean PREDICATE
        // reports BindsCleanItem at index 0.
        let v = evaluate(&naive_draft(), &clean);
        assert_eq!(
            v,
            ToleranceVerdict::BindsCleanItem {
                clean_index: Some(0)
            }
        );
        assert!(!v.is_safe());
        assert!(!spare_clean(&naive_draft(), &clean));
        // The GATE refuses it — now via the (A)-binary bare-structural refusal,
        // which fires BEFORE the spare-clean scan (clean_index: None).
        assert_eq!(
            promote_if_safe(naive_draft(), &clean),
            Err(ToleranceVerdict::BindsCleanItem { clean_index: None })
        );
    }

    #[test]
    fn accepts_the_disjunction_draft() {
        let items = drop_family();
        // The clean corpus must contain a NEAR-MISS to witness the generalization:
        // CleanGuard (.ok()) matches {impl, Drop} and fails only the any_of → a
        // near-miss. So the gate promotes.
        let clean = vec![impl_drop_for(&items, "CleanGuard")];
        let v = evaluate(&disjunction_draft(), &clean);
        assert_eq!(v, ToleranceVerdict::Spared);
        assert!(v.is_safe());
        assert!(spare_clean(&disjunction_draft(), &clean));
        let promoted = promote_if_safe(disjunction_draft(), &clean).expect("near-miss-witnessed");
        // The token wraps the gated fingerprint and carries the conservative floor
        // tier (ADR-050 routing wires the real tier later).
        assert_eq!(promoted.fingerprint(), &disjunction_draft());
        assert_eq!(promoted.tier(), Provenance::DEFAULT);
    }

    #[test]
    fn empty_corpus_spare_clean_predicate_is_vacuously_true() {
        // The PREDICATE `spare_clean` is honestly vacuously true on an empty
        // corpus: there is no clean item for the draft to bind, so "spares every
        // clean item" holds trivially. This is the corpus-bounded claim-scope; the
        // predicate reports the literal fact.
        assert!(spare_clean(&naive_draft(), &[]));
    }

    #[test]
    fn promote_if_safe_refuses_an_empty_corpus_the_gate_g_hazard() {
        // The GATE (`promote_if_safe`) REFUSES an empty corpus despite spare_clean
        // being vacuously true (captain's ruling, gate-G): a vacuous pass is
        // autoimmunity-with-a-green-check — B verified NOTHING. "Cannot certify
        // safety against nothing." Even the OBVIOUSLY-safe disjunction draft must
        // not promote against emptiness — the refusal is about the corpus being
        // empty, not the draft being unsafe.
        assert!(promote_if_safe(naive_draft(), &[]).is_err());
        assert!(promote_if_safe(disjunction_draft(), &[]).is_err());
    }

    #[test]
    fn rejects_when_any_clean_item_binds_not_just_the_first() {
        let items = drop_family();
        // Put a non-binding item first, the clean Drop sibling second: the naive
        // draft binds the SECOND → the PREDICATE reports it, with the right index.
        let nonbinding: syn::Item = syn::parse_quote! { pub struct NotEvenADrop; };
        let clean = vec![nonbinding, impl_drop_for(&items, "CleanGuard")];
        assert_eq!(
            evaluate(&naive_draft(), &clean),
            ToleranceVerdict::BindsCleanItem {
                clean_index: Some(1)
            }
        );
    }

    // ========================================================================
    // ADR-047 §Q9 — the BORN-RED GATE-G near-miss spec. The adversarial's RAN
    // cases ARE the definition of done (the near-miss primitive, rev-3 locked).
    // Ported from `R:/antigen-atk-scratch/src/atk3.rs` (the near-miss harness).
    // ========================================================================

    /// ADR-047 §Q9 ATK-047-N4 — `single_conjunct_draft_is_not_near_miss_via_empty_drop`.
    /// A single-conjunct draft (e.g. a `narrow()`-minted `[body_calls("unwrap")]`
    /// that PASSES (A)-binary) is NOT near-miss-witnessable: the `len >= 2` guard
    /// prevents the empty-drop, so it does not vacuously near-miss-match every
    /// non-binding item. (The empty-`all_of` vacuity `matcher.rs:88` would
    /// otherwise reopen; (A)-binary does NOT subsume it.)
    #[test]
    fn single_conjunct_draft_is_not_near_miss_via_empty_drop() {
        let single = Fingerprint {
            constraints: vec![Constraint::BodyCalls("unwrap".into())],
        };
        let clean: syn::Item = syn::parse_str("fn clean() { let _ = ok(); }").unwrap();
        // It does NOT bind the clean fn, and dropping its sole conjunct would yield
        // the empty (vacuously-Match) all_of — the guard refuses to call that a
        // near-miss.
        assert!(
            !is_near_miss(&single, &clean),
            "a single-conjunct draft has no valid near-miss (len < 2 guard, ATK-047-N4)"
        );
        // And it routes-to-human at the gate (no corpus witness possible).
        assert_eq!(
            promote_if_safe(single, std::slice::from_ref(&clean)),
            Err(ToleranceVerdict::NotCorpusWitnessable)
        );
    }

    /// ADR-047 §Q9 (A)-binary — `bare_structural_draft_rejected_as_autoimmune`. A
    /// bare-structural over-general draft (no discriminating signal) is refused by
    /// the (A)-binary SAFETY check (it over-binds the whole family).
    #[test]
    fn bare_structural_draft_rejected_as_autoimmune() {
        let items = drop_family();
        let clean = vec![impl_drop_for(&items, "CleanGuard")];
        assert!(
            !has_discriminating_conjunct(&naive_draft()),
            "the bare-structural draft carries no discriminating conjunct"
        );
        assert_eq!(
            promote_if_safe(naive_draft(), &clean),
            Err(ToleranceVerdict::BindsCleanItem { clean_index: None }),
            "bare-structural over-general → (A)-binary refusal"
        );
    }

    /// ADR-047 §Q9 ATK-047-2 — `twins_collapsed_draft_routes_to_human`. The
    /// identical-twins cluster (no `any_of`) yields a *precise* draft that is SAFE
    /// (near-miss sees the `.ok()` sibling) but B cannot certify the
    /// twins-generalization, so it routes to human (`NotCorpusWitnessable`), NOT
    /// auto-promote. Here the draft is a precise no-disjunction shape; we assert the
    /// near-miss IS witnessed (the safety primitive sees it) AND it spares clean —
    /// the routing decision (route-to-human for generation-quality) is C's
    /// confidence signal's job (ADR-056), distinct from B's safety verdict.
    #[test]
    fn twins_collapsed_draft_is_near_miss_witnessed_and_spares_clean() {
        // A precise no-disjunction draft (the twins shape): {impl, Drop, flush, unwrap}.
        let twins_draft = flat(
            r#"all_of([item = impl, impl_of_trait("Drop"), body_calls("flush"), body_calls("unwrap")])"#,
        );
        let clean: syn::Item =
            syn::parse_str("impl Drop for Clean { fn drop(&mut self) { let _ = flush().ok(); } }")
                .unwrap();
        // The clean `flush().ok()` sibling matches {impl, Drop, flush} and fails only
        // `unwrap` → a near-miss. B's discrimination is real (it is SAFE).
        assert!(
            is_near_miss(&twins_draft, &clean),
            "the .ok() sibling is one constraint (unwrap) from binding → near-miss"
        );
        // The draft carries a discriminating signal (body_calls), so (A)-binary
        // passes; near-miss is witnessed; it spares clean → the gate PROMOTES it
        // (it is safe). The twins-generalization-confidence concern is C's signal
        // (ADR-056), which caps its tier — NOT a B-side refusal.
        let promoted = promote_if_safe(twins_draft, std::slice::from_ref(&clean));
        assert!(
            promoted.is_ok(),
            "a precise near-miss-witnessed spare-clean draft is SAFE → promotes"
        );
    }

    /// ADR-047 §Q9 (A)-binary positive —
    /// `precise_no_disjunction_draft_with_real_discrimination_promotes`. A
    /// no-`any_of` draft whose core carries a genuine discriminating signal AND is
    /// near-miss-witnessed promotes — proving (A)-binary does NOT brick the precise
    /// no-disjunction case (only the bare-structural one).
    #[test]
    fn precise_no_disjunction_draft_with_real_discrimination_promotes() {
        let precise = flat(r#"all_of([item = impl, impl_of_trait("Drop"), body_calls("unwrap")])"#);
        // A clean sibling one constraint (unwrap) from binding → near-miss.
        let clean: syn::Item =
            syn::parse_str("impl Drop for Clean { fn drop(&mut self) { let _ = flush().ok(); } }")
                .unwrap();
        assert!(has_discriminating_conjunct(&precise));
        assert!(
            promote_if_safe(precise, std::slice::from_ref(&clean)).is_ok(),
            "a precise (no-any_of) draft with real discrimination must promote"
        );
    }

    /// ADR-047 §Q9 ATK-047-3 — `near_miss_verdict_invariant_to_corpus_item_class`.
    /// The same safe Drop draft against a struct-sourced vs impl-sourced clean
    /// corpus must yield the SAME verdict (no silent flip on definedness). A bodyless
    /// struct is NOT a near-miss for an impl-shaped draft (the near-miss reads the
    /// whole draft through the shipped Kleene matcher, no separate skeleton).
    #[test]
    fn near_miss_verdict_invariant_to_corpus_item_class() {
        let draft = disjunction_draft();
        let a_struct: syn::Item = syn::parse_str("struct JustAStruct;").unwrap();
        // A bodyless struct must NOT register as a near-miss for an impl draft.
        assert!(
            !is_near_miss(&draft, &a_struct),
            "a struct cannot witness an impl-shaped draft (no Undefined-collapse spurious witness)"
        );
        // Against a struct-only corpus the gate routes-to-human; the verdict does
        // not silently flip to a promote.
        assert_eq!(
            promote_if_safe(draft, std::slice::from_ref(&a_struct)),
            Err(ToleranceVerdict::NotCorpusWitnessable)
        );
    }

    /// ADR-047 §Q9 positive control — `near_miss_promotes_the_good_drop_family`.
    /// The canonical `{unwrap, expect}` family + `CleanGuard` promotes (Witnessed) —
    /// the happy path stays green.
    #[test]
    fn near_miss_promotes_the_good_drop_family() {
        let items = drop_family();
        let clean = vec![impl_drop_for(&items, "CleanGuard")];
        assert!(
            corpus_witnesses_draft(&disjunction_draft(), &clean),
            "the good family's CleanGuard is a near-miss"
        );
        assert!(promote_if_safe(disjunction_draft(), &clean).is_ok());
    }

    // ========================================================================
    // ADR-048 §Q9 — the BORN-RED PromotedDraft capability-token spec.
    // (The trybuild compile-fail `promoted_draft_has_no_public_constructor` lives
    // in a stable-blessed trybuild fixture, NOT here — a unit test cannot assert a
    // compile error. These assert the runtime/type-shape half.)
    // ========================================================================

    /// ADR-048 §Q9 — `propose_returns_promoted_draft_not_fingerprint` (gate half).
    /// `promote_if_safe`'s `Ok` payload is a `PromotedDraft` (the capability token),
    /// not a bare `Fingerprint`. The token's only construction path is the gate.
    #[test]
    fn promote_if_safe_returns_promoted_draft_not_fingerprint() {
        let items = drop_family();
        let clean = vec![impl_drop_for(&items, "CleanGuard")];
        let promoted: PromotedDraft =
            promote_if_safe(disjunction_draft(), &clean).expect("promotes");
        // Read-only access yields a &Fingerprint (for .matches() / serialize),
        // never an owned un-tokened one.
        let fp: &Fingerprint = promoted.fingerprint();
        assert_eq!(fp, &disjunction_draft());
        assert_eq!(promoted.as_ref(), &disjunction_draft());
    }

    /// ADR-048 §Q9 — `into_fingerprint_downgrades_capability`. The owned extraction
    /// yields a plain `Fingerprint` (assertable as nothing); re-promotion requires
    /// re-routing through the gate. The downgrade is explicit and one-way.
    #[test]
    fn into_fingerprint_downgrades_capability() {
        let items = drop_family();
        let clean = vec![impl_drop_for(&items, "CleanGuard")];
        let promoted = promote_if_safe(disjunction_draft(), &clean).expect("promotes");
        // The downgrade yields an owned bare Fingerprint — no longer a token.
        let downgraded: Fingerprint = promoted.into_fingerprint();
        assert_eq!(downgraded, disjunction_draft());
        // Re-acquiring a token requires re-routing through the gate (the only minter).
        assert!(promote_if_safe(downgraded, &clean).is_ok());
    }
}
