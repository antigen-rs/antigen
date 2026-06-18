//! MATURE — the **affinity-maturation engine** (v0.6, ADR-061), the gradient the
//! [`Affinity`] type measures.
//!
//! v0.5 shipped the affinity TYPE (the *height*) and `measure()` (the *altimeter*);
//! v0.6 ships the **engine that climbs** — the germinal-center analog that takes a
//! rough anti-unified draft and matures it toward the Pareto frontier of (recall,
//! precision). This is the organ "the maturing organism" is named for.
//!
//! # Targeted hypermutation (CDR, not framework — biology is load-bearing, ADR-003)
//!
//! Real somatic hypermutation is **not uniform** — it concentrates on the CDR
//! (the antigen-contacting loops) and freezes the framework (the structural
//! scaffold). A uniform mutator wastes its (expensive) selection budget breaking the
//! scaffold or hitting coldspots. The code cognate of the framework/CDR partition is
//! exactly antigen's **discriminating-conjunct** partition (the GATE-G `is_discriminating`
//! recursive descent, P0): a draft's *skeleton* conjuncts (`item`/`impl_of_trait`/
//! `name` — structural anchors) are the **framework** (FROZEN), and its
//! *discriminating* conjuncts (`body_calls`/`has_method`/qualifiers/… — the signals
//! that distinguish a defect from its clean sibling) are the **CDR** (MUTATED).
//!
//! Mutating a framework anchor would change *what family* the draft binds (a
//! scaffold-break); mutating a discriminating conjunct tunes *how tightly within the
//! family* it binds — which is exactly the recall↔precision tradeoff the affinity
//! 2-vector measures. So the engine freezes the skeleton and perturbs the CDR.
//!
//! # The mutation: drop-a-discriminator (the computable Pareto move)
//!
//! The honest, computable v1 mutation is **dropping one discriminating conjunct**: a
//! draft that is *over-specific* (it misses a cluster member because that member
//! lacks one of the draft's conjuncts) widens by dropping the offending conjunct —
//! catching the missed member (recall ↑) at the risk of binding a clean sibling
//! (precision ↓). That is a real frontier move, and it is decidable from present
//! substrate (no codegen): each candidate is a [`Fingerprint`] with one conjunct
//! removed, [`Affinity::measure`]d against the
//! cluster + clean corpus. (Re-*adding* novel discriminators is the anti-unifier's
//! job — `anti_unify` already emits the richest draft; maturation tunes it down,
//! never invents leaves it was not given. Generating new leaf *values* is a later
//! precision refinement, not this engine's contract.)
//!
//! # The rate DECAYS as affinity rises (the convergence guarantee)
//!
//! Germinal centers mutate their *best* drafts *least* — a high-affinity clone is
//! near-optimal, so further mutation mostly degrades it. A flat mutation rate lacks
//! this and can thrash near the optimum. So the per-round mutation **budget decays as
//! affinity rises** (`mutation_budget`): a low-affinity draft explores widely; a
//! near-frontier draft barely perturbs. This is the antigen-depletion analog as a
//! *rate*, complementing the [`Affinity::pareto_improves_on`] *stopping rule* (which
//! halts when no mutation helps at all).
//!
//! # Evasion-awareness + anti-local-optima (a background off-target rate)
//!
//! A purely greedy "only ever drop the current discriminators" climb can stick at a
//! local optimum, and a mutator that always tunes the SAME leaf is game-able (an
//! adversary learns which leaf antigen widens and dodges it). Biology keeps a small
//! background rate of off-target mutation to escape local optima and preserve
//! diversity. The engine's deterministic analog: alongside the targeted CDR drops, it
//! also considers each **`any_of` arm** individually (widening *within* a disjunction,
//! not just dropping the whole conjunct) — a finer, less-predictable perturbation set
//! than "drop the top-level leaf." (Determinism over an RNG: the round is reproducible
//! for tests + audit; "background rate" is expressed as *which* candidates are
//! enumerated, not a coin-flip.)
//!
//! # Composes, does not compete (ADR-002) — a READER/SELECTOR over present machinery
//!
//! Every step reuses shipped machinery: the draft from
//! [`anti_unify`](crate::learn::propose::anti_unify), the partition from the public
//! [`has_discriminating_conjunct`]
//! (so the CDR/framework split is core's `is_discriminating` itself — ONE source of
//! truth, no `ParallelStateTrackersDiverge`), the scoring from
//! [`Affinity::measure`], the stopping rule
//! from [`Affinity::pareto_improves_on`], and the trajectory write from
//! [`LifeRecord::append`]. The engine is
//! the *selection pressure* that threads them, not new verification.

