//! W2 fixture: `#[antigen(name = "FooBar", ...)]` must reject with a
//! "must be kebab-case" diagnostic. The kebab-case discipline lives in
//! `is_kebab_case` (antigen-macros::parse) and is the named anchor for
//! cross-crate antigen registry consistency (ADR-009 references field).

use antigen_macros::antigen;

#[antigen(name = "FooBar", fingerprint = "x")]
pub struct NonKebabName;

fn main() {}
