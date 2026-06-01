// ATK-SV-3 fixture: NFA-17 sidecar tier-inflation test.
//
// This fixture declares a substrate-witness immunity with a sidecar
// that claims CryptoSigned strength but provides no signature field.
//
// load_sidecar() deserializes the sidecar successfully (serde doesn't
// call validate). Without validate(), the audit trusts the CryptoSigned
// claim and reports SignatureStrength::CryptoSigned in the audit result.
//
// With the fix (validate() called in load_sidecar), the sidecar would be
// rejected as schema-invalid and the audit would emit DisciplineSidecarSchemaInvalid.

#[antigen(
    name = "nfa17-test-class",
    fingerprint = "item: fn"
)]
pub struct Nfa17TestClass;

#[presents(Nfa17TestClass)]
#[immune(
    Nfa17TestClass,
    requires = signers(required = ["alice"])
)]
pub fn guarded_fn() {
    // Body intentionally empty; we're testing the sidecar audit.
}
