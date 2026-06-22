//! Example: `#[igg]` — affinity-matured re-attestation evidence.
//!
//! ADR-024 convergent-evidence family. `#[igg]` captures the pattern of
//! a defense that has been *re-attested over time* by multiple
//! independent signers. The biological cognate is `IgG`-class antibodies
//! — produced AFTER initial `IgM` response, indicating affinity
//! maturation and durable immune memory.
//!
//! ## Named limitation (per ADR-024)
//!
//! `#[igg]` source-independence is NOMINAL only. Different signer
//! identity strings are not structural proof of independent sources —
//! two pseudonymous accounts controlled by the same person produce the
//! same "two distinct signers" signal as two genuinely independent
//! reviewers. The macro surfaces the discipline (re-attestation
//! discipline), not the metaphysical guarantee.
//!
//! Audit emits `igg-identity-collapse-warning` when all witnesses
//! share a single token-string identity.
//!
//! ## When to use `#[igg]`
//!
//! - A defense has been attested by multiple reviewers across time
//! - You want the structural memory of "this discipline has aged well"
//! - You want CI to fail if attestations expire faster than the
//!   `historical_span` floor
//!
//! ## Run this example
//!
//! ```sh
//! cargo run --example convergent_igg --package antigen
//! ```

use antigen::{antigen, igg};

#[antigen(
    name = "transaction-atomicity",
    fingerprint = r#"item = fn, name = matches("*_transaction*")"#,
    family = "concurrency-violation",
    summary = "Transaction operations must be atomic across both branches of any commit/rollback decision."
)]
pub struct TransactionAtomicity;

/// A transactional commit path with `IgG`-class affinity-matured
/// evidence: 4 distinct witnesses attested across 180+ days.
///
/// `min_reattestations = 4` enforces that at least four distinct
/// witness identifiers appear; `historical_span = 180` enforces that
/// the attestations span at least 180 days.
#[igg(
    witnesses = [alice_review, bob_audit, kani_proof, prop_atomicity],
    historical_span = 180,
    min_reattestations = 4,
)]
pub fn commit_transaction(tx_id: u64) -> Result<(), String> {
    if tx_id == 0 {
        Err("invalid tx_id".to_string())
    } else {
        Ok(())
    }
}

/// A weaker form with four identical witness identities.
///
/// All four witnesses are the same identity (`alice_review`). Audit
/// emits `igg-identity-collapse-warning` because the nominal-
/// independence test trivially passes — 4 distinct token-strings is
/// not the criterion here. The criterion is 4 distinct identities,
/// and 4 same-identity entries collapse.
///
/// This is exactly the NAMED LIMITATION.
#[igg(
    witnesses = [alice_review, alice_review, alice_review, alice_review],
    historical_span = 90,
    min_reattestations = 4,
)]
pub fn rollback_transaction(tx_id: u64) -> Result<(), String> {
    if tx_id == 0 {
        Err("invalid tx_id".to_string())
    } else {
        Ok(())
    }
}

fn main() {
    println!("=== antigen convergent-evidence: #[igg] example ===");
    println!();
    println!("Two IgG declarations:");
    println!();
    println!("1. commit_transaction");
    println!("   witnesses: 4 distinct (alice, bob, kani, prop)");
    println!("   historical_span: 180 days");
    println!("   min_reattestations: 4");
    println!("   audit: clean");
    println!();
    println!("2. rollback_transaction (identity-collapse demo)");
    println!("   witnesses: 4 identical (alice_review x4)");
    println!("   historical_span: 90 days");
    println!("   min_reattestations: 4");
    println!("   audit: igg-identity-collapse-warning");
    println!();
    println!("Per ADR-024:");
    println!("  `#[igg]` source-independence is NOMINAL only.");
    println!("  4 different signer-identity strings is not structural proof");
    println!("  of 4 independent sources. The named limitation is in the ADR;");
    println!("  the audit surfaces identity-collapse as a warning.");
    println!();
    println!("Sample evaluations:");
    println!("  commit_transaction(42) = {:?}", commit_transaction(42));
    println!("  rollback_transaction(0) = {:?}", rollback_transaction(0));
}
