//! # VCS-Information-Loss Family (ADR-026)
//!
//! Eleven stdlib antigens covering the modern-git-workflow failure modes
//! that erase load-bearing history. Per ADR-026 the rollback-as-triage
//! discipline is enforced via the sibling `#[triage_commit]` macro (ADR-026
//! Amendment 1); the antigens here name the failure-classes that
//! `#[triage_commit]` + the `cargo antigen vcs` CLI subfamily defend against.
//!
//! ## The central cognate (NON-NEGOTIABLE)
//!
//! **`ForcePushErasingHistory` ↔ Immune Amnesia (measles)** —
//! Mina et al. 2015, *Science*. Measles virus infects memory lymphocytes;
//! post-measles patients show increased susceptibility to other pathogens
//! for 2-3 years. The catastrophic loss of memory-carrying substrates with
//! documented harm and structural defense patterns is the foundational
//! cognate of this entire family. Biology PREDICTS the failure mode and
//! defense pattern.
//!
//! ## Biology cognate table (per ADR-026 §Biology per-primitive cognates)
//!
//! | Antigen | Biological cognate |
//! |---|---|
//! | `ForcePushErasingHistory` | **Immune amnesia (measles)** — central cognate |
//! | `RefactorWithoutPreservationOfWhy` | Original antigenic sin |
//! | `SquashMergeLosingIntermediateState` | Affinity-maturation history loss |
//! | `CherryPickLosingOriginalContext` | Class-switching context loss |
//! | `RebaseRewritingHistoryWithoutLog` | V(D)J recombination without per-cell record |
//! | `StashedWorkAbandoned` | Anergy / primed-without-activation |
//! | `RollbackWithoutTriageCommit` | (cognate not yet assigned; follow-up) |
//! | `BranchDeletionWithoutAttestation` | (cognate not yet assigned; follow-up) |
//! | `UnpushedBranchWithSubstantiveWork` | (cognate not yet assigned; follow-up) |
//! | `MergeConflictResolutionWithoutAttestation` | (cognate not yet assigned; follow-up) |
//! | `AmendedCommitWithoutOldHashPreservation` | (cognate not yet assigned; follow-up) |
//!
//! The five "not yet assigned" entries are deliberate — per
//! `feedback_clean_without_snag_is_argument_mode`, a clean off-the-cuff
//! cognate without substrate is argument-mode, not prediction. The
//! biology-grounding deepening is a follow-up.
//!
//! ## Antigen-category (per ADR-028)
//!
//! All 11 antigens declare `category = SubstrateAlignment` — the
//! representation (git history, commit-message rationale, branch state) is
//! what diverges from the actual state of why-this-was-done. Witness
//! evaluators (git trailers, branch-archive attestation sidecars, rollback
//! triage-chain commits) read the substrate at audit-time; they do NOT
//! exercise behaviour.
//!
//! ## How these antigens are evaluated
//!
//! Unlike source-pattern antigens (which fire via `cargo antigen scan`
//! AST-walking), VCS-info-loss antigens fire via **commit-time hooks +
//! audit-time substrate-witness evaluation** against the git repository's
//! commit-history, branch state, and `.attest/vcs/` sidecars. Drive
//! evaluation via the `cargo antigen vcs` CLI subfamily.
//!
//! ## Enforcement model (per ADR-026 §Decision)
//!
//! - **Friction-only (default v0.2)**: client-side hooks +
//!   `cargo antigen vcs check-commit` audit-time; makes bad behavior
//!   DELIBERATE rather than ACCIDENTAL. Explicitly NOT preventive — `git
//!   commit --no-verify` and `git update-ref -d` bypass client-side hooks.
//! - **Structural (v0.2.1+)**: server-side pre-receive hooks; requires
//!   adopter to control the git remote. See
//!   [`vcs::ServerSideEnforcementMode`](crate::vcs::ServerSideEnforcementMode).
//!
//! ## Fingerprint discipline
//!
//! Like `supply_chain.rs`, the `fingerprint` field uses a uniform
//! `doc_contains("ADR-026")` form: these antigens are NOT matched by
//! AST-walking. The active matching surface is `cargo antigen vcs`.

