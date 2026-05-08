//! W6a fixture: `#[antigen_tolerance(X, bogus = "y")]` must reject with a
//! useful error naming the unknown field.

use antigen_macros::antigen_tolerance;

pub struct PolarityInvertedClassMeet;

#[antigen_tolerance(
    PolarityInvertedClassMeet,
    rationale = "Test fixture",
    bogus = "y"
)]
fn deliberately_constructed_failure() {}

fn main() {}
