//! Adversarial fixture: `category = []` must be rejected with a clear error.
//! An empty category array is meaningless and should not parse.

use antigen_macros::antigen;

#[antigen(name = "test-antigen", fingerprint = "item = struct", category = [])]
pub struct EmptyCategory;

fn main() {}
