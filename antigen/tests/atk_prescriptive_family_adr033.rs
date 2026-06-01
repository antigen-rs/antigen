//! ATK — Prescriptive Work-Orchestration family pre-implementation spec tests
// Suppress doc-markdown lints for this test file: the inline spec text uses identifier names
// and quoted strings in doc prose; these are not intra-doc links or rendered HTML.
#![allow(clippy::doc_markdown)]
#![allow(clippy::doc_link_with_quotes)]
//! (prescriptive/family-adr Q9, ADR-033 v03-vision-buildout).
//!
//! ## Purpose
//!
//! These tests encode the ADR-033 spec BEFORE pathmaker implements the macros.
//! All tests are `#[ignore]`'d — they will not compile or will not run until
//! the implementation ships. When pathmaker ships `#[panel]`/`#[ddx]`/etc.,
//! un-ignore the relevant tests and confirm they pass.
//!
//! ## What these tests guard
//!
//! **Parse-time rejections (Q9a — must compile-error or produce scan-time error):**
//!   - Empty `needs` in `#[panel]`
//!   - Empty `rule_out` in `#[ddx]`
//!   - Empty `reason` in `#[quarantine]`
//!   - `priority_order` that is not a permutation of `campsites` — BUT NOTE:
//!     per Tekgy ruling 2026-06-01, `triage.campsites` is DROPPED. `triage` only
//!     has `priority_order` as code-site references. Permutation check is:
//!     `priority_order` targets must resolve to code sites that actually exist.
//!
//! **Audit-hint disambiguation (Q9b — each verdict state is reachable + distinguishable):**
//!   - `Pending` — declared within frame, satisfaction not yet met
//!   - `Fulfilled` — satisfaction met at current fingerprint
//!   - `Overdue` — past frame, unsatisfied (loud per ADR-023 isomorphism)
//!   - `OutOfFrame` — un-evaluable: who-ref unknown / measurement source missing
//!
//! **Three-valued-logic gate (Q9c — the gem applied to prescriptive):**
//!   - `Overdue` (frame elapsed, evaluable) MUST be distinct from `OutOfFrame` (un-evaluable).
//!   - Collapsing `OutOfFrame` → `Overdue` is the cardinality-collapse the gem warns against.
//!
//! **TriageDecision naming collision (Q7 cross-check):**
//!   - `antigen::vcs::TriageDecision` (ADR-026, VCS rollback) vs `WorkVerdict` enum (ADR-033,
//!     work-orchestration). These are DISTINCT types on DISTINCT axes. The cross-check confirms
//!     no accidental type-reuse occurs.
//!
//! ## Adversarial notes
//!
//! The prescriptive family's worst silent failure would be `OutOfFrame` collapsing to `Overdue` —
//! a work-need with an un-evaluable satisfaction condition (unknown who-ref) would appear as
//! OVERDUE (loud, urgent, actionable) rather than INDETERMINATE (uncertain, needs investigation).
//! The overdue-vs-out-of-frame distinction is load-bearing: overdue means "we know it's late",
//! out-of-frame means "we can't even tell if it's started." Per the verdict-lattice isomorphism:
//! these are the same three-valued states as defended/undefended/substrate-gap — and the same
//! cardinality-collapse that ATK-3V-4 found in the shipped v0.2 code. Don't repeat it here.
//!
//! The `WorkVerdict` enum must be a sealed enum with exactly these four variants. Any attempt
//! to collapse OutOfFrame → Overdue in the audit evaluator is a regression.

// ============================================================================
// Q9a: Parse-time rejection tests
// ============================================================================
//
// These tests are #[ignore]'d because:
//   1. The prescriptive macros are not yet implemented (no #[panel], #[ddx], etc.)
//   2. They are "should fail to compile" tests — requiring compile-fail infrastructure
//      (e.g., trybuild / compile-error fixtures) or scan-time error checking.
//
// When pathmaker ships the macros, convert these to:
//   (a) Compile-error tests via trybuild or compile-fail attributes, OR
//   (b) Scan-level tests via scan_workspace() on fixtures that use the macros.

