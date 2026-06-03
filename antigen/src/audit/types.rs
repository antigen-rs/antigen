//! Shared verdict vocabulary for the audit pipeline.
//!
//! Extracted from the former monolithic `audit.rs` per ADR-036 (the
//! scan/audit orchestration decomposition). This module holds the *types*
//! every audit detector/pass reasons in terms of — the witness-status /
//! tier / hint vocabulary, the immunity-audit record, the aggregate
//! `AuditReport`, the per-site immune verdict, and the work-need verdict
//! lattice. Moving types before logic (extraction step 1) lets every later
//! detector module compile against a stable `types`.
//!
//! API-invisible: every item here is re-exported from the `audit` module
//! root via `pub use types::*;`, so `antigen::audit::WitnessTier` etc.
//! resolve byte-for-byte as before.

use std::path::PathBuf;

use antigen_macros::presents;
use serde::{Deserialize, Serialize};

use crate::scan::Immunity;

/// The status of a single witness validation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum WitnessStatus {
    /// Witness identifier resolves to a function with a recognized testing
    /// attribute in the workspace.
    ///
    /// **Important**: "resolved" means the identifier was found — it does NOT
    /// mean the witness was executed or that it asserts immunity to this specific
    /// failure class. Semantic verification (does the witness actually assert
    /// the antigen's failure mode?) is behavioral-tier work tracked as
    /// the `BehavioralAlignment` witness tier; planned for A4-A5 sweeps
    /// (ADR-001 Amendment 1 Change 4 + ADR-013 phantom-type witness pluralism).
    Resolved {
        /// Where the witness function was found.
        location: PathBuf,
        /// What kind of witness was detected.
        witness_kind: WitnessKind,
    },
    /// Witness identifier appears to reference an external tool (clippy lint,
    /// kani proof, prusti annotation, etc.); deferred to that tool's validator.
    External {
        /// Best-effort guess at the external tool.
        tool_hint: String,
    },
    /// Witness identifier resolves to multiple functions in the workspace
    /// (ATK-A2-005). The caller must qualify the path or rename one
    /// candidate. Audit reports `WitnessTier::None` because no single
    /// resolution was confirmed.
    Ambiguous {
        /// Locations of all candidate functions sharing this name.
        candidates: Vec<PathBuf>,
    },
    /// Witness identifier could not be resolved in the workspace.
    NotFound {
        /// Reason the witness wasn't found (e.g., "no matching function in any
        /// .rs file under the scan root").
        reason: String,
    },
    /// The immunity declaration didn't include a witness identifier at all.
    Missing,
}

/// What kind of witness mechanism was detected.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WitnessKind {
    /// A function with a `#[test]` attribute (and not `#[ignore]`).
    Test,
    /// A function with `#[test]` AND `#[ignore]` — `cargo test` skips it
    /// by default. Audit treats this as Reachability tier, not Execution,
    /// per ADR-005 Amendment 3 (ATK-A2-012).
    IgnoredTest,
    /// A `proptest!` macro invocation.
    Proptest,
    /// A regular function (no testing attribute detected; might be a phantom-type
    /// proof or non-test witness).
    Function,
    /// A phantom-type witness: a path like `Path::<TypeParams>::constructor`
    /// where construction itself is the proof. Recognized structurally per
    /// ADR-013; the audit reports a hint to verify the constructor is sealed.
    PhantomType {
        /// The base path (e.g., `PolarityProof`).
        proof_type: String,
        /// Type parameters if any (e.g., `["FrameTranslation"]`).
        type_params: Vec<String>,
        /// Constructor function name if present (e.g., `verified` in
        /// `PolarityProof::<FrameTranslation>::verified()`).
        constructor: Option<String>,
    },
    /// A substrate-witness predicate evaluated against a sidecar (ADR-019).
    /// The predicate JSON was emitted by `#[immune(requires = ...)]` via P3b
    /// and evaluated by `antigen_attestation::evaluate()` at audit time.
    SubstrateWitness {
        /// Whether the sidecar claimed immunity (`Immunity`) or tolerance
        /// (`Tolerance`) per `antigen_attestation::RatificationKind`.
        kind: antigen_attestation::RatificationKind,
    },
    /// A cross-crate witness: the sidecar lives in a dependency crate, not
    /// in the workspace. Audit reaches it via the dependency's
    /// `.attest/` tree. Distinct from `SubstrateWitness` because trust
    /// boundaries differ — cross-crate witnesses require the
    /// witness-provider-crate enforcement per ADR-019 §F7+T1-R.
    CrossCrateWitness,
}

/// The strength of evidence a witness provides for an immunity claim.
///
/// Per ADR-005 Amendment 3: this enum reports work the audit *actually
/// performed* at the validation point — never potential-maximum evidence.
/// Per-case disambiguation lives on the parallel [`AuditHint`] axis.
///
/// Ordered: higher ordinal = stronger evidence. Stable discriminants
/// reserve room for `BehavioralAlignment` to insert at 3 in a future ADR.
///
/// # CI gating
///
/// `cargo antigen audit --min-tier execution` fails if any immunity claim
/// is below Execution tier.
// `WitnessTier` is one half of a parallel pair with
// `antigen_attestation::tier::WitnessTier`; the two are hand-maintained-in-sync
// because the dep-DAG keeps them in separate crates. That hand-maintained
// parallelism is exactly the `ParallelStateTrackersDiverge` shape — the
// comment-promised "lock-step" enforces nothing; only the
// `atk_witness_tier_parity` test catches drift on derives, discriminants, or
// audit-side-only variants. The peer in `tier.rs` is in a foundation crate
// that can't carry the marker (dep-DAG barrier); fingerprint-scan recall via
// the `doc_contains("lock-step")` pattern provides cross-site coverage there.
#[presents(ParallelStateTrackersDiverge)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum WitnessTier {
    /// No *passing* evidence. Either no witness / unresolved witness (immunity
    /// asserted without evidence), or — for substrate-witnesses — a sidecar that
    /// is missing, schema-invalid, or whose predicate was evaluated and failed.
    /// The parallel [`AuditHint`] carries which case. Kept in lock-step with
    /// `antigen_attestation::tier::WitnessTier::None`.
    None = 0,
    /// Witness identifier resolves but no execution-level verification
    /// happened. Evidence: "this code path / tool reference exists."
    Reachability = 1,
    /// Witness was executed: a test or proptest function whose run was
    /// confirmed (A3+ feature; not yet emitted by v0.1 audit).
    Execution = 2,
    // BehavioralAlignment = 3, reserved per ADR-005 OQ
    /// Compile-time proof: phantom-type construction whose construction is
    /// the proof, or formal-verification tool with confirmed passing proof
    /// (A3+).
    FormalProof = 4,
}

