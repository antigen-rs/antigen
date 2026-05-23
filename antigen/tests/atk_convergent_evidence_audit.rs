//! ATK-CE convergent-evidence audit adversarial tests.
//!
//! These tests attack `audit_convergent_evidence()` in `antigen::audit`.
//! They run against pathmaker's implementation and FAIL where bugs exist.
//!
//! ADR-024 defines the enforcement surface. These tests verify that the
//! enforcement surface is correctly implemented, not just the happy path.

use antigen::audit::{audit_convergent_evidence, AuditHint};
use antigen::scan::{ConvergentEvidence, ConvergentEvidenceKind, ItemTarget, ScanReport};
use std::path::PathBuf;

/// Helper: build a `ScanReport` with a single `ConvergentEvidence` entry.
fn report_with(decl: ConvergentEvidence) -> ScanReport {
    let mut r = ScanReport::default();
    r.convergent_evidences.push(decl);
    r
}

/// Helper: build the minimal `ConvergentEvidence` for a given kind.
fn ce(kind: ConvergentEvidenceKind) -> ConvergentEvidence {
    ConvergentEvidence {
        kind,
        modality_classes: Vec::new(),
        min_independent: None,
        witness: None,
        iterations: None,
        seed_kind: None,
        historical_span: None,
        min_reattestations: None,
        witnesses: Vec::new(),
        fingerprints: Vec::new(),
        file: PathBuf::from("src/lib.rs"),
        line: 1,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Fn("test_fn".to_string()),
    }
}

// ============================================================================
// ATK-CE-1-A: Class-collapse detection (audit-time redundancy check)
//
// Per ADR-024 C1: min_independent = distinct classes, not raw count.
// The proc-macro catches this at compile time. The audit-time check covers
// pre-compiled or older source that bypassed compile-time enforcement.
//
// Two witnesses of same class with min_independent = 2 must emit
// DiagnosticModalitiesClassCollapsed AND DiagnosticModalityInsufficient.
// ============================================================================

#[test]
fn atk_ce1a_same_class_twice_collapses_at_audit_time() {
    let decl = ConvergentEvidence {
        kind: ConvergentEvidenceKind::Diagnostic,
        modality_classes: vec!["StaticAnalysis".to_string(), "StaticAnalysis".to_string()],
        min_independent: Some(2),
        ..ce(ConvergentEvidenceKind::Diagnostic)
    };
    let report = audit_convergent_evidence(&report_with(decl));

    let hints: Vec<_> = report.audits.iter().flat_map(|a| &a.hints).collect();

    assert!(
        hints.contains(&&AuditHint::DiagnosticModalitiesClassCollapsed),
        "ATK-CE-1-A: [StaticAnalysis, StaticAnalysis] with min_independent=2 \
         must emit DiagnosticModalitiesClassCollapsed at audit time. \
         Got: {:?}",
        hints
    );

    // ALSO: 1 distinct class < 2 min_independent → DiagnosticModalityInsufficient
    assert!(
        hints.contains(&&AuditHint::DiagnosticModalityInsufficient),
        "ATK-CE-1-A: 1 distinct class < min_independent=2 must also emit \
         DiagnosticModalityInsufficient. Got: {:?}",
        hints
    );
}

#[test]
fn atk_ce1b_different_classes_no_collapse() {
    // [StaticAnalysis, PropertyTest] is 2 distinct classes — no collapse.
    let decl = ConvergentEvidence {
        kind: ConvergentEvidenceKind::Diagnostic,
        modality_classes: vec!["StaticAnalysis".to_string(), "PropertyTest".to_string()],
        min_independent: Some(2),
        ..ce(ConvergentEvidenceKind::Diagnostic)
    };
    let report = audit_convergent_evidence(&report_with(decl));

    let hints: Vec<_> = report.audits.iter().flat_map(|a| &a.hints).collect();

    assert!(
        !hints.contains(&&AuditHint::DiagnosticModalitiesClassCollapsed),
        "ATK-CE-1-B: [StaticAnalysis, PropertyTest] must NOT collapse. Got: {:?}",
        hints
    );
    assert!(
        !hints.contains(&&AuditHint::DiagnosticModalityInsufficient),
        "ATK-CE-1-B: 2 distinct classes >= min_independent=2 must not be insufficient. \
         Got: {:?}",
        hints
    );
}

// ============================================================================
// ATK-CE-2: Fixed-seed detection at audit time
//
// The proc-macro rejects Fixed seed at compile time. But pre-compiled or
// older source may have it. The audit must still detect and surface it.
// ============================================================================

