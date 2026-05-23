// Fixture: ContentHashMismatch immunity with content_hash_matches substrate-witness.
//
// Purpose: verify that scan_workspace correctly extracts the content_hash_matches
// substrate-witness leaf from the `requires = ...` predicate, AND that the
// immunity addresses the presentation on the same item.
//
// The key correctness invariant to test:
//   ContentHashMismatch MUST fire when the recorded hash differs from the
//   current dep tarball hash — EVEN WHEN the version is identical.
//   Version-pinning does NOT protect against content replacement (ADR-025 B1-R).
//
// Used by: sc_content_hash_immune integration tests.
// ADR-025 §ContentHashMismatch — NON-NEGOTIABLE attack vector.

#[antigen(
    name = "ContentHashMismatch",
    fingerprint = "item = fn"
)]
pub struct ContentHashMismatch;

/// `process_data` uses the `serde` dependency.
/// The content-hash for serde@1.0.195 has been recorded via:
///   cargo antigen verify content-hash record serde@1.0.195
/// and is verified on each CI run.
#[presents(ContentHashMismatch)]
#[immune(
    ContentHashMismatch,
    requires = content_hash_matches("serde", "1.0.195")
)]
pub fn process_data(data: &str) -> String {
    data.to_uppercase()
}
