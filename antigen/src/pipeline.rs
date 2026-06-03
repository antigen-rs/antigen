//! The pipeline coordinator (ADR-036) — the single-conductor host + the
//! SEAM-1 / SEAM-2 convergence point.
//!
//! This is the library-side orchestrated entry point the ADR-036 §Decision +
//! §Open-mechanism-choice call for (the pathmaker's lean: library-side, so a
//! runtime-sensor or external-platform consumer — charter — gets the orchestrated
//! loop, not a re-implementation of `main.rs`'s fan-out). It runs the
//! **scan → audit** pass sequence and is the one place that:
//!
//! - **(SEAM-2, ADR-036 §The out-of-band invariant) holds the sole stop-authority.**
//!   The authority to stop the run lives HERE — in the coordinator — and in no
//!   detector/pass the run might run away in. A runaway is a unit producing more
//!   work than the loop can damp; if that unit also held stop-authority it would
//!   disable its own brake. The detectors are pure fns of their input (the purity
//!   invariant — checkable, strictly stronger than single-conductor), so a
//!   population of pure detectors provably leaves all stop-authority with whoever
//!   sequences them: this coordinator. A future cascade-governor (charter) is
//!   inserted HERE as a separate stage that consumes the emitted population and
//!   can [`RunControl::trip_scram`](crate::pipeline::RunControl::trip_scram) from
//!   *outside* any detector. **No detector module imports or references
//!   [`RunControl`](crate::pipeline::RunControl)** — the sub-clause-F (ADR-005)
//!   check for this boundary; dependency flows one way (coordinator → passes,
//!   never the reverse).
//!
//! - **(SEAM-1, ADR-039 §C) is the merge-locus** where the unified
//!   [`Finding`](crate::finding::Finding) population is assembled. The scan stage
//!   emits its `#[dread]`/`#[aura]` half; the audit stage emits its dial-verdict
//!   half; they merge HERE (the audit stage is the last that sees both halves).
//!   This module does **not define** a `Finding` type — ADR-039 (`crate::finding`)
//!   is the sole schema owner; the coordinator only *lands* that schema into one
//!   population (so there is no second-schema `ParallelStateTrackersDiverge`).
//!
//! The two seams converge on this one population-complete coordination layer
//! (ADR-036 §The two seams converge): a governor damps an aggregate over the
//! population, and an aggregate can only be computed where the population is
//! complete — the same vantage the emit-merge needs.
//!
//! Behavior-preserving: `run` composes the *existing* `scan_workspace` +
//! `audit::orchestrate::run`; it changes nothing about what they compute. The
//! CLI's hand-rolled fan-out can adopt this entry point incrementally. The
//! `Finding` population starts empty — the per-stage emit (scan computing its
//! marked-unknown half, audit its dial-verdict half) lands as the ADR-039/041
//! family + marker waves populate it; the seam (schema + merge-locus +
//! stop-authority host) is what is banked now, near-free, so those waves are
//! buildable later for ~free.

use std::path::Path;

use crate::audit::orchestrate::{self, AuditBundle};
use crate::finding::Finding;
use crate::scan::{scan_workspace, ScanReport};

/// The sole stop-authority handle (ADR-036 §The out-of-band invariant — SEAM-2).
///
/// Lives in the coordinator, ABOVE the pass sequence. A pass/detector NEVER holds
/// one (the sub-clause-F check: no detector module references it). A future
/// cascade-governor trips it via [`Self::trip_scram`] from *above* the loop; the
/// coordinator checks [`Self::should_continue`] between stages. The
/// single-conductor invariant is the *type* boundary: a pass cannot stop the run
/// because it never receives a `RunControl`.
#[derive(Debug, Default)]
pub struct RunControl {
    scram: bool,
}

impl RunControl {
    /// A fresh control with the run permitted to proceed.
    #[must_use]
    pub const fn new() -> Self {
        Self { scram: false }
    }

    /// Trip the emergency stop. Called by a future cascade-governor (charter) from
    /// ABOVE the loop — never by a detector. After this, [`Self::should_continue`]
    /// returns `false` and the coordinator aborts between stages.
    pub const fn trip_scram(&mut self) {
        self.scram = true;
    }

    /// Whether the run may proceed to the next stage. The coordinator checks this
    /// between stages; a tripped SCRAM short-circuits the run out-of-band (no
    /// pass cooperation needed — the passes are pure + control-blind).
    #[must_use]
    pub const fn should_continue(&self) -> bool {
        !self.scram
    }
}

