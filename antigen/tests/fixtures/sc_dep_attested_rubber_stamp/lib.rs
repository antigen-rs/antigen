// Fixture: dep_attested witness without reviewable_artifact (rubber-stamp).
//
// Purpose: verify the rubber-stamp limitation enforcement.
//
// The key correctness invariant: `dep_attested` WITHOUT a non-empty
// `reviewable_artifact` path MUST emit the `dep-attest-without-reviewable-artifact`
// audit hint. An attestation with no reviewable artifact is a rubber-stamp —
// it provides no evidence that anyone actually reviewed the dependency.
//
// This is a NAMED LIMITATION in ADR-025 (known-limitation #1: rubber-stamp
// attestation). The audit hint enforces that adopters are told about the gap.
//
// Contrast with a WELL-FORMED attestation:
//   dep_attested("serde", "1.0.195", reviewable_artifact = "reviews/serde-1.0.195.md")
//
// ADR-025 §dep_attested — requires non-empty reviewable_artifact.

#[antigen(
    name = "UnattestedDependencyInclusion",
    fingerprint = "item = fn"
)]
pub struct UnattestedDependencyInclusion;

/// This function uses `serde` — we "attested" it but without reviewing anything.
/// This is a rubber-stamp: `dep_attested` without `reviewable_artifact` says
/// "I was here" but provides no evidence of actual review.
#[presents(UnattestedDependencyInclusion)]
#[immune(
    UnattestedDependencyInclusion,
    requires = dep_attested("serde", "1.0.195")
)]
pub fn serialize_rubber_stamp(data: &str) -> String {
    data.to_owned()
}
