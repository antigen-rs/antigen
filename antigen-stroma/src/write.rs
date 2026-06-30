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
    /// independent source (ADR-067 Open-seam-4). Taking `&OverlayMarker` makes the derivation-witness
    /// a TYPE obligation: you cannot mint an `InjectedException` without an overlay in hand, and the
    /// private `_private` field forbids the struct-literal bypass (the born-red trybuild fixture
    /// `injected_exception_needs_overlay.rs` proves the literal does not compile).
    ///
    /// Frame epoch: the overlay carries no payload yet (the marker-emit substrate is the
    /// later-epoch STEP-3 concern), so the projection is structural — the existence of this call IS
    /// the "derived-from-overlay" provenance. When `OverlayMarker` gains its payload, this projects it.
    #[must_use]
    pub const fn from_overlay(overlay: &OverlayMarker) -> Self {
        // The overlay is the source-of-truth the injected form projects FROM. Frame epoch has no
        // payload to copy; the borrow is the provenance obligation made structural.
        let _ = overlay;
        Self { _private: () }
    }
}
