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
pub fn syntactic_fq_path(_crate_name: &str, _module_chain: &[String], _item_name: &str) -> FqPath {
    let _ = ResolutionTier::Syntactic;
    todo!("frame epoch: syntactic module-tree walk -> crate::mod::item (the floor-tier locator)")
}

/// Refine a syntactic path with a resolved SCIP symbol when available (raises the locator tier).
///
/// **STUB — frame epoch:** if a SCIP symbol exists for this item, return the resolved-tier `FqPath`;
/// else return the syntactic floor unchanged.
#[must_use]
pub fn refine_with_scip(_syntactic: FqPath, _scip_symbol: Option<&str>) -> FqPath {
    todo!("frame epoch: SCIP symbol refines the syntactic floor (resolved-tier locator)")
}
