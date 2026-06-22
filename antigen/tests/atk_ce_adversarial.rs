//! ATK-CE adversarial test suite — Convergent-Evidence Family (ADR-024).
//!
//! STATUS: PENDING IMPLEMENTATION
//! Tests are commented out and will be enabled when the
//! convergent-evidence family ships. Each comment block describes:
//!   - The attack vector
//!   - What the implementation SHOULD do (spec-derived)
//!   - What a silent failure looks like (wrong but plausible answer)

// ============================================================================
// ATK-CE-1: Class-collapse in #[diagnostic]
// ============================================================================
//
// ADR-024 C1: min_independent = distinct WitnessClass CATEGORIES, not count.
// Attack: provide [StaticAnalysis, StaticAnalysis] with min_independent = 2.
// Expected hint: diagnostic-modalities-class-collapsed.
// Silent failure: implementation counts WITNESSES (gets 2) instead of
// DISTINCT CLASSES (gets 1), and silently accepts the invalid claim.

// #[test]
// fn atk_ce1_same_class_twice_collapses_to_one_independent() {
//     use antigen::audit::{AuditHint, audit_convergent_evidence};
//     use antigen::scan::scan_workspace;
//     use std::path::Path;
//
//     let fx = Path::new(env!("CARGO_MANIFEST_DIR"))
//         .join("tests/fixtures/atk_ce_class_collapse");
//     let scan = scan_workspace(&fx, None).expect("scan completes");
//     let report = audit_convergent_evidence(&scan, &fx);
//
//     // Case A: [StaticAnalysis, StaticAnalysis] with min_independent = 2
//     let same_class_audit = report.audits.iter().find(|a| {
//         a.function_name.as_deref() == Some("atk_ce1_same_class_twice")
//     }).expect("audit for atk_ce1_same_class_twice must exist");
//
//     assert_eq!(
//         same_class_audit.hint,
//         AuditHint::DiagnosticModalitiesClassCollapsed,
//         "ATK-CE-1-A: [StaticAnalysis, StaticAnalysis] with min_independent=2 \
//          MUST emit diagnostic-modalities-class-collapsed. \
//          Silent pass = bypass of the independence requirement. \
//          Got: {:?}", same_class_audit.hint
//     );
//
//     // Case B: [PropertyTest, PropertyTest, PropertyTest] with min_independent = 3
//     let triple_same_audit = report.audits.iter().find(|a| {
//         a.function_name.as_deref() == Some("atk_ce1_three_same_class")
//     }).expect("audit for atk_ce1_three_same_class must exist");
//
//     assert_eq!(
//         triple_same_audit.hint,
//         AuditHint::DiagnosticModalitiesClassCollapsed,
//         "ATK-CE-1-B: three witnesses same class must still collapse. Got: {:?}",
//         triple_same_audit.hint
//     );
//
//     // Case C: [StaticAnalysis, PropertyTest] — CORRECT, should not collapse
//     let correct_audit = report.audits.iter().find(|a| {
//         a.function_name.as_deref() == Some("atk_ce1_correct_two_classes")
//     }).expect("audit for atk_ce1_correct_two_classes must exist");
//
//     assert_ne!(
//         correct_audit.hint,
//         AuditHint::DiagnosticModalitiesClassCollapsed,
//         "ATK-CE-1-C: [StaticAnalysis, PropertyTest] must NOT collapse. Got: {:?}",
//         correct_audit.hint
//     );
// }

// ============================================================================
// ATK-CE-2: Fixed-seed bypass — parse-time vs audit-time gap
// ============================================================================
//
// ADR-024 C2: SeedKind::Fixed(u64) MUST be rejected for #[clonal].
// ADR says "parse-time OR audit-time"; the brief requires parse-time.
//
// The compile-fail test lives in antigen-macros/tests/ui/clonal_fixed_seed_compile_error.rs
// That test exercises the parse-time path.
//
// This file tests the AUDIT-TIME fallback:
// If a user somehow gets a ClonalDeclaration with Fixed seed into a scan report
// (e.g., from pre-compiled code or a scan of text without compilation),
// does audit_convergent_evidence emit clonal-fixed-seed-detected?

