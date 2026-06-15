//! P3b compile-fail: `signers(required = [])` must reject at compile time
//! (NFA-7: empty required list is a semantic no-op that vacuously passes).
//!
//! The `requires =` predicate grammar is shared; ADR-029 folds it onto
//! `#[presents]` (the former `#[immune(requires = ...)]` carrier was removed).

use antigen_macros::presents;

pub struct DummyAntigen;

#[presents(DummyAntigen, requires = signers(required = []))]
pub struct PresentsEmptySigners;

fn main() {}
