//! CE-2 fixture: `#[clonal]` with `seed = SeedKind::Fixed(42)` must compile-fail.
//!
//! Per ADR-024 adversarial C2: SeedKind::Fixed(_) in #[clonal] is REJECTED.
//! A fixed seed means iterations are NOT independent; the claim "independent
//! iterations" is a contradiction.
//!
//! Per CLAUDE.md and the ADR: this is PARSE-TIME enforcement (same pattern as
//! immunosuppress duration-cap). Using Fixed → compile error, not audit hint.
//!
//! This fixture locks the compile error wording for CE-2.

use antigen_macros::clonal;

/// ATK-CE-2: Fixed seed bypasses iteration independence claim.
#[clonal(
    witness = my_property_test,
    iterations = 1000,
    seed = SeedKind::Fixed(42)
)]
pub fn arithmetic_sum(a: i64, b: i64) -> Option<i64> {
    a.checked_add(b)
}

fn my_property_test() {}

fn main() {}