// #[test]
// fn atk_ce2_fixed_seed_in_scan_report_produces_audit_hint() {
//     use antigen::audit::{AuditHint, audit_convergent_evidence};
//     use antigen::scan::{ScanReport, ConvergentEvidence, ClonalDeclaration};
//     use antigen::SeedKind;
//
//     // Synthesize a scan report with a Fixed-seed clonal declaration.
//     // This simulates the case where compile-time enforcement was bypassed.
//     let clonal = ClonalDeclaration {
//         witness: "my_test".to_string(),
//         iterations: 1000,
//         seed: SeedKind::Fixed(42),
//         file: std::path::PathBuf::from("src/lib.rs"),
//         line: 10,
//     };
//
//     let mut scan = ScanReport::default();
//     scan.convergent_evidence.push(ConvergentEvidence::Clonal(clonal));
//
//     let report = audit_convergent_evidence(&scan, std::path::Path::new("."));
//
//     let fixed_seed_audit = report.audits.iter().find(|a| {
//         matches!(&a.declaration, ConvergentEvidence::Clonal(_))
//     }).expect("audit for clonal declaration must exist");
//
//     assert_eq!(
//         fixed_seed_audit.hint,
//         AuditHint::ClonalFixedSeedDetected,
//         "ATK-CE-2: Fixed seed in scan report must emit clonal-fixed-seed-detected \
//          even if compile-time check was bypassed. Got: {:?}", fixed_seed_audit.hint
//     );
// }

// ============================================================================
// ATK-CE-3: IgG identity-collapse
// ============================================================================
//
// ADR-024 C3 + §Enforcement-Surface: source-independence is NOMINAL.
// Duplicate identity strings = obvious collapse. Audit MUST emit warning.
// This is a partial defense (structural verification impossible at v0.2)
// but the DETECTABLE case must be detected.

// #[test]
// fn atk_ce3_duplicate_signer_emits_identity_collapse_warning() {
//     use antigen::audit::{AuditHint, audit_convergent_evidence};
//     use antigen::scan::scan_workspace;
//     use std::path::Path;
//
//     let fx = Path::new(env!("CARGO_MANIFEST_DIR"))
//         .join("tests/fixtures/atk_ce_identity_collapse");
//     let scan = scan_workspace(&fx, None).expect("scan completes");
//     let report = audit_convergent_evidence(&scan, &fx);
//
//     // Case A: same email twice
//     let duplicate_audit = report.audits.iter().find(|a| {
//         a.function_name.as_deref() == Some("atk_ce3_duplicate_signer")
//     }).expect("audit for atk_ce3_duplicate_signer must exist");
//
//     assert_eq!(
//         duplicate_audit.hint,
//         AuditHint::IggIdentityCollapseWarning,
//         "ATK-CE-3-A: duplicate signer identity must emit igg-identity-collapse-warning. \
//          Got: {:?}", duplicate_audit.hint
//     );
//
//     // Case B: triple same identity
//     let triple_audit = report.audits.iter().find(|a| {
//         a.function_name.as_deref() == Some("atk_ce3_triple_same_identity")
//     }).expect("audit for atk_ce3_triple_same_identity must exist");
//
//     assert_eq!(
//         triple_audit.hint,
//         AuditHint::IggIdentityCollapseWarning,
//         "ATK-CE-3-B: triple same identity must also emit warning. Got: {:?}",
//         triple_audit.hint
//     );
//
//     // Case C: distinct identities — no warning
//     let distinct_audit = report.audits.iter().find(|a| {
//         a.function_name.as_deref() == Some("atk_ce3_distinct_signers")
//     }).expect("audit for atk_ce3_distinct_signers must exist");
//
//     assert_ne!(
//         distinct_audit.hint,
//         AuditHint::IggIdentityCollapseWarning,
//         "ATK-CE-3-C: distinct identities must NOT emit collapse warning. Got: {:?}",
//         distinct_audit.hint
//     );
// }

