//! ATK-SC evaluation-layer adversarial tests.
//!
//! These tests attack the supply-chain evaluator functions directly
//! (`antigen::supply_chain::evaluate`). They run against pathmaker's
//! implementation and SHOULD FAIL where the spec hasn't been fully enforced.
//!
//! Each test is named after the attack vector and contains a precise
//! description of the bypass risk.
//!
//! Tests that PASS indicate correct defense.
//! Tests that FAIL indicate a real implementation gap (a bug).

use antigen::supply_chain::evaluate::{
    dep_attest_path, evaluate_content_hash_matches, evaluate_dep_attested, save_content_hash_record,
};
use antigen::supply_chain::schema::{ContentHashRecord, DepAttestation, ReviewScope};
use antigen::supply_chain::witness::{ContentHashState, DepAttestedState};
use std::path::PathBuf;
use tempfile::TempDir;

// ============================================================================
// ATK-SC-1-A: Whitespace bypass of has_reviewable_artifact
//
// ATTACK: `reviewable_artifact = " "` (space) passes the is_empty() check
// because a space is not empty. This allows a rubber-stamp sidecar to
// claim a review artifact while providing a meaningless value.
//
// EXPECTED: AttestedWithoutReviewableArtifact (the bypass should NOT pass)
// CURRENT: Attested (the bypass PASSES — the bug)
//
// Per ADR-025: "reviewable_artifact is REQUIRED non-empty — empty = rubber-stamp."
// The spirit of the requirement is "non-empty AND meaningful," but the
// implementation only checks is_empty(), which allows whitespace.
// ============================================================================

#[test]
fn atk_sc1a_space_artifact_should_be_flagged_as_rubber_stamp() {
    let tmp = TempDir::new().unwrap();

    // Sidecar with a single space as reviewable_artifact.
    // This is NOT a real review artifact — it's a whitespace bypass.
    let att = DepAttestation {
        crate_name: "serde".to_string(),
        version: "1.0.197".to_string(),
        exact_version: true,
        reviewable_artifact: PathBuf::from(" "), // WHITESPACE BYPASS
        review_scope: ReviewScope::MetadataOnly,
        signed_by: "attacker".to_string(),
        date: "2026-05-22".to_string(),
        rationale: None,
    };
    let path = dep_attest_path(tmp.path(), "serde", "1.0.197");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string(&att).unwrap()).unwrap();

    let state = evaluate_dep_attested(tmp.path(), "serde", "1.0.197", true);

    // FAILS RIGHT NOW: current impl returns Attested because " ".is_empty() = false.
    // The test asserts what SHOULD be true per the spec.
    assert_eq!(
        state,
        DepAttestedState::AttestedWithoutReviewableArtifact,
        "ATK-SC-1-A: reviewable_artifact = ' ' (whitespace) must be treated as rubber-stamp. \
         A single space is not a meaningful review artifact. \
         has_reviewable_artifact() must check trim().is_empty(), not just is_empty(). \
         Current behavior allows whitespace bypass of the rubber-stamp check. \
         ADR-025: 'reviewable_artifact REQUIRED non-empty — empty = rubber-stamp.'"
    );
}

#[test]
fn atk_sc1a_dot_artifact_should_be_flagged_as_rubber_stamp() {
    let tmp = TempDir::new().unwrap();

    // Sidecar with "." as reviewable_artifact.
    // "." is a valid path (current directory) but is meaningless as a review doc.
    // This tests whether the check is semantic or purely syntactic.
    let att = DepAttestation {
        crate_name: "serde".to_string(),
        version: "1.0.197".to_string(),
        exact_version: true,
        reviewable_artifact: PathBuf::from("."), // DOT BYPASS
        review_scope: ReviewScope::MetadataOnly,
        signed_by: "attacker".to_string(),
        date: "2026-05-22".to_string(),
        rationale: None,
    };
    let path = dep_attest_path(tmp.path(), "serde", "1.0.197");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string(&att).unwrap()).unwrap();

    let state = evaluate_dep_attested(tmp.path(), "serde", "1.0.197", true);

    // NOTE: "." bypasses is_empty() but may be acceptable per the spec
    // (it's technically non-empty). Document the behavior.
    // The ADR doesn't specify that the artifact must EXIST on disk.
    // But "." is a directory, not a review document.
    //
    // This test captures current behavior. If the spec says "any non-empty path
    // is acceptable" then AttestedWithoutReviewableArtifact here would be wrong.
    // If the spec requires a meaningful path, Attested would be wrong.
    //
    // Document whichever state is returned:
    eprintln!(
        "ATK-SC-1-A-dot: reviewable_artifact='.' evaluates to: {:?}. \
         ADR-025 doesn't specify existence check — this is a named limitation.",
        state
    );
    // The test passes regardless (documentation, not assertion) for "."
    // because the spec is ambiguous here.
}

