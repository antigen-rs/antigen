//! W2 fixture: `#[antigen(name = "", ...)]` must reject with a
//! "name cannot be empty" diagnostic. Anchored to `validate()` in
//! antigen-macros::parse::AntigenArgs.

use antigen_macros::antigen;

#[antigen(name = "", fingerprint = "x")]
pub struct EmptyName;

fn main() {}
