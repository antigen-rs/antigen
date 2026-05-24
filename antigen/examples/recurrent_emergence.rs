//! Example: the Recurrent-Emergence Family (ADR-024 §Family 2).
//!
//! The recurrent-emergence primitives capture failure-classes that
//! re-emerge across project lifetimes — the present-looking arc of
//! ADR-024's temporal taxonomy. Where the convergent family looks BACKWARD
//! (evidence already gathered) and the prescriptive family looks FORWARD
//! (work to orchestrate), recurrent looks at the NOW: a pattern that keeps
//! coming back, noticed before it has fully crystallized into a named
//! failure-class.
//!
//! ## The six primitives + their grounding axes
//!
//! - `#[itch]` (cognitive-organizational) — a pattern noticed below the
//!   threshold of commitment. "I keep seeing this; not sure it's a thing yet."
//! - `#[recurrence_anchor]` (clinical-medicine) — cross-substrate recurrence
//!   that crossed the threshold; formally anchored for action, like a
//!   clinical diagnosis after recurrent symptoms.
//! - `#[crystallize]` (cognitive-organizational) — the promotion event from
//!   itch-cluster to a named failure-class.
//! - `#[chronic]` (immunology-proper) — a low-level persistent signal that
//!   is NOT cross-substrate; sustained inflammation without acute recurrence.
//! - `#[saturate]` (cognitive-organizational) — accumulating evidence toward
//!   a recurrence threshold.
//! - `#[strand]` (cognitive-organizational) — a thread of related noticing
//!   that may spawn an itch or an anchor.
//!
//! Per ADR-024 §Biology grounding — dual-axis honesty: only `#[chronic]`
//! claims immunology-proper grounding; the cognitive-organizational
//! primitives are honestly named as drawn from how teams notice patterns,
//! not from immune biology.
//!
//! ## Antigen-category (ADR-028)
//!
//! Recurrent failure-classes are `SubstrateAlignment`: the representation of
//! "this keeps happening" diverges from the actual cross-lifetime state
//! until the recurrence is anchored. The example antigen below declares the
//! category explicitly.
//!
//! ## Run this example
//!
//! ```sh
//! cargo run --example recurrent_emergence --package antigen
//! cargo run --bin cargo-antigen -- antigen scan   # surfaces the markers
//! ```

// The marker functions below are illustration sites for the recurrent
// attributes; their bodies are intentionally empty (the attribute is the
// point, not the runtime behavior), which trips missing_const_for_fn. The
// allow is example-scoped and does not affect the library crate.
#![allow(clippy::missing_const_for_fn)]

use antigen::{antigen, chronic, crystallize, itch, recurrence_anchor, saturate, strand};

// ============================================================================
// A canonical recurrent failure-class declaration.
//
// MSRV (minimum-supported-rust-version) creeping upward after a transitive
// dependency bump is a classic cross-project-lifetime recurrence: it happens
// again every few releases, in every Rust workspace, and the "we fixed it"
// memory evaporates between occurrences. SubstrateAlignment per ADR-028 —
// the Cargo.toml `rust-version` representation diverges from the actually-
// required MSRV after the bump.
// ============================================================================

#[antigen(
    name = "msrv-creep-after-major-version-bump",
    category = AntigenCategory::SubstrateAlignment,
    fingerprint = r#"doc_contains("ADR-024")"#,
    family = "recurrent-emergence",
    summary = "MSRV silently creeps upward after a transitive dep major-version bump; the declared rust-version diverges from the actually-required floor.",
    references = ["ADR-024", "ADR-024#Family-2"]
)]
pub struct MsrvCreepAfterMajorVersionBump;

// ============================================================================
// #[itch] — a below-threshold noticing.
// ============================================================================

/// We keep seeing the CI Rust-version matrix fail right after `cargo update`.
/// Not yet sure it's worth a formal antigen — but it itches.
#[itch(
    name = "msrv-matrix-flake-after-update",
    antigen = MsrvCreepAfterMajorVersionBump,
    description = "CI's oldest-supported-Rust job fails intermittently right after cargo update; pattern noticed but not yet anchored.",
    threshold = "3 occurrences across 2 releases would warrant an anchor"
)]
pub fn ci_msrv_matrix_job() {
    // Simulated CI gate body.
}

