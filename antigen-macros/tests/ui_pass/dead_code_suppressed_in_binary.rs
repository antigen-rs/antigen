//! Pass-fixture (DX finding 1): a marker struct declared with `#[antigen]` in a
//! binary-like crate (this file has a `fn main`, no external API surface) must
//! compile cleanly under `#![deny(dead_code)]`.
//!
//! Before the fix, `#[antigen] pub struct Foo;` here tripped `dead_code`
//! because `pub` does not exempt items in a crate with no API surface and the
//! marker type is never constructed. The macro now emits a zero-cost use-token
//! (`const _: fn() = || { let _x: Foo; };`) so the type counts as used. This
//! fixture is the regression guard: if the use-token is ever removed, this
//! file fails to compile and the pass-harness goes red.

#![deny(dead_code)]

use antigen_macros::antigen;

#[antigen(name = "binary-marker", fingerprint = "item = struct")]
pub struct BinaryMarker;

fn main() {}
