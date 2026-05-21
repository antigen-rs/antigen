//! Adversarial precision test for the tolerance-ratification audit-hint
//! surface (P3e slice 4 / 7).
//!
//! ## What this test guards
//!
//! ADR-019 closes ADR-011's tolerance vibes-grade gap by introducing an
//! opt-in `RatificationKind::Tolerance` discipline-witness with the SAME
//! isomorphic schema as immunity but a discriminated terminal hint set.
//! This test locks down the kind-discriminator contract at the precise
//! boundaries where confusion would be silently destructive:
//!
//! 1. **Vibes-grade default**: `#[antigen_tolerance]` without `sidecar = true`
//!    opt-in MUST emit `ToleranceVibesGrade` + `WitnessTier::None` +
//!    `EvidenceKind::None`. Emitting `SubstrateState` here would falsely
//!    imply substrate was consulted; emitting any non-`None` tier would
//!    overclaim. The constructor `EvaluatedPredicate::tolerance_vibes_grade`
//!    is the load-bearing API surface here.
//!
//! 2. **Kind-discriminator serde**: `RatificationKind` is a `snake_case`
//!    discriminator on `Ratification`. A schema-evolution that accidentally
//!    swaps the kebab-case-vs-snake_case convention or drops the
//!    discriminator entirely would let an immunity sidecar parse as a
//!    tolerance sidecar (or vice versa) with NO error — both arms of the
//!    enum have isomorphic field layouts. This is the highest-risk silent-
//!    confusion mode in ADR-019.
//!
//! 3. **Terminal-hint discrimination on pass/fail**: when the predicate
//!    pass/fail is the terminal classification (no intermediate state),
//!    tolerance sidecars MUST emit the `Tolerance*` variant and immunity
//!    sidecars MUST emit the `Discipline*` variant. This is the
//!    `RatificationKind`-threading work from NFA-10.
//!
//! 4. **v0.2 gap documentation**: tolerance sidecars in INTERMEDIATE
//!    states (stale signer, delta-chain-near-cap, predicate-passed-via-
//!    delta-chain) currently emit IMMUNITY hints because no `Tolerance*`
//!    intermediate-state variants exist yet in `AuditHint`. This is a
//!    documented v0.2 fix-direction; the assertions here pin the current
//!    behavior so when the v0.2 fix lands, this file's test will fail
//!    and serve as a checklist prompt for cleaning up call-sites.
//!
//! ## The epistemic distinction
//!
//! Immunity says "this site is protected from failure-class X". Tolerance
//! says "this site presents failure-class X and that is acknowledged /
//! accepted by signers". These are CATEGORICALLY DIFFERENT claims; the
//! discriminator is the only thing that prevents a future caller (or a
//! human reading an audit report) from conflating them. Losing the
//! discriminator is the worst possible bug in this subsystem.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use antigen_attestation::{
    evaluate::evaluate_predicate_with_kind, predicate::SignerCurrency, AntigenIdentifier,
    AuditHint, EvaluatedPredicate, EvaluationContext, EvidenceKind, ItemRatification, Leaf,
    Predicate, Ratification, RatificationKind, SchemaVersion, SignatureStrength, Signer,
    SignerBasis, WitnessTier,
};
use chrono::NaiveDate;

// --- Test infrastructure ---

struct TolCtx {
    today: NaiveDate,
    cap: u32,
}

impl EvaluationContext for TolCtx {
    fn today(&self) -> NaiveDate {
        self.today
    }
    fn read_doc(&self, _path: &Path) -> Option<String> {
        None
    }
    fn read_oracle(&self, _path: &Path) -> Option<String> {
        None
    }
    fn read_git_trailers(&self, _item_source_file: &Path, _item_path: &str) -> Vec<String> {
        vec![]
    }
    fn delta_chain_cap(&self) -> u32 {
        self.cap
    }
}

const fn sample_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 5, 19).expect("hard-coded valid date")
}

fn current_signer(name: &str) -> Signer {
    Signer {
        name: name.to_string(),
        role: None,
        date: sample_date(),
        signed_against_fingerprint: "fp-current".to_string(),
        basis: SignerBasis::Fresh { reasoning: None },
        strength: SignatureStrength::GitTrust,
        signature: None,
    }
}

fn stale_signer(name: &str) -> Signer {
    Signer {
        name: name.to_string(),
        role: None,
        date: sample_date(),
        signed_against_fingerprint: "fp-old".to_string(),
        basis: SignerBasis::Fresh { reasoning: None },
        strength: SignatureStrength::GitTrust,
        signature: None,
    }
}

