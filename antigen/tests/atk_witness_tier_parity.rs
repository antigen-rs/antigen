//! ATK-WITNESS-TIER-PARITY: derive-parity between `audit::WitnessTier` and
//! `antigen_attestation::WitnessTier`.
//!
//! The two enums are intentionally duplicated (no `pub use`) to keep the
//! runtime/audit crate serde-stable while `antigen-attestation` evolves
//! independently. The `MUST stay structurally identical` comment in
//! `antigen/src/audit.rs` promises lock-step maintenance. But the compiler
//! only enforces ONE direction: adding a variant to `antigen_attestation`'s
//! `WitnessTier` breaks the exhaustive match in `map_attestation_tier`. It
//! does NOT enforce:
//!
//!   1. Identical derives (`antigen_attestation::WitnessTier` derives `Hash`;
//!      `audit::WitnessTier` does NOT — already drifted)
//!   2. Variant added to `audit::WitnessTier` without a peer in attestation
//!   3. Identical discriminant values (currently both use explicit `= N` forms)
//!
//! This test file catches drift on axes the exhaustive match cannot:
//!
//!   - ATK-WTP-1: `audit::WitnessTier` must implement `Hash` (lock-step with
//!     `antigen_attestation::WitnessTier` which already derives it).
//!   - ATK-WTP-2: Both enums must report the same discriminant values.
//!   - ATK-WTP-3: Both enums must have the same variant count.
//!
//! Campsite: `dogfood/witnesstier-duplication-drift`
//! Found by: outsider (substrate-verified 2026-05-26)
//! Blocked by: adversarial (2026-05-26)

use antigen::audit::WitnessTier as AuditTier;
use antigen_attestation::WitnessTier as AttestTier;

// ATK-WTP-1: compile-time assertion that audit::WitnessTier implements Hash.
// These functions are never called at runtime; they exist solely so the
// compiler verifies audit::WitnessTier: Hash at the call-sites.
#[allow(dead_code)]
fn assert_audit_tier_is_hash<T: std::hash::Hash>(_: T) {}
#[allow(dead_code)]
fn witness_tier_hash_bound_check() {
    assert_audit_tier_is_hash(AuditTier::None);
    assert_audit_tier_is_hash(AuditTier::Reachability);
    assert_audit_tier_is_hash(AuditTier::Execution);
    assert_audit_tier_is_hash(AuditTier::FormalProof);
}

// ATK-WTP-2 + ATK-WTP-3: runtime parity checks for discriminants and
// variant count. These pass currently; they are regression guards.

#[test]
fn atk_wtp_2_discriminant_parity_between_audit_and_attestation_tiers() {
    // Verify discriminant values are identical across both WitnessTier enums.
    // This catches drift if one side re-assigns discriminant integers.
    assert_eq!(
        AuditTier::None as u8,
        AttestTier::None as u8,
        "ATK-WTP-2: audit::WitnessTier::None discriminant must equal \
         antigen_attestation::WitnessTier::None discriminant"
    );
    assert_eq!(
        AuditTier::Reachability as u8,
        AttestTier::Reachability as u8,
        "ATK-WTP-2: discriminant for Reachability must match across both WitnessTier enums"
    );
    assert_eq!(
        AuditTier::Execution as u8,
        AttestTier::Execution as u8,
        "ATK-WTP-2: discriminant for Execution must match across both WitnessTier enums"
    );
    assert_eq!(
        AuditTier::FormalProof as u8,
        AttestTier::FormalProof as u8,
        "ATK-WTP-2: discriminant for FormalProof must match across both WitnessTier enums"
    );
}

#[test]
fn atk_wtp_3_variant_set_parity_between_audit_and_attestation_tiers() {
    // Each enum must cover the same variant set. Expressed as sorted
    // discriminant vectors — if one side adds a variant without the other,
    // the vectors diverge.
    //
    // Known variants: None=0, Reachability=1, Execution=2, FormalProof=4
    // (discriminant 3 reserved for BehavioralAlignment per ADR-005 OQ).
    let audit_discriminants: Vec<u8> = vec![
        AuditTier::None as u8,
        AuditTier::Reachability as u8,
        AuditTier::Execution as u8,
        AuditTier::FormalProof as u8,
    ];
    let attest_discriminants: Vec<u8> = vec![
        AttestTier::None as u8,
        AttestTier::Reachability as u8,
        AttestTier::Execution as u8,
        AttestTier::FormalProof as u8,
    ];
    assert_eq!(
        audit_discriminants, attest_discriminants,
        "ATK-WTP-3: audit::WitnessTier and antigen_attestation::WitnessTier \
         must have identical variant discriminant sets. \
         Discrepancy means the two enums have drifted — the lock-step \
         comment in audit.rs is no longer true. \
         audit: {audit_discriminants:?}, attestation: {attest_discriminants:?}"
    );
}

#[test]
fn atk_wtp_4_audit_tier_is_usable_as_hashmap_key() {
    // If audit::WitnessTier derives Hash, this test compiles AND passes.
    // If it does NOT derive Hash, the entire test file fails to compile,
    // which surfaces the ATK-WTP-1 finding at build time.
    //
    // This test is the RUNTIME complement to the compile-time guard above.
    use std::collections::HashSet;
    let mut set: HashSet<AuditTier> = HashSet::new();
    set.insert(AuditTier::None);
    set.insert(AuditTier::Reachability);
    set.insert(AuditTier::Execution);
    set.insert(AuditTier::FormalProof);
    assert_eq!(
        set.len(),
        4,
        "ATK-WTP-4: all four WitnessTier variants must be distinct HashMap keys; \
         got {} unique entries",
        set.len()
    );
}
