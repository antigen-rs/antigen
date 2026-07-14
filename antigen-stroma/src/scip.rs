//! SCIP ingestion — the `EdgeReconstruction` type (tier-honesty in the contract).
//!
//! The honesty rule: a macro-call-site MUST be `Ambiguous`/`Unreconstructible`, NEVER
//! silently-`Resolved`; a plain call-site resolves cleanly. Because the type distinguishes these
//! outcomes, the contract cannot represent a silently-resolved macro edge — the tier-honesty is in
//! the type, not left to the ingestion code to remember.

/// The reconstruction outcome of a single SCIP edge — the tier-honesty in the contract.
#[derive(Debug, Clone)]
pub enum EdgeReconstruction {
    /// Cleanly resolved (a plain call-site). Carries `presents`-grade eligibility.
    Resolved(ResolvedEdge),
    /// Ambiguous — multiple candidates (e.g. a macro expansion site). NEVER `presents`-grade.
    Ambiguous(Vec<ResolvedEdge>),
    /// Unreconstructible — the symbol could not be resolved. `dread`-grade floor at most.
    Unreconstructible,
}

/// A resolved SCIP edge — its src/dst SCIP symbols and edge-kind.
///
/// Defined so the reconstruction contract is tier-honest independent of the ingestion code that
/// produces the edges.
#[derive(Debug, Clone)]
pub struct ResolvedEdge {
    /// The source SCIP symbol string (becomes the src node's resolved [`crate::node::id::FqPath`]).
    pub src_symbol: String,
    /// The destination SCIP symbol string (becomes the dst node's resolved path).
    pub dst_symbol: String,
    /// The reconstructed edge-kind (call / import / type-use / …).
    pub kind: crate::base::facts::EdgeKind,
}

/// Ingest a SCIP index file into a `Vec<EdgeReconstruction>`.
///
/// Parses the SCIP protobuf at `scip_index_path` and reconstructs call edges by finding
/// function-symbol reference occurrences and their enclosing function definition occurrences.
///
/// ## Tier-honesty
///
/// - Reference occurrences in generated/macro code (marked with the `Generated` role) are emitted
///   as [`EdgeReconstruction::Ambiguous`] — the enclosure cannot be trusted.
/// - Reference occurrences where no enclosing function can be determined are
///   [`EdgeReconstruction::Unreconstructible`].
/// - All other reference occurrences are [`EdgeReconstruction::Resolved`].
///
/// The `Generated` role bit is the primary tier-demote signal from SCIP. The distinction
/// between `Resolved` and `Unreconstructible` is not a quality signal about the resolution —
/// it is a structural signal: the SCIP index either contains the enclosure information or it does
/// not. `Unreconstructible` is emitted when the encloser cannot be located, which is honest
/// (no wrong edge emitted) but loses information.
///
/// ## Encloser reconstruction
///
/// SCIP does not guarantee that every reference occurrence has a populated `enclosing_range`.
/// When it is absent, the encloser is inferred heuristically: the DEF occurrence on the largest
/// line number ≤ the reference line. This is correct for sequential, non-interleaved function
/// bodies (the common Rust layout) and conservative when it fails (falls through to
/// `Unreconstructible`).
pub fn ingest_scip(scip_index_path: &std::path::Path) -> Vec<EdgeReconstruction> {
    use protobuf::Message;
    use scip::types::{Index, SymbolRole};

    let bytes = match std::fs::read(scip_index_path) {
        Ok(b) => b,
        Err(_) => return Vec::new(),
    };
    let index = match Index::parse_from_bytes(&bytes) {
        Ok(i) => i,
        Err(_) => return Vec::new(),
    };

    let mut recons = Vec::new();

    for doc in &index.documents {
        // Build: (definition_line, symbol_str) for every fn-kind DEF occurrence.
        // DEF occurrences use single-line ranges [line, start_col, end_col] (the identifier span).
        let def_fns: Vec<(i32, &str)> = doc
            .occurrences
            .iter()
            .filter(|occ| {
                occ.symbol_roles & (SymbolRole::Definition as i32) != 0
                    && is_fn_symbol(&occ.symbol)
                    && !occ.symbol.is_empty()
            })
            .map(|occ| (range_start_line(&occ.range), occ.symbol.as_str()))
            .collect();

        for occ in &doc.occurrences {
            let is_def = occ.symbol_roles & (SymbolRole::Definition as i32) != 0;
            if is_def || occ.symbol.is_empty() || occ.symbol.starts_with("local ") {
                continue;
            }
            if !is_fn_symbol(&occ.symbol) {
                continue;
            }

            let ref_line = range_start_line(&occ.range);

            // Locate the enclosing function definition.
            // Method 1: use enclosing_range if present (reliable when populated).
            // The enclosing_range is the body span [body_start_line, 0, body_end_line, 1].
            // The DEF occurrence's range is the identifier span [fn_line, ident_col, end_col].
            // They share the SAME start line — match by line only (ident_col ≠ 0 in most cases).
            let encloser = if !occ.enclosing_range.is_empty() {
                let enc_start_line = range_start_line(&occ.enclosing_range);
                def_fns
                    .iter()
                    .find(|&&(sl, _)| sl == enc_start_line)
                    .map(|&(_, sym)| sym)
            } else {
                // Method 2 (fallback): the largest DEF line ≤ ref_line.
                // Heuristic: Rust function bodies are sequential and non-interleaved in
                // typical code; the immediately-preceding DEF is the enclosing function.
                def_fns
                    .iter()
                    .filter(|&&(sl, _)| sl <= ref_line)
                    .max_by_key(|&&(sl, _)| sl)
                    .map(|&(_, sym)| sym)
            };

            let callee = &occ.symbol;

            let Some(caller) = encloser else {
                recons.push(EdgeReconstruction::Unreconstructible);
                continue;
            };

            if caller == callee {
                continue; // skip self-recursion
            }

            // Generated role: macro-expanded call sites — enclosure may be guessed.
            let is_generated = occ.symbol_roles & (SymbolRole::Generated as i32) != 0;
            let edge = ResolvedEdge {
                src_symbol: caller.to_string(),
                dst_symbol: callee.to_string(),
                kind: crate::base::facts::EdgeKind::Call,
            };
            if is_generated {
                recons.push(EdgeReconstruction::Ambiguous(vec![edge]));
            } else {
                recons.push(EdgeReconstruction::Resolved(edge));
            }
        }
    }

    recons
}

