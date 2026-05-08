//! W6a fixture: `#[antigen_tolerance(X, rationale = "..", until = "")]`
//! must reject. Per ADR-011 §Mechanics §2 (aristotle reciprocal Phase 1-8):
//! empty `until` indicates user error.

use antigen_macros::antigen_tolerance;

pub struct PolarityInvertedClassMeet;

#[antigen_tolerance(
    PolarityInvertedClassMeet,
    rationale = "Test fixture deliberately constructs the failure pattern.",
    until = ""
)]
fn deliberately_constructed_failure() {}

fn main() {}
