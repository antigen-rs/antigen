//! # Time-and-Ordering-Hazards Family — stdlib antigens (beta.2 voyage)
//!
//! Clock and ordering failure-classes. The flagship is the **silent-in-tests /
//! panic-in-prod** shape: the system clock can run BACKWARDS (NTP correction,
//! manual set, VM pause), so `SystemTime::duration_since` / `.elapsed()` returns
//! `Err` — an `.unwrap()` on it panics in production but NEVER in tests (test
//! machines do not NTP-skew mid-test). A textbook failure-class antigen exists
//! to name: the bug the test suite structurally cannot reach.
//!
//! Biology cognate: **circadian / signaling-timing failure** — the immune system
//! depends on correctly-ordered signaling cascades; a clock that runs backwards
//! corrupts the cascade timing → the wrong response fires.
//!
//! ## Antigen-category (ADR-028)
//!
//! `FunctionalCorrectness`: the `unwrap`-on-clock-skew verb produces a wrong
//! *effect* (a process panic on an input — backwards-clock — the happy-path
//! tests never exercise).
//!
//! ## How these antigens are evaluated
//!
//! Member 1 carries a **syntactic co-occurrence fingerprint** matched by the
//! AST-walking scanner — a clock-read call (`duration_since` / `elapsed`)
//! together with an `unwrap` / `expect` call in the same body.

use crate::antigen;

// ============================================================================
// 1. SystemTimeUnwrapPanic
// ============================================================================

/// A clock read (`SystemTime::duration_since` / `.elapsed()`) whose `Result` is
/// `unwrap`/`expect`-ed — panics in production when the clock runs backwards,
/// never in tests.
///
/// **Where in the wild:** the canonical clock footgun. The system clock can go
/// BACKWARDS (NTP correction, manual set, VM pause) → `duration_since` returns
/// `Err` → `.unwrap()` panics in prod. The happy-path tests never NTP-skew
/// mid-test, so the bug is structurally invisible to the test suite — the
/// silent-in-tests / panic-in-prod flagship.
///
/// **Tell (and its honest tier):** the PRECISE tell is a method-chain —
/// `x.duration_since(y).unwrap()`. The shipped grammar has no relational/chain
/// leaf, so this member ships the **co-occurrence** form:
/// `all_of([body_calls("duration_since"), any_of([body_calls("unwrap"),
/// body_calls("expect")])])` — a `duration_since` call AND an `unwrap`/`expect`
/// call in the same body. This is honestly **suspected**, NOT named:
/// co-occurrence *correlates* with the panic-chain but does not *prove* it (the
/// `unwrap` could guard a different `Result`). When the precise method-chain leaf
/// ships (charter / next increment), the member graduates suspected → named.
///
/// **Why `elapsed` is NOT in the anchor (the clean-sibling rule):** an `elapsed`
/// arm would fire on `Instant::now().elapsed()` — but `Instant` is monotonic and
/// `Instant::elapsed()` returns `Duration` (not `Result`, can't panic-on-skew):
/// it is the textbook *"use `Instant` instead of `SystemTime`"* **fix**, i.e. the
/// member's own clean sibling. A needle that fires on the anti-correlated safe
/// case (the fix) is dropped at *every* tier (not merely demoted) — so `elapsed`
/// is excluded. (Recall cost: `SystemTime::elapsed().unwrap()` is lost as a
/// true-positive; a v0.4-recoverable FN — receiver-type resolution re-adds
/// `elapsed` *anchored on a `SystemTime` receiver*, recovering the TP without the
/// `Instant` clean-sibling FP. Documented-not-forgotten.) `duration_since` is
/// kept because, while it also exists on `Instant`, the precise `SystemTime`-
/// vs-`Instant` ambiguity is resolved by the witness/tier, and dropping it would
/// leave no anchor at all.
///
/// **Witness:** the `Result` is handled (`.unwrap_or(Duration::ZERO)`, a `match`),
/// OR `Instant` is used instead of `SystemTime` for the measurement.
///
/// **Category:** `FunctionalCorrectness` — the unwrap-on-skew produces a wrong
/// *effect* (a prod panic on an input the tests never reach).
#[antigen(
    name = "system-time-unwrap-panic",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"all_of([body_calls("duration_since"), any_of([body_calls("unwrap"), body_calls("expect")])])"#,
    family = "time-and-ordering-hazards",
    summary = "A SystemTime::duration_since clock read whose Result is unwrap/expect-ed — panics in prod on backwards-clock, never in tests. Suspected tier (co-occurrence, not the precise chain). elapsed excluded (fires on the Instant::elapsed clean sibling = the fix).",
    references = [
        "https://doc.rust-lang.org/std/time/struct.SystemTime.html#method.duration_since",
        "ADR-040",
    ]
)]
pub struct SystemTimeUnwrapPanic;
