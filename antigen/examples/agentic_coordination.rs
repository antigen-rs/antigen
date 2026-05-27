// ADR-029 deprecation-window: this example uses the deprecated-but-functional
// #[immune] API (and antigen fns that present migrated antigens). Full migration
// to the #[defended_by]/#[presents(requires=)] idiom is a tracked follow-on
// examples-quality pass; allow the deprecation here meanwhile.
#![allow(deprecated)]

//! Example: agentic-coordination failure-classes.
//!
//! Failure-classes that emerge specifically in multi-session, multi-agent,
//! or human-LLM-collab development workflows. These classes are rare-to-
//! nonexistent in single-developer single-session work; they emerge when
//! representation (agent context state, coordination substrate, stub API
//! surface) diverges from actual state across session or agent boundaries.
//!
//! This example demonstrates:
//!
//! - `AgentWakeWithoutSubstrateDeltaInjection` — an agent session-start
//!   function that routes work BEFORE reading the substrate delta accumulated
//!   while the agent was idle. The context summary the agent holds is a
//!   SNAPSHOT from the previous session, not the current substrate state.
//!
//! - `DelegateCrossCrateResolutionGap` — a mucosal audit that resolves
//!   delegate handlers using an intra-crate name index, silently producing
//!   `MucosalDisciplineDelegateTargetMissing` for cross-crate delegates
//!   that EXIST but can't be reached by the index.
//!
//! ## Why `SubstrateAlignment` (not `FunctionalCorrectness`)
//!
//! Both antigens are category `SubstrateAlignment`: the agent's representation
//! (context state, resolution index) diverges from actual substrate state (the
//! git log, camp status, cross-crate handler graph) across session or crate
//! boundaries. The failure is not in what the code COMPUTES — it is in what
//! the agent BELIEVES is true. The computation is correct given its inputs;
//! the inputs are stale.
//!
//! ## Run this example
//!
//! ```sh
//! cargo run --example agentic_coordination --package antigen
//! ```
//!
//! Scan the examples directory to see the presentations detected:
//!
//! ```sh
//! cargo run --bin cargo-antigen -- antigen scan --root antigen/examples
//! ```

#![allow(dead_code, unused_variables, unused_imports)]

use antigen::stdlib::agentic_coordination::{
    AgentWakeWithoutSubstrateDeltaInjection, DelegateCrossCrateResolutionGap,
};
use antigen::{immune, presents};

// ============================================================================
// AgentWakeWithoutSubstrateDeltaInjection
// ============================================================================

/// A stub type representing the team's work-tracking substrate.
/// In a real team, this could be a camp substrate, a task queue, or
/// any representation of pending and completed work.
pub struct WorkSubstrate {
    /// Snapshot of "what's pending" — stale if not refreshed at wake.
    pub pending_items: Vec<String>,
    /// Whether the delta (what changed while idle) has been injected.
    pub delta_injected: bool,
}

impl WorkSubstrate {
    /// Returns a `WorkSubstrate` whose delta has not yet been injected — simulates a stale session-start.
    pub fn new_stale_snapshot() -> Self {
        Self {
            pending_items: vec![
                "implement feature X".to_string(),
                "write tests for Y".to_string(),
            ],
            delta_injected: false,
        }
    }

    /// Simulates reading the substrate delta and removing work items that shipped while idle.
    pub fn inject_delta(&mut self) {
        // In real usage: read git log, camp status, substrate delta.
        // Here: simulate discovering that "feature X" was actually shipped.
        self.pending_items
            .retain(|item| !item.contains("feature X"));
        self.delta_injected = true;
    }
}

/// Session-start function that routes work based on STALE context.
///
/// This is the vulnerable pattern: the function begins routing without
/// first reading the substrate delta. The agent's summary says "feature X
/// pending" — but a teammate shipped it while this agent was idle. The
/// agent re-does the work, routes stale claims, or blocks on cleared gates.
///
/// The failure is silent: the function does not error. It acts confidently
/// on its snapshot. The only signal is downstream wrong behavior.
#[presents(AgentWakeWithoutSubstrateDeltaInjection)]
pub fn resume_from_snapshot(substrate: &WorkSubstrate) -> Vec<String> {
    // Route directly from the snapshot WITHOUT injecting the delta.
    // substrate.delta_injected is false — stale state used as routing input.
    substrate.pending_items.clone()
}

