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

use antigen_fingerprint::Fingerprint;

/// The verdict of the spare-clean gate for one draft against one clean corpus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToleranceVerdict {
    /// The draft spares every clean-corpus item — SAFE to promote.
    Spared,
    /// The draft matched at least one clean-corpus item — REJECT (promoting it
    /// would flag clean code: autoimmunity). Carries the index of the first
    /// clean item it bound, for a precise diagnostic.
    BindsCleanItem {
        /// The position (in the supplied corpus slice) of the first clean item
        /// the draft matched.
        clean_index: usize,
    },
}

impl ToleranceVerdict {
    /// `true` iff the draft is safe to promote (it spared the whole clean corpus).
    #[must_use]
    pub const fn is_safe(&self) -> bool {
        matches!(self, Self::Spared)
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
            return ToleranceVerdict::BindsCleanItem { clean_index: i };
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

/// Promote `draft` only if it passes the spare-clean gate.
///
/// Returns `Some(draft)` (moved through) iff the draft spares the clean corpus;
/// `None` (rejected) if it binds any clean item. This is the type-level
/// enforcement of "C must never promote without B": a caller that routes its
/// draft through `promote_if_safe` *cannot* obtain a promoted draft that failed B
/// — the autoimmune draft is structurally unable to pass.
#[must_use]
pub fn promote_if_safe(draft: Fingerprint, clean_corpus: &[syn::Item]) -> Option<Fingerprint> {
    if spare_clean(&draft, clean_corpus) {
        Some(draft)
    } else {
        None
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

    fn naive_draft() -> Fingerprint {
        Fingerprint::parse(r#"all_of([item = impl, impl_of_trait("Drop")])"#).unwrap()
    }

    fn disjunction_draft() -> Fingerprint {
        Fingerprint::parse(
            r#"all_of([item = impl, impl_of_trait("Drop"), any_of([body_calls("unwrap"), body_calls("expect")])])"#,
        )
        .unwrap()
    }

    #[test]
    fn rejects_the_naive_autoimmune_draft() {
        let items = drop_family();
        let clean = vec![impl_drop_for(&items, "CleanGuard")];
        // The naive draft binds the clean sibling → REJECT.
        let v = evaluate(&naive_draft(), &clean);
        assert_eq!(v, ToleranceVerdict::BindsCleanItem { clean_index: 0 });
        assert!(!v.is_safe());
        assert!(!spare_clean(&naive_draft(), &clean));
        assert!(promote_if_safe(naive_draft(), &clean).is_none());
    }

    #[test]
    fn accepts_the_disjunction_draft() {
        let items = drop_family();
        let clean = vec![impl_drop_for(&items, "CleanGuard")];
        // The disjunction draft spares the clean sibling → SAFE.
        let v = evaluate(&disjunction_draft(), &clean);
        assert_eq!(v, ToleranceVerdict::Spared);
        assert!(v.is_safe());
        assert!(spare_clean(&disjunction_draft(), &clean));
        assert!(promote_if_safe(disjunction_draft(), &clean).is_some());
    }

    #[test]
    fn empty_corpus_spares_everything_a_documented_weakness() {
        // With NO clean corpus, the gate cannot reject anything — even the
        // autoimmune draft "passes". This is the corpus-bounded claim-scope: an
        // empty corpus is a vacuous gate. (A real promote path must supply a
        // representative clean corpus; this test pins the boundary behavior so it
        // is a conscious property, not a silent hole.)
        assert!(spare_clean(&naive_draft(), &[]));
    }

    #[test]
    fn rejects_when_any_clean_item_binds_not_just_the_first() {
        let items = drop_family();
        // Put a non-binding item first, the clean Drop sibling second: the naive
        // draft binds the SECOND → rejected, with the right index.
        let nonbinding: syn::Item = syn::parse_quote! { pub struct NotEvenADrop; };
        let clean = vec![nonbinding, impl_drop_for(&items, "CleanGuard")];
        assert_eq!(
            evaluate(&naive_draft(), &clean),
            ToleranceVerdict::BindsCleanItem { clean_index: 1 }
        );
    }
}
