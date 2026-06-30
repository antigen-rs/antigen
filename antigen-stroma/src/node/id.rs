//! STEP 2a — `StromaNodeId`, the SEMANTIC node identity (ADR-067 §A.1).
//!
//! The 3-field collision-free, cfg-aware identity. **NOT the salsa storage handle** (that's
//! [`super::node::Node`]) and **NOT** `ItemTarget` (that's the wider MATCHING key — reusing it
//! re-imports the bare-name collision the frame exists to close, diff.rs:114 last-write-wins).
//!
//! Used for hashing / equality / overlay-anchoring / cross-snapshot comparison.

use super::cfg::CfgSet;
use super::digest::IdentityDigest;

/// The fully-qualified, collision-free, cfg-aware node identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StromaNodeId {
    /// The addressing half — resolved FQ path (SCIP symbol, or syntactic module-path in degraded
    /// mode). Survives edit+rebase → the F4 overlay anchor. See [`super::path`].
    pub fq_path: FqPath,
    /// The integrity half — collision-resistant signing digest (changes on body edit).
    pub identity_digest: IdentityDigest,
    /// The disambiguation half — two items identical except under different cfg are DISTINCT nodes.
    pub cfg_set: CfgSet,
}

/// A fully-qualified path. Newtype over the SCIP symbol string (resolved tier) or the syntactic
/// module-path (degraded tier). Carries its own tier so a degraded path can never corroborate up.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FqPath {
    /// The path string — `crate::module::item` (syntactic) or the SCIP symbol (resolved).
    pub path: String,
    /// The tier this path was constructed at (syntactic floor < SCIP-symbol resolved).
    pub tier: crate::read::ResolutionTier,
}
