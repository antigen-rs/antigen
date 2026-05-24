//! Adversarial tests for the Mucosal Boundary Family (ADR-027).
//!
//! All tests are #[ignore] until the mucosal family ships. When pathmaker
//! lands v02-impl-mucosal-boundary:
//!
//! 1. Remove #[ignore] from each test.
//! 2. Run `cargo test atk_mucosal_adversarial` — tests should FAIL.
//! 3. Fix the production code so tests PASS.
//! 4. These tests are now regression guards.
//!
//! Written by adversarial role as preemptive attack surface documentation.

use antigen::audit::{audit_mucosal, AuditHint};
use antigen::scan::{ItemTarget, MucosalDeclaration, MucosalKindTag, ScanReport};
use std::path::PathBuf;

fn mucosal_decl(
    tag: MucosalKindTag,
    boundary_kind: Option<&str>,
    rationale: Option<&str>,
) -> MucosalDeclaration {
    MucosalDeclaration {
        tag,
        boundary_kind: boundary_kind.map(str::to_owned),
        rationale: rationale.map(str::to_owned),
        handled_by: None,
        accepts: None,
        reviewed_by: None,
        until: None,
        file: PathBuf::from("test.rs"),
        line: 1,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Fn("t".to_string()),
    }
}

// ============================================================================
// ATK-MUCOSAL-1: handled_by empty string
//
// ADR-027 §Decision: #[mucosal_delegate(handled_by = "...")] must reference a
// real function. Empty string is not a valid path.
// Expected: audit emits `mucosal-discipline-delegate-target-missing`
// ============================================================================

#[test]
fn atk_mucosal_1_handled_by_empty_string_emits_missing_hint() {
    // handled_by = None models the missing/empty case — audit.rs evaluates
    // None as MucosalDisciplineDelegateTargetMissing (Tier 1).
    // An empty handled_by is captured by scan as None (no valid path segment).
    let mut report = ScanReport::default();
    let mut d = mucosal_decl(
        MucosalKindTag::MucosalDelegate,
        Some("UserInput"),
        Some("delegated to a handler function for the user input path"),
    );
    d.handled_by = None; // empty/missing handled_by → None after scan
    report.mucosal_declarations.push(d);
    let out = audit_mucosal(&report);
    assert!(
        out.audits[0]
            .hints
            .contains(&AuditHint::MucosalDisciplineDelegateTargetMissing),
        "expected MucosalDisciplineDelegateTargetMissing for handled_by = None"
    );
}

// ============================================================================
// ATK-MUCOSAL-2: handled_by with non-existent function path
//
// A syntactically-valid path string that doesn't resolve to any function
// in the codebase should emit the missing-target hint, not panic.
// ============================================================================

#[test]
fn atk_mucosal_2_handled_by_nonexistent_path_emits_missing_hint() {
    // handled_by = "nonexistent_fn" — function name not in any #[mucosal] declaration
    // in the ScanReport. handler_kinds index built from #[mucosal] declarations only;
    // "nonexistent_fn" not in index → Tier 1: MucosalDisciplineDelegateTargetMissing.
    let mut report = ScanReport::default();
    let mut d = mucosal_decl(
        MucosalKindTag::MucosalDelegate,
        Some("UserInput"),
        Some("delegated to handler that does not exist in this workspace"),
    );
    d.handled_by = Some("nonexistent_fn".to_string());
    // No #[mucosal] declarations in report → handler_kinds index is empty
    // → "nonexistent_fn" lookup returns None → MucosalDisciplineDelegateTargetMissing
    report.mucosal_declarations.push(d);
    let out = audit_mucosal(&report);
    assert!(
        out.audits[0]
            .hints
            .contains(&AuditHint::MucosalDisciplineDelegateTargetMissing),
        "expected MucosalDisciplineDelegateTargetMissing for unresolvable handled_by"
    );
}

// ============================================================================
// ATK-MUCOSAL-3: delegate target exists but has WRONG MucosalKind
//
// The delegate target carries #[mucosal(kind = MucosalKind::DatabaseQuery)]
// but the delegate says `boundary = MucosalKind::UserInput`. The ADR says
// the audit emits `mucosal-discipline-delegate-target-not-mucosal` if target
// lacks corresponding declaration — but it doesn't specify what "corresponding"
// means. Does it check for the SAME kind, or just ANY #[mucosal]?
//
// ADVERSARIAL FINDING: This is the handled_by resolution spec gap flagged by
// observer (campsite v02-impl-mucosal-boundary, #implementation-spec-depth-gap).
// If "any #[mucosal]" is sufficient, a defender can satisfy the audit check by
// pointing to a mucosal handler for the WRONG boundary type — a silent false
// positive in the defense attestation.
// ============================================================================

