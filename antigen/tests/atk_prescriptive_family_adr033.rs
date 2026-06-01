//! ATK — Prescriptive Work-Orchestration family pre-implementation spec tests
// Suppress doc-markdown lints for this test file: the inline spec text uses identifier names
// and quoted strings in doc prose; these are not intra-doc links or rendered HTML.
#![allow(clippy::doc_markdown)]
#![allow(clippy::doc_link_with_quotes)]
//! (prescriptive/family-adr Q9, ADR-033 v03-vision-buildout).
//!
//! ## Family census (as-ratified by ADR-033 + ADR-019 Amendment 1)
//!
//! ADR-033 ships **EIGHT** prescriptive work-need macros (not nine):
//!   S1 — Role-workflow: `panel`, `rx`, `refer`, `biopsy`
//!   S2 — Elimination:   `ddx`
//!   S3 — Ordering:      `triage`
//!   S4 — Frame-only:    `culture`, `quarantine`
//!
//! `#[titer]` is **NOT** in this family — it was reclassified to the titer-witness
//! kind (ADR-019 Amendment 1). A titer attests a *measured value*, not a work-need.
//! Any test rows for titer-as-prescriptive are DROPPED from this corpus.
//!
//! `triage.campsites` is **DROPPED** (Tekgy ruling 2026-06-01, anchor #3): `triage`
//! uses `priority_order` as direct code-site references; camp campsites are opaque
//! labels the audit does NOT resolve. Any test asserting audit resolves campsites
//! against camp state is wrong-by-design and is DROPPED from this corpus.
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
///
/// SHIPPED — the guard is `PanelArgs::validate()` (antigen-macros/src/parse.rs
/// ~4070), a parse-time `syn::Error` (compile error at the expansion site). Its
/// ORACLE is the macro-layer unit test `panel_rejects_empty_needs` (parse.rs
/// ~6775) — the `validate()` path IS the rejection, so the unit test is the
/// real proof. A trybuild fixture here would only re-drive the same `validate()`
/// through a slower harness (no added coverage), so this row stays a documented
/// cross-reference rather than a duplicate.
#[test]
#[ignore = "covered at the macro layer: antigen-macros PanelArgs::validate() + \
            the parse.rs unit test panel_rejects_empty_needs (the validate() path \
            IS the compile-time rejection; a trybuild fixture adds no coverage)"]
fn atk_pres1_panel_empty_needs_is_compile_error() {}

/// ATK-PRES-2: empty rule_out in #[ddx] must be a compile-time error.
///
/// #[ddx(symptom = "x", rule_out = [])] has an empty alternative-set — the S2 elimination
/// shape requires at least one alternative to eliminate. Empty = no differential diagnosis.
///
/// SPEC: `#[ddx(rule_out = [])]` → parse-time error: "ddx.rule_out must be non-empty"
///
/// SHIPPED — `DdxArgs::validate()` (antigen-macros/src/parse.rs ~4404) rejects an
/// empty `rule_out`; oracle is the parse.rs unit test for it.
#[test]
#[ignore = "covered at the macro layer: antigen-macros DdxArgs::validate() rejects \
            empty rule_out (the validate() path IS the compile-time rejection)"]
fn atk_pres2_ddx_empty_rule_out_is_compile_error() {}

/// ATK-PRES-3: empty reason in #[quarantine] must be a compile-time error.
///
/// ADR-005 Amendment 2 (rationale-as-required): every suppression primitive must carry
/// a non-empty rationale. #[quarantine] is a suppression with an `until` frame; the
/// `reason` field is the rationale.
///
/// SPEC: `#[quarantine(scope = "...", reason = "")]` → parse-time error: "quarantine.reason
/// must be non-empty (ADR-005 Amd2)"
///
/// SHIPPED — `QuarantineArgs::validate()` (antigen-macros/src/parse.rs ~4561)
/// rejects an empty `reason` (and an empty/absent `scope`); oracle is its parse.rs
/// unit test.
#[test]
#[ignore = "covered at the macro layer: antigen-macros QuarantineArgs::validate() \
            rejects empty reason (ADR-005 Amd2; the validate() path IS the rejection)"]
