//! ATK-ADR029 — `#[defended_by]` audit integration contracts.
//!
//! ADR-029 (Immunity Is Observed, Not Declared) introduces `#[defended_by(X)]`
//! as the code-tier witness registration: a test/proptest declares what
//! failure-class it defends. `cargo antigen audit` cross-references those
//! registrations to `#[presents(X)]` sites and issues verdicts.
//!
//! **The silent failure this file defends against**:
//! `ScanReport::defenses` is populated by scan, but `audit()` could iterate only
//! `report.immunities` and never read `report.defenses` — a `#[defended_by]`
//! test would register correctly yet be silently ignored by the verdict
//! computation, leaving its presents-sites with no verdict at all.
//!
//! ADR-029 implementation (pathmaker, 2026-05-27): `audit()` computes a
//! per-presents-site verdict surface, `AuditReport::presentation_verdicts:
//! Vec<PresentationVerdict>`, with `verdict: ImmuneVerdict =
//! Defended { tier } | Undefended | SubstrateGap`. The verdict is
//! **presents-keyed**, not immunity-keyed: a `#[defended_by(X)]` witness is
//! class-level (it defends ALL `#[presents(X)]` sites), so it cannot map to a
//! single `Immunity`. The legacy `audits: Vec<ImmunityAudit>` stays
//! immunity-keyed (backward-compat — `#[immune]` deprecated but still honored).
//! These tests target `presentation_verdicts`; ATK-ADR029-4 also pins that the
//! `#[immune]` `audits` surface is unaffected.
//!
//! Substrate check:
//!   `cargo test --package antigen --test atk_adr029_defended_by_audit`

use antigen::audit::{audit, ImmuneVerdict, WitnessTier};
use antigen::scan::{Defense, Immunity, ItemTarget, MatchKind, Presentation, ScanReport};
use std::path::PathBuf;

