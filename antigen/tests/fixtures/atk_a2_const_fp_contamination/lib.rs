// Fingerprint contamination fixture.
// Two consts with different values — both have #[presents].
// If fingerprints are contaminated (carrying the preceding struct's digest),
// they will be IDENTICAL to each other because the struct hasn't changed.
// If fingerprints correctly capture each const's own content, they will DIFFER.

use antigen::presents;

pub struct BoundaryViolation;

#[presents(BoundaryViolation)]
pub const SMALL_LIMIT: usize = 1024;

#[presents(BoundaryViolation)]
pub const LARGE_LIMIT: usize = 65536;