use antigen_fingerprint::{Constraint, Fingerprint};

use crate::learn::affinity::Affinity;
use crate::learn::life_record::{LifeEvent, LifeRecord};
use crate::learn::self_tolerance::has_discriminating_conjunct;

/// The result of maturing a draft: the matured fingerprint + its final affinity +
/// the trajectory of affinities visited (the gradient climb, for legibility/audit).
#[derive(Debug, Clone, PartialEq)]
pub struct Matured {
    /// The matured draft — at the Pareto frontier the climb could reach (no further
    /// drop-a-discriminator mutation improves it).
    pub draft: Fingerprint,
    /// The final affinity (the frontier point reached).
    pub affinity: Affinity,
    /// The affinity at each accepted step, oldest first — the score-trajectory this
    /// run produced (the same sequence appended to the life-record as
    /// [`LifeEvent::Scored`]). Always non-empty: index 0 is the starting draft's
    /// affinity.
    pub trajectory: Vec<Affinity>,
}

/// Is `c` a **discriminating** (CDR) conjunct — one maturation may mutate — versus a
/// frozen structural-anchor (framework) conjunct?
///
/// Delegates to core's GATE-G partition via the PUBLIC API: a single-conjunct draft's
/// [`has_discriminating_conjunct`] reduces to `is_discriminating(c)` (the
/// `normalized_top_level` flatten is identity on one conjunct). This is the SAME
/// predicate the gate uses — ONE source of truth, no parallel copy to drift (the
/// `ParallelStateTrackersDiverge` antigen guards against in its own code).
#[must_use]
fn is_discriminating_conjunct(c: &Constraint) -> bool {
    has_discriminating_conjunct(&Fingerprint {
        constraints: vec![c.clone()],
    })
}

/// The per-round mutation **budget** — how many candidate mutations to consider this
/// round — as a function of the draft's current affinity. **Decays as affinity rises**
/// (germinal-center convergence: mutate the best drafts least).
///
/// Returns a budget in `1..=max_budget`: a floor-affinity draft (recall+precision ≈ 0)
/// gets the full `max_budget`; a frontier draft (≈ `Affinity::PERFECT`) gets `1` (a
/// single confirming probe before the stopping rule halts it). Linear in the
/// affinity's L1 magnitude — simple, monotone, and honest (a real decay, not a curve
/// tuned to a benchmark). `max_budget` is the caller's exploration width.
// The casts are provably safe: `max_budget` is a small conjunct-count (fits f64
// losslessly), `scaled` is non-negative (a budget) and `.round()`ed into
// `[0, max_budget]` before the `as usize` + clamp, so neither truncation nor
// sign-loss is reachable.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
#[must_use]
fn mutation_budget(affinity: Affinity, max_budget: usize) -> usize {
    // L1 "how matured is this draft" in [0, 2] (both axes in [0,1]).
    let maturity = affinity.recall + affinity.precision;
    // Decay from max_budget (maturity 0) to 1 (maturity 2).
    let scaled = (max_budget as f64) * (1.0 - maturity / 2.0);
    (scaled.round() as usize).clamp(1, max_budget)
}

