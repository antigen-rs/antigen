//! Example: `#[anergy]` — deferred-but-muted defense with required time-bound.
//!
//! ADR-023 deferred-defense family. Anergy captures the pattern of
//! "I know I'm not immune to X, I'm choosing not to address it right now,
//! and here is my explicit time-bound and reason." The `until` field is
//! REQUIRED — anergy without a time-bound degrades to silent tolerance.
//!
//! ## Biological analog
//!
//! T-cell anergy: the cell encounters its antigen but fails to mount an
//! immune response due to lack of co-stimulation. The cell is alive but
//! functionally unresponsive. Anergy is reversible — co-stimulation arriving
//! later can re-engage the response. This is structurally distinct from
//! tolerance (where the antigen is recognized as self) and from suppression
//! (where an active process is inhibiting the response).
//!
//! ## When to use `#[anergy]`
//!
//! - You know a failure-class applies to a site
//! - You cannot build immunity right now (upstream blocker, infra constraint)
//! - You have a specific trigger that will enable immunity later
//! - You want that context to be load-bearing — auditable, visible, aging
//!
//! ## Run this example
//!
//! ```sh
//! cargo run --example deferred_defense_anergy --package antigen
//! ```

use antigen::anergy;
// ============================================================================
// A failure-class declaration — this would normally live in a shared antigens
// module. Inline here for self-contained example.
// ============================================================================
use antigen::antigen;

#[antigen(
    name = "unvalidated-external-input",
    fingerprint = r#"item = fn, attr_present("allow")"#,
    family = "trust-boundary-violation",
    summary = "Input parsed without validation; trust boundary not enforced."
)]
pub struct UnvalidatedExternalInput;

// ============================================================================
// A site presenting the antigen — the code handles external input without
// the validation layer it needs. Immunity is blocked on upstream work.
// ============================================================================

/// Process an API response payload.
///
/// Currently parses without validation because the shared `Validator` crate
/// does not yet expose async streaming validation (tracked in ISSUE-4471).
/// Once `validator-rs` v2.0 ships, mark this site
/// `#[presents(UnvalidatedExternalInput)]` and register a defending test with
/// `#[defended_by(UnvalidatedExternalInput)]` (ADR-029 — immunity is observed,
/// not declared).
#[anergy(
    UnvalidatedExternalInput,
    reason = "validator-rs v2.0 async streaming API not yet available; \
              current v1 API blocks the async runtime.",
    until = "2027-03-01",
    expected_co_stimulation = "validator-rs-v2-upgrade-complete",
    signed_by = "reviewer"
)]
pub fn process_api_response(payload: &str) -> Result<String, String> {
    // In production this should validate before processing.
    // Anergy declares the known gap explicitly with an expiry.
    Ok(payload.to_uppercase())
}

// ============================================================================
// A second anergy site — different failure-class, different trigger.
// ============================================================================

#[antigen(
    name = "unbounded-retry-loop",
    fingerprint = r#"item = fn, attr_present("allow")"#,
    family = "resource-exhaustion",
    summary = "Retry loop without an upper bound; could exhaust caller resources."
)]
pub struct UnboundedRetryLoop;

/// Retry a flaky network call.
///
/// Retry-count cap is tracked in ISSUE-4488 — the circuit-breaker
/// implementation is blocked on the async-context refactor (Q3 2026).
#[anergy(
    UnboundedRetryLoop,
    reason = "Circuit-breaker implementation deferred pending async-context \
              refactor completing in Q3 2026. Retry cap cannot be injected \
              cleanly until the refactor lands.",
    until = "2026-10-01",
    expected_co_stimulation = "async-context-refactor-merged"
)]
// The `async` is intentional — this models a real async function signature
// even though the simplified example body has no await points.
#[allow(clippy::unused_async)]
pub async fn retry_network_call(url: &str) -> Result<String, String> {
    // Simplified: in reality this would loop.
    let _ = url;
    Err("not connected".to_string())
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    println!("=== antigen deferred-defense: #[anergy] example ===");
    println!();
    println!("Two anergy declarations:");
    println!();
    println!("1. process_api_response");
    println!("   antigen: UnvalidatedExternalInput");
    println!("   reason:  validator-rs v2.0 async API not yet available");
    println!("   until:   2027-03-01");
    println!("   trigger: validator-rs-v2-upgrade-complete");
    println!();
    println!("2. retry_network_call (async)");
    println!("   antigen: UnboundedRetryLoop");
    println!("   reason:  circuit-breaker blocked on async-context refactor");
    println!("   until:   2026-10-01");
    println!("   trigger: async-context-refactor-merged");
    println!();
    println!("Run `cargo antigen audit` to see anergy-active hints.");
    println!("Both sites will report audit hint: anergy-active");
    println!();
    println!("Key ADR-023 discipline:");
    println!("  - 'until' is REQUIRED (A5: anergy without time-bound = silent tolerance)");
    println!("  - 'reason' minimum 20 characters (loudness-as-discipline)");
    println!("  - After 'until' passes: hint escalates to anergy-co-stimulation-not-arrived");
    println!("  - Past grace period: escalates to anergy-stale");

    let result = process_api_response(r#"{"key": "value"}"#);
    println!();
    println!("process_api_response result: {result:?}");
}