// ============================================================================
// ATK-CE-4: polyclonal/monoclonal/adcc — pure documentation vs enforceable
// ============================================================================
//
// Per ADR-024 enforcement table: polyclonal/monoclonal/adcc are NOT listed.
// Conclusion: they are pure documentation markers at v0.2.
// This is a NAMED-LIMITATION, not a bug.
//
// However: the audit hint vocabulary includes `polyclonal-insufficient-lineages`.
// If the audit emits this hint WITHOUT any threshold being defined,
// that's a false positive. If it never emits it, that's expected (pure doc).
//
// This test documents which behavior ships.

// #[test]
// fn atk_ce4_polyclonal_monoclonal_adcc_are_pure_documentation_markers() {
//     use antigen::audit::{AuditHint, audit_convergent_evidence};
//     use antigen::scan::scan_workspace;
//     use std::path::Path;
//
//     let fx = Path::new(env!("CARGO_MANIFEST_DIR"))
//         .join("tests/fixtures/atk_ce_polyclonal_empty");
//     let scan = scan_workspace(&fx, None).expect("scan completes");
//     let report = audit_convergent_evidence(&scan, &fx);
//
//     // Neither polyclonal nor monoclonal nor adcc should emit false-positive hints.
//     let problematic_hints: Vec<_> = report.audits.iter().filter(|a| {
//         matches!(
//             a.hint,
//             AuditHint::PolyclonalInsufficientLineages
//             | AuditHint::AdccSingleMechanismOnly
//         )
//     }).collect();
//
//     assert!(
//         problematic_hints.is_empty(),
//         "ATK-CE-4: polyclonal/monoclonal/adcc have no enforceable criteria at v0.2. \
//          Emitting polyclonal-insufficient-lineages or adcc-single-mechanism-only \
//          without threshold configuration is a false positive. \
//          Hits: {:?}", problematic_hints
//     );
//
//     // Document finding: if the audit produces NO hints at all for these markers,
//     // they are confirmed pure-documentation. That's CORRECT behavior per ADR-024
//     // enforcement table (they're absent from the table).
//     eprintln!(
//         "ATK-CE-4 CONFIRMED: polyclonal/monoclonal/adcc produce {} audit entries. \
//          Expected: none or only documentation-level entries.",
//         report.audits.len()
//     );
// }

// ============================================================================
// ATK-CE-5: zero min_independent / zero min_reattestations — silent pass.
//
// The threshold comparisons use `distinct.len() < min` (usize < u64). When
// min = 0, the comparison is `N < 0` which is always false for an unsigned
// type — silent pass regardless of how many distinct classes exist (even 0).
//
// A developer who clears a min_independent value to 0 (perhaps by copy-paste
// or a config error) gets a silent acceptance: the audit does not fire
// DiagnosticModalityInsufficient or IggReattestationsInsufficient, even
// though the threshold is semantically null (zero independence = no claim).
//
// Correct behavior: when min_independent = 0 or min_reattestations = 0, the
// audit should surface a dedicated hint (e.g., DiagnosticMinIndependentZero)
// or at minimum refuse to count it as a meaningful threshold (treat same as
// None). Silent acceptance of zero lets intentionally-misconfigured
// declarations look clean.
//
// These tests assert the BROKEN outcome — they pass today (no hint fires).
// Will need updating if a zero-min warning hint is added to the audit.
// ============================================================================

use std::path::PathBuf;

use antigen::audit::{AuditHint, audit_convergent_evidence};
use antigen::scan::{ConvergentEvidence, ConvergentEvidenceKind, ItemTarget, ScanReport};

fn ce_base(kind: ConvergentEvidenceKind) -> ConvergentEvidence {
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
        item_target: ItemTarget::Fn("some_test".to_string()),
    }
}

