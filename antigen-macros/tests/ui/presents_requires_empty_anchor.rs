//! P3b compile-fail: `ratified_doc(anchor = "")` must reject at compile time
//! (NFA-14: empty anchor vacuously passes str::contains("") for any doc content).
//!
//! The `requires =` predicate grammar is shared; ADR-029 folds it onto
//! `#[presents]` (the former `#[immune(requires = ...)]` carrier was removed).

use antigen_macros::presents;

pub struct DummyAntigen;

#[presents(DummyAntigen, requires = ratified_doc(anchor = ""))]
pub struct PresentsEmptyAnchor;

fn main() {}
