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

// When the module exists, add:
// use antigen::recurrent::{RecurrenceAnchor, ItchDeclaration};
// use antigen::scan::{ScanReport, scan_workspace};

// ============================================================================
// ATK-RECURRENT-1: #[itch] threshold = 0 is a silent no-op
//
// ADR-024 §Mechanics: #[itch(threshold = N)] notates a recurrence-awareness
// threshold. A threshold of 0 means "never fires" — the antigen is declared
// but structurally can never trigger the itch-noticed-not-anchored hint.
//
// ATTACK: threshold = 0 should be rejected at parse time (or at least emit
// a warning). An #[itch] with threshold=0 looks like declared discipline but
// produces zero audit signal — exactly the silent-failure class adversarial
// exists to catch.
//
// Expected: parse-time error OR audit-time hint `itch-zero-threshold-is-never`
// ============================================================================

#[test]
#[ignore = "recurrent family not yet implemented — remove ignore when v02-impl-recurrent-emergence ships"]
fn atk_recurrent_1_itch_threshold_zero_is_rejected_or_warned() {
    // #[itch(threshold = 0, description = "pattern noticed")]
    // Should reject at parse time or emit itch-zero-threshold-is-never at audit.
    // A zero threshold means the itch can never be noticed — no-op discipline.
    todo!("implement when recurrent family ships; verify threshold=0 is not silently accepted");
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
#[ignore = "recurrent family not yet implemented — remove ignore when v02-impl-recurrent-emergence ships"]
fn atk_recurrent_3_crystallize_into_nonexistent_antigen_emits_hint() {
    // #[crystallize(into = "NonExistentAntigenName", pattern = "foo")]
    // The named antigen doesn't exist anywhere in the workspace.
    // Should emit crystallize-without-antigen at audit time.
    todo!("implement when recurrent family ships; verify dangling crystallize reference is caught");
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
//   (a) review_date = "not-a-date" — if parsed as raw string, passes validate().
//       Expected: parse-time error for unparseable date format.
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
#[ignore = "recurrent family not yet implemented — remove ignore when v02-impl-recurrent-emergence ships"]
fn atk_recurrent_4a_chronic_review_date_non_date_string_is_rejected() {
    // #[chronic(signal = "memory leak in retry loop", review_date = "not-a-date")]
    // Should produce a parse-time error: review_date must be parseable as YYYY-MM-DD.
    todo!("implement when recurrent family ships; verify review_date format validation");
}

#[test]
#[ignore = "recurrent family not yet implemented — remove ignore when v02-impl-recurrent-emergence ships"]
fn atk_recurrent_4b_chronic_past_review_date_emits_hint() {
    // #[chronic(signal = "memory leak in retry loop", review_date = "2020-01-01")]
    // Date is valid format but in the past. Should emit chronic-signal-past-review-date.
    todo!("implement when recurrent family ships; verify past review_date triggers hint");
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
#[ignore = "recurrent family not yet implemented — remove ignore when v02-impl-recurrent-emergence ships"]
fn atk_recurrent_5_saturate_anchor_nonexistent_emits_hint() {
    // #[saturate(anchor = "NonExistentPattern", evidence = "many occurrences")]
    // The referenced anchor doesn't exist anywhere in the workspace.
    // Should emit saturate-no-anchor at audit time.
    todo!("implement when recurrent family ships; verify dangling saturate anchor is caught");
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