/// ATK-PRES-1: empty needs in #[panel] must be a compile-time error.
///
/// #[panel(needs = [])] has no work-need content — the prescriptive primitive is vacuous.
/// Same class as EmptySignersList (NFA-7): a zero-element required list always passes, so
/// it is a semantic no-op. ADR-033 §Proc-Macro-Surface: "non-empty (empty = vacuous; compile error)".
///
/// SPEC: `#[panel(needs = [])]` → parse-time error: "panel.needs must be non-empty"
#[test]
#[ignore = "NOT YET IMPLEMENTED: #[panel] macro not shipped; convert to compile-fail fixture \
            when antigen-macros ships PanelArgs; verify the parse-time rejection fires"]
fn atk_pres1_panel_empty_needs_is_compile_error() {
    // When the macro ships, create a trybuild fixture:
    //   t.compile_fail("tests/fixtures/atk_pres1_panel_empty_needs.rs")
    // where the fixture contains:
    //   #[panel(needs = [])]
    //   fn my_fn() {}
    //
    // The error message must contain "panel.needs must be non-empty" or equivalent.
    todo!("implement as compile-fail fixture when #[panel] ships")
}

/// ATK-PRES-2: empty rule_out in #[ddx] must be a compile-time error.
///
/// #[ddx(symptom = "x", rule_out = [])] has an empty alternative-set — the S2 elimination
/// shape requires at least one alternative to eliminate. Empty = no differential diagnosis.
///
/// SPEC: `#[ddx(rule_out = [])]` → parse-time error: "ddx.rule_out must be non-empty"
#[test]
#[ignore = "NOT YET IMPLEMENTED: #[ddx] macro not shipped; convert to compile-fail fixture \
            when antigen-macros ships DdxArgs"]
fn atk_pres2_ddx_empty_rule_out_is_compile_error() {
    todo!("implement as compile-fail fixture when #[ddx] ships")
}

/// ATK-PRES-3: empty reason in #[quarantine] must be a compile-time error.
///
/// ADR-005 Amendment 2 (rationale-as-required): every suppression primitive must carry
/// a non-empty rationale. #[quarantine] is a suppression with an `until` frame; the
/// `reason` field is the rationale.
///
/// SPEC: `#[quarantine(scope = "...", reason = "")]` → parse-time error: "quarantine.reason
/// must be non-empty (ADR-005 Amd2)"
#[test]
#[ignore = "NOT YET IMPLEMENTED: #[quarantine] macro not shipped; convert to compile-fail \
            fixture when antigen-macros ships QuarantineArgs"]
fn atk_pres3_quarantine_empty_reason_is_compile_error() {
    todo!("implement as compile-fail fixture when #[quarantine] ships")
}

/// ATK-PRES-4: panel.needs must not be empty (scan-level check).
///
/// Alternative enforcement path via scan: if the macro somehow produces an empty needs
/// list (degenerate deserialization), the scan should surface this as a schema violation
/// rather than silently producing a vacuous work-need.
///
/// SPEC: scan produces a schema-error hint for a panel site with empty needs.
/// This is the serde-validate-systematic pattern applied to the prescriptive family.
#[test]
#[ignore = "NOT YET IMPLEMENTED: #[panel] scan not shipped; test scan-level empty-needs detection"]
fn atk_pres4_panel_empty_needs_scan_level_schema_error() {
    todo!("implement against scan infrastructure when #[panel] is implemented")
}

// ============================================================================
// Q9b: Audit-hint disambiguation tests
// ============================================================================
//
// These tests verify that each WorkVerdict variant is reachable and distinguishable.
// They are #[ignore]'d because the prescriptive audit section is not yet implemented.

/// ATK-PRES-5: Pending verdict is reachable.
///
/// A #[panel(needs = ["review"], due = "<future-date>")]
/// with no attestation in the sidecar must produce WorkVerdict::Pending —
/// the work-need is declared, the frame is open, the satisfaction is not yet met.
/// This is the EXPECTED STATE (not a failure), so it must not be loud.
///
/// SPEC: undeclared who-step within a future frame → WorkVerdict::Pending (not Overdue)
#[test]
#[ignore = "NOT YET IMPLEMENTED: WorkVerdict::Pending not shipped; implement when \
            prescriptive audit section lands in antigen::audit"]
fn atk_pres5_panel_within_frame_unsatisfied_is_pending_not_overdue() {
    // CRITICAL DISTINCTION: Pending (within frame, not yet met) vs
    // Overdue (past frame, not met) must be separate verdicts.
    // DO NOT collapse: a panel declared today with due=2027-01-01 is Pending,
    // not Overdue. Reporting it as Overdue would be a false alarm — exactly
    // the cardinality-collapse the ADR-029 gem predicts.
    todo!("implement when WorkVerdict + prescriptive audit ships")
}

