//! STEP 2b — the STABLE LOCATOR (the salsa key). The git-path half of the locator/identity split.
//!
//! `#[salsa::interned(no_lifetime, debug)]`: value-equality → stable `Id`; `no_lifetime` drops the
//! phantom `'db` lifetime parameter so `LocatorId` is `'static`-usable as a field on
//! `#[salsa::input]` (which cannot take lifetime parameters). Verified vs salsa 0.27.2
//! `tests/interned-structs.rs:49`. API CORRECTION: salsa 0.27 has NO `#[id]` key-field on
//! `#[salsa::input]` — the stable-locator intent is expressed via `#[salsa::interned]`
//! (keyed-BY-VALUE), not a key field on the input.

// salsa's `#[interned]`/`#[input]` macros generate the `new` constructor + field accessors as
// associated fns that cannot carry doc comments. The public TYPES and their FIELDS are documented;
// this allow only covers the unavoidable macro-generated surface.
#![allow(missing_docs)]

use super::cfg::CfgSet;

/// The stable address of a node — interned so equal (path, cfg) values map to the same salsa `Id`.
/// An item at `crate::module::foo` stays the same `Locator` across edits to its body.
///
/// Constructed via `Locator::new(&db, fq_path, cfg_set)` — value-equality gives a stable Id that
/// survives body edits (the locator carries no digest, only the path+cfg coordinate).
#[salsa::interned(no_lifetime, debug)]
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
