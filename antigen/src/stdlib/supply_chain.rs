//! # Supply-Chain Defense Family (ADR-025)
//!
//! Eleven stdlib antigens for the 2026+ dependency-boundary threat landscape.
//! Per ADR-025 these antigens make supply-chain failure-classes first-class
//! structural memory rather than tribal knowledge that drifts.
//!
//! ## Biology cognate (NON-NEGOTIABLE per naturalist reframe)
//!
//! **Distributed-Boundary Innate-Immunity family** — a multi-cell-type
//! integrated system. Per-primitive cognates:
//!
//! | Antigen | Biological cognate |
//! |---|---|
//! | `UnpinnedDependency` | PRR specificity discipline |
//! | `UnpinnedTransitiveDependency` | Chain-of-recognition discipline |
//! | `UnattestedDependencyInclusion` | Naive antigen exposure (no MHC-I) |
//! | `DependencyUpgradeWithoutDiffReview` | Affinity-maturation re-check |
//! | `AutoDependencyChainWithoutPinning` | Receptor-promiscuity vulnerability |
//! | `MaintainerChangeWithoutReattestation` | Transplant-immunology re-attestation |
//! | `SuddenDependencyExpansion` | Trojan-horse + MHC-I internal-antigen presentation |
//! | `UnsandboxedBuildScript` | Macrophage phagosome containment |
//! | `UnsandboxedProcMacro` | Macrophage phagosome containment (higher risk; runs in-rustc) |
//! | `PostInstallScriptInDependency` | Install-time toxin exposure |
//! | `ContentHashMismatch` | Antigenic-identity verification |
//!
//! ## The non-negotiable: `ContentHashMismatch`
//!
//! The 2025 chalk/debug/eslint-config attack pattern was **content replacement
//! at fixed version** — `Cargo.lock` pins VERSION but NOT CONTENT-HASH. The
//! `ContentHashMismatch` antigen is the structural memory of that attack
//! class. It requires proactive first-attestation via
//! `cargo antigen verify content-hash record <crate@version>` to activate.
//!
//! ## How these antigens are evaluated
//!
//! Unlike source-pattern antigens (which fire via `cargo antigen scan`
//! AST-walking), supply-chain antigens fire via **substrate-witness
//! evaluation** against `Cargo.toml`, `Cargo.lock`, the content-hash registry
//! at `.attest/supply-chain/content-hash/`, and (in later phases) the
//! crates.io maintainer-history API. Drive evaluation via the
//! `cargo antigen verify` CLI subfamily.
//!
//! ## Fingerprint discipline
//!
//! The `fingerprint` field on each declaration uses a uniform
//! `doc_contains("ADR-025")` form: these antigens are not matched by
//! source-walking AST patterns, so the fingerprint is descriptive of the
//! declaration's docs rather than a structural matcher over presentations.
//! The active matching surface is the `verify` subcommand family.

use crate::antigen;

// ============================================================================
// 1. UnpinnedDependency
// ============================================================================

/// A `[dependencies]` entry in `Cargo.toml` that does not pin to an
/// exact version (`=X.Y.Z`).
///
/// **Biology cognate**: PRR (Pattern Recognition Receptor) specificity
/// discipline. Innate-immunity receptors that bind too loosely produce
/// noise; receptors that bind exactly trigger correctly. Cargo's caret
/// (`^X.Y.Z`) and tilde (`~X.Y.Z`) requirements are loose binding;
/// exact-pin `=X.Y.Z` is the high-specificity equivalent.
///
/// **Defense**: `cargo antigen verify deps` flags every non-exact dep spec.
/// `cargo antigen verify dep-pin` pins all unpinned deps in one sweep.
///
/// **Evaluate via**: substrate-witness leaf `dep_pinned(crate?)` against
/// the workspace `Cargo.toml`.
#[antigen(
    name = "unpinned-dependency",
    category = AntigenCategory::SubstrateAlignment,
    family = "supply-chain-distributed-boundary-innate-immunity",
    summary = "Cargo.toml dep without exact-pin `=` version specifier. PRR specificity discipline: receptors that bind too loosely produce noise.",
    references = [
        "ADR-025",
        "https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html"
    ]
)]
pub struct UnpinnedDependency;