fn delta_chain_signer(name: &str, chain_depth: u32) -> Signer {
    Signer {
        name: name.to_string(),
        role: None,
        date: sample_date(),
        signed_against_fingerprint: "fp-current".to_string(),
        basis: SignerBasis::DeltaFrom {
            prior_fingerprint: "fp-prior".to_string(),
            cumulative_root_fingerprint: if chain_depth == 1 {
                "fp-prior".to_string()
            } else {
                "fp-root".to_string()
            },
            chain_depth,
            rationale: "reviewed; consistent".to_string(),
        },
        strength: SignatureStrength::GitTrust,
        signature: None,
    }
}

fn item_with(signers: Vec<Signer>) -> ItemRatification {
    ItemRatification {
        item_path: "sinh".to_string(),
        current_fingerprint: "fp-current".to_string(),
        doc_ref: None,
        signers,
        oracles: vec![],
        fresh_through: None,
        extensions: BTreeMap::new(),
    }
}

fn require_alice() -> Predicate {
    Predicate::leaf(Leaf::Signers {
        required: vec!["alice".to_string()],
        roles: BTreeMap::new(),
        against: SignerCurrency::Any,
        signature_allow: vec![],
        signature_prefer: None,
    })
}

fn require_alice_current() -> Predicate {
    Predicate::leaf(Leaf::Signers {
        required: vec!["alice".to_string()],
        roles: BTreeMap::new(),
        against: SignerCurrency::Current,
        signature_allow: vec![],
        signature_prefer: None,
    })
}

// --- 1. Vibes-grade constructor: no substrate consulted ---

#[test]
fn tolerance_vibes_grade_has_none_tier() {
    let r = EvaluatedPredicate::tolerance_vibes_grade();
    assert_eq!(
        r.witness_tier,
        WitnessTier::None,
        "vibes-grade ratification has no substrate to ground a higher tier"
    );
}

#[test]
fn tolerance_vibes_grade_has_none_evidence_kind() {
    let r = EvaluatedPredicate::tolerance_vibes_grade();
    assert_eq!(
        r.evidence_kind,
        EvidenceKind::None,
        "vibes-grade did NOT consult substrate; reporting SubstrateState would overclaim"
    );
}

#[test]
fn tolerance_vibes_grade_emits_vibes_grade_hint() {
    let r = EvaluatedPredicate::tolerance_vibes_grade();
    assert_eq!(r.audit_hint, AuditHint::ToleranceVibesGrade);
}

#[test]
fn tolerance_vibes_grade_has_no_signature_strength() {
    let r = EvaluatedPredicate::tolerance_vibes_grade();
    assert!(
        r.signature_strength.is_none(),
        "no signer was consulted; signature_strength must be None"
    );
}

// --- 2. Kind-discriminator serde: snake_case round-trip ---

#[test]
fn ratification_kind_tolerance_serializes_snake_case() {
    let s = serde_json::to_string(&RatificationKind::Tolerance).unwrap();
    assert_eq!(s, "\"tolerance\"");
}

#[test]
fn ratification_kind_immunity_serializes_snake_case() {
    let s = serde_json::to_string(&RatificationKind::Immunity).unwrap();
    assert_eq!(s, "\"immunity\"");
}

#[test]
fn ratification_kind_tolerance_round_trips() {
    let original = RatificationKind::Tolerance;
    let s = serde_json::to_string(&original).unwrap();
    let parsed: RatificationKind = serde_json::from_str(&s).unwrap();
    assert_eq!(parsed, original);
}

#[test]
fn ratification_with_explicit_tolerance_kind_parses() {
    // The kind discriminator MUST be present on every Ratification. A schema
    // change that defaults it to Immunity would silently swallow tolerance
    // sidecars; absence MUST be a parse error, not a default.
    let json = r#"{
        "schema_version": "v1",
        "kind": "tolerance",
        "antigen": { "name": "TestAntigen" },
        "source_file": "src/test.rs",
        "items": []
    }"#;
    let r: Ratification = serde_json::from_str(json).unwrap();
    assert_eq!(r.kind, RatificationKind::Tolerance);
    assert_eq!(r.schema_version, SchemaVersion::V1);
    assert_eq!(r.antigen.name, "TestAntigen");
}

