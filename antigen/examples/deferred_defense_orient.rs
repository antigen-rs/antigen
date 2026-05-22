//! Example: `#[orient]` — see-also context; lightest-weight deferred-defense.
//!
//! ADR-023 deferred-defense family. Orient captures "I acknowledge this
//! code is in an orientation period — I know what failure-classes to watch
//! for, and I'm leaving explicit pointers for whoever arrives here next."
//!
//! Orient is the lightest-weight primitive: ALL fields are optional.
//! `#[orient]` with no arguments is valid — it's just an explicit marker
//! saying "orientation period acknowledged here."
//!
//! ## Biological analog
//!
//! An orientation period: the immune system is present but has not yet
//! been trained to recognize a specific threat. The system acknowledges
//! the threat landscape and is building up its response repertoire.
//! Not the same as anergy (which is about existing unresponsiveness)
//! or immunosuppression (which is about active muting).
//!
//! ## When to use `#[orient]`
//!
//! - New code entering a domain it hasn't been trained on yet
//! - A migration step where the old system's defenses don't yet apply
//! - A port/rewrite where the destination code needs to be reviewed for
//!   the source code's known failure classes
//! - Anywhere you want to leave a "look here" pointer without making a
//!   stronger claim
//!
//! ## Run this example
//!
//! ```sh
//! cargo run --example deferred_defense_orient --package antigen
//! ```

use antigen::orient;

// ============================================================================
// Form 1: Bare orient with no arguments.
// The lightest-weight form — just marks the orientation period.
// ============================================================================

/// A newly-ported function from a legacy system.
/// The legacy system had documented failure classes; this port has not yet
/// been reviewed against them.
#[orient]
pub fn legacy_ported_compute(input: f64) -> f64 {
    // Simplified port of legacy computation.
    input * 1.618_033_988_749_895
}

// ============================================================================
// Form 2: Orient with see-also references.
// Points to relevant ADRs, issues, or documents.
// ============================================================================

/// Handles incoming webhook events.
///
/// This path was recently extracted from a monolith. The source module had
/// several known failure-classes documented in ADR-023 and ISSUE-4891.
/// The extraction has not yet been audited against those failure-classes.
#[orient(
    see = ["ADR-023", "ISSUE-4891", "https://wiki.example.com/webhook-failure-classes"],
    adr = "ADR-023",
)]
pub fn handle_webhook_event(payload: &[u8]) -> Result<(), String> {
    if payload.is_empty() {
        return Err("empty payload".to_string());
    }
    // Real implementation would parse and dispatch here.
    Ok(())
}

// ============================================================================
// Form 3: Orient with attestation_optional flag.
// Signals that normal attestation requirements are relaxed at this site.
// ============================================================================

/// Experimental feature under active development.
///
/// This code path is in active design flux; normal immunity attestation
/// requirements are relaxed while the design stabilizes. Orientation
/// period closes when the design is locked (tracked in ISSUE-5100).
#[orient(
    see = ["ISSUE-5100"],
    attestation_optional,
)]
pub fn experimental_feature_alpha(config: &str) -> Option<String> {
    if config.is_empty() {
        None
    } else {
        Some(config.to_uppercase())
    }
}

// ============================================================================
// Form 4: Orient on a VCS rollback-as-triage site (ADR-026 use case).
// ADR-026 explicitly references #[orient]-shape for rollback-as-triage.
// ============================================================================

/// Emergency rollback handler.
///
/// This function exists as a triage path only — it rolls back to a prior
/// state when the primary path is failing. Per ADR-026, rollback-as-triage
/// sites should use #[orient] to acknowledge the structural context.
#[orient(
    see = ["ADR-026", "rollback-triage-protocol-v2"],
    adr = "ADR-026",
)]
pub fn rollback_to_last_known_good(snapshot_id: &str) -> Result<(), String> {
    // Simplified rollback.
    let _ = snapshot_id;
    println!("[ROLLBACK] Rolling back to snapshot: {snapshot_id}");
    Ok(())
}

fn main() {
    println!("=== antigen deferred-defense: #[orient] example ===");
    println!();
    println!("Four orient usage patterns:");
    println!();
    println!("1. Bare `#[orient]` — no arguments");
    println!("   legacy_ported_compute: simplest form, just marks the orientation period");
    println!();
    println!("2. `#[orient(see = [...], adr = \"...\")]`");
    println!("   handle_webhook_event: points to ADRs and issues");
    println!();
    println!("3. `#[orient(see = [...], attestation_optional)]`");
    println!("   experimental_feature_alpha: relaxes attestation in design-flux code");
    println!();
    println!("4. `#[orient(see = [...], adr = \"ADR-026\")]`");
    println!("   rollback_to_last_known_good: ADR-026 rollback-as-triage use case");
    println!();
    println!("Key ADR-023 discipline:");
    println!("  - All fields optional (lightest-weight deferred-defense primitive)");
    println!("  - No minimum lengths (unlike anergy/immunosuppress/poxparty)");
    println!("  - Audit emits orient-active hint");
    println!("  - Distinct from #[anergy] (no co-stimulation) and #[immunosuppress]");
    println!("    (no duration cap) — orient is pure orientation, not muting");
    println!();

    let _ = legacy_ported_compute(1.0);
    let _ = handle_webhook_event(b"event-data");
    let _ = experimental_feature_alpha("config");
    let _ = rollback_to_last_known_good("snap-20260522-001");

    println!("All orient-annotated functions executed without issues.");
    println!("Run `cargo antigen audit` to see orient-active hints.");
}
