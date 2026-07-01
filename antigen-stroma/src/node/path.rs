//! FQ-path construction — the two tiers of the locator.
//!
//! [`syntactic_fq_path`] builds the floor: the module chain (from the file structure) assigns each
//! item a `crate::mod::item` path at `ResolutionTier::Syntactic`, needing only `syn`.
//! [`refine_with_scip`] raises a path to `ResolutionTier::Resolved` when given a well-formed SCIP
//! symbol — the symbol is the resolved-tier locator; the syntactic path is the tier below it.

use crate::node::id::FqPath;
use crate::read::ResolutionTier;

/// Build the syntactic FQ path for an item from its module chain.
///
/// Joins `crate_name`, the `module_chain`, and `item_name` with `::` into a `crate::mod::item` path
/// and stamps it `ResolutionTier::Syntactic` — the floor-tier locator, computed without
/// rust-analyzer or SCIP. The full module chain is load-bearing: it distinguishes `foo::bar` from
/// `baz::bar`. A resolved SCIP symbol supersedes this via [`refine_with_scip`].
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
/// A well-formed SCIP symbol is the resolved-tier truth and supersedes the lexical guess: returns a
/// `ResolutionTier::Resolved` `FqPath` built from the symbol. A `None` or malformed symbol falls
/// through to the `syntactic` floor unchanged — a malformed symbol never mints a `Resolved` locator
/// (ADR-070 §5.2 malformed-symbol invariant, which keeps the resolved tier clean).
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
/// Conservative: a symbol is admissible only if it is non-empty and not pure whitespace. This is a
/// well-formedness floor, not full SCIP grammar validation; the invariant it holds is the negative
/// one — a malformed symbol must never become an `fq_path` at `Resolved` tier (it falls through to
/// syntactic instead).
fn is_well_formed_scip_symbol(symbol: &str) -> bool {
    !symbol.trim().is_empty()
}