// ============================================================================
// 2. UnpinnedTransitiveDependency (NARROW per ADR-025 B9-R)
// ============================================================================

/// A direct dependency that specifies `*` or `?` ranges for ITS OWN
/// transitive dependencies, propagating unpinning into the lockfile-resolved
/// tree.
///
/// **CRITICAL NARROW DEFINITION (ADR-025 B9-R)**:
///
/// - **CORRECT**: a direct dep whose own `[dependencies]` table uses
///   wildcard (`*`) or undetermined (`?`) version specifiers for its
///   transitive deps. The unpinning vulnerability propagates through the
///   trust boundary that the direct dep represents.
/// - **INCORRECT**: "any transitive dep with non-exact pins" — this broader
///   definition produces ~100% false positives, because most transitive deps
///   are non-exact-pinned in practice and `Cargo.lock` resolution makes the
///   actual installed version stable. The broad definition was explicitly
///   REJECTED in ADR-025 B9-R.
///
/// **Biology cognate**: chain-of-recognition discipline. A receptor that
/// trusts a downstream receptor's specificity is only as good as the
/// downstream's discipline. Direct deps with wildcard transitive specs are
/// the structural equivalent.
///
/// **Defense**: `cargo antigen verify deps --check-transitive` (v0.3+) reads
/// the manifest of each direct dep and flags only those with wildcard/?
/// transitive specs.
///
/// **Limitation**: requires reading each direct dep's `Cargo.toml` from
/// `.cargo/registry`, which means the dep must be downloaded first. v0.2
/// emits an audit hint only when the manifests are accessible.
#[antigen(
    name = "unpinned-transitive-dependency",
    category = AntigenCategory::SubstrateAlignment,
    family = "supply-chain-distributed-boundary-innate-immunity",
    summary = "NARROW: direct dep that uses `*` or `?` for ITS OWN transitive dependencies. The broad 'any transitive non-exact-pin' definition was REJECTED in ADR-025 B9-R (100% false-positive rate).",
    references = ["ADR-025", "ADR-025#B9-R"]
)]
pub struct UnpinnedTransitiveDependency;

// ============================================================================
// 3. UnattestedDependencyInclusion
// ============================================================================

/// A new dependency was added to `Cargo.toml` without a corresponding
/// team-attestation in the commit history that introduced it.
///
/// **The AI-pair failure pattern**: AI assistants emit `cargo add <name>`
/// frequently and confidently. Without an attestation discipline, those
/// additions land without team review. Each silently-added dep extends the
/// trust boundary.
///
/// **Biology cognate**: naive antigen exposure with no MHC-I presentation.
/// The cell encounters a new molecule without the cellular machinery that
/// would surface it for adaptive review.
///
/// **Defense**: every new dep-add commit must include a
/// `.attest/supply-chain/dep-attest/<crate>@<version>.json` sidecar created
/// via `cargo antigen verify dep-attest <crate@version> --reviewable-artifact
/// <PATH>` (the `--reviewable-artifact` arg is REQUIRED — see
/// `dep-attest-without-reviewable-artifact` hint).
///
/// **Evaluate via**: substrate-witness leaf `dep_attested(crate, version)`.
#[antigen(
    name = "unattested-dependency-inclusion",
    category = AntigenCategory::SubstrateAlignment,
    family = "supply-chain-distributed-boundary-innate-immunity",
    summary = "New dependency added without team-attestation. AI-pair-generated `cargo add` is the canonical failure pattern.",
    references = ["ADR-025"]
)]
pub struct UnattestedDependencyInclusion;

// ============================================================================
// 4. DependencyUpgradeWithoutDiffReview
// ============================================================================

