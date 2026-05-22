//! Example: `#[immunosuppress]` — surgical silencing with hard duration cap.
//!
//! ADR-023 deferred-defense family. Immunosuppress captures "I am deliberately
//! muting a specific family of checks for a specific, bounded duration, with a
//! stated rationale." The duration cap is enforced at PARSE TIME — a compile
//! error is emitted if `until - since > duration_cap`. This closes the
//! audit-only gap (A4 attack absorbed).
//!
//! ## Biological analog
//!
//! Pharmacological or pathological immunosuppression: the immune system is
//! deliberately reduced to prevent rejection or autoimmune damage. Expected
//! to be time-bounded; the patient is monitored and suppression is revisited.
//! Distinct from anergy (which is receptor-level unresponsiveness) because
//! immunosuppression is an active, deliberate, systemic intervention.
//!
//! ## When to use `#[immunosuppress]`
//!
//! - You need to silence a specific check family for a specific CI/infra reason
//! - The suppression has a clear expiry (deadline, dependency, migration)
//! - You want the duration to be machine-enforced at compile time
//! - You want a clear record of who approved the suppression and when
//!
//! ## Run this example
//!
//! ```sh
//! cargo run --example deferred_defense_immunosuppress --package antigen
//! ```

use antigen::antigen;
use antigen::immunosuppress;

// ============================================================================
// A failure-class declaration
// ============================================================================

#[antigen(
    name = "missing-observability-instrumentation",
    fingerprint = r#"item = fn, attr_present("allow")"#,
    family = "operational-failure",
    summary = "Async function lacking tracing instrumentation; debugging blind spots."
)]
pub struct MissingObservabilityInstrumentation;

// ============================================================================
// A site where the check family is temporarily suppressed.
// The suppression has a hard deadline and a clear rationale.
// ============================================================================

/// Process work items from the internal queue.
///
/// Observability instrumentation is suppressed here until the OpenTelemetry
/// migration completes. The legacy tracing backend conflicts with the new
/// collector in staging — adding `tracing::instrument` causes a panic in
/// the test environment until the version lock is resolved.
///
/// Duration cap: 90 days (workspace default). This ensures the suppression
/// cannot silently persist indefinitely.
#[immunosuppress(
    MissingObservabilityInstrumentation,
    rationale = "OpenTelemetry migration: legacy tracing backend conflicts \
                 with new OTEL collector in staging environment. Instrumentation \
                 causes test-environment panics until version lock resolved.",
    until = "2026-08-01",
    signed_by = "platform-team"
)]
// async is intentional — models a real async function even with simplified body.
#[allow(clippy::unused_async)]
pub async fn process_work_item(item_id: u64) -> Result<(), String> {
    // Simplified processing without instrumentation.
    // After until-date: restore tracing::instrument here.
    let _ = item_id;
    Ok(())
}

// ============================================================================
// A second site with an explicit duration_cap override
// ============================================================================

#[antigen(
    name = "synchronous-io-in-async-context",
    fingerprint = r#"item = fn, attr_present("allow")"#,
    family = "boundary-violation",
    summary = "Synchronous IO in an async fn blocks the executor thread."
)]
pub struct SynchronousIoInAsyncContext;

/// Read a configuration file.
///
/// This uses synchronous IO in an async context because the config subsystem
/// predates the async runtime. A proper `tokio::fs` migration is in progress.
/// Short cap (30d) signals urgency.
#[immunosuppress(
    SynchronousIoInAsyncContext,
    rationale = "Config subsystem predates async runtime; tokio::fs migration \
                 tracked in ISSUE-5012. Short cap signals urgency of migration.",
    until = "2026-06-21",
    duration_cap = 30,
    signed_by = "infra-lead"
)]
// async is intentional — models a function that should use tokio::fs.
#[allow(clippy::unused_async)]
pub async fn read_config(path: &str) -> Result<String, String> {
    // Would use std::fs::read_to_string in real code.
    let _ = path;
    Ok("{}".to_string())
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    println!("=== antigen deferred-defense: #[immunosuppress] example ===");
    println!();
    println!("Two immunosuppress declarations:");
    println!();
    println!("1. process_work_item");
    println!("   antigen:  MissingObservabilityInstrumentation");
    println!("   until:    2026-11-01");
    println!("   cap:      90d (workspace default)");
    println!("   signer:   platform-team");
    println!();
    println!("2. read_config");
    println!("   antigen:  SynchronousIoInAsyncContext");
    println!("   until:    2026-07-15");
    println!("   cap:      30d (explicit override — urgency signal)");
    println!("   signer:   infra-lead");
    println!();
    println!("Key ADR-023 discipline:");
    println!("  - Duration cap enforced at PARSE TIME (A4: not audit-only)");
    println!("  - 'rationale' minimum 20 characters (loudness-as-discipline)");
    println!("  - After 'until' passes: hint becomes immunosuppress-expired");
    println!("  - Try setting 'until' to > 90 days from now: compile error");
}
