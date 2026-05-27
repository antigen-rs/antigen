//! Adversarial example: declares an immunity with a witness identifier that
//! does NOT resolve to a real function. `cargo antigen audit` MUST flag this.
//!
//! This is the failing-as-passing pattern at the project level: the example
//! "fails" (the witness is broken) and the audit "passes" by correctly
//! identifying the failure.
//!
//! Run:
//!
//! ```sh
//! cargo run --bin cargo-antigen -- antigen audit --root antigen/examples
//! ```
//!
//! Expected output: 1 broken witness (`nonexistent_test`).

#![allow(dead_code)]
// This example intentionally demonstrates the (deprecated, still-functional)
// #[immune] API — broken_witness exists to show #[immune]'s broken-witness
// failure mode, so it must keep using #[immune]. ADR-029 deprecates #[immune];
// the deprecation warning is allowed here deliberately.
#![allow(deprecated)]

use antigen::{antigen, immune, presents};

#[antigen(
    name = "demo-broken-witness",
    family = "boundary-violation",
    // Narrowed fingerprint per A3.5 onboarding sweep: scoped to types
    // beginning with `Looks` so the example doesn't pollute the workspace
    // scan with cross-reactive hits on every named item.
    //
    // The original fingerprint was `name = matches("*")` — a deliberately-
    // trivial pattern that satisfied #[antigen]'s required-field constraint
    // (ADR-009 Layer 1) but matched everything in the workspace. That
    // produced 7 cross-reactive hits in `basic.rs` and similar noise
    // elsewhere — a poor first encounter for new users running `cargo
    // antigen scan` on the examples directory.
    //
    // Lesson for fingerprint authors: prefer the narrowest shape that
    // captures the failure class. "matches('*')" never does — real
    // antigens recognize structural patterns, not "everything." See
    // `basic.rs`'s `PanickingInDrop` fingerprint for a recall-tuned
    // example that uses `body_contains_macro(...)` to narrow.
    fingerprint = r#"name = matches("Looks*")"#,
    summary = "Demonstrates audit catching a broken witness identifier."
)]
pub struct DemoBrokenWitness;

/// Demonstration type that LOOKS immune to `DemoBrokenWitness` but isn't —
/// the witness function the immunity claim names doesn't actually exist.
pub struct LooksImmuneButIsnt {
    inner: String,
}

#[presents(DemoBrokenWitness)]
#[immune(
    DemoBrokenWitness,
    witness = nonexistent_test,
    rationale = "This witness function does not actually exist; audit should catch it."
)]
impl LooksImmuneButIsnt {
    /// Create a new (empty) instance for the demonstration.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            inner: String::new(),
        }
    }
}

impl Default for LooksImmuneButIsnt {
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    println!("antigen audit demo: see broken witness in action.");
    println!("Run `cargo run --bin cargo-antigen -- antigen audit --root antigen/examples`");
}
