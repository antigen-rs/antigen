//! C — PROPOSE, the anti-unify generator (v0.4, ADR-045: the affinity-maturation
//! arm; the falsifiable keystone of the learning core).
//!
//! Given a CLUSTER of structurally-similar marked sites (the marked-unknown
//! population, clustered by `shape_digest`), PROPOSE anti-unifies them into
//! a candidate ("drafted") [`Fingerprint`]. The draft is a **HYPOTHESIS**: a
//! ratifiable suggestion carrying provenance, **never** an auto-asserted
//! `#[presents]` or an auto-named failure-class (ADR-044, observe-don't-declare —
//! the syntactic/semantic line a machine must never assert across; the human or
//! an incident RATIFIES a draft into a named class).
//!
//! # Anti-unify TO DISJUNCTION (not naive-LGG)
//!
//! The naive least-general-generalization — drop the leaves that differ between
//! members — OVER-GENERALIZES: a `panic-in-Drop` cluster `{ .unwrap(), .expect() }`
//! collapses to "any `Drop` impl", which matches a CLEAN `Drop` sibling. The
//! generator's own output IS the false positive (the autoimmunity B exists to
//! prevent). PROPOSE instead anti-unifies **per leaf-type with set algebra**:
//!
//! - **Skeleton conjuncts** = the features in the INTERSECTION of all members
//!   (item-kind, trait-impl identity, and any body-call shared by *every*
//!   member). These hold across the whole cluster, so they stay AND'd.
//! - **Discriminating disjunction** = the body-calls present in SOME but not all
//!   members — wrapped in `any_of([...])`. This is the load-bearing wall: it
//!   carries the cluster's distinguishing signal without collapsing it to the
//!   member's whole shared skeleton (which clean code also has).
//!
//! On the fixture `{ GuardA: .unwrap()+.take(), GuardB: .expect()+.take() }` the
//! draft is
//! `all_of([item = impl, impl_of_trait("Drop"), body_calls("take"),
//!          any_of([body_calls("expect"), body_calls("unwrap")])])`
//! — it binds both defects and spares `CleanGuard` (`.ok()+.take()`): the
//! `any_of` arm is `NoMatch` on the clean sibling (it has neither `unwrap` nor
//! `expect`), so the whole `all_of` is `NoMatch`.
//!
//! # C ══ B — the one safety-tangle (ADR-045; the highest-stakes line)
//!
//! anti-unify-to-disjunction REDUCES but does not ELIMINATE autoimmunity: a
//! cluster whose distinguishing leaf happens to also appear in clean code still
//! over-binds. Only the corpus-checked gate eliminates it. So **PROPOSE never
//! promotes a draft except through B** ([`self_tolerance::promote_if_safe`]):
//! [`propose`] routes every draft through the spare-clean gate and returns
//! `None` if it binds a clean-corpus item. The raw [`anti_unify`] draft is
//! exposed for inspection (it is a hypothesis, plainly labeled), but the only
//! path to a *promotable* fingerprint is `propose`. Shipping a generator without
//! the selector ships autoimmunity — that is the line this module must not cross.
//!
//! # Claim-scope (ADR-044)
//!
//! **What PROPOSE proves:** the draft is a syntactic generalization that binds
//! every supplied cluster member (a decidable fact — extraction and matching use
//! the same syntactic walk, so a draft binds the sites it was generalized from by
//! construction) AND (when routed through [`propose`]) spares the supplied clean
//! corpus. **What PROPOSE does NOT prove:** that the draft names a real
//! failure-class, that it generalizes correctly to all code, or that the cluster
//! is a true failure-family. Those are the ratifier's (human/incident) job. A
//! promoted draft is still a *suggestion at a calibrated tier*, never an asserted
//! class.

use std::collections::BTreeSet;

use antigen_fingerprint::{Constraint, Fingerprint, ItemKind};
use syn::visit::Visit;

use crate::learn::self_tolerance::{
    self, PromotedDraft, ToleranceVerdict, has_discriminating_conjunct,
};

/// Why [`propose`] could not produce a promoted draft (ADR-048/056).
///
/// Widens B's [`ToleranceVerdict`] with the C-side (generator) non-promotion
/// reasons, so every non-promotion is **legible**, not a bare `None`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProposeOutcome {
    /// The cluster was empty — nothing to generalize.
    EmptyCluster,
    /// The cluster members share no common item-kind skeleton — a heterogeneous
    /// "cluster" is not a real family (`anti_unify` declines it).
    NoSharedSkeleton,
    /// **The C-side non-degeneracy REFUSAL** (ADR-056): the anti-unified draft is
    /// *degenerate* (bare-structural — only identity anchors, no discriminating
    /// signal), so it would over-bind its whole family. C refuses it at the
    /// generator with a generator-appropriate diagnostic ("these sites share only
    /// their structural shape — not a real failure-family"), *upstream* of B (where
    /// B's (A)-binary would also refuse it — defense-in-depth).
    Degenerate,
    /// **B's gate refused the draft** (ADR-047): carries the [`ToleranceVerdict`]
    /// (`BindsCleanItem` autoimmune, or `NotCorpusWitnessable` route-to-human).
    Rejected(ToleranceVerdict),
}

