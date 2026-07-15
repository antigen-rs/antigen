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
/// ## Encloser reconstruction — exact body-span containment
///
/// `enclosing_range` on a **definition** occurrence is that definition's full span (identifier +
/// body). The caller of a reference at line `L` is therefore the function whose body span
/// *contains* `L` — the innermost such span, so a nested definition wins over its outer one. This
/// is an exact lookup, not a proximity guess: measured on antigen's own index, every fn definition
/// carries a body span and 97.9% of fn references fall inside exactly one. A reference contained by
/// no span (2.1% — e.g. a call in an item position outside any fn body) is honestly
/// [`EdgeReconstruction::Unreconstructible`] rather than attributed to a guessed neighbour.
///
/// The field is NOT the enclosing *function* of a reference — verified against the index: an
/// occurrence's `enclosing_range` never names a different symbol's definition (0 of 8398). Reading
/// it that way attributes every edge to the callee itself, which the `src == dst` self-recursion
/// guard then silently drops.
pub fn ingest_scip(scip_index_path: &std::path::Path) -> Vec<EdgeReconstruction> {
    use protobuf::Message;
    use scip::types::{Index, SymbolRole};

    let Ok(bytes) = std::fs::read(scip_index_path) else {
        return Vec::new();
    };
    let Ok(index) = Index::parse_from_bytes(&bytes) else {
        return Vec::new();
    };

    let mut recons = Vec::new();

    for doc in &index.documents {
        // Build: (body_start_line, body_end_line, symbol) for every fn-kind DEF occurrence. A DEF's
        // `enclosing_range` is its full span (identifier + body) — the interval a call must fall
        // inside to be attributed to it.
        let body_spans: Vec<(i32, i32, &str)> = doc
            .occurrences
            .iter()
            .filter(|occ| {
                occ.symbol_roles & (SymbolRole::Definition as i32) != 0
                    && is_fn_symbol(&occ.symbol)
                    && !occ.symbol.is_empty()
                    && !occ.enclosing_range.is_empty()
            })
            .map(|occ| {
                (
                    range_start_line(&occ.enclosing_range),
                    range_end_line(&occ.enclosing_range),
                    occ.symbol.as_str(),
                )
            })
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

            // The caller is the definition whose body span CONTAINS this reference — the innermost
            // one, so a nested fn wins over the outer fn it sits inside. Exact, not a proximity
            // guess; a reference inside no body span stays `Unreconstructible` below.
            let encloser = body_spans
                .iter()
                .filter(|&&(start, end, _)| start <= ref_line && ref_line <= end)
                .min_by_key(|&&(start, end, _)| end - start)
                .map(|&(_, _, sym)| sym);

            let dst_sym = occ.symbol.as_str();

            let Some(src_sym) = encloser else {
                recons.push(EdgeReconstruction::Unreconstructible);
                continue;
            };

            if src_sym == dst_sym {
                continue; // skip self-recursion
            }

            // Generated role: macro-expanded call sites — enclosure may be guessed.
            let is_generated = occ.symbol_roles & (SymbolRole::Generated as i32) != 0;
            let edge = ResolvedEdge {
                src_symbol: src_sym.to_string(),
                dst_symbol: dst_sym.to_string(),
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
pub fn edges_from_reconstructions(
    recons: &[EdgeReconstruction],
) -> Vec<crate::base::facts::EdgeFact> {
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

/// Extract the end line from a SCIP range encoding.
///
/// Four elements `[startLine, startChar, endLine, endChar]` (multi-line) carry the end line at
/// index 2; a three-element single-line range `[startLine, startChar, endChar]` ends on its start
/// line. A definition's body span is multi-line in practice (3414 of 3415 on antigen's own index).
fn range_end_line(range: &[i32]) -> i32 {
    if range.len() >= 4 {
        range[2]
    } else {
        range.first().copied().unwrap_or(0)
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