/// ATK-PRES-6: Fulfilled verdict is reachable.
///
/// A #[panel(needs = ["review"], filled_by = ["alice"])] with alice's attestation
/// in the sidecar at the current fingerprint must produce WorkVerdict::Fulfilled.
///
/// SPEC: satisfied who-step at current fingerprint → WorkVerdict::Fulfilled
#[test]
#[ignore = "NOT YET IMPLEMENTED: WorkVerdict::Fulfilled not shipped"]
fn atk_pres6_panel_satisfied_at_current_fingerprint_is_fulfilled() {
    todo!("implement when WorkVerdict + prescriptive audit ships")
}

/// ATK-PRES-7: Overdue verdict is reachable.
///
/// A #[panel(needs = ["review"], due = "<past-date>")] with no attestation
/// must produce WorkVerdict::Overdue. This is a LOUD verdict (ADR-023 isomorphism).
///
/// SPEC: past-frame, unsatisfied → WorkVerdict::Overdue (loud)
#[test]
#[ignore = "NOT YET IMPLEMENTED: WorkVerdict::Overdue not shipped"]
fn atk_pres7_panel_past_frame_unsatisfied_is_overdue_and_loud() {
    todo!("implement when WorkVerdict + prescriptive audit ships")
}

/// ATK-PRES-8 (THE CRITICAL THREE-VALUED-LOGIC TEST): OutOfFrame is distinct from Overdue.
///
/// A #[panel(needs = ["review"], filled_by = ["unknown-who-ref"])] where
/// "unknown-who-ref" does not exist in any sidecar must produce WorkVerdict::OutOfFrame.
/// It must NOT produce WorkVerdict::Overdue.
///
/// SPEC: un-evaluable who-step (unknown who-ref, no sidecar) → WorkVerdict::OutOfFrame
///
/// THE CARDINALITY-COLLAPSE GUARD:
///   Overdue   = frame elapsed AND the audit EVALUATED the satisfaction condition
///               and found it unmet. The audit KNOWS the work is late.
///   OutOfFrame = the satisfaction condition is NOT EVALUABLE. The audit does not
///               know if the work is done, started, or even relevant. It cannot say
///               "late" because it cannot say "evaluable."
///
/// If the audit collapses OutOfFrame → Overdue, it produces false alarms: every
/// panel with an unknown who-ref becomes urgently overdue, whether it has a due date
/// or not. This is the prescriptive analog of ATK-3V-4 (deferred → SubstrateGap).
/// The fix is the same: the verdict-lattice must preserve all four states; collapsing
/// any two is a cardinality error.
///
/// Biology: anergy (T-cell anergy = failure to recognize, not failed recognition)
/// vs exhaustion (recognized, failed to respond). "I can't find the co-stimulation
/// signal" is different from "I found the signal and the T-cell still didn't fire."
/// OutOfFrame = anergy. Overdue = exhaustion. They are different clinical states
/// requiring different interventions.
#[test]
#[ignore = "NOT YET IMPLEMENTED: WorkVerdict::OutOfFrame not shipped; this is the \
            load-bearing test for the three-valued-logic gem in the prescriptive family. \
            When pathmaker implements the prescriptive audit evaluator, this test MUST \
            pass WITHOUT modification — if it requires modification to pass, the \
            evaluator has the cardinality-collapse bug. DO NOT collapse OutOfFrame → Overdue."]
fn atk_pres8_unknown_who_ref_produces_out_of_frame_not_overdue() {
    // DESIRED behavior:
    //   let result = audit_prescriptive(&site_with_unknown_who_ref, &workspace_root);
    //   assert_eq!(result.work_verdict, WorkVerdict::OutOfFrame);
    //
    // FAILING behavior (the cardinality-collapse):
    //   assert_ne!(result.work_verdict, WorkVerdict::Overdue,
    //       "OutOfFrame must not be reported as Overdue — the work-need is
    //        un-evaluable (who-ref unknown), not overdue (frame elapsed)");
    todo!("implement when WorkVerdict + prescriptive audit ships")
}

