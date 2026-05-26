//! W2 fixture: `#[antigen]` applied to a generic unit struct must reject with a
//! clear error, not a confusing "missing generics in anonymous const" compile error.
//!
//! Antigen marker structs are failure-class identity tokens — they carry no data
//! and no type-level parameterization. A generic antigen struct is semantically
//! meaningless (which failure class does `Foo<T>` name?), and the current use-token
//! emission (`let _x: Foo;` without type params) produces a cryptic E0107 error
//! pointing at the struct declaration, not at the usage of generics.
//!
//! Desired: the macro should detect generic parameters and emit a clear
//! `#[antigen] must be applied to a non-generic unit struct` error.

use antigen_macros::antigen;

#[antigen(name = "generic-atk", fingerprint = r#"name = matches("*")"#)]
pub struct GenericAntigenAtk<T: std::fmt::Debug>;

fn main() {}