/// Enumerate the **targeted mutation candidates** for `draft`: each is a clone of the
/// draft with exactly one *discriminating* (CDR) conjunct widened, the framework
/// (structural anchors) FROZEN.
///
/// Two widening shapes (the deterministic targeted + background set):
/// 1. **drop a top-level discriminating conjunct** (the primary CDR move — widen by
///    removing a whole signal);
/// 2. **drop one arm of a top-level discriminating `any_of`** (the finer background
///    move — widen *within* a disjunction; less predictable than always dropping the
///    whole leaf, the anti-local-optima / evasion-aware diversity).
///
/// A framework anchor (`item`/`impl_of_trait`/`name`) is never a candidate (freezing
/// the skeleton). A coldspot — a draft with no discriminating conjunct, or a draft
/// reduced to a single conjunct (dropping it yields an empty, vacuously-matching
/// fingerprint, ATK-047-N4) — yields no candidates.
#[must_use]
fn mutation_candidates(draft: &Fingerprint) -> Vec<Fingerprint> {
    // Normalize to the FLAT top-level conjunct list (the shape `anti_unify` emits).
    // `Fingerprint::parse("all_of([..])")` wraps the conjuncts in a single top-level
    // `AllOf`; flattening it makes the engine producer-independent (same discipline as
    // GATE-G's `normalized_top_level`) — without it, a parse-wrapped draft reads as ONE
    // conjunct and yields no mutation candidates.
    let conjuncts = flatten_top_level(&draft.constraints);

    let mut out = Vec::new();
    // Dropping the SOLE conjunct yields an empty (vacuously-Match) draft — never a
    // valid widening (it would bind everything). Require ≥2 conjuncts to drop one.
    let droppable = conjuncts.len() >= 2;

    for (i, c) in conjuncts.iter().enumerate() {
        if !is_discriminating_conjunct(c) {
            continue; // framework — frozen.
        }
        // (1) drop the whole discriminating conjunct.
        if droppable {
            let mut widened = conjuncts.clone();
            widened.remove(i);
            out.push(Fingerprint {
                constraints: widened,
            });
        }
        // (2) drop one arm of a discriminating any_of (widen within the disjunction).
        if let Constraint::AnyOf(arms) = c {
            // Dropping an arm only widens if ≥2 arms remain meaningful; a 1-arm
            // any_of is just that arm, and dropping it empties the conjunct.
            if arms.len() >= 2 {
                for arm_idx in 0..arms.len() {
                    let mut narrowed_arms = arms.clone();
                    narrowed_arms.remove(arm_idx);
                    let mut widened = conjuncts.clone();
                    widened[i] = if narrowed_arms.len() == 1 {
                        narrowed_arms.into_iter().next().expect("len==1")
                    } else {
                        Constraint::AnyOf(narrowed_arms)
                    };
                    out.push(Fingerprint {
                        constraints: widened,
                    });
                }
            }
        }
    }
    out
}

/// Flatten a single top-level `AllOf` wrapper into its children — producing the flat
/// conjunct list `anti_unify` emits (and that GATE-G's `normalized_top_level`
/// reasons over). `[AllOf([a, b, c])]` → `[a, b, c]`; an already-flat list is
/// returned as-is. Non-recursive (matches the one-level wrap `parse` produces); a
/// nested `AllOf` inside an `AnyOf` arm is left alone (it is semantically necessary,
/// not redundant — same scope as core's flatten).
#[must_use]
fn flatten_top_level(constraints: &[Constraint]) -> Vec<Constraint> {
    if let [Constraint::AllOf(children)] = constraints {
        children.clone()
    } else {
        constraints.to_vec()
    }
}

