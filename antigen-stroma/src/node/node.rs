//! STEP 2b — the `Node` (the salsa `#[input]`). The git-blob half: carries the CHANGING digests.
//!
//! Held + mutated-in-place across edits (VERIFIED: salsa 0.27.2 `tests/mutate_in_place.rs` —
//! `.set_field(&mut db).to(v)` overwrites in place; the input handle's identity PERSISTS). The
//! maintenance pass finds the existing `Node` for an edited item by its stable `Locator` and
//! `.set_identity_digest(...)` it — the SAME entity sees a CHANGED digest → salsa fires the change
//! (the danger signal) + old-vs-new digest is comparable (the backdate key). Re-minting on edit
//! would destroy both — that's the bug aristotle A4/A8 caught.

use super::digest::{IdentityDigest, ShapeDigest};
use super::locator::Locator;

/// A base node — the salsa input handle.
///
/// Keyed by salsa's allocated `Id` (stable across edits because the maintenance pass mutates IN
/// PLACE), carrying the locator + the changing identity values + the contract.
///
/// **STUB note for the builder:** uncomment `#[salsa::input]` + the `'db` lifetime when wiring
/// against `StromaDb`. Each field becomes a salsa input field (settable via `.set_*`).
// #[salsa::input]
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone, Default)]
pub struct Contract {
    // STUB: provides: Vec<...>, requires: Vec<...>, provenance: ResolutionTier
}

/// A maintenance revision stamp. **STUB — frame epoch:** likely a monotonic counter or the salsa
/// `Revision`; on the compose base it is derived-from-mtime, not an authored claim.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Revision(pub u64);
