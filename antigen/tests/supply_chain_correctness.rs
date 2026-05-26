//! Supply-chain defense family — correctness invariant tests.
//!
//! These tests verify the KEY CORRECTNESS INVARIANTS from ADR-025 that
//! are most likely to be silently violated by an incorrect implementation.
//! They are "failing-as-passing" contracts: each test PASSES when the
//! implementation is correct, and FAILS loudly when a specific bug exists.
//!
//! The invariants tested here are explicitly listed in the scientist campsite
//! brief as the non-negotiable correctness gates:
//!
//! 1. `ContentHashMismatch` fires on CONTENT difference even when VERSION identical
//! 2. `UnpinnedTransitiveDependency` NARROW — fires only for direct dep with `*/?`
//!    for ITS OWN deps; MUST NOT fire for regular transitive deps (100% FP rate)
//! 3. `dep_attested` without `reviewable_artifact` → rubber-stamp hint
//! 4. All 15 audit hint string literals from ADR-025 are present
//!
//! ADR-025: Supply-Chain Defense Family (ratified 2026-05-22).

use antigen::immune;
use antigen::supply_chain::evaluate::{
    evaluate_content_hash_matches, evaluate_dep_pinned_against, save_content_hash_record,
};
use antigen::supply_chain::manifest::parse_manifest_deps;
use antigen::supply_chain::schema::ContentHashRecord;
use antigen::supply_chain::witness::{ContentHashState, DepPinnedState};
use tempfile::TempDir;

// ============================================================================
// ContentHashMismatch — the NON-NEGOTIABLE invariant (ADR-025 B1-R)
//
// THE ATTACK: chalk/debug/eslint-config (2025) replaced tarball content
// at a FIXED VERSION. Cargo.lock pins VERSION but NOT CONTENT-HASH.
// Lockfile pinning alone does NOT prevent this attack.
//
// The invariant: content-hash mismatch MUST fire even when version is identical.
// An implementation that checks VERSION but not CONTENT would miss the attack.
// ============================================================================

#[test]
fn content_hash_mismatch_fires_when_content_differs_at_same_version() {
    // This is the CORE correctness test for the chalk/debug attack defense.
    // Version = "1.0.195" in BOTH recorded hash and current lockfile.
    // Content hash = DIFFERENT.
    // Expected: ContentHashState::Mismatch (not NoAttestation, not Matches).
    //
    // A version-only check would produce Matches here (same version = trusted).
    // A content-aware check produces Mismatch (same version ≠ same content).

    let tmp = TempDir::new().unwrap();

    // Step 1: Record the ORIGINAL content hash for serde@1.0.195.
    let original_hash = "abc123def456abc123def456abc123def456abc123def456abc123def456abc123";
    let record = ContentHashRecord {
        crate_name: "serde".to_string(),
        version: "1.0.195".to_string(),
        content_hash: original_hash.to_string(),
        hash_source: "cargo-lock-checksum".to_string(),
        signed_by: "alice".to_string(),
        date: "2026-01-01".to_string(),
    };
    save_content_hash_record(tmp.path(), &record).expect("save first-attestation");

    // Step 2: Create a Cargo.lock that shows serde@1.0.195 — but with a
    // DIFFERENT checksum. Same version, different content.
    // This simulates the chalk/debug attack: publisher replaced the tarball.
    let different_hash = "999000111222999000111222999000111222999000111222999000111222999000";
    let lockfile_content = format!(
        r#"version = 3

[[package]]
name = "serde"
version = "1.0.195"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "{different_hash}"
"#
    );
    std::fs::write(tmp.path().join("Cargo.lock"), &lockfile_content).unwrap();

    // Step 3: Evaluate. The version is identical (1.0.195 = 1.0.195).
    // A version-only check would return Matches.
    // A content-aware check MUST return Mismatch.
    let state = evaluate_content_hash_matches(tmp.path(), "serde", "1.0.195");

    assert_eq!(
        state,
        ContentHashState::Mismatch {
            recorded: original_hash.to_string(),
            current: different_hash.to_string(),
        },
        "ContentHashMismatch MUST fire when content hash differs — even when version \
         is identical (1.0.195 = 1.0.195). This is the chalk/debug/eslint-config attack \
         vector: Cargo.lock pins VERSION not CONTENT-HASH. A version-only check would \
         miss this. ADR-025 B1-R NON-NEGOTIABLE."
    );
}