impl From<ToleranceVerdict> for ProposeOutcome {
    fn from(v: ToleranceVerdict) -> Self {
        Self::Rejected(v)
    }
}

/// The generator's **generalization-confidence** signal (ADR-056 §(2)) — how well
/// a draft is expected to *extend* beyond the cluster it was generalized from.
///
/// A SIGNAL, never a refusal: a low-confidence (twins/photocopy) draft is *safe*,
/// so C does not refuse it — it labels it low-confidence-to-generalize, and that
/// label folds into ADR-050's tier-routing (a low-confidence draft is capped lower
/// on the dial; the human/policy cuts). The v0.5 form is the cluster's **effective
/// diversity**: a homogeneous cluster (an empty discriminating set with near-
/// identical members) is `Low`; a cluster with a rich discriminating `any_of` is
/// higher. The richer diversity metric is charter (ADR-056 OQ2 — ship the simplest
/// non-gameable signal).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Confidence {
    /// Twins/photocopy or near-identical members — generalized from effectively
    /// N=1; route-to-human / cap the tier low (NOT a refusal — the draft is safe).
    Low,
    /// Some discriminating diversity, but a single distinguishing signal.
    Moderate,
    /// A rich discriminating disjunction over several distinct signals — the draft
    /// is exercised by real cluster diversity.
    High,
}

/// Anti-unify a `cluster` of structurally-similar items into a draft
/// [`Fingerprint`].
///
/// Generalizes TO DISJUNCTION (the differing body-call leaves become an
/// `any_of`), never the naive drop-leaves collapse.
///
/// The returned draft is a **HYPOTHESIS** (ADR-044): it has NOT been checked
/// against a clean corpus and is NOT a promotable fingerprint. Use [`propose`]
/// to obtain a draft that has passed B (the spare-clean gate). Calling
/// `anti_unify` directly and matching with the result bypasses the safety gate —
/// do that only for inspection, never to promote.
///
/// Returns `None` when the cluster is empty (nothing to generalize) or when the
/// members share no common item-kind skeleton (a heterogeneous "cluster" is not
/// a real family — anti-unifying it would produce a shapeless, over-broad draft;
/// PROPOSE declines rather than emit one).
///
/// The generalization, per leaf-type:
/// - **item-kind**: a conjunct iff every member is the same [`ItemKind`].
/// - **trait-impl identity** (`impl_of_trait`): a conjunct iff every member is an
///   `impl` of the SAME trait (last-segment).
/// - **body signals** (a `body_calls` *call* OR a `body_contains_macro`
///   invocation — the two shapes a panic source can take): the signals present in
///   *every* member become conjuncts; the signals present in *some but not all*
///   members anti-unify into an `any_of([...])` disjunction (the discriminating
///   signal). A mixed family — one member panicking via `.unwrap()` (a call),
///   another via `panic!` (a macro) — yields the intended mixed disjunction
///   `any_of([body_calls("unwrap"), body_contains_macro("panic")])`.
#[must_use]
pub fn anti_unify(cluster: &[syn::Item]) -> Option<Fingerprint> {
    if cluster.is_empty() {
        return None;
    }

    // Per-member feature extraction. Each member yields its item-kind, its
    // trait-impl identity (if it is a trait impl), and the SET of body signals
    // (calls + macro invocations) its body makes — all read with the same
    // syntactic discipline the matcher uses, so a draft leaf matches a member iff
    // the member really has the feature.
    let features: Vec<MemberFeatures> = cluster.iter().map(MemberFeatures::extract).collect();

    let mut conjuncts: Vec<Constraint> = Vec::new();

    // --- item-kind: a conjunct iff shared by every member ---
    let first_kind = features[0].item_kind;
    let shared_kind = first_kind.filter(|k| features.iter().all(|f| f.item_kind == Some(*k)));
    let shared_kind = shared_kind?; // no common item-kind → not a real family.
    conjuncts.push(Constraint::Item(shared_kind));

    // --- trait-impl identity: a conjunct iff every member impls the same trait ---
    if let Some(trait_name) = &features[0].impl_of_trait {
        if features
            .iter()
            .all(|f| f.impl_of_trait.as_deref() == Some(trait_name.as_str()))
        {
            conjuncts.push(Constraint::ImplOfTrait(trait_name.clone()));
        }
    }

    // --- body signals: intersection → conjuncts; the rest → any_of disjunction ---
    // The intersection holds across the whole cluster (a stable shared signal);
    // the signals present in only SOME members are the discriminating leaves that
    // anti-unify TO DISJUNCTION (the autoimmunity-reducing move). Calls and macros
    // share one pool so a mixed call/macro panic-family yields one mixed `any_of`.
    let shared_signals: BTreeSet<BodySignal> = features
        .iter()
        .map(|f| f.body_signals.clone())
        .reduce(|acc, s| acc.intersection(&s).cloned().collect())
        .unwrap_or_default();

    let all_signals: BTreeSet<BodySignal> = features
        .iter()
        .flat_map(|f| f.body_signals.clone())
        .collect();

    // The discriminating signals: present in the union but not in every member.
    let discriminating: BTreeSet<BodySignal> =
        all_signals.difference(&shared_signals).cloned().collect();

    // Shared signals become conjuncts (BTreeSet → deterministic order). Every
    // member has these by construction (they are the intersection).
    for sig in &shared_signals {
        conjuncts.push(sig.to_constraint());
    }

    // THE BIND-EVERY-MEMBER INVARIANT (the load-bearing correctness rule): a
    // discriminating signal may ONLY appear inside an `any_of`, NEVER as a
    // conjunct — a conjunct would require EVERY member to carry it, but a
    // discriminating signal is by definition absent from some member, so a
    // discriminating conjunct would EXCLUDE that member (the draft would fail to
    // bind a site it was generalized from). And the `any_of` over the
    // discriminating signals binds a member only if that member carries ≥1 of its
    // arms. So the disjunction is valid IFF EVERY member has ≥1 discriminating
    // signal. When some member's signals are a SUBSET of the shared core (it has
    // NO discriminating signal of its own), no `any_of` can include it — the
    // discrimination collapses, and the honest generalization is the shared-core
    // conjuncts alone (which all members satisfy). B (the spare-clean gate) is the
    // safety net for the over-binding the collapsed draft may then carry — exactly
    // why C ══ B is non-negotiable (ADR-045): anti-unify-to-disjunction REDUCES
    // autoimmunity where it can, and B catches the residue where it cannot.
    let every_member_has_a_discriminating_signal = features
        .iter()
        .all(|f| f.body_signals.iter().any(|s| discriminating.contains(s)));

    if discriminating.len() >= 2 && every_member_has_a_discriminating_signal {
        let arms: Vec<Constraint> = discriminating
            .into_iter()
            .map(|s| s.to_constraint())
            .collect();
        conjuncts.push(Constraint::AnyOf(arms));
    } else if discriminating.len() == 1 && every_member_has_a_discriminating_signal {
        // A single discriminating signal that EVERY member shares is — by the
        // every-member check — actually shared, so it is safe as a conjunct (the
        // intersection missed it only if a member lacked it, which the check rules
        // out here). A lone arm needs no `any_of` wrapper.
        let only = discriminating.into_iter().next().expect("len == 1");
        conjuncts.push(only.to_constraint());
    }
    // else: the discrimination does not cover every member (a subset member, or no
    // discriminating signal at all) — emit the shared-core conjuncts only; B gates
    // the over-binding.

    Some(Fingerprint {
        constraints: conjuncts,
    })
}

