//! Adversarial precision tests for the `cargo antigen oracle` CLI subfamily
//! (ADR-021 §M3 oracle-as-artifact-class).
//!
//! Tests invoke the compiled `cargo-antigen` binary via `std::process::Command`,
//! parallel to `atk_a3_cli.rs` for the attest/tolerate families. Each test
//! creates a fresh `tempfile::tempdir()` workspace so per-test state doesn't
//! leak across runs.
//!
//! Coverage:
//! - `oracle declare` — happy path, missing args, invalid kinds, empty
//!   rationale (Amendment 2), warning emitted for single-steward declaration
//!   (ATK-021-13)
//! - `oracle list` — empty workspace, one oracle, multiple oracles, format
//!   selection
//! - `oracle status` — existing oracle round-trip, missing oracle (exit 1)
//! - `oracle complete` — Draft→Complete happy path, must have ≥2 stewards
//!   (validation enforced via `schema::Oracle::validate`), wrong steward,
//!   empty rationale
//! - `oracle deprecate` — Complete→Deprecated happy path, optional
//!   `superseded_by` field
//! - `oracle retire` — Deprecated→Retired happy path, can also retire from
//!   Complete
//! - `oracle revoke` — `--invalidates-prior true|false` semantics distinguish
//!   from Retired (§D3)
//!
//! Exit code contract (parallel to `atk_a3_cli.rs)`:
//!   0 = success
//!   1 = user-visible validation failure (oracle not found, predicate fails)
//!   2 = IO / parse error

use std::path::{Path, PathBuf};
use std::process::Command;

/// Path to the compiled binary injected by Cargo at test-link time.
fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cargo-antigen"))
}

/// Run `cargo-antigen oracle <subcommand> <args>` and return `(exit_code, stdout, stderr)`.
fn oracle(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(bin())
        .arg("antigen")
        .arg("oracle")
        .args(args)
        .output()
        .expect("failed to run cargo-antigen");
    let exit = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    (exit, stdout, stderr)
}

