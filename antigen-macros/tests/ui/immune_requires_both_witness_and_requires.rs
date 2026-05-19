//! P3b compile-fail: providing both `witness = ...` and `requires = ...` must
//! reject with a diagnostic directing the user to `witnesses = [...]` for
//! compound evidence (ADR-019 §F11).

use antigen_macros::immune;

pub struct DummyAntigen;
pub fn my_test() {}

#[immune(DummyAntigen, witness = my_test, requires = fresh_within_days(90))]
pub struct ImmuneDoubleEvidence;

fn main() {}
