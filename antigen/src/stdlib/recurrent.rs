//! # Recurrent-Emergence Family — stdlib antigens (ADR-024 §Family 2)
//!
//! Canonical failure-classes that re-emerge across project lifetimes. Per
//! ADR-024 the recurrent family is the present-looking arc: patterns that
//! keep coming back, where the "we fixed it" memory evaporates between
//! occurrences. These stdlib `#[antigen]` declarations name the recurring
//! classes so adopters can `#[recurrence_anchor]` / `#[itch]` against them.
//!
//! ## Antigen-category (ADR-028)
//!
//! All recurrent stdlib antigens are `SubstrateAlignment`: the
//! representation of "this keeps happening" (or "we handled it") diverges
//! from the actual cross-lifetime state until the recurrence is anchored
//! and acted on. The witness checks substrate (Cargo.toml `rust-version`,
//! `.gitignore` patterns, lockfile churn), not runtime behaviour.
//!
//! ## Biology grounding (dual-axis honesty per ADR-024)
//!
//! The recurrent family draws on cognitive-organizational grounding (how
//! teams notice recurring patterns) for the itch/saturate/crystallize/strand
//! primitives + immunology-proper for `#[chronic]` (sustained low-level
//! signal) + clinical-medicine for `#[recurrence_anchor]` (formal diagnosis
//! after recurrent symptoms). The stdlib antigens below are the
//! failure-classes those primitives anchor to; they are NOT themselves
//! biology-grounded beyond the family framing.
//!
//! ## How these antigens are evaluated
//!
//! Like supply-chain + vcs-info-loss, recurrent antigens are not matched by
//! AST-walking. Adopters mark sites with the recurrent macros (`#[itch]`,
//! `#[recurrence_anchor]`, etc.); `cargo antigen scan` surfaces the markers
//! and `cargo antigen audit` runs `audit_recurrent` to emit the recurrent
//! hints. The `fingerprint` uses the uniform `doc_contains("ADR-024")` form.

use crate::antigen;

// ============================================================================
// 1. MsrvCreepAfterMajorVersionBump
// ============================================================================

/// MSRV (minimum-supported-rust-version) silently creeps upward after a
/// transitive dependency major-version bump; the declared `rust-version`
/// in `Cargo.toml` diverges from the actually-required floor.
///
/// **The cross-lifetime recurrence**: this happens again every few releases,
/// in every Rust workspace, and the "we pinned the floor" memory evaporates
/// between occurrences. Anchor it once via `#[recurrence_anchor]` so the
/// next occurrence is recognized, not re-discovered.
///
/// **Category**: `SubstrateAlignment` — `Cargo.toml`'s `rust-version` says
/// X but the actually-required floor (per transitive deps' MSRV) is higher.
#[antigen(
    name = "msrv-creep-after-major-version-bump",
    category = AntigenCategory::SubstrateAlignment,
    fingerprint = r#"doc_contains("ADR-024")"#,
    family = "recurrent-emergence",
    summary = "MSRV silently creeps upward after a transitive dep major bump; declared rust-version diverges from the actually-required floor. Recurs across project lifetimes.",
    references = ["ADR-024", "ADR-024#Family-2"]
)]
pub struct MsrvCreepAfterMajorVersionBump;

// ============================================================================
// 2. GitignorePatternDriftOverReleases
// ============================================================================

/// `.gitignore` patterns drift over N releases.
///
/// Build-artifact / editor / OS-junk patterns that were once present get
/// dropped during a rewrite, or new artifact kinds appear that nobody adds
/// an ignore for — so junk starts getting committed again, release after
/// release.
///
/// **The cross-lifetime recurrence**: the same classes of files
/// (`target/`, `*.log`, `.DS_Store`, editor swap files) keep slipping back
/// into commits because the `.gitignore` discipline isn't anchored to a
/// recurrence memory.
///
/// **Category**: `SubstrateAlignment` — the committed file set diverges
/// from the intended-tracked set; `.gitignore` representation drifted.
#[antigen(
    name = "gitignore-pattern-drift-over-releases",
    category = AntigenCategory::SubstrateAlignment,
    fingerprint = r#"doc_contains("ADR-024")"#,
    family = "recurrent-emergence",
    summary = "Build-artifact / editor / OS-junk .gitignore patterns drift out over N releases; junk re-enters commits. Recurs across project lifetimes.",
    references = ["ADR-024"]
)]
pub struct GitignorePatternDriftOverReleases;

// ============================================================================
// 3. LockfileChurnFromUnpinnedTooling
// ============================================================================

/// `Cargo.lock` (or equivalent) churns on every CI run.
///
/// Some tooling dependency is unpinned, producing noisy diffs that mask
/// real dependency changes — release after release, the same churn class
/// re-emerges.
///
/// **The cross-lifetime recurrence**: each new contributor's environment
/// resolves slightly different patch versions; the lockfile diff becomes
/// noise; real supply-chain-relevant changes hide in the churn. Distinct
/// from the supply-chain `UnpinnedDependency` antigen (ADR-025) — that's
/// the point-in-time pinning state; this is the *recurring churn pattern*
/// that keeps re-surfacing.
///
/// **Category**: `SubstrateAlignment` — the lockfile's recorded resolution
/// diverges from a stable intended resolution on each run.
#[antigen(
    name = "lockfile-churn-from-unpinned-tooling",
    category = AntigenCategory::SubstrateAlignment,
    fingerprint = r#"doc_contains("ADR-024")"#,
    family = "recurrent-emergence",
    summary = "Lockfile churns every CI run from unpinned tooling deps; noisy diffs mask real changes. Recurring churn pattern (distinct from ADR-025 point-in-time pinning).",
    references = ["ADR-024", "ADR-025"]
)]
pub struct LockfileChurnFromUnpinnedTooling;
