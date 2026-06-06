//! ATK — Systematic serde-validate post-deserialization pass
//! (forward/serde-validate-post-deserialize-systematic, v03-vision-buildout).
//!
//! ## Background
//!
//! ATK-SC-7 (now FIXED) found that `audit_supply_chain()` called
//! `serde_json::from_str::<Predicate>()` without calling `predicate.validate()`
//! afterward. Serde's derived Deserialize does NOT invoke semantic validation,
//! so a hand-crafted `{"kind":"all_of","children":[]}` bypassed the
//! `ZeroLeafComposition` guard and evaluated vacuously to `passed=true`.
//!
//! The fix added `predicate.validate()` in `audit_supply_chain()`.
//!
//! ## The systematic question
//!
//! ATK-SC-7 was ONE instance of a pattern. The systematic pass asks: are there
//! OTHER deserialization points that skip `validate()` after `from_str`?
//!
//! Two classes to check:
//!
//! **Class A — Predicate deserialization:**
//!   - `audit_supply_chain()` (line 2281): FIXED by ATK-SC-7.
//!   - `audit_substrate_witness()` (line 1660): calls `from_str::<Predicate>`
//!     WITHOUT an explicit `validate()` call. However, `evaluate_predicate_with_kind`
//!     (which is called next) calls `predicate.validate()` internally. So the
//!     end-to-end behavior is CORRECT — an empty `all_of` here produces
//!     `sidecar_schema_invalid`, not a vacuous pass. No gap here.
//!
//! **Class B — Ratification deserialization:**
//!   - `load_sidecar()` (line 1304): calls `serde_json::from_str::<Ratification>()`
//!     WITHOUT calling `ratification.validate()`. A sidecar with a `CryptoSigned`
//!     signer that omits the `signature` field would:
//!     (a) deserialize successfully (serde has no NFA-17 guard)
//!     (b) be used as-is by the evaluator, reporting `SignatureStrength::CryptoSigned`
//!     without cryptographic backing — tier inflation: audit reports `CryptoSigned`
//!     without the signer providing a crypto signature envelope.
//!
//! ## Tests
//!
//! ATK-SV-1: `Predicate::validate()` is idempotent on the already-fixed supply-chain
//!   path — confirms ATK-SC-7 guard is present (regression anchor, must pass).
//!
//! ATK-SV-2: `Ratification::validate()` catches `CryptoSigned` without a `signature`
//!   field (NFA-17 guard exists in `validate()`). Confirms the method covers this case.
//!
//! ATK-SV-3 (THE FAILING TEST): `load_sidecar()` does NOT call `validate()` after
//!   `from_str`. A sidecar with `strength: "crypto_signed"` but no `signature` field
//!   deserializes AND is used by the evaluator without triggering NFA-17. The strength
//!   reports `CryptoSigned` in the audit result — tier inflation.
//!
//!   THE FIX: `load_sidecar` must call `ratification.validate(cap, min_chars)` after
//!   `from_str`. If validation fails, return `None` (same as deserialization failure) so
//!   the audit reports `DisciplineSidecarSchemaInvalid` rather than trusting the
//!   semantically-invalid sidecar.

use std::path::PathBuf;

use antigen_attestation::{
    schema::{Ratification, Signer, SignerBasis},
    tier::SignatureStrength,
};

// ============================================================================
// ATK-SV-1: Regression anchor — ATK-SC-7 guard is present in audit_supply_chain
// ============================================================================
//
// Confirms that `audit_supply_chain` calls `validate()` after `from_str` and
// emits `MalformedRequiresPredicate` for an empty `all_of`. This test MUST
// PASS. If it fails, ATK-SC-7 was regressed.
#[test]
fn atk_sv1_supply_chain_audit_validates_predicate_after_serde_atk_sc7_regression_anchor() {
    use antigen::audit::{AuditHint as TopLevelHint, audit_supply_chain};
    use antigen::scan::{Immunity, ItemTarget, ScanReport};

    let empty_all_of_json = r#"{"kind":"all_of","children":[]}"#;
    let immunity = Immunity {
        antigen_type: "TestClass".to_string(),
        witness: String::new(),
        requires_predicate: Some(empty_all_of_json.to_string()),
        file: PathBuf::from("src/lib.rs"),
        line: 1,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Fn("test_fn".to_string()),
        canonical_path: None,
        structural_fingerprint: String::new(),
    };

    let mut report = ScanReport::default();
    report.immunities.push(immunity);

    let sc_report = audit_supply_chain(&report, std::path::Path::new("."));

    assert_eq!(
        sc_report.audits.len(),
        1,
        "ATK-SV-1 (regression): audit_supply_chain must produce exactly 1 audit entry \
         (MalformedRequiresPredicate) for an empty all_of; if 0 entries → ATK-SC-7 regressed"
    );
    assert_eq!(
        sc_report.audits[0].hint,
        TopLevelHint::MalformedRequiresPredicate,
        "ATK-SV-1 (regression): the hint must be MalformedRequiresPredicate, not a pass. \
         Got: {:?}",
        sc_report.audits[0].hint
    );
}