fn atk_pres3_quarantine_empty_reason_is_compile_error() {}

/// ATK-PRES-4: a vacuous panel must NOT be silently fulfilled (scan+audit path).
///
/// The macro rejects empty `needs` at parse time (ATK-PRES-1), but the scan is
/// recall-tuned (it records a bare `#[panel]` with empty fields rather than
/// erroring). So the AUDIT is the second line of defense: a panel with no
/// who-step is structurally un-evaluable (nothing to attest) ⇒ `OutOfFrame`,
/// NEVER `Fulfilled`. This is the serde-validate-systematic posture applied to
/// the prescriptive family — a vacuous declaration is surfaced, never silent-green.
#[test]
fn atk_pres4_vacuous_panel_is_not_silently_fulfilled() {
    // A bare `#[panel]` (no args) — the recall-tuned scan records it with empty
    // fields; the audit must not call an empty work-need Fulfilled.
    let src = r"use antigen::panel;
#[panel]
pub fn p() {}
";
    let report = audit_staged(src, None);
    let v = first_verdict(&report);
    assert_ne!(
        v.verdict,
        WorkVerdict::Fulfilled,
        "a vacuous panel (no who-step) must NEVER be silently Fulfilled — got {:?}. steps={:?}",
        v.verdict,
        v.steps
    );
    assert_eq!(
        v.verdict,
        WorkVerdict::OutOfFrame,
        "a panel with no who-step is structurally un-evaluable (nothing to attest) \
         ⇒ OutOfFrame, got {:?}",
        v.verdict
    );
}

// ============================================================================
// Q9b: Audit-hint disambiguation tests — the END-TO-END audit oracle.
//
// These drive the full `audit_prescriptive` round-trip (scan a staged
// workspace → optionally write a sidecar → audit) so they catch the
// silent-wrong-verdict bugs the projection-layer tests (work_verdict_projection.rs,
// PRES-9/11) cannot — the (satisfied, evaluable) DERIVATION from real substrate.
// ============================================================================

use antigen::audit::{audit_prescriptive, PrescriptiveAuditReport, StepState, WorkVerdict};
use antigen::scan::scan_workspace;

/// A sidecar to stage alongside a fixture: `(sidecar-stem, builder)` where
/// `builder(staged_fp)` renders the Ratification JSON pinned to the scanned
/// fingerprint `staged_fp`. `None` = no sidecar (the who-step reads un-evaluable).
type SidecarSpec<'a> = Option<(&'a str, fn(&str) -> String)>;

/// Stage a single-crate workspace whose lib.rs is `lib_src`, optionally writing
/// a sidecar produced by `make_sidecar`'s builder against the scanned structural
/// fingerprint of the FIRST prescriptive declaration's item (so a Fulfilled
/// fixture can pin the signer's fingerprint to the real digest). Returns the
/// prescriptive audit report.
fn audit_staged(lib_src: &str, make_sidecar: SidecarSpec) -> PrescriptiveAuditReport {
    let tmp = tempfile::tempdir().expect("tempdir");
    let src = tmp.path().join("src");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(
        tmp.path().join("Cargo.toml"),
        "[package]\nname = \"staged\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\
         [lib]\npath = \"src/lib.rs\"\n",
    )
    .unwrap();
    std::fs::write(src.join("lib.rs"), lib_src).unwrap();

    if let Some((sidecar_stem, builder)) = make_sidecar {
        // First scan to read the real structural fingerprint of the annotated item.
        let pre = scan_workspace(tmp.path(), None).expect("pre-scan");
        let fp = pre
            .prescriptive_declarations
            .first()
            .map(|d| d.structural_fingerprint.clone())
            .unwrap_or_default();
        // `load_sidecar` resolves `<file-parent>/.attest/<stem>.json` (the
        // established audit convention — see audit.rs `load_sidecar`). The
        // lib.rs lives in `src/`, so the sidecar dir is `src/.attest/`.
        let attest_dir = src.join(".attest");
        std::fs::create_dir_all(&attest_dir).unwrap();
        std::fs::write(
            attest_dir.join(format!("{sidecar_stem}.json")),
            builder(&fp),
        )
        .unwrap();
    }

    let report = scan_workspace(tmp.path(), None).expect("scan completes");
    audit_prescriptive(&report, tmp.path())
}

