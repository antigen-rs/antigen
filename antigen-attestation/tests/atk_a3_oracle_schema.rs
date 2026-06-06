//! Adversarial precision tests for the ADR-021 Oracle artifact-class
//! schema invariants.
//!
//! Locks down: `OracleRef` tagged-union serde round-trip; legacy
//! `OracleRef { path, status }` two-pass deserialization fallback;
//! Oracle minimum-2-stewards (ATK-021-13); `authorization_basis` non-empty
//! (Amendment 2 inheritance); `StateTransition.rationale` non-empty
//! (Amendment 2 per transition); `authorized_by ∈ stewards[*].name`
//! (ATK-021-15 orphaned-authorization guard); chronological monotonicity
//! across the transitions log; well-formed state-machine `from`-chain;
//! `OracleState` serde rendering matches `StateTransition.from`/`to`
//! convention.

use std::collections::BTreeMap;
use std::path::PathBuf;

use antigen_attestation::{
    AntigenIdentifier, ItemRatification, Oracle, OracleRef, OracleState, OracleVersion, Provenance,
    Ratification, RatificationKind, SchemaVersion, StateTransition, Steward,
    schema::{DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS},
};
use chrono::NaiveDate;

const fn sample_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 5, 19).expect("hard-coded valid date")
}

fn two_stewards() -> Vec<Steward> {
    vec![
        Steward {
            name: "alice".to_string(),
            role: Some("tech-lead".to_string()),
            authorization_basis: "appointed by team-lead 2026-03-01".to_string(),
        },
        Steward {
            name: "bob".to_string(),
            role: Some("domain-authority".to_string()),
            authorization_basis: "appointed by team-lead 2026-03-01".to_string(),
        },
    ]
}

fn oracle_complete_with_stewards(id: &str, stewards: Vec<Steward>) -> Oracle {
    Oracle {
        id: id.to_string(),
        reference: OracleRef::LocalFile {
            path: PathBuf::from(format!("docs/oracles/{id}.md")),
            status_field: None,
            expected_status: None,
        },
        state: OracleState::Complete,
        stewards,
        created: Provenance {
            recorded_by: "alice".to_string(),
            at: sample_date(),
        },
        version: OracleVersion {
            pinned: "v1.0.0".to_string(),
            pinned_at: sample_date(),
        },
        transitions: vec![],
        extensions: BTreeMap::new(),
    }
}

fn ratification_with_oracles(oracles: Vec<Oracle>) -> Ratification {
    Ratification {
        schema_version: SchemaVersion::V1,
        kind: RatificationKind::Immunity,
        antigen: AntigenIdentifier {
            name: "TestAntigen".to_string(),
            defined_in: None,
        },
        source_file: PathBuf::from("src/test.rs"),
        items: vec![ItemRatification {
            item_path: "sinh".to_string(),
            current_fingerprint: "fp-current".to_string(),
            doc_ref: None,
            signers: vec![],
            oracles,
            fresh_through: None,
            extensions: BTreeMap::new(),
        }],
    }
}

// ============================================================================
// 1. OracleRef tagged-union serde round-trip
// ============================================================================

#[test]
fn oracle_ref_local_file_serializes_with_snake_case_kind() {
    let r = OracleRef::LocalFile {
        path: PathBuf::from("docs/oracle.md"),
        status_field: None,
        expected_status: None,
    };
    let s = serde_json::to_string(&r).unwrap();
    assert!(
        s.contains("\"kind\":\"local_file\""),
        "OracleRef::LocalFile must serialize with kind=local_file: {s}"
    );
}

#[test]
fn oracle_ref_url_serializes_with_snake_case_kind() {
    let r = OracleRef::Url {
        url: "https://example.com/spec".to_string(),
        label: None,
    };
    let s = serde_json::to_string(&r).unwrap();
    assert!(s.contains("\"kind\":\"url\""), "OracleRef::Url: {s}");
}