#[test]
fn atk_ce5_diagnostic_zero_min_independent_warns() {
    // ATK-CE-5-A (FIXED): #[diagnostic] with min_independent = 0 must emit
    // DiagnosticMinIndependentZero — a zero threshold is semantically null
    // (zero independent classes required = no claim) and must not silently
    // accept the declaration as valid.
    //
    // Previously the comparison `u64::try_from(distinct.len()).unwrap_or(u64::MAX) < min`
    // evaluated as `N < 0` (always false for unsigned) when min = 0 — no hint fired.
    // Fix: check `min == 0` first and emit DiagnosticMinIndependentZero.
    let mut decl = ce_base(ConvergentEvidenceKind::Diagnostic);
    decl.modality_classes = vec!["StaticAnalysis".to_string()]; // 1 distinct class
    decl.min_independent = Some(0); // zero threshold — semantically null

    let mut report = ScanReport::default();
    report.convergent_evidences.push(decl);

    let out = audit_convergent_evidence(&report);
    assert_eq!(out.audits.len(), 1);
    let hints = &out.audits[0].hints;

    assert!(
        hints.contains(&AuditHint::DiagnosticMinIndependentZero),
        "ATK-CE-5-A: #[diagnostic] with min_independent=0 must emit \
         DiagnosticMinIndependentZero. A zero threshold is semantically null; \
         silent acceptance allows authors to write a threshold that never enforces \
         independence discipline. hints: {:?}",
        hints
    );
    // DiagnosticModalityInsufficient must NOT fire — zero is not a real threshold
    // that the actual distinct count can fail to meet; it's a misconfiguration.
    assert!(
        !hints.contains(&AuditHint::DiagnosticModalityInsufficient),
        "ATK-CE-5-A: DiagnosticModalityInsufficient must not fire for zero min — \
         it's a misconfiguration hint, not an insufficient-count hint. hints: {:?}",
        hints
    );
}

#[test]
fn atk_ce5_igg_zero_min_reattestations_warns() {
    // ATK-CE-5-B (FIXED): #[igg] with min_reattestations = 0 must emit
    // IggMinReattestationsZero — zero re-attestations required = no reattestation
    // discipline. The audit must surface this rather than accepting silently.
    //
    // Previously `unique_count.len() < 0` was always false for unsigned — no hint fired.
    let mut decl = ce_base(ConvergentEvidenceKind::Igg);
    decl.witnesses = vec!["alice".to_string(), "bob".to_string()]; // 2 distinct
    decl.min_reattestations = Some(0); // zero threshold

    let mut report = ScanReport::default();
    report.convergent_evidences.push(decl);

    let out = audit_convergent_evidence(&report);
    assert_eq!(out.audits.len(), 1);
    let hints = &out.audits[0].hints;

    assert!(
        hints.contains(&AuditHint::IggMinReattestationsZero),
        "ATK-CE-5-B: #[igg] with min_reattestations=0 must emit IggMinReattestationsZero. \
         A zero reattestations threshold is semantically null — silent acceptance allows \
         authors to write reattestation discipline that never enforces itself. hints: {:?}",
        hints
    );
    assert!(
        !hints.contains(&AuditHint::IggReattestationsInsufficient),
        "ATK-CE-5-B: IggReattestationsInsufficient must not fire for zero min — \
         it's a misconfiguration, not an insufficient-count condition. hints: {:?}",
        hints
    );
}

#[test]
fn atk_ce5_diagnostic_zero_min_with_no_modalities_silently_passes() {
    // Degenerate combo: min_independent = 0 AND modality_classes empty.
    // The DiagnosticModalitiesEmpty hint fires first (early return), so this
    // particular case DOES produce a hint. Verifying it's not silently clean.
    let mut decl = ce_base(ConvergentEvidenceKind::Diagnostic);
    decl.modality_classes = Vec::new(); // empty
    decl.min_independent = Some(0);

    let mut report = ScanReport::default();
    report.convergent_evidences.push(decl);

    let out = audit_convergent_evidence(&report);
    assert_eq!(out.audits.len(), 1);
    let hints = &out.audits[0].hints;

    // The early-return for empty modalities fires BEFORE the zero-min check.
    // This case IS caught (DiagnosticModalitiesEmpty), not a silent failure.
    // This test documents that the empty-modalities guard fires first.
    assert!(
        hints.contains(&AuditHint::DiagnosticModalitiesEmpty),
        "ATK-CE-5-C: empty modality_classes fires DiagnosticModalitiesEmpty \
         before the zero-min check; this case IS caught. \
         hints: {:?}",
        hints
    );
}
