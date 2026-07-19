//! # `soft` — soft sets: objects carrying a parameter-set (Molodtsov), the shape every witness-family shares.
//!
//! A soft set is `F: E → P(U)` — each PARAMETER selects a subset of the universe. Transposed (the
//! *object-view* we actually store), each OBJECT carries the set of parameters it satisfies — which is
//! exactly [`EdgeFacts`](crate::base::facts::EdgeFacts) (edge → `{WitnessSource}`),
//! [`AbsenceFacts`](crate::base::facts::AbsenceFacts) (subject → `{AbsenceWitness}`), and the output of the
//! [`verdict`](crate::verdict) join. Soft-set theory names the uniform algebra over that shape:
//!
//! - **restriction** — objects satisfying ≥ k parameters ([`SoftSet::corroborated`]): corroboration IS a
//!   soft-set restriction (k = 2 reproduces [`AbsenceFact::is_corroborated`](crate::base::facts::AbsenceFact::is_corroborated)).
//! - **transpose** — object-view ↔ parameter-view ([`SoftSet::parameter_view`]): the join direction, made
//!   generic (it is the same operation for every witness family).
//! - **union** — merge two families of the SAME parameter kind ([`SoftSet::union`]): two instrument
//!   batches, or two temporal snapshots, of one witness family.
//!
//! It is a build-queue clone (nb 030), and an *organizing* one: soft-set makes no measurable prediction the
//! way rough sets do — it binds as **structure/algebra**, not as a measurement. The cross-family `(T, I, F)`
//! assembly (presence-witnesses × absence-witnesses) is a soft-set *product* of DIFFERENT parameter kinds —
//! a struct, not one generic `SoftSet<P>`; that product with `(T,I,F)`-tagged parameters is the
//! *neutrosophic-soft set*, the weave point of the spine.

use std::collections::{BTreeMap, BTreeSet};

/// A soft set in object-view: each object → the set of parameters it satisfies. Generic over the parameter
/// kind `P` (a witness type) with `String` objects (subjects / edges).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SoftSet<P: Ord + Clone> {
    objects: BTreeMap<String, BTreeSet<P>>,
}

impl<P: Ord + Clone> SoftSet<P> {
    /// Build from the object-view: each object mapped to the parameters it satisfies.
    #[must_use]
    pub const fn new(objects: BTreeMap<String, BTreeSet<P>>) -> Self {
        Self { objects }
    }

    /// The object-view map (each object → its parameter set) — how `EdgeFacts`/`AbsenceFacts` store.
    #[must_use]
    pub const fn objects(&self) -> &BTreeMap<String, BTreeSet<P>> {
        &self.objects
    }

    /// Objects satisfying at least `k` parameters — the corroboration restriction (`k = 2` = corroborated
    /// by two independent instruments). This is the general form of every per-family corroboration check.
    #[must_use]
    pub fn corroborated(&self, k: usize) -> BTreeSet<String> {
        self.objects
            .iter()
            .filter(|(_, params)| params.len() >= k)
            .map(|(obj, _)| obj.clone())
            .collect()
    }

    /// Parameter-view `F: E → P(U)` (Molodtsov's original form) — the transpose of the object-view.
    /// Each parameter maps to the set of objects that satisfy it.
    #[must_use]
    pub fn parameter_view(&self) -> BTreeMap<P, BTreeSet<String>> {
        let mut view: BTreeMap<P, BTreeSet<String>> = BTreeMap::new();
        for (obj, params) in &self.objects {
            for p in params {
                view.entry(p.clone()).or_default().insert(obj.clone());
            }
        }
        view
    }

    /// Soft-set union over the same parameter kind: each object's parameter set becomes the union across
    /// both operands. (Merging two instrument batches, or two temporal snapshots, of one witness family.)
    #[must_use]
    pub fn union(&self, other: &Self) -> Self {
        let mut merged = self.objects.clone();
        for (obj, params) in &other.objects {
            merged
                .entry(obj.clone())
                .or_default()
                .extend(params.iter().cloned());
        }
        Self::new(merged)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::SoftSet;
    use crate::base::facts::{AbsenceFact, AbsenceWitness};

    fn soft_of(pairs: &[(&str, &[u8])]) -> SoftSet<u8> {
        let objects = pairs
            .iter()
            .map(|(o, ps)| ((*o).to_string(), ps.iter().copied().collect()))
            .collect();
        SoftSet::new(objects)
    }

    #[test]
    fn corroboration_is_the_at_least_k_restriction() {
        let soft = soft_of(&[("a", &[1, 2]), ("b", &[1]), ("c", &[1, 2, 3])]);
        assert_eq!(
            soft.corroborated(2),
            ["a", "c"].iter().map(ToString::to_string).collect()
        );
    }

    #[test]
    fn parameter_view_is_the_transpose() {
        // object-view {a:{1,2}, b:{2}} transposes to parameter-view {1:{a}, 2:{a,b}}.
        let pv = soft_of(&[("a", &[1, 2]), ("b", &[2])]).parameter_view();
        assert_eq!(pv[&1], ["a"].iter().map(ToString::to_string).collect());
        assert_eq!(pv[&2], ["a", "b"].iter().map(ToString::to_string).collect());
    }

    #[test]
    fn union_merges_parameters_per_object_across_families() {
        let merged = soft_of(&[("a", &[1])]).union(&soft_of(&[("a", &[2]), ("b", &[3])]));
        assert_eq!(
            merged.objects()["a"],
            [1, 2].into_iter().collect::<BTreeSet<u8>>()
        );
        assert_eq!(
            merged.objects()["b"],
            [3].into_iter().collect::<BTreeSet<u8>>()
        );
    }

    #[test]
    fn absence_facts_are_a_soft_set_and_corroboration_is_the_restriction() {
        // The bind, concretely: AbsenceFacts ARE a soft set, and its restriction == AbsenceFact::is_corroborated.
        let facts = vec![
            AbsenceFact {
                subject: "a".into(),
                witnesses: BTreeSet::from([AbsenceWitness::DeadCode, AbsenceWitness::NotExecuted]),
            },
            AbsenceFact {
                subject: "b".into(),
                witnesses: BTreeSet::from([AbsenceWitness::DeadCode]),
            },
        ];
        let objects: BTreeMap<String, BTreeSet<AbsenceWitness>> = facts
            .iter()
            .map(|f| (f.subject.clone(), f.witnesses.clone()))
            .collect();
        let soft = SoftSet::new(objects);

        let corroborated = soft.corroborated(2);
        assert_eq!(corroborated.contains("a"), facts[0].is_corroborated());
        assert_eq!(corroborated.contains("b"), facts[1].is_corroborated());
        assert!(corroborated.contains("a") && !corroborated.contains("b"));
    }
}
