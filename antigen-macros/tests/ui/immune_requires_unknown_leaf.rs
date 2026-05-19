//! P3b compile-fail: `requires = <unknown_leaf>` must reject with a diagnostic
//! naming the unknown leaf and listing the v0.1 sealed set (ADR-019 T4-R).

use antigen_macros::immune;

pub struct DummyAntigen;

#[immune(DummyAntigen, requires = doc_attested(path = "discipline.md"))]
pub struct ImmuneUnknownLeaf;

fn main() {}
