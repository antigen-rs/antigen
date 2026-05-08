//! W2 fixture: `#[immune(X)]` (no `witness = ...`) must reject with a
//! diagnostic stating that immunity claims require a witness. ADR-005
//! sub-clause F: every trust boundary requires a validation check; a
//! marker without proof is not a claim.

use antigen_macros::immune;

pub struct DummyAntigen;

#[immune(DummyAntigen)]
pub struct ImmuneNoWitness;

fn main() {}