#[test]
fn ratification_missing_kind_field_is_parse_error() {
    // The discriminator MUST be required, not optional with a default.
    // Tolerance and Immunity have isomorphic field layouts; if `kind`
    // defaulted to Immunity on absence, a malformed tolerance sidecar
    // (missing kind) would be silently misread as an immunity sidecar.
    let json = r#"{
        "schema_version": "v1",
        "antigen": { "name": "TestAntigen" },
        "source_file": "src/test.rs",
        "items": []
    }"#;
    let result: Result<Ratification, _> = serde_json::from_str(json);
    assert!(
        result.is_err(),
        "Ratification without explicit `kind` discriminator must be a parse error"
    );
}

#[test]
fn ratification_unknown_kind_value_is_parse_error() {
    // Unknown kind values (e.g., "warranty", "audit") MUST reject, not
    // silently fall back to any default arm of the enum.
    let json = r#"{
        "schema_version": "v1",
        "kind": "warranty",
        "antigen": { "name": "TestAntigen" },
        "source_file": "src/test.rs",
        "items": []
    }"#;
    let result: Result<Ratification, _> = serde_json::from_str(json);
    assert!(
        result.is_err(),
        "Ratification with unknown `kind` discriminator must be a parse error, not a silent fallback"
    );
}

// --- 3. Terminal-hint discrimination on pass/fail ---

#[test]
fn tolerance_predicate_passed_substrate_current_terminal_hint() {
    // Tolerance sidecar with an all-current alice + alice-required-against-any
    // predicate: predicate passes, no stale, no delta → terminal pass hint
    // must be `TolerancePredicatePassedSubstrateCurrent`, NOT the immunity
    // equivalent.
    let item = item_with(vec![current_signer("alice")]);
    let ctx = TolCtx {
        today: sample_date(),
        cap: 3,
    };
    let r = evaluate_predicate_with_kind(
        &require_alice(),
        &item,
        "fp-current",
        Path::new("src/test.rs"),
        RatificationKind::Tolerance,
        &ctx,
    )
    .unwrap();
    assert_eq!(
        r.audit_hint,
        AuditHint::TolerancePredicatePassedSubstrateCurrent,
        "tolerance kind MUST emit Tolerance* terminal hint, NOT Discipline* equivalent"
    );
    assert_eq!(r.witness_tier, WitnessTier::Execution);
    assert_eq!(r.evidence_kind, EvidenceKind::SubstrateState);
}

#[test]
fn immunity_predicate_passed_substrate_current_terminal_hint() {
    // Symmetric to above: immunity sidecar MUST emit Discipline* hint, NOT
    // Tolerance*. Together with the test above, this pins the discriminator-
    // to-hint mapping at the terminal-pass branch.
    let item = item_with(vec![current_signer("alice")]);
    let ctx = TolCtx {
        today: sample_date(),
        cap: 3,
    };
    let r = evaluate_predicate_with_kind(
        &require_alice(),
        &item,
        "fp-current",
        Path::new("src/test.rs"),
        RatificationKind::Immunity,
        &ctx,
    )
    .unwrap();
    assert_eq!(
        r.audit_hint,
        AuditHint::DisciplinePredicatePassedSubstrateCurrent,
        "immunity kind MUST emit Discipline* terminal hint, NOT Tolerance* equivalent"
    );
}

#[test]
fn tolerance_predicate_failed_terminal_hint() {
    // Tolerance sidecar with no signers + a `signers(required=[alice], current)`
    // predicate: predicate fails (alice not present) → terminal fail hint must
    // be `TolerancePredicateFailed`.
    let item = item_with(vec![]);
    let ctx = TolCtx {
        today: sample_date(),
        cap: 3,
    };
    let r = evaluate_predicate_with_kind(
        &require_alice_current(),
        &item,
        "fp-current",
        Path::new("src/test.rs"),
        RatificationKind::Tolerance,
        &ctx,
    )
    .unwrap();
    assert_eq!(
        r.audit_hint,
        AuditHint::TolerancePredicateFailed,
        "tolerance kind MUST emit TolerancePredicateFailed on terminal fail, NOT immunity equivalent"
    );
    assert_eq!(r.witness_tier, WitnessTier::None);
    assert_eq!(
        r.evidence_kind,
        EvidenceKind::SubstrateState,
        "predicate-failed consulted substrate; evidence_kind is SubstrateState even with None tier"
    );
}

