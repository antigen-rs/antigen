//! Adversarial precision test for the v0.1 implicit-scope semantic
//! (P3e slice 7 / 7).
//!
//! ## What this test guards
//!
//! ADR-019 Stage 3 FA-3 fold pins the v0.1 fingerprint-under-scope semantic:
//! v0.1 uses ITEM-LEVEL fingerprints regardless of any future scope
//! parameter; file-scope (hash-of-all-items) is deferred to v0.2
//! ratification. This means:
//!
//! 1. **No `scope` field exists in v0.1 schema** — the schema does not yet
//!    carry an explicit scope discriminator. Each `ItemRatification` is its
//!    own scope unit; the parent `Ratification.source_file` is the file-
//!    grouping mechanism.
//! 2. **Per-item fingerprint independence** — two `ItemRatification`s with
//!    the SAME `source_file` and DIFFERENT `item_path` evaluate to
//!    independent freshness/staleness states. Item-A becoming stale does
//!    NOT mark Item-B stale.
//! 3. **Item-path is the scope identity** — fingerprint currency is checked
//!    per `current_fingerprint`-field-of-this-item, not file-wide nor
//!    workspace-wide.
//! 4. **Forward-compat invariant** — when v0.2 introduces an explicit scope
//!    field, it MUST default to `item` (matching v0.1 semantic) so old
//!    sidecars continue to parse and evaluate identically. A
//!    serde-deny-unknown-fields posture in v0.1 would break the forward
//!    path; this test pins that scope-related fields are silently allowed
//!    via the `extensions` open-integration-surface slot until v0.2.
//!
//! ## Why this slice matters
//!
//! Scope confusion is one of the highest-impact silent failures in the
//! attestation system. If a file-scoped audit incorrectly applies an
//! item-level fingerprint check (or vice versa), the entire freshness/
//! staleness model produces wrong answers across the workspace. Locking
//! the v0.1 per-item semantic explicitly prevents future scope-field
//! additions from accidentally breaking the established behavior.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use antigen_attestation::{
    evaluate::evaluate_predicate, predicate::SignerCurrency, AntigenIdentifier, AuditHint,
    EvaluationContext, ItemRatification, Leaf, Predicate, Ratification, RatificationKind,
    SchemaVersion, SignatureStrength, Signer, SignerBasis, WitnessTier,
};
use chrono::NaiveDate;

// --- Test infrastructure ---

struct ScopeCtx {
    today: NaiveDate,
}

impl EvaluationContext for ScopeCtx {
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
        3
    }
}

fn sample_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 5, 19).unwrap()
}

fn current_signer(name: &str, fp: &str) -> Signer {
    Signer {
        name: name.to_string(),
        role: None,
        date: sample_date(),
        signed_against_fingerprint: fp.to_string(),
        basis: SignerBasis::Fresh { reasoning: None },
        strength: SignatureStrength::GitTrust,
        signature: None,
    }
}

