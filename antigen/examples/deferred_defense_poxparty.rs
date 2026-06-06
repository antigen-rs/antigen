//! Example: `#[poxparty]` — intentional exposure with structural isolation.
//!
//! ADR-023 deferred-defense family. Poxparty captures "I am deliberately
//! introducing controlled exposure to a failure-class as a training exercise,
//! chaos test, or red-team session." The site is STRUCTURALLY ISOLATED: the
//! `#[poxparty]` macro emits a COMPILE ERROR if the `antigen-poxparty` Cargo
//! feature is not active. Production builds cannot accidentally include
//! pox-party exercise code.
//!
//! ## Biological analog
//!
//! Pox parties: intentional exposure to a pathogen (e.g., chickenpox) to
//! build immunity in a controlled setting. The exposure is deliberate, bounded
//! in time, and expected to produce a trained immune response. Structurally
//! distinct from anergy (unresponsiveness) and immunosuppression (muting).
//!
//! ## Structural isolation mechanism (A3)
//!
//! The `#[poxparty]` macro reads the `CARGO_FEATURE_ANTIGEN_POXPARTY`
//! environment variable at proc-macro expansion time. Cargo sets this
//! variable automatically when the `antigen-poxparty` feature is active.
//! This is the CORRECT mechanism — proc-macros run after cfg expansion,
//! so cfg-context inspection doesn't work for structural isolation.
//!
//! ## How to run this example (requires the feature)
//!
//! ```sh
//! cargo run --example deferred_defense_poxparty --package antigen \
//!     --features antigen-poxparty
//! ```
//!
//! Without the feature: compile error (A3 isolation working as designed).

// This module is only reachable with the antigen-poxparty feature.
// Without the feature, any #[poxparty] usage in compiled code would
// produce a compile error. The example guards its exercise content here
// to demonstrate the intended isolation pattern.

use antigen::antigen;

#[antigen(
    name = "retry-budget-exhaustion",
    fingerprint = r#"item = fn, attr_present("allow")"#,
    family = "resource-exhaustion",
    summary = "Retry logic without a per-caller budget; can cascade into resource exhaustion."
)]
pub struct RetryBudgetExhaustion;

/// Regular production function — has the antigen's fingerprint.
/// In a real codebase, immunity would be declared here via `#[immune]`.
pub fn send_with_retry(endpoint: &str, max_retries: u32) -> Result<String, String> {
    for attempt in 0..max_retries {
        let _ = (endpoint, attempt);
        // Simplified: would actually retry here.
    }
    Err("exhausted".to_string())
}

// ============================================================================
// Exercise code — only reachable when antigen-poxparty feature is active.
// Without the feature, any #[poxparty] attribute would emit a compile error.
// ============================================================================

#[cfg(feature = "antigen-poxparty")]
mod poxparty_exercises {
    use antigen::poxparty;

    use super::RetryBudgetExhaustion;

    /// Chaos test: deliberately exhaust the retry budget to verify that the
    /// system-under-test correctly surfaces a budget-exhaustion error to
    /// the caller rather than silently retrying indefinitely.
    ///
    /// This test is structurally isolated by the `antigen-poxparty` feature.
    /// It must NOT be run in production; the feature gate enforces this.
    #[poxparty(
        RetryBudgetExhaustion,
        exercise_type = "Chaos test: exhaust retry budget to verify caller \
                         receives budget-exhaustion error, not infinite loop.",
        until = "2026-12-31",
        name = "retry-budget-chaos-exercise-q4-2026",
        rationale = "Validates that budget exhaustion surfaces correctly under load."
    )]
    pub fn chaos_exercise_retry_budget_exhaustion() {
        // In a real chaos test: run send_with_retry under load with a
        // deliberately low budget and assert the caller receives the error.
        println!("[poxparty exercise] Simulating retry budget exhaustion...");
        let result = super::send_with_retry("https://internal-service/api", 1);
        assert!(result.is_err(), "budget exhaustion should surface as error");
        println!("[poxparty exercise] Confirmed: budget-exhausted state surfaced correctly.");
    }
}

fn main() {
    println!("=== antigen deferred-defense: #[poxparty] example ===");
    println!();

    #[cfg(feature = "antigen-poxparty")]
    {
        println!("antigen-poxparty feature is ACTIVE");
        println!("Running poxparty chaos exercise...");
        println!();
        poxparty_exercises::chaos_exercise_retry_budget_exhaustion();
        println!();
        println!("Exercise complete.");
    }

    #[cfg(not(feature = "antigen-poxparty"))]
    {
        println!("antigen-poxparty feature is NOT active.");
        println!();
        println!("This is the expected state for production builds.");
        println!("Any #[poxparty] attribute in compiled code would produce");
        println!("a COMPILE ERROR — structural isolation working as designed.");
        println!();
        println!("To run exercises:");
        println!("  cargo run --example deferred_defense_poxparty \\");
        println!("      --package antigen --features antigen-poxparty");
    }

    println!();
    println!("Key ADR-023 discipline (A3 absorbed):");
    println!("  - CARGO_FEATURE_ANTIGEN_POXPARTY env var checked at macro-expansion time");
    println!("  - Compile error emitted if feature not active");
    println!("  - antigen-poxparty MUST NOT be in default feature set");
    println!("  - exercise_type minimum 20 characters (loudness-as-discipline)");
    println!("  - until required (bounded exercise, not indefinite exposure)");
}
