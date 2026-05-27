//! Antigen-Category taxonomy (ADR-028).
//!
//! Two structural categories distinguish HOW an antigen fires:
//!
//! - [`AntigenCategory::SubstrateAlignment`]: fires when a REPRESENTATION
//!   diverges from actual state ("this says X but actual state is Y"). Witness
//!   checks the substrate. Example: `UnpinnedDependency` — Cargo.toml claims
//!   `dep = "^1.0"` when it should say `dep = "=1.0.3"`.
//!
//! - [`AntigenCategory::FunctionalCorrectness`]: fires when a VERB produces the
//!   wrong output ("this claims to do X but produces Y"). Witness exercises
//!   behaviour. Example: `PanickingInDrop` — Drop impl panics under some inputs.
//!
//! ## Enforcement (Option A STRICT, per ADR-028 §Decision)
//!
//! - `category = AntigenCategory::SubstrateAlignment` **requires** at least one
//!   substrate-witness predicate leaf.
//! - `category = AntigenCategory::FunctionalCorrectness` **requires** at least
//!   one code-witness predicate leaf.
//! - Hybrid antigens (`category = [SubstrateAlignment, FunctionalCorrectness]`)
//!   require BOTH witness types verified at audit-time; a missing axis is
//!   reported as `antigen-category-hybrid-incomplete-evidence`.
//! - v0.2+ **new** declarations must supply `category` explicitly; absence is a
//!   hard parse-time error (`antigen-category-missing-explicit`).
//! - v0.1 carry-over antigens lacking `category` receive a soft default of
//!   `[FunctionalCorrectness]` + emit the migration hint
//!   `antigen-category-defaulted-implicit-functional`.
//!
//! ## Audit-hint vocabulary (ADR-028 §Schema additions)
//!
//! | Hint key | When |
//! |---|---|
//! | `antigen-category-defaulted-implicit-functional` | v0.1 carryover; category absent; soft default applied |
//! | `antigen-category-missing-explicit` | v0.2+ new declaration without `category` field |
//! | `antigen-category-mismatch-witness-type` | category vs predicate type structural mismatch (advisory) |
//! | `antigen-category-claim-inconsistent-with-predicate-type` | parse-time cross-check fires |
//! | `antigen-category-hybrid-incomplete-evidence` | hybrid antigen; one axis unwitnessed at audit-time |

use serde::{Deserialize, Serialize};

// ============================================================================
// AntigenCategory
// ============================================================================

/// First-class category of an antigen declaration (ADR-028).
///
/// The variant set is sealed at v0.2; extending it requires an ADR amendment
/// per ADR-001 Amendment 1 C6.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AntigenCategory {
    /// The antigen fires when a REPRESENTATION diverges from actual state.
    ///
    /// Examples: unpinned dependencies, untracked files in publish, git-dirty
    /// tagged releases, campsites whose sidecar is missing.
    ///
    /// Minimum witness requirement: at least one substrate-witness predicate
    /// leaf (per ADR-019 + ADR-028 Option A STRICT).
    SubstrateAlignment,

    /// The antigen fires when a VERB produces the wrong output.
    ///
    /// Examples: `PanickingInDrop`, integer overflow in arithmetic, incorrect
    /// boundary parsing.
    ///
    /// Minimum witness requirement: at least one code-witness predicate leaf
    /// (per ADR-019 + ADR-028 Option A STRICT).
    FunctionalCorrectness,
}

