//! # Release-Engineering Failure-Class Family — stdlib antigens
//!
//! The failure-classes that bite at the **release boundary** — the moment a
//! workspace stops being source-on-disk and becomes a tagged, published artifact
//! the world depends on. These are not source-walking code defects; they are
//! **substrate-alignment** classes (a representation diverging from actual state)
//! whose enforcement lives in the `cargo antigen pre-tag` releng gate, not in
//! AST-walking fingerprint matching.
//!
//! Biology cognate: **the irreversible developmental commitment**. Most cellular
//! decisions are reversible — a cell can re-differentiate, a signal can be
//! re-sent. But some steps (apoptosis, the metaphase-to-anaphase transition) are
//! one-way: once crossed, recovery requires external, costly intervention. The
//! git tag is that one-way step in a release; this family names the classes that
//! turn a routine release into a recovery operation.
//!
//! ## Antigen-category (ADR-028)
//!
//! All four members are `SubstrateAlignment`: each fires when a *representation*
//! (the tag set, a crate's metadata, the declared MSRV, a fingerprint registry)
//! diverges from the actual state the release commits to. The witness is
//! substrate — the git tag namespace, `Cargo.toml`, the lockfile, the
//! fingerprint catalog — not runtime behaviour.
//!
//! ## How these antigens are evaluated
//!
//! Like the recurrent / supply-chain / vcs-info-loss families, these are
//! **declaration-only** stdlib antigens: they NAME the failure-class for immune
//! memory, but they are not matched by source-walking AST traversal. The active
//! discipline lives in:
//!
//! - [`NonIdempotentReleaseStep`] → `cargo antigen pre-tag` (the runnable gate —
//!   the version-coherence + no-tag-exists checks ARE its executable defense).
//! - [`CratesIoPublishBlockerMissingMetadata`] → `cargo antigen pre-tag`'s
//!   publish-metadata + readme-presence checks.
//! - [`MsrvAccidentallyRaisedByTransitiveDep`] → CI's `cargo +1.95 check`
//!   (the existing guard; this names the class CI's MSRV job defends).
//! - [`StructuralFingerprintCollision`] → the `spares_namesake_contract.rs` +
//!   fingerprint round-trip tests (the existing guard).
//!
//! Because they carry no source-walking fingerprint, they are **not** bundled
//! into the compile-in `STDLIB_CATALOG` (build.rs reads only the named
//! source-walking flagship modules) — exactly as the recurrent family is
//! excluded. Their shared placeholder fingerprint
//! (`doc_contains("release-engineering")`) is a non-firing marker, never a match
//! predicate; two declaration-only members sharing it is NOT a
//! [`StructuralFingerprintCollision`] (that class is about two **named, matched**
//! classes colliding — see that member's doc).

use crate::antigen;

// ============================================================================
// 1. NonIdempotentReleaseStep
//
// The one genuinely-unguarded class before v0.6.1 — its `cargo antigen pre-tag`
// checks ARE its defense.
// ============================================================================