/// Declare a minimal oracle with two stewards for use in tests that need a
/// Complete-able oracle. Returns the workspace root and oracle id.
fn declare_two_steward_oracle(dir: &Path, id: &str) {
    let (code, _stdout, stderr) = oracle(&[
        "declare",
        "--id",
        id,
        "--kind",
        "doi",
        "--reference",
        "10.1145/3593856",
        "--steward",
        "alice",
        "--steward",
        "bob",
        "--rationale",
        "domain authorities on this methodology",
        "--root",
        dir.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 0,
        "two-steward `oracle declare` must succeed: stderr={stderr}"
    );
}

/// Build the on-disk path the CLI writes for an oracle id.
fn oracle_file(dir: &Path, id: &str) -> PathBuf {
    dir.join(".antigen")
        .join("oracles")
        .join(format!("{id}.oracle.json"))
}

// ============================================================================
// oracle list — empty workspace + multi-oracle inventory
// ============================================================================

#[test]
fn atk_a3_oracle_cli_list_empty_workspace_exits_zero() {
    let tmp = tempfile::tempdir().unwrap();
    let (code, _stdout, stderr) = oracle(&["list", "--root", tmp.path().to_str().unwrap()]);
    assert_eq!(
        code, 0,
        "`oracle list` on empty workspace must exit 0: stderr={stderr}"
    );
    assert!(
        stderr.contains("No oracle records found"),
        "stderr must explain the empty result: {stderr}"
    );
}

#[test]
fn atk_a3_oracle_cli_list_after_declare_reports_oracle() {
    let tmp = tempfile::tempdir().unwrap();
    declare_two_steward_oracle(tmp.path(), "higham-2002");
    let (code, stdout, stderr) = oracle(&["list", "--root", tmp.path().to_str().unwrap()]);
    assert_eq!(code, 0, "list after declare must exit 0: stderr={stderr}");
    assert!(
        stdout.contains("higham-2002"),
        "stdout must list the declared oracle: stdout={stdout}"
    );
}

#[test]
fn atk_a3_oracle_cli_list_json_format_emits_valid_json_lines() {
    let tmp = tempfile::tempdir().unwrap();
    declare_two_steward_oracle(tmp.path(), "higham-2002");
    declare_two_steward_oracle(tmp.path(), "ieee-754-2019");
    let (code, stdout, _stderr) = oracle(&[
        "list",
        "--root",
        tmp.path().to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(code, 0);
    // JSON-lines format: each non-empty line is parseable JSON.
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(
        lines.len() >= 2,
        "json output should have at least 2 lines for 2 oracles: {stdout}"
    );
    for line in &lines {
        let parsed: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("json line not parseable: {line} ({e})"));
        assert!(
            parsed.get("id").is_some(),
            "json line missing id field: {line}"
        );
    }
}

// ============================================================================
// oracle declare — happy path + invariant guards
// ============================================================================

#[test]
fn atk_a3_oracle_cli_declare_creates_file_at_canonical_path() {
    let tmp = tempfile::tempdir().unwrap();
    declare_two_steward_oracle(tmp.path(), "test-oracle");
    let path = oracle_file(tmp.path(), "test-oracle");
    assert!(
        path.exists(),
        "declare must create canonical file at `.antigen/oracles/<id>.oracle.json`: {}",
        path.display()
    );
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("\"id\""),
        "file must contain id field: {content}"
    );
    assert!(
        content.contains("\"state\""),
        "file must contain state discriminator: {content}"
    );
}

#[test]
fn atk_a3_oracle_cli_declare_initial_state_is_draft() {
    let tmp = tempfile::tempdir().unwrap();
    declare_two_steward_oracle(tmp.path(), "draft-test");
    let content = std::fs::read_to_string(oracle_file(tmp.path(), "draft-test")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    let state = parsed.get("state").and_then(|s| s.get("state")).unwrap();
    assert_eq!(
        state.as_str(),
        Some("draft"),
        "newly-declared oracle must start in Draft state per ADR-021 §D3: {content}"
    );
}

#[test]
fn atk_a3_oracle_cli_declare_empty_rationale_rejected() {
    let tmp = tempfile::tempdir().unwrap();
    let (code, _stdout, stderr) = oracle(&[
        "declare",
        "--id",
        "no-rationale",
        "--kind",
        "doi",
        "--reference",
        "10.1145/3593856",
        "--steward",
        "alice",
        "--steward",
        "bob",
        "--rationale",
        "",
        "--root",
        tmp.path().to_str().unwrap(),
    ]);
    assert_ne!(
        code, 0,
        "empty rationale MUST be rejected per ADR-005 Amendment 2"
    );
    assert!(
        stderr.contains("rationale") || stderr.contains("Amendment"),
        "stderr must reference the rationale invariant: {stderr}"
    );
}

#[test]
fn atk_a3_oracle_cli_declare_single_steward_rejected_by_schema() {
    // Schema enforces ATK-021-13 minimum-2-stewards at save time
    // (Oracle::validate). A 1-steward declare CANNOT succeed — the CLI
    // emits a warning before save, but the save itself fails. This is
    // tier-honest: the schema is the enforcement layer, the CLI is the
    // friction layer (F28-R3).
    let tmp = tempfile::tempdir().unwrap();
    let (code, _stdout, stderr) = oracle(&[
        "declare",
        "--id",
        "single-steward",
        "--kind",
        "url",
        "--reference",
        "https://example.com/spec",
        "--steward",
        "alice",
        "--rationale",
        "lone steward for test",
        "--root",
        tmp.path().to_str().unwrap(),
    ]);
    assert_ne!(
        code, 0,
        "1-steward declare MUST fail at schema validation (ATK-021-13)"
    );
    assert!(
        stderr.contains("minimum 2") || stderr.contains("ATK-021-13"),
        "stderr must explain the minimum-2 invariant: {stderr}"
    );
}

// ============================================================================
// oracle status — round-trip happy path + missing oracle
// ============================================================================

#[test]
fn atk_a3_oracle_cli_status_for_declared_oracle_reports_state_and_stewards() {
    let tmp = tempfile::tempdir().unwrap();
    declare_two_steward_oracle(tmp.path(), "status-test");
    let (code, stdout, _stderr) = oracle(&[
        "status",
        "--id",
        "status-test",
        "--root",
        tmp.path().to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "status of existing oracle must succeed");
    assert!(
        stdout.contains("status-test"),
        "stdout must include oracle id: {stdout}"
    );
    assert!(
        stdout.contains("Draft"),
        "stdout must include current state (Draft for new oracle): {stdout}"
    );
    assert!(
        stdout.contains("alice") && stdout.contains("bob"),
        "stdout must list both stewards: {stdout}"
    );
}

#[test]
fn atk_a3_oracle_cli_status_missing_oracle_exits_one() {
    let tmp = tempfile::tempdir().unwrap();
    let (code, _stdout, stderr) = oracle(&[
        "status",
        "--id",
        "does-not-exist",
        "--root",
        tmp.path().to_str().unwrap(),
    ]);
    assert_ne!(code, 0, "missing oracle MUST NOT report success");
    assert!(
        stderr.contains("not found") || stderr.contains("does-not-exist"),
        "stderr must explain the missing oracle: {stderr}"
    );
}

// ============================================================================
// oracle complete — Draft→Complete transition + invariants
// ============================================================================

#[test]
fn atk_a3_oracle_cli_complete_draft_to_complete_happy_path() {
    let tmp = tempfile::tempdir().unwrap();
    declare_two_steward_oracle(tmp.path(), "complete-test");
    let (code, _stdout, stderr) = oracle(&[
        "complete",
        "--id",
        "complete-test",
        "--steward",
        "alice",
        "--version",
        "2nd-edition",
        "--rationale",
        "two stewards reviewed methodology section 6.3",
        "--root",
        tmp.path().to_str().unwrap(),
    ]);
    assert_eq!(
        code, 0,
        "two-steward oracle Draft→Complete must succeed: stderr={stderr}"
    );

    // Verify the state changed on disk.
    let content = std::fs::read_to_string(oracle_file(tmp.path(), "complete-test")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    let state = parsed.get("state").and_then(|s| s.get("state")).unwrap();
    assert_eq!(
        state.as_str(),
        Some("complete"),
        "oracle state must transition to Complete: {content}"
    );

    // Verify the transition log was appended.
    let transitions = parsed.get("transitions").unwrap().as_array().unwrap();
    assert_eq!(
        transitions.len(),
        1,
        "exactly one transition (draft→complete) recorded: {content}"
    );
    let t = &transitions[0];
    assert_eq!(t.get("from").and_then(|v| v.as_str()), Some("draft"));
    assert_eq!(t.get("to").and_then(|v| v.as_str()), Some("complete"));
    assert_eq!(
        t.get("authorized_by").and_then(|v| v.as_str()),
        Some("alice")
    );
}

#[test]
fn atk_a3_oracle_cli_complete_empty_rationale_rejected() {
    let tmp = tempfile::tempdir().unwrap();
    declare_two_steward_oracle(tmp.path(), "no-rationale-complete");
    let (code, _stdout, _stderr) = oracle(&[
        "complete",
        "--id",
        "no-rationale-complete",
        "--steward",
        "alice",
        "--version",
        "v1",
        "--rationale",
        "",
        "--root",
        tmp.path().to_str().unwrap(),
    ]);
    assert_ne!(
        code, 0,
        "empty rationale MUST be rejected on complete (Amendment 2)"
    );
}

#[test]
fn atk_a3_oracle_cli_complete_unknown_steward_rejected() {
    let tmp = tempfile::tempdir().unwrap();
    declare_two_steward_oracle(tmp.path(), "unknown-steward");
    let (code, _stdout, stderr) = oracle(&[
        "complete",
        "--id",
        "unknown-steward",
        "--steward",
        "carol", // not in stewards list
        "--version",
        "v1",
        "--rationale",
        "trying to authorize as non-steward",
        "--root",
        tmp.path().to_str().unwrap(),
    ]);
    assert_ne!(
        code, 0,
        "non-steward authorization MUST be rejected (ATK-021-15): stderr={stderr}"
    );
}

#[test]
fn atk_a3_oracle_cli_complete_missing_oracle_rejected() {
    let tmp = tempfile::tempdir().unwrap();
    let (code, _stdout, _stderr) = oracle(&[
        "complete",
        "--id",
        "no-such-oracle",
        "--steward",
        "alice",
        "--version",
        "v1",
        "--rationale",
        "completing a non-existent oracle",
        "--root",
        tmp.path().to_str().unwrap(),
    ]);
    assert_ne!(code, 0, "completing a missing oracle MUST fail");
}

// ============================================================================
// oracle deprecate — Complete→Deprecated, optional superseded_by
// ============================================================================

#[test]
fn atk_a3_oracle_cli_deprecate_after_complete_succeeds() {
    let tmp = tempfile::tempdir().unwrap();
    declare_two_steward_oracle(tmp.path(), "dep-test");
    // First complete it.
    let (code, _, _) = oracle(&[
        "complete",
        "--id",
        "dep-test",
        "--steward",
        "alice",
        "--version",
        "v1",
        "--rationale",
        "initial completion",
        "--root",
        tmp.path().to_str().unwrap(),
    ]);
    assert_eq!(code, 0);
    // Now deprecate.
    let (code, _, stderr) = oracle(&[
        "deprecate",
        "--id",
        "dep-test",
        "--steward",
        "bob",
        "--superseded-by",
        "dep-test-v2",
        "--rationale",
        "superseded by improved methodology",
        "--root",
        tmp.path().to_str().unwrap(),
    ]);
    assert_eq!(
        code, 0,
        "deprecate after complete must succeed: stderr={stderr}"
    );

    // Verify state + superseded_by.
    let content = std::fs::read_to_string(oracle_file(tmp.path(), "dep-test")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    let state = parsed.get("state").unwrap();
    assert_eq!(
        state.get("state").and_then(|v| v.as_str()),
        Some("deprecated")
    );
    assert_eq!(
        state.get("superseded_by").and_then(|v| v.as_str()),
        Some("dep-test-v2")
    );
}

#[test]
fn atk_a3_oracle_cli_deprecate_without_superseded_by_succeeds() {
    let tmp = tempfile::tempdir().unwrap();
    declare_two_steward_oracle(tmp.path(), "dep-no-succ");
    let (code, _, _) = oracle(&[
        "complete",
        "--id",
        "dep-no-succ",
        "--steward",
        "alice",
        "--version",
        "v1",
        "--rationale",
        "initial",
        "--root",
        tmp.path().to_str().unwrap(),
    ]);
    assert_eq!(code, 0);
    let (code, _, stderr) = oracle(&[
        "deprecate",
        "--id",
        "dep-no-succ",
        "--steward",
        "alice",
        "--rationale",
        "no named successor; methodology simply obsoleted",
        "--root",
        tmp.path().to_str().unwrap(),
    ]);
    assert_eq!(
        code, 0,
        "deprecate without superseded_by must succeed (Optional per §D3): stderr={stderr}"
    );
}

// ============================================================================
// oracle retire — preserves prior attestations at Execution
// ============================================================================

#[test]
fn atk_a3_oracle_cli_retire_from_complete_succeeds() {
    let tmp = tempfile::tempdir().unwrap();
    declare_two_steward_oracle(tmp.path(), "retire-test");
    let (code, _, _) = oracle(&[
        "complete",
        "--id",
        "retire-test",
        "--steward",
        "alice",
        "--version",
        "v1",
        "--rationale",
        "complete",
        "--root",
        tmp.path().to_str().unwrap(),
    ]);
    assert_eq!(code, 0);
    let (code, _, stderr) = oracle(&[
        "retire",
        "--id",
        "retire-test",
        "--steward",
        "alice",
        "--rationale",
        "oracle no longer authoritative; prior attestations preserved",
        "--root",
        tmp.path().to_str().unwrap(),
    ]);
    assert_eq!(
        code, 0,
        "retire from Complete must succeed: stderr={stderr}"
    );
    let content = std::fs::read_to_string(oracle_file(tmp.path(), "retire-test")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(
        parsed
            .get("state")
            .and_then(|s| s.get("state"))
            .and_then(|v| v.as_str()),
        Some("retired")
    );
}

// ============================================================================
// oracle revoke — invalidates_prior flag distinguishes from Retired
// ============================================================================

#[test]
fn atk_a3_oracle_cli_revoke_with_invalidates_prior_true_serializes_flag() {
    let tmp = tempfile::tempdir().unwrap();
    declare_two_steward_oracle(tmp.path(), "revoke-true");
    let (code, _, _) = oracle(&[
        "complete",
        "--id",
        "revoke-true",
        "--steward",
        "alice",
        "--version",
        "v1",
        "--rationale",
        "init",
        "--root",
        tmp.path().to_str().unwrap(),
    ]);
    assert_eq!(code, 0);
    let (code, _, stderr) = oracle(&[
        "revoke",
        "--id",
        "revoke-true",
        "--steward",
        "alice",
        "--rationale",
        "fundamentally incorrect methodology; prior attestations must be demoted",
        "--invalidates-prior",
        "--root",
        tmp.path().to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "revoke must succeed: stderr={stderr}");

    let content = std::fs::read_to_string(oracle_file(tmp.path(), "revoke-true")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    let state = parsed.get("state").unwrap();
    assert_eq!(
        state.get("state").and_then(|v| v.as_str()),
        Some("revoked"),
        "state must be revoked: {content}"
    );
    assert_eq!(
        state
            .get("invalidates_prior_attestations")
            .and_then(serde_json::Value::as_bool),
        Some(true),
        "invalidates_prior_attestations flag must be true: {content}"
    );
}

#[test]
fn atk_a3_oracle_cli_revoke_without_invalidates_prior_defaults_false() {
    let tmp = tempfile::tempdir().unwrap();
    declare_two_steward_oracle(tmp.path(), "revoke-false");
    let (code, _, _) = oracle(&[
        "complete",
        "--id",
        "revoke-false",
        "--steward",
        "alice",
        "--version",
        "v1",
        "--rationale",
        "init",
        "--root",
        tmp.path().to_str().unwrap(),
    ]);
    assert_eq!(code, 0);
    let (code, _, stderr) = oracle(&[
        "revoke",
        "--id",
        "revoke-false",
        "--steward",
        "alice",
        "--rationale",
        "oracle text was correct at the time but is no longer authoritative",
        "--root",
        tmp.path().to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "revoke without flag must succeed: stderr={stderr}");

    let content = std::fs::read_to_string(oracle_file(tmp.path(), "revoke-false")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    let state = parsed.get("state").unwrap();
    assert_eq!(
        state
            .get("invalidates_prior_attestations")
            .and_then(serde_json::Value::as_bool),
        Some(false),
        "no --invalidates-prior flag → false (preserves prior attestations): {content}"
    );
}

// ============================================================================
// Round-trip: declare → status → complete → status → deprecate → status
// ============================================================================

#[test]
fn atk_a3_oracle_cli_full_lifecycle_round_trip() {
    let tmp = tempfile::tempdir().unwrap();
    declare_two_steward_oracle(tmp.path(), "lifecycle");

    // Initial status: Draft.
    let (code, stdout, _) = oracle(&[
        "status",
        "--id",
        "lifecycle",
        "--root",
        tmp.path().to_str().unwrap(),
    ]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("Draft"),
        "initial state must be Draft: {stdout}"
    );

    // Complete.
    let (code, _, _) = oracle(&[
        "complete",
        "--id",
        "lifecycle",
        "--steward",
        "alice",
        "--version",
        "v1",
        "--rationale",
        "two stewards reviewed methodology",
        "--root",
        tmp.path().to_str().unwrap(),
    ]);
    assert_eq!(code, 0);

    // Status now Complete.
    let (_, stdout, _) = oracle(&[
        "status",
        "--id",
        "lifecycle",
        "--root",
        tmp.path().to_str().unwrap(),
    ]);
    assert!(stdout.contains("Complete"), "after complete: {stdout}");

    // Deprecate.
    let (code, _, _) = oracle(&[
        "deprecate",
        "--id",
        "lifecycle",
        "--steward",
        "bob",
        "--rationale",
        "superseded by newer methodology",
        "--root",
        tmp.path().to_str().unwrap(),
    ]);
    assert_eq!(code, 0);

    // Status now Deprecated.
    let (_, stdout, _) = oracle(&[
        "status",
        "--id",
        "lifecycle",
        "--root",
        tmp.path().to_str().unwrap(),
    ]);
    assert!(stdout.contains("Deprecated"), "after deprecate: {stdout}");

    // Transition log has 2 entries (Draft→Complete, Complete→Deprecated).
    let content = std::fs::read_to_string(oracle_file(tmp.path(), "lifecycle")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    let transitions = parsed.get("transitions").unwrap().as_array().unwrap();
    assert_eq!(
        transitions.len(),
        2,
        "lifecycle should record 2 transitions: {content}"
    );
}