use crate::antigen;

// ============================================================================
// 1. RollbackWithoutTriageCommit
// ============================================================================

/// A rollback commit landed without a preceding `#[triage_commit]` marker
/// + `Triage-Decision: <sha>` commit trailer.
///
/// **Failure class**: rollback-as-treatment without informed-consent
/// chart-documentation. The why-we-rolled-back lives only in the rolling-
/// back-developer's head, evaporating once the commit lands.
///
/// **Detection model** (per ADR-026 D1): MUST operate at COMMIT-TIME via
/// hooks. Cannot be detected by post-hoc history inspection — `git reset
/// --hard` removes traces.
///
/// **Biology cognate**: not yet assigned; follow-up.
///
/// **Defense**: `cargo antigen vcs install-hooks` installs the pre-commit
/// hook that requires a `Triage-Decision:` trailer when the commit is a
/// revert/reset of substantive work.
#[antigen(
    name = "rollback-without-triage-commit",
    category = AntigenCategory::SubstrateAlignment,
    family = "vcs-information-loss",
    summary = "Rollback commit without a Triage-Decision trailer pointing to a #[triage_commit] marker. Rationale-before-action vs after-the-fact.",
    references = ["ADR-026", "ADR-026#D1"]
)]
pub struct RollbackWithoutTriageCommit;

// ============================================================================
// 2. RefactorWithoutPreservationOfWhy
// ============================================================================

/// A refactor commit that removes or reshapes load-bearing code without
/// preserving WHY the original shape existed.
///
/// **Failure class**: tribal-knowledge erosion. The corrected code reads
/// cleanly but no longer carries the failure-class memory that produced
/// the original shape. Future developers re-derive the bug.
///
/// **Biology cognate**: Original antigenic sin — the immune system's bias
/// toward already-known antigens. Refactors that erase the original
/// failure-class context bias future developers toward the corrected-but-
/// uncontextualized memory of the code, blinding them to the original
/// antigen.
///
/// **Defense**: `#[descended_from]` propagation per ADR-018; commit-message
/// trailer `Preserves-Why: <ADR|issue|commit-sha>` flagged by audit when
/// load-bearing files are touched.
#[antigen(
    name = "refactor-without-preservation-of-why",
    category = AntigenCategory::SubstrateAlignment,
    family = "vcs-information-loss",
    summary = "Refactor that erases load-bearing 'why' context without a Preserves-Why trailer or descended_from chain. Original antigenic sin: bias toward known-shape blinds future readers to the original antigen.",
    references = ["ADR-026", "ADR-018"]
)]
pub struct RefactorWithoutPreservationOfWhy;

// ============================================================================
// 3. BranchDeletionWithoutAttestation
// ============================================================================

/// A branch was deleted without a corresponding attestation sidecar at
/// `.attest/vcs/branch-archive/<branch-name>.json` recording who deleted
/// it and why.
///
/// **Failure class**: branch state evaporation. `git branch -d` and
/// especially `git update-ref -d` leave no audit trail on the remote;
/// reflog entries are local and expire.
///
/// **Detection**: branch-delete-time hook + `.attest/vcs/branch-archive/`
/// sidecar audit. Client-side bypassable via `git update-ref -d`; server-
/// side enforcement (v0.2.1+) closes the gap.
///
/// **Biology cognate**: not yet assigned; follow-up.
///
/// **Defense**: `cargo antigen vcs branch-archive <branch> --by <role>
/// --rationale "..."` records the attestation BEFORE the delete.
#[antigen(
    name = "branch-deletion-without-attestation",
    category = AntigenCategory::SubstrateAlignment,
    family = "vcs-information-loss",
    summary = "Branch deleted without a .attest/vcs/branch-archive/ attestation sidecar recording who + why. Client-side bypassable via git update-ref -d; structural enforcement requires server-side hook.",
    references = ["ADR-026"]
)]
pub struct BranchDeletionWithoutAttestation;

