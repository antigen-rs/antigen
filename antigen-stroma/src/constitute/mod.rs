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
use crate::node::cfg::CfgSet;

/// The witness a node is constituted FROM.
///
/// Makes "constituted, not authored" unconstructible to violate (WATCH-C2 discharge). A `Node` can
/// only enter the base by being derived from a `SourceWitness` — there is no authored-bypass
/// constructor. The four fields encode what the constitute pass needs at frame epoch:
///
/// - `scan_report`: the scan feeder (per-attribute vecs → base nodes/edges)
/// - `source_root`: workspace root path — needed to read file bytes for BLAKE3 identity digests
/// - `crate_name`: the crate whose scan output this is (seeds `syntactic_fq_path`'s `crate::` prefix)
/// - `cfg_set`: the active cfg at scan time (empty/default for frame epoch; cargo-metadata wiring is
///   engine-epoch)
///
/// **ENGINE-EPOCH TODO**: extend with `scip_index: Option<ScipIndex>` and
/// `cargo_metadata: Option<CargoMetadata>` for the resolved-tier feeder and the cfg-collision
/// handling (ADR-070 §4.5 cfg-aware identity).
#[derive(Debug, Clone)]
pub struct SourceWitness {
    /// The scan feeder output (per-attribute vecs the constitute pass lowers into nodes/edges).
    pub scan_report: antigen::scan::ScanReport,
    /// Workspace root for reading source files (needed for BLAKE3 identity-digest computation).
    pub source_root: std::path::PathBuf,
    /// The crate name whose scan report this is (seeds the `crate::` prefix in `syntactic_fq_path`).
    pub crate_name: String,
    /// The active cfg-set at scan time. `CfgSet::default()` (empty) for frame epoch; the
    /// cargo-metadata feeder (engine epoch) populates this from the active feature set.
    pub cfg_set: CfgSet,
}

/// Constitute the base node-set from sources (recomputable; THIS IS the write on the compose base).
///
/// Lowers the `SourceWitness` into `NodeFacts`/`EdgeFacts` via [`adapter::lower_scan_report`],
/// assigning each node a stable syntactic FQ path (SCIP-refined in engine epoch) + the two
/// digests + cfg-set. Takes `&mut StromaDb` (it ADVANCES the base — the publish side of the
/// borrow-split). There is deliberately NO sovereign-write counterpart.
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