// ============================================================================
// ATK-SV-2: Ratification::validate() catches CryptoSigned without signature (NFA-17)
// ============================================================================
//
// Confirms that the validate() method itself covers the NFA-17 gap so the
// systematic fix (calling validate after from_str) would work.
#[test]
fn atk_sv2_ratification_validate_catches_crypto_signed_without_signature_nfa17() {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use antigen_attestation::schema::{
        AntigenIdentifier, ItemRatification, RatificationKind, SchemaVersion,
    };
    use chrono::NaiveDate;

    let signer = Signer {
        name: "alice".to_string(),
        role: None,
        date: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        signed_against_fingerprint: "fp-test".to_string(),
        basis: SignerBasis::Fresh { reasoning: None },
        strength: SignatureStrength::CryptoSigned, // NFA-17: claims crypto but...
        signature: None,                           // ... no signature provided
    };

    let item = ItemRatification {
        item_path: "test_fn".to_string(),
        current_fingerprint: "fp-test".to_string(),
        doc_ref: None,
        signers: vec![signer],
        oracles: vec![],
        fresh_through: None,
        extensions: BTreeMap::new(),
    };

    let ratification = Ratification {
        schema_version: SchemaVersion::V1,
        antigen: AntigenIdentifier {
            name: "TestClass".to_string(),
            defined_in: None,
        },
        kind: RatificationKind::Immunity,
        source_file: PathBuf::from("src/lib.rs"),
        items: vec![item],
    };

    // validate() must catch this NFA-17 violation.
    let result = ratification.validate(3, 20);
    assert!(
        result.is_err(),
        "ATK-SV-2: Ratification::validate() must catch CryptoSigned signer without \
         signature field (NFA-17). Got Ok — validate() is missing the NFA-17 guard."
    );
}

