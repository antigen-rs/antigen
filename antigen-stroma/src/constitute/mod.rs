//! STEP 4 — CONSTITUTE: the base-population verb.
//!
//! **The synchronization point** (highest fan-in: read-CONTRACT + node-identity + salsa-shape +
//! relational base must ALL exist first).
//!
//! ADR-067 §F1 (resolved): on the COMPOSE base, **write COLLAPSES into constitute** — to constitute
//! a derived node IS to write it (re-deriving what the input determines). So the frame's base
//! exposes TWO verbs, not three:
//!
//! - `read(coord) -> snapshot query`  (the 3-axis `ReadCoord`, point-wise, tier-capped)
//! - `constitute(sources) -> Node/Edge`  (derive from syn/SCIP/cargo-metadata; recomputable)
//!
//! There is NO `write()` on the base. A type-state that let you "author" a base node bypassing
//! derivation would be the WATCH-C2 partition violation — and it's DISCHARGED here by construction
//! (constitute's only input is `SourceWitness`; there is no authored-bypass constructor).

pub mod adapter;

use crate::base::facts::{EdgeFacts, NodeFacts};

/// The witness a node is constituted FROM.
///
/// Makes "constituted, not authored" unconstructible to violate (WATCH-C2 discharge). **STUB — fill
/// (frame epoch):** carries the scan output + SCIP/cargo-metadata the node derives from.
#[derive(Debug, Clone)]
pub struct SourceWitness {
    // STUB: scan_report ref, scip symbols, cfg-set, cargo-metadata
}

/// Constitute the base node-set from sources (recomputable; THIS IS the write on the compose base).
///
/// **STUB — fill (frame epoch):** lower the `SourceWitness` into `NodeFacts`/`EdgeFacts` via
/// [`adapter`], assigning each node a stable `Locator` (syntactic path, SCIP-refined) + the two
/// digests + cfg-set. Takes `&mut StromaDb` (it ADVANCES the base — the publish side of the
/// borrow-split). There is deliberately NO sovereign-write counterpart.
pub fn constitute(
    _db: &mut crate::db::StromaDb,
    _sources: &SourceWitness,
) -> (NodeFacts, EdgeFacts) {
    todo!(
        "frame epoch: derive the base node/edge facts from sources (write ∈ constitute on compose)"
    )
}
