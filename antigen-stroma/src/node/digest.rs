//! STEP 2a — the two-digest split (ADR-067 §A.2). CONFIG/OUTPUT applied to the digest.
//!
//! Two DISTINCT types so one field can never name both:
//! - [`IdentityDigest`] — BLAKE3, collision-RESISTANT, signing tier. The integrity half. FNV-1a is
//!   engineer-collidable and NOT admissible for identity (the born-red FNV-collision ATK proves it).
//! - [`ShapeDigest`] — FNV-1a (reused from `antigen-fingerprint`), cheap-recomputable, the
//!   clustering + near-miss + ADR-068 clause-7 BACKDATE key. Strips name; drift-allowed.
//!
//! **DIFFERENT strip-sets** (aristotle A6 / adversarial GAP-3): `IdentityDigest` KEEPS semantic attrs
//! (signing); `ShapeDigest` STRIPS name (clustering/backdate). Do not unify the preimages.

/// Collision-resistant signing digest (BLAKE3).
///
/// Preimage = canonicalized item tokens ONLY (the implementer's lean, pending adr-reviewer): path +
/// cfg are SIBLING identity fields, not folded in, keeping this a pure function of the item's own
/// bytes (recomputable, parity-guardable).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdentityDigest(pub [u8; 32]);

impl IdentityDigest {
    /// **STUB — fill (frame epoch):** BLAKE3 over canonicalized item tokens.
    /// New dependency `blake3` (declared in Cargo.toml; verified present in the registry cache).
    #[must_use]
    pub fn of_tokens(_canonical_tokens: &[u8]) -> Self {
        todo!("frame epoch: BLAKE3 signing digest over canonical item tokens")
    }
}

/// Fast shape digest (FNV-1a). Reuse `antigen_fingerprint`'s `structural_shape_digest` — name-
/// insensitive, the clustering/backdate key. Strips `ANTIGEN_OWNED_ATTRS` (the maintained strip-list).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShapeDigest(pub String);

impl ShapeDigest {
    /// **STUB — fill (frame epoch):** delegate to `antigen_fingerprint`'s FNV shape digest. Do NOT
    /// reimplement — the strip-list completeness is guarded there (`digest_strip_list_completeness`).
    #[must_use]
    pub fn of_item(_item_tokens: &str) -> Self {
        todo!(
            "frame epoch: delegate to antigen_fingerprint::structural_shape_digest (FNV, name-stripped)"
        )
    }
}
