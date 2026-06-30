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

/// The coarse-filesystem safety margin (1 second).
///
/// FAT32 and some other filesystems have a 2-second mtime granularity — a real post-index edit that
/// happens to land in the same 2-second window reads as `source_mtime == index_mtime` under a strict
/// `>` comparison, producing a false-FRESH verdict. The >=1s guard band (ADR-070 §4.9, attack A7)
/// demotes whenever `source_mtime + COARSE_FS_MARGIN >= index_mtime`: same-second ties are
/// ALWAYS treated as potentially-stale (the conservative/safe direction).
const COARSE_FS_MARGIN: u64 = 1;

/// Prior baseline intent: `StromaFaithfullyReflectsReality`. The witness that checks form-fidelity.
///
/// Checks ONLY `fs::metadata` (mtime) — NEVER r-a/SCIP output. The tool-independence is enforced
/// by SIGNATURE: `check` takes raw `u64` seconds since epoch (from `fs::metadata().modified()`)
/// and has NO parameter through which a tool handle could enter. If a builder made the witness
/// tool-dependent, the ATK in `tests/atk_frame_fidelity.rs` would fail to compile (the fn-pointer
/// coercion asserts `(u64, u64, ResolutionTier) → ResolutionTier` from day one).
pub struct FidelityWitness;

impl FidelityWitness {
    /// Check whether the index is fresh relative to the source. Returns the (possibly demoted) tier.
    ///
    /// **Demotion rule:** if `source_mtime + COARSE_FS_MARGIN >= index_mtime`, the index is
    /// potentially stale — demote `Resolved` → `Syntactic` (presents-grade → dread-grade). The >=1s
    /// guard band closes the same-second false-FRESH window on coarse-granularity filesystems
    /// (ADR-070 §4.9, attack A7).
    ///
    /// Demotion happens AT INGESTION TIME, not query time (adversarial A3) — so a stale-SCIP edge
    /// can never enter the base at presents-grade and later corroborate up.
    ///
    /// `T3Mir` is demoted just like `Resolved` (it is equally stale if the index is stale — a higher
    /// tier is not a freshness exemption). `Syntactic` is tier-passthrough (already the floor).
    #[must_use]
    pub const fn check(
        source_mtime: u64,
        index_mtime: u64,
        claimed: ResolutionTier,
    ) -> ResolutionTier {
        // Conservative: stale if source_mtime + margin >= index_mtime.
        // Same-second (source == index) ties → stale (the guard band closes the FAT32 window).
        let stale = source_mtime.saturating_add(COARSE_FS_MARGIN) >= index_mtime;

        if stale {
            // Demote to the syntactic floor (dread-grade). A stale index cannot support a higher
            // tier claim — the confident-wrong window the fidelity witness closes.
            ResolutionTier::Syntactic
        } else {
            // Fresh: pass the claimed tier through unchanged.
            claimed
        }
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
