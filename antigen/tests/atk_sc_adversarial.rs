//! ATK-SC adversarial test suite — Supply-Chain Defense Family (ADR-025).
//!
//! STATUS: PENDING PATHMAKER COMMIT
//
// These tests will not compile until pathmaker ships the supply-chain family.
// They are written against the SPEC (ADR-025) — they define what SHOULD be true.
// When pathmaker ships, these either:
//   (a) compile and PASS  → implementation is correct
//   (b) compile and FAIL  → implementation has the bug this test catches
//   (c) don't compile     → required types or functions are missing
//
// All three outcomes are actionable.
//
// Adversarial methodology:
//   - Each test is named after the attack vector
//   - Assertions describe what SHOULD be true (spec-derived, not code-derived)
//   - Silent pass is the enemy; tests are chosen to catch plausible-but-wrong answers

// ============================================================================
// ATK-SC-1: Rubber-stamp attestation bypass
// ============================================================================
//
// dep_attested with empty or missing reviewable_artifact MUST emit
// dep-attest-without-reviewable-artifact. Tests both attack variants.
//
// Bypass: supply "" as reviewable_artifact or omit it entirely.
// Expected defense: audit checks artifact field before accepting claim.

// NOTE: These tests require `antigen::audit::audit_supply_chain` and the
// `DepAttested` predicate leaf type. Enable when pathmaker ships.

// #[test]
// fn atk_sc1_empty_reviewable_artifact_produces_dep_attest_hint() {
//     use antigen::audit::{AuditHint, audit_supply_chain};
//     use antigen::scan::scan_workspace;
//     use std::path::{Path, PathBuf};
//
//     let fx = Path::new(env!("CARGO_MANIFEST_DIR"))
//         .join("tests/fixtures/atk_sc_rubber_stamp_attestation");
//     let scan = scan_workspace(&fx, None).expect("scan completes");
//     let report = audit_supply_chain(&scan, &fx);
//
//     // atk_sc1_empty_artifact uses dep_attested("serde", "1.0.200", reviewable_artifact = "")
//     let empty_artifact_audit = report.audits.iter().find(|a| {
//         a.function_name.as_deref() == Some("atk_sc1_empty_artifact")
//     }).expect("audit for atk_sc1_empty_artifact must exist");
//
//     assert_eq!(
//         empty_artifact_audit.hint,
//         AuditHint::DepAttestWithoutReviewableArtifact,
//         "ATK-SC-1-A: empty reviewable_artifact must emit dep-attest-without-reviewable-artifact; \
//          got: {:?}", empty_artifact_audit.hint
//     );
//
//     // atk_sc1_missing_artifact has no reviewable_artifact at all
//     let missing_artifact_audit = report.audits.iter().find(|a| {
//         a.function_name.as_deref() == Some("atk_sc1_missing_artifact")
//     }).expect("audit for atk_sc1_missing_artifact must exist");
//
//     assert_eq!(
//         missing_artifact_audit.hint,
//         AuditHint::DepAttestWithoutReviewableArtifact,
//         "ATK-SC-1-B: missing reviewable_artifact must emit dep-attest-without-reviewable-artifact; \
//          got: {:?}", missing_artifact_audit.hint
//     );
// }

// ============================================================================
// ATK-SC-2: ContentHashMismatch timing gap — no first-attestation
// ============================================================================
//
// content_hash_matches witness with NO .attest sidecar must emit
// content-hash-no-attestation, NOT silently pass.
//
// The silent-pass attack: attacker replaces content BEFORE first attestation.
// Defense: absence of attestation = warning, not acceptance.

// #[test]
// fn atk_sc2_no_first_attestation_emits_content_hash_no_attestation() {
//     use antigen::audit::{AuditHint, audit_supply_chain};
//     use antigen::scan::scan_workspace;
//     use std::path::Path;
//
//     let fx = Path::new(env!("CARGO_MANIFEST_DIR"))
//         .join("tests/fixtures/atk_sc_content_hash_no_attestation");
//     let scan = scan_workspace(&fx, None).expect("scan completes");
//     let report = audit_supply_chain(&scan, &fx);
//
//     let content_hash_audit = report.audits.iter().find(|a| {
//         a.function_name.as_deref() == Some("atk_sc2_no_first_attestation")
//     }).expect("audit for atk_sc2_no_first_attestation must exist");
//
//     assert_eq!(
//         content_hash_audit.hint,
//         AuditHint::ContentHashNoAttestation,
//         "ATK-SC-2: content_hash_matches with no .attest sidecar must emit \
//          content-hash-no-attestation, not pass silently. \
//          Silent pass = bypass of chalk/debug attack defense. Got: {:?}",
//         content_hash_audit.hint
//     );
// }

