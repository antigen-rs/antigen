//! Example: VCS-information-loss family (ADR-026).
//!
//! Eleven stdlib antigens covering modern-git-workflow failure modes that
//! erase load-bearing history. This example shows the most common patterns.
//!
//! ## The central insight
//!
//! Git operations that rewrite or erase history remove the structural memory
//! of WHY decisions were made. `git reset --hard`, `git push --force`,
//! `git squash-merge`, and unrecorded rollbacks are the most common vectors.
//! These are `SubstrateAlignment` failures: the git-history representation
//! diverges from the actual state of why-this-was-done.
//!
//! ## What `#[triage_commit]` is
//!
//! `#[triage_commit]` is the SPEECH-ACT marker for a rollback-as-triage
//! decision. It is placed on the function that PERFORMS the rollback, before
//! the rollback happens — the marker says "I have diagnosed this as requiring
//! rollback; here is my rationale." The `Triage-Decision:` git commit trailer
//! is the substrate-witness that links the triage commit to the rollback commit.
//!
//! `#[triage_commit]` is NOT `#[orient]`: `orient` says "I don't have a defense
//! yet, I'll get one by `<date>`." `triage_commit` says "I have diagnosed this as
//! needing rollback NOW, here is why." Different speech-acts, different scopes.
//!
//! ## Run this example
//!
//! ```sh
//! cargo run --example vcs_info_loss --package antigen
//! ```
//!
//! See ADR-026 for the full family, witness leaves, and enforcement model.

#![allow(dead_code, unused_variables, unused_imports)]

use antigen::stdlib::vcs_info_loss::{
    ForcePushErasingHistory, RefactorWithoutPreservationOfWhy, RollbackWithoutTriageCommit,
    SquashMergeLosingIntermediateState,
};
use antigen::{TriageDecision, presents, triage_commit};

// ============================================================================
// Pattern 1: RollbackWithoutTriageCommit — force-reset without a triage record
//
// The vulnerable pattern: a function that performs a git reset --hard without
// first placing a #[triage_commit] marker and a Triage-Decision: commit trailer.
// ============================================================================

/// Reset the repository to a known-good snapshot.
///
/// VULNERABLE: this rollback function has no `#[triage_commit]` annotation.
/// The WHY-we-rolled-back lives only in the rolling-back-developer's head;
/// once this commit lands, the rationale is gone. Future developers who grep
/// git log for "why did we reset to snapshot X" will find nothing.
///
/// Detection model (ADR-026 D1): must be detected at COMMIT-TIME via hooks;
/// post-hoc history inspection cannot catch `git reset --hard` rollbacks.
#[presents(RollbackWithoutTriageCommit)]
pub fn rollback_to_snapshot(snapshot_sha: &str) -> Result<(), String> {
    // Simulate: git reset --hard <snapshot_sha>
    println!("[ROLLBACK] Resetting to {snapshot_sha} (no triage record)");
    Ok(())
}

/// Reset the repository with full triage documentation.
///
/// DEFENDED: the function carries `#[triage_commit]` before the rollback
/// is performed, and the caller is expected to write a `Triage-Decision:`
/// trailer to the git commit message. The `signed_trailer(key = "Triage-Decision")`
/// substrate-witness confirms the trailer is present at audit time.
///
/// `requires = signed_trailer(key = "Triage-Decision")` evaluates whether the
/// commit that triggered this rollback has a Triage-Decision trailer. Without
/// the trailer, audit reports `rollback-without-triage-commit`.
#[triage_commit(
    triage_decision = TriageDecision::Red,
    rollback_target = "83a7f2c",
    triaged_by = "oncall-reviewer",
    rationale = "Confirmed regression in payment path (issue #4421); snapshot 83a7f2c is the last green CI for this module",
    rollback_due_within_minutes = 30
)]
// ADR-029: `requires = signed_trailer(key = "Triage-Decision")` lives directly
// on `#[presents]`. The triage_commit annotation + the `Triage-Decision:` git
// trailer link this rollback to the diagnosis; `signed_trailer` confirms the
// trailer is present in the commit at audit time.
#[presents(
    RollbackWithoutTriageCommit,
    requires = signed_trailer(key = "Triage-Decision"),
)]
pub fn rollback_to_snapshot_with_triage(snapshot_sha: &str) -> Result<(), String> {
    println!("[ROLLBACK] Resetting to {snapshot_sha} (triage documented)");
    Ok(())
}

// ============================================================================
// Pattern 2: ForcePushErasingHistory — the immune-amnesia cognate
//
// ForcePushErasingHistory is the CENTRAL cognate of the VCS family:
// Mina et al. 2015 (Science) — measles infects memory lymphocytes, erasing
// immunological memory. git push --force erases commit history.
// ============================================================================

/// Perform a force-push to overwrite remote history.
///
/// VULNERABLE: no attestation of WHY the force-push was necessary, no record
/// of what history was erased. The remote reflog will not preserve the old
/// head; once pushed, those commits are gone from the shared history.
///
/// Biology cognate: immune amnesia (measles) — catastrophic loss of memory-
/// carrying substrates with documented harm and structural defense patterns.
#[presents(ForcePushErasingHistory)]
pub fn force_push_to_main(branch: &str) -> Result<(), String> {
    // Simulate: git push --force origin <branch>
    println!("[FORCE-PUSH] Rewriting {branch} (no attestation)");
    Ok(())
}