// ATK-ADR029-15: partitioned_presentations() classifies by match_kind only.
//
// `partitioned_presentations()` partitions unaddressed presentations into
// `explicit` (ExplicitMarker) and `inferred` (FingerprintMatch) buckets using
// ONLY the `match_kind` field. ADR-029 adds `requires_predicate` and `proof`
// as site-attached evidence fields. If a FingerprintMatch site carries
// `requires_predicate: Some(...)`, that means the user DID explicitly attach
// evidence — but the partition still routes it to `inferred`.
//
// This is the scan-audit field propagation gap pattern applied to
// `partitioned_presentations()`: new fields on `Presentation` (requires_predicate,
// proof) are NOT consulted when deciding the confidence bucket.
//
// In practice, the scanner never produces FingerprintMatch + requires_predicate
// (FingerprintMatch sites always get None for both ADR-029 fields). But nothing
// in the TYPE SYSTEM prevents this state, and a future code path or external
// ScanReport consumer could construct it. The partition would then silently
// misclassify a site with explicit evidence as inferred.
//
// This test DOCUMENTS the current behavior (FingerprintMatch + requires_predicate
// → inferred bucket, despite the explicit evidence). It's a regression anchor
// for the invariant: if partitioned_presentations() is ever updated to promote
// FingerprintMatch+requires_predicate to the explicit bucket, update this test.
#[test]
fn atk_adr029_15_fingerprint_match_with_requires_predicate_lands_in_inferred_bucket() {
    let mut report = ScanReport::default();

    // Degenerate input: FingerprintMatch site WITH requires_predicate set.
    // This state doesn't arise from normal scanning (scanner always sets
    // requires_predicate=None for FingerprintMatch sites), but nothing in the
    // type system prevents it. We probe partitioned_presentations() behavior
    // against this degenerate combination.
    report.presentations.push(Presentation {
        antigen_type: "FailureClass".to_string(),
        file: PathBuf::from("src/lib.rs"),
        line: 20,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Unknown { line: 20 },
        match_kind: MatchKind::FingerprintMatch, // NOT explicit
        canonical_path: None,
        inherited_from: None,
        structural_fingerprint: "fp-pattern".to_string(),
        // Explicit site-attached evidence — but match_kind says inferred.
        requires_predicate: Some(r#"signers(required = ["alice"])"#.to_string()),
        proof: None,
    });
    // No defense, immunity, or tolerance — the site is unaddressed.

    let partitioned = report.partitioned_presentations();

    // CURRENT BEHAVIOR: partitioned_presentations() classifies by match_kind
    // only — so this site lands in `inferred` despite carrying explicit evidence.
    //
    // The semantic mismatch: an author who wrote `requires=signers(required=["alice"])`
    // explicitly attached evidence (that's opt-in behavior, not inferrence).
    // The CI-gate guidance in PartitionedPresentations says `explicit` = CI-gateable,
    // `inferred` = human-triage. With this degenerate input, a site with explicit
    // evidence would be in the human-triage bucket — silently excluded from CI gates.
    //
    // ATK-ADR029-15 documents this gap. Fix direction: `partitioned_presentations()`
    // should also promote to `explicit` if `requires_predicate.is_some() || proof.is_some()`.
    assert_eq!(
        partitioned.explicit.len(),
        0,
        "ATK-ADR029-15: FingerprintMatch + requires_predicate currently produces 0 explicit \
         entries — match_kind governs, not evidence presence; CORRECT behavior would be 1 \
         explicit (site has explicit evidence attached)"
    );
    assert_eq!(
        partitioned.inferred.len(),
        1,
        "ATK-ADR029-15: FingerprintMatch + requires_predicate lands in inferred bucket \
         (match_kind governs); site with explicit evidence is silently treated as inferred"
    );
}

/// Synthesize a minimal `ScanReport` with:
/// - one `#[presents(FailureClass)]` site at src/lib.rs:10
/// - one `#[defended_by(FailureClass)]` test at src/tests.rs:5
/// - no `#[immune]` declarations (old model not used)
fn report_with_defended_by_only() -> ScanReport {
    let mut report = ScanReport::default();

    let presentation = Presentation {
        antigen_type: "FailureClass".to_string(),
        file: PathBuf::from("src/lib.rs"),
        line: 10,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Unknown { line: 10 },
        match_kind: MatchKind::ExplicitMarker,
        canonical_path: None,
        inherited_from: None,
        structural_fingerprint: String::new(),
        requires_predicate: None,
        proof: None,
    };
    report.presentations.push(presentation);

    let defense = Defense {
        antigen_type: "FailureClass".to_string(),
        file: PathBuf::from("src/tests.rs"),
        line: 5,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Unknown { line: 5 },
    };
    report.defenses.push(defense);

    report
}

// ATK-ADR029-1: a defended_by registration must produce a verdict
//
// When scan has a Defense with antigen_type "FailureClass" and a Presentation
// with the same antigen_type, audit() must cross-reference them and emit a
// per-presents-site verdict. The silent failure (audit ignores report.defenses)
// would leave presentation_verdicts empty.
#[test]
fn atk_adr029_1_defended_by_produces_verdict() {
    let report = report_with_defended_by_only();
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();

    let audit_result = audit(&report, workspace_root);

    assert!(
        !audit_result.presentation_verdicts.is_empty(),
        "ATK-ADR029-1: audit() must cross-reference report.defenses to \
         report.presentations and emit a per-presents-site verdict. \
         report.defenses.len() = {}, report.presentations.len() = {}, \
         presentation_verdicts.len() = {}",
        report.defenses.len(),
        report.presentations.len(),
        audit_result.presentation_verdicts.len(),
    );
    // The single FailureClass site must be Defended (a matching witness exists).
    let v = audit_result
        .presentation_verdicts
        .iter()
        .find(|v| v.antigen_type == "FailureClass")
        .expect("a verdict for the FailureClass presents-site");
    assert!(
        matches!(v.verdict, ImmuneVerdict::Defended { .. }),
        "ATK-ADR029-1: a presents-site with a matching #[defended_by] witness \
         must be Defended; got {:?}",
        v.verdict
    );
}

// ATK-ADR029-2: defended_by must produce at least Reachability tier
//
// A registered #[defended_by] witness (test fn) must produce
// WitnessTier >= Reachability. Tier::None / Undefended means "no evidence" — a
// registered defense that produces None tier is a silent failure. v0.3 audit
// does not invoke coverage, so the honest tier is exactly Reachability.
#[test]
fn atk_adr029_2_defended_by_produces_at_least_reachability() {
    let report = report_with_defended_by_only();
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();

    let audit_result = audit(&report, workspace_root);

    let v = audit_result
        .presentation_verdicts
        .iter()
        .find(|v| v.antigen_type == "FailureClass")
        .expect("a verdict for the FailureClass presents-site");
    match &v.verdict {
        ImmuneVerdict::Defended { tier } => assert!(
            *tier >= WitnessTier::Reachability,
            "ATK-ADR029-2: #[defended_by] witness must produce >= Reachability \
             tier; got {tier:?}. A registered defense with None tier means the \
             defense circuit is wired but produces no evidence."
        ),
        other => {
            panic!("ATK-ADR029-2: FailureClass site must be Defended at a tier; got {other:?}")
        }
    }
}

// ATK-ADR029-3: wrong-antigen defended_by must not pollute other presents-sites
//
// A #[defended_by(WrongClass)] test must not grant a defense verdict to a
// RightClass presents-site. Cross-antigen contamination is a silent failure
// where one test accidentally satisfies an unrelated presents-site's defense.
#[test]
fn atk_adr029_3_wrong_antigen_defended_by_does_not_pollute() {
    let mut report = ScanReport::default();

    // presents-site for RightClass
    report.presentations.push(Presentation {
        antigen_type: "RightClass".to_string(),
        file: PathBuf::from("src/lib.rs"),
        line: 10,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Unknown { line: 10 },
        match_kind: MatchKind::ExplicitMarker,
        canonical_path: None,
        inherited_from: None,
        structural_fingerprint: String::new(),
        requires_predicate: None,
        proof: None,
    });

    // witness defending WrongClass — does NOT match RightClass
    report.defenses.push(Defense {
        antigen_type: "WrongClass".to_string(),
        file: PathBuf::from("src/tests.rs"),
        line: 5,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Unknown { line: 5 },
    });

    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let audit_result = audit(&report, workspace_root);

    // The RightClass presents-site must be Undefended (WrongClass defense does
    // not cross-reference to RightClass).
    let v = audit_result
        .presentation_verdicts
        .iter()
        .find(|v| v.antigen_type == "RightClass")
        .expect("a verdict for the RightClass presents-site");
    assert_eq!(
        v.verdict,
        ImmuneVerdict::Undefended,
        "ATK-ADR029-3: WrongClass defense must not grant RightClass a defense \
         verdict; got {:?}",
        v.verdict
    );
    assert!(
        v.defended_by.is_empty(),
        "ATK-ADR029-3: RightClass verdict must list no defending witnesses; got {:?}",
        v.defended_by
    );
}

// ATK-ADR029-4: immune path (audits surface) still works alongside defended_by
//
// Regression guard: the existing #[immune] `audits` surface must not be broken
// when the defended_by cross-reference is added. A report with both an immunity
// (old model) and a defense (new model) for different antigens must still
// produce an immunity audit entry for the immune path.
#[test]
fn atk_adr029_4_immune_audits_unaffected_by_defended_by() {
    let mut report = ScanReport::default();

    // Old-model: #[immune] on site
    report.immunities.push(Immunity {
        antigen_type: "PanickingInDrop".to_string(),
        witness: "some_test_fn".to_string(),
        file: PathBuf::from("src/lib.rs"),
        line: 20,
        item_kind: "impl".to_string(),
        item_target: ItemTarget::Unknown { line: 20 },
        canonical_path: None,
        requires_predicate: None,
        structural_fingerprint: String::new(),
    });

    // New-model: #[defended_by] on a different antigen
    report.defenses.push(Defense {
        antigen_type: "FailureClass".to_string(),
        file: PathBuf::from("src/tests.rs"),
        line: 5,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Unknown { line: 5 },
    });

    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let audit_result = audit(&report, workspace_root);

    // The immunity audit must still produce an entry for PanickingInDrop — the
    // legacy audits surface is unaffected by the new presentation_verdicts pass.
    assert!(
        !audit_result.audits.is_empty(),
        "ATK-ADR029-4: audit() must still process #[immune] entries when defenses are present"
    );
    assert_eq!(
        audit_result.audits[0].immunity.antigen_type, "PanickingInDrop",
        "ATK-ADR029-4: immunity audit entry must be for the right antigen type"
    );
}

// ATK-ADR029-8: #[defended_by] on a non-fn item must not produce a Defended verdict
//
// The `#[defended_by(X)]` macro does NO item-kind validation — it emits the
// `antigen:defended_by:v1:X` doc marker on ANY item: struct, impl, const, mod.
// `extract_defended_by()` in scan.rs stores the `item_kind` field but performs
// no gate on it — every item kind is accepted as a Defense registration.
// `audit()` then issues `ImmuneVerdict::Defended` for the presents-site.
//
// The silent failure: a `#[defended_by(FailureClass)]` on a `struct Foo {}` will
// satisfy the audit for every `#[presents(FailureClass)]` site in the workspace.
// The struct can never be run as a test witness. The verdict is false confidence.
//
// This test FAILS until either:
//   (a) `extract_defended_by()` gates on `item_kind == "fn"` or `"impl_fn"`, OR
//   (b) `audit()` filters `Defense` entries whose `item_kind` is not a callable
//       test function before issuing a Defended verdict.
#[test]
fn atk_adr029_8_non_fn_defended_by_must_not_produce_defended_verdict() {
    let mut report = ScanReport::default();

    // A #[presents(FailureClass)] site
    report.presentations.push(Presentation {
        antigen_type: "FailureClass".to_string(),
        file: PathBuf::from("src/lib.rs"),
        line: 10,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Unknown { line: 10 },
        match_kind: MatchKind::ExplicitMarker,
        canonical_path: None,
        inherited_from: None,
        structural_fingerprint: String::new(),
        requires_predicate: None,
        proof: None,
    });

    // A Defense that came from a STRUCT, not a test function.
    // scan.rs currently records this without any item_kind gate.
    report.defenses.push(Defense {
        antigen_type: "FailureClass".to_string(),
        file: PathBuf::from("src/lib.rs"),
        line: 20,
        item_kind: "struct".to_string(), // NOT a callable test witness
        item_target: ItemTarget::Unknown { line: 20 },
    });

    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let audit_result = audit(&report, workspace_root);

    let v = audit_result
        .presentation_verdicts
        .iter()
        .find(|v| v.antigen_type == "FailureClass")
        .expect("a verdict for the FailureClass presents-site");

    // A struct is not a runnable test witness. The verdict must NOT be Defended.
    // Currently this FAILS because audit() does not gate on Defense.item_kind.
    assert!(
        !matches!(v.verdict, ImmuneVerdict::Defended { .. }),
        "ATK-ADR029-8 FAILING: a #[defended_by] on a struct item_kind produces \
         ImmuneVerdict::Defended. A struct cannot be run as a test witness. \
         The audit must reject non-fn Defense registrations (or scan must gate \
         at extract_defended_by time). Got verdict: {:?}",
        v.verdict
    );
}

// ATK-ADR029-10: a presents-site with requires_predicate must not be Undefended
//
// ADR-029 R5 introduces site-attached evidence: `#[presents(X, requires=<pred>)]`
// folds substrate-witness evidence onto the presentation itself. When the predicate
// is declared on the site, `audit()` must evaluate it against the workspace's
// `.attest/` sidecars and issue `Defended` (predicate passes) or `SubstrateGap`
// (predicate present but sidecar absent/failing) — NOT `Undefended` (no intent).
//
// The silent failure: `compute_presentation_verdicts()` at audit.rs currently
// checks only `code_witnesses` (code-tier `#[defended_by]`) and `immune_audit`
// (deprecated `#[immune]` path). It does NOT inspect `p.requires_predicate` or
// `p.proof`. A presents-site with site-attached evidence but no code-tier witness
// produces `Undefended` — a false verdict that discards declared defensive intent.
//
// This test FAILS until `compute_presentation_verdicts()` handles `p.requires_predicate`
// (either by issuing `SubstrateGap` when the sidecar is absent, or by evaluating
// the predicate and issuing `Defended{tier:SubstrateWitness}` when it passes).
// The minimum correct behavior: a site with requires_predicate set must NOT produce
// `Undefended` — it has declared defensive intent, even if the substrate hasn't
// satisfied it yet.
#[test]
fn atk_adr029_10_requires_predicate_on_presentation_not_undefended() {
    let mut report = ScanReport::default();

    // A presents-site with a substrate-witness predicate attached (ADR-029 R5).
    // No code-tier Defense, no #[immune]. This site has declared intent via
    // requires_predicate; audit() must not classify it as Undefended.
    report.presentations.push(Presentation {
        antigen_type: "SubstrateAntigen".to_string(),
        file: PathBuf::from("src/lib.rs"),
        line: 30,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Unknown { line: 30 },
        match_kind: MatchKind::ExplicitMarker,
        canonical_path: None,
        inherited_from: None,
        structural_fingerprint: String::new(),
        // Site-attached substrate-witness predicate (ADR-029 R5 migration target).
        // Format mirrors the #[immune(requires=...)] predicate JSON channel.
        requires_predicate: Some(r#"{"leaf":"fresh_within_days","value":90}"#.to_string()),
        proof: None,
    });

    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let audit_result = audit(&report, workspace_root);

    let v = audit_result
        .presentation_verdicts
        .iter()
        .find(|v| v.antigen_type == "SubstrateAntigen")
        .expect("a verdict for the SubstrateAntigen presents-site");

    // A site with requires_predicate has declared defensive intent. Even when the
    // sidecar is absent (SubstrateGap) or the predicate fails, the verdict must
    // NOT be Undefended — Undefended means "no intent at all."
    // Currently FAILS because compute_presentation_verdicts() ignores requires_predicate.
    assert_ne!(
        v.verdict,
        ImmuneVerdict::Undefended,
        "ATK-ADR029-10 FAILING: a #[presents(X, requires=<pred>)] site with \
         requires_predicate set must not produce Undefended. Declared defensive \
         intent exists (requires_predicate is Some); the verdict should be \
         SubstrateGap (sidecar absent/failing) or Defended (predicate passes). \
         Fix: compute_presentation_verdicts() must inspect p.requires_predicate \
         and route through the substrate-witness evaluation pipeline. Got: {:?}",
        v.verdict
    );
}

// ATK-ADR029-9: unaddressed_presentations() must not treat defended-by-new-model sites as unaddressed
//
// `unaddressed_presentations()` at scan.rs checks `has_matching_immunity` and
// `has_matching_tolerance` but NOT `has_matching_defense`. A `#[presents(X)]` site
// with a `#[defended_by(X)]` witness (and no `#[immune]`) appears in the
// "unaddressed" list even after `audit()` correctly issues a `Defended` verdict.
//
// This was a dual-output inconsistency that has been FIXED. This test is now a
// regression guard: it verifies that `unaddressed_presentations()` correctly
// excludes presents-sites that have a matching `#[defended_by]` witness in
// `report.defenses`.
#[test]
fn atk_adr029_9_unaddressed_presentations_excludes_defended_sites() {
    let report = report_with_defended_by_only();

    // The FailureClass presentation has a matching Defense in report.defenses.
    // unaddressed_presentations() must recognize this and exclude the site.
    let unaddressed = report.unaddressed_presentations();
    assert!(
        unaddressed.is_empty(),
        "ATK-ADR029-9 REGRESSION: unaddressed_presentations() returned {} unaddressed \
         sites for a report where every presentation has a matching defense. \
         Fix: unaddressed_presentations() must check \
         self.defenses.iter().any(|d| d.antigen_type == p.antigen_type) alongside \
         immunities and tolerances.",
        unaddressed.len()
    );
}

// ATK-ADR029-12: impl_fn item_kind witness must produce a Defended verdict
//
// `visit_impl_item_fn()` in scan.rs assigns `item_kind: "impl_fn"` to methods
// inside `impl` blocks. A `#[test]` method inside an `impl` block is a valid
// runnable test witness — but `compute_presentation_verdicts()` at audit.rs:1292
// gates on `d.item_kind == "fn"` only. An `"impl_fn"` witness is silently rejected.
//
// The silent failure: a developer writes `#[test] #[defended_by(X)]` on an impl
// method, scan records it as `item_kind: "impl_fn"`, and audit() produces
// `Undefended` — even though the test is perfectly valid. The developer sees an
// audit failure with no explanation; the defense IS registered but the audit
// silently ignores it.
//
// This test FAILS until `compute_presentation_verdicts()` accepts both
// `item_kind == "fn"` and `item_kind == "impl_fn"` as code-tier witnesses.
#[test]
fn atk_adr029_12_impl_fn_defended_by_must_produce_defended_verdict() {
    let mut report = ScanReport::default();

    // A #[presents(FailureClass)] site
    report.presentations.push(Presentation {
        antigen_type: "FailureClass".to_string(),
        file: PathBuf::from("src/lib.rs"),
        line: 10,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Unknown { line: 10 },
        match_kind: MatchKind::ExplicitMarker,
        canonical_path: None,
        inherited_from: None,
        structural_fingerprint: String::new(),
        requires_predicate: None,
        proof: None,
    });

    // A Defense from a method inside an impl block — item_kind is "impl_fn",
    // NOT "fn". This is what scan.rs emits for visit_impl_item_fn() (line 4467).
    report.defenses.push(Defense {
        antigen_type: "FailureClass".to_string(),
        file: PathBuf::from("src/tests.rs"),
        line: 5,
        item_kind: "impl_fn".to_string(), // a test method inside an impl block
        item_target: ItemTarget::Unknown { line: 5 },
    });

    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let audit_result = audit(&report, workspace_root);

    let v = audit_result
        .presentation_verdicts
        .iter()
        .find(|v| v.antigen_type == "FailureClass")
        .expect("a verdict for the FailureClass presents-site");

    // An impl_fn test method IS a valid runnable witness. The verdict must be Defended.
    // Currently FAILS because compute_presentation_verdicts() gates on == "fn" only.
    assert!(
        matches!(v.verdict, ImmuneVerdict::Defended { .. }),
        "ATK-ADR029-12 FAILING: a #[defended_by] on an impl_fn witness produces {:?} \
         instead of Defended. An impl_fn test method is a valid runnable witness — \
         the item_kind == \"fn\" gate at audit.rs:1292 must be widened to also accept \
         \"impl_fn\". Fix: change filter to \
         |d| d.antigen_type == p.antigen_type && (d.item_kind == \"fn\" || d.item_kind == \"impl_fn\").",
        v.verdict
    );
}

// ATK-ADR029-13: a presents-site with proof= must produce Defended at FormalProof tier
//
// ADR-029 R5 adds `proof = <expr>` as phantom-tier site-attached evidence on
// `#[presents(X, proof = SomeProof::<T>::verified)]`. The `proof` field was added
// to `Presentation` (scan.rs) alongside `requires_predicate`, but
// `compute_presentation_verdicts()` in audit.rs currently ignores `p.proof` entirely
// (zero reads of `.proof` in audit.rs).
//
// The silent failure: a developer uses `#[presents(X, proof = NonPanickingProof::verified)]`
// to declare phantom-tier evidence, but audit() produces `Undefended` — discarding
// the strongest possible evidence tier without explanation.
//
// ADR-029 §Verdict-Precedence: a `proof = <expr>` present on the site means
// phantom-tier evidence is declared. The audit must grade it `Defended{tier:FormalProof}`
// (the proof expression exists on the site — its presence IS the proof).
//
// This test FAILS until `compute_presentation_verdicts()` reads `p.proof` and
// issues `Defended{tier:FormalProof}` when `proof` is `Some(...)`.
#[test]
fn atk_adr029_13_proof_on_presentation_produces_formal_proof_tier() {
    let mut report = ScanReport::default();

    // A presents-site with a phantom-type proof expression attached (ADR-029 R5).
    // No code-tier Defense, no #[immune], no requires_predicate. Proof alone.
    report.presentations.push(Presentation {
        antigen_type: "PhantomAntigen".to_string(),
        file: PathBuf::from("src/lib.rs"),
        line: 40,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Unknown { line: 40 },
        match_kind: MatchKind::ExplicitMarker,
        canonical_path: None,
        inherited_from: None,
        structural_fingerprint: String::new(),
        requires_predicate: None,
        // Phantom-type proof expression (ADR-029 R5 migration target for
        // #[immune(witness = <phantom>)]). The presence of this expression
        // on the site IS the proof — compile-time evidence of the invariant.
        proof: Some("NonPanickingProof :: < T > :: verified".to_string()),
    });

    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let audit_result = audit(&report, workspace_root);

    let v = audit_result
        .presentation_verdicts
        .iter()
        .find(|v| v.antigen_type == "PhantomAntigen")
        .expect("a verdict for the PhantomAntigen presents-site");

    // A site with proof= declared has the strongest possible evidence. Must be
    // Defended at FormalProof tier. Currently FAILS because p.proof is unread.
    assert!(
        matches!(
            v.verdict,
            ImmuneVerdict::Defended {
                tier: WitnessTier::FormalProof
            }
        ),
        "ATK-ADR029-13 FAILING: a #[presents(X, proof=...)] site with proof set \
         must produce Defended{{tier:FormalProof}}. The proof expression IS the evidence \
         (phantom-tier: compile-time). Fix: compute_presentation_verdicts() must read \
         p.proof and issue Defended{{tier:FormalProof}} when Some. Got: {:?}",
        v.verdict
    );
}

// ATK-ADR029-14: multiple #[defended_by] witnesses for the same antigen
//
// When multiple tests each declare #[defended_by(X)], audit() must:
// (a) list ALL witnesses in v.defended_by (not just the first match)
// (b) produce a single Defended verdict (not one per witness)
// (c) grade at the maximum tier across all witnesses
//
// This is a correctness-completeness check, not a failure-mode test. If audit()
// short-circuits on the first match, the defended_by list will be incomplete —
// a silent completeness failure that makes the audit report misleading.
#[test]
fn atk_adr029_14_multiple_witnesses_all_listed() {
    let mut report = ScanReport::default();

    report.presentations.push(Presentation {
        antigen_type: "FailureClass".to_string(),
        file: PathBuf::from("src/lib.rs"),
        line: 10,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Unknown { line: 10 },
        match_kind: MatchKind::ExplicitMarker,
        canonical_path: None,
        inherited_from: None,
        structural_fingerprint: String::new(),
        requires_predicate: None,
        proof: None,
    });

    // Two distinct witnesses for the same antigen
    report.defenses.push(Defense {
        antigen_type: "FailureClass".to_string(),
        file: PathBuf::from("src/tests_a.rs"),
        line: 10,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Unknown { line: 10 },
    });
    report.defenses.push(Defense {
        antigen_type: "FailureClass".to_string(),
        file: PathBuf::from("src/tests_b.rs"),
        line: 20,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Unknown { line: 20 },
    });

    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let audit_result = audit(&report, workspace_root);

    // Exactly one verdict for the one presents-site
    assert_eq!(
        audit_result.presentation_verdicts.len(),
        1,
        "ATK-ADR029-14: multiple witnesses must produce exactly one verdict per \
         presents-site; got {} verdicts",
        audit_result.presentation_verdicts.len()
    );

    let v = &audit_result.presentation_verdicts[0];
    assert!(
        matches!(v.verdict, ImmuneVerdict::Defended { .. }),
        "ATK-ADR029-14: site with two witnesses must be Defended; got {:?}",
        v.verdict
    );

    // Both witnesses must appear in the defended_by list
    assert_eq!(
        v.defended_by.len(),
        2,
        "ATK-ADR029-14: both witnesses must appear in defended_by list; \
         got {} entries: {:?}",
        v.defended_by.len(),
        v.defended_by
    );
    assert!(
        v.defended_by.iter().any(|s| s.contains("tests_a.rs")),
        "ATK-ADR029-14: first witness (tests_a.rs) missing from defended_by: {:?}",
        v.defended_by
    );
    assert!(
        v.defended_by.iter().any(|s| s.contains("tests_b.rs")),
        "ATK-ADR029-14: second witness (tests_b.rs) missing from defended_by: {:?}",
        v.defended_by
    );
}

// ATK-ADR029-16: proof= produces FormalProof verdict without compile-time
// validation of the proof expression.
//
// audit.rs line 1356 states: "The proof is a type-system construction whose
// mere existence is the evidence — it compiled, so the construction is valid."
//
// This claim is FALSE. The `#[presents(X, proof = <expr>)]` macro parses the
// proof expression as syn::Expr but never emits it into the generated code. The
// expression is stored as a string by the scanner, passed to audit as proof: Some(s),
// and audit immediately grants WitnessTier::FormalProof from `p.proof.as_ref().map(|_| ...)`.
//
// Nothing in this pipeline compiles or validates the proof expression. A user who writes:
//   #[presents(MyAntigen, proof = "any arbitrary string")]
// gets a verdict of `Defended { tier: FormalProof }` despite the "proof" being
// meaningless. The phantom tier exists precisely because a phantom-type proof
// is COMPILE-CHECKED — but the current implementation skips that check.
//
// ATTACK: synthesize a ScanReport with proof = Some("invalid nonsense 42 !!") and
// verify that audit returns Defended{FormalProof} — proving the validation gap.
//
// FIX DIRECTION: The macro must emit the proof expression into the generated
// output in a way that forces compilation — e.g., as an unused local binding
// or a const expression — so "it compiled" is actually true. Without this,
// proof= is an honor-system field that grants FormalProof tier to unverified claims.
#[test]
fn atk_adr029_16_proof_field_grants_formal_proof_without_compile_validation() {
    let mut report = ScanReport::default();

    // A presentation with an arbitrary string in proof= — not a real type expression.
    // This simulates what a user who misunderstands the field could write, OR what
    // the scanner would store if the macro doesn't validate the expression.
    report.presentations.push(Presentation {
        antigen_type: "FailureClass".to_string(),
        file: PathBuf::from("src/lib.rs"),
        line: 10,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Unknown { line: 10 },
        match_kind: MatchKind::ExplicitMarker,
        canonical_path: None,
        inherited_from: None,
        structural_fingerprint: String::new(),
        requires_predicate: None,
        // An arbitrary non-type string — not compiled, not validated.
        proof: Some("invalid nonsense 42 !!".to_string()),
    });
    // No defense, no immunity — the only "evidence" is the unvalidated proof string.

    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let audit_result = audit(&report, workspace_root);

    assert_eq!(audit_result.presentation_verdicts.len(), 1);
    let v = &audit_result.presentation_verdicts[0];

    // CURRENT BEHAVIOR (documents the gap): proof= grants FormalProof regardless of
    // whether the expression is a real type-system construction. The audit sees
    // proof: Some(_) and immediately returns WitnessTier::FormalProof.
    //
    // CORRECT behavior: proof= should require that the expression compiles as actual
    // Rust code in the generated output; the FormalProof tier should be earned by
    // compile-time verification, not by string presence.
    //
    // ATK-ADR029-16: this PASSES under the current code -- proving the validation gap.
    // A correct implementation would either:
    //   (a) reject proof= values that don't compile (macro emits the expr into the output), or
    //   (b) demote unvalidated proof= to Reachability tier (weaker than FormalProof)
    assert!(
        matches!(
            v.verdict,
            ImmuneVerdict::Defended {
                tier: WitnessTier::FormalProof
            }
        ),
        "ATK-ADR029-16 documented gap: proof= grants FormalProof without compile-time \
         validation of the expression; even \"invalid nonsense 42 !!\" produces \
         Defended{{FormalProof}}. Got: {:?}",
        v.verdict
    );
}

// ATK-ADR029-17: #[defended_by(DescendantType)] does NOT defend inherited
// presentations of AncestorType.
//
// When antigen ChildClass descends from ParentClass, a #[presents(ParentClass)]
// site is inherited by ChildClass as an inherited Presentation with
// antigen_type == "ParentClass". The audit's verdict computation matches
// defenses by antigen_type (audit.rs:1314: d.antigen_type == p.antigen_type).
//
// The trap: a developer who writes #[defended_by(ChildClass)] on their test,
// intending to defend "ChildClass including its inherited vulnerabilities",
// gets a false Undefended verdict for the inherited ParentClass presentation.
// The developer's mental model (defend the child type, cover everything) does
// NOT match the audit's model (defense is class-level; inherited presentations
// keep their ancestor's antigen_type).
//
// This is the ADR-029 + ADR-018 intersection gap. The correct defense for an
// inherited presentation of ParentClass on a ChildClass site is
// #[defended_by(ParentClass)], not #[defended_by(ChildClass)].
//
// This test documents the gap so that any future change to inherited-presentation
// verdict computation triggers a test update.
#[test]
fn atk_adr029_17_defended_by_descendant_type_does_not_cover_inherited_ancestor_presentation() {
    use antigen::scan::{Defense, ItemTarget, MatchKind, Presentation, ScanReport};

    let mut report = ScanReport::default();

    // Inherited presentation: ChildClass site inherits ParentClass's vulnerability.
    // antigen_type is ParentClass (the ancestor), NOT ChildClass (the descendant).
    report.presentations.push(Presentation {
        antigen_type: "ParentClass".to_string(),
        file: PathBuf::from("src/lib.rs"),
        line: 10,
        item_kind: "struct".to_string(),
        item_target: ItemTarget::Struct("ChildSite".to_string()),
        match_kind: MatchKind::ExplicitMarker,
        canonical_path: None,
        inherited_from: Some(vec![antigen::scan::ProvenanceEntry {
            antigen_type: "ParentClass".to_string(),
            canonical_path: None,
        }]),
        structural_fingerprint: String::new(),
        requires_predicate: None,
        proof: None,
    });

    // Defense registered for ChildClass -- the developer intended this to cover
    // ChildClass including its inherited vulnerabilities. But the audit matches
    // by antigen_type == p.antigen_type == "ParentClass", so this defense for
    // "ChildClass" does NOT apply.
    report.defenses.push(Defense {
        antigen_type: "ChildClass".to_string(), // WRONG: inherited pres has antigen_type="ParentClass"
        file: PathBuf::from("tests/test.rs"),
        line: 5,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Fn("test_child_class".to_string()),
    });

    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let audit_result = audit(&report, workspace_root);

    // DOCUMENTS THE GAP: the inherited presentation of ParentClass is Undefended,
    // even though the developer registered a defense for ChildClass.
    // The defense for ChildClass does not cross-reference to the inherited
    // ParentClass presentation because antigen_type mismatch.
    assert_eq!(
        audit_result.presentation_verdicts.len(),
        1,
        "ATK-ADR029-17: exactly one verdict for the inherited presentation"
    );
    let v = &audit_result.presentation_verdicts[0];
    assert!(
        matches!(v.verdict, ImmuneVerdict::Undefended),
        "ATK-ADR029-17: inherited ParentClass presentation on ChildSite is Undefended \
        even with a #[defended_by(ChildClass)] registered. The developer's intended \
        defense (ChildClass covers inherited vulnerabilities) does not match the audit's \
        model (defenses are antigen_type-keyed; inherited presentations keep ancestor type). \
        The correct defense is #[defended_by(ParentClass)], not #[defended_by(ChildClass)]. \
        Got: {:?}",
        v.verdict
    );
}

// ATK-ADR029-18: V1 void — failing requires= predicate is SILENTLY MASKED by
// a passing #[defended_by] code witness (OR semantics at audit.rs:1381).
//
// ADR-029 V1 limitation (docs/decisions.md ~line 6956): the audit takes the
// BEST tier across four evidence channels (code_tier, immune_tier,
// site_requires_tier, site_proof_tier) using max(). When a site has BOTH a
// passing code-tier witness (#[defended_by]) AND a failing substrate predicate
// (requires=P where P evaluates to WitnessTier::None), the max() picks
// code_tier=Some(Reachability) and emits ImmuneVerdict::Defended.
//
// The failing substrate requirement is completely invisible in the verdict:
// the site appears Defended when its substrate invariant is actually not met.
//
// EXPLOITATION SHAPE (scientist, forward/conjunctive-defense-void-or-semantics):
//   #[defended_by(X)]  -- passes, code-tier = Some(Reachability)
//   #[presents(X, requires = <failing-predicate>)]  -- fails, site_requires_tier = None
//   Result: best_tier = Some(Reachability), verdict = Defended
//   Expected (post V1 fix): Defended with a SubstrateGap co-annotation,
//   or a new Conjunctive-Defended verdict that exposes the substrate gap.
//
// This test documents the V1 gap. The assertion should be INVERTED after
// ADR-030/v0.3 conjunctive-defense semantics land.
//
// The requires_predicate used here (fresh_within_days:90) evaluates against a
// sidecar file that does not exist in the synthetic report's file path
// (PathBuf::from("src/lib.rs") has no .attest/ sidecar on disk), so it
// evaluates to WitnessTier::None regardless of wall-clock time.
#[test]
fn atk_adr029_18_v1_void_failing_requires_masked_by_passing_defended_by() {
    let mut report = ScanReport::default();

    // A presentation site with a failing substrate predicate.
    // The sidecar for "src/lib.rs" does not exist -> fresh_within_days
    // evaluates to WitnessTier::None.
    let presentation = Presentation {
        antigen_type: "SubstrateClass".to_string(),
        file: PathBuf::from("src/lib.rs"),
        line: 20,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Unknown { line: 20 },
        match_kind: MatchKind::ExplicitMarker,
        canonical_path: None,
        inherited_from: None,
        structural_fingerprint: String::new(),
        // Failing substrate predicate: requires sidecar to be fresh within 90
        // days, but the sidecar does not exist.
        requires_predicate: Some(r#"{"leaf":"fresh_within_days","value":90}"#.to_string()),
        proof: None,
    };
    report.presentations.push(presentation);

    // A passing code-tier defense registered via #[defended_by].
    let defense = Defense {
        antigen_type: "SubstrateClass".to_string(),
        file: PathBuf::from("src/tests.rs"),
        line: 5,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Unknown { line: 5 },
    };
    report.defenses.push(defense);

    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let audit_result = audit(&report, workspace_root);

    assert_eq!(
        audit_result.presentation_verdicts.len(),
        1,
        "ATK-ADR029-18: exactly one verdict for the presentation"
    );
    let v = &audit_result.presentation_verdicts[0];

    // CURRENT (V1 void): verdict is Defended at Reachability because
    // code_tier=Some(Reachability) wins the max() over site_requires_tier=None.
    // The failing substrate predicate is silently masked.
    //
    // EXPECTED (post V1 fix): SubstrateGap co-annotation or conjunctive
    // verdict that exposes the failed substrate requirement alongside the
    // passing code witness.
    assert!(
        matches!(v.verdict, ImmuneVerdict::Defended { tier: WitnessTier::Reachability }),
        "ATK-ADR029-18 (V1 VOID): site with failing requires= AND passing #[defended_by] \
        shows Defended at Reachability. The failing substrate predicate is masked by the \
        code witness. OR-semantics (audit.rs:1381) picks best_tier=Reachability; the \
        substrate gap is invisible. After V1 fix, expect SubstrateGap annotation or \
        conjunctive Defended-with-gap verdict. Got: {:?}",
        v.verdict
    );
}

// ATK-ADR029-19: malformed JSON in requires_predicate yields SubstrateGap (not Undefended)
//
// When `requires_predicate` holds syntactically invalid JSON, `serde_json::from_str`
// fails inside `audit_substrate_witness`. The function returns
// `EvaluatedPredicate::sidecar_schema_invalid()` which has `witness_tier = WitnessTier::None`.
// This flows into `site_requires_eval = Some(WitnessTier::None)` -- which means
// `site_requires_eval.is_some()` is `true`, triggering the SubstrateGap arm at
// audit.rs:1394 even though the tier is None.
//
// The correct verdict: SubstrateGap (not Undefended). A malformed predicate has
// declared defensive intent (requires_predicate is Some); the schema failure is a
// substrate issue, not an absence of intent. SubstrateGap accurately signals
// "intent is present; substrate cannot be evaluated."
//
// DEGENERATE INPUT: requires_predicate = "this is not json at all"
#[test]
fn atk_adr029_19_malformed_requires_predicate_json_yields_substrate_gap() {
    let mut report = ScanReport::default();

    report.presentations.push(Presentation {
        antigen_type: "TestClass".to_string(),
        file: PathBuf::from("src/lib.rs"),
        line: 42,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Unknown { line: 42 },
        match_kind: MatchKind::ExplicitMarker,
        canonical_path: None,
        inherited_from: None,
        structural_fingerprint: String::new(),
        // Malformed JSON -- serde_json::from_str will fail.
        requires_predicate: Some("this is not json at all".to_string()),
        proof: None,
    });

    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let audit_result = audit(&report, workspace_root);

    assert_eq!(
        audit_result.presentation_verdicts.len(),
        1,
        "ATK-ADR029-19: exactly one verdict for the presentation"
    );
    let v = &audit_result.presentation_verdicts[0];

    // Malformed JSON predicate -> sidecar_schema_invalid -> WitnessTier::None
    // -> site_requires_eval = Some(None-tier) -> is_some() triggers SubstrateGap.
    // Must NOT be Undefended (intent was declared, even if unparseable).
    assert!(
        matches!(v.verdict, ImmuneVerdict::SubstrateGap),
        "ATK-ADR029-19: malformed requires_predicate JSON must yield SubstrateGap, \
        not Undefended. The author declared intent (Some(...)); the JSON failure is a \
        substrate problem (DisciplineSidecarSchemaInvalid), not an absence of intent. \
        Got: {:?}",
        v.verdict
    );
}

// ATK-ADR029-20: empty-string proof field yields FormalProof tier (overclaim risk)
//
// `site_proof_tier` is computed as `p.proof.as_ref().map(|_| WitnessTier::FormalProof)`
// at audit.rs:1379. The inner value is IGNORED by the map closure -- only the
// presence of Some(_) matters. This means proof=Some("") (empty string) is graded
// at the same FormalProof tier as proof=Some("NonPanickingProof::<T>::verified").
//
// A developer who accidentally writes `#[presents(X, proof="")]` -- perhaps as a
// placeholder while developing -- gets FormalProof verdict on the strongest tier
// possible for nothing. The phantom-proof design intent is: "the mere presence of
// a well-formed phantom-type constructor expression is the proof." But an empty
// string is NOT a valid phantom-type expression; it's a placeholder.
//
// CURRENT BEHAVIOR (documented here as regression anchor): proof="" yields FormalProof.
// Whether this is correct by design (the scanner should gate empty-string proof at
// macro-expand time) or a gap (the audit should validate the expression is non-empty)
// is a design question. This test locks the current behavior so that if the
// empty-string case is ever gated, the assertion is inverted.
//
// The scanner's macro-expand-time parse may already reject empty proof= -- if so,
// this state is unreachable from normal usage but can still arise via hand-built
// ScanReports or future external consumers.
#[test]
fn atk_adr029_20_empty_string_proof_overclaims_formal_proof_tier() {
    let mut report = ScanReport::default();

    report.presentations.push(Presentation {
        antigen_type: "PhantomAntigen".to_string(),
        file: PathBuf::from("src/lib.rs"),
        line: 99,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Unknown { line: 99 },
        match_kind: MatchKind::ExplicitMarker,
        canonical_path: None,
        inherited_from: None,
        structural_fingerprint: String::new(),
        requires_predicate: None,
        // Empty-string proof: should this be FormalProof? The map(|_| FormalProof)
        // closure ignores the inner value entirely.
        proof: Some(String::new()),
    });

    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let audit_result = audit(&report, workspace_root);

    assert_eq!(
        audit_result.presentation_verdicts.len(),
        1,
        "ATK-ADR029-20: exactly one verdict"
    );
    let v = &audit_result.presentation_verdicts[0];

    // CURRENT: empty-string proof yields FormalProof (inner value ignored by map closure).
    // POTENTIAL ALTERNATIVE: a future guard could treat proof=Some("") as Undefended
    // (no actual phantom expression) or SubstrateGap (intent present, expression missing).
    // The assertion here documents CURRENT behavior. Invert if the empty-string
    // case is ever gated at the audit or scanner level.
    assert!(
        matches!(v.verdict, ImmuneVerdict::Defended { tier: WitnessTier::FormalProof }),
        "ATK-ADR029-20 (OVERCLAIM): proof=Some('') yields Defended at FormalProof. \
        The map(|_| FormalProof) closure ignores the inner value; an empty-string proof \
        expression is indistinguishable from a real phantom constructor. \
        Current behavior: FormalProof. Expected after empty-string gate: Undefended or \
        SubstrateGap. Got: {:?}",
        v.verdict
    );
}