// ============================================================================
// ATK-SC-3: MaintainerChangeWithoutReattestation sequencing — ordering constraint
// ============================================================================
//
// The CI sequencing constraint: verify BEFORE cargo update.
// If cargo update runs first, the maintainer transition is already in Cargo.lock
// and the "change" can no longer be detected as a change.
//
// This is tested via CLI output inspection — the `verify maintainer-changes`
// subcommand must document its BEFORE-cargo-update constraint in help text.

// NOTE: This test requires `cargo-antigen` binary testing infrastructure.
// Documented as NAMED-LIMITATION if detection is trust-based.

// ============================================================================
// ATK-SC-4: UnpinnedTransitiveDependency false positive storm
// ============================================================================
//
// NARROW definition must be enforced. Two cases:
//   FALSE POSITIVE: workspace with only-transitive non-exact-pinned deps
//   TRUE POSITIVE: workspace where a direct dep has `*` for its OWN deps

// NOTE: These require audit_supply_chain to exist and the
// `UnpinnedTransitiveDependency` struct to be addressable.

// #[test]
// fn atk_sc4_false_positive_only_transitive_should_not_fire() {
//     // If this fixture's audit fires unpinned-transitive-dependency,
//     // it has the WRONG (wide) definition.
//     // Fixture: serde = "=1.0.200" (exact-pinned direct dep).
//     // serde's OWN transitive deps may be loose — NOT our problem.
//     use antigen::audit::{AuditHint, audit_supply_chain};
//     use antigen::scan::scan_workspace;
//     use std::path::Path;
//
//     let fx = Path::new(env!("CARGO_MANIFEST_DIR"))
//         .join("tests/fixtures/atk_sc_unpinned_transitive_false_positive");
//     let scan = scan_workspace(&fx, None).expect("scan completes");
//     let report = audit_supply_chain(&scan, &fx);
//
//     let transitive_hints: Vec<_> = report.audits.iter()
//         .filter(|a| a.hint == AuditHint::UnpinnedTransitiveDependency)
//         .collect();
//
//     assert!(
//         transitive_hints.is_empty(),
//         "ATK-SC-4 FALSE POSITIVE: audit fires UnpinnedTransitiveDependency \
//          on workspace with only exact-pinned direct deps. This is the wide \
//          definition which has ~100% false positive rate. Hits: {:?}",
//         transitive_hints
//     );
// }

// ============================================================================
// ATK-SC-5: Solo developer single-signer limitation
// ============================================================================
//
// The limitation: dep_attested single-signer (solo dev reviewing their own dep)
// provides no independent review. This is a NAMED LIMITATION.
// Check: is the limitation documented in user-visible output?

// This is tested via CLI help text inspection and dep_attested sidecar schema.
// If the limitation is NOT named in audit output, it degrades silently.

// ============================================================================
// ATK-SC-6: Sandbox-detection limitation — time-bomb attack
// ============================================================================
//
// UnsandboxedProcMacro/BuildScript can't detect environment-aware malicious code.
// If code checks IS_CI or a timestamp, the sandbox test passes but prod fails.
// This is a NAMED LIMITATION — check that the antigen summary names it.

// #[test]
// fn atk_sc6_unsandboxed_proc_macro_summary_names_time_bomb_limitation() {
//     use antigen::scan::scan_workspace;
//     use std::path::Path;
//
//     // The UnsandboxedProcMacro antigen is a stdlib declaration.
//     // Its `summary` field must name "environment-aware" or "time-bomb" limitation.
//     // Scan the stdlib module to verify the summary is present and honest.
//     //
//     // NOTE: this requires the stdlib to be scannable. If the stdlib antigens
//     // live in antigen/src/stdlib/, scan that path.
//     let stdlib_path = Path::new(env!("CARGO_MANIFEST_DIR"))
//         .parent().unwrap()  // antigen/
//         .join("src/stdlib");
//
//     if !stdlib_path.exists() {
//         panic!("ATK-SC-6: stdlib path does not exist — supply-chain family not yet shipped");
//     }
//
//     let scan = scan_workspace(&stdlib_path, None).expect("scan completes");
//     let unsandboxed_proc_macro = scan.antigens.iter()
//         .find(|a| a.name == "unsandboxed-proc-macro")
//         .expect("UnsandboxedProcMacro must be in the stdlib");
//
//     let summary = unsandboxed_proc_macro.summary.as_deref().unwrap_or("");
//     let names_limitation = summary.contains("environment")
//         || summary.contains("time-bomb")
//         || summary.contains("sandbox-detection")
//         || summary.contains("environment-aware");
//
//     assert!(
//         names_limitation,
//         "ATK-SC-6: UnsandboxedProcMacro summary must name the sandbox-detection \
//          limitation (time-bomb attacks, environment-aware code). \
//          Current summary: {:?}", summary
//     );
// }
