//! STEP 7 — the WRITE taxonomy.
//!
//! On the compose base, `write ∈ constitute` (no separate write verb); this module is the
//! OUTPUT-substrate disposition of a written MARKER (the 4-cell taxonomy, ADR-067 §F / Open-seam-4),
//! used by organs that emit markers ONTO the base, not by the base itself.
//!
//! Overlay (SCIP-symbol-anchored) is PRIMARY for DERIVED markers (survives edit+rebase, severable).
//! Injection is ONLY for the DECLARED `#[antigen_tolerance]` exception. An injected marker with an
//! overlay counterpart MUST be DERIVED FROM the overlay — enforced by construction below.
//!
//! WRITE-BACK (mutating the user's source) is THE safety boundary — one of the four NEVER-DONE moves,
//! GATED, never the default. Not in frame scope to implement; named so it isn't built by accident.

/// A written marker's output-substrate disposition.
pub enum Write {
    /// D1 / MATERIALIZED-D1: derived, recomputable, SCIP-anchored. The safe default (F4 primary).
    Overlay(OverlayMarker),
    /// TRUE-EMBED: the DECLARED `#[antigen_tolerance]` exception. Constructible ONLY from an overlay.
    Injected(InjectedException),
}

/// A derived overlay marker, anchored on the SCIP symbol (survives edit+rebase). **STUB — fill.**
pub struct OverlayMarker {
    // STUB: scip_symbol anchor, the derived marker payload, tier
}

/// The declared embedded exception. The ONLY constructor is [`InjectedException::from_overlay`] —
/// making "injected must derive from overlay" unconstructible to violate (ADR-067 Open-seam-4).
pub struct InjectedException {
    _private: (),
}

impl InjectedException {
    /// The sole constructor — an injected exception is a PROJECTION of an overlay, never an
    /// independent source. **STUB — fill (frame epoch):** derive the embedded form from the overlay.
    #[must_use]
    pub fn from_overlay(_overlay: &OverlayMarker) -> Self {
        todo!("frame epoch: injected exception is a projection of the overlay (never independent)")
    }
}
