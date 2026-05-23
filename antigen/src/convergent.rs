//! Convergent-Evidence Family — public types (ADR-024).
//!
//! The Convergent-Evidence Family covers backward-looking evidence
//! aggregation: independent witnesses converging on a defense claim.
//! Per ADR-024, the family ships seven proc-macro primitives:
//!
//! - `#[diagnostic(modalities = [...], min_independent = N)]`
//! - `#[clonal(witness = ..., iterations = N, seed = SeedKind::...)]`
//! - `#[igg(witnesses = [...], historical_span = N, min_reattestations = N)]`
//! - `#[crossreactive(fingerprints = [...])]`
//! - `#[polyclonal]`
//! - `#[monoclonal]`
//! - `#[adcc]`
//!
//! This module hosts the two **public** enum types adopters use to populate
//! the macros' arguments: [`WitnessClass`] (independence-checking via
//! distinct categories) and [`SeedKind`] (non-deterministic seed
//! enforcement for `#[clonal]`).
//!
//! ## ADR-024 invariants
//!
//! - **`min_independent` = distinct WitnessClass CATEGORIES, not raw witness
//!   count** (per adversarial C1). Two `StaticAnalysis` witnesses count as
//!   ONE independent class. The `#[diagnostic]` audit hint
//!   `diagnostic-modalities-class-collapsed` fires when all witnesses share
//!   a single class.
//! - **`SeedKind::Fixed(u64)` is REJECTED for `#[clonal]`** (per adversarial
//!   C2). The clonal proc-macro emits a compile error at parse time
//!   (parallel to the `#[immunosuppress]` duration-cap enforcement). A
//!   fixed seed makes "independent iterations" a misnomer — each iteration
//!   replays the same RNG state.
//! - **IgG source-independence is NOMINAL only** (per adversarial C3 and
//!   ADR-024 §What this ADR does NOT do). Different signer-identity
//!   strings are not structural proof of independent sources; the antigen
//!   surfaces the discipline, not the metaphysical guarantee.

use serde::{Deserialize, Serialize};

// ============================================================================
// WitnessClass
// ============================================================================

/// Categorical type of witness backing a defense claim.
///
/// Per ADR-024 §Decision + adversarial C1: `#[diagnostic]`'s
/// `min_independent` is measured in distinct CLASSES, not witness count.
/// Two property-tests count as ONE `PropertyTest` class; the discipline is
/// against argument-from-uniformity (running the same kind of test in
/// triplicate doesn't add evidence).
///
/// The variant set is the v0.2 sealed-set; future adversarial findings or
/// recognized witness modalities can extend it via additive ADR amendment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WitnessClass {
    /// Compile-time analysis without execution (clippy, custom lints,
    /// type-checker reductions, AST scans).
    StaticAnalysis,
    /// Randomized property exploration (proptest, quickcheck, fuzzy
    /// generators with shrinking).
    PropertyTest,
    /// Formal verification proof (Kani, Prusti, Verus, Creusot,
    /// hand-checked machine proofs).
    FormalVerification,
    /// Human review attested in writing (PR review, ADR substrate,
    /// reading-notebook commitment).
    ManualReview,
    /// Coverage-guided runtime fuzzing (cargo-fuzz, AFL, libfuzzer).
    /// Distinct from `PropertyTest` — fuzzing prioritizes structural
    /// coverage; property-tests prioritize logical invariants.
    RuntimeFuzz,
    /// Substrate-witness evaluation per ADR-019 — `.attest/` sidecar
    /// predicates, ratified-doc references, signed-trailer audits,
    /// oracle completeness checks.
    SubstrateWitness,
}

