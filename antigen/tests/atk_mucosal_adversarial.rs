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

// When the module exists, add:
// use antigen::mucosal::{MucosalKind, MucosalDeclaration};
// use antigen::scan::{ScanReport, scan_workspace};

// ============================================================================
// ATK-MUCOSAL-1: handled_by empty string
//
// ADR-027 §Decision: #[mucosal_delegate(handled_by = "...")] must reference a
// real function. Empty string is not a valid path.
// Expected: audit emits `mucosal-discipline-delegate-target-missing`
// ============================================================================

#[test]
#[ignore = "mucosal family not yet implemented — remove ignore when v02-impl-mucosal-boundary ships"]
fn atk_mucosal_1_handled_by_empty_string_emits_missing_hint() {
    // #[mucosal_delegate(boundary = MucosalKind::UserInput, handled_by = "", rationale = "test")]
    // Should emit `mucosal-discipline-delegate-target-missing` at audit time.
    // An empty handled_by is unresolvable — equivalent to a missing target.
    todo!("implement when mucosal module ships")
}

// ============================================================================
// ATK-MUCOSAL-2: handled_by with non-existent function path
//
// A syntactically-valid path string that doesn't resolve to any function
// in the codebase should emit the missing-target hint, not panic.
// ============================================================================

#[test]
#[ignore = "mucosal family not yet implemented — remove ignore when v02-impl-mucosal-boundary ships"]
fn atk_mucosal_2_handled_by_nonexistent_path_emits_missing_hint() {
    // #[mucosal_delegate(boundary = MucosalKind::UserInput,
    //     handled_by = "does::not::exist::sanitize_input",
    //     rationale = "test")]
    // Should emit `mucosal-discipline-delegate-target-missing`
    // NOT a panic or silent success.
    todo!("implement when mucosal module ships")
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
#[ignore = "mucosal family not yet implemented — remove ignore when v02-impl-mucosal-boundary ships"]
fn atk_mucosal_3_delegate_target_with_wrong_kind_should_not_pass_audit() {
    // Setup:
    //   fn process_user_input(input: &str) { sanitize(input); }
    //   #[mucosal(kind = MucosalKind::DatabaseQuery, rationale = "protects DB")]
    //   fn sanitize(input: &str) { /* sql escaping */ }
    //   #[mucosal_delegate(boundary = MucosalKind::UserInput,
    //                       handled_by = "sanitize",
    //                       rationale = "delegated to sanitize")]
    //   fn process_user_input_outer(input: &str) { process_user_input(input); }
    //
    // The delegate target `sanitize` carries #[mucosal] for DatabaseQuery,
    // NOT UserInput. The audit should flag this as wrong-kind mismatch.
    // If it doesn't, a developer has silently miscategorized the defense.
    todo!("implement when mucosal module ships")
}

// ============================================================================
// ATK-MUCOSAL-4: duplicate #[mucosal] on the same function
//
// What happens if a function carries two #[mucosal] declarations with the same
// kind? This is an antipattern (mucosal-kind-mismatch or duplicate declaration).
// Should it warn? Hard error? Silent dedup?
// ============================================================================

#[test]
#[ignore = "mucosal family not yet implemented — remove ignore when v02-impl-mucosal-boundary ships"]
fn atk_mucosal_4_duplicate_mucosal_declarations_on_same_function() {
    // #[mucosal(kind = MucosalKind::UserInput, rationale = "r1")]
    // #[mucosal(kind = MucosalKind::UserInput, rationale = "r2")]
    // fn sanitize(input: &str) { ... }
    //
    // Two declarations for the same kind on one function. Should the second
    // be silently accepted? Should it warn? The ADR doesn't specify.
    // The silent-accept case is a false-positive defense: two declarations
    // look more defended but add no actual coverage.
    todo!("implement when mucosal module ships")
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
#[ignore = "mucosal family not yet implemented — remove ignore when v02-impl-mucosal-boundary ships"]
fn atk_mucosal_5_rationale_too_short_should_emit_hint() {
    // #[mucosal(kind = MucosalKind::UserInput, rationale = "x")]
    // The one-character rationale should trigger `mucosal-rationale-insufficient`.
    // If it doesn't, the rationale field is a free-pass that adds no information.
    todo!("implement when mucosal module ships")
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
