//! The `Node` (the salsa `#[input]`). The git-blob half: carries the CHANGING digests.
//!
//! Held and mutated in place across edits: `.set_field(&mut db).to(v)` overwrites in place and the
//! input handle's identity persists. The maintenance pass finds the existing `Node` for an edited
//! item by its stable `Locator` and `.set_identity_digest(...)` it — the same entity sees a changed
//! digest, so salsa fires the change (the danger signal) and the old-vs-new digest is comparable
//! (the backdate key). Re-minting a node on edit would destroy both, which is why identity is keyed
//! on the stable locator, not the digest.

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
    /// Maintenance-stamped last-changed revision (ADR-067 LE6 — a `#[salsa::input]` field, not a
    /// tracked fn). On the compose base this is recomputable from fs-mtime; see [`Revision::merge`]
    /// for its concurrent-write join.
    pub last_changed: Revision,
}

/// A node's local contract — its provides/requires relation and provenance tier (ADR-067 §A.3).
///
/// Empty at this layer: the base carries the contract slot on every node, and the
/// provides/requires/provenance fields are populated by the datalog-closure layer that reconstructs
/// the relation. A default `Contract` is the honest "no contract reconstructed yet" value.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Contract {
    // provides / requires / provenance fields are added when the closure layer reconstructs them.
}

/// A maintenance revision stamp (the observed fs-mtime, ADR-067 LE6).
///
/// On the compose base this is derived from fs-mtime, not an authored claim. Its concurrent-write
/// merge is a deterministic, monotone, idempotent, order-independent join — see [`Revision::merge`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Revision(pub u64);

impl Revision {
    /// The concurrent-write merge: take the LATER timestamp (`max`, most-recent-knowledge wins).
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