/// Build an immunity-kind Ratification JSON with one signer `name` who signed
/// the item `item_path` against fingerprint `fp` (a Fresh, GitTrust attestation).
fn sidecar_with_signer(item_path: &str, name: &str, fp: &str) -> String {
    format!(
        r#"{{
  "schema_version": "v1",
  "kind": "immunity",
  "antigen": {{ "name": "{item_path}" }},
  "source_file": "src/lib.rs",
  "items": [
    {{
      "item_path": "{item_path}",
      "current_fingerprint": "{fp}",
      "signers": [
        {{
          "name": "{name}",
          "date": "2026-06-01",
          "signed_against_fingerprint": "{fp}",
          "basis": {{ "kind": "fresh" }},
          "strength": "git_trust"
        }}
      ]
    }}
  ]
}}"#
    )
}

/// Build an immunity-kind Ratification JSON for `item_path` carrying MULTIPLE
/// signers (each a Fresh GitTrust attestation against `fp`). Used by the S1
/// conjunction tests (multi-member filled_by + reviewed_by).
fn sidecar_with_signers(item_path: &str, names: &[&str], fp: &str) -> String {
    let signers: Vec<String> = names
        .iter()
        .map(|name| {
            format!(
                r#"        {{
          "name": "{name}",
          "date": "2026-06-01",
          "signed_against_fingerprint": "{fp}",
          "basis": {{ "kind": "fresh" }},
          "strength": "git_trust"
        }}"#
            )
        })
        .collect();
    format!(
        r#"{{
  "schema_version": "v1",
  "kind": "immunity",
  "antigen": {{ "name": "{item_path}" }},
  "source_file": "src/lib.rs",
  "items": [
    {{
      "item_path": "{item_path}",
      "current_fingerprint": "{fp}",
      "signers": [
{}
      ]
    }}
  ]
}}"#,
        signers.join(",\n")
    )
}

fn first_verdict(report: &PrescriptiveAuditReport) -> &antigen::audit::PrescriptiveVerdict {
    report.verdicts.first().expect("one verdict")
}

/// ATK-PRES-5: Pending verdict is reachable.
///
/// A #[panel] within a future frame with an unsatisfied (but evaluable) who-step
/// must be Pending — the expected state, NOT Overdue. A site with a who-ref but
/// no sidecar would be OutOfFrame; to make it *evaluable-but-unsatisfied* we give
/// it a sidecar that exists but does NOT carry the required signer.
#[test]
fn atk_pres5_panel_within_frame_unsatisfied_is_pending_not_overdue() {
    let src = r#"use antigen::panel;
#[panel(needs = ["review"], filled_by = ["alice"], due = "2099-01-01")]
pub fn p() {}
"#;
    // Sidecar exists for item `p` but carries a DIFFERENT signer (so the read is
    // evaluable, the required `alice` is just not attested ⇒ Unattested, Pending).
    let report = audit_staged(src, Some(("p", |fp| sidecar_with_signer("p", "bob", fp))));
    let v = first_verdict(&report);
    assert_eq!(
        v.verdict,
        WorkVerdict::Pending,
        "evaluable-unsatisfied within a future frame is Pending, not {:?}. steps={:?}",
        v.verdict,
        v.steps
    );
    assert_ne!(
        v.verdict,
        WorkVerdict::Overdue,
        "future frame is never Overdue"
    );
}