fn item_at(item_path: &str, current_fingerprint: &str, signers: Vec<Signer>) -> ItemRatification {
    ItemRatification {
        item_path: item_path.to_string(),
        current_fingerprint: current_fingerprint.to_string(),
        doc_ref: None,
        signers,
        oracles: vec![],
        fresh_through: None,
        extensions: BTreeMap::new(),
    }
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

// ============================================================================
// 1. v0.1 schema does NOT carry an explicit `scope` field
// ============================================================================
//
// A serde round-trip of a fully-populated Ratification + ItemRatification
// MUST NOT contain a `scope` key. If a future change adds the field with a
// non-defaulted serializer, this test catches it before silent on-disk
// schema drift ships.

#[test]
fn ratification_serialization_does_not_contain_scope_key() {
    let r = Ratification {
        schema_version: SchemaVersion::V1,
        kind: RatificationKind::Immunity,
        antigen: AntigenIdentifier {
            name: "TestAntigen".to_string(),
            defined_in: None,
        },
        source_file: PathBuf::from("src/test.rs"),
        items: vec![item_at(
            "sinh",
            "fp-x",
            vec![current_signer("alice", "fp-x")],
        )],
    };
    let s = serde_json::to_string(&r).unwrap();
    assert!(
        !s.contains("\"scope\""),
        "v0.1 schema MUST NOT serialize a `scope` field; \
         FA-3 fold deferred file-scope hash-of-all-items to v0.2: {s}"
    );
}

#[test]
fn item_ratification_serialization_does_not_contain_scope_key() {
    let item = item_at("sinh", "fp-x", vec![current_signer("alice", "fp-x")]);
    let s = serde_json::to_string(&item).unwrap();
    assert!(
        !s.contains("\"scope\""),
        "v0.1 ItemRatification MUST NOT serialize a `scope` field: {s}"
    );
}

// ============================================================================
// 2. Per-item fingerprint independence under shared source_file
// ============================================================================
//
// Two ItemRatifications sharing source_file but at different item_paths
// have INDEPENDENT freshness states. Item-A becoming stale does NOT
// mark Item-B stale.

#[test]
fn two_items_same_source_file_have_independent_currency() {
    let item_a = item_at(
        "sinh",
        "fp-current-a",
        vec![current_signer("alice", "fp-current-a")],
    );
    let item_b = item_at(
        "cosh",
        "fp-current-b",
        vec![current_signer("alice", "fp-current-b")],
    );
    let ctx = ScopeCtx {
        today: sample_date(),
    };

    // Item A evaluated against its own current fingerprint passes.
    let r_a = evaluate_predicate(
        &require_alice_current(),
        &item_a,
        "fp-current-a",
        Path::new("src/trig.rs"),
        &ctx,
    )
    .unwrap();
    assert_eq!(
        r_a.audit_hint,
        AuditHint::DisciplinePredicatePassedSubstrateCurrent,
        "item A is current against its own fingerprint"
    );

    // Item B evaluated against its own current fingerprint also passes —
    // independently of item A's state.
    let r_b = evaluate_predicate(
        &require_alice_current(),
        &item_b,
        "fp-current-b",
        Path::new("src/trig.rs"),
        &ctx,
    )
    .unwrap();
    assert_eq!(
        r_b.audit_hint,
        AuditHint::DisciplinePredicatePassedSubstrateCurrent,
        "item B is independent: its currency is checked against fp-current-b, not fp-current-a"
    );
}

#[test]
fn item_a_stale_does_not_propagate_to_item_b() {
    // Item A has only stale signers; item B has fresh signers. They share
    // source_file. The audit MUST NOT mark item B stale due to item A.
    let item_a = item_at(
        "sinh",
        "fp-current-a",
        vec![current_signer("alice", "fp-OLD-a")],
    );
    let item_b = item_at(
        "cosh",
        "fp-current-b",
        vec![current_signer("alice", "fp-current-b")],
    );
    let ctx = ScopeCtx {
        today: sample_date(),
    };

    let r_a = evaluate_predicate(
        &require_alice_current(),
        &item_a,
        "fp-current-a",
        Path::new("src/trig.rs"),
        &ctx,
    )
    .unwrap();
    let r_b = evaluate_predicate(
        &require_alice_current(),
        &item_b,
        "fp-current-b",
        Path::new("src/trig.rs"),
        &ctx,
    )
    .unwrap();

    // Item A's all-stale-signers state: predicate fails (against=Current,
    // alice has no signature against fp-current-a).
    assert_eq!(
        r_a.audit_hint,
        AuditHint::DisciplinePredicateFailed,
        "item A all-stale must fail predicate (against=Current)"
    );

    // Item B remains independently passing.
    assert_eq!(
        r_b.audit_hint,
        AuditHint::DisciplinePredicatePassedSubstrateCurrent,
        "item B independent passing state MUST NOT be affected by item A's failure"
    );
    assert_eq!(r_b.witness_tier, WitnessTier::Execution);
}

// ============================================================================
// 3. item_path is the scope-identity for fingerprint-currency checks
// ============================================================================
//
// The audit-side `current_fingerprint` argument is the per-item value. If
// the evaluator accidentally used a workspace-wide or file-wide fingerprint,
// items with distinct fingerprints would all share staleness/freshness
// state. Lock per-item fingerprint scoping.

#[test]
fn evaluator_uses_per_item_current_fingerprint_not_workspace_wide() {
    let item = item_at(
        "sinh",
        "fp-item-stored",
        vec![current_signer("alice", "fp-AUDIT")],
    );
    let ctx = ScopeCtx {
        today: sample_date(),
    };

    // Audit passes "fp-AUDIT" as the audit-time fingerprint (re-computed
    // for THIS item, not workspace-wide). Alice's signature matches "fp-AUDIT"
    // so predicate passes.
    let r = evaluate_predicate(
        &require_alice_current(),
        &item,
        "fp-AUDIT",
        Path::new("src/test.rs"),
        &ctx,
    )
    .unwrap();
    assert_eq!(
        r.audit_hint,
        AuditHint::DisciplinePredicatePassedSubstrateCurrent,
        "audit-time fingerprint is per-item; alice's matching signature passes"
    );
}

#[test]
fn item_path_distinguishes_independent_audit_invocations() {
    // Two items at the SAME source_file with INDEPENDENT item-paths each get
    // their own evaluate_predicate invocation. Each invocation's
    // `current_fingerprint` argument is THIS item's fingerprint, not shared.
    let item_sinh = item_at(
        "sinh",
        "fp-sinh-current",
        vec![current_signer("alice", "fp-sinh-current")],
    );
    let item_cosh = item_at(
        "cosh",
        "fp-cosh-current",
        vec![current_signer("alice", "fp-cosh-current")],
    );
    let ctx = ScopeCtx {
        today: sample_date(),
    };

    let r_sinh = evaluate_predicate(
        &require_alice_current(),
        &item_sinh,
        "fp-sinh-current",
        Path::new("src/math.rs"),
        &ctx,
    )
    .unwrap();
    let r_cosh = evaluate_predicate(
        &require_alice_current(),
        &item_cosh,
        "fp-cosh-current",
        Path::new("src/math.rs"),
        &ctx,
    )
    .unwrap();

    assert_eq!(
        r_sinh.audit_hint,
        AuditHint::DisciplinePredicatePassedSubstrateCurrent
    );
    assert_eq!(
        r_cosh.audit_hint,
        AuditHint::DisciplinePredicatePassedSubstrateCurrent
    );
}

// ============================================================================
// 4. Forward-compat: unknown fields in sidecar JSON do NOT break parse
// ============================================================================
//
// v0.2 will introduce an explicit `scope` field. For old sidecars to
// continue parsing under v0.2-aware code, the serde posture today MUST
// be tolerant of unknown fields (NOT deny_unknown_fields). Conversely,
// today's parser MUST tolerate a v0.2-shaped sidecar with `scope` set
// (the v0.2 ratification will require defaulting to `item` for back-
// compat with v0.1 semantic).

#[test]
fn unknown_field_in_ratification_does_not_break_parse() {
    let json = r#"{
        "schema_version": "v1",
        "kind": "immunity",
        "antigen": { "name": "TestAntigen" },
        "source_file": "src/test.rs",
        "items": [],
        "future_v02_field_with_unknown_shape": "something"
    }"#;
    let r: Result<Ratification, _> = serde_json::from_str(json);
    assert!(
        r.is_ok(),
        "Ratification parse MUST tolerate unknown fields for v0.2 forward-compat: {r:?}"
    );
}