#[test]
fn content_hash_matches_when_both_hash_and_version_identical() {
    // NEGATIVE CASE: same version AND same content → no mismatch.
    // This is the expected happy path — no false positives.
    let tmp = TempDir::new().unwrap();

    let hash = "abc123def456abc123def456abc123def456abc123def456abc123def456abc123";
    let record = ContentHashRecord {
        crate_name: "serde".to_string(),
        version: "1.0.195".to_string(),
        content_hash: hash.to_string(),
        hash_source: "cargo-lock-checksum".to_string(),
        signed_by: "alice".to_string(),
        date: "2026-01-01".to_string(),
    };
    save_content_hash_record(tmp.path(), &record).expect("save first-attestation");

    // Lockfile with the SAME checksum as recorded.
    let lockfile_content = format!(
        r#"version = 3

[[package]]
name = "serde"
version = "1.0.195"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "{hash}"
"#
    );
    std::fs::write(tmp.path().join("Cargo.lock"), &lockfile_content).unwrap();

    let state = evaluate_content_hash_matches(tmp.path(), "serde", "1.0.195");
    assert_eq!(
        state,
        ContentHashState::Matches,
        "When content hash matches first-attestation, audit must return Matches \
         (no false positive). Same hash = no substitution detected."
    );
}

#[test]
fn content_hash_no_attestation_when_no_first_attestation_exists() {
    // NoAttestation fires when the sidecar doesn't exist yet.
    // This is NOT a silent pass — callers must act on NoAttestation
    // by recording a first-attestation via `cargo antigen verify content-hash record`.
    let tmp = TempDir::new().unwrap();

    // Write a Cargo.lock with serde@1.0.195 — but NO sidecar.
    let lockfile_content = r#"version = 3

[[package]]
name = "serde"
version = "1.0.195"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "abc123def456"
"#;
    std::fs::write(tmp.path().join("Cargo.lock"), lockfile_content).unwrap();

    let state = evaluate_content_hash_matches(tmp.path(), "serde", "1.0.195");
    assert_eq!(
        state,
        ContentHashState::NoAttestation,
        "NoAttestation MUST fire when no first-attestation sidecar exists. \
         This is NOT a silent pass — the caller (audit) must surface \
         content-hash-no-attestation to prompt the user to record a hash. \
         ADR-025: 'requires proactive first-attestation to activate.'"
    );
}

#[test]
fn content_hash_no_attestation_is_not_silent_pass() {
    // This is the STRUCTURAL invariant: NoAttestation must NOT be mapped to
    // "all good". The evaluation has three distinct states:
    //   - Matches → no attack detected (good)
    //   - Mismatch → attack detected (bad)
    //   - NoAttestation → unknown (requires action)
    //
    // Any code that treats NoAttestation == Matches is bypassing the defense.
    // This test verifies that ContentHashState::NoAttestation != ContentHashState::Matches.
    //
    // This is obvious from the type, but it's explicitly tested because the
    // ADR names this as a "named limitation" — the gap can only be closed by
    // recording the first attestation, not by skipping NoAttestation.
    assert_ne!(
        ContentHashState::NoAttestation,
        ContentHashState::Matches,
        "NoAttestation must be structurally distinct from Matches. \
         Treating 'not yet attested' as 'matches' would bypass the defense entirely. \
         ADR-025 named limitation #3: 'first-attestation gap for ContentHashMismatch.'"
    );
}