/// A dependency version was bumped without a diff-reviewed attestation
/// (`ReviewScope = Diff` or stricter).
///
/// **Account-compromise threat model**: an attacker who compromises a
/// maintainer account can publish a new version that differs structurally
/// from the prior one. Reviewing only the version-bump line in
/// `Cargo.toml`/`Cargo.lock` is necessary but not sufficient; the upgrade
/// must be accompanied by a diff-attestation that the new code's behavior
/// is as expected.
///
/// **Biology cognate**: affinity-maturation re-check — the immune response
/// must re-verify identity when an antigen mutates.
///
/// **Defense**: `cargo antigen verify deps` flags upgrades whose sidecar
/// does not have `review_scope ∈ {Diff, Full}`.
#[antigen(
    name = "dependency-upgrade-without-diff-review",
    category = AntigenCategory::SubstrateAlignment,
    family = "supply-chain-distributed-boundary-innate-immunity",
    summary = "Version bump without diff-reviewed attestation. Account-compromise control: maintainer-account takeovers publish drift inside fixed major versions.",
    references = ["ADR-025"]
)]
pub struct DependencyUpgradeWithoutDiffReview;

// ============================================================================
// 5. AutoDependencyChainWithoutPinning
// ============================================================================

/// A `*` or `?` version-range specifier exists anywhere in the dependency
/// tree, allowing automatic chain-of-updates without human gate.
///
/// **Distinction vs `UnpinnedDependency`**: `UnpinnedDependency` covers the
/// caret/tilde case where a small range is allowed (`^1.2.3` allows `1.x`).
/// This antigen covers the **unbounded** case (`*`, `?`) where any future
/// version is automatically eligible.
///
/// **Biology cognate**: receptor-promiscuity vulnerability — a receptor
/// with degenerate specificity binds anything, including future antigens
/// the immune system has never encountered.
///
/// **Defense**: `cargo antigen verify deps` rejects `*`/`?` specifiers
/// outright. Pin via `cargo antigen verify dep-pin`.
#[antigen(
    name = "auto-dependency-chain-without-pinning",
    category = AntigenCategory::SubstrateAlignment,
    family = "supply-chain-distributed-boundary-innate-immunity",
    summary = "`*` or `?` version-range specifier anywhere in dep tree allows automatic chain-of-updates with no human gate.",
    references = ["ADR-025"]
)]
pub struct AutoDependencyChainWithoutPinning;

// ============================================================================
// 6. MaintainerChangeWithoutReattestation
// ============================================================================

/// A crate's maintainer/ownership set changed (or expanded) since the last
/// team-attestation, and no re-attestation has been recorded.
///
/// **CRITICAL SEQUENCING (LOAD-BEARING)**: the CI verification
/// `cargo antigen verify maintainer-changes` **MUST run BEFORE
/// `cargo update`**. Running afterwards means the new maintainer's code
/// has already been resolved into the lockfile — the gate has passed and
/// the trust boundary has already been extended. Document this sequencing
/// in CI scripts.
///
/// **Biology cognate**: transplant-immunology re-attestation. A tissue that
/// was self yesterday may carry a new MHC profile today; re-attestation
/// reasserts identity.
///
/// **Defense**: `cargo antigen verify maintainer-changes` queries
/// crates.io for the current owner set of each pinned dep, compares
/// against the sidecar-recorded set, and refuses to allow `cargo update`
/// if a divergence is detected without a fresh attestation.
///
/// **Limitation**: requires a live crates.io API call. Failed queries
/// emit the `crates-io-metadata-query-failed` hint; CI should treat that
/// as a soft-fail rather than a green light.
#[antigen(
    name = "maintainer-change-without-reattestation",
    category = AntigenCategory::SubstrateAlignment,
    family = "supply-chain-distributed-boundary-innate-immunity",
    summary = "Crate ownership change without team-reattestation. CI sequencing constraint: `verify maintainer-changes` MUST run BEFORE `cargo update`.",
    references = ["ADR-025"]
)]
pub struct MaintainerChangeWithoutReattestation;