#[test]
fn future_scope_field_value_does_not_break_parse() {
    // A v0.2-shaped sidecar where `scope: "item"` is set must parse cleanly
    // today (unknown field treated as part of the open extension surface).
    let json = r#"{
        "schema_version": "v1",
        "kind": "immunity",
        "antigen": { "name": "TestAntigen" },
        "source_file": "src/test.rs",
        "scope": "item",
        "items": []
    }"#;
    let r: Result<Ratification, _> = serde_json::from_str(json);
    assert!(
        r.is_ok(),
        "Ratification parse MUST tolerate future `scope` field for v0.2 forward-compat: {r:?}"
    );
}

// ============================================================================
// 5. `extensions` field is the v0.1 open-integration-surface for forward-compat
// ============================================================================
//
// ADR-019 §Posture names `extensions` as the open slot for v0.2+ amendments.
// v0.1 leaves it empty by convention; v0.2 can use it before promoting fields
// to first-class. Lock that arbitrary extension keys round-trip cleanly.

#[test]
fn extensions_field_roundtrips_arbitrary_json() {
    let mut ext = BTreeMap::new();
    ext.insert(
        "v02_proposal_scope".to_string(),
        serde_json::json!({ "kind": "item" }),
    );
    ext.insert("v04_lifetime_claim".to_string(), serde_json::json!(180_u32));

    let item = ItemRatification {
        item_path: "sinh".to_string(),
        current_fingerprint: "fp-x".to_string(),
        doc_ref: None,
        signers: vec![],
        oracles: vec![],
        fresh_through: None,
        extensions: ext.clone(),
    };

    let s = serde_json::to_string(&item).unwrap();
    let parsed: ItemRatification = serde_json::from_str(&s).unwrap();
    assert_eq!(
        parsed.extensions, ext,
        "extensions field round-trips arbitrary JSON values"
    );
}

