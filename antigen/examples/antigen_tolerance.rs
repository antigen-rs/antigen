//! Tolerance example: `#[antigen_tolerance]` documents intentional opt-out.
//! ADR-011.
//!
//! Sometimes a site genuinely needs to exhibit a failure-class for reasons
//! the architecture has weighed and accepted. The vocabulary supports
//! explicit, rationale-bearing opt-outs via `#[antigen_tolerance]`:
//!
//! - rationale (required, non-empty) — the *why* must be on the page
//! - until (optional) — a tag indicating when this tolerance should be
//!   re-evaluated (a version, a milestone, a date)
//! - see (optional) — references to PRs, ADRs, design docs that contextualise
//!   the decision
//!
//! Tolerated sites surface in scan output under the "tolerated" state
//! (state 4 of the 7-state matrix, ADR-018) rather than "unaddressed."
//!
//! Run:
//!
//! ```sh
//! cargo run --bin cargo-antigen -- antigen scan --root antigen/examples
//! ```
//!
//! Expected: `IntentionalPanicSite` has `#[presents(IntentionalPanicAntigen)]`
//! AND `#[antigen_tolerance(IntentionalPanicAntigen, rationale = "...")]`.
//! Scan reports the site as tolerated, not unaddressed.

#![allow(dead_code)]

use antigen::{antigen, antigen_tolerance, presents};

#[antigen(
    name = "intentional-panic-antigen",
    family = "boundary-violation",
    fingerprint = r#"name = matches("IntentionalPanicSite")"#,
    summary = "Marks code that panics deliberately (assertion-style invariants in test scaffolding)."
)]
pub struct IntentionalPanicAntigen;

/// A site that genuinely needs to panic — test scaffolding that asserts
/// an invariant via panic.
///
/// The tolerance carries the rationale on the page; `until = "v0.2"` flags
/// it for re-evaluation at the next minor release; the `see` array points
/// readers at the surrounding substrate.
pub struct IntentionalPanicSite;

#[presents(IntentionalPanicAntigen)]
#[antigen_tolerance(
    IntentionalPanicAntigen,
    rationale = "Test scaffolding only — production callers go through a different code path \
                 that returns Result<T, E> rather than panicking. Verified by integration tests.",
    until = "v0.2",
    see = [
        "https://github.com/antigen-rs/antigen/blob/main/docs/decisions.md#adr-011",
    ]
)]
impl IntentionalPanicSite {
    /// Asserts an invariant; in test code this is intentional.
    ///
    /// The literal `panic!` is required here — `assert!` would not match
    /// the `PanickingInDrop` fingerprint's `body_contains_macro("panic")`
    /// constraint, and the whole point of this example is to demonstrate
    /// tolerance of a real fingerprint match.
    #[allow(
        clippy::manual_assert,
        reason = "panic! literal required for fingerprint match"
    )]
    pub fn assert_invariant(condition: bool) {
        if !condition {
            panic!("invariant violated — this is intentional in test scaffolding");
        }
    }
}

fn main() {
    println!("antigen_tolerance example: intentional opt-out with rationale on the page.");
    println!();
    println!("Run `cargo run --bin cargo-antigen -- antigen scan --root antigen/examples`");
    println!("to see the tolerated site reported separately from unaddressed presentations.");
}
