//! Adversarial fixtures for the VCS-Information-Loss Family (ADR-026).
//!
//! All tests are #[ignore] until the VCS stdlib antigens and substrate-witness
//! evaluators ship. When v02-impl-vcs-info-loss lands:
//!
//! 1. Remove #[ignore] from each test.
//! 2. Run `cargo test atk_vcs_adversarial` — tests should FAIL.
//! 3. Fix the production code so tests PASS.
//! 4. These tests are now regression guards.
//!
//! Preemptive attack-surface documentation.

// When the module exists, add:
// use antigen::vcs::{TriageDecision, VcsAttestation};
// use antigen::scan::{ScanReport, scan_workspace};
// use antigen::audit::{AuditHint, audit_workspace};

// ============================================================================
// ATK-VCS-1: vcs-rollback-without-triage-commit detection — false negative
//
// ADR-026 §Audit-hint vocabulary lists `vcs-rollback-without-triage-commit`.
// SPEC GAP: how does cargo antigen detect that a commit IS a rollback?
// Candidates: (a) commit message contains "Revert", (b) `git revert` metadata,
// (c) commit introduces a `Triage-Decision:` trailer, (d) explicit annotation.
//
// FINDING: A rollback commit authored without `git revert` (e.g.,
// manually cherry-picking the inverse diff with message "Fix: reapply old state")
// may NOT be detected as a rollback. The hint fires on git-revert commits
// but misses manual inverse-cherry-picks. This is a false-negative class.
//
// Expected: either (a) the scan documents this gap as residual risk, OR
// (b) the scan uses a triage-commit-trailer requirement (structural: ANY
// commit that lacks a Triage-Decision trailer when the workspace has
// rollback-discipline active is flagged — not just `git revert` commits).
// ============================================================================

#[test]
#[ignore = "VCS stdlib not yet implemented — remove ignore when v02-impl-vcs-info-loss ships"]
fn atk_vcs_1_manual_rollback_without_git_revert_may_miss_hint() {
    // A commit that manually reverts state (no `git revert` ancestry) but
    // lacks a `#[triage_commit]` marker should still fire
    // `vcs-rollback-without-triage-commit`. If detection is keyed on
    // `git revert` metadata only, manual rollbacks are a silent gap.
    todo!("implement when VCS stdlib ships")
}

// ============================================================================
// ATK-VCS-2: vcs-force-push-erased-substantive-history — force-with-lease
//
// ADR-026 §D2 explicitly says --force-with-lease IS covered by the hint
// (both --force and --force-with-lease erase history). The hint name is
// `vcs-force-push-erased-substantive-history` for both.
//
// FINDING: `--force-with-lease` can be used when the local
// tracking ref matches remote, which means it DOES erase remote history.
// A git hook that checks only for `--force` flags and not `--force-with-lease`
// flags would miss this class. Test that the pre-push hook catches
// `--force-with-lease` as a push that erases history.
//
// Expected: the install-hooks command installs a hook that catches BOTH.
// ============================================================================

#[test]
#[ignore = "VCS stdlib not yet implemented — remove ignore when v02-impl-vcs-info-loss ships"]
fn atk_vcs_2_force_with_lease_triggers_hint_same_as_force() {
    // A `git push --force-with-lease` must trigger the same hint as
    // `git push --force`. If the hook checks only for `--force` flags,
    // `--force-with-lease` is a bypass that the audit hint misses.
    todo!("implement when VCS stdlib ships")
}

// ============================================================================
// ATK-VCS-3: TriageDecision::Green with rollback_due_within_minutes = 0
//
// This is already partially covered by the parse-level test
// `triage_commit_zero_deadline_is_rejected` in parse.rs. But the SEMANTIC
// variant: Green triage means "no regression detected" — no rollback planned.
// A tight deadline (e.g., 1 minute) on Green triage is semantically absurd.
//
// FINDING: The current validate() does not cross-check triage
// decision against deadline reasonableness. A Green triage with a 1-minute
// deadline passes validation but signals confusion about the triage semantics.
//
// ADR-026 §Decision says Green = "no functional regression; analysis chain
// attests non-regression." A rollback deadline on a Green triage suggests
// the developer doesn't understand what Green means.
//
// Expected: either (a) an audit hint `vcs-triage-decision-contradicts-deadline`
// OR (b) documented in ADR-026 as intentional (some teams use Green-with-deadline
// for precautionary rollback capability, which is a legitimate use case).
// This test documents the ambiguity for the ADR-026 Amendment 1 authors.
// ============================================================================

