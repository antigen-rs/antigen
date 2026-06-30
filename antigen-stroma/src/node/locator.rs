//! STEP 2b — the STABLE LOCATOR (the salsa key). The git-path half of the locator/identity split.
//!
//! `#[salsa::interned]`: value-equality → stable `Id`. Survives BODY edits (the locator is path+cfg,
//! NOT the digest). API CORRECTION (implementer, verified vs salsa 0.27.2): salsa 0.27 has NO `#[id]`
//! key-field on `#[salsa::input]` — the stable-locator intent is expressed via `#[salsa::interned]`
//! (keyed-BY-VALUE), not a key field on the input.

use super::cfg::CfgSet;

/// The stable address of a node — interned so equal (path, cfg) values map to the same salsa `Id`.
/// An item at `crate::module::foo` stays the same `Locator` across edits to its body.
///
/// **STUB note for the builder:** uncomment the salsa attribute and the `'db` lifetime when wiring
/// against `StromaDb`. Kept attribute-free in the skeleton so the module tree reads without a salsa
/// build. Fields must be salsa-storable (interned data types).
// #[salsa::interned]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Locator {
    /// `crate::mod::item` (syntactic-tier) or the SCIP symbol (resolved-tier). See [`super::path`].
    pub fq_path: String,
    /// The snapshot's active cfg (canonical/sorted — see [`CfgSet`]).
    pub cfg_set: CfgSet,
}

// NAMED LIMITATION (adversarial A2): a RENAME or MOVE of an item changes its fq_path, so the
// interned Locator is a NEW value → salsa sees delete-old + create-new, NOT an in-place edit. The
// in-place-mutation property (entity-identity across edits) holds only for BODY edits, NOT
// rename/move. This is acceptable at the frame's batch cadence (a rename re-constitutes the node),
// but the builder must NOT assume rename preserves entity-identity. The backdate/danger-compare
// across a rename is a known gap — a future move-detector (locator-similarity) is named-deferred.
