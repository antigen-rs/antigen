//! Adversarial fixture: `category = AntigenCategory::Hybrid` must be rejected.
//! Only SubstrateAlignment and FunctionalCorrectness are valid variants.

use antigen_macros::antigen;

#[antigen(name = "test-antigen", fingerprint = "item = struct", category = AntigenCategory::Hybrid)]
pub struct UnknownCategoryVariant;

fn main() {}