// ============================================================================
// #[recurrence_anchor] — cross-substrate recurrence, threshold reached.
// ============================================================================

/// It crossed the threshold: MSRV crept three times across major bumps. We
/// formally anchor the recurrence so the next occurrence is recognized, not
/// re-discovered.
#[recurrence_anchor(
    MsrvCreepAfterMajorVersionBump,
    instances = 3,
    since = "v0.1.0",
    rationale = "MSRV crept upward in v0.1.4, v0.2.1, and v0.3.0 — each after a transitive major bump; anchoring so the pattern is recognized on recurrence."
)]
pub fn msrv_floor_guard() {
    // Simulated guard that pins the MSRV floor.
}

// ============================================================================
// #[crystallize] — itch cluster promotes to named failure-class.
// ============================================================================

/// The matrix-flake itch + two sibling noticings crystallized into the
/// formal antigen above.
#[crystallize(
    name = "msrv-creep",
    from_itches = [ci_msrv_matrix_job],
    antigen = MsrvCreepAfterMajorVersionBump,
    summary = "Crystallized from the CI-matrix-flake itch once the cross-release pattern became undeniable."
)]
pub fn msrv_creep_crystallization() {}

// ============================================================================
// #[chronic] — low-level persistent, NOT cross-substrate.
// ============================================================================

/// A single flaky integration test that has never been fully fixed — a
/// chronic low-level signal local to one substrate, distinct from the
/// cross-substrate MSRV recurrence.
#[chronic(
    antigen = MsrvCreepAfterMajorVersionBump,
    since = "2026-01-15",
    status = "flaky ~2% of runs; retry masks it",
    managed_by = "ci-team"
)]
pub fn flaky_retry_buffer_test() {}

// ============================================================================
// #[saturate] — accumulating evidence toward a threshold.
// ============================================================================

/// Each new occurrence adds saturation evidence toward the recurrence anchor.
#[saturate(
    antigen = MsrvCreepAfterMajorVersionBump,
    contributing_to = "msrv-creep",
    description = "Fourth occurrence observed in v0.3.2; saturating the recurrence-anchor evidence."
)]
pub fn msrv_occurrence_v032() {}

// ============================================================================
// #[strand] — a thread of related noticing.
// ============================================================================

/// A strand connecting the MSRV-creep noticings with a sibling thread about
/// lockfile churn — they may share a root cause in transitive-dep policy.
#[strand(
    name = "transitive-dep-policy-drift",
    anchored_by = [ci_msrv_matrix_job, msrv_floor_guard],
    description = "Thread connecting MSRV-creep and lockfile-churn noticings; both may root in lax transitive-dependency version policy."
)]
pub fn transitive_dep_policy_strand() {}

fn main() {
    println!("=== antigen recurrent-emergence family example ===");
    println!();
    println!("Six present-looking primitives demonstrated on the MSRV-creep");
    println!("cross-project-lifetime recurrence:");
    println!();
    println!("  #[itch]              — ci_msrv_matrix_job (below-threshold noticing)");
    println!("  #[recurrence_anchor] — msrv_floor_guard (threshold reached, anchored)");
    println!("  #[crystallize]       — msrv_creep_crystallization (itch → named class)");
    println!("  #[chronic]           — flaky_retry_buffer_test (persistent, NOT cross-substrate)");
    println!("  #[saturate]          — msrv_occurrence_v032 (accumulating evidence)");
    println!("  #[strand]            — transitive_dep_policy_strand (thread of noticing)");
    println!();
    println!("Run `cargo antigen scan` to surface these markers; `cargo antigen audit`");
    println!("to see recurrent audit hints (itch-noticed-not-anchored when an itch lacks");
    println!("an antigen path, chronic-signal-unmanaged when chronic lacks managed_by, etc.).");

    ci_msrv_matrix_job();
    msrv_floor_guard();
    msrv_creep_crystallization();
    flaky_retry_buffer_test();
    msrv_occurrence_v032();
    transitive_dep_policy_strand();
}
