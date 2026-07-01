//! The stable LOCATOR (the salsa key). The git-path half of the locator/identity split.
//!
//! `#[salsa::interned(no_lifetime, debug)]`: value-equality gives a stable `Id`; `no_lifetime` drops
//! the phantom `'db` lifetime parameter so the interned id is `'static`-usable as a field on a
//! `#[salsa::input]` (which cannot take lifetime parameters). The stable-locator intent is expressed
//! via `#[salsa::interned]` (keyed by value), not a key field on the input.

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
///
/// # What it does not do
///
/// Entity-identity holds across BODY edits only, not across a rename or move. A rename changes the
/// `fq_path`, so the interned `Locator` is a new value — salsa sees delete-old + create-new, not an
/// in-place edit. Comparing a node to its pre-rename self (backdate/danger-compare across a rename)
/// is not something the locator supports; re-homing across a rename is a lifecycle-layer concern.
#[salsa::interned(no_lifetime, debug)]
pub struct Locator {
    /// `crate::mod::item` (syntactic-tier) or the SCIP symbol (resolved-tier). See [`super::path`].
    pub fq_path: String,
    /// The snapshot's active cfg (canonical/sorted — see [`CfgSet`]).
    pub cfg_set: CfgSet,
}