// ============================================================================
// 4. ForcePushErasingHistory (CENTRAL FOUNDATIONAL CONGNATE)
// ============================================================================

/// A force-push (either `--force` OR `--force-with-lease` per ADR-026 D2)
/// erased substantive commit history from a published branch.
///
/// **CENTRAL FOUNDATIONAL FAMILY COGNATE**: this is the antigen that
/// proves biology PREDICTS the failure mode + defense pattern, not just
/// rhymes with it.
///
/// **Biology cognate — Immune amnesia (measles)**:
/// Mina et al. 2015, *Science* — measles virus infects memory lymphocytes;
/// post-measles patients show increased susceptibility to other pathogens
/// for 2-3 years. The catastrophic loss of memory-carrying substrates,
/// documented harm, and structural defense patterns is the foundational
/// shape of this entire family (NON-NEGOTIABLE): this is
/// IDENTITY-tier cognate, not RHYME-tier — the structural prediction is
/// substantive.
///
/// **D2 coverage** (per ADR-026): covers both `--force` AND
/// `--force-with-lease`. The lease variant is NOT a safe alternative; it
/// only narrows the race window, not the loss-of-history failure class.
///
/// **Defense**: `cargo antigen vcs install-server-hooks` installs the
/// server-side pre-receive hook that refuses non-fast-forward pushes
/// without a `Force-Push-Attestation:` trailer pointing to an attestation
/// sidecar. `receive.denyNonFastForwards = true` on the remote provides
/// the structural-mode hard-stop per ADR-026 §Enforcement-Surface.
#[antigen(
    name = "force-push-erasing-history",
    category = AntigenCategory::SubstrateAlignment,
    family = "vcs-information-loss",
    summary = "Force-push (--force OR --force-with-lease per ADR-026 D2) erasing substantive history. Central family cognate: Immune amnesia (measles) — memory-carrying-substrate loss with documented harm + structural defense.",
    references = [
        "ADR-026",
        "ADR-026#D2",
        "https://www.science.org/doi/10.1126/science.aay6485"
    ]
)]
pub struct ForcePushErasingHistory;

// ============================================================================
// 5. SquashMergeLosingIntermediateState
// ============================================================================

/// A squash-merge collapsed N intermediate commits into one, erasing the
/// step-by-step reasoning trail.
///
/// **Failure class**: linearized history reads cleanly but conceals the
/// affinity-maturation path. Future developers cannot bisect across the
/// internal steps; cannot see which intermediate refactor enabled the
/// final shape; cannot identify which commit introduced a regression.
///
/// **Biology cognate**: Affinity-maturation history loss. B-cell affinity
/// maturation is a step-by-step optimization over generations of somatic
/// hypermutation. Losing the per-generation record erases the path through
/// which the high-affinity antibody emerged. Squash-merges have the same
/// shape.
///
/// **Defense**: `cargo antigen vcs check-commit` flags squash-merges that
/// collapse > N substantive commits (configurable threshold) without a
/// `Squash-Preserves: <full-PR-link-or-archived-branch>` trailer recording
/// where the intermediate states ARE preserved (PR archive, separate
/// long-form-history branch, etc.).
#[antigen(
    name = "squash-merge-losing-intermediate-state",
    category = AntigenCategory::SubstrateAlignment,
    family = "vcs-information-loss",
    summary = "Squash-merge collapsing substantive intermediate commits without a Squash-Preserves trailer. Affinity-maturation history loss: per-generation record is the optimization path.",
    references = ["ADR-026"]
)]
pub struct SquashMergeLosingIntermediateState;

// ============================================================================
// 6. CherryPickLosingOriginalContext
// ============================================================================