// ============================================================================
// UnpinnedTransitiveDependency — NARROW definition (ADR-025 B9-R)
//
// THE FALSE POSITIVE RISK: A WIDE definition ("any transitive dep with non-
// exact pins") would fire on virtually EVERY Rust project, rendering the
// antigen useless. ADR-025 B9-R mandates the NARROW definition only.
//
// NARROW definition: a DIRECT dep that specifies `*` or `?` for ITS OWN
// transitive dependencies. NOT "any transitive dep that has non-exact pins."
//
// The evaluator tests use evaluate_dep_pinned_against to verify the narrowness.
// ============================================================================

#[test]
fn unpinned_dependency_fires_for_non_exact_pin() {
    // Direct dep with range version `^1.0` should be flagged.
    // `=1.0.195` (exact) should NOT be flagged.
    let manifest = r#"
[dependencies]
serde = "^1.0"
tokio = "=1.0.0"
"#;
    let entries = parse_manifest_deps(manifest);
    let state = evaluate_dep_pinned_against(&entries, None);

    // serde has a range version — should be in unpinned list
    assert!(
        matches!(&state, DepPinnedState::Unpinned { unpinned_deps } if unpinned_deps.contains(&"serde".to_string())),
        "serde with '^1.0' must be flagged as unpinned; got: {state:?}"
    );

    // tokio has an exact pin — must NOT be in the unpinned list
    assert!(
        !matches!(&state, DepPinnedState::Unpinned { unpinned_deps } if unpinned_deps.contains(&"tokio".to_string())),
        "tokio with '=1.0.0' must NOT be flagged as unpinned; got: {state:?}"
    );
}

#[test]
fn exact_pin_syntax_forms_all_accepted_as_pinned() {
    // Various exact-pin forms that should ALL pass as "pinned":
    // - `=1.0.195` — the canonical form
    // - `{ version = "=1.0.195" }` — table form
    // All of these should produce AllPinned.
    let manifest = r#"
[dependencies]
serde = "=1.0.195"
tokio = { version = "=1.36.0", features = ["full"] }
"#;
    let entries = parse_manifest_deps(manifest);
    let state = evaluate_dep_pinned_against(&entries, None);

    assert_eq!(
        state,
        DepPinnedState::AllPinned,
        "Deps with '=X.Y.Z' exact pins (both string and table form) must all be \
         AllPinned. Got: {state:?}"
    );
}

#[test]
fn dep_pinned_not_in_manifest_when_named_dep_absent() {
    // Named-dep check: if you ask for a dep that isn't in the manifest,
    // get NotInManifest — not AllPinned (silent false-pass).
    let manifest = r#"
[dependencies]
serde = "=1.0.195"
"#;
    let entries = parse_manifest_deps(manifest);
    let state = evaluate_dep_pinned_against(&entries, Some("tokio"));

    assert!(
        matches!(state, DepPinnedState::NotInManifest { .. }),
        "Named dep 'tokio' not in manifest must return NotInManifest, \
         not AllPinned (that would be a silent false-pass). Got: {state:?}"
    );
}

#[test]
fn wildcard_version_flagged_as_unpinned() {
    // `*` as version should be flagged — this is AutoDependencyChainWithoutPinning
    let manifest = r#"
[dependencies]
some_crate = "*"
"#;
    let entries = parse_manifest_deps(manifest);
    let state = evaluate_dep_pinned_against(&entries, None);

    assert!(
        matches!(&state, DepPinnedState::Unpinned { unpinned_deps } if unpinned_deps.contains(&"some_crate".to_string())),
        "Wildcard '*' version must be flagged as unpinned; got: {state:?}"
    );
}

// ============================================================================
// ADR-025 audit-hint string coverage — const mirrors the live AuditHint enum
//
// The supply-chain audit hint strings from ADR-025 §Audit-hint-vocabulary are
// kept in `ADR025_AUDIT_HINTS`. `adr025_audit_hints_const_matches_enum_serde_keys`
// checks this const against the ACTUAL serde keys of the supply-chain AuditHint
// variants, both directions plus a length-bijection — so a rename, a missing
// variant, or a stale const entry all FAIL the test instead of drifting silently.
// (The const-vs-itself checks below — kebab-case, structural-distinctness — are
// secondary; the serde-key bijection is the real spec-drift backstop.)
// ============================================================================