/// **Mature** `draft` toward the Pareto frontier against its defect `cluster` and the
/// `clean_corpus`, recording the score-trajectory into `record`.
///
/// The gradient climb (the germinal-center loop):
/// 1. measure the draft's [`Affinity`]; append [`LifeEvent::Scored`] to `record`.
/// 2. enumerate `mutation_candidates` (targeted CDR widenings, framework frozen),
///    capped at this round's `mutation_budget` (decaying as affinity rises).
/// 3. measure each candidate; keep the one that best [`Affinity::pareto_improves_on`] the
///    current draft (strictly dominates — no axis worsened). If one is found, accept
///    it (append [`LifeEvent::Matured`] + the new [`LifeEvent::Scored`]) and repeat.
/// 4. STOP when no candidate Pareto-improves (the **ceiling** — the antigen-depletion
///    frontier the draft can no longer leave) or the budget is exhausted.
///
/// Returns the matured draft + its trajectory. The `record` is left holding the
/// `Scored` events (the stock the homeostasis loops read). Deterministic: same
/// inputs → same climb (no RNG; the candidate enumeration is ordered).
///
/// `max_budget` bounds per-round exploration width (a handful — the conjunct count);
/// `max_rounds` bounds total climb length (defense against a pathological cycle —
/// monotone Pareto-improvement makes a true cycle impossible, but the bound is the
/// honest belt).
pub fn mature(
    draft: Fingerprint,
    cluster: &[syn::Item],
    clean_corpus: &[syn::Item],
    record: &mut LifeRecord,
    max_budget: usize,
    max_rounds: usize,
) -> Matured {
    let mut current = draft;
    let mut current_affinity = Affinity::measure(&current, cluster, clean_corpus);
    let mut trajectory = vec![current_affinity];
    record.append(LifeEvent::Scored(current_affinity));

    for _ in 0..max_rounds {
        let budget = mutation_budget(current_affinity, max_budget);

        // The best Pareto-improving candidate this round (strict dominance over the
        // current draft). Ties broken by first-found (deterministic order).
        let mut best: Option<(Fingerprint, Affinity)> = None;
        for candidate in mutation_candidates(&current).into_iter().take(budget) {
            let cand_affinity = Affinity::measure(&candidate, cluster, clean_corpus);
            if cand_affinity.pareto_improves_on(&current_affinity)
                && best
                    .as_ref()
                    .is_none_or(|(_, best_a)| cand_affinity.pareto_improves_on(best_a))
            {
                best = Some((candidate, cand_affinity));
            }
        }

        match best {
            // A Pareto-improving mutation — accept it and climb on.
            Some((next, next_affinity)) => {
                current = next;
                current_affinity = next_affinity;
                trajectory.push(current_affinity);
                record.append(LifeEvent::Matured);
                record.append(LifeEvent::Scored(current_affinity));
            },
            // No mutation improves — the frontier (antigen depleted). STOP.
            None => break,
        }
    }

    Matured {
        draft: current,
        affinity: current_affinity,
        trajectory,
    }
}