impl WitnessClass {
    /// String form for CLI rendering and audit-hint detail strings.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StaticAnalysis => "static-analysis",
            Self::PropertyTest => "property-test",
            Self::FormalVerification => "formal-verification",
            Self::ManualReview => "manual-review",
            Self::RuntimeFuzz => "runtime-fuzz",
            Self::SubstrateWitness => "substrate-witness",
        }
    }

    /// Parse from the kebab-case form. Returns `None` for unknown variants.
    #[must_use]
    pub fn parse_class(s: &str) -> Option<Self> {
        match s {
            "static-analysis" | "static_analysis" | "StaticAnalysis" => Some(Self::StaticAnalysis),
            "property-test" | "property_test" | "PropertyTest" => Some(Self::PropertyTest),
            "formal-verification" | "formal_verification" | "FormalVerification" => {
                Some(Self::FormalVerification)
            }
            "manual-review" | "manual_review" | "ManualReview" => Some(Self::ManualReview),
            "runtime-fuzz" | "runtime_fuzz" | "RuntimeFuzz" => Some(Self::RuntimeFuzz),
            "substrate-witness" | "substrate_witness" | "SubstrateWitness" => {
                Some(Self::SubstrateWitness)
            }
            _ => None,
        }
    }
}

// ============================================================================
// SeedKind
// ============================================================================

/// RNG-seed policy for `#[clonal]`-style iterated witness evaluation.
///
/// Per ADR-024 §Decision + adversarial C2: `#[clonal]` declares that a
/// witness is being run with many independent iterations. If the seed is
/// fixed, the iterations are NOT independent — they all replay the same
/// RNG state. `SeedKind::Fixed(_)` in `#[clonal]` is therefore a
/// COMPILE-TIME ERROR (the macro emits the error at parse time, parallel
/// to the `#[immunosuppress]` duration-cap enforcement).
///
/// Adopters who genuinely want a fixed seed should use a non-clonal
/// witness primitive (or `#[antigen_tolerance]` with a documented
/// rationale).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum SeedKind {
    /// Standard random seed (e.g., `thread_rng()` at iteration start).
    Random,
    /// Entropy sourced from CI environment (`GITHUB_RUN_ID`, build hash,
    /// commit SHA-derived hash). Run-deterministic but cross-run
    /// independent.
    EntropyFromCi,
    /// Timestamp-derived seed (millisecond-precision wall clock).
    /// Coarser than `Random` but sufficient for typical clonal iteration.
    TimestampSeeded,
    /// **REJECTED for `#[clonal]`** — fixed u64 seed. The clonal proc-
    /// macro emits a compile error when this variant is supplied. Per
    /// ADR-024 C2: a fixed seed means iterations are NOT independent;
    /// "independent iterations with a fixed seed" is a contradiction.
    Fixed(u64),
}

impl SeedKind {
    /// True when the seed kind is non-deterministic (i.e., legal for
    /// `#[clonal]`).
    #[must_use]
    pub const fn is_non_deterministic(self) -> bool {
        matches!(
            self,
            Self::Random | Self::EntropyFromCi | Self::TimestampSeeded
        )
    }

    /// True when the seed kind is fixed (i.e., REJECTED for `#[clonal]`).
    #[must_use]
    pub const fn is_fixed(self) -> bool {
        matches!(self, Self::Fixed(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn witness_class_str_roundtrip() {
        for variant in [
            WitnessClass::StaticAnalysis,
            WitnessClass::PropertyTest,
            WitnessClass::FormalVerification,
            WitnessClass::ManualReview,
            WitnessClass::RuntimeFuzz,
            WitnessClass::SubstrateWitness,
        ] {
            let s = variant.as_str();
            let back = WitnessClass::parse_class(s).expect("kebab roundtrip");
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn witness_class_parses_snake_and_camel_too() {
        assert_eq!(
            WitnessClass::parse_class("property_test"),
            Some(WitnessClass::PropertyTest)
        );
        assert_eq!(
            WitnessClass::parse_class("PropertyTest"),
            Some(WitnessClass::PropertyTest)
        );
        assert_eq!(WitnessClass::parse_class("unknown"), None);
    }

    #[test]
    fn seed_kind_non_determinism() {
        assert!(SeedKind::Random.is_non_deterministic());
        assert!(SeedKind::EntropyFromCi.is_non_deterministic());
        assert!(SeedKind::TimestampSeeded.is_non_deterministic());
        assert!(!SeedKind::Fixed(42).is_non_deterministic());

        assert!(!SeedKind::Random.is_fixed());
        assert!(SeedKind::Fixed(42).is_fixed());
    }
}
