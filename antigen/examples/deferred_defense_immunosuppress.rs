//! Example: `#[immunosuppress]` — surgical silencing with hard duration cap.
//!
//! **TEMPORARILY STUBBED.** The `#[immunosuppress(...)]` sites below are commented
//! out, so this example does not currently demonstrate a live suppression. The
//! `#[immunosuppress]` time-bound `until`-date mechanism is being replaced by a
//! self-presenting expiry class: an expired window will *present* a failure-class
//! (which an audit surfaces) rather than emit a hard compile error, and hardcoded
//! example dates — which rot — go away. The `#[dread]` on `process_work_item`
//! self-presents this stub, so an audit reminds us to restore / rewrite the example
//! against the new mechanism instead of it being silently lost.
//!
//! ADR-023 deferred-defense family. Immunosuppress captures "I am deliberately
//! muting a specific family of checks for a specific, bounded duration, with a
//! stated rationale."
//!
//! ## Biological analog
//!
//! Pharmacological or pathological immunosuppression: the immune system is
//! deliberately reduced to prevent rejection or autoimmune damage. Expected
//! to be time-bounded; the patient is monitored and suppression is revisited.
//! Distinct from anergy (which is receptor-level unresponsiveness) because
//! immunosuppression is an active, deliberate, systemic intervention.
//!
//! ## Run this example
//!
//! ```sh
//! cargo run --example deferred_defense_immunosuppress --package antigen
//! ```

use antigen::antigen;
use antigen::dread;
// use antigen::immunosuppress; // stubbed — see the module note + the #[dread] below.

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
// A site where the check family would be temporarily suppressed.
//
// The live `#[immunosuppress(...)]` is commented out pending the presents-expiry
// redesign (see the module note); the `#[dread]` below self-presents the stub.
// ============================================================================

/// Process work items from the internal queue.
///
/// Observability instrumentation would be suppressed here until the OpenTelemetry
/// migration completes. (Demonstration stubbed — see the module note.)
#[dread(
    trigger = "the #[immunosuppress] demonstration in this example is commented \
                   out. Its time-bound `until`-date mechanism is being replaced by a \
                   self-presenting expiry class (the presents-expiry redesign), and \
                   hardcoded example dates rot. Restore / rewrite this example against \
                   the new mechanism once it lands."
)]
// The live suppression, stubbed pending the presents-expiry redesign:
// #[immunosuppress(
//     MissingObservabilityInstrumentation,
//     rationale = "OpenTelemetry migration: legacy tracing backend conflicts \
//                  with new OTEL collector in staging environment. Instrumentation \
//                  causes test-environment panics until version lock resolved.",
//     until = "<future date>",
//     signed_by = "platform-team"
// )]
// async is intentional — models a real async function even with simplified body.
#[allow(clippy::unused_async)]
pub async fn process_work_item(item_id: u64) -> Result<(), String> {
    // Simplified processing without instrumentation.
    let _ = item_id;
    Ok(())
}

// ============================================================================
// A second site with an explicit duration_cap override (also stubbed)
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
/// Synchronous IO in an async context (the config subsystem predates the async
/// runtime; a `tokio::fs` migration is in progress). (Demonstration stubbed — see
/// the module note.)
// The live suppression, stubbed pending the presents-expiry redesign:
// #[immunosuppress(
//     SynchronousIoInAsyncContext,
//     rationale = "Config subsystem predates async runtime; tokio::fs migration \
//                  tracked in ISSUE-5012. Short cap signals urgency of migration.",
//     since = "<start>",
//     until = "<future date>",
//     duration_cap = 30,
//     signed_by = "infra-lead"
// )]
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
    println!("This example is TEMPORARILY STUBBED: the #[immunosuppress] sites are");
    println!("commented out pending the presents-expiry redesign (an expired window");
    println!("will present a failure-class rather than hard-error, and example dates");
    println!("stop being hardcoded). The #[dread] on process_work_item self-presents");
    println!("the stub so an audit reminds us to restore it.");
    println!();
    println!("ADR-023 discipline (for reference, once restored):");
    println!("  - rationale minimum 20 characters (loudness-as-discipline)");
    println!("  - after 'until' passes: hint becomes immunosuppress-expired");
}
