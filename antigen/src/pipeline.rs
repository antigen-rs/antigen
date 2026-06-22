//! The pipeline coordinator (ADR-036) — the single-conductor host + the
//! SEAM-1 / SEAM-2 convergence point.
//!
//! This is the library-side orchestrated entry point the ADR-036 §Decision +
//! §Open-mechanism-choice call for (library-side, so a
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
//! **scan-time marked-unknown half is now WIRED** (the ADR-041 marker phase): each
//! `#[aura]`/`#[dread]`/`#[red_flag]` the scan surfaces lands as a
//! `FindingBody::MarkedUnknown` in the unified population. The **audit-time
//! dial-verdict half is still the deferred ADR-039 dial phase** — the merge-locus
//! exists so it lands for ~free; until then the population carries only the
//! marked-unknown half.

use std::path::Path;

use crate::audit::orchestrate::{self, AuditBundle};
use crate::finding::Finding;
use crate::scan::{ScanReport, scan_workspace};

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
    /// markers (WIRED, the ADR-041 marker phase) merged with audit-time dial
    /// verdicts (the deferred ADR-039 dial phase). Currently carries the
    /// marked-unknown half; the dial half lands at the same merge-locus when wired.
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
    // SEAM-1, scan half (ADR-041 marker phase — WIRED): the scan stage's
    // `#[aura]`/`#[dread]`/`#[red_flag]` marked-unknown markers merge UP into the
    // unified population first, each as a `FindingBody::MarkedUnknown` (authored,
    // encountered, active — ADR-041 §Emit-seam). A monotonic per-record timestamp
    // (index-based) keeps the records ordered + distinct without a clock dependency
    // in this pure sequencer; a real wall-clock emit is a downstream concern.
    for (i, mu) in scan.marked_unknowns.iter().enumerate() {
        findings.push(mu.to_finding(i as u64));
    }

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
    // ADR-039 dial phase; the merge-at-audit locus is established here now.
    // `findings` is intentionally the empty unified population until those phases
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

    #[test]
    fn marked_unknown_markers_emit_into_the_finding_population() {
        // SEAM-1 scan-half (ADR-041 marker phase): the scan-surfaced markers land as
        // FindingBody::MarkedUnknown in the unified population. Point at the marker
        // fixture (read-as-text by the scan walk).
        use crate::finding::{ExistenceCertainty, FindingBody, Magnitude};
        let mut control = RunControl::new();
        let fixture = Path::new("tests")
            .join("fixtures")
            .join("marked_unknown_markers");
        let out = run(&fixture, &mut control).expect("pipeline runs over the fixture");
        assert!(out.completed);

        // Three markers in the fixture → three MarkedUnknown findings (the audit half
        // is still the deferred dial phase, so the population is exactly these three).
        let markers: Vec<_> = out
            .findings
            .iter()
            .filter(|f| matches!(f.body, FindingBody::MarkedUnknown { .. }))
            .collect();
        assert_eq!(
            markers.len(),
            3,
            "three markers emit into the population; got: {:?}",
            out.findings
        );

        // The red_flag (existence_certainty = Sure) auto-escalates to High severity,
        // and its fixed corner survived the scan → Finding round-trip.
        let red_flag = out
            .findings
            .iter()
            .find(|f| f.source.ends_with("red-flag"))
            .expect("a red-flag finding");
        assert_eq!(red_flag.severity, crate::finding::Severity::High);
        assert!(matches!(
            red_flag.body,
            FindingBody::MarkedUnknown {
                existence_certainty: ExistenceCertainty::Sure,
                ..
            }
        ));

        // An aura is the low-magnitude corner (Magnitude::Aura), Medium severity.
        let aura = out
            .findings
            .iter()
            .find(|f| f.source.ends_with("aura"))
            .expect("an aura finding");
        assert!(matches!(
            aura.body,
            FindingBody::MarkedUnknown {
                magnitude: Magnitude::Aura,
                ..
            }
        ));

        // Every emitted marker carries its required, non-empty trigger (guard 3).
        for f in &markers {
            if let FindingBody::MarkedUnknown { trigger, .. } = &f.body {
                assert!(!trigger.trim().is_empty(), "guard 3: non-empty trigger");
            }
        }
    }
}
