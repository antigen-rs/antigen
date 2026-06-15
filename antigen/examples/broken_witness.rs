//! Adversarial example: a site declares it *presents* a failure-class but no
//! witness ever defends it. `cargo antigen audit` MUST flag this as undefended.
//!
//! This is the failing-as-passing pattern at the project level: the example
//! "fails" (the defense is missing) and the audit "passes" by correctly
//! identifying the gap.
//!
//! Run:
//!
//! ```sh
//! cargo run --bin cargo-antigen -- antigen audit --root antigen/examples
//! ```
//!
//! Expected output: the `LooksImmuneButIsnt` presents-site reported as
//! **undefended** — no `#[defended_by(DemoBrokenWitness)]` test registers a
//! defense for it.
//!
//! ## ADR-029 note (immunity is observed, not declared)
//!
//! The old `#[immune(X, witness = a_test)]` form — which let the site *name* its
//! own witness, so a witness identifier could be broken (point at a function
//! that doesn't exist) — was removed (ADR-029). Defense is now *observed* by the
//! audit: a `#[defended_by(X)]` test registers what it defends, and the audit
//! cross-references it to the `#[presents(X)]` site. The failure mode this
//! example demonstrates is the ADR-029 equivalent of a broken witness: a
//! presents-site whose defense never resolves — here, because no `#[defended_by]`
//! test exists at all.

#![allow(dead_code)]

use antigen::{antigen, presents};

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
    summary = "Demonstrates audit catching an undefended presents-site."
)]
pub struct DemoBrokenWitness;

/// Demonstration type that LOOKS defended against `DemoBrokenWitness` but isn't
/// — it declares `#[presents]` but no `#[defended_by]` test ever registers a
/// defense, so the audit reports it undefended.
pub struct LooksImmuneButIsnt {
    inner: String,
}

#[presents(DemoBrokenWitness)]
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
    println!("antigen audit demo: see an undefended presents-site in action.");
    println!("Run `cargo run --bin cargo-antigen -- antigen audit --root antigen/examples`");
}