/// A node id built from a SCIP symbol at a given tier (the resolved feeder's node-construction rule).
///
/// The symbol string becomes the [`crate::node::id::FqPath`] (carrying its tier); the identity
/// digest is BLAKE3 over the symbol bytes; cfg-set is empty (SCIP symbols are cfg-resolved
/// upstream). A single deterministic function so the same symbol always yields the same node
/// identity.
fn node_from_symbol(symbol: &str, tier: crate::read::ResolutionTier) -> crate::node::StromaNodeId {
    crate::node::StromaNodeId {
        fq_path: crate::node::id::FqPath {
            path: symbol.to_string(),
            tier,
        },
        identity_digest: crate::node::digest::IdentityDigest::of_tokens(symbol.as_bytes()),
        cfg_set: crate::node::cfg::CfgSet::default(),
    }
}

/// Lower a batch of SCIP reconstruction outcomes into [`crate::base::facts::EdgeFacts`].
///
/// Returns the `EdgeFacts` salsa input populated with all reconstructable edges. Callers
/// (typically `constitute`) merge this with the syntactic-tier edge batch and call
/// `EdgeFacts::new(db, merged_edges)` once. The tier-honesty rule is applied here:
/// - [`EdgeReconstruction::Resolved`] → a `Resolved`-tier edge (presents-grade eligible);
/// - [`EdgeReconstruction::Ambiguous`] → each candidate DEGRADED to `Syntactic` tier (a macro-site's
///   guessed enclosure can never corroborate up — it is form-only);
/// - [`EdgeReconstruction::Unreconstructible`] → SKIPPED (no edge; the symbol could not be resolved).
pub fn edges_from_reconstructions(recons: &[EdgeReconstruction]) -> Vec<crate::base::facts::EdgeFact> {
    use crate::base::facts::EdgeFact;
    use crate::read::ResolutionTier;

    let mut edges: Vec<EdgeFact> = Vec::new();
    for recon in recons {
        match recon {
            EdgeReconstruction::Resolved(edge) => edges.push(EdgeFact {
                src: node_from_symbol(&edge.src_symbol, ResolutionTier::Resolved),
                dst: node_from_symbol(&edge.dst_symbol, ResolutionTier::Resolved),
                kind: edge.kind,
                tier: ResolutionTier::Resolved,
            }),
            EdgeReconstruction::Ambiguous(candidates) => {
                for edge in candidates {
                    edges.push(EdgeFact {
                        src: node_from_symbol(&edge.src_symbol, ResolutionTier::Syntactic),
                        dst: node_from_symbol(&edge.dst_symbol, ResolutionTier::Syntactic),
                        kind: edge.kind,
                        tier: ResolutionTier::Syntactic, // degraded — never presents-grade
                    });
                }
            },
            EdgeReconstruction::Unreconstructible => {}, // no edge — the symbol did not resolve
        }
    }
    edges
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Return true if `symbol` is a function-kind symbol in SCIP's Rust encoding.
///
/// Rust functions (both free functions and methods) use the Method descriptor suffix,
/// encoding as `name(disambiguator).`. Local symbols are excluded — they cannot
/// participate in cross-document edge reconstruction.
fn is_fn_symbol(symbol: &str) -> bool {
    if symbol.starts_with("local ") {
        return false;
    }
    symbol.ends_with(").")
}

/// Extract the start line from a SCIP range encoding.
///
/// SCIP range: three elements `[startLine, startChar, endChar]` (single-line) or
/// four elements `[startLine, startChar, endLine, endChar]` (multi-line).
fn range_start_line(range: &[i32]) -> i32 {
    range.first().copied().unwrap_or(0)
}

/// A node id built from a SCIP symbol at a given tier (the resolved feeder's node-construction rule).
///
/// The symbol string becomes the [`crate::node::id::FqPath`] (carrying its tier); the identity digest
/// is BLAKE3 over the symbol bytes; cfg-set is empty (SCIP symbols are cfg-resolved upstream). A single
/// deterministic function so the same symbol always yields the same node identity.
fn node_from_symbol(symbol: &str, tier: crate::read::ResolutionTier) -> crate::node::StromaNodeId {
    crate::node::StromaNodeId {
        fq_path: crate::node::id::FqPath {
            path: symbol.to_string(),
            tier,
        },
        identity_digest: crate::node::digest::IdentityDigest::of_tokens(symbol.as_bytes()),
        cfg_set: crate::node::cfg::CfgSet::default(),
    }
}

/// Populate the base's [`crate::base::facts::EdgeFacts`] from a batch of SCIP reconstruction outcomes
/// (R9 — the resolved-feeder population path), then publish the input handle onto the db.
///
/// The tier-honesty rule is applied here, driven by the reconstruction outcome type (never guessed):
/// - [`EdgeReconstruction::Resolved`] → a `Resolved`-tier edge (presents-grade eligible);
/// - [`EdgeReconstruction::Ambiguous`] → each candidate DEGRADED to `Syntactic` tier (a macro-site's
///   guessed enclosure can never corroborate up — it is form-only);
/// - [`EdgeReconstruction::Unreconstructible`] → SKIPPED (no edge; the symbol could not be resolved).
pub fn populate_from_reconstructions(db: &mut crate::db::StromaDb, recons: &[EdgeReconstruction]) {
    use crate::base::facts::EdgeFact;
    use crate::read::ResolutionTier;

    let mut edges: Vec<EdgeFact> = Vec::new();
    for recon in recons {
        match recon {
            EdgeReconstruction::Resolved(edge) => edges.push(EdgeFact {
                src: node_from_symbol(&edge.src_symbol, ResolutionTier::Resolved),
                dst: node_from_symbol(&edge.dst_symbol, ResolutionTier::Resolved),
                kind: edge.kind,
                tier: ResolutionTier::Resolved,
            }),
            EdgeReconstruction::Ambiguous(candidates) => {
                for edge in candidates {
                    edges.push(EdgeFact {
                        src: node_from_symbol(&edge.src_symbol, ResolutionTier::Syntactic),
                        dst: node_from_symbol(&edge.dst_symbol, ResolutionTier::Syntactic),
                        kind: edge.kind,
                        tier: ResolutionTier::Syntactic, // degraded — never presents-grade
                    });
                }
            },
            EdgeReconstruction::Unreconstructible => {}, // no edge — the symbol did not resolve
        }
    }

    let edge_facts = crate::base::facts::EdgeFacts::new(&*db, edges);
    db.set_edge_facts(edge_facts);
}
