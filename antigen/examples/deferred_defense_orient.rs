//! Example: `#[orient]` — a loud, time-bounded, pre-immunity orientation
//! period (ADR-023 deferred-defense family) + its sibling `#[triage_commit]`.
//!
//! Orient captures "I acknowledge this code is in an orientation period — here
//! is the explicit path out (`learning_path`) and the horizon by which it must
//! close (`until`)." Per ADR-023 §Decision + the Option-A ruling, both fields
//! are **REQUIRED**: an orient without a path-out and a time-bound is silent
//! deferred non-immunity, which is just tolerance — and orient exists to be
//! loud about it. A bare `#[orient]` is a compile error.
//!
//! ## Biological analog
//!
//! An orientation period: the immune system is present but has not yet been
//! trained to recognize a specific threat. It acknowledges the threat
//! landscape and commits to building up its response repertoire by a deadline.
//! Distinct from anergy (existing unresponsiveness) and immunosuppression
//! (active muting).
//!
//! ## When to use `#[orient]`
//!
//! - New code entering a domain it hasn't been trained on yet
//! - A migration step where the old system's defenses don't yet apply
//! - A port/rewrite whose destination needs review against the source's known
//!   failure-classes
//!
//! For a *decisional* rollback-as-triage site, use `#[triage_commit]` (the
//! sibling primitive, ADR-026), NOT `#[orient]` — shown as Form 4 below.
//!
//! ## Run this example
//!
//! ```sh
//! cargo run --example deferred_defense_orient --package antigen
//! ```

use antigen::{orient, triage_commit};

// ============================================================================
// Form 1: Canonical orient — the ADR-023 spec form.
// learning_path (the explicit path out) + until (the horizon) are required.
// ============================================================================

/// A newly-ported function from a legacy system whose documented
/// failure-classes this port has not yet been reviewed against.
#[orient(
    learning_path = "Review against the legacy system's documented failure-classes before the v1 tag",
    until = "2026-09-01"
)]
pub fn legacy_ported_compute(input: f64) -> f64 {
    // Simplified port of legacy computation.
    input * 1.618_033_988_749_895
}

// ============================================================================
// Form 2: Orient on code extracted from a monolith.
// See-also context that the old `see`/`adr` fields used to carry now lives
// inside the learning_path text (those fields were removed in the restoration).
// ============================================================================

/// Handles incoming webhook events, recently extracted from a monolith.
///
/// The source module's known failure-classes (ADR-023, ISSUE-4891) have not
/// yet been audited against this extraction.
#[orient(
    PanickingInDrop,
    learning_path = "Audit the extraction against the monolith's failure-classes (ADR-023, ISSUE-4891) before promoting off orientation",
    until = "2026-08-15"
)]
pub fn handle_webhook_event(payload: &[u8]) -> Result<(), String> {
    if payload.is_empty() {
        return Err("empty payload".to_string());
    }
    // Real implementation would parse and dispatch here.
    Ok(())
}

// ============================================================================
// Form 3: Orient on a design-flux feature.
// The old `attestation_optional` field (which inverted loudness-as-discipline)
// was removed with no replacement — a design-flux site still gets a real,
// loud orientation period with a closing horizon, not relaxed attestation.
// ============================================================================

/// Experimental feature under active development.
///
/// The orientation period closes when the design is locked (ISSUE-5100).
#[orient(
    learning_path = "Lock the design (ISSUE-5100), then declare real immunity or tolerance before this horizon",
    until = "2026-07-31"
)]
pub fn experimental_feature_alpha(config: &str) -> Option<String> {
    if config.is_empty() {
        None
    } else {
        Some(config.to_uppercase())
    }
}

// ============================================================================
// Form 4: Rollback-as-triage — NOT orient. Use #[triage_commit] (ADR-026).
// A *decisional* rollback site classifies system state and commits to action
// in a bounded window; that is the triage_commit primitive, not orient.
// ============================================================================

/// Emergency rollback handler — reverts to a prior state when the primary path fails.
///
/// Per ADR-026 this is a triage commitment, not an orientation period, so it
/// carries `#[triage_commit]`.
#[triage_commit(
    triage_decision = TriageDecision::Red,
    rollback_target = "abc1234",
    triaged_by = "navigator",
    rationale = "primary path failing in production; rolling back to last-known-good snapshot pending root-cause",
    rollback_due_within_minutes = 30
)]
pub fn rollback_to_last_known_good(snapshot_id: &str) -> Result<(), String> {
    // Simplified rollback.
    let _ = snapshot_id;
    println!("[ROLLBACK] Rolling back to snapshot: {snapshot_id}");
    Ok(())
}

fn main() {
    println!("=== antigen deferred-defense: #[orient] (+ #[triage_commit]) example ===");
    println!();
    println!("Orient is a LOUD, time-bounded, pre-immunity orientation period.");
    println!("learning_path + until are REQUIRED (ADR-023 Option-A); a bare #[orient]");
    println!("is a compile error — silent deferred non-immunity is just tolerance.");
    println!();
    println!("1. Canonical orient(learning_path, until)");
    println!("   legacy_ported_compute: the ADR-023 spec form");
    println!();
    println!("2. orient(antigen, learning_path, until)");
    println!("   handle_webhook_event: see-also context folded into learning_path");
    println!();
    println!("3. orient(learning_path, until) on a design-flux feature");
    println!("   experimental_feature_alpha: a real orientation period, not relaxed attestation");
    println!();
    println!("4. #[triage_commit(...)] — NOT orient");
    println!("   rollback_to_last_known_good: decisional rollback-as-triage (ADR-026 sibling)");
    println!();
    println!("Key ADR-023 discipline:");
    println!("  - learning_path (>= 20 chars) + until (UTC, within 180d horizon) REQUIRED");
    println!("  - the horizon is enforced at PARSE time (a date too far out is a compile error)");
    println!("  - rollback-as-triage uses #[triage_commit], not #[orient]");
    println!();

    let _ = legacy_ported_compute(1.0);
    let _ = handle_webhook_event(b"event-data");
    let _ = experimental_feature_alpha("config");
    let _ = rollback_to_last_known_good("snap-20260522-001");

    println!("All annotated functions executed without issues.");
    println!("Run `cargo antigen audit` to see orient-active + triage-commit hints.");
}
