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
//! cargo run --bin cargo-antigen -- antigen scan --root antigen/examples
//! ```
//!
//! Note: both siblings are `#[presents]`-marked, so audit lists both — the safe
//! sibling is spared by the *fingerprint* (it doesn't bind), not hidden from the
//! console. To *read* the bind/spare side by side, see the guard tests
//! `antigen/tests/stdlib_family_fingerprints.rs`
//! (`system_time_unwrap_binds_duration_since_then_unwrap` beside
//! `system_time_unwrap_spares_instant_elapsed_clean_sibling`).
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
    fingerprint = r#"all_of([body_calls("duration_since"), any_of([body_calls("unwrap"), body_calls("expect")])])"#,
    family = "time-and-ordering-hazards",
    summary = "A SystemTime::duration_since clock read whose Result is unwrap/expect-ed — panics in prod on backwards-clock, never in tests. (elapsed excluded: it fires on the Instant::elapsed clean sibling = the recommended fix.)",
    references = ["https://doc.rust-lang.org/std/time/struct.SystemTime.html#method.duration_since"],
)]
pub struct SystemTimeUnwrapPanic;

/// BAD (the bind): reads the clock and `unwrap`s the `Result`. If the system
/// clock has moved backwards since `earlier`, `duration_since` returns `Err` and
/// this panics — in production, on an input the happy-path tests never produce.
///
/// `body_calls("duration_since")` matches AND `any_of([unwrap, expect])` matches
/// → the `all_of` **binds**.
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
        "Both siblings are #[presents]-marked, so audit lists both; the handled path is spared by the FINGERPRINT (it doesn't bind). To read the bind/spare side by side, see antigen/tests/stdlib_family_fingerprints.rs."
    );

    let earlier = SystemTime::now();
    // The safe path always works; the panicking path works too while the clock
    // is monotonic in this run (its danger is the backwards-clock edge).
    let _ = age_since_safe(earlier);
    let _ = age_since_panicking(earlier);
}