/// Per-case verification-work disambiguation, parallel to [`WitnessTier`].
/// Per ADR-005 Amendment 3 Mechanics §2.
///
/// Two witnesses can carry the same [`WitnessTier`] but different
/// `AuditHint` — for example, an unrun `#[test]` and an external clippy
/// reference both sit at `Reachability` (zero confirmed assertions about
/// this site) but the disambiguation tells the user how to upgrade.
#[presents(DeclaredCapabilityWithNoProductionPath)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AuditHint {
    /// No hint applicable (status is Missing or `NotFound`).
    NoneApplicable,
    /// Identifier resolves to a function; no further check.
    FunctionResolves,
    /// Function has `#[test]`, audit did not invoke `cargo test`.
    TestAttributePresentNotInvoked,
    /// Function has `#[test]` AND `#[ignore]`; `cargo test` would skip it.
    TestAttributePresentIgnoreSkipped,
    /// `proptest!` macro invocation found; harness not invoked.
    ProptestPresentNotInvoked,
    /// External-tool prefix recognized (`clippy::`, `kani::`, ...);
    /// tool not invoked.
    ExternalToolPrefixRecognized,
    /// External tool actually invoked; deferred to A3+.
    ExternalToolInvoked,
    /// Phantom-type witness shape recognized; constructor not validated.
    PhantomTypeShapeRecognized,
    /// Phantom-type witness construction validated; deferred to future ADR.
    PhantomTypeConstructionValidated,
    /// Witness name matches more than one function in the workspace
    /// (ATK-A2-005). Caller should qualify the path.
    AmbiguousResolution,
    /// Witness path's module prefix does not exist in the workspace
    /// (ATK-A2-011). The last segment was found but in an unrelated location.
    FabricatedPathPrefix,
    /// Inherited Presentation lacks re-attestation on the descendant site
    /// (state 7 of the 7-state matrix, ADR-018 `§"AuditHint integration"`).
    /// Behavioral re-validation that the ancestor's witness applies to
    /// the descendant is A4-A5 work; reachability-tier audit cannot
    /// perform this check. The descendant should declare its own
    /// `#[immune]` or `#[antigen_tolerance]`.
    InheritedPresentationNotReAttested,

    /// A `#[descended_from(Parent)]` lineage edge whose CHILD antigen's
    /// structural fingerprint is detectably NOT a refinement of the PARENT's
    /// (lineage-fidelity check; scientist severity ruling 2026-05-27: ADVISORY
    /// for v0.3, hard-fail deferred to a future ADR).
    ///
    /// A child fingerprint *refines* its parent's when every item matching the
    /// child also matches the parent (`child.matches ⊆ parent.matches`) — the
    /// child is at-least-as-specific. This hint fires only on the conservative,
    /// statically-decidable NON-refinement cases (no false positives):
    /// - the child's top-level `item = <kind>` differs from the parent's, or
    /// - the parent requires a `doc_contains(s)` substring that no child
    ///   `doc_contains` contains.
    ///
    /// Glob-containment for `name = matches(...)` is deferred (the harder case).
    ///
    /// Biology cognate: MHC restriction / negative selection — a lineage claim
    /// that doesn't survive the structural check is a mis-matched TCR. Negative
    /// selection is strict (the autoreactive clone is deleted), so the eventual
    /// posture is hard-fail; v0.3 advisory is the AIRE testing window.
    DescendedFromFingerprintDivergence,

    // ------------------------------------------------------------------
    // Substrate-witness hints (ADR-019). These exist as legacy-enum echoes
    // of `antigen_attestation::SubstrateAuditHint` so the user-facing
    // audit output names the actual state the substrate-witness pipeline
    // reached. Mapped from the attestation enum by
    // [`map_attestation_audit_hint`].
    //
    // History: rc.1 mapped every substrate hint to `NoneApplicable` /
    // `ExternalToolPrefixRecognized` — collapsing real diagnostic
    // information. rc.2 surfaces the substrate hints natively so the user
    // can distinguish `discipline-sidecar-missing` (no proof yet) from
    // `discipline-predicate-failed` (proof attempted and failed) from
    // `discipline-substrate-stale` (proof attempted and is now expired).
    // ------------------------------------------------------------------
    /// `#[immune(X, requires = ...)]` declared but no `.attest/<X>.json`
    /// sidecar exists. The substrate-witness pipeline engaged; no
    /// substrate to evaluate.
    DisciplineSidecarMissing,
    /// `.attest/<X>.json` exists but did not deserialize as a valid
    /// `Ratification`. Treat as a hard failure — the sidecar is the
    /// load-bearing trust artifact; a corrupt one cannot back a claim.
    DisciplineSidecarSchemaInvalid,
    /// Sidecar parsed but the substrate-witness predicate failed
    /// evaluation (a leaf returned false). Per-leaf detail surfaces
    /// elsewhere in the audit output.
    DisciplinePredicateFailed,
    /// Sidecar parsed; no leaf evaluated to false, but ≥1 leaf was
    /// deferred (not evaluated by this evaluator — e.g. supply-chain
    /// leaves on the standard path). Indeterminate — not failed.
    /// Drive `cargo antigen verify` (supply-chain audit) to resolve.
    DisciplinePredicateDeferred,
    /// Predicate passes but ≥1 signer's recorded fingerprint diverges
    /// from the current item fingerprint, AND the leaf used
    /// `against = "current"`. Re-attestation required.
    DisciplineSubstrateStale,
    /// Predicate passes via a delta chain whose depth is at or near the
    /// configured cap (`chain_depth >= cap - 1`). Informational; the next
    /// delta will be refused.
    DisciplineSubstrateDeltaChainNearCap,
    /// Predicate passes, all current, ≥1 signer's basis is `DeltaFrom`
    /// (within caps). Carry-forward attestation rather than fresh.
    DisciplinePredicatePassedViaDeltaChain,
    /// Predicate passes, all current, all signers Fresh. Strongest
    /// substrate-witness state available in v0.1.
    DisciplinePredicatePassedSubstrateCurrent,
    /// `#[antigen_tolerance(X)]` declared without `sidecar = true`
    /// opt-in. Vibes-grade tolerance — no substrate consulted.
    ToleranceVibesGrade,
    /// `#[antigen_tolerance(X, sidecar = true)]` declared but no sidecar
    /// exists at the expected `.attest/<X>.json` location.
    ToleranceSidecarMissing,
    /// Tolerance sidecar exists but predicate failed.
    TolerancePredicateFailed,
    /// Tolerance sidecar exists, predicate passes, all signers current
    /// and Fresh. Strongest tolerance-attestation state in v0.1.
    TolerancePredicatePassedSubstrateCurrent,
    /// `#[immune(X, requires = ...)]` site but the sidecar's `kind` is
    /// `Tolerance`. Likely a stale sidecar from a prior `#[antigen_tolerance]`
    /// declaration; regenerate the sidecar.
    DisciplineSidecarKindMismatchExpectedImmunityGotTolerance,
    /// `#[antigen_tolerance(X, sidecar = true, requires = ...)]` site but
    /// the sidecar's `kind` is `Immunity`. Symmetric to the immunity-side
    /// mismatch above.
    ToleranceSidecarKindMismatchExpectedToleranceGotImmunity,
    /// Site declares BOTH `#[immune(X, ...)]` and
    /// `#[antigen_tolerance(X, sidecar = true, ...)]` for the same
    /// antigen. Logically incoherent — overrides individual tier reports.
    DisciplineImmunityToleranceContradiction,

    // ------------------------------------------------------------------
    // Deferred-Defense Family hints (ADR-023).
    // These are emitted by `cargo antigen audit` / `cargo antigen defer status`
    // for sites annotated with the deferred-defense primitives.
    // ------------------------------------------------------------------

    // --- Anergy hints ---
    /// `#[anergy]` present; `until` date has not passed. Anergy is active;
    /// the deferred defense is intentionally muted.
    AnergyActive,
    /// `#[anergy]` past its `until` date; `expected_co_stimulation` has not
    /// arrived. Time to re-evaluate immunity or re-declare with a new `until`.
    AnergyCostimulationNotArrived,
    /// `#[anergy]` significantly past its `until` date (past grace period).
    /// Escalates to warning-level. Structural memory says immunity was
    /// supposed to be revisited.
    AnergyStale,

    // --- Immunosuppress hints ---
    /// `#[immunosuppress]` present; `until` date has not passed.
    ImmunosuppressActive,
    /// `#[immunosuppress]` past its `until` date. Suppression has expired;
    /// re-evaluate and re-declare or restore immunity checks.
    ImmunosuppressExpired,
    /// `#[immunosuppress]` duration exceeded the workspace cap. Should not
    /// appear post-compile (parse-time enforced), but retained for audit
    /// re-evaluation of pre-cap-enforcement code in the repo.
    ImmunosuppressDurationCapExceeded,

    // --- Poxparty hints ---
    /// `#[poxparty]` present; exercise in progress (`until` not passed).
    PoxpartyActive,
    /// `#[poxparty]` past `until`; outcome not yet recorded.
    PoxpartyOutcomePending,
    /// `#[poxparty]` past `until`; outcome attestation has been recorded.
    PoxpartyOutcomeRecorded,
    /// `#[poxparty]` site found outside expected cfg-gated isolation scope.
    /// Indicates the `antigen-poxparty` feature isolation may be bypassed.
    PoxpartyOutsideIsolation,

    // --- Orient hints ---
    /// `#[orient]` present; orientation in progress.
    OrientActive,
    /// `#[orient]` past its required `until` deadline (ADR-023: `until` is
    /// mandatory for orient). The orientation period elapsed without the
    /// failure-class being resolved — escalates to action-required. Emitted by
    /// `audit_deferred_defenses` once the until-date passes.
    OrientPendingActionRequired,

    // --- Cross-cutting deferred-defense hint ---
    /// A deferred-defense hint (e.g., `anergy-active`) was suppressed in
    /// workspace config without a non-empty rationale. Per ADR-023
    /// hint-fatigue-protection: suppression requires rationale.
    DeferredDefenseHintSuppressedWithoutRationale,

    // ------------------------------------------------------------------
    // Supply-Chain Defense Family hints (ADR-025).
    //
    // 15 hints covering the v0.2 supply-chain antigens. Emitted by
    // `audit_supply_chain()` (NOT by the standard `audit()` pipeline,
    // which evaluates substrate-witness predicates). The supply-chain
    // audit drives the witness-leaf evaluators in
    // `crate::supply_chain::evaluate` and maps the resulting states
    // onto these hints.
    // ------------------------------------------------------------------
    /// A dependency in `Cargo.toml` is not exact-pinned (`=X.Y.Z`).
    /// Backs `UnpinnedDependency`.
    UnpinnedDependency,
    /// A direct dep declares `*`/`?` ranges for its own transitive
    /// dependencies — NARROW form per ADR-025 B9-R. Backs
    /// `UnpinnedTransitiveDependency`. v0.2: emitted only when the
    /// direct dep's manifest is accessible; v0.3+ broadens coverage.
    UnpinnedTransitiveDependency,
    /// A new dependency was added without a dep-attest sidecar at
    /// `.attest/supply-chain/dep-attest/<crate>@<version>.json`.
    /// Backs `UnattestedDependencyInclusion`.
    UnattestedDependencyInclusion,
    /// A dep upgrade was attested only at `MetadataOnly` or
    /// `BuildScriptOnly` scope — not diff-reviewed. Backs
    /// `DependencyUpgradeWithoutDiffReview`. Account-compromise control.
    DependencyUpgradeWithoutDiffReview,
    /// A crate's maintainer set has changed (or query unavailable) and
    /// no fresh re-attestation has landed. Backs
    /// `MaintainerChangeWithoutReattestation`. CI sequencing constraint:
    /// the verifying CLI MUST run BEFORE `cargo update`.
    MaintainerChangeWithoutReattestation,
    /// Variant of `MaintainerChangeWithoutReattestation` surfaced AFTER
    /// `cargo update` has already incorporated the new maintainer's
    /// code. The check has effectively already-failed; document the
    /// sequencing for next time.
    MaintainerChangeDetectedAfterCargoUpdate,
    /// A dep version bump grew the source tree (LOC) or transitive-dep
    /// count significantly. Backs `SuddenDependencyExpansion`.
    /// Account-compromise complement to
    /// `DependencyUpgradeWithoutDiffReview`.
    SuddenDependencyExpansion,
    /// An external dep ships a `build.rs` that ran at compile time
    /// without sandbox containment. Backs `UnsandboxedBuildScript`.
    /// v0.4+ sandbox execution.
    UnsandboxedBuildScript,
    /// An external proc-macro dep ran in-rustc at compile time without
    /// sandbox containment. Backs `UnsandboxedProcMacro`. HIGHER risk
    /// than `unsandboxed-build-script`.
    UnsandboxedProcMacro,
    /// An external dep declares install-time scripts (post-install
    /// hooks, vendored binary downloads, FFI bridges). Backs
    /// `PostInstallScriptInDependency`.
    PostInstallScriptInDependency,
    /// The recorded `.attest/supply-chain/content-hash/<crate>@<version>.json`
    /// hash DIFFERS from the current `Cargo.lock` checksum. **The
    /// chalk/debug/eslint-config attack signal.** Backs
    /// `ContentHashMismatch`.
    ContentHashMismatch,
    /// No `.attest/supply-chain/content-hash/<crate>@<version>.json`
    /// record exists for this dep. The antigen is dormant until
    /// first-attestation lands via `cargo antigen verify content-hash
    /// record <crate@version>`. Surfaces explicitly per ATK-SC-2 (NOT
    /// a silent pass).
    ContentHashNoAttestation,
    /// The dep-attest sidecar exists but `reviewable_artifact` is empty
    /// or whitespace-only — a rubber-stamp. Per ATK-SC-1-A.
    /// Backs the rubber-stamp limitation named in ADR-025.
    DepAttestWithoutReviewableArtifact,
    /// A `cargo antigen verify maintainer-changes` query to crates.io
    /// failed (network, rate-limit, or v0.2 stub). CI should treat this
    /// as a soft-fail, not a green light. Backs the
    /// `MaintainerChangeWithoutReattestation` named limitation.
    CratesIoMetadataQueryFailed,
    /// A dep-attest sidecar's recorded version is older than the
    /// requested version (and `exact_version = true`). Re-attestation
    /// needed before the upgrade lands.
    DepAttestationStale,
    /// A `*` or `?` version specifier exists somewhere in the
    /// dependency tree, allowing automatic chain-of-updates with no
    /// human gate. Backs `AutoDependencyChainWithoutPinning`.
    AutoDependencyChainWithoutPinning,
    /// `evaluate_content_hash_matches` returned `SidecarMalformed` —
    /// the `.attest/supply-chain/content-hash/<crate>@<version>.json`
    /// file exists but does NOT deserialize cleanly. **Per ATK-SC-2-A,
    /// this MUST be a distinct hint from `content-hash-no-attestation`**:
    /// an attacker who can write to the sidecar should not be able to
    /// downgrade a Mismatch (alert) into a `NoAttestation` (warning) by
    /// corrupting the file.
    ContentHashSidecarMalformed,
    /// A `requires_predicate` JSON string on an immunity deserialized
    /// successfully but failed structural validation (e.g., `all_of([])` —
    /// an empty combinator that vacuously evaluates to `passed=true` with no
    /// leaves). Emitted by `audit_supply_chain` when `Predicate::validate()`
    /// fails after serde deserialization. (ATK-SC-7)
    MalformedRequiresPredicate,

    // ------------------------------------------------------------------
    // Convergent-Evidence Family hints (ADR-024).
    //
    // 11 hints covering the seven convergent primitives. Emitted by
    // `audit_convergent_evidence()`.
    // ------------------------------------------------------------------
    /// `#[diagnostic]` has fewer distinct `WitnessClass` categories than
    /// `min_independent` requires. Per ADR-024 §Decision + adversarial C1.
    DiagnosticModalityInsufficient,
    /// `#[diagnostic]` modalities all share a single `WitnessClass` —
    /// the `min_independent` floor is structurally unmet even if the
    /// raw count matches. Per adversarial C1 (class-collapse).
    DiagnosticModalitiesClassCollapsed,
    /// `#[diagnostic]` declared with no modalities. Empty modality
    /// list is structurally meaningless.
    DiagnosticModalitiesEmpty,
    /// `#[diagnostic]` `min_independent` is zero — a null threshold that
    /// never fires `DiagnosticModalityInsufficient` regardless of how many
    /// (or few) independent classes exist. Zero independence claimed = no
    /// claim. (ATK-CE-5)
    DiagnosticMinIndependentZero,
    /// `#[igg]` `min_reattestations` is zero — a null threshold that never
    /// fires `IggReattestationsInsufficient`. Zero re-attestations required
    /// = no reattestation discipline. (ATK-CE-5)
    IggMinReattestationsZero,
    /// `#[clonal]` declared with `seed = SeedKind::Fixed(_)`. The
    /// proc-macro rejects this at parse time; this hint is for the
    /// audit-time re-evaluation of pre-cap-enforcement source.
    ClonalFixedSeedDetected,
    /// `#[clonal]` `iterations` below the configured workspace
    /// threshold. Default threshold: 100 iterations.
    ClonalIterationsBelowThreshold,
    /// `#[igg]` all re-attestations share the same signer identity —
    /// nominal source-independence collapses to identity-collapse.
    /// Per adversarial C3 named limitation.
    IggIdentityCollapseWarning,
    /// `#[igg]` `historical_span` shorter than the configured workspace
    /// threshold.
    IggSpanTooShort,
    /// `#[igg]` `min_reattestations` not met by available signatures.
    IggReattestationsInsufficient,
    /// `#[crossreactive]` references a fingerprint string that doesn't
    /// match any known antigen in the scan report.
    CrossreactiveFingerprintUnresolved,
    /// `#[polyclonal]` site has fewer independent witness lineages than
    /// the configured floor (default: 2). **Planned — not yet emitted at v0.2**:
    /// no lineage-counting audit pass exists; do not rely on this hint firing.
    PolyclonalInsufficientLineages,
    /// `#[adcc]` site has only one of the two committed mechanisms
    /// (antibody-style + cellular-effector-style) detectable. **Planned — not yet
    /// emitted at v0.2**: no mechanism-detection pass exists; do not rely on this
    /// hint firing.
    AdccSingleMechanismOnly,

    // ------------------------------------------------------------------
    // Recurrent-Emergence Family hints (ADR-024 §Family 2).
    //
    // Pre-authorized under ADR-024 §5471 "~30 examples:" open-set wording
    // per aristotle Reading-A (744471a3): family-prefixed, substrate-grep-
    // clean, semantically within the recurrent audit taxonomy. Emitted by
    // `audit_recurrent()`.
    // ------------------------------------------------------------------
    /// `#[itch]` declared with no `antigen` path — an unlinked noticing.
    /// The pattern was noticed but never tied to a failure-class, so it
    /// can't graduate via `#[crystallize]`. Informational, not a failure.
    ItchNoticedNotAnchored,
    /// `#[recurrence_anchor]` declared but no `#[immune]` / `#[presents]`
    /// in the workspace addresses the anchored antigen — the recurrence
    /// crossed threshold but no action followed.
    RecurrenceThresholdReachedNoAction,
    /// `#[recurrence_anchor]` declared but no `#[itch]` declarations in the
    /// workspace reference the same antigen type — the anchor has no upstream
    /// noticing precondition. The temporal progression (itch → anchor →
    /// crystallize) is bypassed: commitment declared without prior noticing.
    RecurrenceAnchorNoItchPrecondition,
    /// `#[crystallize]` declared with no `antigen` path AND no `from_itches`
    /// — a crystallization event with nothing it crystallized FROM or INTO.
    CrystallizeWithoutAntigen,
    /// `#[chronic]` declared with no `managed_by` — a persistent signal
    /// with no owning role/team. Surfaces drift toward unmanaged chronicity.
    ChronicSignalUnmanaged,
    /// `#[chronic]` whose `since` date is far enough in the past (past the
    /// configured review horizon) that the chronic state warrants re-review.
    ChronicSignalPastReviewDate,
    /// `#[chronic]` whose `since` value is neither a parseable ISO-8601 date
    /// NOR a recognizable version tag (e.g., `"not-a-date"`). Per ATK-RECURRENT-4a:
    /// ISO dates enforce the review horizon; version tags are tolerated (the
    /// chronic state is anchored to a release, not a calendar); but an
    /// unparseable garbage string is a malformed anchor — the `since` claims
    /// a temporal/version origin that resolves to nothing.
    ChronicSinceNotADate,
    /// `#[saturate]` declared with no `contributing_to` target — saturation
    /// evidence accumulating toward nothing nameable.
    SaturateNoAnchor,
    /// `#[strand]` declared with no `anchored_by` entries — a thread of
    /// noticing that anchors nothing.
    StrandNoAnchors,

    // ------------------------------------------------------------------
    // Mucosal Boundary Family hints (ADR-027 + Amendment 1).
    //
    // Pre-authorized under ADR-027 §Audit-hint vocabulary "examples:"
    // open-set framing per aristotle F7. Emitted by `audit_mucosal()`.
    // ------------------------------------------------------------------
    /// A boundary surfaced by scan carries no `#[mucosal]` /
    /// `#[mucosal_tolerant]` declaration. (v0.2: emitted by
    /// `mucosal-map --undefended`; retained for vocabulary completeness.)
    MucosalBoundaryUndefended,
    /// `#[mucosal]` / `#[mucosal_delegate]` declared with no recognized
    /// `kind` / `boundary` — the `MucosalKind` didn't resolve.
    MucosalKindMismatch,
    /// `#[mucosal]` rationale missing or below the ≥20-char floor.
    MucosalRationaleInsufficient,
    /// `#[mucosal_delegate]` whose `handled_by` path does not resolve to any
    /// function in the workspace. Three-tier diagnosis tier 1 (Change 5a).
    MucosalDisciplineDelegateTargetMissing,
    /// `#[mucosal_delegate]` whose `handled_by` target exists but carries no
    /// `#[mucosal]` declaration. Three-tier diagnosis tier 2 (Change 5b).
    MucosalDisciplineDelegateTargetNotMucosal,
    /// `#[mucosal_delegate]` whose handler carries `#[mucosal]` but none of
    /// its `kind`s match the delegate's `boundary` (set-membership, NOT
    /// exact-equality). Three-tier diagnosis tier 3 (Change 5c).
    MucosalDisciplineDelegateTargetKindMismatch,
    /// `#[mucosal_delegate]` `handled_by` matches multiple `#[mucosal]` functions
    /// with the SAME bare name in DIFFERENT source files — the target is ambiguous.
    /// The kind-set union of all same-named functions may silently pass using the
    /// wrong file's kinds. Fix: qualify `handled_by` to a unique target.
    /// (findings/mucosal-same-name-fn-collision)
    MucosalDisciplineDelegateTargetAmbiguous,
    /// `#[mucosal_tolerant]` rationale missing or below the ≥40-char floor
    /// (higher than `#[mucosal]` — tolerance is the riskier declaration).
    MucosalTolerantRationaleInsufficient,
    /// `#[mucosal_tolerant]` whose `until` review-deadline has passed.
    MucosalTolerantPastReviewDate,
    /// `#[mucosal_tolerant]` with an empty/missing `accepts` description.
    MucosalTolerantAcceptsEmpty,
    /// `#[mucosal_tolerant]` with no `reviewed_by` (v0.2.1+ migration hint).
    MucosalTolerantWithoutReviewer,

    // ------------------------------------------------------------------
    // Antigen-Category hints (ADR-028).
    //
    // G1 deliverable: the category-defaulted migration hint, emitted at
    // scan/audit time for antigen declarations with an absent (empty)
    // `category` field. Per adversarial's G1 ratification (scan-time-only
    // for v0.2), this hint is the load-bearing signal that makes
    // absent-category VISIBLE rather than a silent false-green. The
    // parse-time hard-error + v0.1/v0.2 discrimination (migration-record)
    // are deferred to v0.2.x.
    // ------------------------------------------------------------------
    /// An `#[antigen]` declaration has no `category = AntigenCategory::X`
    /// field. Per ADR-028 §v0.2-backward-compat, absent category defaults to
    /// `[FunctionalCorrectness]` + this migration hint. v0.2 ships scan-time
    /// emission (this hint); parse-time hard-error for v0.2+-new declarations
    /// is the v0.2.x migration-record slice. The hint fires equally for v0.1
    /// carry-overs and new v0.2 declarations until that discrimination lands
    /// — both should migrate to an explicit category.
    AntigenCategoryDefaultedImplicitFunctional,

    // ------------------------------------------------------------------
    // G2 deliverable (ADR-028 + Amendment 2 + aristotle F1 on
    // v02-impl-category-witness-cross-check): the category-vs-witness-type
    // cross-check, emitted at AUDIT time (not parse time). A single
    // `#[antigen]` cannot see its `#[immune]` declarations at macro-expand
    // time — the immunities are separate declarations, joined only when the
    // scan report assembles. So the check that an antigen's declared
    // `category` is backed by the right witness TYPE lives here, where the
    // antigen↔immunity join exists. The witness-type is read structurally
    // from each immunity: `requires_predicate.is_some()` is a substrate-
    // witness; a non-empty `witness` is a code-witness.
    // ------------------------------------------------------------------
    /// An antigen's declared `category` is not backed by an immunity of the
    /// matching witness type. Per ADR-028 §Schema: `SubstrateAlignment`
    /// requires ≥1 substrate-witness immunity (`requires = <predicate>`);
    /// `FunctionalCorrectness` requires ≥1 code-witness immunity
    /// (`witness = <fn>`); a hybrid `[SubstrateAlignment, FunctionalCorrectness]`
    /// requires both. The mismatch is advisory (an audit hint, not a hard
    /// error) per Amendment 2's "enforced at audit-time" disclosure; CI-gating
    /// the audit preserves the strict-enforcement value.
    AntigenCategoryClaimInconsistentWithPredicateType,

    /// A hybrid antigen (`category = [SubstrateAlignment, FunctionalCorrectness]`)
    /// has exactly ONE of its two axes witnessed at audit time — one axis is
    /// backed by a matching immunity, the other is unwitnessed. Per aristotle's
    /// G3 F1 ruling, this is distinct from
    /// [`Self::AntigenCategoryClaimInconsistentWithPredicateType`]: a hybrid
    /// with one axis covered is INCOMPLETE (partial evidence), not a full
    /// structural violation (which is the zero-axes case, still reported as
    /// claim-inconsistent). ADR-028 §Schema: "hybrid antigen; one axis
    /// unwitnessed at audit-time."
    AntigenCategoryHybridIncompleteEvidence,

    // ------------------------------------------------------------------
    // Silence-witness shape-mismatch hints (scientist design 2026-05-27 in
    // forward/silence-witness-shape-mismatch-hint; aristotle architectural
    // gate cleared 2026-05-27 in forward/silence-witness-shape-mismatch-impl).
    //
    // A `SubstrateAlignment` antigen fails by SILENCE — a representation
    // drifts from actual state and nothing fires, because the antigen is
    // about ABSENCE of a closure-mechanism, not a wrong output. The
    // silence-by-absence generator (scientist's 2x2 cap analysis) is defeated
    // only by a witness that asserts the mechanism EXISTS (a substrate
    // predicate or a bijection/parity test), not by a code-behavior test.
    // These two advisory hints flag the witness-shape that cannot detect
    // silence. They live in `audit_category()`'s per-decl loop, sharing the
    // antigen_type-keyed witness correlation G2 already computes (the
    // locus-dispatch family: G2 + silence-witness + ADR-030 + ADR-031 all
    // reuse the same audit-cross-reference path). Advisory, audit-time —
    // same pattern as G1/G2.
    // ------------------------------------------------------------------
    /// A [`SubstrateAlignment`](crate::category::AntigenCategory::SubstrateAlignment)
    /// antigen has NO registered witness of any kind — no `#[immune]`, no
    /// `#[defended_by]`, no `requires =` predicate. This is the
    /// silence-by-absence generator: a substrate-alignment failure is detected
    /// only by a mechanism that asserts the closure exists, and no mechanism is
    /// wired. Distinct from G2
    /// ([`Self::AntigenCategoryClaimInconsistentWithPredicateType`]), which
    /// deliberately treats no-witness as an orthogonal coverage gap and bails
    /// before the category check; this hint fills exactly that gap for the SA
    /// category, where the absence is itself the silence-generator. Advisory.
    /// Recommends a parity/bijection witness that asserts the closure-mechanism
    /// exists, not merely that the two representations agree at this moment.
    AntigenWitnessShapeMismatchForSilenceNoWitness,

    /// A [`SubstrateAlignment`](crate::category::AntigenCategory::SubstrateAlignment)
    /// antigen's ONLY witnesses are code-tier (a `witness = fn` immunity or a
    /// `#[defended_by]` registration — [`WitnessTier::Reachability`] /
    /// [`WitnessTier::Execution`]) with no `requires =` substrate predicate. A
    /// code-reachability test detects BEHAVIORAL failures; a substrate-alignment
    /// failure needs a substrate-state evaluator (`requires =`) or a
    /// bijection-parity test. Co-emitted alongside G2's
    /// [`Self::AntigenCategoryClaimInconsistentWithPredicateType`] (same root
    /// cause, witness-type mismatch) but carries the silence-generator framing
    /// and the actionable guidance G2 does not. Per scientist's design, the
    /// wrong-weighting generator legitimately uses a code-tier
    /// confidence-discrimination test — so this is advisory, and the reader
    /// should confirm the intended generator before treating it as a mismatch.
    /// Suppressed when a substrate witness is also present (the code test is
    /// then supplementary, not the sole defense).
    AntigenWitnessShapeMismatchForSilenceWrongTier,
}