/// A cherry-pick brought a commit to a new branch without preserving a
/// pointer back to the original commit + context.
///
/// **Failure class**: a cherry-picked commit reads correctly in its new
/// location but the surrounding-state context (what else was in flight,
/// what review thread, what test environment) is lost. The same fix
/// applied in two different contexts can mean two different things.
///
/// **Biology cognate**: Class-switching context loss. Antibody class
/// switching (`IgM` → `IgG` → `IgA`) preserves the antigen-binding region but
/// changes effector function. The same Fab region in a different Fc
/// context targets different tissues. Cherry-picks without context
/// preservation have the same shape — same code, different operational
/// context, different behavior.
///
/// **Defense**: `cargo antigen vcs check-commit` enforces the
/// `Cherry-Picked-From: <original-sha>` trailer that git's `cherry-pick -x`
/// flag emits, and additionally requires a `Cherry-Pick-Context:` trailer
/// when the original commit was part of a multi-commit PR or feature.
#[antigen(
    name = "cherry-pick-losing-original-context",
    category = AntigenCategory::SubstrateAlignment,
    family = "vcs-information-loss",
    summary = "Cherry-pick without Cherry-Picked-From trailer (or with no Cherry-Pick-Context for multi-commit-PR commits). Class-switching context loss: same Fab, different Fc, different behavior.",
    references = ["ADR-026"]
)]
pub struct CherryPickLosingOriginalContext;

// ============================================================================
// 7. RebaseRewritingHistoryWithoutLog
// ============================================================================

/// A rebase rewrote commit history (reordering, fixup-squashing, edits)
/// without preserving a log of what was rewritten.
///
/// **Failure class**: post-rebase the original commit hashes are gone;
/// any external reference to those hashes (review comments, deploy logs,
/// issue trackers) becomes a dangling pointer to a no-longer-existing
/// commit.
///
/// **Biology cognate**: V(D)J recombination without per-cell record. The
/// immune system's V(D)J locus rearrangement is irreversible per-cell;
/// however the per-cell record IS preserved (the rearranged DNA itself).
/// Rebases without a rewrite log are the failure mode where the per-cell
/// record is destroyed — the rearrangement happens but the documentation
/// of WHAT was rearranged is lost.
///
/// **Defense**: pre-rebase the developer runs `cargo antigen vcs rebase-
/// prepare <upstream>` which archives the pre-rebase commit hashes + their
/// messages to `.attest/vcs/rebase-archive/<branch>@<timestamp>.json`. The
/// rebase commit itself carries a `Rebase-Archive:` trailer pointing to
/// the archive.
#[antigen(
    name = "rebase-rewriting-history-without-log",
    category = AntigenCategory::SubstrateAlignment,
    family = "vcs-information-loss",
    summary = "Rebase erasing original commit hashes without a Rebase-Archive trailer pointing to the pre-rebase archive sidecar. V(D)J recombination without per-cell record.",
    references = ["ADR-026"]
)]
pub struct RebaseRewritingHistoryWithoutLog;

// ============================================================================
// 8. UnpushedBranchWithSubstantiveWork
// ============================================================================

/// A local branch contains substantive work that has never been pushed to
/// any remote, and the branch has been stale for >N days.
///
/// **Failure class**: developer-machine-only work is a single-point-of-
/// failure substrate. Disk failure, accidental `git checkout -` followed
/// by `git branch -D`, OS reinstall — any of these destroys work that no
/// one else knows exists.
///
/// **Detection**: `cargo antigen vcs scan` walks `git branch --no-merged`
/// + checks each branch's last-push timestamp; flags branches with > N
/// substantive commits not pushed in > M days.
///
/// **Biology cognate**: not yet assigned; follow-up.
///
/// **Defense**: `cargo antigen vcs scan --check-unpushed` lists candidate
/// branches; CI / pre-commit hooks can nag the developer to push or
/// archive.
#[antigen(
    name = "unpushed-branch-with-substantive-work",
    category = AntigenCategory::SubstrateAlignment,
    family = "vcs-information-loss",
    summary = "Local branch with >N substantive commits not pushed for >M days. Single-point-of-failure substrate.",
    references = ["ADR-026"]
)]
pub struct UnpushedBranchWithSubstantiveWork;

// ============================================================================
// 9. StashedWorkAbandoned
// ============================================================================

