//! End-to-end integration test: real `.attest/<Antigen>.json` on disk →
//! schema parse → semantic validation → predicate evaluation → audit
//! axes (`WitnessTier × AuditHint × EvidenceKind`).
//!
//! Exercises the v0.1-rc substrate-witness primitive against a temporary
//! workspace fixture rather than in-memory contexts. Covers the
//! happy-path immunity flow + the `tolerance-vibes-grade` distinction
//! (ADR-011 gap closure) + the schema-invalid path (anti-laundering).
//!
//! This test is part of the antigen-attestation lock-down corpus that
//! prevents the v0.1-rc invariants from silently drifting.

use std::collections::BTreeMap;
use std::path::Path;

use antigen_attestation::{
    AntigenIdentifier, AuditHint, EvaluationContext, EvidenceKind, ItemRatification, Leaf,
    Predicate, Ratification, RatificationKind, SchemaVersion, SignatureStrength, Signer,
    SignerBasis, WitnessTier,
    evaluate::evaluate_predicate_with_kind,
    predicate::SignerCurrency,
    schema::{DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS, ValidationError},
};
use chrono::NaiveDate;

/// Real-filesystem evaluation context using `std::fs` for doc/oracle
/// reads. Trailers / clock are fakeable per-test.
struct FsContext {
    today: NaiveDate,
    workspace_root: std::path::PathBuf,
    trailers: BTreeMap<(std::path::PathBuf, String), Vec<String>>,
}

impl FsContext {
    const fn new(workspace_root: std::path::PathBuf, today: NaiveDate) -> Self {
        Self {
            today,
            workspace_root,
            trailers: BTreeMap::new(),
        }
    }
}

impl EvaluationContext for FsContext {
    fn today(&self) -> NaiveDate {
        self.today
    }

    fn read_doc(&self, path: &Path) -> Option<String> {
        let full = self.workspace_root.join(path);
        std::fs::read_to_string(&full).ok()
    }

    fn read_oracle(&self, path: &Path) -> Option<String> {
        let full = self.workspace_root.join(path);
        std::fs::read_to_string(&full).ok()
    }

    fn read_git_trailers(&self, item_source_file: &Path, item_path: &str) -> Vec<String> {
        self.trailers
            .get(&(item_source_file.to_path_buf(), item_path.to_string()))
            .cloned()
            .unwrap_or_default()
    }
}

const fn sample_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 5, 19).expect("hard-coded valid date")
}

fn fresh_signer(name: &str, date: NaiveDate, fp: &str) -> Signer {
    Signer {
        name: name.to_string(),
        role: None,
        date,
        signed_against_fingerprint: fp.to_string(),
        basis: SignerBasis::Fresh { reasoning: None },
        strength: SignatureStrength::GitTrust,
        signature: None,
    }
}

