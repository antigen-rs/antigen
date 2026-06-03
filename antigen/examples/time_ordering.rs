//! Time-and-Ordering-Hazards family — the admitting-specimen.
//!
//! The affinity-pair exhibit (ADR-039 §C worth-multiplier) for
//! [`antigen::stdlib::time_ordering::SystemTimeUnwrapPanic`]: a clock read whose
//! `Result` is `unwrap`-ed (binds) + a sibling that handles the `Result`
//! (spared). The flagship silent-in-tests / panic-in-prod shape.
//!
//! Run:
//!
//! ```sh
//! cargo run --example time_ordering --package antigen
//! ```
//!
//! Scan to see the pair separate:
//!
//! ```sh
//! cargo run --bin cargo-antigen -- antigen scan --root antigen/examples
//! ```
//!
//! ## BIOSAFETY NOTE
//!
//! The "bad" path below really would panic on a backwards clock — it is kept
//! intentional for the exhibit. Do not copy it; the safe sibling shows the fix.

use antigen::{antigen, presents};
use std::time::{Duration, SystemTime};

/// A clock read whose `Result` is `unwrap`/`expect`-ed — panics in production
/// when the clock runs backwards, never in tests.
#[antigen(
    name = "system-time-unwrap-panic",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"all_of([any_of([body_calls("duration_since"), body_calls("elapsed")]), any_of([body_calls("unwrap"), body_calls("expect")])])"#,
    family = "time-and-ordering-hazards",
    summary = "A SystemTime clock read (duration_since / elapsed) whose Result is unwrap/expect-ed — panics in prod on backwards-clock, never in tests.",
    references = ["https://doc.rust-lang.org/std/time/struct.SystemTime.html#method.duration_since"],
)]
pub struct SystemTimeUnwrapPanic;

/// BAD (the bind): reads the clock and `unwrap`s the `Result`. If the system
/// clock has moved backwards since `earlier`, `duration_since` returns `Err` and
/// this panics — in production, on an input the happy-path tests never produce.
///
/// `any_of([duration_since, elapsed])` matches AND `any_of([unwrap, expect])`
/// matches → the `all_of` **binds**.
#[presents(SystemTimeUnwrapPanic)]
fn age_since_panicking(earlier: SystemTime) -> Duration {
    SystemTime::now().duration_since(earlier).unwrap()
}

/// GOOD (the spare): the same clock read, but the `Result` is HANDLED — a
/// backwards clock yields `Duration::ZERO` instead of a panic.
///
/// The clock-read half still matches, but `any_of([unwrap, expect])` does NOT
/// (there is no `unwrap`/`expect` in the body) → the `all_of` is **spared**.
#[presents(SystemTimeUnwrapPanic)]
fn age_since_safe(earlier: SystemTime) -> Duration {
    SystemTime::now()
        .duration_since(earlier)
        .unwrap_or(Duration::ZERO)
}

fn main() {
    println!("antigen time-ordering example: see source for the affinity-pair.");
    println!(
        "Run `cargo run --bin cargo-antigen -- antigen scan` to see the panicking path flagged, the handled path spared."
    );

    let earlier = SystemTime::now();
    // The safe path always works; the panicking path works too while the clock
    // is monotonic in this run (its danger is the backwards-clock edge).
    let _ = age_since_safe(earlier);
    let _ = age_since_panicking(earlier);
}