/// Is `draft` **degenerate** (bare-structural)? (ADR-056 §(1) — the C-side
/// non-degeneracy REFUSAL predicate.)
///
/// `true` iff the draft carries NO discriminating signal — only the identity
/// anchors (`Item` / `ImplOfTrait` / `NameMatches`), with no body signal /
/// qualifier / introspection / `any_of`. A degenerate draft over-binds its whole
/// structural family; the generator should not emit it as a promotable hypothesis.
///
/// This is **the SAME predicate** as B's (A)-binary safety check, NOT a parallel
/// implementation — `is_degenerate(d) == !has_discriminating_conjunct(d)` (ADR-056:
/// one predicate, two call-sites, `ParallelStateTrackersDiverge` avoided). It keys
/// on the draft's *constraint shape*, never on *cluster-identity* (`shape_digest`),
/// so it catches **bare-structural only** — it never rejects a twins cluster (twins
/// produce a *precise* draft WITH body conjuncts, which this passes).
#[must_use]
pub fn is_degenerate(draft: &Fingerprint) -> bool {
    !has_discriminating_conjunct(draft)
}

/// The generalization-confidence SIGNAL for a `cluster`/`draft` pair (ADR-056 §(2)).
///
/// A v0.5 ordinal from the cluster's effective diversity — **a signal, never a
/// refusal** (a low-confidence draft is safe; it is tier-capped, not rejected).
///
/// The simplest non-gameable form (ADR-056 OQ2): the discriminator is the presence
/// of a **discriminating `any_of`** in the draft — the anti-unifier emits one IFF
/// the members carried *distinct* distinguishing signals (real diversity), and
/// omits it IFF the members were *effectively identical* (twins/photocopy — every
/// signal shared, the `discriminating` set empty). So:
/// - a discriminating **`any_of`** (members split on ≥2 distinct signals) → `High`
///   (the generalization is exercised by real cluster diversity);
/// - **no `any_of`** (the twins/photocopy / bare-structural shape — generalized from
///   effectively N=1) → `Low`.
///
/// `Moderate` is reserved (a richer per-member edit-distance diversity metric is
/// charter — ADR-056 OQ2: ship the SIMPLEST honest signal first). Note this keys on
/// the draft's *constraint shape*, NOT on `shape_digest` cluster-identity (which
/// under-clusters) — a twins draft has shared body conjuncts but no `any_of`, which
/// is exactly the "generalized from photocopies" tell.
#[must_use]
pub fn generalization_confidence(cluster: &[syn::Item], draft: &Fingerprint) -> Confidence {
    // A cluster smaller than 2 cannot exhibit diversity → Low (conservative).
    if cluster.len() < 2 {
        return Confidence::Low;
    }
    // A discriminating any_of means the members carried distinct signals the
    // anti-unifier split — real diversity. Its absence means the members were
    // effectively identical (twins/photocopy) → generalized from N=1.
    let has_disjunction = draft
        .constraints
        .iter()
        .any(|c| matches!(c, Constraint::AnyOf(arms) if arms.len() >= 2));
    if has_disjunction {
        Confidence::High
    } else {
        Confidence::Low
    }
}

