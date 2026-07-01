//! The read coordinate: SOURCE × PERSPECTIVE × POLARITY (ADR-069 §B).

use super::tier::ResolutionTier;
use crate::base::facts::EdgeKind;

/// A point in the read coordinate frame. The three axes the whole stroma is read through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadCoord {
    /// The source-resolution tier (caps the detection grade — see [`ResolutionTier`]).
    pub source: ResolutionTier,
    /// Receptor-selectivity: which edge-kinds this read traverses.
    pub perspective: Perspective,
    /// Intent-default: should-vs-is (the danger-model axis).
    pub polarity: Polarity,
}

/// Which edge-kinds a read traverses (receptor-selectivity).
///
/// The immune metaphor: a receptor binds a specific epitope, a `Perspective` binds a specific
/// [`EdgeKind`]. A read at a given perspective follows ONLY the matching edges; `AllEdges` is the
/// pan-receptor that follows every kind.
///
/// Mirrors the OPEN [`EdgeKind`] registry; `#[non_exhaustive]` so a new edge-kind can accrete a new
/// perspective selector without breaking downstream matches (accrete-never-migrate at the type level).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Perspective {
    /// Follow every edge-kind (the pan-receptor — the default whole-graph traversal).
    AllEdges,
    /// Follow only edges of one specific kind (the selective receptor).
    Only(EdgeKind),
}

impl Perspective {
    /// Whether a read at this perspective traverses an edge of the given kind.
    #[must_use]
    pub fn admits(self, kind: EdgeKind) -> bool {
        match self {
            Self::AllEdges => true,
            Self::Only(selected) => selected == kind,
        }
    }
}

/// Should-vs-is (ADR-069 §B; the danger-model polarity).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Polarity {
    /// What the code *is* (observed).
    Is,
    /// What the code *should* be (intent/contract).
    Should,
}
