//! STEP 2b — the `Node` (the salsa `#[input]`). The git-blob half: carries the CHANGING digests.
//!
//! Held + mutated-in-place across edits (VERIFIED: salsa 0.27.2 `tests/mutate_in_place.rs` —
//! `.set_field(&mut db).to(v)` overwrites in place; the input handle's identity PERSISTS). The
//! maintenance pass finds the existing `Node` for an edited item by its stable `Locator` and
//! `.set_identity_digest(...)` it — the SAME entity sees a CHANGED digest → salsa fires the change
//! (the danger signal) + old-vs-new digest is comparable (the backdate key). Re-minting on edit
//! would destroy both — that's the bug aristotle A4/A8 caught.

// salsa's `#[input]` macro generates `new` + field accessors + setters as associated fns that cannot
// carry doc comments. The public type and its fields ARE documented; this allow covers only the
// macro-generated surface.
#![allow(missing_docs)]

use super::digest::{IdentityDigest, ShapeDigest};
use super::locator::Locator;

/// A base node — the salsa input handle.
///
/// Keyed by salsa's allocated `Id` (stable across edits because the maintenance pass mutates IN
/// PLACE), carrying the locator + the changing identity values + the contract.
///
/// Construction: `Node::new(&mut db, locator, identity_digest, shape_digest, kind, contract, last_changed)`.
/// Mutation: `node.set_identity_digest(&mut db).to(new_digest)` (and so on for other fields).
/// Access: `node.identity_digest(&db)`, `node.locator(&db)`, etc.
#[salsa::input]
#[derive(Debug)]
pub struct Node {
    /// The stable interned locator (a value, but value-stable — survives body edits).
    pub locator: Locator,
    /// Collision-resistant signing digest — changes on ANY edit (the danger signal).
    pub identity_digest: IdentityDigest,
    /// FNV backdate/clustering key — changes on a MEANING edit.
    pub shape_digest: ShapeDigest,
    /// The node-kind vocabulary, REUSED from scan (`antigen::scan::ItemTarget`). It is the matching
    /// discriminant, NOT the node-id.
    pub kind: antigen::scan::ItemTarget,
    /// The local contract: provides/requires + provenance-tier (see [`Contract`]).
    pub contract: Contract,
    /// Maintenance-stamped last-changed revision (ADR-067 LE6 — a `#[salsa::input]` field, NEVER a
    /// tracked-fn). See WATCH/GAP-13: on the COMPOSE base this is recomputable-from-mtime; the
    /// authority-arbiter tension is a sovereign-side concern, out of frame scope.
    pub last_changed: Revision,
}

/// A node's local contract (provides/requires + provenance), ADR-067 §A.3.
/// **STUB — fill (frame epoch):** the provides/requires relation shape.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Contract {
    // STUB: provides: Vec<...>, requires: Vec<...>, provenance: ResolutionTier
}

/// A maintenance revision stamp (the observed fs-mtime, ADR-067 LE6).
///
/// On the COMPOSE base this is derived-from-mtime, not an authored claim. Its concurrent-write merge
/// is the §4.5a ruling: a deterministic, monotone, idempotent, order-independent join — see
/// [`Revision::merge`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Revision(pub u64);

impl Revision {
    /// The §4.5a concurrent-write merge: take the LATER timestamp (`max`, most-recent-knowledge wins).
    ///
    /// Both writers derive the value from the SAME fs source, so whichever `set` wins LAST under
    /// salsa's write-serialization is correct REGARDLESS of order. This collapses the general
    /// `SovereignMerge` authority-arbiter to a monotone join for this fs-derived field: it is
    /// commutative (`merge(a,b) == merge(b,a)`), idempotent (`merge(a,a) == a`), and never regresses
    /// backward (a stale observed mtime cannot pull `last_changed` earlier). The maintenance `set`
    /// must use this — `node.set_last_changed(&mut db).to(current.merge(observed))` — never a blind
    /// overwrite-with-older.
    #[must_use]
    pub fn merge(self, observed: Self) -> Self {
        Self(self.0.max(observed.0))
    }
}
