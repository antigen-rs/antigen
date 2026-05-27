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
        canonical_path: None,
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
        canonical_path: None,
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
        canonical_path: None,
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
        canonical_path: None,
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
    // Fixed by 396d146 (adr-029: fold site-attached evidence) -- the audit now
    // gates on Defense.item_kind or scan gates at extract_defended_by time.
    assert!(
        !matches!(v.verdict, ImmuneVerdict::Defended { .. }),
        "ATK-ADR029-8: a #[defended_by] on a struct item_kind must not produce \
         ImmuneVerdict::Defended. A struct cannot be run as a test witness. \
         If this fails, the item_kind gate was removed. Got verdict: {:?}",
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
    // (Fixed by pathmaker's adr-029 wiring; compute_presentation_verdicts()
    // now inspects p.requires_predicate and routes through substrate-witness eval.)
    assert_ne!(
        v.verdict,
        ImmuneVerdict::Undefended,
        "ATK-ADR029-10: a #[presents(X, requires=<pred>)] site with \
         requires_predicate set must not produce Undefended. Declared defensive \
         intent exists (requires_predicate is Some); the verdict should be \
         SubstrateGap (sidecar absent/failing) or Defended (predicate passes). \
         If this fails, p.requires_predicate evaluation was removed. Got: {:?}",
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
        canonical_path: None,
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
    // (Fixed by pathmaker's adr-029 wiring; the impl_fn gate is now accepted.)
    assert!(
        matches!(v.verdict, ImmuneVerdict::Defended { .. }),
        "ATK-ADR029-12: a #[defended_by] on an impl_fn witness must produce Defended. \
         An impl_fn test method is a valid runnable witness. If this fails, \
         the impl_fn item_kind gate was removed. Got: {:?}",
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
    // Defended at FormalProof tier. (Fixed by pathmaker's adr-029 wiring;
    // compute_presentation_verdicts() now reads p.proof.)
    assert!(
        matches!(
            v.verdict,
            ImmuneVerdict::Defended {
                tier: WitnessTier::FormalProof
            }
        ),
        "ATK-ADR029-13: a #[presents(X, proof=...)] site with proof set \
         must produce Defended{{tier:FormalProof}}. The proof expression IS the evidence \
         (phantom-tier: compile-time). If this fails, p.proof reading was removed. Got: {:?}",
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
        canonical_path: None,
    });
    report.defenses.push(Defense {
        antigen_type: "FailureClass".to_string(),
        file: PathBuf::from("src/tests_b.rs"),
        line: 20,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Unknown { line: 20 },
        canonical_path: None,
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
        canonical_path: None,
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
        canonical_path: None,
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
        matches!(
            v.verdict,
            ImmuneVerdict::Defended {
                tier: WitnessTier::Reachability
            }
        ),
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

    // FIXED (findings/proof-empty-string-overclaims-formal-proof): the empty-proof
    // overclaim is gated on TWO layers — the macro now rejects a string-literal
    // `proof =` at authoring time (parse.rs PresentsArgs::validate), and the audit
    // defends-in-depth by not crediting a blank `proof` string (site_proof_tier
    // filters `!s.trim().is_empty()`). A hand-built Presentation with
    // proof=Some("") therefore no longer grades FormalProof — with no other
    // evidence it falls through to Undefended (an empty proof is no proof).
    assert_eq!(
        v.verdict,
        ImmuneVerdict::Undefended,
        "ATK-ADR029-20 (FIXED): proof=Some('') must NOT grade FormalProof — an empty \
        proof string is not a phantom construction. With no other evidence the verdict \
        is Undefended. Got: {:?}",
        v.verdict
    );
}

// ATK-ADR029-22: #[immune(X, requires=P)] with missing sidecar must yield SubstrateGap
//
// `audit()` routes `Immunity { requires_predicate: Some(pred_json), .. }` through
// `audit_substrate_witness()`. When the `.attest/<antigen>.json` sidecar is absent,
// `load_sidecar` returns None and `audit_substrate_witness` returns
// `EvaluatedPredicate::sidecar_missing()` → `ImmunityAudit { witness_tier: None,
// evaluated_predicate: Some(pred_json) }`.
//
// `compute_presentation_verdicts()` then checks `immune_audit_is_substrate_gap`:
// the predicate is `a.evaluated_predicate.is_some() && a.witness_tier == None` —
// BOTH true for sidecar-missing. The verdict must be SubstrateGap, not Undefended.
//
// INVARIANT: defensive intent is declared (requires_predicate is Some);
// a missing sidecar is substrate drift, not absence of intent.
// SubstrateGap is the correct verdict — "intent present, substrate drifted."
// Undefended would be a FALSE NEGATIVE: the audit would appear to say "no
// defense at all" when the adopter HAS declared a substrate-witness defense.
//
// This test has no prior coverage. The `#[presents(X, requires=P)]` path
// (ATK-ADR029-10/19) only tests the Presentation-side eval; neither test
// exercises the deprecated Immunity-side path through `immune_audit_is_substrate_gap`.
//
// DEGENERATE INPUT: Immunity.requires_predicate=Some(<valid predicate JSON>)
// with no .attest/ sidecar on disk.
#[test]
fn atk_adr029_22_immune_requires_predicate_missing_sidecar_yields_substrate_gap() {
    let mut report = ScanReport::default();

    // A presents-site for the same antigen class.
    report.presentations.push(Presentation {
        antigen_type: "SubstrateGapClass".to_string(),
        file: PathBuf::from("src/lib.rs"),
        line: 10,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Fn("do_thing".to_string()),
        match_kind: MatchKind::ExplicitMarker,
        canonical_path: None,
        inherited_from: None,
        structural_fingerprint: String::new(),
        requires_predicate: None,
        proof: None,
    });

    // The deprecated #[immune(SubstrateGapClass, requires=<predicate>)] site — same
    // item as the presents site. `requires_predicate` is Some with a well-formed
    // predicate JSON. No .attest/ sidecar exists at `src/lib.rs` for this antigen.
    // `audit_substrate_witness` will call `load_sidecar("src/lib.rs", "SubstrateGapClass")`
    // → None (file doesn't exist) → EvaluatedPredicate::sidecar_missing()
    // → ImmunityAudit { witness_tier: None, evaluated_predicate: Some(...) }.
    report.immunities.push(Immunity {
        antigen_type: "SubstrateGapClass".to_string(),
        witness: String::new(), // not used on the requires= path
        file: PathBuf::from("src/lib.rs"),
        line: 10,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Fn("do_thing".to_string()),
        canonical_path: None,
        // Valid predicate JSON — the predicate is well-formed but the sidecar is absent.
        requires_predicate: Some(r#"{"leaf":"fresh_within_days","value":90}"#.to_string()),
        structural_fingerprint: String::new(),
    });

    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let audit_result = audit(&report, workspace_root);

    let v = audit_result
        .presentation_verdicts
        .iter()
        .find(|v| v.antigen_type == "SubstrateGapClass")
        .expect("a verdict for SubstrateGapClass");

    // INVARIANT: must be SubstrateGap (not Undefended).
    // The adopter declared requires= intent; the sidecar absence is a substrate
    // drift, not an absence of intent. `immune_audit_is_substrate_gap` should
    // detect: evaluated_predicate.is_some() (set to the predicate JSON) AND
    // witness_tier == None (sidecar missing → None tier). Both conditions true
    // → SubstrateGap arm fires in compute_presentation_verdicts().
    //
    // If this fails with Undefended: the `immune_audit_is_substrate_gap` path
    // at audit.rs:1439-1442 does not reach the ImmunityAudit produced by the
    // Immunity match lookup (check audit.rs:1357-1361 item_target matching, or
    // check that `evaluated_predicate` is actually Some in the missing-sidecar case).
    assert!(
        matches!(v.verdict, ImmuneVerdict::SubstrateGap),
        "ATK-ADR029-22: #[immune(X, requires=P)] with missing sidecar must yield \
        SubstrateGap, not Undefended. The adopter declared substrate-witness intent \
        (requires_predicate is Some); a missing sidecar is substrate drift, not absence \
        of intent. SubstrateGap signals 'intent present, substrate drifted.' \
        immune_audit_is_substrate_gap should have fired. Got: {:?}",
        v.verdict
    );
}

// ATK-ADR029-21: defense canonical_path mismatch — cross-crate overclaim
//
// `Defense` has no `canonical_path` field. `unaddressed_presentations()` checks
// only `d.antigen_type == p.antigen_type` (line 2076 in scan.rs). A
// `#[defended_by(Foo)]` declaration in crate A silently covers `#[presents(Foo)]`
// from crate B (different `canonical_path`) because the antigen_type bare name
// matches.
//
// This is a cross-crate defense overclaim: the author's intent was to defend
// their OWN `Foo`, not a `Foo` from a different crate. When both crates are
// scanned together (cross-crate scan), the defense incorrectly suppresses the
// audit verdict for the foreign presentation.
//
// CURRENT BEHAVIOR: defense without canonical_path matches presentation with
// canonical_path (or vice versa) as long as antigen_type matches. The
// `unaddressed_presentations()` function returns an empty list (the site is
// "addressed") even though the defense is from a different crate's Foo.
//
// FIX DIRECTION: `Defense` should carry a `canonical_path: Option<String>` field
// mirroring `Presentation.canonical_path`. The matching logic should use the
// same `(antigen_type, canonical_path)` tuple semantics as the antigen-known
// lookup (lines 2047-2051 in scan.rs). A defense with `canonical_path=None`
// could continue to match any canonical_path (backward compat), but a defense
// with a specific canonical_path should only match presentations with the same
// canonical_path.
//
// NOTE: This gap requires a `Defense.canonical_path` field addition to the
// scan.rs struct + the macro parser. The audit-level fix in audit.rs:1314
// (antigen_type exact match) is consistent with the current scan.rs behavior --
// both use bare antigen_type, so both have the same cross-crate gap.
#[test]
fn atk_adr029_21_defense_matches_cross_crate_presentation_with_same_type_name() {
    // FIXED (findings/defense-canonical-path-cross-crate-overclaim): in a real
    // cross-crate scan both sides are canonical-stamped (ADR-017). A presentation
    // from other_crate (canonical_path = other_crate::Foo) and a defense from
    // this_crate (canonical_path = this_crate::Foo) carry DIFFERENT canonical
    // paths, so the (antigen_type, canonical_path) tuple match does NOT treat the
    // local defense as covering the foreign presentation.
    let mut report = ScanReport::default();

    report.presentations.push(Presentation {
        antigen_type: "Foo".to_string(),
        file: PathBuf::from("src/lib.rs"),
        line: 1,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Unknown { line: 1 },
        match_kind: MatchKind::ExplicitMarker,
        canonical_path: Some("other_crate::Foo".to_string()), // cross-crate!
        inherited_from: None,
        structural_fingerprint: String::new(),
        requires_predicate: None,
        proof: None,
    });

    // The defense is for THIS crate's Foo — stamped with this crate's canonical
    // path (≠ other_crate). It must NOT cover the foreign presentation.
    report.defenses.push(Defense {
        antigen_type: "Foo".to_string(), // same bare name, DIFFERENT crate
        file: PathBuf::from("src/tests.rs"),
        line: 5,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Unknown { line: 5 },
        canonical_path: Some("this_crate::Foo".to_string()),
    });

    // The foreign presentation is NOT addressed by the local defense — the
    // canonical_path tuple distinguishes other_crate::Foo from this_crate::Foo.
    let unaddressed = report.unaddressed_presentations();
    assert_eq!(
        unaddressed.len(),
        1,
        "ATK-ADR029-21 (FIXED): a this_crate defense must NOT cover an other_crate \
         presentation of the same bare name — the (antigen_type, canonical_path) \
         tuple match keeps them distinct. The foreign presentation stays unaddressed. \
         Got unaddressed: {:?}",
        unaddressed
    );

    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let audit_result = audit(&report, workspace_root);
    assert_eq!(
        audit_result.presentation_verdicts.len(),
        1,
        "ATK-ADR029-21: exactly one verdict for the one presentation"
    );
    let v = &audit_result.presentation_verdicts[0];
    assert_eq!(
        v.verdict,
        ImmuneVerdict::Undefended,
        "ATK-ADR029-21 (FIXED): the cross-crate presentation is Undefended — the \
         local (this_crate) defense does not cross-reference a different crate's Foo. \
         Got: {:?}",
        v.verdict
    );
}

// ========================================================================
// ATK-G2-22: G2 defense check inherits the cross-crate bare-name overclaim
// (2026-05-27, adversarial)
//
// The G2 fix (5cdbad9) uses `d.antigen_type == decl.type_name` (bare-name)
// when checking whether a #[defended_by] registration contributes code-tier
// evidence to an antigen. This is the same canonical_path-less comparison
// as ATK-ADR029-21, applied to a new location (audit.rs:3100 in audit_category).
//
// ATTACK SCENARIO (false positive):
//   - Crate A declares SubstrateAlignment antigen `Foo`
//     (canonical_path = "crate_a::Foo").
//   - Crate B has `#[test] #[defended_by(Foo)]` for crate B's own `Foo`
//     (canonical_path = "crate_b::Foo") — a completely different failure class.
//   - In a cross-crate scan, G2 sees the crate_b defense and sets
//     has_code_witness=true for crate_a::Foo, then correctly emits
//     AntigenCategoryClaimInconsistentWithPredicateType.
//
// BUT: the hint is SPURIOUS. Crate A's SubstrateAlignment `Foo` has NO
// code-tier evidence; the matching defense belongs to a different antigen.
// The G2 hint fires based on a wrong-crate defense.
//
// ATTACK SCENARIO (false negative, harder to construct):
//   - SubstrateAlignment antigen with no real evidence, but a same-name
//     defense exists in a different crate. G2 fires the wrong hint but
//     does not report the REAL gap (no actual evidence for this antigen).
//
// Root cause: same as ATK-ADR029-21. Fix: audit_category's defense lookup
// should use (antigen_type, canonical_path) tuple matching, parallel to the
// immunity lookup's existing canonical_path handling.
// ========================================================================
#[test]
fn atk_g2_22_cross_crate_defense_triggers_spurious_g2_hint() {
    use antigen::audit::{audit_category, AuditHint};
    use antigen::category::AntigenCategory;
    use antigen::scan::{AntigenDeclaration, Defense, ScanReport};

    let mut report = ScanReport::default();

    // Crate A: SubstrateAlignment antigen "Foo" (external crate).
    report.antigens.push(AntigenDeclaration {
        type_name: "Foo".to_string(),
        name: "foo".to_string(),
        fingerprint: None,
        canonical_path: Some("crate_a::antigens::Foo".to_string()),
        file: std::path::PathBuf::from("src/antigens.rs"),
        line: 1,
        summary: Some("A substrate-alignment failure class.".to_string()),
        category: vec![AntigenCategory::SubstrateAlignment],
        family: None,
    });

    // Crate B: a `#[defended_by(Foo)]` for a DIFFERENT Foo — same bare name,
    // different canonical_path. In a real cross-crate scan, BOTH the crate_a
    // antigen and the crate_b defense are canonical-stamped (ADR-017,
    // stamp_canonical_path), so crate_b's defense carries
    // canonical_path = Some("crate_b::..."). The G2 match is now
    // (antigen_type, canonical_path) tuple-aware, so this defense — belonging to
    // crate_b::Foo — does NOT count as evidence for crate_a::Foo.
    report.defenses.push(Defense {
        antigen_type: "Foo".to_string(), // same bare name as crate_a::Foo
        file: std::path::PathBuf::from("tests/crate_b_tests.rs"),
        line: 5,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Fn("defends_crate_b_foo".to_string()),
        canonical_path: Some("crate_b::antigens::Foo".to_string()), // stamped, ≠ crate_a
    });

    let category_report = audit_category(&report);

    // FIXED (findings/g2-cross-crate-bare-name-overclaim): G2 now matches the
    // defense by (antigen_type, canonical_path). crate_b's defense
    // (canonical_path = crate_b::…) does NOT match crate_a::Foo, so it is NOT
    // counted as evidence — no spurious mismatch. crate_a::Foo has zero real
    // evidence addressing it (the coverage-gap case, which G2 does not flag).
    assert_eq!(
        category_report.mismatch_count, 0,
        "ATK-G2-22 (FIXED): a cross-crate defense (crate_b::Foo) must NOT count as \
         evidence for crate_a::Foo — the canonical_path tuple match distinguishes them. \
         No real evidence addresses crate_a::Foo, so no G2 mismatch fires (a pure \
         coverage gap, not a witness-type mismatch). Got: {}",
        category_report.mismatch_count
    );
    let has_g2_hint = category_report.audits.iter().any(|a| {
        a.hints
            .contains(&AuditHint::AntigenCategoryClaimInconsistentWithPredicateType)
    });
    assert!(
        !has_g2_hint,
        "ATK-G2-22 (FIXED): no spurious G2 hint when the only defense is from a \
         DIFFERENT crate (canonical_path mismatch)"
    );
}

// ATK-ADR029-23: unstamped intra-workspace defense (canonical_path=None) covers
// cross-crate presentations (canonical_path=Some("dep@version")) via wildcard match.
//
// `defense_addresses()` in scan.rs uses:
//   d.canonical_path.is_none() || d.canonical_path == p.canonical_path
//
// When `d.canonical_path = None` (intra-workspace, unstamped), the `is_none()`
// branch is true regardless of p.canonical_path — the defense matches ANY
// presentation with the same bare antigen_type, including cross-dep presentations
// that have been stamped with `canonical_path = Some("dep@version")`.
//
// This is intentional backward-compat for intra-workspace scans (where neither
// defense nor presentation gets stamped — both None matches both None via
// the is_none() branch). But in `--include-deps` scans, dep presentations get
// stamped while own-workspace defenses stay None. An own-workspace
// `#[defended_by(Foo)]` with canonical_path=None then acts as a wildcard:
// it covers its own crate's Foo AND any dep's Foo with the same bare name.
//
// IMPACT: In a `--include-deps` scan, an intra-workspace defense incorrectly
// satisfies a dep's presentation → the dep's presents-site shows Defended (not
// Undefended) in the verdict, hiding the dep's undefended vulnerability.
//
// POSTURE: This test DOCUMENTS current behavior (unstamped None = wildcard).
// It is NOT a fatal error for most use cases (intra-workspace-only scans are
// unaffected). But it is a gap for `--include-deps` users. A future fix:
// when a presentation has canonical_path=Some(X), require defenses to ALSO
// have canonical_path=Some(X) (no None wildcard against a stamped presentation).
// The backward-compat concern: existing intra-workspace sidecars would need
// stamp_canonical_path to be called on primary-workspace defenses too.
//
// This test pins the current behavior so the wildcard semantics are explicit.
// If defense_addresses() is tightened (None defense does NOT match Some presentation),
// invert the assertion sense and update the comment above.
#[test]
fn atk_adr029_23_unstamped_defense_wildcard_covers_cross_crate_presentation() {
    let mut report = ScanReport::default();

    // A cross-dep presentation: stamped with a dep crate ID (as --include-deps would).
    report.presentations.push(Presentation {
        antigen_type: "SharedName".to_string(),
        file: PathBuf::from("dep/src/lib.rs"),
        line: 10,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Fn("dep_fn".to_string()),
        match_kind: MatchKind::ExplicitMarker,
        canonical_path: Some("some-dep@1.0.0".to_string()), // stamped by --include-deps driver
        inherited_from: None,
        structural_fingerprint: String::new(),
        requires_predicate: None,
        proof: None,
    });

    // An intra-workspace defense: unstamped (canonical_path = None), as the
    // primary-workspace defenses are before stamp_canonical_path is called on them.
    report.defenses.push(Defense {
        antigen_type: "SharedName".to_string(),
        file: PathBuf::from("src/tests.rs"),
        line: 5,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Fn("my_test".to_string()),
        canonical_path: None, // intra-workspace, not stamped
    });

    // defense_addresses is pub(crate) — test via the public unaddressed_presentations().
    // If the unstamped defense DOES cover the dep's presentation, the presentation
    // appears addressed (not in unaddressed list). If it does NOT cover it, the
    // presentation appears unaddressed.
    let unaddressed = report.unaddressed_presentations();

    // CURRENT BEHAVIOR: unaddressed list is EMPTY — the unstamped None defense acts
    // as a wildcard and "addresses" the stamped dep presentation. The dep's
    // undefended vulnerability is invisible.
    //
    // This is the wildcard semantics: None = "match any crate."
    assert!(
        unaddressed.is_empty(),
        "ATK-ADR029-23 (CURRENT BEHAVIOR — wildcard): an intra-workspace defense \
        (canonical_path=None) should match the stamped dep presentation \
        (canonical_path=Some('some-dep@1.0.0')) via the wildcard semantics. \
        The unaddressed list must be empty because the None defense 'covers' the dep. \
        If this assertion FAILS: the wildcard semantics were tightened — the None \
        defense no longer wildcards against stamped presentations. Invert the \
        assert to expect unaddressed.len()==1 and document the fix."
    );
}
