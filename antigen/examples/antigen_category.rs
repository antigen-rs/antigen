//! Example: antigen-category taxonomy (ADR-028).
//!
//! Every antigen declaration carries a `category` field that classifies HOW
//! the failure-class fires. Two categories:
//!
//! - `SubstrateAlignment` — the failure-class fires when a REPRESENTATION
//!   diverges from actual state. The thing that's wrong is what the system
//!   SAYS, not what it COMPUTES. Evidence lives outside the code: a sign-off,
//!   a ratified doc, an un-reviewed discipline record. Use `requires =`.
//!
//! - `FunctionalCorrectness` — the failure-class fires when a VERB produces
//!   the wrong output. The thing that's wrong is what the code DOES. Evidence
//!   is behavioral: a test, proptest, formal proof, or lint exercises the verb.
//!   Use `witness =`.
//!
//! ## The quick choice test
//!
//! **Can a test execute the thing you're defending?**
//! - Yes → `FunctionalCorrectness` + `witness = some_test`
//! - No (the failure-class is about substrate state that code execution can't
//!   verify) → `SubstrateAlignment` + `requires = ratified_doc(...)` or
//!   `requires = signers(...)`
//!
//! ## CLI: filter by category
//!
//! ```sh
//! # Show only substrate-alignment presentations:
//! cargo run --bin cargo-antigen -- antigen scan --root antigen/examples \
//!     --category substrate-alignment
//!
//! # Show only functional-correctness presentations:
//! cargo run --bin cargo-antigen -- antigen scan --root antigen/examples \
//!     --category functional-correctness
//! ```
//!
//! ## Run this example
//!
//! ```sh
//! cargo run --example antigen_category --package antigen
//! ```

#![allow(dead_code, unused_variables)]

use antigen::{antigen, immune, presents};

// ============================================================================
// FunctionalCorrectness: a verb that can be wrong
//
// Scenario: a data-cleaning function that must not produce NaN values. The
// failure is behavioral — "clean_values() returned NaN when it shouldn't."
// A test that exercises clean_values() and checks the output is the natural
// witness. This is FunctionalCorrectness.
// ============================================================================

#[antigen(
    name = "nan-in-cleaned-output",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"name = matches("DataCleaner")"#,
    family = "numeric-correctness",
    summary = "DataCleaner::clean_values() returned NaN values — the output representation diverges from the claimed invariant (no NaN in cleaned data).",
    references = []
)]
pub struct NanInCleanedOutput;

/// A data cleaner that should never produce NaN.
///
/// VULNERABLE: missing NaN guard — f64::sqrt(-1.0) produces NaN silently.
#[presents(NanInCleanedOutput)]
pub struct DataCleaner {
    /// Raw float values to clean.
    pub data: Vec<f64>,
}

impl DataCleaner {
    /// Clean values without NaN guard — vulnerable path.
    pub fn clean_values(&self) -> Vec<f64> {
        // BUG: no NaN guard — negative sqrt silently produces NaN
        self.data.iter().map(|&x| x.sqrt()).collect()
    }
}

/// A corrected cleaner with a NaN guard.
///
/// DEFENDED: clean_values_safe() is covered by the property test
/// `test_clean_values_no_nan` which exercises a range of float inputs and
/// asserts the output contains no NaN. The witness resolves the claim.
pub struct DataCleanerSafe {
    /// Raw float values to clean; negatives are filtered before sqrt.
    pub data: Vec<f64>,
}

#[immune(
    NanInCleanedOutput,
    witness = test_clean_values_no_nan
)]
impl DataCleanerSafe {
    /// Clean values with NaN guard — defended path.
    pub fn clean_values_safe(&self) -> Vec<f64> {
        self.data
            .iter()
            .filter(|&&x| x >= 0.0)
            .map(|&x| x.sqrt())
            .collect()
    }
}

