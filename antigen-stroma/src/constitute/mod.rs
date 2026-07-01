//! CONSTITUTE: the base-population verb.
//!
//! To populate the base, call [`constitute(db, &witness)`](constitute) with a [`SourceWitness`]. There
//! is no separate `write()` — on the compose base, **write collapses into constitute** (ADR-067 §F1):
//! to constitute a derived node IS to write it, re-deriving what the sources determine. So the base
//! exposes two verbs, not three:
//!
//! - `read(coord) -> snapshot query`  (the 3-axis `ReadCoord`, point-wise, tier-capped)
//! - `constitute(sources) -> Node/Edge`  (derive the base from a [`SourceWitness`]; recomputable)
//!
//! A type-state that let you "author" a base node bypassing derivation is unconstructible:
//! `constitute`'s only input is a `SourceWitness`, and there is no authored-bypass constructor.

pub mod adapter;

use crate::base::facts::{EdgeFacts, NodeFacts};
use crate::node::cfg::CfgSet;

/// The witness a node is constituted FROM.
///
/// Makes "constituted, not authored" unconstructible to violate: a `Node` can only enter the base by
/// being derived from a `SourceWitness` — there is no authored-bypass constructor. The four fields
/// are what the constitute pass reads:
///
/// - `scan_report`: the scan feeder (per-attribute vecs → base nodes/edges)
/// - `source_root`: workspace root path — needed to read file bytes for BLAKE3 identity digests
/// - `crate_name`: the crate whose scan output this is (seeds `syntactic_fq_path`'s `crate::` prefix)
/// - `cfg_set`: the active cfg at scan time (`CfgSet::default()` — empty — when a single active
///   config is captured)
///
/// The witness carries the syntactic-tier inputs the constitution reads. A SCIP index (for
/// resolved-tier edges) and cargo metadata (for cfg-collision handling, ADR-070 §4.5 cfg-aware
/// identity) are not part of it.
#[derive(Debug, Clone)]
pub struct SourceWitness {
    /// The scan feeder output (per-attribute vecs the constitute pass lowers into nodes).
    pub scan_report: antigen::scan::ScanReport,
    /// Workspace root for reading source files (needed for BLAKE3 identity-digest computation).
    pub source_root: std::path::PathBuf,
    /// The crate name whose scan report this is (seeds the `crate::` prefix in `syntactic_fq_path`).
    pub crate_name: String,
    /// The active cfg-set at scan time. `CfgSet::default()` (empty) when a single active config is
    /// captured.
    pub cfg_set: CfgSet,
}

/// Constitute the base node-set from sources (recomputable; THIS IS the write on the compose base).
///
/// Lowers the `SourceWitness` into `NodeFacts`/`EdgeFacts` via [`adapter::lower_scan_report`],
/// assigning each node a stable syntactic-tier FQ path + the two digests + cfg-set. Takes
/// `&mut StromaDb` (it ADVANCES the base — the publish side of the borrow-split). There is
/// deliberately NO sovereign-write counterpart.
pub fn constitute(db: &mut crate::db::StromaDb, sources: &SourceWitness) -> (NodeFacts, EdgeFacts) {
    let (nodes, edges) = adapter::lower_scan_report(
        &sources.scan_report,
        &sources.source_root,
        &sources.crate_name,
        &sources.cfg_set,
    );
    // Wrap the plain-value fact payloads as salsa inputs (requires `&mut db`).
    let node_facts = NodeFacts::new(db, nodes);
    let edge_facts = EdgeFacts::new(db, edges);
    (node_facts, edge_facts)
}