// ============================================================================
// ATK-SC-1-B: Reviewable artifact field in Leaf not compared to sidecar field
//
// ATTACK: The Leaf::DepAttested spec says "when Some, the sidecar's
// reviewable_artifact field must equal this path (string compare)."
// But the evaluator IGNORES the Leaf's reviewable_artifact field entirely.
//
// An attacker can:
// 1. Create sidecar with reviewable_artifact = "real-review.md"
// 2. Write predicate: dep_attested("serde", "1.0", reviewable_artifact = "fake-review.md")
// 3. Audit evaluates the predicate's artifact requirement against sidecar
//    Expected: FAIL (sidecar has "real-review.md", predicate requires "fake-review.md")
//    Current: PASS (evaluator doesn't check the predicate's artifact requirement)
//
// Note: This is adversarially contrived. The spec may intend the artifact field
// as documentation-only. Verify with pathmaker which interpretation is correct.
// ============================================================================

#[test]
fn atk_sc1b_predicate_artifact_not_matched_against_sidecar() {
    let tmp = TempDir::new().unwrap();

    // Sidecar with one artifact path
    let att = DepAttestation {
        crate_name: "serde".to_string(),
        version: "1.0.197".to_string(),
        exact_version: true,
        reviewable_artifact: PathBuf::from("reviews/serde-1.0.197.md"),
        review_scope: ReviewScope::Full,
        signed_by: "alice".to_string(),
        date: "2026-05-22".to_string(),
        rationale: None,
    };
    let path = dep_attest_path(tmp.path(), "serde", "1.0.197");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string(&att).unwrap()).unwrap();

    // The evaluator doesn't take a `required_artifact` parameter — it just
    // checks that the sidecar has a non-empty artifact. The Leaf's
    // `reviewable_artifact` field (when Some) is NOT passed to the evaluator.
    // This test documents this gap.
    let state = evaluate_dep_attested(tmp.path(), "serde", "1.0.197", true);
    assert_eq!(
        state,
        DepAttestedState::Attested {
            review_scope: ReviewScope::Full
        },
        "Baseline: sidecar with valid artifact evaluates to Attested"
    );

    // The evaluator signature doesn't accept a required_artifact parameter.
    // To verify the Leaf's reviewable_artifact field is checked, we'd need
    // either:
    //   evaluate_dep_attested(root, crate, version, exact_version, required_artifact?)
    // or a separate evaluation path for Leaf::DepAttested with Some(artifact).
    //
    // FINDING: evaluate_dep_attested doesn't accept a required_artifact constraint.
    // The predicate spec says when reviewable_artifact is Some, the sidecar must match.
    // But the evaluator has no mechanism to enforce this constraint.
    // ADR ref: Leaf::DepAttested comment ("when Some, the sidecar's reviewable_artifact
    // field must equal this path").
    eprintln!(
        "ATK-SC-1-B: evaluate_dep_attested has no required_artifact parameter. \
         Leaf::DepAttested.reviewable_artifact = Some(path) cannot be enforced. \
         Predicate spec says this must match sidecar — not implemented."
    );
}

// ============================================================================
// ATK-SC-2-A: Malformed content-hash sidecar treated as NoAttestation
//
// ATTACK: load_content_hash_record uses .ok() to silently discard malformed JSON.
// If the sidecar is corrupted (e.g., by an attacker who has write access),
// evaluate_content_hash_matches returns NoAttestation instead of an error.
//
// The severity difference:
//   NoAttestation → audit hint: content-hash-no-attestation (WARNING)
//   Mismatch → audit hint: content-hash-mismatch (ERROR)
//
// An attacker who can corrupt the sidecar can downgrade a Mismatch (ERROR)
// to NoAttestation (WARNING), reducing the urgency of the alert.
//
// Contrast: evaluate_dep_attested correctly handles malformed JSON:
//   DepAttestedState::SidecarMalformed { error: e.to_string() }
//
// The inconsistency: dep-attest sidecars surface malformation explicitly;
// content-hash sidecars silently treat malformation as absence.
// ============================================================================