/// Result of auditing a single immunity declaration.
///
/// Two structured fields express what the audit found:
/// - [`witness_tier`](Self::witness_tier): Ord-able strength of evidence,
///   what CI gates check (`--min-tier`)
/// - [`audit_hint`](Self::audit_hint): per-case verification-work
///   disambiguation, what humans read in reports
///
/// Both are derived from [`witness_status`](Self::witness_status) at audit
/// time per ADR-005 Amendment 3 §Mechanics §2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImmunityAudit {
    /// The original immunity declaration.
    pub immunity: Immunity,
    /// What we determined about its witness.
    pub witness_status: WitnessStatus,
    /// Strength of evidence the witness provides, derived from
    /// `witness_status` per ADR-005 Amendment 3.
    pub witness_tier: WitnessTier,
    /// Per-case verification-work disambiguation; carries the signal that
    /// the tier ordinal alone cannot.
    pub audit_hint: AuditHint,
    /// What kind of evidence the witness produces (third axis added by
    /// ADR-019 §M5 alongside `WitnessTier` and `AuditHint`).
    ///
    /// Defaults to `EvidenceKind::None` for backward compatibility with
    /// pre-ADR-019 serialized audit reports. Existing code-witness paths
    /// (Test / `IgnoredTest` / Proptest / Function) map to `Behavioral`;
    /// `PhantomType` maps to `TypeSystemProof`. Substrate-witness audits
    /// (P3c integration) set this to `SubstrateState`.
    #[serde(default = "default_evidence_kind")]
    pub evidence_kind: antigen_attestation::EvidenceKind,
    /// Strength of the signer-identity binding for substrate-witness
    /// audits. `None` for code-witness paths; `Some(GitTrust)` for v0.1
    /// substrate-witnesses; `Some(CryptoSigned)` reserved for v0.4+
    /// DSSE + Sigstore activation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature_strength: Option<antigen_attestation::SignatureStrength>,
    /// `true` when this immunity is supported by evidence from multiple
    /// witness tiers simultaneously (e.g., a code-tier test AND a
    /// substrate-witness sidecar). F19 Gap-2 / ADR-019 §F11.
    /// Reserved for `witnesses = [...]` multi-witness syntax; `false`
    /// for all v0.1 single-witness audits.
    #[serde(default)]
    pub compound_evidence: bool,
    /// The predicate JSON that was evaluated, if this was a substrate-witness
    /// audit. Populated when `immunity.requires_predicate` is `Some`.
    /// `None` for code-witness paths. F19 Gap-4 / ADR-019 §M4.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evaluated_predicate: Option<String>,
    /// `true` when this is a *code-witness* immunity (`witness = ...`, not
    /// `requires = ...`) for which a `.attest/<antigen>.json` substrate-witness
    /// sidecar nonetheless exists on disk. The sidecar can never be credited:
    /// substrate-witness sidecars are evaluated only for `requires = ...`
    /// immunities, so a sidecar scaffolded + signed against a `witness = ...`
    /// site is silently dead. Audit surfaces this as a warning (DX finding 3 —
    /// the silent disconnect between the attestation surface and the declared
    /// witness kind). `false` for the common case (no orphan sidecar).
    #[serde(default)]
    pub code_witness_sidecar_ignored: bool,
    /// Per-leaf substrate-witness evaluation outcomes (Finding 7), in
    /// evaluation order. Populated for `requires = ...` (substrate-witness)
    /// audits so `cargo antigen audit` / `attest check` can render which leaf
    /// of a compound predicate passed or failed and why (expected-vs-found),
    /// rather than only the tree-level hint. Empty for code-witness audits and
    /// for pre-Finding-7 serialized reports.
    #[serde(default)]
    pub leaf_outcomes: Vec<antigen_attestation::LeafOutcome>,
}