// ============================================================================
// 7. SuddenDependencyExpansion
// ============================================================================

/// A dep version bump with significant LOC or transitive-dep expansion.
///
/// The new version's source/dep tree has grown notably compared to the
/// prior version, suggesting the bump may carry hidden new transitive
/// deps or substantial new code paths beyond what the version-bump
/// surface implies.
///
/// **Complement to `DependencyUpgradeWithoutDiffReview`**: that antigen
/// catches the *no review attempted* case; this one catches the *review
/// attempted but the new surface is so large that the review is
/// implausibly thorough*.
///
/// **Biology cognate**: Trojan-horse + MHC-I internal antigen presentation.
/// A familiar carrier delivers a larger payload than its surface declared.
///
/// **Defense**: `cargo antigen verify deps` compares LOC + dep count
/// between adjacent recorded versions; flags expansions above a workspace-
/// configurable threshold.
#[antigen(
    name = "sudden-dependency-expansion",
    category = AntigenCategory::SubstrateAlignment,
    family = "supply-chain-distributed-boundary-innate-immunity",
    summary = "Version bump with large LOC/dep-count delta. Complements `DependencyUpgradeWithoutDiffReview` for the implausibly-thorough-review case.",
    references = ["ADR-025"]
)]
pub struct SuddenDependencyExpansion;

// ============================================================================
// 8. UnsandboxedBuildScript
// ============================================================================

/// A `build.rs` from an external dependency executes at compile time
/// without sandbox containment.
///
/// **Risk**: `build.rs` runs as the developer/CI user, with file-system,
/// network, and environment access. A compromised `build.rs` can exfiltrate
/// secrets, plant persistence, or modify the build output.
///
/// **Biology cognate**: macrophage phagosome containment — pathogens
/// internalized for inspection must be contained inside a degradation
/// vesicle, not released into the cytoplasm.
///
/// **Defense (v0.4+)**: sandboxed pre-update via `cargo antigen verify
/// sandbox` (currently stub). v0.2 emits the audit hint as an awareness
/// signal but cannot enforce sandbox execution.
#[antigen(
    name = "unsandboxed-build-script",
    category = [AntigenCategory::SubstrateAlignment, AntigenCategory::FunctionalCorrectness],
    family = "supply-chain-distributed-boundary-innate-immunity",
    summary = "External-dep `build.rs` runs at compile time with full developer/CI privilege. Named limitation: environment-aware (time-bomb) attacks bypass sandbox detection — code that checks IS_CI or timestamps passes the sandbox test but misbehaves in prod. Macrophage phagosome containment cognate.",
    references = ["ADR-025"]
)]
pub struct UnsandboxedBuildScript;

// ============================================================================
// 9. UnsandboxedProcMacro (NEW per B3-R; HIGHER-RISK than build.rs)
// ============================================================================

/// An external proc-macro dependency executes in-`rustc` at compile time
/// without sandbox containment.
///
/// **HIGHER RISK THAN `UnsandboxedBuildScript`**: proc-macros run inside
/// the `rustc` process and have arbitrary code execution there. They can
/// rewrite source token streams, embed payloads in compiler output, and
/// inspect every macro-expansion site in the workspace. A compromised
/// proc-macro is strictly more dangerous than a compromised build.rs.
///
/// **Biology cognate**: macrophage phagosome containment, applied to a
/// privileged compartment (the rustc process itself).
///
/// **Defense (v0.4+)**: `cargo antigen verify proc-macro-sandbox` —
/// stub in v0.2. Adopters should manually audit proc-macro deps and
/// pin them tightly.
#[antigen(
    name = "unsandboxed-proc-macro",
    category = [AntigenCategory::SubstrateAlignment, AntigenCategory::FunctionalCorrectness],
    family = "supply-chain-distributed-boundary-innate-immunity",
    summary = "External proc-macro runs in-`rustc` at compile time. HIGHER risk than `UnsandboxedBuildScript`: arbitrary code execution inside the compiler process. Named limitation: environment-aware (time-bomb) attacks and environment-detection bypass sandbox testing.",
    references = ["ADR-025", "ADR-025#B3-R"]
)]
pub struct UnsandboxedProcMacro;