#[test]
fn atk_sc2a_malformed_content_hash_sidecar_should_not_be_silent() {
    let tmp = TempDir::new().unwrap();

    // First, write a VALID sidecar (establish baseline state)
    let record = ContentHashRecord {
        crate_name: "serde".to_string(),
        version: "1.0.197".to_string(),
        content_hash: "recorded-hash-abc123".to_string(),
        hash_source: "cargo-lock-checksum".to_string(),
        signed_by: "alice".to_string(),
        date: "2026-05-22".to_string(),
    };
    save_content_hash_record(tmp.path(), &record).unwrap();

    // Write a Cargo.lock with a MISMATCHING hash
    std::fs::write(
        tmp.path().join("Cargo.lock"),
        r#"
[[package]]
name = "serde"
version = "1.0.197"
checksum = "swapped-malicious-hash"
"#,
    )
    .unwrap();

    // Verify baseline: Mismatch is correctly detected
    let baseline = evaluate_content_hash_matches(tmp.path(), "serde", "1.0.197");
    assert_eq!(
        baseline,
        ContentHashState::Mismatch {
            recorded: "recorded-hash-abc123".to_string(),
            current: "swapped-malicious-hash".to_string(),
        },
        "Baseline: mismatch must be detected before corruption attack"
    );

    // Now CORRUPT the sidecar (attacker write access)
    let sidecar_path =
        antigen::supply_chain::evaluate::content_hash_path(tmp.path(), "serde", "1.0.197");
    std::fs::write(&sidecar_path, "{ this is not valid json }").unwrap();

    // ATTACK: corrupt sidecar should NOT silently become NoAttestation
    let after_corruption = evaluate_content_hash_matches(tmp.path(), "serde", "1.0.197");

    // FAILING RIGHT NOW: current impl returns NoAttestation (silent downgrade)
    // EXPECTED: Some error state (like ContentHashState::SidecarMalformed)
    //
    // The bug: load_content_hash_record uses .ok() to discard malformed JSON.
    // This converts a MISMATCH (high-severity alert) into NoAttestation (warning)
    // when an attacker corrupts the sidecar file.
    assert_ne!(
        after_corruption,
        ContentHashState::NoAttestation,
        "ATK-SC-2-A: Corrupted content-hash sidecar must NOT silently become \
         NoAttestation. A Mismatch (error-level) was established before corruption; \
         corrupting the sidecar to invalid JSON silently downgrades it to \
         NoAttestation (warning). \
         \
         Fix: load_content_hash_record should return a Result<Option<ContentHashRecord>> \
         and evaluate_content_hash_matches should map malformed JSON to a \
         SidecarMalformed state (like evaluate_dep_attested already does). \
         \
         Contrast: evaluate_dep_attested correctly surfaces SidecarMalformed. \
         The inconsistency is the bug."
    );
}

// ============================================================================
// ATK-SC-2-B: NoAttestation when Cargo.lock has entry but NO checksum
//
// Some packages in Cargo.lock don't have checksums (path deps, workspace members,
// git deps). The current implementation returns CrateNotInLockfile for these,
// but this is a misleading error — the crate IS in the lockfile, it just has
// no checksum.
//
// This documents current behavior to verify pathmaker's intent.
// ============================================================================

#[test]
fn atk_sc2b_package_in_lockfile_without_checksum_behavior() {
    let tmp = TempDir::new().unwrap();

    // Write a sidecar
    let record = ContentHashRecord {
        crate_name: "my-local-crate".to_string(),
        version: "0.1.0".to_string(),
        content_hash: "abc".to_string(),
        hash_source: "cargo-lock-checksum".to_string(),
        signed_by: "alice".to_string(),
        date: "2026-05-22".to_string(),
    };
    save_content_hash_record(tmp.path(), &record).unwrap();

    // Write a Cargo.lock where the package has NO checksum (path dep)
    std::fs::write(
        tmp.path().join("Cargo.lock"),
        r#"
[[package]]
name = "my-local-crate"
version = "0.1.0"
# No checksum — path dependency
"#,
    )
    .unwrap();

    let state = evaluate_content_hash_matches(tmp.path(), "my-local-crate", "0.1.0");

    // Document current behavior: path deps without checksums
    eprintln!(
        "ATK-SC-2-B: my-local-crate (no checksum in lockfile) evaluates to: {:?}. \
         Expected: CrateNotInLockfile (current) or a new 'NoChecksum' state. \
         Path deps don't have checksums — 'CrateNotInLockfile' is misleading.",
        state
    );

    // Current behavior is CrateNotInLockfile — document this.
    assert_eq!(
        state,
        ContentHashState::CrateNotInLockfile {
            crate_name: "my-local-crate".to_string(),
        },
        "ATK-SC-2-B: Path dep (no checksum) returns CrateNotInLockfile. \
         This is current behavior — the name is misleading (the crate IS in \
         the lockfile, it just has no checksum). NAMED-LIMITATION: content-hash \
         verification only applies to registry deps with checksums."
    );
}

