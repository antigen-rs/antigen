//! The cfg-set (ADR-067 §A.1 cfg-collision handling).
//!
//! Two items identical except under different cfg are DISTINCT nodes. A cfg-set is a sorted
//! `Vec<CfgAtom>`, canonicalized so equal cfg-sets compare equal as an interned [`crate::node::Locator`]
//! key. The active cfg-set is the one that was in effect when the item was captured.

/// One cfg predicate atom, e.g. `feature = "std"`, `target_os = "linux"`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CfgAtom(pub String);

/// The active cfg-set folded into identity. SORTED for canonical equality (so the interned locator
/// is value-stable across capture order).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct CfgSet(pub Vec<CfgAtom>);

impl CfgSet {
    /// Canonicalize a cfg-set: sort and deduplicate the atoms so two captures of the same active
    /// cfg-set compare equal regardless of capture order.
    #[must_use]
    pub fn canonical(mut atoms: Vec<CfgAtom>) -> Self {
        // Sort + dedup so two captures of the SAME active cfg-set compare bit-equal regardless of
        // capture order — the stability the interned `Locator` key needs (equal cfg-sets must map to
        // the same salsa `Id`). `CfgAtom` derives `Ord` over its inner string, so this is a total,
        // deterministic canonical order.
        atoms.sort();
        atoms.dedup();
        Self(atoms)
    }
}