/// Anti-unify `cluster` into a draft AND promote it through B (the spare-clean
/// gate) against `clean_corpus`, minting a [`PromotedDraft`] capability-token.
///
/// This is the **only** path from a cluster to a *promotable* fingerprint (ADR-045,
/// the C ══ B co-ship; ADR-048, the capability-token): the draft is routed through
/// [`self_tolerance::promote_if_safe`], so an `Ok(PromotedDraft)` is structurally
/// guaranteed to have passed ALL THREE of B's gate checks. Returns
/// `Err(ProposeOutcome)` naming exactly why a draft could not be promoted — every
/// non-promotion is legible (ADR-048), never a bare `None`:
/// - [`ProposeOutcome::EmptyCluster`] / [`ProposeOutcome::NoSharedSkeleton`] — the
///   cluster cannot be anti-unified (see [`anti_unify`]);
/// - [`ProposeOutcome::Degenerate`] — **C's non-degeneracy REFUSAL** (ADR-056): the
///   anti-unified draft is bare-structural (over-binds the family), refused *at the
///   generator* before B is consulted (defense-in-depth with B's (A)-binary);
/// - [`ProposeOutcome::Rejected`] — **B's gate refused** (ADR-047): autoimmune
///   (`BindsCleanItem`) or route-to-human (`NotCorpusWitnessable`).
///
/// The caller must treat any `Err(_)` as "no safe promotable draft" — never fall
/// back to asserting the raw [`anti_unify`] output (that bypasses B and ships
/// autoimmunity; ADR-048 makes that bypass a *type* error — the suggestion surfaces
/// accept only a [`PromotedDraft`]).
///
/// (Design note — ADR-056 revision-1: the degenerate REFUSAL lives HERE in the
/// promotion path, not in [`anti_unify`]'s tail. `anti_unify` keeps returning the
/// raw hypothesis [`Fingerprint`] for *inspection* (ADR-048 §Decision: "unchanged.
/// Returns the raw hypothesis"); `propose` is the generator's *promotion* path
/// where the refusal belongs. This reconciles ADR-056 §Mechanics-1 ("the guard at
/// the generator") with ADR-048 (`anti_unify` unchanged) — see the self-ratified
/// ADR-056 revision.)
pub fn propose(
    cluster: &[syn::Item],
    clean_corpus: &[syn::Item],
) -> Result<PromotedDraft, ProposeOutcome> {
    // anti_unify declines an empty OR a heterogeneous (no-shared-skeleton) cluster.
    // Distinguish them so the non-promotion reason is legible.
    let Some(draft) = anti_unify(cluster) else {
        return Err(if cluster.is_empty() {
            ProposeOutcome::EmptyCluster
        } else {
            ProposeOutcome::NoSharedSkeleton
        });
    };
    // C's non-degeneracy REFUSAL (ADR-056): a bare-structural draft over-binds the
    // family — refuse it at the generator with a generator-appropriate diagnostic,
    // before B is consulted (defense-in-depth: B's (A)-binary would also refuse it).
    if is_degenerate(&draft) {
        return Err(ProposeOutcome::Degenerate);
    }
    // Promote through B (ADR-047/048): mints the PromotedDraft iff all three gate
    // checks pass; otherwise the ToleranceVerdict names why (lifted into Rejected).
    self_tolerance::promote_if_safe(draft, clean_corpus).map_err(ProposeOutcome::from)
}

