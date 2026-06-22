// Fixture: #[igg] with identity-collapsed witnesses (same identity string twice).
//
// Purpose: verify the IgG IDENTITY-COLLAPSE correctness invariant.
//
// ADR-024 C3 + named limitation: `#[igg]` source-independence is
// NOMINAL (different signer identity strings) not STRUCTURAL. The audit MUST
// emit `igg-identity-collapse-warning` when the witnesses array contains
// duplicate identity strings — same person signing twice is NOT independent
// re-attestation regardless of timing.
//
// The named limitation: antigen cannot MECHANICALLY verify source-independence
// (ADR-024 §"What this ADR does NOT do"). But it CAN detect the obvious case
// of duplicate identity strings and warn.
//
// Failing-as-passing intent: audit MUST detect identity collapse here.
// If this fixture does NOT trigger igg-identity-collapse-warning, the audit
// has a false-negative on the most basic identity-collapse case.
//
// ADR-024 §IgG — source-independence NOMINAL; identity-collapse is detectable.

#[antigen(
    name = "ReviewedAlgorithm",
    fingerprint = "item: fn"
)]
pub struct ReviewedAlgorithm;

/// An algorithm with an IgG declaration where the same person ("alice") appears
/// twice in the witnesses list. This is identity collapse — alice reviewing her
/// own work at two different times does not provide independent re-attestation.
/// The audit must warn about this.
#[presents(ReviewedAlgorithm)]
#[igg(
    witnesses = ["alice", "alice"],
    historical_span = 180,
    min_reattestations = 3
)]
pub fn sort_and_deduplicate(data: &mut Vec<i64>) {
    data.sort_unstable();
    data.dedup();
}