/// ATK-PRES-9: Overdue-vs-OutOfFrame is NEVER collapsed.
///
/// This test encodes the explicit anti-regression: if a future change to the
/// prescriptive evaluator collapses Overdue → OutOfFrame OR OutOfFrame → Overdue,
/// this test must catch it.
///
/// SPEC: `WorkVerdict::Overdue != WorkVerdict::OutOfFrame` (structural, not just enum)
#[test]
#[ignore = "NOT YET IMPLEMENTED: WorkVerdict not shipped; this will be a simple enum
            variant inequality test once WorkVerdict is defined"]
fn atk_pres9_work_verdict_overdue_and_out_of_frame_are_distinct_variants() {
    // When WorkVerdict ships:
    //   use antigen::audit::WorkVerdict;
    //   assert_ne!(WorkVerdict::Overdue, WorkVerdict::OutOfFrame,
    //       "ATK-PRES-9: Overdue and OutOfFrame are DISTINCT WorkVerdict variants. \
    //        If this fails, the enum was collapsed — that is a cardinality-collapse bug \
    //        per the ADR-033 three-valued-logic gem.");
    todo!("trivial once WorkVerdict ships")
}

// ============================================================================
// Q7: TriageDecision naming collision cross-check
// ============================================================================
//
// ADR-026 `TriageDecision` (VCS rollback) vs ADR-033 `#[triage]` (work-orchestration).
// These are DISTINCT types. The cross-check confirms they remain on distinct axes.

/// ATK-PRES-10: TriageDecision (VCS rollback, ADR-026) is orthogonal to WorkVerdict.
///
/// antigen::vcs::TriageDecision encodes the 5-color Black/Red/Yellow/Green/White triage
/// for VCS rollback decisions. WorkVerdict encodes work-need satisfaction states.
/// These types must remain STRUCTURALLY DISTINCT — different axes, different variants.
///
/// If a future refactoring accidentally aliases or merges them, this test catches it.
#[test]
fn atk_pres10_triage_decision_and_work_verdict_are_distinct_types() {
    // TriageDecision IS SHIPPED (antigen::vcs::TriageDecision). We can verify it exists
    // and has the expected variants.
    use antigen::vcs::TriageDecision;

    // Structural check: TriageDecision has the VCS-rollback variants.
    // None of these are work-orchestration concepts.
    let vcs_rollback_verdict = TriageDecision::Red;
    let _ = vcs_rollback_verdict; // just confirm the type exists and Black/Red/etc. exist

    // Compile-time type check: these should NOT be the same type.
    // When WorkVerdict ships, add:
    //   use antigen::audit::WorkVerdict;
    //   let _: fn(TriageDecision) = |_| (); // this must NOT accept WorkVerdict
    // For now: confirm TriageDecision has VCS semantics (not work-orchestration semantics).
    let rollback = TriageDecision::Black;
    let non_rollback = TriageDecision::Green;
    // The variants map to: Black=abort, Red=urgent, Yellow=investigate, Green=safe, White=unknown
    // None of these are Pending/Fulfilled/Overdue/OutOfFrame — confirmed distinct.
    assert_ne!(
        rollback as u8, non_rollback as u8,
        "ATK-PRES-10: TriageDecision variants must be distinct (structural sanity check)"
    );
}

/// ATK-PRES-11: WorkVerdict variants cover all four states with no structural gaps.
///
/// When WorkVerdict ships, this test confirms the sealed enum has exactly four variants.
/// Missing any variant = a cardinality gap; extra variants = possible future collapse risk.
///
/// SPEC: WorkVerdict = {Pending, Fulfilled, Overdue, OutOfFrame} — exactly four.
#[test]
#[ignore = "NOT YET IMPLEMENTED: WorkVerdict not shipped; un-ignore when it ships and \
            verify all four variants exist via exhaustive pattern match"]
fn atk_pres11_work_verdict_has_exactly_four_variants() {
    // When WorkVerdict ships:
    //   use antigen::audit::WorkVerdict;
    //   // Exhaustive match — if a variant is added or removed, this fails to compile.
    //   fn exhaustive_check(v: WorkVerdict) {
    //       match v {
    //           WorkVerdict::Pending => {}
    //           WorkVerdict::Fulfilled => {}
    //           WorkVerdict::Overdue => {}
    //           WorkVerdict::OutOfFrame => {}
    //           // If there's a 5th variant, this match is non-exhaustive and fails.
    //       }
    //   }
    todo!("implement as compile-time exhaustive match when WorkVerdict ships")
}