/// Backward-compat default for [`ImmunityAudit::evidence_kind`] on
/// pre-ADR-019 serialized reports. New audits derive the kind via
/// [`evidence_kind_from_status`].
const fn default_evidence_kind() -> antigen_attestation::EvidenceKind {
    antigen_attestation::EvidenceKind::None
}

impl ImmunityAudit {
    /// True if the witness provides any evidence (tier > None).
    #[must_use]
    pub const fn has_witness(&self) -> bool {
        !matches!(self.witness_tier, WitnessTier::None)
    }

    /// True if the witness meets a minimum evidence tier. Used by `--strict`
    /// mode and CI gates.
    #[must_use]
    pub fn meets_tier(&self, minimum: WitnessTier) -> bool {
        self.witness_tier >= minimum
    }

    /// True if the audit considers the immunity claim well-formed.
    ///
    /// Per ADR-005 Amendment 3: well-formed requires execution-tier evidence
    /// or stronger. `Reachability`-tier witnesses (e.g., a `#[test]` that
    /// hasn't been run, or a fabricated `clippy::` lint) are NOT well-formed.
    /// This is the post-W7 honest definition; the pre-W7 `is_well_formed`
    /// returned true for any `Resolved`/`External`, which is the bug
    /// ATK-A2-003/004/005/011/012 named.
    #[must_use]
    pub fn is_well_formed(&self) -> bool {
        self.meets_tier(WitnessTier::Execution)
    }
}