#[test]
fn oracle_ref_doi_serializes_with_snake_case_kind() {
    let r = OracleRef::Doi {
        doi: "10.1145/3593856".to_string(),
        section: Some("6.3".to_string()),
    };
    let s = serde_json::to_string(&r).unwrap();
    assert!(s.contains("\"kind\":\"doi\""), "OracleRef::Doi: {s}");
}

#[test]
fn oracle_ref_arxiv_serializes_with_snake_case_kind() {
    let r = OracleRef::Arxiv {
        arxiv_id: "2401.12345".to_string(),
        section: None,
    };
    let s = serde_json::to_string(&r).unwrap();
    assert!(s.contains("\"kind\":\"arxiv\""), "OracleRef::Arxiv: {s}");
}

#[test]
fn oracle_ref_github_issue_serializes_with_snake_case_kind() {
    let r = OracleRef::GitHubIssue {
        repo: "antigen-rs/antigen".to_string(),
        issue: 42,
    };
    let s = serde_json::to_string(&r).unwrap();
    assert!(
        s.contains("\"kind\":\"git_hub_issue\""),
        "OracleRef::GitHubIssue must use serde snake_case (git_hub_issue): {s}"
    );
}

#[test]
fn oracle_ref_other_uses_subkind_not_kind_to_avoid_serde_tag_collision() {
    // Critical: `Other { kind, ... }` would clash with the outer serde
    // discriminator tag (also `kind`). Renamed to `subkind` to avoid.
    let r = OracleRef::Other {
        subkind: "rfc".to_string(),
        reference: "RFC 8259".to_string(),
        label: None,
    };
    let s = serde_json::to_string(&r).unwrap();
    assert!(
        s.contains("\"kind\":\"other\""),
        "Other variant kind discriminator: {s}"
    );
    assert!(
        s.contains("\"subkind\":\"rfc\""),
        "Other variant must use subkind, not kind, for the sub-discriminator: {s}"
    );
}

#[test]
fn oracle_ref_round_trips_all_variants() {
    let variants = vec![
        OracleRef::LocalFile {
            path: PathBuf::from("a.md"),
            status_field: Some("status".to_string()),
            expected_status: Some("complete".to_string()),
        },
        OracleRef::Url {
            url: "https://example.com".to_string(),
            label: Some("spec".to_string()),
        },
        OracleRef::Doi {
            doi: "10.1145/123".to_string(),
            section: None,
        },
        OracleRef::Arxiv {
            arxiv_id: "2401.0001".to_string(),
            section: Some("§3".to_string()),
        },
        OracleRef::GitHubIssue {
            repo: "owner/repo".to_string(),
            issue: 1,
        },
        OracleRef::Other {
            subkind: "rfc".to_string(),
            reference: "RFC 8259".to_string(),
            label: None,
        },
    ];
    for v in variants {
        let s = serde_json::to_string(&v).unwrap();
        let parsed: OracleRef = serde_json::from_str(&s).unwrap();
        assert_eq!(
            parsed, v,
            "OracleRef variant must round-trip through serde unchanged: {s}"
        );
    }
}

// ============================================================================
// 2. Legacy OracleRef { path, status } two-pass deserialization fallback
// ============================================================================
//
// Per ADR-021 §D2: the field-shape change from Vec<OracleRef> (v0.0 struct)
// to Vec<Oracle> (v0.1 artifact-class) is the LAST breaking schema change.
// Existing v0.0 sidecars MUST continue to parse so v0.1 audit can emit the
// `oracle-ref-needs-migration` hint without forcing operator hand-edits.

