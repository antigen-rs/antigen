//! # `rough` — Pawlak rough-set approximation over an indiscernibility partition.
//!
//! An instrument that reads only a COARSE feature of each element (syn sees a symbol's BARE NAME, not
//! which of several same-named symbols it is) induces an *indiscernibility relation*: two elements are
//! indiscernible when that feature agrees. A target CONCEPT then splits three ways under the partition:
//!
//! - **lower** — classes wholly inside the concept: a member is CERTAINLY in.
//! - **upper** — classes that touch the concept: a member MIGHT be in (`lower ⊆ upper`).
//! - **boundary** — `upper − lower`: classes that STRADDLE the concept edge — undiscernible at this
//!   resolution. The boundary is the tier's ignorance made into a measurable *set*.
//!
//! `accuracy = |lower| / |upper| ∈ [0,1]` grades how well the feature separates the concept. A **singleton
//! class** (a distinctive feature value) can never straddle — always fully in or fully out — so
//! distinctiveness ⇒ certainty. This is the exact structure of the fidelity tier: a coarser instrument
//! has a bigger boundary, and refining the instrument (syn → SCIP) shrinks it.
//!
//! It is a build-queue hypothesis, not a settled import: the primitive is proven here in the abstract; the
//! [`rough_phantom`](../../examples/rough_phantom.rs) example measures whether it *binds* to antigen's
//! phantom residue (does the boundary explain the census's collision inflation?) or stays a naming rhyme.

use std::collections::{BTreeSet, HashMap};

/// The three-way rough decomposition of a concept relative to an indiscernibility partition.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Approximation {
    /// Elements in classes WHOLLY inside the concept — certain members.
    pub lower: BTreeSet<String>,
    /// Elements in classes that TOUCH the concept — possible members (`lower ⊆ upper`).
    pub upper: BTreeSet<String>,
    /// `upper − lower`: elements the instrument's resolution CANNOT classify.
    pub boundary: BTreeSet<String>,
}

impl Approximation {
    /// Pawlak accuracy `|lower| / |upper| ∈ [0,1]` — 1.0 when the boundary is empty, and 1.0 by
    /// convention for an empty concept (nothing to be uncertain about).
    #[must_use]
    #[allow(clippy::cast_precision_loss)] // class counts are small; f64 is exact well past our scale
    pub fn accuracy(&self) -> f64 {
        if self.upper.is_empty() {
            1.0
        } else {
            self.lower.len() as f64 / self.upper.len() as f64
        }
    }
}

/// Compute the rough approximation of `concept` under the indiscernibility relation induced by
/// `class_of` (each element → its class key), over the elements of `universe`.
///
/// A class lands in `lower` when every member is in the concept, in `upper` when any member is; the
/// `boundary` is the set difference. Elements of the universe not in the concept but sharing a class
/// with a concept member are pulled into `upper`/`boundary` — that is the whole point (they are the
/// undiscernible neighbours the coarse feature cannot separate out).
#[must_use]
pub fn approximate<'a, U, K>(universe: U, concept: &BTreeSet<String>, class_of: K) -> Approximation
where
    U: IntoIterator<Item = &'a str>,
    K: Fn(&str) -> String,
{
    let mut classes: HashMap<String, Vec<String>> = HashMap::new();
    for element in universe {
        classes
            .entry(class_of(element))
            .or_default()
            .push(element.to_string());
    }
    let (mut lower, mut upper) = (BTreeSet::new(), BTreeSet::new());
    for members in classes.values() {
        if members.iter().any(|m| concept.contains(m)) {
            upper.extend(members.iter().cloned());
            if members.iter().all(|m| concept.contains(m)) {
                lower.extend(members.iter().cloned());
            }
        }
    }
    let boundary = upper.difference(&lower).cloned().collect();
    Approximation {
        lower,
        upper,
        boundary,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{Approximation, approximate};

    fn concept(items: &[&str]) -> BTreeSet<String> {
        items.iter().map(|s| (*s).to_string()).collect()
    }
    // Class = the leading letter, so "a1" and "a2" are indiscernible; "b1" is its own class.
    fn first_letter(e: &str) -> String {
        e[..1].to_string()
    }

    #[test]
    fn a_class_wholly_inside_the_concept_is_lower_with_no_boundary() {
        let approx = approximate(["a1", "a2", "b1"], &concept(&["a1", "a2"]), first_letter);
        assert!(approx.lower.contains("a1") && approx.lower.contains("a2"));
        assert!(approx.boundary.is_empty()); // the "a" class does not straddle
        assert!((approx.accuracy() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn a_class_straddling_the_concept_is_boundary_not_lower() {
        // Concept holds only a1, but a2 is indiscernible from it → the whole "a" class is boundary.
        let approx = approximate(["a1", "a2"], &concept(&["a1"]), first_letter);
        assert!(approx.lower.is_empty());
        assert!(approx.upper.contains("a1") && approx.upper.contains("a2"));
        assert_eq!(approx.boundary, concept(&["a1", "a2"]));
        assert!((approx.accuracy() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn a_distinctive_singleton_class_can_never_straddle() {
        // Each element is its own class → a distinctive feature fully determines membership.
        let approx = approximate(["a", "b", "c"], &concept(&["a", "c"]), |e| e.to_string());
        assert_eq!(approx.lower, concept(&["a", "c"]));
        assert!(approx.boundary.is_empty());
        assert!((approx.accuracy() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn an_empty_concept_is_vacuously_accurate() {
        let approx = approximate(["a1", "a2"], &BTreeSet::new(), first_letter);
        assert_eq!(approx, Approximation::default());
        assert!((approx.accuracy() - 1.0).abs() < f64::EPSILON);
    }
}
