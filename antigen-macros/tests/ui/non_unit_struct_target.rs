//! W2 fixture: `#[antigen]` applied to a non-unit struct must reject.
//! Per the macro's contract in antigen-macros::lib::antigen — antigens
//! are unit-only declarations (the *type* is the antigen identity; data
//! fields would imply runtime state, which violates ADR-001's
//! zero-runtime-cost commitment).

use antigen_macros::antigen;

#[antigen(name = "non-unit", fingerprint = r#"name = matches("*")"#)]
pub struct NonUnit(pub u32);

fn main() {}
