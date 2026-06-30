//! STEP 5 — `FidelityWitness` + `StromaFidelityUnwitnessed` (the 009-kernel amendment).
//!
//! Observational-autoimmunity: ADR-067 §F3 ratifies this INSIDE the stroma ADR — there is NO
//! standalone section (WATCH-C1 newcomer-trap: grepping "ADR-009 Amendment" misses it; the converge
//! guidance adds an explicit pointer).
//!
//! THE INVARIANT: the form (not the content) of the stroma must be witnessed TOOL-INDEPENDENTLY —
//! filesystem `mtime`, NEVER rust-analyzer/SCIP output. If the index is older than the source, a
//! `presents`-grade resolved edge is DEMOTED to `dread`-grade. The failure this closes:
//! observational-autoimmunity — the system confidently wrong with NO error signal.

use crate::read::ResolutionTier;

/// Prior baseline intent: `StromaFaithfullyReflectsReality`. The witness that checks form-fidelity.
///
/// **STUB — fill (frame epoch):** [`FidelityWitness::check`] touches ONLY `fs::metadata` (mtime) —
/// it must NEVER read r-a/SCIP output (that would make the witness tool-DEPENDENT, defeating it).
pub struct FidelityWitness;

impl FidelityWitness {
    /// Check whether the index is fresh relative to the source. Returns the (possibly demoted) tier.
    /// **STUB — fill (frame epoch):** if `source_mtime > index_mtime`, demote `Resolved -> Syntactic`
    /// (presents-grade -> dread-grade) — the always-on index-staleness guard (ADR-069 Open-seam-1).
    /// Touch ONLY `fs::metadata`.
    ///
    /// SAFETY MARGIN (adversarial A7): coarse-mtime filesystems (FAT32, 2s granularity) report a
    /// false-Fresh on same-second edits. Add a >=1s safety margin to the comparison — treat
    /// `source_mtime >= index_mtime - MARGIN` as potentially-stale (demote), not just strictly-newer.
    /// Demotion happens AT INGESTION TIME, not query time (adversarial A3) — so a stale-SCIP edge can
    /// never enter the base at presents-grade and later corroborate up.
    #[must_use]
    pub fn check(_source_mtime: u64, _index_mtime: u64, claimed: ResolutionTier) -> ResolutionTier {
        let _ = claimed;
        todo!(
            "frame epoch: mtime-only freshness check w/ >=1s coarse-fs margin; demote at ingestion"
        )
    }
}

/// The born-red presents-trigger: `StromaFidelityUnwitnessed` (form-not-content).
///
/// Fires when a resolved edge would carry `presents`-grade but the index is stale (or
/// SCIP-reconstruction is unverified). Two named triggers: (a) index-staleness; (b)
/// SCIP-reconstruction-unverified (see
/// [`crate::scip`]). **STUB — the ATK test asserts this fires; the type carries the unwitnessed signal.**
#[derive(Debug, Clone)]
pub struct StromaFidelityUnwitnessed {
    /// Why the fidelity could not be witnessed (stale-index | scip-unverified).
    pub reason: UnwitnessedReason,
}

/// Why a stroma edge's fidelity is unwitnessed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnwitnessedReason {
    /// The source file is newer than the index (mtime witness).
    StaleIndex,
    /// A SCIP edge whose reconstruction was not verified (macro-call-site degenerate input).
    ScipUnverified,
}
