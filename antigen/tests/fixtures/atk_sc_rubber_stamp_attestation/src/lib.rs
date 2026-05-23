// ATK-SC-1: Rubber-stamp attestation bypass fixture.
//
// This fixture verifies that dep_attested with an empty reviewable_artifact
// is caught by the audit. The bypass attack is:
//
//   An attacker (or lazy dev) provides `reviewable_artifact = ""`
//   hoping to satisfy the dep_attested witness without actual review.
//
// Per ADR-025: dep_attested requires non-empty `reviewable_artifact`.
// Audit MUST emit `dep-attest-without-reviewable-artifact`.
//
// The two attack variants:
//   1. reviewable_artifact = "" (empty string)
//   2. reviewable_artifact omitted entirely
//
// Both MUST produce the `dep-attest-without-reviewable-artifact` hint.

use antigen::immune;

/// ATK-SC-1-A: Empty string reviewable_artifact.
/// This should fail audit with dep-attest-without-reviewable-artifact.
/// The antigen type referenced here will be UnpinnedDependency or
/// UnattestedDependencyInclusion — test verifies the audit hint fires.
#[immune(
    UnattestedDependencyInclusion,
    requires = dep_attested("serde", "1.0.200", reviewable_artifact = "")
)]
pub fn atk_sc1_empty_artifact() {}

/// ATK-SC-1-B: Missing reviewable_artifact entirely.
/// dep_attested with no artifact arg = rubber stamp.
/// This should ALSO fail audit with dep-attest-without-reviewable-artifact.
#[immune(
    UnattestedDependencyInclusion,
    requires = dep_attested("serde", "1.0.200")
)]
pub fn atk_sc1_missing_artifact() {}