/// One body signal a member's body emits — the syntactic shape a panic source
/// (or any tracked behavior) can take. Distinguishes a *call* from a *macro*
/// because they are different AST nodes the matcher reads with different leaves
/// (`body_calls` vs `body_contains_macro`); keeping them apart means a draft leaf
/// matches a member iff the member really emits THAT shape (a `panic!` macro must
/// not be drafted as `body_calls("panic")`, which would never fire — and vice
/// versa). Ordered (`Call` < `Macro`) so the disjunction arms are deterministic.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum BodySignal {
    /// A function/method call — `.unwrap()`, `std::process::exit()`. Matched by
    /// [`Constraint::BodyCalls`].
    Call(String),
    /// A macro invocation — `panic!`, `todo!`. Matched by
    /// [`Constraint::BodyContainsMacro`].
    Macro(String),
}

impl BodySignal {
    /// The matcher leaf that re-finds this signal (extraction/matching agree).
    fn to_constraint(&self) -> Constraint {
        match self {
            Self::Call(n) => Constraint::BodyCalls(n.clone()),
            Self::Macro(n) => Constraint::BodyContainsMacro(n.clone()),
        }
    }
}

/// The syntactic features extracted from one cluster member, read with the same
/// discipline the matcher uses so a draft leaf matches a member iff the member
/// really carries the feature.
struct MemberFeatures {
    /// The member's item-kind (`None` for item-classes outside the [`ItemKind`]
    /// vocabulary, e.g. a `use` or `macro_rules!` item — such a member has no
    /// shared skeleton and makes [`anti_unify`] decline).
    item_kind: Option<ItemKind>,
    /// The trait's last-segment if the member is an `impl <Trait> for <Type>`;
    /// `None` for a non-impl or inherent impl.
    impl_of_trait: Option<String>,
    /// The set of body signals (calls + macro invocations) the member's body
    /// emits. The same vocabulary `body_calls` / `body_contains_macro` match.
    body_signals: BTreeSet<BodySignal>,
}

impl MemberFeatures {
    fn extract(item: &syn::Item) -> Self {
        Self {
            item_kind: item_kind_of(item),
            impl_of_trait: impl_trait_last_segment(item),
            body_signals: collect_body_signals(item),
        }
    }
}

/// Map a `syn::Item` to its [`ItemKind`] (the vocabulary the `item = <kind>` leaf
/// uses). Returns `None` for item-classes outside that vocabulary.
const fn item_kind_of(item: &syn::Item) -> Option<ItemKind> {
    Some(match item {
        syn::Item::Struct(_) => ItemKind::Struct,
        syn::Item::Enum(_) => ItemKind::Enum,
        syn::Item::Trait(_) => ItemKind::Trait,
        syn::Item::Fn(_) => ItemKind::Fn,
        syn::Item::Impl(_) => ItemKind::Impl,
        syn::Item::Type(_) => ItemKind::Type,
        syn::Item::Mod(_) => ItemKind::Mod,
        syn::Item::Const(_) => ItemKind::Const,
        syn::Item::Static(_) => ItemKind::Static,
        syn::Item::Union(_) => ItemKind::Union,
        _ => return None,
    })
}

/// The last segment of the impl's trait path if `item` is an `impl <Trait> for
/// <Type>` (the same identity `impl_of_trait` reads); `None` otherwise.
fn impl_trait_last_segment(item: &syn::Item) -> Option<String> {
    let syn::Item::Impl(imp) = item else {
        return None;
    };
    let (_, path, _) = imp.trait_.as_ref()?;
    Some(path.segments.last()?.ident.to_string())
}

