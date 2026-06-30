//! STEP 2c — syntactic FQ-path construction (GAP A5 / G2, RESOLVED Type-A).
//!
//! The locator MUST be constructible BEFORE r-a/SCIP (the syntactic-tier base ships first, degraded
//! mode). But `antigen`'s `scan::parse` is a single-file walk with NO module-path. So constitute
//! needs this sub-step: walk the module tree (inline `mod` blocks + the file structure) to assign
//! `crate::mod::item` paths SYNTACTICALLY. Cheap, r-a-free (syn gives the nesting). The SCIP symbol
//! (resolved tier) REFINES this when present; the syntactic module-path is the floor-tier locator.

use crate::node::id::FqPath;
use crate::read::ResolutionTier;

/// Build the syntactic FQ path for an item from the module tree.
///
/// **STUB — fill (frame epoch):** walk inline `mod` blocks + the directory/file structure; emit
/// `crate::mod::item` at `ResolutionTier::Syntactic`. r-a-free; the floor-tier locator.
#[must_use]
pub fn syntactic_fq_path(crate_name: &str, module_chain: &[String], item_name: &str) -> FqPath {
    // Join `crate :: mod1 :: mod2 :: … :: item` — the FULL module chain is load-bearing (the closing
    // of the bare-name collision: `foo::bar` ≠ `baz::bar`, and `a::item` ≠ `a::b::item`). A pure
    // function of (crate, module_chain, item) so two captures of the SAME item collide (the NC).
    let mut segments = Vec::with_capacity(module_chain.len() + 2);
    segments.push(crate_name);
    segments.extend(module_chain.iter().map(String::as_str));
    segments.push(item_name);
    FqPath {
        path: segments.join("::"),
        // The syntactic walk assumes file-structure = module-structure — WRONG on re-exports,
        // `#[path]`, and macro-modules (ADR-070 §4.2). That is exactly why this is `dread`-grade and
        // never corroborates up; the SCIP symbol (resolved tier) supersedes the lexical guess.
        tier: ResolutionTier::Syntactic,
    }
}

/// Refine a syntactic path with a resolved SCIP symbol when available (raises the locator tier).
///
/// **STUB — frame epoch:** if a SCIP symbol exists for this item, return the resolved-tier `FqPath`;
/// else return the syntactic floor unchanged.
#[must_use]
pub fn refine_with_scip(syntactic: FqPath, scip_symbol: Option<&str>) -> FqPath {
    // A well-formed SCIP symbol IS the resolved-tier truth (symbol-level keying drops name-ambiguity
    // 91% → 0.6%). It SUPERSEDES the lexical guess. A malformed/empty symbol is NOT a valid locator —
    // fall through to the syntactic floor (`dread`-grade), NEVER mint a resolved node with a malformed
    // symbol (ADR-070 §5.2 malformed-symbol invariant; keeps the resolved tier CLEAN).
    match scip_symbol {
        Some(symbol) if is_well_formed_scip_symbol(symbol) => FqPath {
            path: symbol.to_string(),
            tier: ResolutionTier::Resolved,
        },
        // No symbol, or a malformed one → the syntactic floor is the honest answer.
        _ => syntactic,
    }
}

/// A minimal well-formedness gate for a SCIP symbol used as a resolved-tier locator.
///
/// Frame-epoch conservative: a symbol is admissible only if it is non-empty and not pure whitespace.
/// The full SCIP grammar validation is engine-epoch (where `ingest_scip` wires the real run); the
/// invariant the FRAME must hold is the negative one — a malformed symbol must never become an
/// `fq_path` at `Resolved` tier (it falls through to syntactic instead).
fn is_well_formed_scip_symbol(symbol: &str) -> bool {
    !symbol.trim().is_empty()
}