/// The complete output of one pipeline run — the scan report, the audit bundle,
/// and the unified [`Finding`] population (SEAM-1) assembled at the merge-locus.
#[derive(Debug)]
pub struct PipelineRun {
    /// The scan report (pass 1's output — the declarations + synthesized matches).
    pub scan: ScanReport,
    /// The audit bundle (the per-detector reports the audit sequence produced).
    pub audit: AuditBundle,
    /// The unified typed-event population (ADR-039 §C) — scan-time marked-unknown
    /// markers merged with audit-time dial verdicts. Empty until the ADR-039/041
    /// emit waves populate it; the merge-locus exists now so they land for ~free.
    pub findings: Vec<Finding>,
    /// `true` when the run completed; `false` when a SCRAM tripped it early. The
    /// scan half (if produced before the trip) is still returned.
    pub completed: bool,
}

/// Run the full **scan → audit** pipeline against `root`, under the coordinator's
/// sole stop-authority, assembling the unified [`Finding`] population.
///
/// This is the single-conductor sequencer (ADR-036): it owns the stage ORDER +
/// the stop-authority; each stage is a pure fn it calls, and no stage triggers
/// the next (every stage returns HERE, which decides what runs next — the forced
/// stage-sequencing invariant). The scan walk's `Err` (a hard scan failure) is
/// propagated; everything else is behavior-preserving composition of the existing
/// `scan_workspace` + `audit::orchestrate::run`.
///
/// # Errors
/// Propagates the `scan_workspace` IO error (a hard scan failure).
pub fn run(root: &Path, control: &mut RunControl) -> std::io::Result<PipelineRun> {
    // SEAM-1 merge starts as one empty population the stages emit into.
    let mut findings: Vec<Finding> = Vec::new();

    // Stage 1 — scan (the collection walk + lineage + finalize passes). A pure-ish
    // directory scanner; it returns to the coordinator, never triggering audit.
    let scan = scan_workspace(root, None)?;
    // SEAM-1, scan half: the scan stage's `#[dread]`/`#[aura]` marked-unknown
    // markers merge UP into the unified population first. The population starts
    // empty — the scan-side emit lands when the ADR-041 marker wave wires it; the
    // MERGE POINT is here regardless (banking the locus is the near-free seam).

    // SEAM-2: the stop-authority is the coordinator's; a tripped SCRAM aborts
    // BETWEEN stages, out-of-band — no stage cooperation needed.
    if !control.should_continue() {
        return Ok(PipelineRun {
            scan,
            audit: AuditBundle::default(),
            findings,
            completed: false,
        });
    }

    // Stage 2 — audit (the detector sequence, run by the thin audit-side
    // sequencer). Pure fns of `&scan`; the coordinator holds the authority to
    // stop, the detectors hold none.
    let audit = orchestrate::run(&scan, root);
    // SEAM-1, audit half: the audit stage's dial verdicts merge into the SAME
    // population (audit is the last stage that sees both halves). Wired by the
    // ADR-039 dial wave; the merge-at-audit locus is established here now.
    // `findings` is intentionally the empty unified population until those waves
    // populate it — see the module doc.
    let _ = &mut findings;

    // The future cascade-governor (charter) would run HERE as a separate stage:
    // inspect the population-complete `findings`, and if it is a storm,
    // `control.trip_scram()` — from outside any detector. The hook is the
    // population-complete vantage this layer uniquely has.

    Ok(PipelineRun {
        scan,
        audit,
        findings,
        completed: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scram_tripped_above_the_loop_aborts_before_audit() {
        // SEAM-2: trip the stop from ABOVE, run, and confirm the audit stage never
        // runs (the bundle is the default empty one) — out-of-band, no pass
        // cooperation. We point at this crate's own src so the scan walk has real
        // input; the assertion is about the STOP, not the scan contents.
        let mut tripped = RunControl::new();
        tripped.trip_scram();
        let out = run(Path::new("src"), &mut tripped).expect("scan walk succeeds");
        assert!(!out.completed, "a tripped SCRAM must abort the run");
        // The audit bundle is the default (audit never ran) — the conductor stopped
        // the run from above the detector loop.
        assert!(out.audit.audit.audits.is_empty());
    }

    #[test]
    fn uninterrupted_run_completes_and_runs_both_stages() {
        let mut control = RunControl::new();
        let out = run(Path::new("src"), &mut control).expect("pipeline runs");
        assert!(out.completed, "an untripped run completes");
        // Behavior-preserving: this is the same scan + audit the CLI runs; we only
        // assert the run reached the audit stage (the conductor sequenced both).
        // (The scan of this crate's own src finds antigen's dogfood declarations.)
        let _ = &out.scan;
    }
}