#[test]
fn happy_path_immunity_predicate_passes_substrate_current() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().to_path_buf();

    // Write the discipline doc to disk.
    std::fs::create_dir_all(root.join("docs")).unwrap();
    std::fs::write(
        root.join("docs/sinh.md"),
        "---\nversion: 1.2\n---\n# Sinh discipline\nbody",
    )
    .unwrap();

    // Build the sidecar in-memory.
    let item = ItemRatification {
        item_path: "sinh".to_string(),
        current_fingerprint: "fp-current".to_string(),
        doc_ref: None,
        signers: vec![
            fresh_signer("alice", sample_date(), "fp-current"),
            fresh_signer("bob", sample_date(), "fp-current"),
        ],
        oracles: vec![],
        fresh_through: None,
        extensions: BTreeMap::new(),
    };
    let rat = Ratification {
        schema_version: SchemaVersion::V1,
        kind: RatificationKind::Immunity,
        antigen: AntigenIdentifier {
            name: "SignedZeroDiscipline".to_string(),
            defined_in: None,
        },
        source_file: std::path::PathBuf::from("src/numerics.rs"),
        items: vec![item],
    };

    // Serialize to JSON + re-parse (round-trip integrity).
    let json = serde_json::to_string_pretty(&rat).unwrap();
    let parsed: Ratification = serde_json::from_str(&json).unwrap();

    // Validate semantic invariants.
    parsed
        .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
        .expect("sidecar should validate");

    // Build the predicate: all_of([signers(alice, bob), ratified_doc(min=1.0), fresh_within_days(180)]).
    let pred = Predicate::all_of(vec![
        Predicate::leaf(Leaf::Signers {
            required: vec!["alice".to_string(), "bob".to_string()],
            roles: BTreeMap::new(),
            against: SignerCurrency::default(),
            signature_allow: vec![],
            signature_prefer: None,
        }),
        Predicate::leaf(Leaf::RatifiedDoc {
            path: Some(std::path::PathBuf::from("docs/sinh.md")),
            min_version: Some("1.0".to_string()),
            anchor: None,
            sibling_json: false,
        }),
        Predicate::leaf(Leaf::FreshWithinDays { days: 180 }),
    ])
    .unwrap();

    // Evaluate.
    let ctx = FsContext::new(root, sample_date());
    let item = &parsed.items[0];
    let result = antigen_attestation::evaluate::evaluate_predicate(
        &pred,
        item,
        "fp-current",
        &std::path::PathBuf::from("src/numerics.rs"),
        &ctx,
    )
    .unwrap();

    assert_eq!(result.witness_tier, WitnessTier::Execution);
    assert_eq!(
        result.audit_hint,
        AuditHint::DisciplinePredicatePassedSubstrateCurrent
    );
    assert_eq!(result.evidence_kind, EvidenceKind::SubstrateState);
    assert_eq!(result.signature_strength, Some(SignatureStrength::GitTrust));
}

#[test]
fn tolerance_vibes_grade_distinct_from_substrate_state() {
    // Constructed result: the canonical "tolerance vibes-grade" output
    // (returned when `#[antigen_tolerance(X)]` has no `sidecar = true`).
    let result = antigen_attestation::EvaluatedPredicate::tolerance_vibes_grade();
    // Per ADR-019 §M5: vibes-grade tolerance reports
    // EvidenceKind::None (not SubstrateState) to surface that NO substrate
    // was consulted. This is the load-bearing distinction that lets CI
    // gates separate vibes-grade tolerance from attested tolerance.
    assert_eq!(result.witness_tier, WitnessTier::None);
    assert_eq!(result.audit_hint, AuditHint::ToleranceVibesGrade);
    assert_eq!(result.evidence_kind, EvidenceKind::None);
    assert_eq!(result.signature_strength, None);
}

#[test]
fn substrate_witness_cannot_reach_formal_proof_per_kind_ceiling() {
    // EvidenceKind::SubstrateState ceiling: cannot reach FormalProof.
    // This is the load-bearing tier-honesty invariant from ADR-019
    // §Decision — substrate-state evidence is bounded above by Execution
    // regardless of how strong the substrate is.
    assert_eq!(
        EvidenceKind::SubstrateState.max_tier(),
        WitnessTier::Execution
    );
    assert!(!EvidenceKind::SubstrateState.can_reach(WitnessTier::FormalProof));
    // Behavioral has the same ceiling (Execution).
    assert_eq!(EvidenceKind::Behavioral.max_tier(), WitnessTier::Execution);
    // Only TypeSystemProof reaches FormalProof.
    assert_eq!(
        EvidenceKind::TypeSystemProof.max_tier(),
        WitnessTier::FormalProof
    );
}

