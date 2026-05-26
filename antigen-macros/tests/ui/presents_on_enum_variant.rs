// Compile-fail fixture: `#[presents]` (a proc-macro attribute) on an enum
// VARIANT does not compile. Rust forbids proc-macro attribute macros in variant
// position — variants accept only inert/built-in attributes (`#[cfg]`,
// `#[deprecated]`, doc). This is a hard COMPILE error, not a doc-build-only one.
//
// This fixture is the durable, CI-enforced proof of the placement rule: mark
// the enum TYPE (or a newtype), never a variant directly. A scanner/parse test
// (`syn::parse_file`) accepts variant-attribute *syntax* and so cannot witness
// this — only an actual compile can, which is exactly what trybuild runs here.

use antigen_macros::presents;

pub struct BoundaryViolation;

pub enum RequestKind {
    #[presents(BoundaryViolation)]
    External { payload: Vec<u8> },
    Internal { payload: Vec<u8> },
}

fn main() {}
