//! The thin audit-side sequencer (ADR-036 §Decision).
//!
//! This module owns ONLY the *order* the audit detectors run in — no detection
//! logic of its own. It gives the `cargo antigen audit` fan-out (previously
//! smeared across `cargo-antigen/src/main.rs`) a home and a name: a single
//! [`run`](crate::audit::orchestrate::run) that drives the detector sequence and
//! bundles each detector's report into one
//! [`AuditBundle`](crate::audit::orchestrate::AuditBundle). The pipeline
//! coordinator (ADR-036 §Decision; the
//! library-side `pipeline.rs` or the CLI) calls this above the scan pass; it is
//! the layer a future cascade-governor's SCRAM sits *above*, never inside.
//!
//! Per the single-conductor invariant (ADR-036 §The out-of-band invariant):
//! each detector is a pure fn of `&ScanReport`; this sequencer holds the
//! authority to order them and (in a future revision) to stop the run, but no
//! detector self-coordinates. Adding the unified `Finding` emit/merge (ADR-039
//! §C SEAM-1) is a later step — this sequencer first names the order.
//!
//! Behavior-preserving: `run` calls exactly the detectors the `audit` command
//! already called, in the same order, with the same arguments; bundling them in
//! a struct changes *where the fan-out lives*, not *what it computes*.

use std::path::Path;

use super::{
    AuditReport, CategoryAuditReport, ConvergentEvidenceAuditReport, CoverageAuditReport,
    DefendedStatusReport, DeferredDefenseAuditReport, LineageFidelityAuditReport,
    PrescriptiveAuditReport, RecurrentAuditReport, audit, audit_category,
    audit_convergent_evidence, audit_coverage, audit_defended_status, audit_deferred_defenses,
    audit_lineage_fidelity, audit_prescriptive, audit_recurrent,
};
use crate::scan::ScanReport;

/// The default stale-grace window (days) past a deferred-defense `until` date.
///
/// Past this, a deferred defense escalates to `anergy-stale` (vs
/// `co-stimulation-not-arrived`). The audit command has always used 30 (the
/// library contract); named here so the sequencer carries the one knob the
/// fan-out passed.
pub const DEFERRED_STALE_GRACE_DAYS: i64 = 30;

/// The bundle of audit reports computed from one scan, in one pass.
///
/// This is the recognized shape of the former `cargo antigen audit` `main.rs`
/// fan-out. Each field is one detector's own report type (the detectors are
/// siblings; none calls another). The CLI renders each; future stages (the
/// ADR-039 unified `Finding` population, a cascade-governor) consume the bundle
/// from *above*, where the whole population is visible.
#[derive(Debug, Default)]
pub struct AuditBundle {
    /// The core immunity audit (witness resolution + per-site immune verdicts).
    pub audit: AuditReport,
    /// ADR-028 antigen-category coverage (G1 defaulted-implicit-functional, G2
    /// category↔witness-type cross-check).
    pub category: CategoryAuditReport,
    /// ADR-023 deferred-defense state (anergy / immunosuppress / poxparty /
    /// orient): active/expired/stale counts + per-declaration hints.
    pub deferred: DeferredDefenseAuditReport,
    /// ADR-024 convergent-evidence audit (`#[diagnostic]` / `#[clonal]` /
    /// `#[igg]` / ...).
    pub convergent: ConvergentEvidenceAuditReport,
    /// ADR-024 recurrent-emergence audit (`#[itch]` / `#[recurrence_anchor]` /
    /// `#[crystallize]` / ...).
    pub recurrent: RecurrentAuditReport,
    /// `#[descended_from]` lineage-fidelity advisory (scientist 2026-05-27).
    pub lineage_fidelity: LineageFidelityAuditReport,
    /// Coverage / reachability audit — the ignorance frontier as per-site
    /// verdicts (the 4th peripheral-tolerance mechanism).
    pub coverage: CoverageAuditReport,
    /// Prescriptive work-orchestration audit (ADR-033) — every work-need
    /// projected to a four-valued `WorkVerdict` ("code IS the board").
    pub prescriptive: PrescriptiveAuditReport,
    /// P2' defended-status sensor — the per-class witness-resolution roll-up the
    /// obsolete/well-defended discriminator reads. Derived from [`Self::audit`]
    /// (the per-immunity witness tiers, rolled up by failure-class), so it adds no
    /// new scan/resolution work — it is a projection of the core audit the bundle
    /// already carries.
    pub defended_status: DefendedStatusReport,
}

/// Run the audit detector sequence and bundle the per-detector reports.
///
/// This is the thin sequencer: it owns the *order* the detectors run in and
/// nothing else. The order matches the established `cargo antigen audit`
/// fan-out exactly (immunity audit first, then the additive family/coverage/
/// prescriptive detectors) — a behavior-preserving recognition of the fan-out
/// that already ran in `main.rs`, now with a name. `root` is the workspace root
/// the immunity + prescriptive detectors read sidecars/function-index from
/// (typically the same path passed to [`crate::scan::scan_workspace`]).
#[must_use]
pub fn run(report: &ScanReport, root: &Path) -> AuditBundle {
    // The core immunity audit runs first; the P2' defended-status roll-up is a
    // projection of it (per-immunity tiers folded by failure-class), so it reads
    // the result rather than re-scanning.
    let core_audit = audit(report, root);
    let defended_status = audit_defended_status(&core_audit);
    AuditBundle {
        audit: core_audit,
        category: audit_category(report),
        deferred: audit_deferred_defenses(report, DEFERRED_STALE_GRACE_DAYS),
        convergent: audit_convergent_evidence(report),
        recurrent: audit_recurrent(report),
        lineage_fidelity: audit_lineage_fidelity(report),
        coverage: audit_coverage(report),
        prescriptive: audit_prescriptive(report, root),
        defended_status,
    }
}