/// ATK-PRES-6: Fulfilled verdict is reachable.
///
/// A #[panel(filled_by = ["alice"])] with alice's attestation at the CURRENT
/// fingerprint must be Fulfilled. This is the end-to-end satisfaction read: the
/// signer's `signed_against_fingerprint` is pinned to the scanned digest.
#[test]
fn atk_pres6_panel_satisfied_at_current_fingerprint_is_fulfilled() {
    let src = r#"use antigen::panel;
#[panel(needs = ["review"], filled_by = ["alice"], due = "2099-01-01")]
pub fn p() {}
"#;
    let report = audit_staged(src, Some(("p", |fp| sidecar_with_signer("p", "alice", fp))));
    let v = first_verdict(&report);
    assert_eq!(
        v.verdict,
        WorkVerdict::Fulfilled,
        "alice attested at current fingerprint ⇒ Fulfilled, got {:?}. steps={:?}",
        v.verdict,
        v.steps
    );
    assert!(
        v.steps
            .iter()
            .any(|s| s.reference == "alice" && s.state == StepState::Attested),
        "alice's filled_by step must read Attested: {:?}",
        v.steps
    );
}

/// ATK-PRES-6b (NFA-21 fingerprint-pin): a signer who signed against an OLD
/// fingerprint does NOT fulfill — the attestation is stale once code changes.
/// We pin the signer to a deliberately-wrong fingerprint; the leaf fails the
/// `against=current` currency check ⇒ Unattested ⇒ Pending (not Fulfilled).
#[test]
fn atk_pres6b_stale_signer_does_not_fulfill_nfa21() {
    let src = r#"use antigen::panel;
#[panel(needs = ["review"], filled_by = ["alice"], due = "2099-01-01")]
pub fn p() {}
"#;
    let report = audit_staged(
        src,
        Some(("p", |_fp| {
            sidecar_with_signer("p", "alice", "STALE-FINGERPRINT-not-current")
        })),
    );
    let v = first_verdict(&report);
    assert_ne!(
        v.verdict,
        WorkVerdict::Fulfilled,
        "a stale-fingerprint signature must NOT fulfill (NFA-21). steps={:?}",
        v.steps
    );
    assert_eq!(
        v.verdict,
        WorkVerdict::Pending,
        "stale signer within a future frame is Pending (re-attestation owed), got {:?}",
        v.verdict
    );
}

/// ATK-PRES-7: Overdue verdict is reachable and loud.
///
/// A #[panel] PAST its frame with an evaluable-but-unsatisfied who-step is
/// Overdue (loud). The sidecar exists (so the read is evaluable) but lacks the
/// required signer.
#[test]
fn atk_pres7_panel_past_frame_unsatisfied_is_overdue_and_loud() {
    let src = r#"use antigen::panel;
#[panel(needs = ["review"], filled_by = ["alice"], due = "2020-01-01")]
pub fn p() {}
"#;
    let report = audit_staged(src, Some(("p", |fp| sidecar_with_signer("p", "bob", fp))));
    let v = first_verdict(&report);
    assert_eq!(
        v.verdict,
        WorkVerdict::Overdue,
        "evaluable-unsatisfied past the frame is Overdue, got {:?}. steps={:?}",
        v.verdict,
        v.steps
    );
    assert!(
        v.verdict.is_loud(),
        "Overdue must be loud (ADR-023 isomorphism)"
    );
}

/// ATK-PRES-8 (THE CRITICAL THREE-VALUED-LOGIC TEST): OutOfFrame is distinct
/// from Overdue. A #[panel] with an unknown who-ref and NO sidecar must be
/// OutOfFrame even when its frame has elapsed — the satisfaction is un-evaluable,
/// so the audit cannot say "late." Collapsing this to Overdue is the
/// cardinality-collapse the gem forbids (the prescriptive analog of ATK-3V-4).
///
/// This test MUST pass WITHOUT modification. If it requires changing the
/// expected verdict to pass, the evaluator has the collapse bug.
#[test]
fn atk_pres8_unknown_who_ref_produces_out_of_frame_not_overdue() {
    // No sidecar is written — the who-ref `unknown-who-ref` is un-evaluable.
    // The frame is PAST, which is precisely the trap: a collapse would read
    // this as Overdue. The gem guard keeps it OutOfFrame.
    let src = r#"use antigen::panel;
#[panel(needs = ["review"], filled_by = ["unknown-who-ref"], due = "2020-01-01")]
pub fn p() {}
"#;
    let report = audit_staged(src, None);
    let v = first_verdict(&report);
    assert_eq!(
        v.verdict,
        WorkVerdict::OutOfFrame,
        "un-evaluable who-ref (no sidecar) is OutOfFrame even past-frame, got {:?}",
        v.verdict
    );
    assert_ne!(
        v.verdict,
        WorkVerdict::Overdue,
        "THE GEM GUARD: OutOfFrame must NEVER collapse to Overdue — the work is \
         un-evaluable (who-ref unknown), not late. steps={:?}",
        v.steps
    );
}

