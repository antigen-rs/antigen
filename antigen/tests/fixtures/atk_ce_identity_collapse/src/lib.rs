// ATK-CE-3: IgG identity-collapse.
//
// Per ADR-024 C3: #[igg] source-independence is NOMINAL only.
// Same identity strings = same person = no independence.
// Audit MUST emit `igg-identity-collapse-warning` when duplicate signers appear.
//
// The bypass: a single person attests three times with the same email.
// min_reattestations = 3 is satisfied by COUNT but NOT by INDEPENDENCE.
// The audit should warn that independence is nominal-only and collapsed.
//
// Note: full structural verification of independence is a named limitation.
// But the DETECTION of OBVIOUS collapse (same identity string repeated)
// IS what the audit must do — it's a partial defense, not a complete one.

use antigen::igg;

/// ATK-CE-3-A: Duplicate signer strings (same person attesting twice).
/// min_reattestations = 2 satisfied by count, but signer identity collapses.
/// Expected: igg-identity-collapse-warning.
#[igg(
    witnesses = ["alice@example.com", "alice@example.com"],
    historical_span = 90,
    min_reattestations = 2
)]
pub fn atk_ce3_duplicate_signer() {}

/// ATK-CE-3-B: Three attestations, all same identity.
/// min_reattestations = 3. Maximum collapse.
/// Expected: igg-identity-collapse-warning.
#[igg(
    witnesses = ["bob@corp.com", "bob@corp.com", "bob@corp.com"],
    historical_span = 180,
    min_reattestations = 3
)]
pub fn atk_ce3_triple_same_identity() {}

/// ATK-CE-3-C: Correct case — distinct identities.
/// Should NOT emit identity-collapse-warning.
#[igg(
    witnesses = ["alice@example.com", "bob@corp.com", "carol@org.net"],
    historical_span = 180,
    min_reattestations = 3
)]
pub fn atk_ce3_distinct_signers() {}
