//! P3b compile-fail: `requires = <unknown_leaf>` must reject with a diagnostic
//! naming the unknown leaf and listing the v0.1 sealed set (ADR-019 T4-R).
//!
//! The `requires =` predicate grammar is shared; ADR-029 folds it onto
//! `#[presents]` (the former `#[immune(requires = ...)]` carrier was removed).

use antigen_macros::presents;

pub struct DummyAntigen;

#[presents(DummyAntigen, requires = doc_attested(path = "discipline.md"))]
pub struct PresentsUnknownLeaf;

fn main() {}
