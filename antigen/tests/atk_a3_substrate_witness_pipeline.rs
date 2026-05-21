//! ATK-A3 substrate-witness pipeline wiring regression test.
//!
//! Bug history (rc.1 → rc.2 hotfix):
//!
//! The substrate-witness audit path (ADR-019 P3c) was structurally
//! unreachable. The macro-side `#[immune(X, requires = ...)]` lowered the
//! predicate to a `#[doc = " antigen:requires:v1:<json>"]` marker emitted
//! during macro expansion. The scan layer walked WRITTEN SOURCE via
//! `syn::parse_file` — the marker only appears AFTER expansion, so scan
//! never saw it. Every `Immunity::requires_predicate` was `None`, the
//! audit always fell through to `validate_witness`, and the user-facing
//! report was `tier = None, hint = NoneApplicable` ("missing witness
//! identifier") even when a valid substrate predicate was declared.
//!
//! The fix:
//! - Move `RequiresExpr` from `antigen-macros` to `antigen-attestation`
//!   behind a `parser` feature so the predicate AST + grammar live with
//!   the predicate type itself.
//! - Have scan parse `requires = <predicate>` directly from source
//!   attributes (primary), keeping the doc-marker channel as a fallback
//!   for backward compatibility with rc.1-compiled code.
//! - Route `to_json` through the real `Predicate` type so the JSON wire
//!   format is byte-identical to what the audit evaluator deserializes
//!   (rc.1's hand-rolled JSON did not match `Predicate` serde shape).
//!
//! This test pins the wiring at THREE layers — the substrate-currency
//! discipline applies to test substrate too. Each assertion calls out the
//! specific rc.1 failure mode it would have caught.

use antigen::audit::{audit, AuditHint, WitnessTier};
use antigen::scan::scan_workspace;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

/// Scan must populate `Immunity::requires_predicate` from a source
/// attribute. rc.1: this was always `None` because scan only looked at
/// post-expansion doc markers.
#[test]
fn scan_captures_requires_predicate_from_source_attribute() {
    let fx = fixture("atk_a3_substrate_witness_pipeline");
    let scan = scan_workspace(&fx, None).expect("scan completes");

    let immunity = scan
        .immunities
        .iter()
        .find(|i| i.antigen_type == "SignedZeroDiscipline")
        .expect("the fixture's substrate-witness immunity must be captured by scan");

    let predicate_json = immunity.requires_predicate.as_deref().unwrap_or_else(|| {
        panic!(
            "rc.1 regression: scan did not capture requires_predicate from \
             source attribute. immunity = {immunity:?}"
        )
    });

    // The JSON must round-trip as a real Predicate — locks the byte-format
    // contract between scan and the audit evaluator. rc.1 produced JSON
    // shaped like `{"kind":"leaf","leaf":{...}}` which `Predicate` serde
    // rejected as `sidecar_schema_invalid`.
    let _predicate: antigen_attestation::Predicate = serde_json::from_str(predicate_json)
        .unwrap_or_else(|e| {
            panic!(
                "rc.1 regression: scan-emitted predicate JSON does not deserialize as \
                 antigen_attestation::Predicate. JSON = {predicate_json}, error = {e}"
            )
        });
}

/// Audit must route a substrate-witness immunity through the substrate
/// path and report a substrate-specific hint. rc.1: the route was never
/// taken (because `requires_predicate` was always `None`), so the audit
/// fell through to code-witness handling and reported the misleading
/// `NoneApplicable` hint with a "missing witness identifier" diagnostic.
#[test]
fn audit_routes_substrate_witness_immunity_through_substrate_path() {
    let fx = fixture("atk_a3_substrate_witness_pipeline");
    let scan = scan_workspace(&fx, None).expect("scan completes");
    let report = audit(&scan, &fx);

    let audit_for_substrate = report
        .audits
        .iter()
        .find(|a| a.immunity.antigen_type == "SignedZeroDiscipline")
        .expect("immunity audit for SignedZeroDiscipline must exist");

    // No `.attest/SignedZeroDiscipline.json` sidecar exists in the fixture,
    // so the audit should report DisciplineSidecarMissing. This is the
    // exact diagnostic the rc.1 substrate pipeline would have produced
    // *if* it had been engaged — and the one the user actually needs to
    // see, since it points at the real next step (write the sidecar).
    assert_eq!(
        audit_for_substrate.audit_hint,
        AuditHint::DisciplineSidecarMissing,
        "rc.1 regression: substrate-witness audit must report \
         DisciplineSidecarMissing when no sidecar is present, not the \
         legacy `NoneApplicable` code-witness fallthrough. Got: {audit_for_substrate:?}"
    );

    // Tier-honesty (ADR-005 Amendment 3): sidecar-missing maps to
    // WitnessTier::None per `EvaluatedPredicate::sidecar_missing`. This
    // is the right answer — no substrate was found, so no claim has any
    // evidence yet.
    assert_eq!(
        audit_for_substrate.witness_tier,
        WitnessTier::None,
        "substrate-witness without sidecar must report tier=None"
    );

    // The evidence kind axis (ADR-019 §M5) is the load-bearing
    // distinguisher between "code-witness gave us nothing" and
    // "substrate-witness gave us nothing". rc.1 reported
    // EvidenceKind::None for both, collapsing the diagnostic.
    assert_eq!(
        audit_for_substrate.evidence_kind,
        antigen_attestation::EvidenceKind::SubstrateState,
        "substrate-witness pipeline must report EvidenceKind::SubstrateState \
         even when the sidecar is missing — the route was taken, the substrate \
         was checked, the result was empty"
    );

    // And evaluated_predicate must be present — proof that the substrate
    // evaluator ran. rc.1 left this `None` because the substrate path was
    // never entered.
    assert!(
        audit_for_substrate.evaluated_predicate.is_some(),
        "audit must surface the predicate JSON that was evaluated; rc.1 left this \
         None because the substrate pipeline never engaged"
    );
}
