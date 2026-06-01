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

// ============================================================================
// Adversarial consistency gaps — found in ADR-033 draft sweep (adversarial,
// 2026-06-01). These encode SPECIFICATION HOLES, not just implementation gaps.
// The answers must be settled in ADR-033 before pathmaker implements. Questions
// logged to aristotle via camp question [ae2e3a2d].
// ============================================================================

/// ATK-PRES-12: S3 `triage` WorkVerdict::Fulfilled — is it structurally reachable?
///
/// `#[triage]` coordinates an ordered set of work items (campsites). The ordering
/// is attested via `triaged_by`. The campsites are OPAQUE LABELS — the audit
/// cannot see their done-state (anchor #3 forbids camp reads).
///
/// Q: when does a `#[triage]` site reach WorkVerdict::Fulfilled?
///
/// The ADR specifies:
/// - "attested: the order is attested (triaged_by)" as S3 satisfaction.
/// - But attesting the ORDER is different from completing the WORK.
/// - If Fulfilled = "triaged_by is attested at current fingerprint," then a
///   triage that has a triaged_by attestation is immediately Fulfilled regardless
///   of whether any work was actually done — that's a cardinality collapse
///   (Pending and Fulfilled look the same once the attestation exists).
/// - If Fulfilled requires the campsites to be resolved, the audit can never
///   compute it (opaque labels + no camp reads).
///
/// SPEC GAP: WorkVerdict::Fulfilled for S3 is underspecified. The ADR must
/// clarify whether `#[triage]` can reach Fulfilled at all in v0.3, or whether
/// it is structurally Pending-until-triaged / OutOfFrame-after-expiry.
///
/// ADVERSARIAL PREDICTION: if the implementation treats `triaged_by` attestation
/// as Fulfilled, it creates a trivial bypass — add an attestation, mark Fulfilled,
/// never resolve the actual campsites. This is the same class as
/// `fresh_through=today` (bypass by writing a substrate signal without doing the
/// work). If the implementation leaves Fulfilled unreachable, the audit always
/// shows Pending (no loudness signal) — a silent-information-deficit.
#[test]
#[ignore = "SPEC GAP: ADR-033 does not specify when #[triage] reaches WorkVerdict::Fulfilled. \
            aristotle must resolve this before pathmaker implements S3. See camp question ae2e3a2d. \
            When resolved, convert this to a concrete fixture test."]
fn atk_pres12_triage_fulfilled_is_structurally_reachable_or_unreachable() {
    // When spec is resolved:
    // Option A (triaged_by = Fulfilled):
    //   let result = audit_prescriptive(&triage_site_with_triaged_by, &root);
    //   assert_eq!(result.work_verdict, WorkVerdict::Fulfilled);
    //   // Risk: bypass by attesting order without doing work.
    // Option B (Fulfilled not reachable for S3, frame-expiry = OutOfFrame):
    //   let result = audit_prescriptive(&triage_site_past_re_triage_due, &root);
    //   assert_eq!(result.work_verdict, WorkVerdict::OutOfFrame);
    //   // Correct if triage is a staleness-frame, not a completion-frame.
    todo!("spec must resolve S3 Fulfilled semantics first (question ae2e3a2d)")
}

/// ATK-PRES-13: S4 `quarantine`/`culture` — frame-expiry verdict is Fulfilled or OutOfFrame?
///
/// S4 is described as "until-passes (`quarantine`) / test-green-within-frame (`culture`)."
/// The `until` / `runs_until` field is a temporal window.
///
/// Q: what verdict fires when the frame expires?
///
/// Interpretation A — frame-expiry = Fulfilled:
///   `#[quarantine(scope = "...", until = "<past-date>")]` → WorkVerdict::Fulfilled.
///   Rationale: the quarantine ran its course, the frame closed.
///   Risk: a site that was NEVER actually quarantined (no isolation evidence)
///   automatically becomes Fulfilled when the date passes — bypass by setting `until`
///   to a past date.
///
/// Interpretation B — frame-expiry requires closure attestation = still Pending/Overdue:
///   Expiry without attestation = OutOfFrame (the closing-attestation source is missing).
///   Rationale: same as substrate-witness fingerprint-pinned — a declaration without
///   a closure witness is undefended.
///   Risk: adds a new closure-attestation mechanism not described in the ADR.
///
/// ADVERSARIAL PREDICTION: Interpretation A introduces the same bypass class as
/// `fresh_through=today` — set a past date, mark Fulfilled, no actual work done.
/// Interpretation B is safer but introduces implementation complexity not mentioned.
///
/// SPEC GAP: the ADR must clarify whether S4 frame-expiry alone satisfies WorkVerdict::Fulfilled
/// or whether a closure attestation is required.
#[test]
#[ignore = "SPEC GAP: ADR-033 S4 frame-expiry verdict is underspecified. \
            See camp question ae2e3a2d. Convert to fixture test when resolved."]
