//! W6a fixture: `#[antigen_tolerance(X)]` (no `rationale`) must reject.
//! Per ADR-011 §Mechanics §1: empty/absent rationale rejected at parse time
//! — a tolerance without rationale is not a claim, it's silent suppression.

use antigen_macros::antigen_tolerance;

pub struct PolarityInvertedClassMeet;

#[antigen_tolerance(PolarityInvertedClassMeet)]
fn deliberately_constructed_failure() {}

fn main() {}