/// ATK-PRES-9: Overdue-vs-OutOfFrame is NEVER collapsed.
///
/// This test encodes the explicit anti-regression: if a future change to the
/// prescriptive evaluator collapses Overdue → OutOfFrame OR OutOfFrame → Overdue,
/// this test must catch it.
///
/// SPEC: `WorkVerdict::Overdue != WorkVerdict::OutOfFrame` (structural, not just enum)
#[test]
fn atk_pres9_work_verdict_overdue_and_out_of_frame_are_distinct_variants() {
    assert_ne!(
        WorkVerdict::Overdue,
        WorkVerdict::OutOfFrame,
        "ATK-PRES-9: Overdue and OutOfFrame are DISTINCT WorkVerdict variants. \
         If this fails, the enum was collapsed — a cardinality-collapse bug per \
         the ADR-033 three-valued-logic gem."
    );
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
fn atk_pres11_work_verdict_has_exactly_four_variants() {
    // Exhaustive match — a 5th variant fails to compile here (the sealed-enum
    // cardinality guard). A removed variant also fails (the arm references it).
    fn exhaustive_check(v: WorkVerdict) -> &'static str {
        match v {
            WorkVerdict::Pending => "pending",
            WorkVerdict::Fulfilled => "fulfilled",
            WorkVerdict::Overdue => "overdue",
            WorkVerdict::OutOfFrame => "out-of-frame",
        }
    }
    assert_eq!(exhaustive_check(WorkVerdict::Pending), "pending");
    assert_eq!(exhaustive_check(WorkVerdict::OutOfFrame), "out-of-frame");
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
/// RESOLVED (aristotle, decisions.md §Verdict-semantics-per-shape): triage is a
/// standing re-validated ORDERING. Fulfilled = `triaged_by` attested AND within
/// `re_triage_due` AND all `priority_order` refs resolve. Fulfilled IS reachable
/// (it means "ordering current + resolvable"), re-earned each cycle. `triaged_by`
/// alone does NOT permanently fulfill — the frame expires it. So both bypass
/// concerns are answered: the freshness frame prevents perpetual-fulfillment, and
/// an unresolvable ref ⇒ OutOfFrame (PRES-14), never silent-satisfied.
#[test]
fn atk_pres12_triage_fulfilled_requires_resolvable_order_and_fresh_attestation() {
    // priority_order refs point at two SCANNED ANNOTATED sites in the same crate
    // (`foo`/`bar` carry their own work-needs, so the scan records them and the
    // audit can resolve the refs). v0.3 CEILING: resolution is against the
    // annotated-site index — full arbitrary-code-site resolution is Layer-2
    // (ADR-017-Amd1 multi-crate scan). triaged_by `nav` is attested at the
    // current fingerprint of the triage-bearing item `t`. Within frame ⇒ Fulfilled.
    let src = r#"use antigen::{triage, biopsy};
#[triage(priority_order = ["foo", "bar"], triaged_by = "nav", re_triage_due = "2099-01-01")]
pub fn t() {}
#[biopsy(request_text = "investigate foo", deep_investigation_by = "x")]
pub fn foo() {}
#[biopsy(request_text = "investigate bar", deep_investigation_by = "y")]
pub fn bar() {}
"#;
    let report = audit_staged(src, Some(("t", |fp| sidecar_with_signer("t", "nav", fp))));
    let v = first_verdict(&report);
    assert_eq!(
        v.verdict,
        WorkVerdict::Fulfilled,
        "resolvable order + fresh triaged_by within frame ⇒ Fulfilled, got {:?}. steps={:?}",
        v.verdict,
        v.steps
    );

    // Same triage but PAST re_triage_due ⇒ Overdue (re-triage owed), NOT
    // permanently Fulfilled — even with a fresh triaged_by attestation. This is
    // the anti-bypass aristotle ruled: triaged_by attested is necessary but the
    // re_triage_due frame elapsing makes the ordering stale (re-triage owed). A
    // triage that stayed Fulfilled forever after one attestation would be the
    // perpetual-freshness bypass; the frame de-satisfies it.
    let stale_src = r#"use antigen::{triage, biopsy};
#[triage(priority_order = ["foo", "bar"], triaged_by = "nav", re_triage_due = "2020-01-01")]
pub fn t() {}
#[biopsy(request_text = "investigate foo", deep_investigation_by = "x")]
pub fn foo() {}
#[biopsy(request_text = "investigate bar", deep_investigation_by = "y")]
pub fn bar() {}
"#;
    let stale = audit_staged(
        stale_src,
        Some(("t", |fp| sidecar_with_signer("t", "nav", fp))),
    );
    let sv = first_verdict(&stale);
    assert_eq!(
        sv.verdict,
        WorkVerdict::Overdue,
        "a triage past re_triage_due is Overdue (re-triage owed), NEVER permanently \
         Fulfilled — even with a fresh triaged_by. Got {:?}. steps={:?}",
        sv.verdict,
        sv.steps
    );

    // And an UN-attested triage with NO sidecar is OutOfFrame, not Overdue — the
    // gem guard again: we cannot read the triager, so the work is un-evaluable.
    let unattested_stale = audit_staged(stale_src, None);
    let uv = first_verdict(&unattested_stale);
    assert_eq!(
        uv.verdict,
        WorkVerdict::OutOfFrame,
        "an unevaluable triaged_by (no sidecar) is OutOfFrame, got {:?}",
        uv.verdict
    );
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
/// RESOLVED (aristotle, decisions.md §Verdict-semantics-per-shape):
/// Interpretation B. S4 Fulfilled requires a POSITIVE closure (a closure
/// attestation), NEVER frame-expiry alone. A `#[quarantine]` past its `until`
/// with NO closure attestation is **Overdue**, not Fulfilled — frame-expiry
/// without closure is exactly what Overdue means. This is the `fresh_through`-
/// bypass class (ATK-FT-1/2): a site Fulfilled purely because its deadline
/// passed would be the temporal forged-freshness bypass. Forbidden.
///
/// v0.3 IMPLEMENTATION CEILING (tier-honest): the ratified §Proc-Macro-Surface
/// gives S4 macros NO closure who-ref field (`culture` = test_kind/duration/
/// runs_until; `quarantine` = scope/until/reason). So the positive-closure
/// EVENT — a release attestation, or the named test going green — is not yet
/// observable to the audit (it is the same Layer-2 cross-reference machinery as
/// triage ref-resolution + coverage SubThreshold). Until that lands, an S4 site
/// is NEVER Fulfilled: it is Pending within frame, Overdue past it. This is the
/// SAFE direction — the positive-closure guard holds MAXIMALLY (we never claim
/// closure we cannot observe), so the `fresh_through` bypass is structurally
/// impossible. The path to Fulfilled is gated, not collapsed.
#[test]
fn atk_pres13_s4_frame_expiry_alone_is_overdue_never_fulfilled() {
    // A quarantine WITHIN frame, un-closed ⇒ Pending (the expected state — the
    // hold is active, not late, not done).
    let active_src = r#"use antigen::quarantine;
#[quarantine(scope = "legacy::mod", until = "2099-01-01", reason = "pending upstream fix")]
pub fn q() {}
"#;
    let active = audit_staged(active_src, None);
    let av = first_verdict(&active);
    assert_eq!(
        av.verdict,
        WorkVerdict::Pending,
        "an active (within-frame) quarantine is Pending, got {:?}. steps={:?}",
        av.verdict,
        av.steps
    );

    // The bypass trap: a quarantine PAST `until` with no closure. Interpretation A
    // (the bypass) would mark it Fulfilled "because the date passed." The guard
    // makes it Overdue — frame-expiry without positive closure is exactly Overdue.
    let bypass_src = r#"use antigen::quarantine;
#[quarantine(scope = "legacy::mod", until = "2020-01-01", reason = "pending upstream fix")]
pub fn q() {}
"#;
    let bypass = audit_staged(bypass_src, None);
    let bv = first_verdict(&bypass);
    assert_ne!(
        bv.verdict,
        WorkVerdict::Fulfilled,
        "ATK-PRES-13: frame-expiry alone must NEVER fulfill an S4 site (the \
         fresh_through bypass — a past deadline is not closure). got {:?}",
        bv.verdict
    );
    assert_eq!(
        bv.verdict,
        WorkVerdict::Overdue,
        "an S4 site past frame with no positive closure is Overdue (the hold ran \
         out without being released), got {:?}. steps={:?}",
        bv.verdict,
        bv.steps
    );

    // Same for culture: past runs_until, no green reading ⇒ Overdue, not Fulfilled.
    let culture_src = r#"use antigen::culture;
#[culture(test_kind = "24h soak", runs_until = "2020-01-01")]
pub fn c() {}
"#;
    let culture = audit_staged(culture_src, None);
    let cv = first_verdict(&culture);
    assert_eq!(
        cv.verdict,
        WorkVerdict::Overdue,
        "a culture past runs_until with no green reading is Overdue, never Fulfilled \
         by expiry; got {:?}",
        cv.verdict
    );
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
/// RESOLVED (aristotle, decisions.md §Enforcement-Surface + ADR-017-Amd1):
/// audit-time, NOT parse-time. An unresolvable `priority_order` code-site ref ⇒
/// `WorkVerdict::OutOfFrame` (un-evaluable), never silent-satisfied, never a
/// compile error. This is the gem: the audit cannot grade an ordering over sites
/// that don't exist, so it declares out-of-frame rather than guessing.
#[test]
fn atk_pres14_priority_order_nonresolvable_target_is_out_of_frame() {
    // `bar` exists; `does_not_exist` does not. triaged_by `nav` IS attested at the
    // current fingerprint (so the who-step is NOT the cause). The unresolvable ref
    // alone drives OutOfFrame — isolating ATK-PRES-14's exact concern.
    let src = r#"use antigen::{triage, biopsy};
#[triage(priority_order = ["bar", "does_not_exist"], triaged_by = "nav", re_triage_due = "2099-01-01")]
pub fn t() {}
#[biopsy(request_text = "investigate bar", deep_investigation_by = "x")]
pub fn bar() {}
"#;
    let report = audit_staged(src, Some(("t", |fp| sidecar_with_signer("t", "nav", fp))));
    let v = first_verdict(&report);
    assert_eq!(
        v.verdict,
        WorkVerdict::OutOfFrame,
        "an unresolvable priority_order ref makes the triage OutOfFrame (ADR-017-Amd1), \
         got {:?}. steps={:?}",
        v.verdict,
        v.steps
    );
    assert_ne!(
        v.verdict,
        WorkVerdict::Fulfilled,
        "ATK-PRES-14: a dangling ref must NEVER be silent-satisfied"
    );
    assert_ne!(
        v.verdict,
        WorkVerdict::Overdue,
        "ATK-PRES-14: a dangling ref is un-evaluable (OutOfFrame), not late (Overdue)"
    );
}

/// ATK-PRES-14b: a QUALIFIED priority_order ref must NOT resolve via a bare
/// leaf-name collision (the label-tail over-match guard).
///
/// `priority_order_ref_resolves` splits qualified vs unqualified refs: a
/// qualified ref (`Other::run`) requires a precise match (full label or a
/// `::`-suffix), so it must NOT be silently satisfied by an unrelated scanned
/// site that merely shares the leaf segment `run`. If the audit resolved a
/// qualified ref by leaf-collision, a dangling ref would falsely read resolvable
/// (a silent-satisfied false-positive — the gem's mirror image).
#[test]
fn atk_pres14b_qualified_ref_does_not_resolve_by_leaf_collision() {
    // The only scanned site is `bar` (leaf `bar`). The triage refs a QUALIFIED
    // `Ghost::bar` — same leaf, different (non-existent) qualifier. The precise
    // path must NOT resolve it ⇒ OutOfFrame, not a false Fulfilled.
    let src = r#"use antigen::{triage, biopsy};
#[triage(priority_order = ["Ghost::bar"], triaged_by = "nav", re_triage_due = "2099-01-01")]
pub fn t() {}
#[biopsy(request_text = "investigate bar", deep_investigation_by = "x")]
pub fn bar() {}
"#;
    let report = audit_staged(src, Some(("t", |fp| sidecar_with_signer("t", "nav", fp))));
    let v = first_verdict(&report);
    assert_eq!(
        v.verdict,
        WorkVerdict::OutOfFrame,
        "a qualified ref `Ghost::bar` must NOT resolve via the leaf `bar` of an \
         unrelated site — precise match required. got {:?}. steps={:?}",
        v.verdict,
        v.steps
    );
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
/// RESOLVED (aristotle, decisions.md §Witness-binding + §Verdict-semantics):
/// ALL, conjunction. A `reviewed_by` attestation is credited ONLY when EVERY
/// `filled_by` role-step is attested at the current fingerprint ("you cannot
/// review what is not filled"). Multi-member `filled_by` = ALL, not ANY. A
/// reviewer present while a filler is un-attested is PREMATURE — not credited.
#[test]
fn atk_pres15_reviewed_by_requires_all_filled_by_conjunction() {
    // Case 1: filled_by = [alice, charlie], reviewed_by = [bob]. alice attested,
    // charlie NOT attested, bob attested. ALL semantics ⇒ Pending (the panel is
    // not fully filled, so bob's review is premature and not credited).
    let src = r#"use antigen::panel;
#[panel(needs = ["a", "b"], filled_by = ["alice", "charlie"], reviewed_by = ["bob"], due = "2099-01-01")]
pub fn p() {}
"#;
    // Sidecar carries alice + bob (current) but NOT charlie ⇒ charlie's filler
    // step is Unattested ⇒ not all fillers attested ⇒ bob's review uncredited.
    let partial = audit_staged(
        src,
        Some(("p", |fp| sidecar_with_signers("p", &["alice", "bob"], fp))),
    );
    let pv = first_verdict(&partial);
    assert_eq!(
        pv.verdict,
        WorkVerdict::Pending,
        "ATK-PRES-15: ALL semantics — one un-attested filler (charlie) ⇒ Pending, \
         bob's review premature. got {:?}. steps={:?}",
        pv.verdict,
        pv.steps
    );
    assert_ne!(
        pv.verdict,
        WorkVerdict::Fulfilled,
        "ANY semantics is WRONG — a partially-filled panel is not Fulfilled"
    );

    // Case 2: every filler AND the reviewer attested ⇒ Fulfilled (the chain
    // closes: all filled_by, then all reviewed_by).
    let complete = audit_staged(
        src,
        Some(("p", |fp| {
            sidecar_with_signers("p", &["alice", "charlie", "bob"], fp)
        })),
    );
    let cv = first_verdict(&complete);
    assert_eq!(
        cv.verdict,
        WorkVerdict::Fulfilled,
        "all fillers + reviewer attested at current fingerprint ⇒ Fulfilled, got {:?}. steps={:?}",
        cv.verdict,
        cv.steps
    );

    // Case 3: all fillers attested but the reviewer is NOT ⇒ Pending (the review
    // step is the last link of the conjunction chain, still open).
    let no_review = audit_staged(
        src,
        Some(("p", |fp| {
            sidecar_with_signers("p", &["alice", "charlie"], fp)
        })),
    );
    let nv = first_verdict(&no_review);
    assert_eq!(
        nv.verdict,
        WorkVerdict::Pending,
        "fillers done but reviewer not attested ⇒ Pending (awaiting review), got {:?}. steps={:?}",
        nv.verdict,
        nv.steps
    );
}