#[test]
fn immunity_predicate_failed_terminal_hint() {
    let item = item_with(vec![]);
    let ctx = TolCtx {
        today: sample_date(),
        cap: 3,
    };
    let r = evaluate_predicate_with_kind(
        &require_alice_current(),
        &item,
        "fp-current",
        Path::new("src/test.rs"),
        RatificationKind::Immunity,
        &ctx,
    )
    .unwrap();
    assert_eq!(
        r.audit_hint,
        AuditHint::DisciplinePredicateFailed,
        "immunity kind MUST emit DisciplinePredicateFailed on terminal fail"
    );
}

// --- 4. v0.2 gap documentation: intermediate-state hints leak immunity ---

#[test]
fn tolerance_stale_signer_emits_immunity_stale_hint_v02_documented_gap() {
    // DOCUMENTED v0.2 GAP. The `classify_passed_predicate` intermediate-state
    // branches (stale, delta-near-cap, via-delta) do NOT thread RatificationKind;
    // they unconditionally emit Discipline* variants. There are no Tolerance*
    // equivalents in AuditHint for these intermediate states.
    //
    // ADR-019 v0.2 fix direction: add `ToleranceSubstrateStale`,
    // `ToleranceDeltaChainNearCap`, `TolerancePredicatePassedViaDeltaChain`
    // variants and thread `kind` through the intermediate branches.
    //
    // When that v0.2 fix lands, this assertion will fail — that failure is
    // INTENTIONAL: it serves as a checklist prompt to update all callers and
    // CI gates that branch on the hint name.
    let item = item_with(vec![stale_signer("alice")]);
    let ctx = TolCtx {
        today: sample_date(),
        cap: 3,
    };
    let r = evaluate_predicate_with_kind(
        &require_alice(),
        &item,
        "fp-current",
        Path::new("src/test.rs"),
        RatificationKind::Tolerance,
        &ctx,
    )
    .unwrap();
    assert_eq!(
        r.audit_hint,
        AuditHint::DisciplineSubstrateStale,
        "v0.2 GAP: tolerance sidecar with stale signer emits Discipline* hint; \
         ToleranceSubstrateStale does not exist yet. When this fails, the v0.2 \
         intermediate-hint discrimination fix has landed and call-sites need audit."
    );
    assert_eq!(r.witness_tier, WitnessTier::Reachability);
}

#[test]
fn tolerance_delta_chain_near_cap_emits_immunity_hint_v02_documented_gap() {
    // Same v0.2 gap as above, for the delta-chain-near-cap intermediate state.
    // cap = 3 → max_chain_depth >= 2 triggers near-cap. Use chain_depth = 2.
    let item = item_with(vec![delta_chain_signer("alice", 2)]);
    let ctx = TolCtx {
        today: sample_date(),
        cap: 3,
    };
    let r = evaluate_predicate_with_kind(
        &require_alice(),
        &item,
        "fp-current",
        Path::new("src/test.rs"),
        RatificationKind::Tolerance,
        &ctx,
    )
    .unwrap();
    assert_eq!(
        r.audit_hint,
        AuditHint::DisciplineSubstrateDeltaChainNearCap,
        "v0.2 GAP: tolerance sidecar with delta-chain-near-cap emits Discipline* hint"
    );
    assert_eq!(r.witness_tier, WitnessTier::Execution);
}

#[test]
fn tolerance_via_delta_chain_emits_immunity_hint_v02_documented_gap() {
    // Same v0.2 gap, for the via-delta-chain intermediate state (depth < near-cap
    // threshold but a delta exists). chain_depth = 1 with cap = 3 → not near-cap
    // (needs >= 2), but has_delta = true.
    let item = item_with(vec![delta_chain_signer("alice", 1)]);
    let ctx = TolCtx {
        today: sample_date(),
        cap: 3,
    };
    let r = evaluate_predicate_with_kind(
        &require_alice(),
        &item,
        "fp-current",
        Path::new("src/test.rs"),
        RatificationKind::Tolerance,
        &ctx,
    )
    .unwrap();
    assert_eq!(
        r.audit_hint,
        AuditHint::DisciplinePredicatePassedViaDeltaChain,
        "v0.2 GAP: tolerance sidecar with via-delta-chain emits Discipline* hint"
    );
    assert_eq!(r.witness_tier, WitnessTier::Execution);
}

// --- 5. AuditHint serde discriminates kebab-case-stable ---

#[test]
fn tolerance_vibes_grade_hint_serializes_kebab_case() {
    let s = serde_json::to_string(&AuditHint::ToleranceVibesGrade).unwrap();
    assert_eq!(s, "\"tolerance-vibes-grade\"");
}