fn atk_pres13_s4_frame_expiry_verdict_is_fulfilled_or_out_of_frame() {
    // When spec is resolved:
    // Option A (expiry = Fulfilled):
    //   let site = quarantine_site_with_past_until();
    //   assert_eq!(audit(site).work_verdict, WorkVerdict::Fulfilled);
    //   // Adversarial test: site with `until = yesterday` and NO isolation work.
    //   // Expected: must NOT be Fulfilled if no isolation attestation.
    // Option B (expiry without attestation = OutOfFrame or Overdue):
    //   assert_ne!(audit(site).work_verdict, WorkVerdict::Fulfilled);
    todo!("spec must resolve S4 frame-expiry semantics (question ae2e3a2d)")
}

/// ATK-PRES-14: `triage.priority_order` non-permutation — parse-error or audit OutOfFrame?
///
/// ADR-033 §Proc-Macro-Surface says `priority_order` (if present) "must be a permutation
/// of `campsites`." But note: per the ATK-PRES corpus comment at line 22, `triage.campsites`
/// was DROPPED and `triage` uses `priority_order` as direct code-site references. So the
/// permutation check is now "priority_order targets must resolve to code sites that exist."
///
/// Q: what happens when `priority_order` references a code site that DOESN'T exist?
///
/// ADR says "audit does NOT resolve against camp" — but code sites ARE resolved. So a
/// `priority_order` item that doesn't match any code site is a structural mismatch.
///
/// ADVERSARIAL PREDICTION: if this is a parse-time error, the attacker can make a triage
/// fail to compile by removing a referenced code site (forcing all sites that priority_order
/// reference to be valid at compile time). If this is audit-time OutOfFrame, the triage site
/// silently produces OutOfFrame when references dangle — potentially hiding a real gap.
///
/// SPEC GAP: the resolution tier of the "priority_order items must be valid code-site
/// references" constraint is not specified.
#[test]
#[ignore = "SPEC GAP: ADR-033 does not specify whether triage.priority_order non-resolution \
            fires at parse-time (compile error) or audit-time (OutOfFrame). \
            See camp question ae2e3a2d. Convert to fixture when resolved."]
fn atk_pres14_priority_order_nonresolvable_target_fires_correct_verdict() {
    // Depends on resolution:
    // Option A (parse-time): trybuild compile-fail fixture with `priority_order = ["nonexistent"]`
    // Option B (audit-time): scan_workspace(fixture) → audit → WorkVerdict::OutOfFrame
    todo!("spec must resolve priority_order resolution tier (question ae2e3a2d)")
}

/// ATK-PRES-15: `reviewed_by` ordering — ALL `filled_by` attested, or ANY?
///
/// §Witness-binding says "a `reviewed_by` attestation is only counted if the
/// corresponding `filled_by` step is itself attested."
///
/// The ADR says the verdict is "the conjunction over role-steps." But `filled_by` is
/// `Vec<String>` (multiple fillers), and `reviewed_by` is also `Vec<String>`.
///
/// Q: for a panel with `filled_by = ["alice", "charlie"], reviewed_by = ["bob"]`:
///   - Does bob's review require BOTH alice AND charlie to be attested? (ALL)
///   - Does bob's review require at least ONE of alice/charlie? (ANY)
///   - Is there a per-filler review ordering (alice filled → bob reviews alice's part;
///     charlie filled → a different reviewer for charlie's part)?
///
/// ADVERSARIAL PREDICTION:
/// - "ALL filled_by" is safer (no review before all fill is done) but can produce
///   false-Overdue if one filler is delayed (the conjunction makes the review
///   un-initiatable until every fill is done).
/// - "ANY filled_by" allows review before all fill is done — partial closure risk.
/// - Per-filler mapping re-introduces the positional pairing the ADR explicitly rejected.
///
/// SPEC GAP: the conjunction-over-role-steps semantics is underspecified when filled_by
/// has multiple members. The ADR rejects positional pairing but doesn't specify what
/// "filled_by attested" means across a multi-member list.
#[test]
#[ignore = "SPEC GAP: ADR-033 §Witness-binding 'reviewed_by requires filled_by' does not \
            specify ALL vs ANY for multi-member filled_by. See camp question ae2e3a2d. \
            Implement once spec clarifies."]
fn atk_pres15_reviewed_by_ordering_requires_all_or_any_filled_by() {
    // Degenerate inputs to test when implemented:
    //
    // 1. filled_by = ["alice", "charlie"], reviewed_by = ["bob"]
    //    alice attested, charlie NOT attested, bob attested.
    //    Expected: ??? (Pending if ALL required, Fulfilled if ANY sufficient)
    //
    // 2. filled_by = ["alice"], reviewed_by = ["bob"]
    //    alice NOT attested, bob HAS attestation (out-of-order review).
    //    Expected: OutOfFrame (review precedes fill — ordering violation)
    //
    // 3. filled_by = [], reviewed_by = ["bob"]
    //    No fillers declared, bob attested.
    //    Expected: Fulfilled? (no fill required, only review) or OutOfFrame?
    todo!("spec must resolve multi-member filled_by semantics (question ae2e3a2d)")
}
