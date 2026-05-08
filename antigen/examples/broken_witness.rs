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

use antigen::{antigen, immune, presents};

#[antigen(
    name = "demo-broken-witness",
    family = "boundary-violation",
    // Minimal valid DSL fingerprint — the example's job is to exercise audit's
    // broken-witness path, not the fingerprint matcher. `name = matches("*")`
    // is the trivially-applicable shape.
    fingerprint = r#"name = matches("*")"#,
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