/// Session-start function that injects the substrate delta FIRST.
///
/// This is the defended pattern: read git log + camp status before
/// routing. The defense is a discipline, not a code invariant — the
/// agent MUST run `camp wake` and read `git log` before any routing.
/// A ratified wake-protocol doc is the substrate-witness for this claim.
///
/// `requires = ratified_doc(...)` evaluates at audit time by checking
/// that the specified doc exists and is current in the `.attest/` sidecar.
/// The doc names the discipline: run `camp wake` + `git log --oneline -20`
/// before claiming any work item.
#[immune(
    AgentWakeWithoutSubstrateDeltaInjection,
    requires = ratified_doc(path = "docs/agentic-wake-protocol.md", min_version = "1.0"),
    rationale = "wake-protocol doc mandates substrate-delta injection before \
                 routing; this function follows the protocol."
)]
pub fn resume_with_delta(substrate: &mut WorkSubstrate) -> Vec<String> {
    substrate.inject_delta(); // inject first, route second
    substrate.pending_items.clone()
}

// ============================================================================
// DelegateCrossCrateResolutionGap
// ============================================================================

/// A stub audit function that resolves mucosal delegate targets using
/// an intra-crate index.
///
/// The structural gap: `audit_mucosal` builds its handler index from
/// `#[mucosal]` declarations in the current `ScanReport`. A cross-crate
/// handler (`handled_by = other_crate::sanitize_request`) is NOT in the
/// index — the audit emits `MucosalDisciplineDelegateTargetMissing` even
/// though the handler exists. The antigen for centralizing defense
/// (ADR-027 `#[mucosal_delegate]`) produces false-positives at exactly
/// the cross-crate boundaries it incentivizes.
#[presents(DelegateCrossCrateResolutionGap)]
pub fn audit_mucosal_delegates(report: &ScanReport) -> Vec<AuditFinding> {
    let mut findings = Vec::new();
    for delegate in &report.delegates {
        // Intra-crate only: cross-crate handlers silently appear "missing"
        if !report.local_handlers.contains(&delegate.target) {
            findings.push(AuditFinding {
                item: delegate.item_path.clone(),
                hint: "mucosal-discipline-delegate-target-missing".to_string(),
            });
        }
    }
    findings
}

/// Stub types for the audit example above.
pub struct ScanReport {
    /// Mucosal delegate declarations found by the scanner.
    pub delegates: Vec<Delegate>,
    /// Handler names resolvable within the current crate.
    pub local_handlers: Vec<String>,
}

/// A mucosal delegate declaration — an item that routes to a handler in another crate.
pub struct Delegate {
    /// Source path of the item that declares the delegate.
    pub item_path: String,
    /// Fully-qualified handler target (may be cross-crate).
    pub target: String,
}

/// A single audit finding emitted when a delegate target cannot be resolved.
pub struct AuditFinding {
    /// Source item that produced the finding.
    pub item: String,
    /// Audit hint string for the finding.
    pub hint: String,
}

fn main() {
    println!("=== antigen agentic-coordination example ===");
    println!();
    println!("1. AgentWakeWithoutSubstrateDeltaInjection");
    println!("   Category: SubstrateAlignment");
    println!(
        "   The agent's context snapshot diverges from substrate state across session boundary."
    );
    println!();

    // Demonstrate the vulnerable path
    let snapshot = WorkSubstrate::new_stale_snapshot();
    let stale_work = resume_from_snapshot(&snapshot);
    println!("   resume_from_snapshot (PRESENTS — stale routing):");
    println!("   pending: {:?}", stale_work);
    println!("   → 'implement feature X' is listed but may have shipped while idle.");
    println!();

    // Demonstrate the defended path (without a real sidecar — requires= evaluates at audit time)
    let mut substrate = WorkSubstrate::new_stale_snapshot();
    let fresh_work = resume_with_delta(&mut substrate);
    println!("   resume_with_delta (IMMUNE — delta injected first):");
    println!("   pending after delta: {:?}", fresh_work);
    println!("   → 'implement feature X' correctly removed (was shipped while idle).");
    println!();

    println!("2. DelegateCrossCrateResolutionGap");
    println!("   Category: SubstrateAlignment");
    println!("   Intra-crate handler index silently misses cross-crate delegates.");
    println!();

    let report = ScanReport {
        delegates: vec![Delegate {
            item_path: "api::process_request".to_string(),
            target: "sanitizer_crate::sanitize_request".to_string(), // cross-crate
        }],
        local_handlers: vec![
            "local_sanitize".to_string(), // only local handlers indexed
        ],
    };
    let findings = audit_mucosal_delegates(&report);
    println!(
        "   audit_mucosal_delegates (PRESENTS — cross-crate handler {:?} reported missing):",
        findings.iter().map(|f| f.hint.as_str()).collect::<Vec<_>>()
    );
    println!(
        "   → false MucosalDisciplineDelegateTargetMissing despite handler existing cross-crate."
    );
    println!();

    println!("Scan for presentations:");
    println!("  cargo run --bin cargo-antigen -- antigen scan --root antigen/examples");
}