#[test]
fn extensions_field_default_empty_omitted_from_serialization() {
    // Empty extensions skip_serializing_if pattern keeps v0.1 sidecars
    // clean by omitting the field rather than emitting `"extensions": {}`.
    let item = item_at("sinh", "fp-x", vec![]);
    let s = serde_json::to_string(&item).unwrap();
    assert!(
        !s.contains("\"extensions\""),
        "empty extensions field must be omitted from serialization, not emitted as {{}}: {s}"
    );
}

// ============================================================================
// 6. source_file is the file-grouping mechanism in v0.1
// ============================================================================
//
// Per ADR-019 §M3, the file-grouping primitive is Ratification.source_file,
// NOT a separate scope field. One Ratification per (source_file, antigen)
// pair; multiple ItemRatifications inside for multiple items presenting the
// same antigen.

#[test]
fn ratification_groups_multiple_items_under_one_source_file() {
    let r = Ratification {
        schema_version: SchemaVersion::V1,
        kind: RatificationKind::Immunity,
        antigen: AntigenIdentifier {
            name: "SignedZeroPreservation".to_string(),
            defined_in: None,
        },
        source_file: PathBuf::from("src/math/hyperbolic.rs"),
        items: vec![
            item_at("sinh", "fp-sinh", vec![current_signer("alice", "fp-sinh")]),
            item_at("cosh", "fp-cosh", vec![current_signer("alice", "fp-cosh")]),
            item_at("tanh", "fp-tanh", vec![current_signer("alice", "fp-tanh")]),
        ],
    };

    assert_eq!(
        r.items.len(),
        3,
        "one Ratification carries multiple items per source file"
    );
    assert_eq!(r.source_file, PathBuf::from("src/math/hyperbolic.rs"));

    // Each item has its own item_path and fingerprint — scope is per-item.
    let paths: Vec<&str> = r.items.iter().map(|i| i.item_path.as_str()).collect();
    assert_eq!(paths, vec!["sinh", "cosh", "tanh"]);

    let fingerprints: Vec<&str> = r
        .items
        .iter()
        .map(|i| i.current_fingerprint.as_str())
        .collect();
    assert_eq!(fingerprints, vec!["fp-sinh", "fp-cosh", "fp-tanh"]);
}

#[test]
fn ratification_round_trips_multi_item_grouping() {
    let original = Ratification {
        schema_version: SchemaVersion::V1,
        kind: RatificationKind::Immunity,
        antigen: AntigenIdentifier {
            name: "TestAntigen".to_string(),
            defined_in: None,
        },
        source_file: PathBuf::from("src/test.rs"),
        items: vec![
            item_at("item_a", "fp-a", vec![]),
            item_at("item_b", "fp-b", vec![]),
        ],
    };
    let s = serde_json::to_string(&original).unwrap();
    let parsed: Ratification = serde_json::from_str(&s).unwrap();
    assert_eq!(parsed.items.len(), 2);
    assert_eq!(parsed.items[0].item_path, "item_a");
    assert_eq!(parsed.items[1].item_path, "item_b");
    assert_eq!(parsed.source_file, original.source_file);
}
