//! STEP 5 — the cfg-set (ADR-067 §A.1 cfg-collision handling). (Also referenced by STEP 2a id.)
//!
//! Two items identical except under different cfg are DISTINCT nodes. **STUB decisions for the
//! builder (adversarial GAP-2):** (a) representation = sorted `Vec<CfgAtom>` (canonical, so equal
//! cfg-sets compare equal as an interned `Locator` key); (b) the active cfg-set comes from
//! cargo-metadata features at scan time; (c) the key encodes which cfg-set was active at capture.

/// One cfg predicate atom, e.g. `feature = "std"`, `target_os = "linux"`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CfgAtom(pub String);

/// The active cfg-set folded into identity. SORTED for canonical equality (so the interned locator
/// is value-stable across capture order).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct CfgSet(pub Vec<CfgAtom>);

impl CfgSet {
    /// **STUB — fill (frame epoch):** sort + dedup the atoms so equal cfg-sets are bit-equal.
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