#[test]
fn atk_ce2_fixed_seed_detected_at_audit_time() {
    let decl = ConvergentEvidence {
        kind: ConvergentEvidenceKind::Clonal,
        witness: Some("my_test".to_string()),
        iterations: Some(1000),
        seed_kind: Some("Fixed".to_string()), // pre-compile-check source
        ..ce(ConvergentEvidenceKind::Clonal)
    };
    let report = audit_convergent_evidence(&report_with(decl));

    let hints: Vec<_> = report.audits.iter().flat_map(|a| &a.hints).collect();

    assert!(
        hints.contains(&&AuditHint::ClonalFixedSeedDetected),
        "ATK-CE-2: Fixed seed_kind in scan report must emit ClonalFixedSeedDetected \
         even though proc-macro enforces at parse time. \
         Pre-compiled or older source can bypass parse-time; audit is the fallback. \
         Got: {:?}",
        hints
    );
}

// ============================================================================
// ATK-CE-3-A: COMPLETE identity collapse — all same signer
//
// `witnesses = [alice, alice, alice]` with min_reattestations=3 MUST emit
// IggIdentityCollapseWarning because all witnesses are the same identity.
// ============================================================================

#[test]
fn atk_ce3a_complete_identity_collapse_emits_warning() {
    let decl = ConvergentEvidence {
        kind: ConvergentEvidenceKind::Igg,
        witnesses: vec![
            "alice@example.com".to_string(),
            "alice@example.com".to_string(),
            "alice@example.com".to_string(),
        ],
        historical_span: Some(90),
        min_reattestations: Some(3),
        ..ce(ConvergentEvidenceKind::Igg)
    };
    let report = audit_convergent_evidence(&report_with(decl));

    let hints: Vec<_> = report.audits.iter().flat_map(|a| &a.hints).collect();

    assert!(
        hints.contains(&&AuditHint::IggIdentityCollapseWarning),
        "ATK-CE-3-A: [alice, alice, alice] with min_reattestations=3 must emit \
         IggIdentityCollapseWarning — one person attesting 3 times is NOT 3 \
         independent reattestations. Got: {:?}",
        hints
    );
}

// ============================================================================
// ATK-CE-3-B: PARTIAL identity collapse — some duplicates
//
// ATTACK: `witnesses = [alice, alice, bob]` with min_reattestations=3.
// Raw count = 3 >= 3 → no IggReattestationsInsufficient.
// Unique count = 2 < 3 → SHOULD emit warning but currently doesn't.
//
// The bug: `IggReattestationsInsufficient` uses raw witnesses.len() instead
// of unique count. Alice's double-signing inflates the apparent attestation
// count. The adopter believes they have 3-way independent attestation but
// actually have only 2 independent sources.
//
// Per ADR-024 C3: source-independence is NOMINAL only; this is a best-effort
// check. But the REATTESTATIONS INSUFFICIENT check should at minimum detect
// obvious duplication when unique count < min_reattestations.
// ============================================================================

#[test]
fn atk_ce3b_partial_identity_collapse_with_insufficient_unique_sources() {
    // alice appears twice; bob appears once.
    // unique sources = 2, but min_reattestations = 3.
    // The adopter claims 3-way independence but only has 2 sources.
    let decl = ConvergentEvidence {
        kind: ConvergentEvidenceKind::Igg,
        witnesses: vec![
            "alice@example.com".to_string(),
            "alice@example.com".to_string(), // duplicate
            "bob@corp.com".to_string(),
        ],
        historical_span: Some(90),
        min_reattestations: Some(3),
        ..ce(ConvergentEvidenceKind::Igg)
    };
    let report = audit_convergent_evidence(&report_with(decl));

    let hints: Vec<_> = report.audits.iter().flat_map(|a| &a.hints).collect();

    // ADVERSARIAL PATTERN: this test FAILS when the bug EXISTS.
    // When the fix is applied (use unique count for IggReattestationsInsufficient),
    // the test passes.
    //
    // CURRENT BUG: raw count = 3 >= min_re = 3 → no hint emitted.
    // FIX: unique count = 2 < min_re = 3 → IggReattestationsInsufficient should fire.
    //
    // Also: alice appears twice → partial collapse → IggIdentityCollapseWarning
    // should also fire (currently only fires when unique.len() == 1, not for partial).
    assert!(
        !hints.is_empty(),
        "ATK-CE-3-B: [alice, alice, bob] with min_reattestations=3 must emit hints. \
         Unique sources = 2 < min_re = 3 means the reattestations requirement is NOT met \
         by unique independent sources. The raw-count check (3 >= 3) passes but is \
         MISLEADING — alice's double signature inflates the apparent count. \
         \nBUG: IggReattestationsInsufficient uses witnesses.len() instead of unique count. \
         \nFix: count unique witnesses and compare against min_reattestations. \
         \nGot hints: {:?}",
        hints
    );
}

