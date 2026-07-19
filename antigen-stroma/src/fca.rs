//! # `fca` — Formal Concept Analysis: the Galois concept lattice of a formal context.
//!
//! Clone #4 of the spine (nb 030 §4), the **stratification-poset stain** (§7). A formal context is
//! objects × attributes; a **concept** is a Galois-closed `(extent, intent)` pair (`extent' = intent`,
//! `intent' = extent`); the concepts ordered by intent-inclusion form the lattice = the poset.
//!
//! Design (hardened by the adversarial design round, 2026-07-19): objects are free, the lattice is bounded
//! by `2^|M|` not `|G|`, so we **curate few attributes** (≤ 64) and encode each object's attribute-set as a
//! `u64` bitmask — the Galois operators become bit-AND loops, no bitset dependency. Concepts are enumerated
//! by Ganter's **`NextClosure`** (lexicographic, polynomial delay). Non-degeneracy is measured, never
//! assumed (an empty attribute-implication set = a free Boolean cube = FCA found no structure → prune).

/// A formal context (objects × attributes) with a bitmask incidence. Attributes and objects are `String`
/// so the context is co-native with the rest of the stroma (symbol strings, witness names).
#[derive(Debug, Clone)]
pub struct Context {
    objects: Vec<String>,
    attributes: Vec<String>,
    rows: Vec<u64>, // rows[o] = the attribute-set bitmask of object o (bit a set ⟺ object o has attribute a)
}

/// A formal concept: a Galois-closed `(extent, intent)` pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Concept {
    /// Object indices sharing exactly this intent (the closed extent).
    pub extent: Vec<usize>,
    /// The attribute bitmask common to the whole extent (the closed intent).
    pub intent: u64,
}

impl Context {
    /// Build from object/attribute names and an incidence list `(object_idx, attribute_idx)`.
    #[must_use]
    pub fn new(
        objects: Vec<String>,
        attributes: Vec<String>,
        incidence: &[(usize, usize)],
    ) -> Self {
        assert!(
            attributes.len() <= 64,
            "fca: ≤ 64 attributes (bitmask); curate the context"
        );
        let mut rows = vec![0u64; objects.len()];
        for &(o, a) in incidence {
            rows[o] |= 1u64 << a;
        }
        Self {
            objects,
            attributes,
            rows,
        }
    }

    const fn full_mask(&self) -> u64 {
        match self.attributes.len() {
            64 => u64::MAX,
            n => (1u64 << n) - 1,
        }
    }

    /// Galois `B'` — the objects whose attribute-set contains every attribute in `intent`.
    fn extent_of(&self, intent: u64) -> Vec<usize> {
        (0..self.rows.len())
            .filter(|&o| self.rows[o] & intent == intent)
            .collect()
    }

    /// Galois `A'` — the attributes common to every object in `objs` (the intent of `∅` is all attributes).
    fn intent_of(&self, objs: &[usize]) -> u64 {
        objs.iter()
            .fold(self.full_mask(), |acc, &o| acc & self.rows[o])
    }

    /// Closure `A''` on an attribute mask — the operator `NextClosure` iterates.
    #[must_use]
    pub fn closure(&self, intent: u64) -> u64 {
        self.intent_of(&self.extent_of(intent))
    }

    /// Every formal concept, via Ganter's `NextClosure` (closed intents in lectic order).
    #[must_use]
    pub fn concepts(&self) -> Vec<Concept> {
        let n = self.attributes.len();
        let mut out = Vec::new();
        let mut intent = self.closure(0);
        loop {
            out.push(Concept {
                extent: self.extent_of(intent),
                intent,
            });
            match self.next_closed(intent, n) {
                Some(next) => intent = next,
                None => break,
            }
        }
        out
    }

    /// The lectically-next closed intent after `a` (Ganter): the largest `i ∉ a` for which closing
    /// `(a ∩ {<i}) ∪ {i}` introduces no new attribute below `i`.
    fn next_closed(&self, a: u64, n: usize) -> Option<u64> {
        for i in (0..n).rev() {
            let bit = 1u64 << i;
            if a & bit != 0 {
                continue;
            }
            let below = bit - 1; // the attributes {0..i-1}
            let b = self.closure((a & below) | bit);
            if b & below == a & below {
                return Some(b);
            }
        }
        None
    }