/// A `git stash` entry has been sitting for >N days without being applied
/// or dropped — likely abandoned work, but not declaratively so.
///
/// **Failure class**: `git stash list` shows ambiguous state. A 90-day-old
/// stash MIGHT be cherished work-in-progress; MIGHT be forgotten
/// experiment; without explicit attestation the substrate cannot
/// distinguish.
///
/// **Biology cognate**: Anergy / primed-without-activation. Anergic
/// lymphocytes are primed to respond but never activated; they linger in
/// the system as a load without producing antibody. Stashed work that
/// neither lands nor gets dropped has the same shape — work-in-progress
/// without commitment direction.
///
/// **Defense**: `cargo antigen vcs scan --check-stashes` lists stash
/// entries older than the configured threshold; developer attests via
/// `cargo antigen vcs stash-attest <stash-ref> --intent <apply|drop|park
/// --rationale "...">`. Park-with-rationale is the loud equivalent of
/// `#[anergy]` for stashes.
#[antigen(
    name = "stashed-work-abandoned",
    category = AntigenCategory::SubstrateAlignment,
    family = "vcs-information-loss",
    summary = "git stash entry older than threshold without an attestation of intent (apply/drop/park). Anergy/primed-without-activation: shape of work-in-progress without commitment direction.",
    references = ["ADR-026", "ADR-023"]
)]
pub struct StashedWorkAbandoned;

// ============================================================================
// 10. MergeConflictResolutionWithoutAttestation
// ============================================================================

/// A merge commit resolved conflicts without any record of HOW the
/// conflicts were resolved or WHY the chosen resolution was correct.
///
/// **Failure class**: merge-commit semantics carry the choice (which side
/// won, which combination) but not the reasoning. A future bisect lands
/// on the merge commit; the developer cannot reconstruct why this
/// resolution was chosen vs. the alternatives.
///
/// **Biology cognate**: not yet assigned; follow-up.
///
/// **Defense**: `cargo antigen vcs check-commit` on merge commits requires
/// either (a) zero conflicts resolved (trivial merge); or (b) a
/// `Conflict-Resolution: <attestation-sidecar-sha-or-PR-link>` trailer
/// recording the resolution rationale. The attestation can live in
/// `.attest/vcs/merge-resolution/<merge-sha>.json` or in the PR thread.
#[antigen(
    name = "merge-conflict-resolution-without-attestation",
    category = AntigenCategory::SubstrateAlignment,
    family = "vcs-information-loss",
    summary = "Merge commit with non-trivial conflict resolution but no Conflict-Resolution trailer recording the rationale.",
    references = ["ADR-026"]
)]
pub struct MergeConflictResolutionWithoutAttestation;

// ============================================================================
// 11. AmendedCommitWithoutOldHashPreservation
// ============================================================================

/// A `git commit --amend` rewrote a published commit (or a commit
/// referenced externally) without preserving the pre-amend hash.
///
/// **Failure class**: same shape as `RebaseRewritingHistoryWithoutLog` but
/// at single-commit granularity. The amended commit replaces the original
/// in-place; any external reference to the original hash (CI logs, deploy
/// pipelines, downstream cherry-picks) becomes a dangling pointer.
///
/// **Detection**: pre-amend hook captures the pre-amend hash + commit
/// message; the amended commit carries an `Amended-From: <old-sha>`
/// trailer.
///
/// **Biology cognate**: not yet assigned; follow-up.
///
/// **Defense**: `cargo antigen vcs install-hooks` installs the pre-commit
/// hook that, when the commit is an amend AND the original commit has
/// been pushed, requires the `Amended-From:` trailer.
#[antigen(
    name = "amended-commit-without-old-hash-preservation",
    category = AntigenCategory::SubstrateAlignment,
    family = "vcs-information-loss",
    summary = "git commit --amend on a published or externally-referenced commit without an Amended-From trailer carrying the pre-amend hash.",
    references = ["ADR-026"]
)]
pub struct AmendedCommitWithoutOldHashPreservation;
