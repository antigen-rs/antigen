//! Reserved extension points — the hooks organs and lenses implement against.
//!
//! Each type here names an extension point in the type system, so an obligation is marked rather
//! than left implicit. Each is a hook, not an implementation.

/// The accrete/migrate void.
///
/// A sheaf/field lens that reinterprets an earlier lens's attribute is an accretion acting as a
/// migration, and there is no build-time detector for it. This type is the hook a lens-migration
/// detector attaches to.
pub struct AccreteMigrateVoid;

/// The parity-surveillance hook.
///
/// This trait is the surface a parity-guard implements: its `check_parity` asks whether the compose
/// output still matches its recomputation. The trait declares the surface; it holds no oracle itself.
pub trait ParitySurveillance {
    /// Re-derive the compose output and compare it to the stored one.
    fn check_parity(&self) -> bool;
}

// T3Mir (see read::tier::ResolutionTier::T3Mir) is a reserved tier slot — the variant is in the read
// contract, and its mir-exact population is scoped to the queried region.
//
// IFDS summary-edge condensation (data-flow tier) is a reserved extension (ADR-067 §E.11); the
// SCC-condense half for blast lives in the closure. (base::facts documents the condensation
// two-operator split.)