impl WitnessTier {
    /// Derive the tier from a [`WitnessStatus`] per the Amendment 3 mapping
    /// table. The audit reports the work it actually performed — never
    /// potential maximum evidence.
    #[must_use]
    pub const fn from_status(status: &WitnessStatus) -> Self {
        match status {
            WitnessStatus::Missing
            | WitnessStatus::NotFound { .. }
            | WitnessStatus::Ambiguous { .. } => Self::None,
            WitnessStatus::External { .. } => Self::Reachability,
            WitnessStatus::Resolved { witness_kind, .. } => match witness_kind {
                // v0.1 audit does not invoke cargo test or proptest harness;
                // witness presence means "this code path exists" — Reachability.
                // Execution tier requires confirmed invocation (A3+ work).
                WitnessKind::Test
                | WitnessKind::IgnoredTest
                | WitnessKind::Proptest
                | WitnessKind::Function => Self::Reachability,
                WitnessKind::PhantomType { .. } => Self::FormalProof,
                // Substrate-witness predicates evaluated by
                // antigen_attestation::evaluate() report Reachability: the
                // sidecar exists and was read, but we don't confirm the
                // external signers ran any executable process. Substrate
                // review evidence is comparable to a `#[test]` that hasn't
                // been invoked by cargo test — it exists, was checked for
                // structural validity, but execution-tier confirmation is
                // A3+ work (invoking git-trailer-based oracles etc.).
                WitnessKind::SubstrateWitness { .. } | WitnessKind::CrossCrateWitness => {
                    Self::Reachability
                }
            },
        }
    }
}