#[test]
fn delta_with_rubber_stamp_rationale_caught_post_deserialization_t2r_b() {
    // Build a sidecar with a rubber-stamp DeltaFrom rationale, serialize,
    // and verify validation catches it at the schema layer. This exercises
    // the full path (in-memory build → serde JSON → from_str → validate)
    // rather than just the in-memory shape check.
    let item = ItemRatification {
        item_path: "sinh".to_string(),
        current_fingerprint: "fp-current".to_string(),
        doc_ref: None,
        signers: vec![Signer {
            name: "alice".to_string(),
            role: None,
            date: sample_date(),
            signed_against_fingerprint: "fp-current".to_string(),
            basis: SignerBasis::DeltaFrom {
                prior_fingerprint: "fp-prior".to_string(),
                // chain_depth=1 → cumulative_root must equal prior (NFA-12 invariant)
                cumulative_root_fingerprint: "fp-prior".to_string(),
                chain_depth: 1,
                rationale: "ok".to_string(), // rubber-stamp; should be rejected
            },
            strength: SignatureStrength::GitTrust,
            signature: None,
        }],
        oracles: vec![],
        fresh_through: None,
        extensions: BTreeMap::new(),
    };
    let rat = Ratification {
        schema_version: SchemaVersion::V1,
        kind: RatificationKind::Immunity,
        antigen: AntigenIdentifier {
            name: "TestAntigen".to_string(),
            defined_in: None,
        },
        source_file: std::path::PathBuf::from("src/test.rs"),
        items: vec![item],
    };
    let json = serde_json::to_string(&rat).unwrap();
    let parsed: Ratification = serde_json::from_str(&json).unwrap();
    let err = parsed
        .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
        .unwrap_err();
    assert!(
        matches!(err, ValidationError::RationaleTooShort { .. }),
        "expected RationaleTooShort, got {err:?}"
    );
}

#[test]
fn workspace_chain_cap_floor_invariant_t2r_c() {
    // Workspace cannot configure cap = 0 (bypass attack from adversarial
    // T2R-C). The hard floor protects the anti-laundering safeguard.
    let err = antigen_attestation::schema::validate_chain_cap(0).unwrap_err();
    assert!(matches!(
        err,
        ValidationError::WorkspaceConfigOutOfBounds { .. }
    ));
    // Workspace cannot configure cap > HARD_DELTA_CHAIN_CAP_MAX either.
    let err = antigen_attestation::schema::validate_chain_cap(999).unwrap_err();
    assert!(matches!(
        err,
        ValidationError::WorkspaceConfigOutOfBounds { .. }
    ));
    // Default cap is valid.
    assert!(antigen_attestation::schema::validate_chain_cap(DEFAULT_DELTA_CHAIN_CAP).is_ok());
}

#[test]
fn tolerance_predicate_pass_reports_tolerance_hint_not_immunity_hint_nfa10() {
    // BUG REGRESSION TEST (adversarial NFA-10): evaluate_predicate() has no
    // RatificationKind parameter. A Tolerance sidecar with a passing predicate
    // returns DisciplinePredicatePassedSubstrateCurrent (an immunity hint)
    // instead of TolerancePredicatePassedSubstrateCurrent. The tolerance-
    // specific AuditHints are defined in tier.rs but never emitted because the
    // evaluator cannot distinguish kind.
    //
    // Fix: add `kind: RatificationKind` parameter to evaluate_predicate() and
    // thread it to classify_passed_predicate() for kind-specific hint selection.
    //
    // This test FAILS against the current code which returns the immunity hint.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().to_path_buf();

    // Write a discipline doc.
    std::fs::create_dir_all(root.join("docs")).unwrap();
    std::fs::write(
        root.join("docs/tolerance-basis.md"),
        "---\nversion: 1.0\n---\n# Tolerance basis\nbody",
    )
    .unwrap();

    let item = ItemRatification {
        item_path: "overflow_fn".to_string(),
        current_fingerprint: "fp-overflow".to_string(),
        doc_ref: None,
        signers: vec![fresh_signer("carol", sample_date(), "fp-overflow")],
        oracles: vec![],
        fresh_through: None,
        extensions: BTreeMap::new(),
    };
    // Key: RatificationKind::Tolerance
    let rat = Ratification {
        schema_version: SchemaVersion::V1,
        kind: RatificationKind::Tolerance,
        antigen: AntigenIdentifier {
            name: "SignedZeroDiscipline".to_string(),
            defined_in: None,
        },
        source_file: std::path::PathBuf::from("src/numerics.rs"),
        items: vec![item],
    };
    rat.validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
        .expect("tolerance sidecar is valid");

    let pred = Predicate::all_of(vec![
        Predicate::leaf(Leaf::Signers {
            required: vec!["carol".to_string()],
            roles: BTreeMap::new(),
            against: SignerCurrency::default(),
            signature_allow: vec![],
            signature_prefer: None,
        }),
        Predicate::leaf(Leaf::RatifiedDoc {
            path: Some(std::path::PathBuf::from("docs/tolerance-basis.md")),
            min_version: Some("1.0".to_string()),
            anchor: None,
            sibling_json: false,
        }),
    ])
    .unwrap();

    let ctx = FsContext::new(root, sample_date());
    let item = &rat.items[0];
    // Use evaluate_predicate_with_kind so the evaluator knows this is a
    // Tolerance sidecar and selects the correct hint variant.
    let result = evaluate_predicate_with_kind(
        &pred,
        item,
        "fp-overflow",
        &std::path::PathBuf::from("src/numerics.rs"),
        RatificationKind::Tolerance,
        &ctx,
    )
    .unwrap();

    // CRITICAL: tolerance sidecar must return tolerance-specific hint,
    // not the immunity hint.
    assert_eq!(
        result.audit_hint,
        antigen_attestation::AuditHint::TolerancePredicatePassedSubstrateCurrent,
        "tolerance sidecar passing predicate must emit tolerance hint, not immunity hint"
    );
}