// ============================================================================
// 10. PostInstallScriptInDependency
// ============================================================================

/// A dependency declares scripts that execute at install/build time
/// (`build.rs`, system-package post-install hooks via FFI, etc.) without a
/// recorded review of what those scripts do.
///
/// **Generalization over `UnsandboxedBuildScript`**: this antigen covers
/// the full surface of install/build-time code, including
/// system-package-manager invocations, vendored binary downloads,
/// `cmake`/`make` invocations, and any other code path that runs
/// arbitrary commands at install time.
///
/// **Defense**: review each dep's `build.rs` + `Cargo.toml`
/// `[package.metadata.*]` for install-time hooks; record the review in
/// the dep-attest sidecar.
#[antigen(
    name = "post-install-script-in-dependency",
    category = AntigenCategory::SubstrateAlignment,
    family = "supply-chain-distributed-boundary-innate-immunity",
    summary = "External code runs at install/build time. Generalizes `UnsandboxedBuildScript` to cover FFI bridges, vendored downloads, cmake/make invocations.",
    references = ["ADR-025"]
)]
pub struct PostInstallScriptInDependency;

// ============================================================================
// 11. ContentHashMismatch (NON-NEGOTIABLE per B1-R)
// ============================================================================

/// The content-hash of the published artifact for a pinned crate version
/// differs from the first-attestation hash recorded in
/// `.attest/supply-chain/content-hash/<crate>@<version>.json`.
///
/// **THE NON-NEGOTIABLE ANTIGEN FOR THE 2025 CHALK/DEBUG/ESLINT-CONFIG
/// ATTACK CLASS**. Cargo.lock pins VERSION but NOT CONTENT-HASH:
/// `[[package]] name = "chalk" version = "5.3.0" checksum = "..."` —
/// but the checksum is recorded at first-resolve time and is not
/// re-verified on subsequent re-resolves if the version is unchanged.
/// An attacker who can swap the artifact at the registry can deliver
/// modified code under the same version+checksum to first-time downloaders.
///
/// **Requires proactive first-attestation**: the
/// `.attest/supply-chain/content-hash/<crate>@<version>.json` sidecar
/// must be created via `cargo antigen verify content-hash record
/// <crate@version>` before this antigen can detect mismatch. Until
/// first-attestation exists, `cargo antigen verify content-hash` emits
/// the `content-hash-no-attestation` hint, not `content-hash-mismatch`.
///
/// **Named limitation (v0.2)**: hash-source is the `checksum =` field in
/// `Cargo.lock` for the recorded version. Direct crates.io API +
/// tarball SHA-256 verification is deferred to v0.3+. This means the
/// v0.2 implementation detects mismatch against the locally-resolved
/// hash, not against the registry's current hash — sufficient for most
/// chalk/debug attack patterns but not for first-resolve poisoning.
///
/// **Biology cognate**: antigenic-identity verification. The immune
/// system identifies pathogens by molecular shape; if the shape changes
/// while the label stays the same, identity verification catches the
/// substitution.
#[antigen(
    name = "content-hash-mismatch",
    category = AntigenCategory::SubstrateAlignment,
    family = "supply-chain-distributed-boundary-innate-immunity",
    summary = "Content-hash of pinned crate@version differs from first-attestation. NON-NEGOTIABLE: defends the chalk/debug/eslint-config (2025) content-replacement-at-fixed-version attack. Cargo.lock pins version, not content-hash.",
    references = [
        "ADR-025",
        "ADR-025#B1-R",
        "https://blog.chainguard.dev/chalk-debug-eslint-config-supply-chain-attack-2025/"
    ]
)]
pub struct ContentHashMismatch;