/// Force-push with a Force-Push-Attestation commit trailer.
///
/// DEFENDED: the `Force-Push-Attestation:` git trailer documents why the
/// force-push was necessary and what history was preserved. The
/// `signed_trailer` witness confirms the trailer is present at audit time.
// ADR-029: `requires = signed_trailer(key = "Force-Push-Attestation")` lives
// directly on `#[presents]`. The `Force-Push-Attestation:` trailer records
// the preserved-history commit range and reason; `signed_trailer` confirms
// it is present before the push is considered attested.
#[presents(
    ForcePushErasingHistory,
    requires = signed_trailer(key = "Force-Push-Attestation"),
)]
pub fn force_push_with_attestation(branch: &str, preserved_range: &str) -> Result<(), String> {
    // Caller adds git commit trailer: Force-Push-Attestation: <reason + range>
    println!("[FORCE-PUSH] Rewriting {branch}; preserved range: {preserved_range}");
    Ok(())
}

// ============================================================================
// Pattern 3: RefactorWithoutPreservationOfWhy — original-antigenic-sin
//
// A refactor that removes load-bearing code without recording WHY the
// original shape existed. Future developers re-derive the bug.
// ============================================================================

/// Simplify a function by removing what looks like dead code.
///
/// VULNERABLE: the early-return for the negative-zero case LOOKS redundant
/// (x is already 0.0, why special-case it?), but it's IEEE 754 sign-bit
/// preservation. Removing it breaks `f(-0.0) → -0.0` while keeping
/// `f(-0.0) == 0.0`. The refactor has no Preserves-Why trailer and no
/// `#[descended_from]` link — the WHY is gone.
#[presents(RefactorWithoutPreservationOfWhy)]
pub fn refactored_sinh(x: f64) -> f64 {
    // Simplified, but sign-bit preservation removed. Looks clean, breaks IEEE 754.
    x.sinh()
}

/// The preserved version with WHY captured.
///
/// DEFENDED: `#[descended_from(SignedZeroDiscipline)]` propagates the
/// failure-class memory, and a `Preserves-Why:` git trailer points to the
/// original issue or ADR. The refactor cannot silently erase the context.
// ADR-029: `requires = signed_trailer(key = "Preserves-Why")` lives directly
// on `#[presents]`. The `Preserves-Why:` trailer links this refactor to the
// signed-zero discipline ADR; `signed_trailer` confirms the WHY-link is
// present so future readers see the constraint.
#[presents(
    RefactorWithoutPreservationOfWhy,
    requires = signed_trailer(key = "Preserves-Why"),
)]
pub fn preserved_sinh(x: f64) -> f64 {
    // IEEE 754 sign-bit preservation: sinh(-0.0) must return -0.0.
    if x == 0.0 {
        return x; // preserve sign bit: -0.0 → -0.0, not +0.0
    }
    x.sinh()
}

// ============================================================================
// Pattern 4: SquashMergeLosingIntermediateState
//
// Squash-merging a feature branch condenses N commits into 1, losing the
// intermediate decision trail (which commit introduced which invariant,
// which red-bar fixed what).
// ============================================================================

/// Merge a feature branch by squashing all commits into one.
///
/// VULNERABLE: squash-merging is ergonomically attractive (clean main log)
/// but erases intermediate state — specifically, the commit that introduced
/// a subtle invariant, the commit that re-added a guard that was previously
/// removed, the red-bar/green-bar rhythm that documents what each fix did.
/// Biology cognate: affinity-maturation history loss — the process of
/// selection is erased; only the selected result survives.
#[presents(SquashMergeLosingIntermediateState)]
pub fn squash_merge_feature_branch(feature_branch: &str) -> Result<(), String> {
    // Simulate: git merge --squash <feature_branch>
    println!("[SQUASH-MERGE] Squashing {feature_branch} into a single commit");
    Ok(())
}

fn main() {
    println!("=== antigen VCS-information-loss example ===");
    println!();
    println!("Pattern 1: RollbackWithoutTriageCommit");
    println!("  rollback_to_snapshot: PRESENTS — no triage record");
    println!(
        "  rollback_to_snapshot_with_triage: IMMUNE — triage_commit + signed_trailer(Triage-Decision)"
    );
    println!();
    println!("Pattern 2: ForcePushErasingHistory (central cognate: immune amnesia)");
    println!("  force_push_to_main: PRESENTS — no attestation");
    println!("  force_push_with_attestation: IMMUNE — Force-Push-Attestation trailer");
    println!();
    println!("Pattern 3: RefactorWithoutPreservationOfWhy (original antigenic sin)");
    println!("  refactored_sinh: PRESENTS — WHY context erased");
    println!("  preserved_sinh: IMMUNE — Preserves-Why trailer + descended_from");
    println!();
    println!("Pattern 4: SquashMergeLosingIntermediateState (affinity-maturation loss)");
    println!("  squash_merge_feature_branch: PRESENTS — intermediate commits lost");
    println!();
    println!("Key distinction — #[triage_commit] vs #[orient]:");
    println!("  orient: 'I don't have a defense yet; I'll get one by <date>'");
    println!("  triage_commit: 'I have diagnosed this as needing rollback NOW; here is why'");
    println!("  Different speech-acts. orient = deferral; triage_commit = decision.");
    println!();
    println!("Scan for presentations:");
    println!("  cargo run --bin cargo-antigen -- antigen scan --root antigen/examples");

    let _ = rollback_to_snapshot("83a7f2c");
    let _ = rollback_to_snapshot_with_triage("83a7f2c");
    let _ = force_push_to_main("feature/auth-refactor");
    let _ = force_push_with_attestation("feature/auth-refactor", "HEAD~12..HEAD~1");
    let _ = squash_merge_feature_branch("feature/payment-rework");
}