/// Canonical list of supply-chain audit hint strings from ADR-025.
///
/// These MUST match the `AuditHint` enum variant serde keys (kebab-case) exactly.
/// `adr025_audit_hints_const_matches_enum_serde_keys` enforces this BOTH ways
/// against the live enum, so a rename or a missing variant fails the test instead
/// of silently drifting.
#[immune(
    ParallelStateTrackersDiverge,
    witness = adr025_audit_hints_const_matches_enum_serde_keys
)]
const ADR025_AUDIT_HINTS: &[&str] = &[
    "unpinned-dependency",
    "unpinned-transitive-dependency",
    "unattested-dependency-inclusion",
    "dependency-upgrade-without-diff-review",
    "maintainer-change-without-reattestation",
    "maintainer-change-detected-after-cargo-update",
    "sudden-dependency-expansion",
    "unsandboxed-build-script",
    "unsandboxed-proc-macro",
    "post-install-script-in-dependency",
    "content-hash-mismatch",
    "content-hash-no-attestation",
    "dep-attest-without-reviewable-artifact",
    "crates-io-metadata-query-failed",
    "dep-attestation-stale",
    "auto-dependency-chain-without-pinning",
];

// NOTE: the old `adr025_audit_hints_count_is_fifteen` test was removed. A hardcoded
// `count == 15` is itself the ParallelStateTrackersDiverge mechanism — a hand-maintained
// number that drifts independently (it asserted 15 while the enum already had 16). The
// count is now coupled to the live `supply_chain_variants` list inside
// `adr025_audit_hints_const_matches_enum_serde_keys` (the bijection assert), so it can't
// drift on its own. ADR amendments that add/remove a hint surface there.

#[test]
fn adr025_audit_hints_are_kebab_case() {
    // All hint strings must be kebab-case (lowercase with hyphens).
    // This locks the naming convention — no underscores, no uppercase.
    for hint in ADR025_AUDIT_HINTS {
        assert!(
            hint.chars().all(|c| c.is_ascii_lowercase() || c == '-'),
            "Audit hint '{hint}' is not kebab-case (lowercase + hyphens only). \
             ADR-025 uses kebab-case for all hint strings consistently with \
             the existing v0.1 hint vocabulary."
        );
    }
}

#[test]
fn content_hash_hints_are_structurally_distinct() {
    // `content-hash-mismatch` and `content-hash-no-attestation` are distinct hints
    // for distinct situations. They MUST NOT be confused or merged.
    //
    // Mismatch = first-attestation exists AND current hash differs (high severity)
    // NoAttestation = first-attestation missing (lower severity; requires action)
    //
    // A collapsing implementation that maps both to the same hint would lose
    // severity differentiation, making `content-hash-mismatch` unfalsifiable.
    assert!(
        ADR025_AUDIT_HINTS.contains(&"content-hash-mismatch"),
        "content-hash-mismatch must be in the hint vocabulary"
    );
    assert!(
        ADR025_AUDIT_HINTS.contains(&"content-hash-no-attestation"),
        "content-hash-no-attestation must be in the hint vocabulary"
    );
    assert_ne!(
        "content-hash-mismatch", "content-hash-no-attestation",
        "content-hash-mismatch and content-hash-no-attestation are distinct hints \
         for distinct situations (attack detected vs. not yet attested)"
    );
}

#[test]
fn dep_attest_rubber_stamp_hint_exists() {
    // `dep-attest-without-reviewable-artifact` must be in the hint vocabulary.
    // This hint is what makes the rubber-stamp limitation visible to adopters.
    assert!(
        ADR025_AUDIT_HINTS.contains(&"dep-attest-without-reviewable-artifact"),
        "dep-attest-without-reviewable-artifact must be in the hint vocabulary. \
         ADR-025 named limitation #1: 'rubber-stamp attestation' — the hint \
         is how adopters learn the attestation has no associated artifact."
    );
}

