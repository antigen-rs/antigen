//! Example: `ContentHashMismatch` — the chalk/debug/eslint-config
//! attack defense.
//!
//! ADR-025 supply-chain defense family. `ContentHashMismatch` is the
//! NON-NEGOTIABLE antigen for the 2025 content-replacement-at-fixed-
//! version attack class. `Cargo.lock` pins VERSION but NOT CONTENT-
//! HASH; lockfile pinning alone would not have prevented chalk/debug.
//!
//! ## The attack
//!
//! 1. Attacker compromises a maintainer account
//! 2. Attacker publishes a *modified* version of an already-released
//!    `<crate>@<version>`
//! 3. Adopters downloading `<crate>@<version>` for the first time get
//!    the modified content; their `Cargo.lock` checksum records the
//!    *modified* hash as the baseline
//! 4. No version bump, no diff to review — the attack is invisible to
//!    standard dep-management discipline
//!
//! ## The defense
//!
//! Proactive first-attestation:
//!
//! ```sh
//! cargo antigen verify content-hash record serde@1.0.197
//! ```
//!
//! creates `.attest/supply-chain/content-hash/serde@1.0.197.json` with
//! the current content-hash. Subsequent CI runs invoke
//!
//! ```sh
//! cargo antigen verify content-hash serde@1.0.197
//! ```
//!
//! which compares the current `Cargo.lock` checksum against the
//! recorded hash. Divergence emits `content-hash-mismatch` (the attack
//! signal); missing record emits `content-hash-no-attestation` (the
//! cold-start signal — NOT a silent pass).
//!
//! ## Three-way distinguishable failure surface (per ATK-SC-2-A)
//!
//! 1. `NoAttestation` — sidecar missing (cold start)
//! 2. `Mismatch` — sidecar present + hash differs (THE ALERT)
//! 3. ``SidecarMalformed`` — sidecar exists but JSON is corrupt
//!    (someone tampered with the .attest/ file)
//!
//! Collapsing #3 into #1 would let an attacker downgrade a `Mismatch`
//! into a `NoAttestation` by corrupting the sidecar JSON. The audit
//! distinguishes them explicitly.
//!
//! ## Run this example
//!
//! ```sh
//! cargo run --example supply_chain_content_hash --package antigen
//! ```

// Note: stdlib antigen paths in #[presents] / #[immune] are tokenized
// by the proc-macros; they don't need to resolve as Rust values.
#[allow(unused_imports)]
use antigen::stdlib::supply_chain::ContentHashMismatch;
use antigen::{immune, presents};

/// A function whose `serde` dependency is content-hash-attested.
///
/// The `requires = content_hash_matches(...)` substrate-witness
/// predicate is evaluated by `audit_supply_chain` against the
/// `.attest/supply-chain/content-hash/serde@1.0.197.json` record + the
/// current `Cargo.lock` checksum.
#[presents(ContentHashMismatch)]
#[immune(
    ContentHashMismatch,
    requires = content_hash_matches("serde", "1.0.197"),
)]
pub fn deserialize_config(input: &str) -> Result<String, String> {
    // In real code this would invoke serde_json::from_str or similar.
    // The defense isn't in the function body — it's in the structural
    // memory that this function depends on serde@1.0.197 with a
    // recorded content-hash.
    Ok(input.to_uppercase())
}

fn main() {
    println!("=== antigen supply-chain: ContentHashMismatch example ===");
    println!();
    println!("The NON-NEGOTIABLE defense against the chalk/debug attack class.");
    println!();
    println!("To activate:");
    println!("  cargo antigen verify content-hash record serde@1.0.197");
    println!("    → records current hash in .attest/supply-chain/content-hash/");
    println!();
    println!("In CI:");
    println!("  cargo antigen verify content-hash serde@1.0.197");
    println!("    → MATCHES: predicate passes");
    println!("    → MISMATCH: emits `content-hash-mismatch` (attack signal)");
    println!("    → `NoAttestation`: emits `content-hash-no-attestation`");
    println!("    → Malformed: emits `content-hash-sidecar-malformed`");
    println!();
    println!("Per ATK-SC-2-A: all three failure modes are STRUCTURALLY DISTINCT.");
    println!("Corrupting the sidecar to convert `Mismatch` into `NoAttestation`");
    println!("would be the attacker's next move; the audit blocks that path.");
    println!();
    let result = deserialize_config("hello");
    println!("deserialize_config(\"hello\") = {result:?}");
}