/// Derive [`antigen_attestation::EvidenceKind`] from a [`WitnessStatus`]
/// per ADR-019 §M5.
///
/// Code-witness paths (Test / `IgnoredTest` / Proptest / Function) map to
/// `Behavioral` because they exercise the code at runtime. `PhantomType`
/// maps to `TypeSystemProof` (compile-time construction-is-the-proof).
/// Substrate-witness paths (predicate-evaluated via
/// `antigen_attestation::evaluate`) set `SubstrateState` directly when
/// the audit constructs the [`ImmunityAudit`] — they don't go through
/// this mapping.
#[must_use]
pub const fn evidence_kind_from_status(
    status: &WitnessStatus,
) -> antigen_attestation::EvidenceKind {
    match status {
        WitnessStatus::Missing
        | WitnessStatus::NotFound { .. }
        | WitnessStatus::Ambiguous { .. } => antigen_attestation::EvidenceKind::None,
        WitnessStatus::External { .. } => antigen_attestation::EvidenceKind::Behavioral,
        WitnessStatus::Resolved { witness_kind, .. } => match witness_kind {
            WitnessKind::Test
            | WitnessKind::IgnoredTest
            | WitnessKind::Proptest
            | WitnessKind::Function => antigen_attestation::EvidenceKind::Behavioral,
            WitnessKind::PhantomType { .. } => antigen_attestation::EvidenceKind::TypeSystemProof,
            WitnessKind::SubstrateWitness { .. } | WitnessKind::CrossCrateWitness => {
                antigen_attestation::EvidenceKind::SubstrateState
            }
        },
    }
}

