//! The WRITE taxonomy — how a written marker is placed relative to the user's source.
//!
//! On the compose base, `write` collapses into `constitute` (no separate write verb); this module is
//! the output-substrate disposition of a written MARKER (ADR-067 §F), used by organs that emit
//! markers ONTO the base, not by the base itself.
//!
//! Overlay (SCIP-symbol-anchored) is primary for derived markers: it survives edit+rebase and is
//! severable. Injection is only for the declared `#[antigen_tolerance]` exception, and an injected
//! marker with an overlay counterpart must be derived FROM the overlay — enforced by construction
//! (see [`InjectedException::from_overlay`]).
//!
//! Write-back — mutating the user's own source in place — is a gated safety boundary, never the
//! default, and is not implemented here. It is named so it is not reached for by accident.

/// A written marker's output-substrate disposition.
pub enum Write {
    /// D1 / MATERIALIZED-D1: derived, recomputable, SCIP-anchored. The safe default (F4 primary).
    Overlay(OverlayMarker),
    /// TRUE-EMBED: the DECLARED `#[antigen_tolerance]` exception. Constructible ONLY from an overlay.
    Injected(InjectedException),
}

/// A derived overlay marker, anchored on the SCIP symbol (survives edit+rebase).
///
/// An overlay is the anchor an injected exception projects from. It holds no marker payload (the
/// SCIP-symbol anchor, the derived marker, and its tier belong to the marker-emit substrate, not to
/// this type).
pub struct OverlayMarker {
    // No fields: the anchor + derived marker payload + tier belong to the marker-emit substrate.
}

/// The declared embedded exception. The ONLY constructor is [`InjectedException::from_overlay`] —
/// making "injected must derive from overlay" unconstructible to violate (ADR-067 §F).
pub struct InjectedException {
    _private: (),
}

impl InjectedException {
    /// The sole constructor — an injected exception is a PROJECTION of an overlay, never an
    /// independent source (ADR-067 §F). Taking `&OverlayMarker` makes the derivation-witness a TYPE
    /// obligation: you cannot mint an `InjectedException` without an overlay in hand, and the private
    /// `_private` field forbids the struct-literal bypass (a syntactic construction attempt does not
    /// compile).
    ///
    /// The projection is structural: [`OverlayMarker`] carries no payload, so the existence of this
    /// call IS the "derived-from-overlay" provenance — the borrow proves an overlay was in hand.
    #[must_use]
    pub const fn from_overlay(overlay: &OverlayMarker) -> Self {
        // The overlay is the source-of-truth the injected form projects FROM. There is no payload to
        // copy at this layer; the borrow is the provenance obligation made structural.
        let _ = overlay;
        Self { _private: () }
    }
}