impl AntigenCategory {
    /// kebab-case string for CLI rendering and audit-hint detail strings.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SubstrateAlignment => "substrate-alignment",
            Self::FunctionalCorrectness => "functional-correctness",
        }
    }

    /// Parse from the kebab-case, `PascalCase`, or path-qualified forms.
    ///
    /// Accepted sources:
    /// - kebab (`substrate-alignment`) — CLI `--category` flag and serde-deserialized JSON
    /// - Pascal (`SubstrateAlignment`) — scanner reading unqualified `category =` path
    /// - path-qualified (`AntigenCategory::SubstrateAlignment`) — scanner reading qualified form
    ///
    /// Snake-case (`substrate_alignment`) is intentionally NOT accepted: no real
    /// input source produces it (serde/CLI use kebab; the macro scanner produces
    /// Pascal/path from Rust path tokens).
    #[must_use]
    pub fn parse_category(s: &str) -> Option<Self> {
        match s {
            "substrate-alignment"
            | "SubstrateAlignment"
            | "AntigenCategory::SubstrateAlignment" => Some(Self::SubstrateAlignment),
            "functional-correctness"
            | "FunctionalCorrectness"
            | "AntigenCategory::FunctionalCorrectness" => Some(Self::FunctionalCorrectness),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_str_roundtrip() {
        for variant in [
            AntigenCategory::SubstrateAlignment,
            AntigenCategory::FunctionalCorrectness,
        ] {
            let s = variant.as_str();
            let back = AntigenCategory::parse_category(s).expect("kebab roundtrip");
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn category_parses_all_forms() {
        // kebab — CLI/serde canonical
        assert_eq!(
            AntigenCategory::parse_category("substrate-alignment"),
            Some(AntigenCategory::SubstrateAlignment)
        );
        // Pascal — scanner unqualified path
        assert_eq!(
            AntigenCategory::parse_category("SubstrateAlignment"),
            Some(AntigenCategory::SubstrateAlignment)
        );
        // path-qualified — scanner qualified path
        assert_eq!(
            AntigenCategory::parse_category("AntigenCategory::SubstrateAlignment"),
            Some(AntigenCategory::SubstrateAlignment)
        );
        assert_eq!(
            AntigenCategory::parse_category("functional-correctness"),
            Some(AntigenCategory::FunctionalCorrectness)
        );
        assert_eq!(
            AntigenCategory::parse_category("FunctionalCorrectness"),
            Some(AntigenCategory::FunctionalCorrectness)
        );
        assert_eq!(AntigenCategory::parse_category("unknown"), None);
    }

    #[test]
    fn category_rejects_snake_case() {
        // No real input source produces snake_case — serde/CLI use kebab,
        // macro scanner produces Pascal/path from Rust path tokens.
        assert_eq!(
            AntigenCategory::parse_category("substrate_alignment"),
            None
        );
        assert_eq!(
            AntigenCategory::parse_category("functional_correctness"),
            None
        );
    }

    #[test]
    fn category_is_copy() {
        let c = AntigenCategory::SubstrateAlignment;
        let d = c; // proves Copy
        assert_eq!(c, d);
    }

    // -------------------------------------------------------------------------
    // Adversarial tests (added by adversarial role)
    // -------------------------------------------------------------------------

    #[test]
    fn category_rejects_empty_string() {
        assert_eq!(AntigenCategory::parse_category(""), None);
    }

    #[test]
    fn category_rejects_whitespace_padded() {
        // Trailing/leading whitespace must not match.
        assert_eq!(
            AntigenCategory::parse_category("substrate-alignment "),
            None
        );
        assert_eq!(AntigenCategory::parse_category(" SubstrateAlignment"), None);
    }

    #[test]
    fn category_rejects_mixed_separator() {
        assert_eq!(AntigenCategory::parse_category("substrate-Alignment"), None);
        assert_eq!(AntigenCategory::parse_category("Substrate-alignment"), None);
    }

    #[test]
    fn category_rejects_partial_path() {
        // "Category::SubstrateAlignment" (missing "Antigen" prefix) must NOT match.
        // Only the full "AntigenCategory::SubstrateAlignment" path form is accepted.
        assert_eq!(
            AntigenCategory::parse_category("Category::SubstrateAlignment"),
            None
        );
    }

    #[test]
    fn category_serde_roundtrip() {
        let s = serde_json::to_string(&AntigenCategory::SubstrateAlignment).unwrap();
        assert_eq!(s, "\"substrate-alignment\"");
        let back: AntigenCategory = serde_json::from_str("\"substrate-alignment\"").unwrap();
        assert_eq!(back, AntigenCategory::SubstrateAlignment);
    }

    #[test]
    fn category_serde_rejects_unknown_variant() {
        let result: Result<AntigenCategory, _> = serde_json::from_str("\"hybrid\"");
        assert!(
            result.is_err(),
            "serde should reject unknown variant 'hybrid', got Ok"
        );
    }

    #[test]
    fn category_serde_rejects_uppercase_form() {
        // serde kebab-case means "SubstrateAlignment" is NOT the canonical serde form.
        let result: Result<AntigenCategory, _> = serde_json::from_str("\"SubstrateAlignment\"");
        assert!(
            result.is_err(),
            "serde should reject PascalCase 'SubstrateAlignment'; canonical is 'substrate-alignment'"
        );
    }

    #[test]
    fn category_rejects_null_byte() {
        assert_eq!(AntigenCategory::parse_category("\0"), None);
        assert_eq!(
            AntigenCategory::parse_category("substrate-alignment\0"),
            None
        );
    }

    #[test]
    fn category_hash_consistent_with_eq() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(AntigenCategory::SubstrateAlignment);
        set.insert(AntigenCategory::SubstrateAlignment);
        // Hash equality must hold — the set should contain only 1 element.
        assert_eq!(set.len(), 1);
        set.insert(AntigenCategory::FunctionalCorrectness);
        assert_eq!(set.len(), 2);
    }
}