impl AuditHint {
    /// Derive the audit hint from a [`WitnessStatus`] per the Amendment 3
    /// mapping table.
    #[must_use]
    pub const fn from_status(status: &WitnessStatus) -> Self {
        match status {
            WitnessStatus::Missing | WitnessStatus::NotFound { .. } => Self::NoneApplicable,
            WitnessStatus::Ambiguous { .. } => Self::AmbiguousResolution,
            WitnessStatus::External { .. } => Self::ExternalToolPrefixRecognized,
            WitnessStatus::Resolved { witness_kind, .. } => match witness_kind {
                WitnessKind::Test => Self::TestAttributePresentNotInvoked,
                WitnessKind::IgnoredTest => Self::TestAttributePresentIgnoreSkipped,
                WitnessKind::Proptest => Self::ProptestPresentNotInvoked,
                WitnessKind::Function => Self::FunctionResolves,
                WitnessKind::PhantomType { .. } => Self::PhantomTypeShapeRecognized,
                // Substrate-witness predicates report ExternalToolPrefixRecognized
                // because the sidecar was located and structurally validated but
                // no executable was invoked. The hint surfaces the "upgrade path"
                // (invoke attest check / run oracles) parallel to clippy's hint.
                WitnessKind::SubstrateWitness { .. } | WitnessKind::CrossCrateWitness => {
                    Self::ExternalToolPrefixRecognized
                }
            },
        }
    }
}

/// A diagnostic for state 7 of the 7-state interaction matrix:
/// inherited Presentation that lacks immune or tolerance re-attestation
/// on the descendant site. ADR-018 §"Audit diagnostic text".
///
/// Emitted at warn level by default; `--strict` promotes to error.
/// The descendant inherited a presentation from one or more ancestors
/// via `#[descended_from]` propagation; ADR-005 sub-clause F requires
/// the descendant to re-attest the witness rather than silently
/// extending the ancestor's trust.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InheritedUnaddressed {
    /// The inherited presentation that lacks re-attestation.
    pub presentation: crate::scan::Presentation,
    /// The behavioral-tier audit hint per ADR-018 `§"AuditHint integration"`:
    /// `inherited-presentation-not-re-attested`. Behavioral re-validation
    /// (does the ancestor's witness actually apply to descendant?) is
    /// A4-A5 work; reachability-tier audit cannot perform this check.
    pub audit_hint: AuditHint,
}

/// Aggregate audit report for a workspace.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditReport {
    /// Per-immunity audit results.
    pub audits: Vec<ImmunityAudit>,
    /// Number of immunities whose witness resolved cleanly.
    pub resolved_count: usize,
    /// Number of immunities whose witness defers to an external tool.
    pub external_count: usize,
    /// Number of immunities whose witness name resolves ambiguously
    /// (multiple workspace functions share the name). Per ATK-A2-005.
    pub ambiguous_count: usize,
    /// Number of immunities whose witness was not found.
    pub broken_count: usize,
    /// Number of immunities with no witness identifier at all.
    pub missing_count: usize,
    /// Inherited Presentations on a descendant that have no matching
    /// Immunity or Toleration on the same site (state 7 of the 7-state
    /// interaction matrix, ADR-018). Audit emits warn-level diagnostics
    /// for each; `--strict` promotes to error.
    #[serde(default)]
    pub inherited_unaddressed: Vec<InheritedUnaddressed>,
    /// Per-presents-site immune-state verdicts (ADR-029: Immunity Is Observed,
    /// Not Declared). Each `#[presents(X)]` site is cross-referenced against the
    /// `#[defended_by(X)]` code-tier witnesses + site-attached evidence and
    /// graded `defended` / `undefended` / `substrate-gap`. This is the audit's
    /// authoritative voice on immune state — no code site ever *claims*
    /// immunity; the audit *observes* it.
    ///
    /// `#[serde(default)]` so pre-ADR-029 serialized reports deserialize cleanly.
    #[serde(default)]
    pub presentation_verdicts: Vec<PresentationVerdict>,
}

impl AuditReport {
    /// True if all immunity claims meet at least Execution tier
    /// (per `is_well_formed`). Per ADR-005 Amendment 3, a Reachability-tier
    /// witness is NOT a well-formed claim — it has zero confirmed evidence.
    #[must_use]
    pub fn all_valid(&self) -> bool {
        self.audits.iter().all(ImmunityAudit::is_well_formed)
    }

    /// True if all immunity claims meet the given minimum tier. Used by
    /// `cargo antigen audit --min-tier <tier>` for CI gating.
    #[must_use]
    pub fn all_meet_tier(&self, minimum: WitnessTier) -> bool {
        self.audits.iter().all(|a| a.meets_tier(minimum))
    }

    /// Returns audits whose witness status indicates a problem.
    #[must_use]
    pub fn problematic_audits(&self) -> Vec<&ImmunityAudit> {
        self.audits.iter().filter(|a| !a.is_well_formed()).collect()
    }

    /// Per-presents-site verdicts that the audit graded `undefended` (ADR-029).
    /// These are the presents-sites with no registered code-tier witness and no
    /// passing site-attached evidence — the sites a CI gate should fail on.
    #[must_use]
    pub fn undefended_verdicts(&self) -> Vec<&PresentationVerdict> {
        self.presentation_verdicts
            .iter()
            .filter(|v| matches!(v.verdict, ImmuneVerdict::Undefended))
            .collect()
    }
}

