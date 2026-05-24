//! Adversarial tests for the Recurrent Emergence Family (ADR-024).
//!
//! All tests are #[ignore] until the recurrent family ships. When pathmaker
//! lands v02-impl-recurrent-emergence:
//!
//! 1. Remove #[ignore] from each test.
//! 2. Run `cargo test atk_recurrent_adversarial` — tests should FAIL.
//! 3. Fix the production code so tests PASS.
//! 4. These tests are now regression guards.
//!
//! Written by adversarial role as preemptive attack surface documentation.
//! Campsite: v02-impl-recurrent-emergence

use antigen::audit::{audit_recurrent, AuditHint};
use antigen::scan::{ItemTarget, RecurrentDeclaration, RecurrentKind, ScanReport};
use std::path::PathBuf;

fn base_decl(kind: RecurrentKind, antigen_type: Option<&str>) -> RecurrentDeclaration {
    RecurrentDeclaration {
        kind,
        name: None,
        antigen_type: antigen_type.map(str::to_owned),
        description: None,
        instances: None,
        since: None,
        rationale: None,
        from_itches: Vec::new(),
        anchored_by: Vec::new(),
        managed_by: None,
        contributing_to: None,
        file: PathBuf::from("test.rs"),
        line: 1,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Fn("t".to_string()),
    }
}

fn chronic_decl(since: Option<&str>) -> RecurrentDeclaration {
    RecurrentDeclaration {
        managed_by: Some("team".to_string()),
        since: since.map(str::to_owned),
        ..base_decl(RecurrentKind::Chronic, Some("SomeSignal"))
    }
}

// ============================================================================
// ATK-RECURRENT-1: #[itch] threshold = 0 is a silent no-op
//
// ADR-024 §Mechanics: #[itch(threshold = N)] notates a recurrence-awareness
// threshold. A threshold of 0 means "never fires" — the antigen is declared
// but structurally can never trigger the itch-noticed-not-anchored hint.
//
// ATTACK: threshold = 0 should be rejected at PROC-MACRO PARSE TIME (compile
// error). It is a structural impossibility — the declaration claims to track
// recurrence but can never accumulate a count that triggers action. This is
// self-contained within the single attribute (no cross-reference needed), so
// compile-time rejection is correct and has zero false-positive risk.
//
// DESIGN DECISION (adversarial 2026-05-24): proc-macro compile error, NOT
// audit-time hint. Audit-time is too late — green compile on broken discipline
// is the silent-failure class this test exists to catch. Same class as
// rejecting negative rollback_due_within_minutes: value makes declaration
// structurally meaningless.
//
// Expected: proc-macro compile error with message indicating threshold=0
// makes the itch unable to trigger.
// ============================================================================

#[test]
#[ignore = "recurrent family not yet implemented — remove ignore when v02-impl-recurrent-emergence ships; proc-macro must reject threshold=0 at compile time"]
fn atk_recurrent_1_itch_threshold_zero_is_compile_error() {
    // #[itch(threshold = 0, description = "pattern noticed")]
    // Must produce a proc-macro compile error — threshold=0 means the itch
    // can never be noticed, making the declaration structurally meaningless.
    // Use trybuild or compile_fail to assert the error fires at compile time.
    todo!("implement when recurrent family ships; assert proc-macro rejects threshold=0 via trybuild compile_fail test");
}

// ============================================================================
// ATK-RECURRENT-2: #[recurrence_anchor] without any matching #[itch]
//
// ADR-024 §Mechanics: #[recurrence_anchor] is the structural commitment after
// threshold is reached. But if no #[itch] in the codebase references the same
// pattern, the anchor is floating — declared commitment with no pre-condition.
//
// ATTACK: a floating recurrence_anchor (no corresponding itch declarations in
// the codebase) should emit `recurrence-anchor-no-itch-precondition` at audit
// time. Without this check, engineers can declare "we've anchored our response"
// without ever having declared "we noticed the pattern" — the temporal
// progression (itch -> anchor -> crystallize) is bypassed.
//
// Expected: audit emits `recurrence-anchor-no-itch-precondition`
// ============================================================================