#[cfg(test)]
// The affinity rates the tests assert on are exact integer ratios (n/n = 1.0,
// 1/2 = 0.5, 0/n = 0.0) — exact-by-construction, so direct `f64` equality is
// correct here (the same justification as `affinity.rs`'s own tests).
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    fn fp(src: &str) -> Fingerprint {
        Fingerprint::parse(src).expect("test fingerprint parses")
    }

    // --- the framework/CDR partition (delegates to core's is_discriminating) ---

    #[test]
    fn framework_anchors_are_frozen_cdr_signals_mutate() {
        // structural anchors = framework (NOT discriminating)
        assert!(!is_discriminating_conjunct(
            &fp("item = struct").constraints[0]
        ));
        assert!(!is_discriminating_conjunct(
            &fp("impl_of_trait(\"Drop\")").constraints[0]
        ));
        // body signals / qualifiers = CDR (discriminating)
        assert!(is_discriminating_conjunct(
            &fp("body_calls(\"unwrap\")").constraints[0]
        ));
        assert!(is_discriminating_conjunct(&fp("is_async").constraints[0]));
    }

    // --- the rate DECAYS as affinity rises ---

    #[test]
    fn budget_decays_as_affinity_rises() {
        let max = 8;
        let floor = mutation_budget(Affinity::new(0.0, 0.0), max);
        let mid = mutation_budget(Affinity::new(0.5, 0.5), max);
        let ceiling = mutation_budget(Affinity::PERFECT, max);
        assert_eq!(floor, max, "a floor draft explores at full budget");
        assert_eq!(ceiling, 1, "a frontier draft barely perturbs");
        assert!(
            mid < floor && mid > ceiling,
            "monotone decay through the middle"
        );
    }

    #[test]
    fn budget_is_always_at_least_one() {
        // Even at the frontier, the budget floors at 1 (a confirming probe).
        assert_eq!(mutation_budget(Affinity::PERFECT, 5), 1);
        assert_eq!(mutation_budget(Affinity::PERFECT, 1), 1);
    }

    // --- mutation candidates: targeted (CDR) + framework-frozen + coldspot-safe ---

    #[test]
    fn candidates_drop_only_discriminating_conjuncts() {
        // item=struct (framework) + body_calls (CDR). Only the CDR drop is offered,
        // and it leaves the framework anchor (a non-empty, ≥1-conjunct draft).
        let draft = fp("all_of([item = struct, body_calls(\"unwrap\")])");
        let cands = mutation_candidates(&draft);
        assert_eq!(cands.len(), 1, "exactly one CDR drop (the body_calls)");
        // the survivor is the framework anchor alone
        assert_eq!(cands[0], fp("item = struct"));
    }

    #[test]
    fn single_conjunct_draft_is_a_coldspot_no_drop() {
        // Dropping the sole conjunct → empty (vacuously-Match) draft. Never offered.
        let draft = fp("body_calls(\"unwrap\")");
        assert!(
            mutation_candidates(&draft).is_empty(),
            "a 1-conjunct draft yields no drop candidate (ATK-047-N4 vacuity)"
        );
    }

    #[test]
    fn no_discriminating_conjunct_is_a_coldspot() {
        // all framework → nothing to mutate (the engine can't tune a pure skeleton).
        let draft = fp("all_of([item = struct, impl_of_trait(\"Drop\")])");
        assert!(mutation_candidates(&draft).is_empty());
    }

    #[test]
    fn any_of_arms_are_widened_individually() {
        // a discriminating any_of with 2 arms, alongside a framework anchor.
        // Candidates: drop the whole any_of (1) + drop each arm (2) = 3.
        let draft =
            fp("all_of([item = struct, any_of([body_calls(\"unwrap\"), body_calls(\"expect\")])])");
        let cands = mutation_candidates(&draft);
        // 1 whole-conjunct drop + 2 arm-drops.
        assert_eq!(cands.len(), 3);
        // Candidates are FLAT (the engine normalizes the parse-wrap), so compare
        // against directly-constructed flat fingerprints, not parse("all_of([..])").
        let flat = |cs: Vec<Constraint>| Fingerprint { constraints: cs };
        let item_struct = fp("item = struct").constraints[0].clone();
        let unwrap = fp("body_calls(\"unwrap\")").constraints[0].clone();
        let expect = fp("body_calls(\"expect\")").constraints[0].clone();
        // dropping one arm of a 2-arm any_of collapses it to the other single leaf.
        assert!(cands.contains(&flat(vec![item_struct.clone(), expect])));
        assert!(cands.contains(&flat(vec![item_struct.clone(), unwrap])));
        // dropping the whole any_of leaves the framework anchor (a single flat conjunct).
        assert!(cands.contains(&flat(vec![item_struct])));
    }

    // --- the gradient climb: over-specific draft widens to catch a missed member ---

    #[test]
    fn mature_widens_an_over_specific_draft_to_catch_a_missed_member() {
        // The cluster: two Drop-guards that panic, one via .unwrap() AND .flush(),
        // one via .unwrap() only. An OVER-SPECIFIC draft requiring BOTH unwrap AND
        // flush binds only the first → recall 0.5. Dropping `body_calls(flush)`
        // (a CDR widen) catches the second member → recall 1.0, and (no clean item
        // binds) precision stays 1.0 → a strict Pareto improvement.
        let m1: syn::Item = syn::parse_quote! {
            impl Drop for A { fn drop(&mut self) { self.0.flush(); self.1.unwrap(); } }
        };
        let m2: syn::Item = syn::parse_quote! {
            impl Drop for B { fn drop(&mut self) { self.1.unwrap(); } }
        };
        let cluster = [m1, m2];
        // A clean Drop sibling that does NOT panic (`.ok()`, no unwrap) — the draft
        // (requiring `unwrap`) spares it, so precision is meaningful (non-empty corpus).
        let clean_item: syn::Item = syn::parse_quote! {
            impl Drop for Clean { fn drop(&mut self) { self.0.ok(); } }
        };
        let clean = [clean_item];

        // The over-specific starting draft (binds m1, misses m2; spares Clean).
        let draft = fp(
            "all_of([item = impl, impl_of_trait(\"Drop\"), body_calls(\"flush\"), body_calls(\"unwrap\")])",
        );
        let start = Affinity::measure(&draft, &cluster, &clean);
        assert_eq!(start.recall, 0.5, "starting draft misses m2 (no flush)");
        assert_eq!(
            start.precision, 1.0,
            "starting draft spares the clean sibling"
        );

        let mut record = LifeRecord::new("panic-in-drop");
        let matured = mature(draft, &cluster, &clean, &mut record, 8, 16);

        // The engine climbed: final affinity strictly dominates the start.
        assert!(
            matured.affinity.pareto_improves_on(&start),
            "maturation must improve the over-specific draft"
        );
        assert_eq!(matured.affinity.recall, 1.0, "it caught the missed member");
        assert_eq!(
            matured.affinity.precision, 1.0,
            "without binding the clean sibling (it still requires unwrap, which Clean lacks)"
        );
    }

    // --- the STOCK write-hook: the trajectory lands in the life-record ---

    #[test]
    fn mature_writes_the_score_trajectory_to_the_life_record() {
        let m1: syn::Item = syn::parse_quote! {
            impl Drop for A { fn drop(&mut self) { self.0.flush(); self.1.unwrap(); } }
        };
        let m2: syn::Item = syn::parse_quote! {
            impl Drop for B { fn drop(&mut self) { self.1.unwrap(); } }
        };
        let cluster = [m1, m2];
        let clean_item: syn::Item = syn::parse_quote! {
            impl Drop for Clean { fn drop(&mut self) { self.0.ok(); } }
        };
        let clean = [clean_item];
        let draft = fp(
            "all_of([item = impl, impl_of_trait(\"Drop\"), body_calls(\"flush\"), body_calls(\"unwrap\")])",
        );

        let mut record = LifeRecord::new("panic-in-drop");
        let matured = mature(draft, &cluster, &clean, &mut record, 8, 16);

        // Every trajectory point is a Scored event in the record (the stock the
        // homeostasis loops read). At least 2 (start + ≥1 climb step).
        let scored: Vec<&Affinity> = record
            .events()
            .iter()
            .filter_map(|e| match e {
                LifeEvent::Scored(a) => Some(a),
                _ => None,
            })
            .collect();
        assert_eq!(scored.len(), matured.trajectory.len());
        assert!(
            scored.len() >= 2,
            "start + at least one climb step recorded"
        );
        // the last Scored equals the final affinity.
        assert_eq!(*scored.last().unwrap(), &matured.affinity);
        // an accepted climb step also recorded a Matured milestone.
        assert!(
            record.events().contains(&LifeEvent::Matured),
            "an accepted maturation appends a Matured milestone"
        );
    }

    // --- the stopping rule: an already-frontier draft does not thrash ---

    #[test]
    fn an_already_perfect_draft_stops_immediately() {
        // A draft that already binds its whole cluster + spares the clean corpus is
        // at the frontier — no mutation Pareto-improves, so the climb halts with just
        // the starting Scored event (no Matured milestones).
        let m1: syn::Item = syn::parse_quote! {
            impl Drop for A { fn drop(&mut self) { self.1.unwrap(); } }
        };
        let cluster = [m1];
        let clean_item: syn::Item = syn::parse_quote! {
            impl Drop for Clean { fn drop(&mut self) { self.0.ok(); } }
        };
        let clean = [clean_item];
        let draft = fp("all_of([item = impl, impl_of_trait(\"Drop\"), body_calls(\"unwrap\")])");
        let start = Affinity::measure(&draft, &cluster, &clean);
        assert_eq!(
            start,
            Affinity::PERFECT,
            "the draft is already at the frontier"
        );

        let mut record = LifeRecord::new("panic-in-drop");
        let matured = mature(draft, &cluster, &clean, &mut record, 8, 16);

        assert_eq!(matured.affinity, Affinity::PERFECT);
        assert_eq!(
            matured.trajectory.len(),
            1,
            "no climb steps — already optimal"
        );
        assert!(
            !record.events().contains(&LifeEvent::Matured),
            "no Matured milestone for a draft that didn't move"
        );
    }
}