#[test]
fn legacy_oracleref_shape_parses_via_two_pass_fallback() {
    let json = r#"{
        "schema_version": "v1",
        "kind": "immunity",
        "antigen": { "name": "Legacy" },
        "source_file": "src/test.rs",
        "items": [{
            "item_path": "y",
            "current_fingerprint": "fp",
            "oracles": [
                { "path": "docs/legacy_oracle.md", "status": "complete" }
            ]
        }]
    }"#;
    let r: Ratification = serde_json::from_str(json).unwrap();
    assert_eq!(r.items.len(), 1);
    assert_eq!(r.items[0].oracles.len(), 1);
    let oracle = &r.items[0].oracles[0];
    assert!(
        matches!(
            &oracle.reference,
            OracleRef::LocalFile { path, .. } if path == &PathBuf::from("docs/legacy_oracle.md")
        ),
        "legacy OracleRef must lift into OracleRef::LocalFile: {:?}",
        oracle.reference
    );
    assert_eq!(
        oracle.created.recorded_by, "<legacy-import>",
        "legacy-import provenance marker must be set so the audit can emit \
         oracle-ref-needs-migration hint"
    );
    assert_eq!(
        oracle.state,
        OracleState::Draft,
        "legacy-import state must default to Draft so signers cannot attest \
         until operator runs `oracle declare --import-legacy`"
    );
}

#[test]
fn legacy_import_oracle_exempt_from_minimum_2_stewards_check() {
    // Legacy-import sidecars carry empty steward list; they're exempt from
    // the minimum-2 check until the operator runs `oracle declare
    // --import-legacy`. The audit emits `oracle-ref-needs-migration` instead.
    let json = r#"{
        "schema_version": "v1",
        "kind": "immunity",
        "antigen": { "name": "Legacy" },
        "source_file": "src/test.rs",
        "items": [{
            "item_path": "y",
            "current_fingerprint": "fp",
            "oracles": [{ "path": "docs/o.md" }]
        }]
    }"#;
    let r: Ratification = serde_json::from_str(json).unwrap();
    // Validation passes despite no stewards because of legacy-import marker.
    r.validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
        .expect("legacy-import oracle should pass validate() despite 0 stewards");
}

// ============================================================================
// 3. Minimum 2 stewards at creation (ATK-021-13)
// ============================================================================

#[test]
fn oracle_with_zero_stewards_fails_validation() {
    let oracle = oracle_complete_with_stewards("z0", vec![]);
    let r = ratification_with_oracles(vec![oracle]);
    let err = r
        .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
        .expect_err("0-steward oracle MUST fail validation (ATK-021-13)");
    assert!(
        format!("{err}").contains("minimum 2 required"),
        "error must reference minimum-2 stewards: {err}"
    );
}

#[test]
fn oracle_with_one_steward_fails_validation() {
    let oracle = oracle_complete_with_stewards(
        "z1",
        vec![Steward {
            name: "alice".to_string(),
            role: None,
            authorization_basis: "lone steward".to_string(),
        }],
    );
    let r = ratification_with_oracles(vec![oracle]);
    let err = r
        .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
        .expect_err("1-steward oracle MUST fail validation (ATK-021-13)");
    assert!(format!("{err}").contains("minimum 2 required"));
}

#[test]
fn oracle_with_two_stewards_passes_validation() {
    let oracle = oracle_complete_with_stewards("z2", two_stewards());
    let r = ratification_with_oracles(vec![oracle]);
    r.validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
        .expect("2-steward oracle MUST pass minimum-stewards check");
}

// ============================================================================
// 4. Steward authorization_basis non-empty (Amendment 2 inheritance)
// ============================================================================

#[test]
fn steward_with_empty_authorization_basis_fails_validation() {
    let oracle = oracle_complete_with_stewards(
        "empty-basis",
        vec![
            Steward {
                name: "alice".to_string(),
                role: None,
                authorization_basis: String::new(),
            },
            Steward {
                name: "bob".to_string(),
                role: None,
                authorization_basis: "ok basis".to_string(),
            },
        ],
    );
    let r = ratification_with_oracles(vec![oracle]);
    let err = r
        .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
        .expect_err("empty authorization_basis MUST fail (Amendment 2)");
    assert!(format!("{err}").contains("authorization_basis"));
}

#[test]
fn steward_with_whitespace_only_authorization_basis_fails_validation() {
    let oracle = oracle_complete_with_stewards(
        "ws-basis",
        vec![
            Steward {
                name: "alice".to_string(),
                role: None,
                authorization_basis: "   \t  \n  ".to_string(),
            },
            Steward {
                name: "bob".to_string(),
                role: None,
                authorization_basis: "ok".to_string(),
            },
        ],
    );
    let r = ratification_with_oracles(vec![oracle]);
    let err = r
        .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
        .expect_err("whitespace-only authorization_basis MUST fail");
    assert!(format!("{err}").contains("authorization_basis"));
}

