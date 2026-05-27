//! ATK-ADR029-7 fixture: `#[immune]` must emit a `#[deprecated]` compiler
//! warning when used — ADR-029 §Mechanics requires this to guide
//! users toward the new `#[defended_by]` model.
//!
//! With `#![deny(deprecated)]` in scope, a `#[deprecated]` on the immune macro's
//! output item promotes the warning to an error. The `main()` function below
//! references `DefendedSite` so the deprecation fires at a real use site.
//!
//! This fixture must fail to compile because `immune()` wraps the output item
//! with `#[deprecated(note = "...")]` and main() uses the resulting type.

#![deny(deprecated)]

use antigen_macros::immune;

pub struct SomeAntigen;
pub fn some_witness_fn() {}

// A valid #[immune] invocation (witness= provided; passes existing validation).
// immune() wraps DefendedSite with #[deprecated]; using it below triggers the error.
#[immune(SomeAntigen, witness = some_witness_fn)]
pub struct DefendedSite;

fn main() {
    // Use the deprecated item so the deprecation-as-error fires.
    let _: DefendedSite;
}
