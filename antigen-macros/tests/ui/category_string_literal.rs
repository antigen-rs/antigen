//! Adversarial fixture: `category = "substrate-alignment"` must be rejected.
//! The category field requires a path expression (AntigenCategory::X),
//! NOT a string literal, for compile-time discoverability.

use antigen_macros::antigen;

#[antigen(name = "test-antigen", fingerprint = "item = struct", category = "substrate-alignment")]
pub struct StringLiteralCategory;

fn main() {}