#[test]
#[ignore = "recurrent family not yet implemented — remove ignore when v02-impl-recurrent-emergence ships"]
fn atk_recurrent_2_recurrence_anchor_without_matching_itch_emits_hint() {
    // Workspace has #[recurrence_anchor(pattern = "X", ...)] but zero #[itch]
    // declarations. The temporal progression is short-circuited.
    // Should emit recurrence-anchor-no-itch-precondition.
    todo!("implement when recurrent family ships; verify orphan anchor is flagged");
}

// ============================================================================
// ATK-RECURRENT-3: #[crystallize] referencing non-existent antigen name
//
// ADR-024 §Mechanics: #[crystallize(into = "AntigenName")] promotes a recurrent
// pattern to a named antigen. The `into` value must resolve to an existing
// #[antigen] declaration in the codebase.
//
// ATTACK: #[crystallize(into = "NonExistentAntigen")] should emit
// `crystallize-without-antigen` at audit time (already in ADR-024 §5471 hint
// vocabulary). Without resolution checking, crystallize looks like complete
// discipline but the promoted antigen doesn't actually exist — the pattern
// was "crystallized" into nothing.
//
// Expected: audit emits `crystallize-without-antigen`
// ============================================================================

#[test]
fn atk_recurrent_3_crystallize_into_nonexistent_antigen_emits_hint() {
    // #[crystallize(into = "NonExistentAntigenName", pattern = "foo")]
    // The named antigen doesn't exist anywhere in the workspace.
    // audit.rs Crystallize arm: if antigen_type is None AND from_itches is empty → CrystallizeWithoutAntigen.
    // This models the case where crystallize has no backing antigen reference.
    let mut report = ScanReport::default();
    report
        .recurrent_declarations
        .push(base_decl(RecurrentKind::Crystallize, None));
    let out = audit_recurrent(&report);
    assert!(
        out.audits[0]
            .hints
            .contains(&AuditHint::CrystallizeWithoutAntigen),
        "expected CrystallizeWithoutAntigen for crystallize with no antigen_type"
    );
    assert_eq!(out.concern_count, 1);
}

// ============================================================================
// ATK-RECURRENT-4: #[chronic] review_date accepted as arbitrary string
//
// ADR-024 §Mechanics: #[chronic(signal = "...", review_date = "YYYY-MM-DD")]
// declares a low-level persistent signal with a review deadline. The review_date
// field must be a parseable date; a past date should emit
// `chronic-signal-past-review-date`.
//
// ATTACK (two sub-cases):
//   (a) review_date = "not-a-date" — unambiguous non-date string.
//       Expected: audit emits `chronic-since-not-a-date` hint.
//       NOTE: version-tag-shaped strings like "v0.2.0" are TOLERATED silently
//       (informal use; no hint). Only unambiguous garbage triggers the hint.
//       DESIGN DECISION (adversarial 2026-05-24): audit-time hint, NOT
//       parse-time error. The scan layer is recall-tuned (ADR-010); the audit
//       layer applies the two-path logic: ISO-8601 parseable → enforce;
//       version-tag-shaped (v\d+\.\d+.*) → tolerate; everything else → hint.
//   (b) review_date = "2020-01-01" — past date, valid format.
//       Expected: audit emits `chronic-signal-past-review-date` (pre-authorized
//       per aristotle F1 on v02-impl-recurrent-emergence campsite).
//
// Both sub-cases are silent failures if review_date is stored as an opaque
// string without validation. An engineer writes `review_date = "some day"` and
// the chronic signal is declared as having a review deadline that is never
// checked — the discipline appears present but is unenforceable.
//
// ============================================================================

#[test]
fn atk_recurrent_4a_chronic_review_date_non_date_string_emits_hint() {
    // "not-a-date" fails ISO-8601 parse AND does not match version-tag pattern
    // (v\d+\.\d+.*), so it falls into the hint-emitting path.
    let mut report_bad = ScanReport::default();
    report_bad
        .recurrent_declarations
        .push(chronic_decl(Some("not-a-date")));
    let out_bad = audit_recurrent(&report_bad);
    assert!(
        out_bad.audits[0]
            .hints
            .contains(&AuditHint::ChronicSinceNotADate),
        "expected ChronicSinceNotADate for since = 'not-a-date'"
    );

    // Version-tag-shaped strings are TOLERATED silently — no hint.
    let mut report_vtag = ScanReport::default();
    report_vtag
        .recurrent_declarations
        .push(chronic_decl(Some("v0.2.0")));
    let out_vtag = audit_recurrent(&report_vtag);
    assert!(
        !out_vtag.audits[0]
            .hints
            .contains(&AuditHint::ChronicSinceNotADate),
        "v0.2.0 is a version-tag; must NOT emit ChronicSinceNotADate"
    );
}