#[test]
#[ignore = "VCS stdlib not yet implemented — remove ignore when v02-impl-vcs-info-loss ships"]
fn atk_vcs_3_green_triage_with_deadline_semantic_inconsistency() {
    // Green = no regression; rollback_due_within_minutes = 1 = imminent rollback.
    // These are semantically contradictory. Should the audit surface a hint,
    // or is this a legitimate "Green but keeping rollback option open" posture?
    // Test documents the ambiguity. Production code must make a choice.
    todo!("implement when VCS stdlib ships")
}

// ============================================================================
// ATK-VCS-4: vcs-enforcement-friction-only-no-server-hook false negative
//
// ADR-026 §Enforcement-Surface says FrictionOnly mode emits
// `vcs-enforcement-friction-only-no-server-hook` at audit time.
// The DETECTION mechanism: how does cargo antigen audit KNOW whether the
// remote has server-side hooks installed?
//
// FINDING: Without checking the actual remote configuration,
// the audit must rely on the `ServerSideEnforcementMode` declaration in the
// codebase. If an adopter declares `enforcement = Structural` but hasn't
// actually installed server hooks, the audit is fooled — it assumes Structural
// enforcement is active based on the declaration alone.
//
// This is the foundational substrate-alignment vs functional-correctness split
// applied to VCS enforcement: the DECLARATION says Structural; the ACTUAL
// remote config may be FrictionOnly. An audit that doesn't check the remote
// has no way to detect this divergence.
//
// Expected: audit documentation acknowledges this gap explicitly — the
// `vcs_server_side_enforcement_active(repo, antigen_name)` substrate-witness
// is the mitigation (it actually checks the remote config). Tests that the
// audit emits `vcs-enforcement-friction-only-no-server-hook` when the
// substrate-witness is absent from the #[immune] declaration.
// ============================================================================

#[test]
#[ignore = "VCS stdlib not yet implemented — remove ignore when v02-impl-vcs-info-loss ships"]
fn atk_vcs_4_structural_enforcement_declared_but_server_hook_not_installed() {
    // If ServerSideEnforcementMode::Structural is declared but the repo
    // does not have `receive.denyNonFastForwards = true`, the declaration
    // is a lie. The audit should require vcs_server_side_enforcement_active()
    // substrate-witness to verify. Absence of that witness = FrictionOnly
    // in practice regardless of declaration.
    todo!("implement when VCS stdlib ships")
}

// ============================================================================
// ATK-VCS-5: #[triage_commit] triaged_by field — empty string or whitespace
//
// This is the whitespace-only variant of the rollback_target finding from
// the parse-level pin test. `triaged_by = "   "` passes the current
// non-empty check. The field is supposed to name a role or person who
// performed the triage — whitespace-only is meaningless.
//
// FINDING: validate() checks Some("") but "   " is not empty.
// This is structurally identical to the rollback_target whitespace gap.
//
// Expected: validate() trims the string before checking emptiness, OR
// the ADR specifies that triaged_by accepts any non-empty string including
// whitespace (which would be a deliberate permissive choice, not a gap).
// ============================================================================

#[test]
#[ignore = "VCS stdlib not yet implemented — remove ignore when v02-impl-vcs-info-loss ships"]
fn atk_vcs_5_triaged_by_whitespace_only_should_be_rejected() {
    // triaged_by = "   " should fail — whitespace-only is not a meaningful
    // role or person name. The current validate() doesn't catch this.
    // This test documents the gap; the fix is trim+check in validate().
    todo!("implement when VCS stdlib ships")
}

// ============================================================================
// ATK-VCS-6: branch deletion attestation — deleted then re-created branch
//
// ADR-026 names `BranchDeletionWithoutAttestation` as one of the 11 VCS
// stdlib antigens. The attestation mechanism: `vcs_attest_branch_deletion(branch, by_role)`.
//
// FINDING: What happens when a branch is deleted WITH attestation,
// then re-created and deleted AGAIN without attestation? The audit must
// track whether attestation applies to the CURRENT deletion or any prior
// deletion of that branch name. Branch names are reusable; attestation
// should be per-deletion-event, not per-branch-name.
//
// Expected: the attestation substrate (.attest/vcs/) uses a unique identifier
// per deletion event (timestamp? commit sha of the deleted head?) not just
// the branch name. A branch name re-use without a new attestation should
// fire the hint.
// ============================================================================

#[test]
#[ignore = "VCS stdlib not yet implemented — remove ignore when v02-impl-vcs-info-loss ships"]
fn atk_vcs_6_branch_deletion_attestation_per_event_not_per_name() {
    // Deleting branch 'feature/x', attesting it, then re-creating and
    // deleting 'feature/x' again without attestation should fire the hint.
    // If attestation is keyed by branch name only, the second deletion
    // is silently covered by the first attestation. That's a false negative.
    todo!("implement when VCS stdlib ships")
}