#[test]
fn atk_mucosal_3_delegate_target_with_wrong_kind_should_not_pass_audit() {
    // handler `sanitize` carries #[mucosal(kind = DatabaseQuery)] — correct for
    // SQL injection defense but NOT UserInput. The delegate claims boundary =
    // UserInput and handled_by = "sanitize". Set-membership check must use the
    // delegate's boundary_kind against handler's kind-set — "UserInput" NOT in
    // {"DatabaseQuery"} → Tier 3: MucosalDisciplineDelegateTargetKindMismatch.
    let mut report = ScanReport::default();

    // The handler function `sanitize` with DatabaseQuery mucosal declaration.
    let mut handler = mucosal_decl(
        MucosalKindTag::Mucosal,
        Some("DatabaseQuery"),
        Some("escapes SQL injection via parameterized query construction"),
    );
    handler.item_target = ItemTarget::Fn("sanitize".to_string());
    report.mucosal_declarations.push(handler);

    // The delegate pointing to `sanitize` but claiming UserInput boundary.
    let mut delegate = mucosal_decl(
        MucosalKindTag::MucosalDelegate,
        Some("UserInput"),
        Some("UserInput defense delegated to sanitize fn for unified handling"),
    );
    delegate.handled_by = Some("sanitize".to_string());
    delegate.item_target = ItemTarget::Fn("process_user_input".to_string());
    report.mucosal_declarations.push(delegate);

    let out = audit_mucosal(&report);
    // The delegate audit (second entry) should flag kind mismatch.
    let delegate_hints = &out.audits[1].hints;
    assert!(
        delegate_hints.contains(&AuditHint::MucosalDisciplineDelegateTargetKindMismatch),
        "expected MucosalDisciplineDelegateTargetKindMismatch: 'sanitize' has \
         DatabaseQuery kinds only, delegate claims UserInput boundary"
    );
    // The handler itself (first entry) must be clean — it's a valid mucosal decl.
    assert!(out.audits[0].hints.is_empty(), "handler should be clean");
}

// ============================================================================
// ATK-MUCOSAL-4: duplicate #[mucosal] on the same function
//
// What happens if a function carries two #[mucosal] declarations with the same
// kind? This is an antipattern (mucosal-kind-mismatch or duplicate declaration).
// Should it warn? Hard error? Silent dedup?
// ============================================================================

#[test]
fn atk_mucosal_4_duplicate_mucosal_declarations_on_same_function() {
    // Two #[mucosal(kind = UserInput)] on the same function. The audit builds
    // handler_kinds as a HashMap<fn_name, HashSet<kind>>. Duplicate kinds are
    // deduplicated by the HashSet — two UserInput entries collapse to one.
    // DESIGN DECISION: silent dedup is correct here. Hybrid handlers (one fn
    // handling multiple boundary kinds) are a valid and intended use case. The
    // issue ATK-4 surfaces — false-extra-coverage by duplication — is real but
    // is enforced at the PARSE layer (#[mucosal] duplicate enforcement, not
    // audit layer). This test verifies that audit doesn't PANIC and that the
    // duplicate kind counts as exactly one coverage entry.
    let mut report = ScanReport::default();
    let rationale = "protects the user input path against injection attacks";
    let mut d1 = mucosal_decl(MucosalKindTag::Mucosal, Some("UserInput"), Some(rationale));
    d1.item_target = ItemTarget::Fn("sanitize".to_string());
    let mut d2 = mucosal_decl(MucosalKindTag::Mucosal, Some("UserInput"), Some(rationale));
    d2.item_target = ItemTarget::Fn("sanitize".to_string());
    report.mucosal_declarations.push(d1);
    report.mucosal_declarations.push(d2);

    // Must not panic; both declarations should be clean (valid rationale/kind).
    let out = audit_mucosal(&report);
    assert_eq!(
        out.audits.len(),
        2,
        "both declarations should produce audit entries"
    );
    for audit in &out.audits {
        assert!(
            audit.hints.is_empty(),
            "duplicate valid declarations should each be clean (dedup handled by parse layer)"
        );
    }
}

// ============================================================================
// ATK-MUCOSAL-5: #[mucosal] with rationale below minimum length
//
// ADR-027 mentions `rationale = "..."` as a required field. Is there a
// minimum length enforced (like ADR-023 uses 20-char minimum for learning_path)?
// If not, `rationale = "x"` or `rationale = ""` could satisfy the audit check
// with meaningless content.
// ============================================================================

#[test]
fn atk_mucosal_5_rationale_too_short_should_emit_hint() {
    // rationale = "x" is 1 char — below the ≥20-char floor for #[mucosal].
    let mut report = ScanReport::default();
    report.mucosal_declarations.push(mucosal_decl(
        MucosalKindTag::Mucosal,
        Some("UserInput"),
        Some("x"),
    ));
    let out = audit_mucosal(&report);
    assert!(
        out.audits[0]
            .hints
            .contains(&AuditHint::MucosalRationaleInsufficient),
        "expected MucosalRationaleInsufficient for rationale = 'x' (1 char < 20-char floor)"
    );
}

// ============================================================================
// ATK-MUCOSAL-6: mucosal-map --undefended false negative
//
// The most dangerous silent failure: a boundary that exists in code but is
// NOT detected by mucosal-map as a boundary at all. The scanner misses it,
// so --undefended doesn't surface it, so it has no obligation to declare.
//
// Example: an HTTP handler that uses a custom request type (not a standard
// actix_web::web::Path or axum::extract::Json) — the boundary-detection
// heuristic might not recognize it as a boundary.
//
// This is structural to static-analysis-based boundary detection (can't be
// fixed completely) but should be documented as the residual risk.
// ============================================================================

#[test]
#[ignore = "mucosal family not yet implemented — remove ignore when v02-impl-mucosal-boundary ships"]
fn atk_mucosal_6_custom_input_type_not_detected_as_boundary_is_expected_gap() {
    // Document the false-negative residual risk explicitly.
    // The test should verify that:
    // 1. The scan correctly reports "0 detected boundaries" for a custom type
    // 2. The audit hint surface documents that scan is best-effort
    // 3. The --undefended flag's output includes a residual-risk note
    todo!("implement when mucosal module ships")
}
