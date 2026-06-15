//! P3b compile-fail: `ratified_doc(min_version = "")` must reject at compile time
//! (NFA-15: empty min_version vacuously passes the version floor for any versioned doc).
//!
//! The `requires =` predicate grammar is shared; ADR-029 folds it onto
//! `#[presents]` (the former `#[immune(requires = ...)]` carrier was removed).

use antigen_macros::presents;

pub struct DummyAntigen;

#[presents(DummyAntigen, requires = ratified_doc(min_version = ""))]
pub struct PresentsEmptyMinVersion;

fn main() {}
