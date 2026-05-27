//! ATK-ADR029-7 fixture: `#[immune]` must emit a compiler warning when used —
//! ADR-029 §Mechanics requires this to guide authors toward `#[defended_by]`
//! (code-tier) / `#[presents(requires=...)]` (substrate-tier).
//!
//! The migration nudge uses a `const _: () = { #[deprecated] struct ...; let _ = ...; }`
//! pattern rather than `#[deprecated]` on the annotated item. This avoids two defects
//! of the item-level approach:
//!   1. Stacking collision — two `#[immune]` on one item would produce two `#[deprecated]`
//!      attrs, which is a hard compile error ("multiple deprecated attributes").
//!   2. Target mis-match — `#[deprecated]` on the item fires at CALLERS, not at the
//!      author of `#[immune]`.
//!
//! The const-block pattern fires at the `#[immune]` call site (here, line ~25),
//! regardless of how many times it's stacked, and callers of `DefendedSite`
//! see no warning.
//!
//! With `#![deny(deprecated)]` the warning promotes to an error, which is what
//! trybuild matches against.

#![deny(deprecated)]

use antigen_macros::immune;

pub struct SomeAntigen;
pub fn some_witness_fn() {}

// A valid #[immune] invocation (witness= provided; passes validation).
// The const-block nudge fires here at the attribute site.
#[immune(SomeAntigen, witness = some_witness_fn)]
pub struct DefendedSite;

fn main() {
    // Callers of DefendedSite see no deprecation warning — the nudge only
    // fires at the #[immune] attribute call site, not at use sites of the item.
    let _: DefendedSite;
}