#[test]
fn test_clean_values_no_nan() {
    let cleaner = DataCleanerSafe {
        data: vec![-1.0, 0.0, 4.0, 9.0, 16.0],
    };
    let result = cleaner.clean_values_safe();
    for val in &result {
        assert!(!val.is_nan(), "clean_values_safe() produced NaN: {val}");
    }
    // Negative inputs are filtered, not NaN-ified
    assert_eq!(result.len(), 4, "expected 4 non-negative values cleaned");
}

// ============================================================================
// SubstrateAlignment: a representation that can be stale
//
// Scenario: a security policy document must be reviewed and signed by the
// security team before each release. The failure-class fires when the policy
// doc is out-of-date or unsigned — not because any code ran incorrectly, but
// because a REPRESENTATION (the signed doc record) diverges from the required
// state (signed, current). A test can't verify this; the `.attest/` sidecar
// carries the evidence. This is SubstrateAlignment.
// ============================================================================

#[antigen(
    name = "unsigned-security-policy",
    category = AntigenCategory::SubstrateAlignment,
    fingerprint = r#"doc_contains("security_policy")"#,
    family = "governance-compliance",
    summary = "A security policy document that must be reviewed and signed before release was not attested — the attestation record diverges from the required signed state.",
    references = ["SECURITY.md"]
)]
pub struct UnsignedSecurityPolicy;

/// Placeholder for release-gating logic that depends on a signed security policy.
///
/// VULNERABLE: no attestation check — policy sign-off is assumed, not verified.
#[presents(UnsignedSecurityPolicy)]
pub fn gate_release_unverified(version: &str) -> Result<(), String> {
    println!("[GATE] Releasing {version} — security policy assumed signed");
    Ok(())
}

/// Release gate with a substrate-witness that verifies the sign-off record.
///
/// DEFENDED: the `requires = ratified_doc(...)` predicate evaluates whether the
/// security policy sidecar in `.attest/` was signed by the `security-team` role
/// before this release gate is crossed. `cargo antigen audit` evaluates the
/// predicate; unsigned = `DisciplinePredicateFailed`.
///
/// The defense is SubstrateAlignment: the thing being checked is not whether
/// `gate_release` *computes* correctly, but whether the *sign-off record*
/// (a substrate artifact) is present and current. No test can run the
/// security team's sign-off; only `cargo antigen audit` can check the sidecar.
#[immune(
    UnsignedSecurityPolicy,
    requires = all_of([
        ratified_doc(path = "docs/security-policy.md", min_version = "1.0"),
        signers(required = ["security-team"]),
        fresh_within_days(days = 90),
    ])
)]
pub fn gate_release_verified(version: &str) -> Result<(), String> {
    println!("[GATE] Releasing {version} — security policy attested");
    Ok(())
}

// ============================================================================
// Main: show both patterns
// ============================================================================

fn main() {
    println!("=== antigen-category example ===");
    println!();
    println!("Pattern 1: FunctionalCorrectness — NanInCleanedOutput");
    println!("  Vulnerable: DataCleaner::clean_values() (no NaN guard)");
    println!("  Defended:   DataCleanerSafe::clean_values_safe() (witnessed by test)");
    println!("  Witness:    test_clean_values_no_nan — a test *executes* the defense");
    println!();
    println!("Pattern 2: SubstrateAlignment — UnsignedSecurityPolicy");
    println!("  Vulnerable: gate_release_unverified() (assumes policy signed)");
    println!("  Defended:   gate_release_verified() (requires = signers(...))");
    println!("  Witness:    cargo antigen audit evaluates .attest/ sidecar record");
    println!();
    println!("The quick test: can a *test* execute the thing you're defending?");
    println!("  NaN guard: yes → FunctionalCorrectness + witness=");
    println!("  Policy sign-off: no → SubstrateAlignment + requires=");
    println!();
    println!("Filter by category:");
    println!("  cargo run --bin cargo-antigen -- antigen scan --root antigen/examples \\");
    println!("      --category substrate-alignment");
}