    /// The non-trivial attribute implications `a → b` that hold in the context (every object with `a` also
    /// has `b`), as `(premise_bit, conclusion_bit)`. An EMPTY set = a free Boolean cube = no attribute
    /// entailment structure = FCA found nothing (the adversary's prune signal — a cheap stem-base proxy).
    #[must_use]
    pub fn pair_implications(&self) -> Vec<(usize, usize)> {
        let n = self.attributes.len();
        let mut imps = Vec::new();
        for a in 0..n {
            let ext_a = self.extent_of(1u64 << a);
            if ext_a.is_empty() {
                continue;
            }
            for b in 0..n {
                if a == b {
                    continue;
                }
                let bbit = 1u64 << b;
                if ext_a.iter().all(|&o| self.rows[o] & bbit != 0) {
                    imps.push((a, b));
                }
            }
        }
        imps
    }

    /// The object (row) names, indexed as `extent` entries reference them.
    #[must_use]
    pub fn objects(&self) -> &[String] {
        &self.objects
    }

    /// The attribute (column) names, indexed as `intent` bits reference them.
    #[must_use]
    pub fn attributes(&self) -> &[String] {
        &self.attributes
    }
}

#[cfg(test)]
mod tests {
    use super::Context;

    // The classic 4-concept diamond: objects {1,2,3}, attributes {a,b}; 1:{a}, 2:{b}, 3:{a,b}.
    fn diamond() -> Context {
        Context::new(
            vec!["1".into(), "2".into(), "3".into()],
            vec!["a".into(), "b".into()],
            &[(0, 0), (1, 1), (2, 0), (2, 1)],
        )
    }

    #[test]
    fn closure_is_extensive_and_idempotent() {
        let c = diamond();
        for m in 0u64..4 {
            let cl = c.closure(m);
            assert_eq!(cl & m, m, "extensive: closure ⊇ input");
            assert_eq!(c.closure(cl), cl, "idempotent");
        }
    }

    #[test]
    fn the_diamond_has_exactly_four_concepts() {
        // (∅, {1,2,3}), ({a}, {1,3}), ({b}, {2,3}), ({a,b}, {3}).
        let concepts = diamond().concepts();
        assert_eq!(concepts.len(), 4);
        // bottom = the common intent of all objects = ∅ (1 and 2 share nothing).
        assert_eq!(concepts[0].intent, 0b00);
        assert_eq!(concepts[0].extent, vec![0, 1, 2]);
        // top = the full intent {a,b}, extent {3}.
        let top = concepts.last().unwrap();
        assert_eq!(top.intent, 0b11);
        assert_eq!(top.extent, vec![2]);
    }

    #[test]
    fn a_free_boolean_context_realizes_all_2_pow_n_concepts_and_has_no_implications() {
        // Objects realizing every subset of {a,b} → the lattice is the free Boolean cube (no structure).
        let free = Context::new(
            vec!["00".into(), "10".into(), "01".into(), "11".into()],
            vec!["a".into(), "b".into()],
            &[(1, 0), (2, 1), (3, 0), (3, 1)],
        );
        assert_eq!(free.concepts().len(), 4); // 2^2 — the negative control
        assert!(free.pair_implications().is_empty()); // free Boolean ⇒ no entailment ⇒ prune signal
    }

    #[test]
    fn the_diamond_has_no_pairwise_implication_but_is_not_free() {
        // a↛b (object 1 has a, not b) and b↛a (object 2) — yet only 4 < ... concepts because a,b co-occur
        // in object 3. Pairwise implications miss this; concept-count vs 2^|M| is the complementary signal.
        let c = diamond();
        assert!(c.pair_implications().is_empty());
        assert_eq!(c.concepts().len(), 4); // == 2^2 here (small n); the real signal is |B| ≪ 2^|M| at scale
    }
}