#[test]
fn signerless_predicate_reports_no_signature_strength() {
    // BUG REGRESSION TEST (adversarial NFA-5): a predicate that passes via
    // non-signer leaves only (ratified_doc + fresh_within_days, no `signers`
    // leaf) must report `signature_strength: None`. There is no git-trust
    // identity binding when no signer exists. The previous code incorrectly
    // returned `Some(SignatureStrength::GitTrust)` for signerless items.
    //
    // This test FAILS against the buggy code at evaluate.rs:435 which
    // unconditionally returns `Some(GitTrust)` when `item.signers.is_empty()`.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().to_path_buf();

    // Write the discipline doc to disk.
    std::fs::create_dir_all(root.join("docs")).unwrap();
    std::fs::write(
        root.join("docs/discipline.md"),
        "---\nversion: 1.0\n---\n# Discipline doc\nbody",
    )
    .unwrap();

    // Signerless item — no signers at all. (Formerly this carried a
    // `fresh_through` and the predicate below included a `fresh_within_days`
    // leaf, which relied on the now-closed temporal forged-freshness bypass
    // ATK-FT-1: fresh_through alone, with no current-fp signer, no longer
    // anchors freshness. To keep this test about its REAL contract —
    // "a signerless predicate that PASSES must report signature_strength: None,
    // not a fabricated GitTrust" — the predicate now uses only the
    // signerless-passable `ratified_doc` leaf, no freshness dependency.)
    let item = ItemRatification {
        item_path: "some_fn".to_string(),
        current_fingerprint: "fp-abc".to_string(),
        doc_ref: None,
        signers: vec![], // no signers — predicate must not claim GitTrust
        oracles: vec![],
        fresh_through: None,
        extensions: BTreeMap::new(),
    };
    let rat = Ratification {
        schema_version: SchemaVersion::V1,
        kind: RatificationKind::Immunity,
        antigen: AntigenIdentifier {
            name: "TestAntigen".to_string(),
            defined_in: None,
        },
        source_file: std::path::PathBuf::from("src/lib.rs"),
        items: vec![item],
    };
    rat.validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
        .expect("signerless sidecar is structurally valid");

    // Predicate: ratified_doc only — a signerless-passable leaf, zero signer
    // requirements and no freshness dependency (see fixture note above).
    let pred = Predicate::leaf(Leaf::RatifiedDoc {
        path: Some(std::path::PathBuf::from("docs/discipline.md")),
        min_version: Some("1.0".to_string()),
        anchor: None,
        sibling_json: false,
    });

    let ctx = FsContext::new(root, sample_date());
    let item = &rat.items[0];
    let result = antigen_attestation::evaluate::evaluate_predicate(
        &pred,
        item,
        "fp-abc",
        &std::path::PathBuf::from("src/lib.rs"),
        &ctx,
    )
    .unwrap();

    // Predicate should pass — doc exists with the right version, item is fresh.
    assert_eq!(result.witness_tier, WitnessTier::Execution);
    assert_eq!(result.evidence_kind, EvidenceKind::SubstrateState);

    // CRITICAL: no signers → no git-trust identity binding → signature_strength must be None.
    assert_eq!(
        result.signature_strength, None,
        "signerless predicate must not claim GitTrust — no signer exists to bind identity"
    );
}

