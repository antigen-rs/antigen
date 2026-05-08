//! W6a fixture: `#[antigen_tolerance(X, rationale = "")]` (empty) must
//! reject. Per ADR-011 §Mechanics §1.

use antigen_macros::antigen_tolerance;

pub struct PolarityInvertedClassMeet;

#[antigen_tolerance(PolarityInvertedClassMeet, rationale = "")]
fn deliberately_constructed_failure() {}

fn main() {}