/// A release step that is **not safe to re-run** — most prominently the git tag,
/// whose existence makes a second attempt a recovery operation, not a retry.
///
/// **Where in the wild:** the canonical release foot-cannon. A release is a
/// sequence of steps; almost all are idempotent (re-bump, re-write the
/// CHANGELOG, re-publish a yanked crate). The **git tag** is not: once `v<version>`
/// exists — especially once it is pushed and a consumer has fetched it — undoing
/// it is a manual, error-prone intervention (force-delete the remote tag, hope no
/// one pinned to it). The pin-drift sibling is the same shape: a crate left
/// pinned to the OLD version while the workspace bumped resolves a stale internal
/// dep on a fresh publish, and the publish cannot be cleanly un-done.
///
/// **The recovery asymmetry:** CI catches a bad release — but it runs on *push*,
/// AFTER the tag is already created. By the time CI is red, the non-recoverable
/// step has already happened. This class exists because the guard that runs at
/// the right time (before the tag) did not exist until `cargo antigen pre-tag`.
///
/// **Tier:** **named** — the tag-exists / pin-drift tells are precise and the
/// non-recoverability is a hard property of git's tag namespace, not a heuristic.
///
/// **Witness:** `cargo antigen pre-tag` PASSES — the `v<version>` tag exists
/// neither locally nor on `origin`, and the workspace version == every internal
/// `=`-pin == the version-to-tag (the checks ARE the immune memory made
/// executable).
///
/// **Category:** `SubstrateAlignment` — the tag namespace (or the pinned-version
/// representation) is asserted clean/coherent when it is not.
#[antigen(
    name = "non-idempotent-release-step",
    category = AntigenCategory::SubstrateAlignment,
    fingerprint = r#"doc_contains("release-engineering")"#,
    family = "release-engineering",
    summary = "A release step not safe to re-run — the git tag (non-recoverable once pushed) + the pin-drift sibling (a crate left on the old version resolves a stale internal dep). CI runs AFTER the tag is cut; `cargo antigen pre-tag` is the gate that runs in time.",
    references = ["ADR-024", "cargo antigen pre-tag"]
)]
pub struct NonIdempotentReleaseStep;

// ============================================================================
// 2. CratesIoPublishBlockerMissingMetadata
// ============================================================================

/// A crate missing `description` / `license` / `repository` / `readme` — any one
/// blocks `cargo publish`.
///
/// **Where in the wild:** crates.io enforces a minimum metadata set at publish
/// time. A crate (or a workspace member that forgot to inherit
/// `license.workspace = true`) with a missing key is rejected by the registry —
/// and a `readme = "README.md"` pointing at a file that does not exist renders a
/// "coming soon" placeholder on the crate's crates.io page even when the publish
/// itself succeeds. Both are caught only at the publish step, which (like the
/// tag) comes late in the release.
///
/// **Tell:** a workspace member whose `Cargo.toml` resolves fewer than the four
/// required keys — counting workspace inheritance (`<key>.workspace = true`) as
/// resolved. README presence is the on-disk half: the key can resolve while the
/// pointed-at file is absent.
///
/// **Tier:** **named** — the required-key set is a fixed, documented crates.io
/// contract; the tell is exact, not heuristic.
///
/// **Witness:** `cargo antigen pre-tag`'s publish-metadata + readme-presence
/// checks PASS for every member crate.
///
/// **Category:** `SubstrateAlignment` — the crate's manifest claims publish-ready
/// when a required field is unresolved.
#[antigen(
    name = "crates-io-publish-blocker-missing-metadata",
    category = AntigenCategory::SubstrateAlignment,
    fingerprint = r#"doc_contains("release-engineering")"#,
    family = "release-engineering",
    summary = "Missing description / license / repository / readme blocks `cargo publish`; a readme pointing at an absent file renders a crates.io \"coming soon\" page. Caught only at publish-time (late); pre-tag catches it before the tag.",
    references = ["ADR-025", "cargo antigen pre-tag"]
)]
pub struct CratesIoPublishBlockerMissingMetadata;

// ============================================================================
// 3. MsrvAccidentallyRaisedByTransitiveDep
//
// Sibling of the recurrent family's MsrvCreepAfterMajorVersionBump (ADR-024):
// that names the recurring CHURN pattern; this names the point-in-time release
// hazard CI's MSRV job guards.
// ============================================================================