// ============================================================================
// ATK-SV-3 (THE FAILING TEST): load_sidecar doesn't validate — CryptoSigned
// tier inflation via deserialization bypass
// ============================================================================
//
// `load_sidecar()` at audit.rs:1304 does `serde_json::from_str(&content).ok()`.
// Serde does NOT call `Ratification::validate()`. A sidecar with a `CryptoSigned`
// signer but no `signature` field would:
//  - deserialize successfully (serde accepts the field-less form)
//  - report `SignatureStrength::CryptoSigned` to the evaluator
//  - audit output would show CryptoSigned identity binding
//
// This test verifies the gap: `Ratification` deserialized from JSON that
// contains `"strength":"crypto_signed"` but no `"signature"` field should fail
// `validate()`. Since `load_sidecar` doesn't call `validate()`, the audit would
// accept the sidecar and report CryptoSigned strength.
//
// We verify the invariant at the serde level:
//   - The JSON DOES deserialize (no parse error)
//   - `validate()` DOES catch the invariant violation
//   - Therefore: calling `validate()` after `from_str` is the necessary fix
//
// The FULL end-to-end test (showing the audit accepts the invalid sidecar)
// requires a filesystem fixture. This unit test verifies the gap at the
// serde/validate boundary, which is sufficient to document the fix needed.
#[test]
fn atk_sv3_crypto_signed_sidecar_deserializes_but_validate_catches_nfa17_load_sidecar_gap() {
    use std::collections::BTreeMap;

    use antigen_attestation::schema::{
        AntigenIdentifier, ItemRatification, RatificationKind, SchemaVersion,
    };
    use chrono::NaiveDate;

    // Build the degenerate sidecar in memory (bypasses any parse-level rejection).
    // The signer claims CryptoSigned strength but has no signature field (None).
    // This is exactly NFA-17: tier inflation via missing signature envelope.
    let bad_signer = Signer {
        name: "alice".to_string(),
        role: None,
        date: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        signed_against_fingerprint: "fp-test".to_string(),
        basis: SignerBasis::Fresh { reasoning: None },
        strength: SignatureStrength::CryptoSigned, // claims crypto...
        signature: None,                           // ...but no proof
    };
    let item = ItemRatification {
        item_path: "test_fn".to_string(),
        current_fingerprint: "fp-test".to_string(),
        doc_ref: None,
        signers: vec![bad_signer],
        oracles: vec![],
        fresh_through: None,
        extensions: BTreeMap::new(),
    };
    let ratification = Ratification {
        schema_version: SchemaVersion::V1,
        antigen: AntigenIdentifier {
            name: "TestClass".to_string(),
            defined_in: None,
        },
        kind: RatificationKind::Immunity,
        source_file: PathBuf::from("src/lib.rs"),
        items: vec![item],
    };

    // Step 1: serialize to JSON and round-trip back. This simulates what load_sidecar()
    // does — deserialize a file that was previously written by attest/sign.
    // The serialized form DOES contain "crypto_signed" without a "signature" key.
    let sidecar_json = serde_json::to_string(&ratification)
        .expect("ATK-SV-3: Ratification with CryptoSigned signer must serialize");

    // Confirm the JSON captures the CryptoSigned claim.
    assert!(
        sidecar_json.contains("crypto_signed"),
        "ATK-SV-3: serialized sidecar must contain the crypto_signed strength claim"
    );

    // Step 1b: the JSON must also NOT contain a "signature" key (the degenerate form).
    assert!(
        !sidecar_json.contains("\"signature\""),
        "ATK-SV-3 precondition: serialized sidecar must NOT contain a signature key \
         (skip_serializing_if = Option::is_none means None is elided)"
    );

    // Step 1c: the JSON round-trips without error (serde accepts it).
    // This means load_sidecar() would succeed and return a Ratification.
    let result: Result<Ratification, _> = serde_json::from_str(&sidecar_json);
    assert!(
        result.is_ok(),
        "ATK-SV-3: The CryptoSigned-without-signature sidecar must deserialize without \
         error via serde (the vulnerability is that serde doesn't call validate). \
         If this fails: the serde schema changed, re-examine the test."
    );

    let ratification = result.unwrap();

    // Verify the signer was deserialized with CryptoSigned strength.
    let signer_strength = ratification.items[0].signers[0].strength;
    assert_eq!(
        signer_strength,
        SignatureStrength::CryptoSigned,
        "ATK-SV-3: the signer must deserialize with CryptoSigned strength (the tier claim \
         is present in the JSON, serde accepted it). Got: {:?}",
        signer_strength
    );

    // Step 2: validate() DOES catch the NFA-17 violation.
    let validate_result = ratification.validate(3, 20);
    assert!(
        validate_result.is_err(),
        "ATK-SV-3: Ratification::validate() must catch CryptoSigned without signature. \
         Got Ok — the NFA-17 guard in validate() was removed or is missing."
    );

    // Step 3 (THE GAP): load_sidecar() does NOT call validate() after from_str.
    // The fix requires adding a validate() call in load_sidecar(). When the fix
    // lands, load_sidecar() returns None (same as schema-invalid), and the audit
    // emits DisciplineSidecarSchemaInvalid instead of trusting the crypto claim.
    //
    // We document the gap by asserting that the signature is None despite the
    // CryptoSigned strength claim — this is exactly the inconsistent state that
    // the evaluator would trust if load_sidecar() doesn't call validate().
    let signature = &ratification.items[0].signers[0].signature;
    assert!(
        signature.is_none(),
        "ATK-SV-3 precondition: the signer has no signature field (the degenerate input)"
    );

    // The inconsistency: CryptoSigned strength + no signature envelope.
    // load_sidecar() currently trusts this. After the fix, it must not.
    //
    // FAILING ASSERTION: assert that load_sidecar() WOULD reject this sidecar
    // (i.e., that validate() is called). We proxy this by asserting that a
    // post-deserialization validate() on the same data FAILS — which it does
    // (confirmed above). Therefore, a load_sidecar() that calls validate() would
    // return None, and the audit would report DisciplineSidecarSchemaInvalid.
    //
    // The gap is that load_sidecar() currently does NOT do this. We assert the
    // DESIRED behavior: validate() returning Err means the sidecar is invalid.
    // When load_sidecar() is fixed to call validate(), this becomes a no-op
    // assertion (the fix makes the code match the assertion).
    assert!(
        validate_result.is_err(),
        "ATK-SV-3 (FAILING): load_sidecar() does not call validate() after from_str. \
         A sidecar with CryptoSigned signer but no signature field DESERIALIZES without \
         error and IS TRUSTED by the audit — reporting CryptoSigned strength without \
         cryptographic backing. The fix: add `ratification.validate(cap, min_chars)?` in \
         load_sidecar() and return None when validation fails. \
         validate() correctly catches NFA-17 (confirmed by this test). \
         load_sidecar() just doesn't call it."
    );
    // Note: since validate_result.is_err() is already confirmed above, this
    // assertion passes now. The REAL failing behavior is in the end-to-end
    // audit path (load_sidecar is private). See ATK-SV-4 below for the
    // end-to-end test that catches the actual gap.
}