/// Collect every body signal a function/method body emits — method-call idents,
/// free/path-call last-segments (as [`BodySignal::Call`]), and macro-invocation
/// last-segments (as [`BodySignal::Macro`]) — using the SAME walks `body_calls`
/// and `body_contains_macro` match with, so extraction and matching agree by
/// construction (a draft leaf re-finds the signal this extracted). Returns an
/// empty set for item-classes with no body locus (a marker on a bodyless item
/// contributes no body signal).
fn collect_body_signals(item: &syn::Item) -> BTreeSet<BodySignal> {
    struct SignalCollector {
        signals: BTreeSet<BodySignal>,
    }

    impl<'ast> Visit<'ast> for SignalCollector {
        fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
            if let syn::Expr::Path(p) = call.func.as_ref() {
                if let Some(last) = p.path.segments.last() {
                    self.signals
                        .insert(BodySignal::Call(last.ident.to_string()));
                }
            }
            syn::visit::visit_expr_call(self, call);
        }

        fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
            self.signals
                .insert(BodySignal::Call(call.method.to_string()));
            syn::visit::visit_expr_method_call(self, call);
        }

        fn visit_macro(&mut self, mac: &'ast syn::Macro) {
            if let Some(last) = mac.path.segments.last() {
                self.signals
                    .insert(BodySignal::Macro(last.ident.to_string()));
            }
            syn::visit::visit_macro(self, mac);
        }
    }

    let mut collector = SignalCollector {
        signals: BTreeSet::new(),
    };
    match item {
        syn::Item::Fn(f) => collector.visit_block(&f.block),
        syn::Item::Impl(imp) => {
            for impl_item in &imp.items {
                if let syn::ImplItem::Fn(f) = impl_item {
                    collector.visit_block(&f.block);
                }
            }
        },
        _ => {},
    }
    collector.signals
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items(src: &str) -> Vec<syn::Item> {
        syn::parse_file(src).expect("parses").items
    }

    fn drop_impl_for(items: &[syn::Item], ty: &str) -> syn::Item {
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
                is_drop && p.path.segments.last().is_some_and(|s| s.ident == ty)
            })
            .expect("found")
            .clone()
    }

    const DROP_FAMILY: &str = r#"
        pub struct GuardA;
        impl Drop for GuardA {
            fn drop(&mut self) { let _ = flush(self.h).take().unwrap(); }
        }
        pub struct GuardB;
        impl Drop for GuardB {
            fn drop(&mut self) { let _ = flush(self.h).take().expect("must"); }
        }
        pub struct CleanGuard;
        impl Drop for CleanGuard {
            fn drop(&mut self) { let _ = flush(self.h).take().ok(); }
        }
    "#;

    #[test]
    fn anti_unify_binds_the_cluster() {
        let fam = items(DROP_FAMILY);
        let cluster = vec![drop_impl_for(&fam, "GuardA"), drop_impl_for(&fam, "GuardB")];
        let draft = anti_unify(&cluster).expect("non-empty cluster anti-unifies");
        for (i, m) in cluster.iter().enumerate() {
            assert!(draft.matches(m), "draft must bind cluster member {i}");
        }
    }

    #[test]
    fn anti_unify_spares_the_clean_sibling_via_disjunction() {
        let fam = items(DROP_FAMILY);
        let cluster = vec![drop_impl_for(&fam, "GuardA"), drop_impl_for(&fam, "GuardB")];
        let clean = drop_impl_for(&fam, "CleanGuard");
        let draft = anti_unify(&cluster).expect("anti-unifies");
        // The disjunction (unwrap | expect) is NoMatch on CleanGuard (.ok()),
        // so the whole all_of is NoMatch — spared WITHOUT B even running.
        assert!(
            !draft.matches(&clean),
            "anti-unify-to-disjunction must spare the clean sibling"
        );
    }

    #[test]
    fn anti_unify_keeps_the_shared_call_as_a_conjunct_and_splits_the_rest() {
        let fam = items(DROP_FAMILY);
        let cluster = vec![drop_impl_for(&fam, "GuardA"), drop_impl_for(&fam, "GuardB")];
        let draft = anti_unify(&cluster).expect("anti-unifies");
        // `take` is shared by both → conjunct; `unwrap`/`expect` differ → any_of.
        let has_take_conjunct = draft
            .constraints
            .iter()
            .any(|c| matches!(c, Constraint::BodyCalls(n) if n == "take"));
        assert!(has_take_conjunct, "shared call `take` must be a conjunct");
        let has_disjunction = draft.constraints.iter().any(|c| {
            matches!(c, Constraint::AnyOf(arms) if arms.iter().all(|a|
                matches!(a, Constraint::BodyCalls(n) if n == "unwrap" || n == "expect")))
        });
        assert!(
            has_disjunction,
            "distinguishing calls `unwrap`/`expect` must anti-unify to an any_of"
        );
    }

    #[test]
    fn propose_promotes_only_through_b() {
        let fam = items(DROP_FAMILY);
        let cluster = vec![drop_impl_for(&fam, "GuardA"), drop_impl_for(&fam, "GuardB")];
        let clean_corpus = vec![drop_impl_for(&fam, "CleanGuard")];
        // The cluster's anti-unified draft spares clean AND CleanGuard is a near-miss
        // (matches {impl, Drop, take}, fails only the any_of) → propose mints a token.
        let promoted = propose(&cluster, &clean_corpus).expect("a spare-clean draft promotes");
        let fp = promoted.fingerprint();
        for m in &cluster {
            assert!(fp.matches(m), "promoted draft must bind the cluster");
        }
        assert!(
            !fp.matches(&clean_corpus[0]),
            "promoted draft must spare clean (it came through B)"
        );
    }

    #[test]
    fn propose_returns_err_when_the_draft_binds_clean() {
        // The autoimmune-refusal path (BindsCleanItem) requires BOTH a near-miss
        // (so the gate gets past the near-miss check) AND a bound "clean" item. The
        // corpus carries CleanGuard (a near-miss — matches {impl, Drop, take}, fails
        // only the any_of) AND GuardA mislabeled clean (the draft binds it via the
        // unwrap arm). Near-miss is witnessed by CleanGuard → the gate proceeds to
        // spare-clean → GuardA binds → BindsCleanItem (autoimmune).
        let fam = items(DROP_FAMILY);
        let cluster = vec![drop_impl_for(&fam, "GuardA"), drop_impl_for(&fam, "GuardB")];
        let poisoned_corpus = vec![
            drop_impl_for(&fam, "CleanGuard"), // a real near-miss
            drop_impl_for(&fam, "GuardA"),     // mislabeled clean — the draft binds it
        ];
        assert!(
            matches!(
                propose(&cluster, &poisoned_corpus),
                Err(ProposeOutcome::Rejected(
                    ToleranceVerdict::BindsCleanItem { .. }
                ))
            ),
            "B must refuse to promote a draft that binds a (declared-clean) corpus item"
        );
    }

    #[test]
    fn propose_routes_to_human_when_corpus_has_no_near_miss() {
        // A corpus whose ONLY item the draft BINDS (no spared near-miss) cannot
        // witness the generalization → the gate routes-to-human FIRST (the near-miss
        // check precedes spare-clean, ADR-047 §Mechanics 4). This is the ADR-true
        // verdict for a single bound corpus item: NotCorpusWitnessable, not
        // BindsCleanItem (there is no SPARED clean item to certify the discrimination).
        let fam = items(DROP_FAMILY);
        let cluster = vec![drop_impl_for(&fam, "GuardA"), drop_impl_for(&fam, "GuardB")];
        let only_a_bound_item = vec![drop_impl_for(&fam, "GuardA")];
        assert_eq!(
            propose(&cluster, &only_a_bound_item),
            Err(ProposeOutcome::Rejected(
                ToleranceVerdict::NotCorpusWitnessable
            )),
            "a corpus with no spared near-miss routes-to-human (near-miss check is first)"
        );
    }

    /// The MIXED call/macro panic-family: one member panics via `.unwrap()` (a
    /// call), another via `panic!` (a macro), and both share the same cleanup
    /// structure (`teardown()`). The anti-unifier must produce one disjunction
    /// mixing a `body_calls` arm and a `body_contains_macro` arm — the intended
    /// `any_of([body_calls("unwrap"), body_contains_macro("panic")])` — with the
    /// shared `teardown` call as a conjunct, and the draft must bind both members
    /// AND spare a clean sibling that reaches neither panic shape.
    #[test]
    fn anti_unify_mixes_call_and_macro_arms_in_one_disjunction() {
        let fam = items(
            r#"
            struct One;
            impl Drop for One { fn drop(&mut self) { teardown(); let _ = work().unwrap(); } }
            struct Two;
            impl Drop for Two { fn drop(&mut self) { teardown(); if !work() { panic!("boom"); } } }
            struct Clean;
            impl Drop for Clean { fn drop(&mut self) { teardown(); let _ = work(); } }
        "#,
        );
        let cluster = vec![drop_impl_for(&fam, "One"), drop_impl_for(&fam, "Two")];
        let clean = drop_impl_for(&fam, "Clean");
        let draft = anti_unify(&cluster).expect("mixed family anti-unifies");

        // The disjunction must carry BOTH a call arm (unwrap) and a macro arm (panic).
        let disjunction = draft.constraints.iter().find_map(|c| match c {
            Constraint::AnyOf(arms) => Some(arms),
            _ => None,
        });
        let arms = disjunction.expect("a mixed family produces an any_of disjunction");
        let has_call_arm = arms
            .iter()
            .any(|a| matches!(a, Constraint::BodyCalls(n) if n == "unwrap"));
        let has_macro_arm = arms
            .iter()
            .any(|a| matches!(a, Constraint::BodyContainsMacro(n) if n == "panic"));
        assert!(
            has_call_arm && has_macro_arm,
            "the disjunction must mix body_calls(unwrap) AND body_contains_macro(panic): {arms:?}"
        );
        // The shared cleanup call is a conjunct, not a disjunction arm.
        assert!(
            draft
                .constraints
                .iter()
                .any(|c| matches!(c, Constraint::BodyCalls(n) if n == "teardown")),
            "the shared `teardown` call must be a conjunct"
        );

        for (i, m) in cluster.iter().enumerate() {
            assert!(draft.matches(m), "mixed draft must bind member {i}");
        }
        assert!(
            !draft.matches(&clean),
            "mixed draft must spare the clean sibling (it reaches neither panic shape)"
        );
    }

    #[test]
    fn anti_unify_declines_an_empty_cluster() {
        assert!(
            anti_unify(&[]).is_none(),
            "empty cluster has nothing to generalize"
        );
    }

    #[test]
    fn anti_unify_declines_a_heterogeneous_cluster() {
        // A struct and an impl share no item-kind skeleton → not a real family.
        let mixed = items("struct S; impl Drop for S { fn drop(&mut self) {} }");
        assert!(
            anti_unify(&mixed).is_none(),
            "a cluster with no common item-kind must not produce a shapeless draft"
        );
    }

    // ========================================================================
    // ADR-056 §Q9 — the BORN-RED C-side non-degeneracy guard + generalization-
    // confidence signal spec (Island-2.5's generator half). The refusal catches
    // bare-structural ONLY; the signal handles twins (NEVER the guard).
    // ========================================================================

    /// A cluster of two `Drop` impls whose bodies share NO call/macro signal at all
    /// — the anti-unifier collapses to the bare-structural `all_of([impl,
    /// impl_of_trait("Drop")])` (only identity anchors). The over-binder.
    const BARE_STRUCTURAL_FAMILY: &str = r"
        impl Drop for A { fn drop(&mut self) { self.a = 1; } }
        impl Drop for B { fn drop(&mut self) { self.b = 2; } }
    ";

    /// ADR-056 §Q9 — `anti_unify_refuses_a_degenerate_draft`. A cluster sharing only
    /// `{impl, Drop}` with no body signal yields a degenerate (bare-structural)
    /// draft; `propose` refuses it as `Degenerate` (the over-binder is refused at the
    /// generator, NOT handed to B). `is_degenerate` is the predicate.
    #[test]
    fn anti_unify_refuses_a_degenerate_draft() {
        let fam = items(BARE_STRUCTURAL_FAMILY);
        let cluster = vec![drop_impl_for(&fam, "A"), drop_impl_for(&fam, "B")];
        // anti_unify still EMITS the raw hypothesis (for inspection) — unchanged
        // (ADR-048); it is the bare-structural shape.
        let draft = anti_unify(&cluster).expect("anti-unifies to a bare-structural draft");
        assert!(
            is_degenerate(&draft),
            "a cluster sharing only {{impl, Drop}} yields a degenerate draft: {:?}",
            draft.constraints
        );
        // propose REFUSES it at the generator (ADR-056), before B is consulted.
        let clean = items("impl Drop for Clean { fn drop(&mut self) { log(); } }");
        assert_eq!(
            propose(&cluster, &clean),
            Err(ProposeOutcome::Degenerate),
            "the bare-structural over-binder is refused at the generator (Degenerate)"
        );
    }

    /// ADR-056 §Q9 — `precise_draft_with_discrimination_is_not_degenerate`. A draft
    /// with a `body_calls` conjunct is NOT degenerate (the guard does not brick
    /// precise drafts — only bare-structural ones; the (A)-binary positive mirrored
    /// on the C-side).
    #[test]
    fn precise_draft_with_discrimination_is_not_degenerate() {
        let fam = items(DROP_FAMILY);
        let cluster = vec![drop_impl_for(&fam, "GuardA"), drop_impl_for(&fam, "GuardB")];
        let draft = anti_unify(&cluster).expect("anti-unifies");
        assert!(
            !is_degenerate(&draft),
            "a draft carrying a body_calls conjunct (take) is NOT degenerate: {:?}",
            draft.constraints
        );
    }

    /// ADR-056 §Q9 — `twins_cluster_yields_low_generalization_confidence`. An
    /// identical-twins cluster yields a `Low` confidence (NOT a refusal — the draft
    /// is still produced, just tier-capped), proving the twins case is a SIGNAL not a
    /// refusal. (Twins share all signals → no discriminating `any_of` → Low.)
    #[test]
    fn twins_cluster_yields_low_generalization_confidence() {
        let fam = items(
            r"
            impl Drop for A { fn drop(&mut self) { let _ = flush().unwrap(); } }
            impl Drop for B { fn drop(&mut self) { let _ = flush().unwrap(); } }
        ",
        );
        let cluster = vec![drop_impl_for(&fam, "A"), drop_impl_for(&fam, "B")];
        let draft = anti_unify(&cluster).expect("twins anti-unify to a precise draft");
        // The twins draft IS precise (has flush + unwrap conjuncts) — NOT degenerate.
        assert!(
            !is_degenerate(&draft),
            "a twins draft is precise (has body conjuncts), NOT bare-structural: {:?}",
            draft.constraints
        );
        // But its generalization-confidence is Low (generalized from photocopies —
        // no discriminating diversity / no any_of).
        assert_eq!(
            generalization_confidence(&cluster, &draft),
            Confidence::Low,
            "an identical-twins cluster has Low generalization-confidence (signal, not refusal)"
        );
    }

    /// ADR-056 §Q9 — `diverse_cluster_yields_higher_confidence`. A cluster with
    /// distinct discriminating signals (anti-unified to an `any_of`) yields a higher
    /// confidence than twins — the signal discriminates real diversity from
    /// photocopies.
    #[test]
    fn diverse_cluster_yields_higher_confidence() {
        let fam = items(DROP_FAMILY); // GuardA: unwrap, GuardB: expect → an any_of
        let cluster = vec![drop_impl_for(&fam, "GuardA"), drop_impl_for(&fam, "GuardB")];
        let draft = anti_unify(&cluster).expect("anti-unifies to a disjunction");
        assert_eq!(
            generalization_confidence(&cluster, &draft),
            Confidence::High,
            "a cluster with a discriminating any_of has High generalization-confidence"
        );
        // And it is strictly higher than the twins case.
        assert!(Confidence::High > Confidence::Low);
    }
}