#[test]
fn tolerance_predicate_passed_substrate_current_serializes_kebab_case() {
    let s = serde_json::to_string(&AuditHint::TolerancePredicatePassedSubstrateCurrent).unwrap();
    assert_eq!(s, "\"tolerance-predicate-passed-substrate-current\"");
}

#[test]
fn tolerance_predicate_failed_serializes_kebab_case() {
    let s = serde_json::to_string(&AuditHint::TolerancePredicateFailed).unwrap();
    assert_eq!(s, "\"tolerance-predicate-failed\"");
}

// --- 6. Symmetric-error: zero-leaf composition rejected for both kinds ---

#[test]
fn zero_leaf_composition_emits_schema_invalid_for_tolerance_kind() {
    // ATK-019-TOL: a malformed sidecar with a zero-leaf AllOf must be rejected
    // identically whether parsed as immunity or tolerance — the schema-invalid
    // path doesn't depend on the discriminator.
    let item = item_with(vec![current_signer("alice")]);
    let ctx = TolCtx {
        today: sample_date(),
        cap: 3,
    };
    let bad_predicate = Predicate::AllOf { children: vec![] };
    let r = evaluate_predicate_with_kind(
        &bad_predicate,
        &item,
        "fp-current",
        Path::new("src/test.rs"),
        RatificationKind::Tolerance,
        &ctx,
    )
    .unwrap();
    assert_eq!(r.audit_hint, AuditHint::DisciplineSidecarSchemaInvalid);
    assert_eq!(r.witness_tier, WitnessTier::None);
    assert_eq!(
        r.evidence_kind,
        EvidenceKind::SubstrateState,
        "schema-invalid still inspected the substrate; evidence_kind is SubstrateState"
    );
}

#[test]
fn zero_leaf_composition_emits_schema_invalid_for_immunity_kind() {
    let item = item_with(vec![current_signer("alice")]);
    let ctx = TolCtx {
        today: sample_date(),
        cap: 3,
    };
    let bad_predicate = Predicate::AllOf { children: vec![] };
    let r = evaluate_predicate_with_kind(
        &bad_predicate,
        &item,
        "fp-current",
        Path::new("src/test.rs"),
        RatificationKind::Immunity,
        &ctx,
    )
    .unwrap();
    assert_eq!(r.audit_hint, AuditHint::DisciplineSidecarSchemaInvalid);
}

// --- 7. AntigenIdentifier sanity-tying to fixture (compile-witness) ---

#[test]
fn antigen_identifier_constructs_without_defined_in() {
    // Anchor that the fixture builder doesn't need `defined_in` populated for
    // basic tolerance-test construction. Catches a future schema change that
    // would make `defined_in` required.
    let id = AntigenIdentifier {
        name: "TestTolerance".to_string(),
        defined_in: None,
    };
    assert_eq!(id.name, "TestTolerance");
    assert!(id.defined_in.is_none());
}

// --- 8. Tolerance / Immunity discriminator round-trip via full Ratification ---

#[test]
fn full_ratification_tolerance_roundtrips_with_kind_preserved() {
    // Full-shape round-trip: build a Ratification with kind=Tolerance,
    // serialize, parse, confirm kind survives unchanged.
    let original = Ratification {
        schema_version: SchemaVersion::V1,
        kind: RatificationKind::Tolerance,
        antigen: AntigenIdentifier {
            name: "TestTolerance".to_string(),
            defined_in: None,
        },
        source_file: PathBuf::from("src/test.rs"),
        items: vec![],
    };
    let s = serde_json::to_string(&original).unwrap();
    let parsed: Ratification = serde_json::from_str(&s).unwrap();
    assert_eq!(
        parsed.kind,
        RatificationKind::Tolerance,
        "kind discriminator MUST survive serialize/deserialize round-trip unchanged"
    );
}

#[test]
fn full_ratification_immunity_roundtrips_with_kind_preserved() {
    let original = Ratification {
        schema_version: SchemaVersion::V1,
        kind: RatificationKind::Immunity,
        antigen: AntigenIdentifier {
            name: "TestImmunity".to_string(),
            defined_in: None,
        },
        source_file: PathBuf::from("src/test.rs"),
        items: vec![],
    };
    let s = serde_json::to_string(&original).unwrap();
    let parsed: Ratification = serde_json::from_str(&s).unwrap();
    assert_eq!(parsed.kind, RatificationKind::Immunity);
}