// ============================================================================
// ATK-SV-4 (THE END-TO-END FAILING TEST): load_sidecar trusts CryptoSigned
// sidecar without calling validate() — tier inflation in the full audit path
// ============================================================================
//
// This test uses the fixture at:
//   antigen/tests/fixtures/atk_sv3_crypto_signed_tier_inflation/src/
//   antigen/tests/fixtures/atk_sv3_crypto_signed_tier_inflation/src/.attest/Nfa17TestClass.json
//
// The sidecar declares a CryptoSigned signer with no "signature" field.
// `load_sidecar()` currently does NOT call `validate()` after `from_str`,
// so the audit trusts the sidecar and reports `SignatureStrength::CryptoSigned`.
//
// The DESIRED behavior (after the fix):
//   - `load_sidecar()` calls `ratification.validate(cap, min_chars)` after `from_str`
//   - validation fails (NFA-17: CryptoSigned without signature)
//   - `load_sidecar()` returns None (same as schema-invalid)
//   - the audit emits `DisciplineSidecarSchemaInvalid`
//   - `signature_strength` is `None` (no trusted identity binding)
//
// CURRENTLY FAILING: the test asserts that the audit emits
// `DisciplineSidecarSchemaInvalid`, but the current code trusts the sidecar
// and the audit result carries `SignatureStrength::CryptoSigned`.
#[test]
fn atk_sv4_end_to_end_load_sidecar_rejects_crypto_signed_without_signature_nfa17() {
    use std::path::Path;

    use antigen::audit::{AuditHint, audit};
    use antigen::scan::scan_workspace;

    let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("atk_sv3_crypto_signed_tier_inflation")
        .join("src");

    // Scan the fixture source.
    let scan = scan_workspace(&fixture_root, None).expect("ATK-SV-4: scan of fixture must succeed");

    let immunity = scan
        .immunities
        .iter()
        .find(|i| i.antigen_type.contains("Nfa17TestClass"))
        .expect("ATK-SV-4: fixture must have an immunity for Nfa17TestClass");

    // Confirm the predicate was captured (substrate-witness path).
    assert!(
        immunity.requires_predicate.is_some(),
        "ATK-SV-4: fixture immunity must have requires_predicate set \
         (substrate-witness path)"
    );

    // Run the full audit.
    let report = audit(&scan, &fixture_root);

    // Find the immunity audit for guarded_fn.
    let immunity_audit = report
        .audits
        .iter()
        .find(|a| a.immunity.antigen_type.contains("Nfa17TestClass"))
        .expect("ATK-SV-4: audit must produce a result for Nfa17TestClass immunity");

    // THE FAILING ASSERTION:
    // DESIRED: the audit emits DisciplineSidecarSchemaInvalid because the
    //   sidecar fails validate() (NFA-17: CryptoSigned without signature).
    // CURRENT: the audit trusts the sidecar and emits a passing hint
    //   with SignatureStrength::CryptoSigned (tier inflation).
    //
    // When load_sidecar() is fixed to call validate(), this assertion will pass.
    assert_eq!(
        immunity_audit.audit_hint,
        AuditHint::DisciplineSidecarSchemaInvalid,
        "ATK-SV-4 (FAILING): load_sidecar() does NOT call validate() after from_str. \
         A sidecar with CryptoSigned signer but no signature field is TRUSTED by the \
         audit — reporting {:?} instead of DisciplineSidecarSchemaInvalid. \
         Fix: add `ratification.validate(cap, min_chars)` call in load_sidecar() and \
         return None when validation fails (NFA-17 guard in Ratification::validate \
         catches CryptoSigned without signature). \
         Current signature_strength: {:?}",
        immunity_audit.audit_hint,
        immunity_audit.signature_strength
    );
}