// ============================================================================
// ATK-SC-4-A: Dev-dependencies included in UnpinnedDependency check
//
// evaluate_dep_pinned reads ALL dep entries including dev-dependencies.
// A dev-dep with non-exact version would fire UnpinnedDependency.
// Dev-deps are typically not in the supply-chain attack surface
// (they're only used at compile time for the workspace itself, not shipped).
//
// This documents whether dev-deps produce false positives.
// ============================================================================

#[test]
fn atk_sc4a_dev_dep_unpinned_fires_unpinned_dependency_check() {
    use antigen::supply_chain::evaluate::evaluate_dep_pinned_against;
    use antigen::supply_chain::manifest::parse_manifest_deps;
    use antigen::supply_chain::witness::DepPinnedState;

    let content = r#"
[dependencies]
serde = "=1.0.197"

[dev-dependencies]
proptest = "1.0"
"#;
    let entries = parse_manifest_deps(content);
    let state = evaluate_dep_pinned_against(&entries, None);

    // Documents current behavior: dev-deps are included.
    eprintln!(
        "ATK-SC-4-A: Manifest with exact-pinned dep + unpinned dev-dep evaluates to: {:?}",
        state
    );

    // FINDING: proptest = "1.0" is a dev-dep but UnpinnedDependency fires.
    // This may produce false positives for projects with many unpinned dev-deps.
    // If dev-deps should be excluded from the check, filter by section.
    match &state {
        DepPinnedState::Unpinned { unpinned_deps } => {
            assert!(
                unpinned_deps.contains(&"proptest".to_string()),
                "ATK-SC-4-A: proptest (unpinned dev-dep) must appear in unpinned list: {:?}",
                unpinned_deps
            );
            eprintln!(
                "ATK-SC-4-A CONFIRMED: dev-dep 'proptest' fires UnpinnedDependency. \
                 This is a potential false positive for projects with unpinned dev-deps. \
                 ADR-025 doesn't distinguish dep types — document as NAMED-LIMITATION."
            );
        }
        DepPinnedState::AllPinned => {
            panic!(
                "ATK-SC-4-A: evaluate_dep_pinned_against returned AllPinned even with \
                 unpinned dev-dep 'proptest'. Either dev-deps are excluded (intentional?) \
                 or this is a bug. Document the section-filtering decision."
            );
        }
        DepPinnedState::NotInManifest { .. } => {
            panic!("ATK-SC-4-A: unexpected NotInManifest state");
        }
    }
}

// ============================================================================
// ATK-SC-AUDIT-1: any_of semantics broken in audit_supply_chain
//
// ATTACK: collect_leaves() flattens ALL leaves from both any_of AND all_of
// combinators identically. For any_of, the predicate passes if ANY child passes.
// But the audit emits failure hints for ALL failed children, even in any_of.
//
// Scenario:
//   any_of([
//     content_hash_matches("serde", "1.0.197"),  -- has matching sidecar (PASS)
//     content_hash_matches("serde", "1.0.196"),  -- no sidecar (FAIL)
//   ])
//
// EXPECTED: any_of is satisfied → all_pass() = true (zero failure hints)
// CURRENT: both leaves evaluated → 1.0.196 emits ContentHashNoAttestation →
//          fail_count = 1 → all_pass() = false (FALSE POSITIVE)
//
// This is a logical error in collect_leaves: it doesn't distinguish
// AllOf from AnyOf semantics. For any_of, a single passing child
// should suppress failure hints from sibling children.
// ============================================================================

