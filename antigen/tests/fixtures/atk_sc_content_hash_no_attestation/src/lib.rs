// ATK-SC-2: ContentHashMismatch — no first-attestation exists.
//
// Per ADR-025: ContentHashMismatch requires proactive first-attestation
// via `cargo antigen verify content-hash record`. If no attestation exists,
// the audit MUST emit `content-hash-no-attestation` (not silently pass).
//
// The silent-pass would be the bypass: if no hash is recorded,
// the check can't fire. The defense is: absence of attestation = warning.
//
// This fixture has a content_hash_matches witness but NO corresponding
// .attest/supply-chain/content-hash/serde@1.0.200.json file.
// Audit MUST emit content-hash-no-attestation.

use antigen::immune;

/// ATK-SC-2: content_hash_matches witness with no recorded attestation.
/// Should produce content-hash-no-attestation hint (not pass silently).
#[immune(
    ContentHashMismatch,
    requires = content_hash_matches("serde", "1.0.200")
)]
pub fn atk_sc2_no_first_attestation() {}
