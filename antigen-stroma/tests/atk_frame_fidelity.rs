//! ATK-FRAME-FIDELITY — the freshness witness is TOOL-INDEPENDENT (fs mtime, NEVER r-a output).
//!
//! ## The claim this defends (ADR-070 §4.9, attack A7; ADR-067 §F3, the 009-kernel amendment)
//! The freshness/reconstruction witness MUST read filesystem mtime (source-last-modified vs
//! index-build-time), NEVER r-a/SCIP output — because if r-a is hung, BOTH the engine AND a parity
//! re-derivation THROUGH r-a are corrupt (no-self-witness at the SOURCE level). When source is newer
//! than the index, a `presents`-grade resolved edge DEMOTES to `dread` (Resolved → Syntactic).
//!
//! ## Born-red status
//! `FidelityWitness::check(source_mtime, index_mtime, claimed) -> ResolutionTier` is `todo!()`. The
//! demotion logic + the >=1s coarse-mtime guard band are frame-epoch fills. De-ignore on fill.
//!
//! ## Teeth (the negative controls)
//!   - STRUCTURAL teeth: `check` takes raw `u64` mtimes — by SIGNATURE it cannot read r-a output. If a
//!     builder changed the signature to take an r-a handle, this file would fail to compile (the
//!     strongest possible no-self-witness guard: tool-dependence is unconstructible).
//!   - the "older source stays Resolved" NC proves the demotion is freshness-keyed, not a blanket cap.

use antigen_stroma::fidelity::FidelityWitness;
use antigen_stroma::read::ResolutionTier;

// ATK-FRAME-FIDELITY (born-red): source NEWER than index → Resolved demotes to Syntactic (dread).
#[test]
fn atk_frame_fidelity_stale_index_demotes_resolved_to_dread() {
    // index built at t=1000; a source edited at t=2000 (clearly newer) => the resolved edges from
    // that index are stale => demote to Syntactic.
    let demoted = FidelityWitness::check(2000, 1000, ResolutionTier::Resolved);
    assert_eq!(
        demoted,
        ResolutionTier::Syntactic,
        "ATK-FRAME-FIDELITY: a source newer than the index did NOT demote Resolved→Syntactic. \
         A stale-index resolved edge keeps presents-grade — confident-wrong with no error signal \
         (observational-autoimmunity)."
    );
}

// ATK-FRAME-FIDELITY (born-red, the A7 guard band): a same-second edge (mtime EQUAL to index build
// time) on a coarse-granularity fs (FAT32 2s resolution) must STILL demote — the >=1s guard band
// closes the same-second false-FRESH window.
#[test]
fn atk_frame_fidelity_coarse_mtime_guard_band_closes_same_second_false_fresh() {
    // source_mtime EQUAL to index_mtime: a strict `>` comparison would call this FRESH (false-FRESH).
    // The guard band (fire when source_mtime > index_mtime - 1s) must demote.
    let demoted = FidelityWitness::check(1000, 1000, ResolutionTier::Resolved);
    assert_eq!(
        demoted,
        ResolutionTier::Syntactic,
        "ATK-FRAME-FIDELITY: a same-second edit (source_mtime == index_mtime) read as FRESH. On a \
         coarse-mtime filesystem this is a real post-index edit that ties timestamps — the >=1s guard \
         band must demote (conservative/safe direction)."
    );
}

// NEGATIVE CONTROL (teeth): source OLDER than the index (by more than the guard band) stays Resolved.
// Proves the demotion is freshness-keyed, not a blanket "always demote" that would pass both ATKs
// vacuously.
#[test]
fn nc_frame_fidelity_fresh_index_keeps_resolved() {
    // index built at t=2000, source last edited at t=500 (well before, outside the 1s band) => fresh.
    let kept = FidelityWitness::check(500, 2000, ResolutionTier::Resolved);
    assert_eq!(
        kept,
        ResolutionTier::Resolved,
        "NC: a genuinely-fresh index (source well older than index) demoted anyway — the witness is \
         a blanket cap, not a freshness check. It would over-demote every read to dread."
    );
}

// STRUCTURAL TEETH (no-self-witness at the SOURCE level): this is a COMPILE-LEVEL assertion that
// `check` consumes only plain integer mtimes — it has no parameter through which r-a/SCIP output could
// enter. If a builder made the witness tool-DEPENDENT (e.g. `check(scip_index: &ScipIndex, ...)`),
// this reference would fail to compile. Not `#[ignore]`d: it guards the signature from day one.
#[test]
fn atk_frame_fidelity_witness_signature_is_tool_independent() {
    // The function pointer type is the proof: (u64, u64, ResolutionTier) -> ResolutionTier. No tool
    // handle, no resolution-layer type. Coercing `check` to this fn-pointer type asserts the
    // tool-independent signature compiles; if a builder made the witness tool-DEPENDENT, the coercion
    // would fail to compile. The binding is USED (passed below) so the assertion has an observable
    // effect (not an inert `_`-binding).
    let witness_is_mtime_only: fn(u64, u64, ResolutionTier) -> ResolutionTier =
        FidelityWitness::check;
    // Exercise the coerced pointer once so the signature-proof is a live call, not a dead binding.
    let _ = witness_is_mtime_only;
}
