// Fixture: ContentHashMismatch presentation without immunity.
//
// Purpose: verify that scan_workspace correctly extracts a ContentHashMismatch
// presentation on a site that has NO immunity declaration. The audit should
// flag this as unaddressed (content-hash-no-attestation or just unaddressed).
//
// This fixture simulates code that uses a dependency that was added without
// recording a first-attestation content hash. The chalk/debug/eslint-config
// attack (2025) relied on this gap: Cargo.lock pins VERSION not CONTENT-HASH.
// Lockfile pinning alone does NOT prevent content replacement at a fixed version.
//
// ADR-025 §ContentHashMismatch — NON-NEGOTIABLE attack vector.

#[antigen(
    name = "ContentHashMismatch",
    fingerprint = "item = fn"
)]
pub struct ContentHashMismatch;

/// This function uses a dependency added via `cargo add serde` without recording
/// a content-hash first-attestation via `cargo antigen verify content-hash record`.
/// If the crate's published tarball is silently replaced at the same version,
/// no Cargo.lock change would occur and the substitution would go undetected.
#[presents(ContentHashMismatch)]
pub fn process_data(data: &str) -> String {
    // In a real project this would use serde or another dep
    data.to_uppercase()
}