#[test]
fn atk_sc_audit1_any_of_emits_false_positive_for_passing_branch() {
    use antigen::audit::{audit_supply_chain, AuditHint};
    use antigen::scan::{Immunity, ItemTarget, ScanReport};
    use antigen_attestation::{Leaf, Predicate};

    let tmp = TempDir::new().unwrap();

    // Set up sidecar for serde@1.0.197 WITH matching hash
    let record = ContentHashRecord {
        crate_name: "serde".to_string(),
        version: "1.0.197".to_string(),
        content_hash: "goodhash".to_string(),
        hash_source: "cargo-lock-checksum".to_string(),
        signed_by: "alice".to_string(),
        date: "2026-05-22".to_string(),
    };
    save_content_hash_record(tmp.path(), &record).unwrap();

    // Cargo.lock with matching hash for 1.0.197
    std::fs::write(
        tmp.path().join("Cargo.lock"),
        "[[package]]\nname = \"serde\"\nversion = \"1.0.197\"\nchecksum = \"goodhash\"\n",
    )
    .unwrap();
    // NO sidecar for serde@1.0.196 — that branch will fail

    // Build any_of([content_hash_matches("serde", "1.0.197"),
    //               content_hash_matches("serde", "1.0.196")])
    let pred = Predicate::any_of(vec![
        Predicate::leaf(Leaf::ContentHashMatches {
            crate_name: "serde".to_string(),
            version: "1.0.197".to_string(),
        }),
        Predicate::leaf(Leaf::ContentHashMatches {
            crate_name: "serde".to_string(),
            version: "1.0.196".to_string(),
        }),
    ])
    .unwrap();

    let pred_json = serde_json::to_string(&pred).unwrap();

    // Create synthetic scan report with the any_of immunity
    let immunity = Immunity {
        antigen_type: "ContentHashMismatch".to_string(),
        witness: String::new(),
        file: std::path::PathBuf::from("src/lib.rs"),
        line: 10,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Fn("check_serde".to_string()),
        canonical_path: None,
        requires_predicate: Some(pred_json),
    };
    let mut scan = ScanReport::default();
    scan.immunities.push(immunity);

    let report = audit_supply_chain(&scan, tmp.path());

    let failure_hints: Vec<_> = report
        .audits
        .iter()
        .filter(|a| a.hint != AuditHint::FunctionResolves)
        .collect();

    // ADVERSARIAL PATTERN: this test FAILS when the bug EXISTS.
    // When pathmaker fixes any_of semantics, the test passes.
    //
    // Correct behavior: any_of is satisfied by the 1.0.197 branch (PASS).
    // No failure hints should be emitted for sibling branches that failed.
    //
    // Bug: collect_leaves() treats AllOf and AnyOf identically, evaluating
    // ALL children and emitting failure hints for ANY failed leaf,
    // even when another sibling leaf passed and the any_of is satisfied.
    assert!(
        report.all_pass(),
        "ATK-SC-AUDIT-1: any_of([match_197(PASS), no_sidecar_196(FAIL)]) MUST pass \
         because the 1.0.197 branch satisfies the any_of predicate. \
         \n\nBUG: collect_leaves() treats AllOf and AnyOf identically. For any_of, \
         if any child passes, sibling failures MUST NOT produce audit hints. \
         \n\ncurrent fail_count={}, failure_hints={:?} \
         \n\nFix: propagate predicate-structure context (allOf vs anyOf) through \
         collect_leaves. For anyOf nodes, suppress sibling failure hints when \
         any child passes.",
        report.fail_count,
        failure_hints.iter().map(|a| &a.hint).collect::<Vec<_>>()
    );
}

// ============================================================================
// ATK-SC-4-B: Workspace inheritance vacuously exact-pinned
//
// A dep with workspace = true has version = None, which is_exact_pinned()
// treats as vacuously true. But the workspace's own Cargo.toml may have
// non-exact-pinned deps that go unchecked.
//
// This documents the workspace inheritance gap.
// ============================================================================

#[test]
fn atk_sc4b_workspace_dep_vacuously_exact_pinned() {
    use antigen::supply_chain::evaluate::evaluate_dep_pinned_against;
    use antigen::supply_chain::manifest::parse_manifest_deps;
    use antigen::supply_chain::witness::DepPinnedState;

    let content = r"
[dependencies]
serde = { workspace = true }
";
    let entries = parse_manifest_deps(content);
    let state = evaluate_dep_pinned_against(&entries, None);

    assert_eq!(
        state,
        DepPinnedState::AllPinned,
        "ATK-SC-4-B: workspace dep treated as vacuously exact-pinned. \
         This is a NAMED-LIMITATION (in manifest.rs comments): \
         'workspace = true' means version is not inline; not checked. \
         An adopter with workspace.dependencies that have non-exact pins \
         won't get flagged at the member-crate level. \
         The workspace Cargo.toml must be checked separately."
    );
}