// ============================================================================
// 5. StateTransition rationale non-empty (Amendment 2)
// ============================================================================

#[test]
fn transition_with_empty_rationale_fails_validation() {
    let mut oracle = oracle_complete_with_stewards("trans-empty", two_stewards());
    oracle.transitions = vec![StateTransition {
        from: "draft".to_string(),
        to: "complete".to_string(),
        authorized_by: "alice".to_string(),
        at: sample_date(),
        rationale: String::new(),
    }];
    let r = ratification_with_oracles(vec![oracle]);
    let err = r
        .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
        .expect_err("empty transition rationale MUST fail (Amendment 2)");
    assert!(format!("{err}").contains("rationale"));
}

// ============================================================================
// 6. authorized_by must appear in stewards[*].name (ATK-021-15)
// ============================================================================

#[test]
fn transition_authorized_by_non_steward_fails_validation() {
    let mut oracle = oracle_complete_with_stewards("orphan-auth", two_stewards());
    oracle.transitions = vec![StateTransition {
        from: "draft".to_string(),
        to: "complete".to_string(),
        authorized_by: "carol".to_string(), // not in stewards
        at: sample_date(),
        rationale: "moving to complete after review".to_string(),
    }];
    let r = ratification_with_oracles(vec![oracle]);
    let err = r
        .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
        .expect_err("authorized_by non-steward MUST fail (ATK-021-15)");
    assert!(format!("{err}").contains("does not match any declared steward"));
}

#[test]
fn transition_authorized_by_known_steward_passes() {
    let mut oracle = oracle_complete_with_stewards("ok-auth", two_stewards());
    oracle.transitions = vec![StateTransition {
        from: "draft".to_string(),
        to: "complete".to_string(),
        authorized_by: "alice".to_string(), // in stewards
        at: sample_date(),
        rationale: "moving to complete after review".to_string(),
    }];
    let r = ratification_with_oracles(vec![oracle]);
    r.validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
        .expect("steward-authorized transition should validate");
}

// ============================================================================
// 7. Chronological monotonicity across transitions log
// ============================================================================

#[test]
fn out_of_order_transitions_fail_validation() {
    let mut oracle = oracle_complete_with_stewards("mono", two_stewards());
    oracle.transitions = vec![
        StateTransition {
            from: "draft".to_string(),
            to: "complete".to_string(),
            authorized_by: "alice".to_string(),
            at: NaiveDate::from_ymd_opt(2026, 5, 19).unwrap(),
            rationale: "moving to complete".to_string(),
        },
        StateTransition {
            from: "complete".to_string(),
            to: "deprecated".to_string(),
            authorized_by: "bob".to_string(),
            at: NaiveDate::from_ymd_opt(2026, 5, 10).unwrap(), // older than prior
            rationale: "deprecating".to_string(),
        },
    ];
    let r = ratification_with_oracles(vec![oracle]);
    let err = r
        .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
        .expect_err("out-of-order transition dates MUST fail");
    assert!(format!("{err}").contains("older than the previous"));
}

// ============================================================================
// 8. State-machine from-chain validity
// ============================================================================

#[test]
fn transition_from_mismatch_fails_validation() {
    let mut oracle = oracle_complete_with_stewards("chain", two_stewards());
    // First transition: draft→complete (oracle starts implicitly at Draft).
    // Second transition: claims `from: draft` but should be `from: complete`.
    oracle.transitions = vec![
        StateTransition {
            from: "draft".to_string(),
            to: "complete".to_string(),
            authorized_by: "alice".to_string(),
            at: NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
            rationale: "to complete".to_string(),
        },
        StateTransition {
            from: "draft".to_string(), // WRONG; should be "complete"
            to: "deprecated".to_string(),
            authorized_by: "bob".to_string(),
            at: NaiveDate::from_ymd_opt(2026, 5, 10).unwrap(),
            rationale: "to deprecated".to_string(),
        },
    ];
    let r = ratification_with_oracles(vec![oracle]);
    let err = r
        .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
        .expect_err("from-mismatch in transitions chain MUST fail");
    assert!(format!("{err}").contains("expected `complete`"));
}