#[test]
fn delta_chain_depth1_inconsistent_cumulative_root_rejected_nfa12() {
    // BUG REGRESSION TEST (adversarial NFA-12): `SignerBasis::DeltaFrom` carries
    // `cumulative_root_fingerprint` documented as "Anti-laundering safeguard #2"
    // (schema.rs §DeltaFrom). For chain_depth=1, `cumulative_root_fingerprint`
    // MUST equal `prior_fingerprint` — they refer to the same event (the last
    // Fresh signature IS the prior signature at depth 1). If they differ, the
    // sidecar is internally inconsistent: the field that's supposed to anchor
    // cumulative-diff tracking points at a different location than the chain root.
    //
    // CURRENT STATE: `Ratification::validate()` destructures `DeltaFrom` with `..`,
    // ignoring both `prior_fingerprint` and `cumulative_root_fingerprint`. The
    // evaluator never reads `cumulative_root_fingerprint` either. A sidecar with
    // a completely fabricated `cumulative_root_fingerprint` passes validation,
    // serializes, re-parses, and evaluates to `DisciplinePredicatePassedViaDeltaChain`
    // with `SignatureStrength::GitTrust` — silently reporting a plausible-but-wrong
    // anti-laundering state because safeguard #2 was never checked.
    //
    // FIX DIRECTION: `Ratification::validate()` should check that for chain_depth=1
    // signers, `cumulative_root_fingerprint == prior_fingerprint` (they are the same
    // event at depth 1; any divergence is a sidecar construction error or tamper).
    //
    // This test FAILS until the fix is applied.
    let item = ItemRatification {
        item_path: "sinh".to_string(),
        current_fingerprint: "fp-current".to_string(),
        doc_ref: None,
        signers: vec![Signer {
            name: "alice".to_string(),
            role: None,
            date: sample_date(),
            signed_against_fingerprint: "fp-current".to_string(),
            basis: SignerBasis::DeltaFrom {
                prior_fingerprint: "fp-prior-real".to_string(),
                // INCONSISTENT: at chain_depth=1 this must equal prior_fingerprint.
                // A different value here is either a mistake or an attempt to anchor
                // cumulative-diff tracking at a non-existent fingerprint.
                cumulative_root_fingerprint: "fp-root-FABRICATED".to_string(),
                chain_depth: 1,
                rationale: "reviewed the diff carefully, no invariant impact".to_string(),
            },
            strength: SignatureStrength::GitTrust,
            signature: None,
        }],
        oracles: vec![],
        fresh_through: None,
        extensions: BTreeMap::new(),
    };
    let rat = Ratification {
        schema_version: SchemaVersion::V1,
        kind: RatificationKind::Immunity,
        antigen: AntigenIdentifier {
            name: "TestAntigen".to_string(),
            defined_in: None,
        },
        source_file: std::path::PathBuf::from("src/test.rs"),
        items: vec![item],
    };
    let json = serde_json::to_string(&rat).unwrap();
    let parsed: Ratification = serde_json::from_str(&json).unwrap();

    // This SHOULD fail validation: cumulative_root_fingerprint != prior_fingerprint
    // at chain_depth=1 is an internal inconsistency in the anti-laundering chain.
    let err = parsed
        .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
        .expect_err(
            "NFA-12: validate() must reject chain_depth=1 sidecar where \
             cumulative_root_fingerprint != prior_fingerprint",
        );
    assert!(
        matches!(err, ValidationError::InconsistentCumulativeRoot { .. }),
        "expected InconsistentCumulativeRoot, got {err:?}"
    );
}