/// A transitive dependency silently raises the effective MSRV past the declared
/// `rust-version` — the release builds on the maintainer's toolchain but breaks
/// for users on the promised floor.
///
/// **Where in the wild:** the declared `rust-version = "1.95"` is a promise: the
/// crate compiles on Rust 1.95. A dependency bump (often transitive, often a
/// patch release) can raise *its* MSRV above 1.95 — and because the maintainer is
/// usually on a newer toolchain, nothing locally surfaces it. The release ships,
/// and a user on exactly 1.95 gets a compile error the maintainer never saw.
///
/// **Distinct from the recurrent
/// [`MsrvCreepAfterMajorVersionBump`](crate::stdlib::recurrent::MsrvCreepAfterMajorVersionBump):**
/// that names the *cross-lifetime recurring churn* (it keeps happening, the
/// memory evaporates between occurrences). This names the *point-in-time release
/// hazard* a single release must clear — the thing CI's `cargo +1.95 check`
/// catches on this release, not the pattern across releases.
///
/// **Tier:** **named** — "compiles on the declared floor" is a binary,
/// CI-checkable property; the tell is exact.
///
/// **Witness:** `cargo +<declared rust-version> check --workspace` succeeds (the
/// existing CI MSRV job) — the effective floor still matches the declared one.
///
/// **Category:** `SubstrateAlignment` — `Cargo.toml`'s `rust-version` claims a
/// floor the dependency graph no longer honors.
#[antigen(
    name = "msrv-accidentally-raised-by-transitive-dep",
    category = AntigenCategory::SubstrateAlignment,
    fingerprint = r#"doc_contains("release-engineering")"#,
    family = "release-engineering",
    summary = "A transitive dep silently raises the effective MSRV past the declared rust-version; builds for the maintainer, breaks for users on the promised floor. Point-in-time release hazard (distinct from the recurrent MSRV-creep churn); CI's `cargo +1.95 check` is its guard.",
    references = ["ADR-024", "ADR-025"]
)]
pub struct MsrvAccidentallyRaisedByTransitiveDep;

// ============================================================================
// 4. StructuralFingerprintCollision
// ============================================================================

/// Two **distinct, named, matched** failure-classes share one structural
/// fingerprint.
///
/// A scan match then cannot tell which class it found, and the two classes'
/// provenance/tier/witness contracts silently merge.
///
/// **Where in the wild:** antigen's own founding hazard. Each named source-walking
/// flagship carries a structural fingerprint that is supposed to discriminate
/// *its* failure-class. If two distinct named classes resolve to the same
/// fingerprint string, a match is ambiguous — the render attributes it to one
/// class, but it could equally be the other, and the weaker class's tier can
/// launder into the stronger one's verdict. This is the "passing test, wrong
/// answer" antigen exists to catch, turned on antigen itself.
///
/// **Scope note (NOT every shared string):** declaration-only members
/// (recurrent, release-engineering, supply-chain, vcs-info-loss) deliberately
/// share a non-firing PLACEHOLDER fingerprint (`doc_contains("ADR-024")`,
/// `doc_contains("release-engineering")`). That is not a collision — those
/// fingerprints never *match*, so there is nothing to disambiguate. This class
/// is specifically about two **matched** classes (the bundled source-walking
/// flagships) colliding in the shape that actually fires.
///
/// **Tell:** two named flagships in the bundled `STDLIB_CATALOG` whose parsed
/// `Fingerprint` are structurally equal.
///
/// **Tier:** **named** — fingerprint equality is decidable and the collision's
/// consequence (ambiguous attribution) is structural, not heuristic.
///
/// **Witness:** the `spares_namesake_contract.rs` guards + the fingerprint
/// round-trip / catalog-distinctness tests hold — each named member's fingerprint
/// binds its own class and spares its siblings'.
///
/// **Category:** `SubstrateAlignment` — a matched fingerprint claims to identify
/// one class while structurally also identifying another.
#[antigen(
    name = "structural-fingerprint-collision",
    category = AntigenCategory::SubstrateAlignment,
    fingerprint = r#"doc_contains("release-engineering")"#,
    family = "release-engineering",
    summary = "Two distinct NAMED, matched failure-classes share one structural fingerprint — ambiguous attribution + tier laundering. antigen's own founding hazard turned inward. (Shared PLACEHOLDER fingerprints on declaration-only members are NOT collisions — they never match.) Guarded by spares_namesake_contract.rs.",
    references = ["ADR-039", "spares_namesake_contract.rs"]
)]
pub struct StructuralFingerprintCollision;