#[test]
fn well_formed_transition_chain_passes_validation() {
    let mut oracle = oracle_complete_with_stewards("ok-chain", two_stewards());
    oracle.transitions = vec![
        StateTransition {
            from: "draft".to_string(),
            to: "complete".to_string(),
            authorized_by: "alice".to_string(),
            at: NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
            rationale: "complete after review".to_string(),
        },
        StateTransition {
            from: "complete".to_string(),
            to: "deprecated".to_string(),
            authorized_by: "bob".to_string(),
            at: NaiveDate::from_ymd_opt(2026, 5, 10).unwrap(),
            rationale: "deprecated in favor of v2".to_string(),
        },
    ];
    let r = ratification_with_oracles(vec![oracle]);
    r.validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
        .expect("well-formed transition chain should validate");
}

// ============================================================================
// 9. OracleState serde rendering matches transition discriminant convention
// ============================================================================

#[test]
fn oracle_state_serializes_with_snake_case_state_tag() {
    let states = vec![
        (OracleState::Draft, "draft"),
        (OracleState::Complete, "complete"),
    ];
    for (state, expected_tag) in states {
        let s = serde_json::to_string(&state).unwrap();
        let tag_pattern = format!("\"state\":\"{expected_tag}\"");
        assert!(
            s.contains(&tag_pattern),
            "OracleState rendering must use snake_case state tag: expected {tag_pattern}, got {s}"
        );
    }
}

#[test]
fn oracle_state_deprecated_carries_superseded_by_and_reason() {
    let state = OracleState::Deprecated {
        superseded_by: Some("oracle-v2".to_string()),
        reason: "superseded by v2 methodology".to_string(),
    };
    let s = serde_json::to_string(&state).unwrap();
    let parsed: OracleState = serde_json::from_str(&s).unwrap();
    assert_eq!(parsed, state);
}

#[test]
fn oracle_state_revoked_carries_invalidates_prior_attestations_flag() {
    let state = OracleState::Revoked {
        reason: "fundamentally incorrect".to_string(),
        revoked_by: "alice".to_string(),
        invalidates_prior_attestations: true,
    };
    let s = serde_json::to_string(&state).unwrap();
    let parsed: OracleState = serde_json::from_str(&s).unwrap();
    assert_eq!(parsed, state);
    assert!(
        s.contains("\"invalidates_prior_attestations\":true"),
        "Revoked variant must serialize the invalidates flag: {s}"
    );
}

// ============================================================================
// 10. Oracle round-trip preserves all v0.1 fields
// ============================================================================

#[test]
fn full_oracle_round_trips_through_serde() {
    let oracle = Oracle {
        id: "higham-2002-section-6-3".to_string(),
        reference: OracleRef::Doi {
            doi: "10.1145/3593856".to_string(),
            section: Some("6.3".to_string()),
        },
        state: OracleState::Complete,
        stewards: two_stewards(),
        created: Provenance {
            recorded_by: "alice".to_string(),
            at: sample_date(),
        },
        version: OracleVersion {
            pinned: "2nd-ed".to_string(),
            pinned_at: sample_date(),
        },
        transitions: vec![StateTransition {
            from: "draft".to_string(),
            to: "complete".to_string(),
            authorized_by: "alice".to_string(),
            at: sample_date(),
            rationale: "two stewards reviewed methodology pattern".to_string(),
        }],
        extensions: BTreeMap::new(),
    };
    let s = serde_json::to_string(&oracle).unwrap();
    let parsed: Oracle = serde_json::from_str(&s).unwrap();
    assert_eq!(
        parsed, oracle,
        "Oracle must round-trip through serde unchanged"
    );
}
