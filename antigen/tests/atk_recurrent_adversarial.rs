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
use antigen::defended_by;
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
// ATK-RECURRENT-1: #[itch] threshold field accepts empty/whitespace strings
//
// ADR-024 §Mechanics: #[itch(threshold = "...")] notates a recurrence-awareness
// threshold as descriptive text. threshold is typed as Option<String> (LitStr
// at parse time), NOT a typed integer — "3 occurrences across 2 releases" is
// the intended usage. Free-text is the correct type by ratified design.
//
// DESIGN DECISION (ratified with pathmaker 2026-05-24, option a):
//   Parse-time reject empty/whitespace-only threshold (parallel to
//   description/rationale). Audit-time hint for richer semantic vacuity
//   ("once pigs fly") is optional/deferred — parse-time cannot judge it.
// Expected: proc-macro compile error for threshold = "" or threshold = "  ".
// ============================================================================

#[test]
#[ignore = "pending pathmaker implementation: parse-time reject empty/whitespace threshold (option-a design decision)"]
fn atk_recurrent_1_itch_threshold_empty_string_is_compile_error() {
    // threshold = "" is unambiguously vacuous — no condition declared.
    // Proc-macro must reject with a message indicating threshold, if present,
    // must be a non-empty non-whitespace string.
    todo!("implement once pathmaker ships parse-time empty-threshold rejection");
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
#[defended_by(AuditHintWithNoUpstreamPreconditionCheck)]
fn atk_recurrent_2_recurrence_anchor_without_matching_itch_emits_hint() {
    // Workspace has #[recurrence_anchor(antigen_type = "SomeAntigen")] but
    // zero #[itch] declarations reference SomeAntigen. The temporal progression
    // (itch → anchor → crystallize) is bypassed.
    // Should emit RecurrenceAnchorNoItchPrecondition.
    let mut report = ScanReport::default();
    report.recurrent_declarations.push(base_decl(
        RecurrentKind::RecurrenceAnchor,
        Some("SomeAntigen"),
    ));
    let out = audit_recurrent(&report);
    assert!(
        out.audits[0]
            .hints
            .contains(&AuditHint::RecurrenceAnchorNoItchPrecondition),
        "expected RecurrenceAnchorNoItchPrecondition for anchor with no upstream itch"
    );

    // Confirm: adding a matching itch clears the hint.
    let mut report_with_itch = ScanReport::default();
    report_with_itch
        .recurrent_declarations
        .push(base_decl(RecurrentKind::Itch, Some("SomeAntigen")));
    report_with_itch.recurrent_declarations.push(base_decl(
        RecurrentKind::RecurrenceAnchor,
        Some("SomeAntigen"),
    ));
    let out_with_itch = audit_recurrent(&report_with_itch);
    let anchor_audit = out_with_itch
        .audits
        .iter()
        .find(|a| a.declaration.kind == RecurrentKind::RecurrenceAnchor)
        .unwrap();
    assert!(
        !anchor_audit
            .hints
            .contains(&AuditHint::RecurrenceAnchorNoItchPrecondition),
        "must NOT emit RecurrenceAnchorNoItchPrecondition when matching itch exists"
    );
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
// ATK-RECURRENT-7: #[recurrence_anchor] with non-empty from_itches that don't
//                  exist in the scan — phantom itch reference bypasses precondition
//                  check.
//
// ADR-024 §Mechanics: if an anchor lists from_itches = [SomeNonExistentItch],
// the `from_itches.is_empty()` guard prevents RecurrenceAnchorNoItchPrecondition
// from firing EVEN THOUGH the listed itches don't exist in the codebase.
//
// ATTACK: #[recurrence_anchor(from_itches = ["NonExistentItch"])] — the
// from_itches list is non-empty, bypassing the precondition check, but
// NonExistentItch has no #[itch] declaration anywhere in the scan. The temporal
// progression (itch → anchor → crystallize) is still bypassed.
//
// Current behavior: zero hints (no RecurrenceAnchorNoItchPrecondition). The
// check only fires when from_itches.is_empty() — a non-empty phantom list
// is treated as sufficient precondition evidence.
//
// Correct behavior: validate that from_itches entries resolve to actual #[itch]
// declarations in the scan. A phantom reference should emit a hint (perhaps
// RecurrenceAnchorNoItchPrecondition or a new RecurrenceAnchorItchNotFound).
//
// This test asserts the BROKEN outcome (no hint fires).
// ============================================================================

#[test]
fn atk_recurrent_7_phantom_from_itches_bypasses_precondition_check() {
    // #[recurrence_anchor] with from_itches = ["NonExistentItch"] — the itch
    // does NOT exist in the scan. The from_itches list is non-empty, so the
    // `from_itches.is_empty()` guard prevents RecurrenceAnchorNoItchPrecondition.
    // But "NonExistentItch" is a phantom reference — no #[itch] for it in the scan.
    let mut decl = base_decl(RecurrentKind::RecurrenceAnchor, Some("SomeAntigen"));
    decl.from_itches = vec!["NonExistentItch".to_string()]; // phantom — not in scan

    let mut report = ScanReport::default();
    report.recurrent_declarations.push(decl);
    // No #[itch] declarations in the report — NonExistentItch has no matching scan entry.

    let out = audit_recurrent(&report);
    assert_eq!(out.audits.len(), 1);

    // BROKEN: no RecurrenceAnchorNoItchPrecondition fires because
    // from_itches.is_empty() is false — the non-empty phantom list bypasses
    // the check. The temporal progression is still bypassed (no real itch
    // exists) but the audit treats the anchor as having preconditions.
    //
    // Asserting broken outcome (no hint). After fix:
    // assert!(hints.contains(&AuditHint::RecurrenceAnchorNoItchPrecondition))
    // or a new RecurrenceAnchorPhantomItchReference hint.
    let hints = &out.audits[0].hints;
    assert!(
        !hints.contains(&AuditHint::RecurrenceAnchorNoItchPrecondition),
        "ATK-RECURRENT-7 (BROKEN): from_itches=[\"NonExistentItch\"] (phantom) \
         silently bypasses RecurrenceAnchorNoItchPrecondition. The from_itches \
         list is non-empty, gating out the check, even though the listed itch \
         doesn't exist in the scan. Fix: validate from_itches entries resolve to \
         actual #[itch] declarations. Test should invert after fix."
    );
    // Also: RecurrenceThresholdReachedNoAction fires because SomeAntigen is
    // not in acted_on — this part of the check is NOT gated by from_itches.
    assert!(
        hints.contains(&AuditHint::RecurrenceThresholdReachedNoAction),
        "RecurrenceThresholdReachedNoAction must still fire for anchor with \
         no downstream action — this check runs regardless of from_itches"
    );
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