#[test]
fn atk_recurrent_4b_chronic_past_review_date_emits_hint() {
    // "2020-01-01" is a valid ISO-8601 date far in the past (> CHRONIC_REVIEW_HORIZON_DAYS).
    let mut report = ScanReport::default();
    report
        .recurrent_declarations
        .push(chronic_decl(Some("2020-01-01")));
    let out = audit_recurrent(&report);
    assert!(
        out.audits[0]
            .hints
            .contains(&AuditHint::ChronicSignalPastReviewDate),
        "expected ChronicSignalPastReviewDate for since = '2020-01-01'"
    );
}

// ============================================================================
// ATK-RECURRENT-5: #[saturate] anchor field references non-existent anchor
//
// ADR-024 §Mechanics: #[saturate(anchor = "AnchorName", ...)] declares that
// the recurrence pattern has reached saturation threshold — tied to a named
// recurrence_anchor. The anchor field must resolve to an existing
// #[recurrence_anchor(pattern = "AnchorName")] in the codebase.
//
// ATTACK: #[saturate(anchor = "NonExistentAnchor")] should emit
// `saturate-no-anchor` at audit time (pre-authorized per aristotle F1).
// Without resolution checking, saturate looks like threshold-exceeded discipline
// but the anchor it references doesn't exist — the recurrence lifecycle is
// broken at the saturation step.
//
// Expected: audit emits `saturate-no-anchor`
// ============================================================================

#[test]
fn atk_recurrent_5_saturate_anchor_nonexistent_emits_hint() {
    // #[saturate(anchor = "NonExistentPattern")] with contributing_to = None.
    // audit.rs Saturate arm: if contributing_to is None → SaturateNoAnchor.
    let mut report = ScanReport::default();
    report
        .recurrent_declarations
        .push(base_decl(RecurrentKind::Saturate, None));
    let out = audit_recurrent(&report);
    assert!(
        out.audits[0].hints.contains(&AuditHint::SaturateNoAnchor),
        "expected SaturateNoAnchor for saturate with no contributing_to"
    );
    assert_eq!(out.concern_count, 1);
}

// ============================================================================
// ATK-RECURRENT-6: #[strand] category mismatch — recurrent declared with
//                  wrong antigen-category
//
// ADR-024 §Antigen-category: Recurrent = mostly SubstrateAlignment.
// ADR-028 §Decision: category is enforced at audit time; SubstrateAlignment
// antigens require substrate-witness leaves, not code-witness leaves.
//
// ATTACK: an #[antigen] declaration for a Recurrent family antigen that
// specifies category = FunctionalCorrectness should emit a category-mismatch
// hint at audit time — specifically the `v02-impl-category-witness-cross-check`
// enforcement (category-vs-predicate-type structural check per ADR-028 G2).
//
// NOTE: This test encodes the category-witness-cross-check discipline that
// v02-impl-category-witness-cross-check campsite aims to ship. If that campsite
// ships first, this test will pass. If recurrent ships first, this test
// ensures the category guard is properly applied to recurrent antigens.
//
// Expected: audit emits category-witness-type-mismatch (or equivalent G2 hint)
// ============================================================================

#[test]
#[ignore = "recurrent family not yet implemented; category-witness-cross-check also pending — remove ignore when both ship"]
fn atk_recurrent_6_strand_wrong_category_emits_mismatch_hint() {
    // An antigen that should be SubstrateAlignment (recurrent pattern detection)
    // but is declared with category = FunctionalCorrectness.
    // Strand connects to VCS substrate; declaring FunctionalCorrectness is wrong.
    // Should emit category-witness-type-mismatch at audit time.
    todo!("implement when recurrent family + category-witness-cross-check both ship");
}