/// The audit's immune-state verdict for one presents-site (ADR-029).
///
/// Immunity is *observed, not declared*: this verdict is computed by
/// `cargo antigen audit` cross-referencing the presents-site against the
/// `#[defended_by(X)]` witnesses + site-attached evidence in scope. It is
/// never asserted by a code site. The verdict describes *the state of the
/// defense circuit*, not whether the failure mode can fire.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "verdict", rename_all = "kebab-case")]
pub enum ImmuneVerdict {
    /// ≥1 witness defends this failure-class. `tier` is the strongest evidence
    /// tier observed across the registered witnesses (code-tier `#[defended_by]`
    /// witnesses grade `Reachability` until coverage-confirmed; substrate-tier
    /// `requires=` predicates that pass grade `Execution`; phantom-tier `proof=`
    /// grades `FormalProof`).
    Defended {
        /// Strongest evidence tier observed across the defending witnesses.
        tier: WitnessTier,
    },
    /// No witness defends this failure-class: no `#[defended_by(X)]` registration
    /// cross-references it and no passing site-attached evidence exists. This is
    /// the CI-gateable failure state.
    Undefended,
    /// Site-attached evidence (`requires=` substrate predicate) was declared but
    /// the current substrate does not satisfy it. The defense intent exists; the
    /// substrate has drifted out of compliance. Distinct from `undefended` (no
    /// intent at all) — this is "intent present, substrate gap."
    SubstrateGap,
}

/// The four-valued verdict for a prescriptive work-need (ADR-033 §Decision 3).
///
/// This is the [`ImmuneVerdict`] tri-state with the unsatisfied cell *temporally
/// split by the frame* (the verdict-lattice isomorphism, math-researcher): the
/// prescriptive evaluator REUSES the ADR-029 satisfaction read and applies a
/// frame-aware projection — it does NOT fork a parallel evaluator (forking
/// re-introduces the cardinality-collapse the three-valued-logic gem warns
/// against). `Undefended` splits into `Pending` (within frame) + `Overdue` (past
/// frame); `SubstrateGap` maps to `OutOfFrame`; `Defended` maps to `Fulfilled`.
///
/// **`Overdue` and `OutOfFrame` must NEVER collapse** (ATK-PRES-8, the load-bearing
/// guard, the prescriptive analog of ATK-3V-4): `Overdue` means the frame elapsed
/// AND the audit evaluated the satisfaction and found it unmet (it KNOWS the work
/// is late); `OutOfFrame` means the satisfaction is un-evaluable (an unknown
/// who-ref, an unresolvable code-site ref) — the audit cannot even tell whether
/// the work is done. Different states, different interventions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkVerdict {
    /// Declared, within frame, satisfaction not yet met. The EXPECTED state — not
    /// a failure, must not be loud.
    Pending,
    /// Satisfaction met at the current fingerprint.
    Fulfilled,
    /// Past the frame and unsatisfied (and evaluable). **Loud** (ADR-023 loudness
    /// isomorphism).
    Overdue,
    /// The satisfaction condition is un-evaluable in the current substrate (an
    /// unknown who-ref, a missing source, an unresolvable code-site reference).
    /// ADR-029 Amendment 1 well-posedness — outside the frame, NOT overdue.
    OutOfFrame,
}

/// Whether a work-need's intrinsic temporal frame has elapsed, relative to a
/// reference date. Pure input to the [`WorkVerdict`] projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameState {
    /// No frame declared — the need has no deadline (timeless until satisfied).
    None,
    /// A frame is declared and the reference date is on or before it.
    Within,
    /// A frame is declared and the reference date is past it.
    Past,
    /// A frame string was declared but is not a parseable ISO-8601 date — the
    /// frame is un-evaluable (feeds `OutOfFrame`, never a silent pass).
    Unparseable,
}

impl FrameState {
    /// Classify an optional ISO-8601 frame string against a reference date.
    /// Absent ⇒ [`FrameState::None`]; present-and-parseable ⇒ `Within`/`Past`;
    /// present-but-malformed ⇒ [`FrameState::Unparseable`] (tier-honest — a
    /// garbage date is not silently "within frame").
    #[must_use]
    pub fn classify(frame: Option<&str>, today: chrono::NaiveDate) -> Self {
        frame.map_or(Self::None, |s| {
            match chrono::NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d") {
                Ok(d) if d >= today => Self::Within,
                Ok(_) => Self::Past,
                Err(_) => Self::Unparseable,
            }
        })
    }
}

impl WorkVerdict {
    /// Project the four-valued verdict from a satisfaction state + a frame state
    /// — the ADR-033 §Decision 3 isomorphism made explicit.
    ///
    /// `satisfied` is the ADR-029 categorical read (the who-steps / closure
    /// attested at the current fingerprint). `evaluable` is whether the audit
    /// could evaluate satisfaction at all (false ⇒ `OutOfFrame`, the gem guard).
    ///
    /// Truth table (the load-bearing distinction is the last two rows — they
    /// MUST differ):
    /// - not evaluable, any frame      → `OutOfFrame`
    /// - satisfied (evaluable)          → `Fulfilled`
    /// - unsatisfied, frame Within/None → `Pending`
    /// - unsatisfied, frame Past        → `Overdue`
    /// - unsatisfied, frame Unparseable → `OutOfFrame` (un-evaluable frame)
    #[must_use]
    pub const fn project(satisfied: bool, evaluable: bool, frame: FrameState) -> Self {
        if !evaluable {
            return Self::OutOfFrame;
        }
        if satisfied {
            return Self::Fulfilled;
        }
        match frame {
            FrameState::Past => Self::Overdue,
            FrameState::Within | FrameState::None => Self::Pending,
            // An unparseable frame on an unsatisfied need is un-evaluable: we
            // cannot say "late" because we cannot read the deadline. Keep it
            // OutOfFrame rather than silently Pending or falsely Overdue.
            FrameState::Unparseable => Self::OutOfFrame,
        }
    }

    /// True for the one LOUD verdict — `Overdue` (ADR-023 loudness isomorphism).
    /// `Pending` is expected (quiet); `OutOfFrame` is advisory (needs
    /// investigation, not alarm); `Fulfilled` is clean.
    #[must_use]
    pub const fn is_loud(self) -> bool {
        matches!(self, Self::Overdue)
    }
}

/// The audit's per-presents-site immune-state verdict record (ADR-029).
///
/// Pairs a presents-site with the verdict the audit computed for it and the
/// witnesses that contributed. The `defended_by` field names the source-locations
/// of the code-tier witnesses that cross-referenced this site (for the report's
/// "defended at <file:line>" rendering).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresentationVerdict {
    /// The presents-site being graded.
    pub presentation: crate::scan::Presentation,
    /// The failure-class this verdict is about (the presents-site's antigen).
    pub antigen_type: String,
    /// The computed immune-state verdict.
    pub verdict: ImmuneVerdict,
    /// Source locations (`<file>:<line>`) of the code-tier `#[defended_by(X)]`
    /// witnesses that cross-referenced this site. Empty for `undefended` /
    /// `substrate-gap` and for verdicts defended solely by site-attached evidence.
    #[serde(default)]
    pub defended_by: Vec<String>,
}

/// Default workspace floor for `#[clonal]` iterations. Configurable via
/// `[package.metadata.antigen.clonal_iterations_floor]` in a future
/// amendment.
pub const CLONAL_ITERATIONS_DEFAULT_FLOOR: u64 = 100;

/// Default workspace floor for `#[igg]` historical span (days).
pub const IGG_HISTORICAL_SPAN_DEFAULT_FLOOR: u64 = 30;