#[test]
fn atk_ce3c_distinct_signers_no_collapse() {
    // Three distinct signers — no collapse.
    let decl = ConvergentEvidence {
        kind: ConvergentEvidenceKind::Igg,
        witnesses: vec![
            "alice@example.com".to_string(),
            "bob@corp.com".to_string(),
            "carol@org.net".to_string(),
        ],
        historical_span: Some(90),
        min_reattestations: Some(3),
        ..ce(ConvergentEvidenceKind::Igg)
    };
    let report = audit_convergent_evidence(&report_with(decl));

    let hints: Vec<_> = report.audits.iter().flat_map(|a| &a.hints).collect();

    assert!(
        !hints.contains(&&AuditHint::IggIdentityCollapseWarning),
        "ATK-CE-3-C: distinct signers must NOT emit IggIdentityCollapseWarning. \
         Got: {:?}",
        hints
    );
    assert!(
        !hints.contains(&&AuditHint::IggReattestationsInsufficient),
        "ATK-CE-3-C: 3 distinct witnesses >= min_reattestations=3 must not be \
         insufficient. Got: {:?}",
        hints
    );
}

// ============================================================================
// ATK-CE-4: polyclonal/monoclonal/adcc are pure documentation markers
//
// Per ADR-024 enforcement table: none of these are in the enforcement surface.
// v0.2 implementation: no audit hints emitted. Pure documentation.
//
// This test CONFIRMS the correct behavior — any hint would be a false positive.
// ============================================================================

#[test]
fn atk_ce4a_polyclonal_emits_no_audit_hints() {
    let decl = ce(ConvergentEvidenceKind::Polyclonal);
    let report = audit_convergent_evidence(&report_with(decl));

    let hints: Vec<_> = report.audits.iter().flat_map(|a| &a.hints).collect();

    assert!(
        hints.is_empty(),
        "ATK-CE-4-A: #[polyclonal] must emit NO audit hints at v0.2. \
         It is a pure documentation marker (not in ADR-024 enforcement table). \
         Any hint here is a false positive. Got: {:?}",
        hints
    );
}

#[test]
fn atk_ce4b_monoclonal_emits_no_audit_hints() {
    let decl = ce(ConvergentEvidenceKind::Monoclonal);
    let report = audit_convergent_evidence(&report_with(decl));

    let hints: Vec<_> = report.audits.iter().flat_map(|a| &a.hints).collect();

    assert!(
        hints.is_empty(),
        "ATK-CE-4-B: #[monoclonal] must emit NO audit hints at v0.2. Got: {:?}",
        hints
    );
}

#[test]
fn atk_ce4c_adcc_emits_no_audit_hints() {
    let decl = ce(ConvergentEvidenceKind::Adcc);
    let report = audit_convergent_evidence(&report_with(decl));

    let hints: Vec<_> = report.audits.iter().flat_map(|a| &a.hints).collect();

    assert!(
        hints.is_empty(),
        "ATK-CE-4-C: #[adcc] must emit NO audit hints at v0.2. Got: {:?}",
        hints
    );
}

// ============================================================================
// ATK-CE-1-C: Empty modalities in #[diagnostic] must emit specific hint
// ============================================================================

#[test]
fn atk_ce1c_empty_modalities_emits_diagnostic_modalities_empty() {
    let decl = ConvergentEvidence {
        kind: ConvergentEvidenceKind::Diagnostic,
        modality_classes: vec![], // empty
        min_independent: Some(1),
        ..ce(ConvergentEvidenceKind::Diagnostic)
    };
    let report = audit_convergent_evidence(&report_with(decl));

    let hints: Vec<_> = report.audits.iter().flat_map(|a| &a.hints).collect();

    assert!(
        hints.contains(&&AuditHint::DiagnosticModalitiesEmpty),
        "ATK-CE-1-C: empty modalities must emit DiagnosticModalitiesEmpty. \
         Got: {:?}",
        hints
    );
}