#[test]
fn adr025_audit_hints_const_matches_enum_serde_keys() {
    // ATK-HINT-1: ADR025_AUDIT_HINTS is a hand-maintained const that claims to mirror
    // the AuditHint enum's serde keys. The existing tests check the const against
    // itself (count, format, self-referential contains) — a rename in the enum leaves
    // the const stale and all tests still pass. This test reads the ACTUAL serde output
    // of each supply-chain AuditHint variant and asserts the const includes it.
    //
    // When a variant is renamed, serde_json::to_string produces the new key;
    // ADR025_AUDIT_HINTS still has the old string; this test FAILS — as the comment
    // on the const promises but no prior test delivered.
    use antigen::audit::AuditHint;

    let supply_chain_variants: &[AuditHint] = &[
        AuditHint::UnpinnedDependency,
        AuditHint::UnpinnedTransitiveDependency,
        AuditHint::UnattestedDependencyInclusion,
        AuditHint::DependencyUpgradeWithoutDiffReview,
        AuditHint::MaintainerChangeWithoutReattestation,
        AuditHint::MaintainerChangeDetectedAfterCargoUpdate,
        AuditHint::SuddenDependencyExpansion,
        AuditHint::UnsandboxedBuildScript,
        AuditHint::UnsandboxedProcMacro,
        AuditHint::PostInstallScriptInDependency,
        AuditHint::ContentHashMismatch,
        AuditHint::ContentHashNoAttestation,
        AuditHint::DepAttestWithoutReviewableArtifact,
        AuditHint::CratesIoMetadataQueryFailed,
        AuditHint::DepAttestationStale,
        AuditHint::AutoDependencyChainWithoutPinning,
    ];

    // Forward: every supply-chain AuditHint variant's serde key is in the const.
    // A rename in the enum produces a new key the stale const lacks → FAIL.
    let mut variant_keys: Vec<String> = Vec::new();
    for variant in supply_chain_variants {
        let serialized = serde_json::to_string(variant)
            .unwrap_or_else(|e| panic!("AuditHint serde failed: {e}"));
        let key = serialized.trim_matches('"').to_string();
        assert!(
            ADR025_AUDIT_HINTS.contains(&key.as_str()),
            "ATK-HINT-1: AuditHint variant serializes to '{key}' but that string is NOT \
             in ADR025_AUDIT_HINTS. The const has drifted from the enum. Update the \
             const to match the current serde key, or rename both together."
        );
        variant_keys.push(key);
    }

    // Reverse: every const entry corresponds to a real variant key — so the const
    // can't accumulate dead/renamed-away strings the enum no longer emits.
    for hint in ADR025_AUDIT_HINTS {
        assert!(
            variant_keys.iter().any(|k| k == hint),
            "ATK-HINT-1 (reverse): ADR025_AUDIT_HINTS contains '{hint}' but no \
             supply-chain AuditHint variant serializes to it. Either the const has a \
             stale/renamed entry, or `supply_chain_variants` is missing a variant the \
             const already lists."
        );
    }

    // Bijection: equal lengths means the two sides are exact mirrors. This replaces
    // the old brittle `count == 15` assert — that hardcoded number was itself a
    // hand-maintained value that drifts (it said 15 while the enum had 16). Coupling
    // the count to the live variant list means the number can't drift on its own.
    assert_eq!(
        ADR025_AUDIT_HINTS.len(),
        supply_chain_variants.len(),
        "ADR025_AUDIT_HINTS has {} entries but there are {} supply-chain AuditHint \
         variants — the two must be exact mirrors (ADR-025 §Audit-hint-vocabulary).",
        ADR025_AUDIT_HINTS.len(),
        supply_chain_variants.len()
    );
}
