//! Witness validation and immunity audit.
//!
//! The audit module operates a layer above [`crate::scan`]: where scan finds
//! antigen-related declarations as syntactic facts, audit reasons about whether
//! the immunity claims are actually backed by working witnesses.
//!
//! This is the "trust-boundary check" required by ADR-005 (sub-clause F at every
//! trust boundary). A declaration of `#[immune(X, witness = Y)]` is meaningful
//! only if `Y` resolves to a real function, test, lint reference, or proof that
//! demonstrates immunity. A marker without a working witness is not a claim.
//!
//! ## What audit checks (v0.0.1)
//!
//! - Witness identifiers resolve to a function/test in the workspace
//! - Witness functions have a recognized testing attribute (`#[test]`, recognizable
//!   `proptest!` invocation, or known external delegations like `clippy::lint_name`)
//!
//! ## What audit doesn't check (yet)
//!
//! - **Witness execution**: doesn't actually run the test/proptest. The team
//!   should add `cargo test` integration in sweep A3+.
//! - **Witness semantics**: doesn't verify the witness asserts the antigen's
//!   specific failure pattern. That requires fingerprint-aware reasoning.
//! - **External tool delegation**: clippy/kani/prusti adapters are stubbed with
//!   "external; manual validation required" status. Sweep A3+ adds adapters.
//! - **Cross-crate witnesses**: a witness that lives in a dependency isn't
//!   followed. v0.0.1 audit is workspace-local only — A3 sweep extends this
//!   via cross-crate source walking (per scope-lock).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::scan::{Immunity, ScanReport};

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
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum WitnessTier {
    /// No witness or unresolved witness. Immunity asserted without evidence.
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
    /// `#[orient]` past its deadline (if `until` added in future versions).
    /// Currently `#[orient]` does not require `until`, so this is reserved.
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
    /// the configured floor (default: 2).
    PolyclonalInsufficientLineages,
    /// `#[adcc]` site has only one of the two committed mechanisms
    /// (antibody-style + cellular-effector-style) detectable.
    AdccSingleMechanismOnly,
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
}

// ============================================================================
// Deferred-Defense Family audit (ADR-023)
// ============================================================================

/// Audit result for a single deferred-defense declaration.
///
/// Each deferred-defense site is evaluated against the current UTC date
/// and the relevant workspace config caps to produce a hint that reflects
/// its current state in the loudness-as-discipline lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeferredDefenseAudit {
    /// The original deferred-defense declaration from the scan.
    pub declaration: crate::scan::DeferredDefense,
    /// The hint code reflecting this declaration's current state.
    pub hint: AuditHint,
}

/// Aggregate deferred-defense audit report.
///
/// Consumed by `cargo antigen defer status` and `cargo antigen audit`
/// to surface the loudness-as-discipline state of all deferred defenses.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeferredDefenseAuditReport {
    /// Per-declaration audit results.
    pub audits: Vec<DeferredDefenseAudit>,
    /// Count of active (not yet expired) deferred defenses.
    pub active_count: usize,
    /// Count of expired deferred defenses past their `until` date.
    pub expired_count: usize,
    /// Count of stale deferred defenses (significantly past `until`).
    pub stale_count: usize,
}

/// Evaluate all deferred-defense declarations in a `ScanReport` against
/// the current UTC date, producing a `DeferredDefenseAuditReport`.
///
/// This is the v0.2 audit implementation. All date comparisons use UTC
/// per ADR-023 §Enforcement-Surface.
///
/// `stale_grace_days`: how many days past `until` before `anergy-stale`
/// (vs `anergy-co-stimulation-not-arrived`). Default 30 days.
#[must_use]
pub fn audit_deferred_defenses(
    scan: &crate::scan::ScanReport,
    stale_grace_days: i64,
) -> DeferredDefenseAuditReport {
    use chrono::Utc;

    let today = Utc::now().date_naive();
    let mut audits = Vec::new();
    let mut active_count = 0usize;
    let mut expired_count = 0usize;
    let mut stale_count = 0usize;

    for decl in &scan.deferred_defenses {
        let hint = evaluate_deferred_defense_hint(decl, today, stale_grace_days);

        // Tally
        match &hint {
            AuditHint::AnergyActive
            | AuditHint::ImmunosuppressActive
            | AuditHint::PoxpartyActive
            | AuditHint::OrientActive => {
                active_count += 1;
            }
            AuditHint::AnergyCostimulationNotArrived
            | AuditHint::ImmunosuppressExpired
            | AuditHint::PoxpartyOutcomePending => {
                expired_count += 1;
            }
            AuditHint::AnergyStale => {
                stale_count += 1;
            }
            _ => {}
        }

        audits.push(DeferredDefenseAudit {
            declaration: decl.clone(),
            hint,
        });
    }

    DeferredDefenseAuditReport {
        audits,
        active_count,
        expired_count,
        stale_count,
    }
}

/// Derive the `AuditHint` for a single deferred-defense declaration.
///
/// UTC date comparison throughout per ADR-023.
fn evaluate_deferred_defense_hint(
    decl: &crate::scan::DeferredDefense,
    today: chrono::NaiveDate,
    stale_grace_days: i64,
) -> AuditHint {
    use crate::scan::DeferredDefenseKind;

    match &decl.kind {
        DeferredDefenseKind::Anergy => {
            match parse_iso_date(decl.until.as_deref().unwrap_or("")) {
                Some(until) if until >= today => AuditHint::AnergyActive,
                Some(until) => {
                    let days_past = (today - until).num_days();
                    if days_past > stale_grace_days {
                        AuditHint::AnergyStale
                    } else {
                        AuditHint::AnergyCostimulationNotArrived
                    }
                }
                None => AuditHint::AnergyActive, // No parseable date = treat as active
            }
        }
        DeferredDefenseKind::Immunosuppress => {
            match parse_iso_date(decl.until.as_deref().unwrap_or("")) {
                Some(until) if until >= today => AuditHint::ImmunosuppressActive,
                Some(_) => AuditHint::ImmunosuppressExpired,
                None => AuditHint::ImmunosuppressActive,
            }
        }
        DeferredDefenseKind::Poxparty => {
            match parse_iso_date(decl.until.as_deref().unwrap_or("")) {
                Some(until) if until >= today => AuditHint::PoxpartyActive,
                Some(_) => AuditHint::PoxpartyOutcomePending,
                None => AuditHint::PoxpartyActive,
            }
        }
        DeferredDefenseKind::Orient => AuditHint::OrientActive,
    }
}

/// Parse an ISO-8601 date string for audit-time UTC comparison.
/// Returns `None` if the string is not a valid date.
fn parse_iso_date(s: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

/// Filesystem-backed [`antigen_attestation::EvaluationContext`] for use
/// during real audit runs. Reads docs and oracles directly from disk; reads
/// git trailers by shelling out to `git interpret-trailers`. Tests in
/// `antigen-attestation` use an in-memory context instead (see
/// `evaluate.rs` `TestContext`).
struct FilesystemAuditContext;

impl antigen_attestation::EvaluationContext for FilesystemAuditContext {
    fn today(&self) -> chrono::NaiveDate {
        chrono::Local::now().date_naive()
    }

    fn read_doc(&self, path: &std::path::Path) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }

    fn read_oracle(&self, path: &std::path::Path) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }

    // v0.1: git trailers require subprocess + git; returns empty vec when
    // git is unavailable or the item has no commits. A3+ work wires this
    // to a proper git2 adapter or subprocess; for now the trait contract
    // is satisfied and `SignedTrailer` leaf evaluates to false.
    fn read_git_trailers(
        &self,
        _item_source_file: &std::path::Path,
        _item_path: &str,
    ) -> Vec<String> {
        Vec::new()
    }
}

/// Attempt to load and deserialize a `.attest/<antigen_name>.json` sidecar
/// for the given immunity. Returns `None` when the file doesn't exist or
/// fails to deserialize (both are treated as sidecar-missing by the evaluator).
fn load_sidecar(
    immunity_file: &Path,
    antigen_type: &str,
) -> Option<antigen_attestation::Ratification> {
    let dir = immunity_file.parent()?;
    // Antigen type may be a fully-qualified path (`crate::antigens::SomeAntigen`);
    // use only the last segment as the filename component for v0.1 convention.
    let stem = antigen_type.rsplit("::").next().unwrap_or(antigen_type);
    let sidecar_path = dir.join(".attest").join(format!("{stem}.json"));
    let content = std::fs::read_to_string(&sidecar_path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Run audit against a [`ScanReport`].
///
/// For each immunity declaration, attempts to validate the witness identifier
/// by walking the workspace looking for the function it names.
///
/// `workspace_root` is used to look for witness functions; passing the same
/// path used for [`crate::scan::scan_workspace`] is typical.
///
/// Files that fail to parse during the function-index walk are silently
/// skipped (matching `scan_workspace`'s behavior); this function does not
/// itself surface IO errors to the caller.
#[must_use]
pub fn audit(report: &ScanReport, workspace_root: &Path) -> AuditReport {
    let workspace_functions = collect_function_index(workspace_root);

    let mut audits = Vec::new();
    for immunity in &report.immunities {
        // Substrate-witness path (ADR-019 P3c) when `requires = <predicate>` was
        // declared; code-witness path otherwise (validate witness identifier via
        // function index).
        let immunity_audit = immunity.requires_predicate.as_ref().map_or_else(
            || {
                let status = validate_witness(&immunity.witness, &workspace_functions);
                let witness_tier = WitnessTier::from_status(&status);
                let audit_hint = AuditHint::from_status(&status);
                let evidence_kind = evidence_kind_from_status(&status);
                ImmunityAudit {
                    immunity: immunity.clone(),
                    witness_status: status,
                    witness_tier,
                    audit_hint,
                    evidence_kind,
                    signature_strength: None,
                    compound_evidence: false,
                    evaluated_predicate: None,
                }
            },
            |predicate_json| audit_substrate_witness(immunity, predicate_json),
        );
        audits.push(immunity_audit);
    }

    let mut audit_report = AuditReport {
        audits,
        ..AuditReport::default()
    };
    for a in &audit_report.audits {
        match &a.witness_status {
            WitnessStatus::Resolved { .. } => audit_report.resolved_count += 1,
            WitnessStatus::External { .. } => audit_report.external_count += 1,
            WitnessStatus::Ambiguous { .. } => audit_report.ambiguous_count += 1,
            WitnessStatus::NotFound { .. } => audit_report.broken_count += 1,
            WitnessStatus::Missing => audit_report.missing_count += 1,
        }
    }

    // State 7 detection (ADR-018 §"7-state interaction matrix"): an
    // inherited Presentation (inherited_from = Some(_)) without matching
    // Immunity or Toleration on the descendant site is unaddressed.
    // `unaddressed_presentations()` already encodes the "no matching
    // immune/tolerance" check; we filter its output to the inherited
    // subset.
    for u in report.unaddressed_presentations() {
        if u.presentation.inherited_from.is_some() {
            audit_report
                .inherited_unaddressed
                .push(InheritedUnaddressed {
                    presentation: u.presentation,
                    audit_hint: AuditHint::InheritedPresentationNotReAttested,
                });
        }
    }

    audit_report
}

/// Evaluate a substrate-witness predicate for one immunity declaration and
/// return the populated [`ImmunityAudit`].
///
/// Called from `audit()` when `immunity.requires_predicate` is `Some`.
fn audit_substrate_witness(immunity: &Immunity, predicate_json: &str) -> ImmunityAudit {
    use antigen_attestation::evaluate::evaluate_predicate_with_kind;

    // Deserialize the predicate. The JSON was emitted at macro-expand time by
    // `antigen-macros` and round-trips through the doc marker; any failure here
    // means the marker was corrupted in transit (shouldn't happen, but we surface
    // it as sidecar-schema-invalid rather than panicking).
    let Ok(predicate) = serde_json::from_str::<antigen_attestation::Predicate>(predicate_json)
    else {
        let result = antigen_attestation::EvaluatedPredicate::sidecar_schema_invalid();
        // Sidecar not yet loaded; default kind to Immunity (kind field on this path is unused).
        return immunity_audit_from_evaluated(
            immunity,
            result,
            predicate_json.to_string(),
            antigen_attestation::RatificationKind::Immunity,
        );
    };

    // Load the sidecar. Missing → sidecar_missing result.
    let Some(sidecar) = load_sidecar(&immunity.file, &immunity.antigen_type) else {
        let result = antigen_attestation::EvaluatedPredicate::sidecar_missing();
        return immunity_audit_from_evaluated(
            immunity,
            result,
            predicate_json.to_string(),
            antigen_attestation::RatificationKind::Immunity,
        );
    };

    // v0.1: use the first item in the sidecar's items list as the evaluation
    // target. A3+ work will match by item_path (function path) to support
    // per-item predicates when multiple items share an antigen sidecar.
    let Some(item) = sidecar.items.first() else {
        let result = antigen_attestation::EvaluatedPredicate::sidecar_missing();
        return immunity_audit_from_evaluated(
            immunity,
            result,
            predicate_json.to_string(),
            sidecar.kind,
        );
    };

    let ctx = FilesystemAuditContext;
    // Audit-SF-1: stale-signer detection is SELF-REFERENTIAL in v0.1.
    //
    // `current_fingerprint` here is the sidecar's stored value, not the real
    // fingerprint of the item as it exists on disk right now. The evaluator
    // compares `s.signed_against_fingerprint == current_fingerprint` — both
    // sides come from the same sidecar, so stale signers always appear current
    // in `cargo antigen audit`.
    //
    // To detect real staleness: use `cargo antigen attest check --fingerprint <fp>`
    // with the fingerprint from `cargo antigen scan --format json`.
    //
    // The correct fix (A3 work): integrate antigen-fingerprint recomputation at
    // audit time so the real current fingerprint can be compared against sidecar
    // entries. Until then, stale detection only fires when the sidecar was written
    // with a fingerprint that differs from another sidecar entry — within-sidecar
    // consistency is maintained but real-code-change drift is not detected.
    let current_fingerprint = &item.current_fingerprint;
    let result = evaluate_predicate_with_kind(
        &predicate,
        item,
        current_fingerprint,
        &immunity.file,
        sidecar.kind,
        &ctx,
    )
    .unwrap_or_else(|_| antigen_attestation::EvaluatedPredicate::sidecar_schema_invalid());

    immunity_audit_from_evaluated(immunity, result, predicate_json.to_string(), sidecar.kind)
}

/// Build an [`ImmunityAudit`] from the output of `evaluate_predicate_with_kind`.
fn immunity_audit_from_evaluated(
    immunity: &Immunity,
    result: antigen_attestation::EvaluatedPredicate,
    predicate_json: String,
    sidecar_kind: antigen_attestation::RatificationKind,
) -> ImmunityAudit {
    let status = WitnessStatus::Resolved {
        location: immunity.file.clone(),
        witness_kind: WitnessKind::SubstrateWitness { kind: sidecar_kind },
    };
    ImmunityAudit {
        immunity: immunity.clone(),
        witness_status: status,
        witness_tier: map_attestation_tier(result.witness_tier),
        audit_hint: map_attestation_audit_hint(result.audit_hint),
        evidence_kind: result.evidence_kind,
        signature_strength: result.signature_strength,
        compound_evidence: false,
        evaluated_predicate: Some(predicate_json),
    }
}

/// Map [`antigen_attestation::WitnessTier`] to [`WitnessTier`].
///
/// The two enums are structurally identical (defined in lock-step per `tier.rs`)
/// but are distinct types to avoid a circular crate dependency. For v0.1 this
/// mapping is lossless; a future ADR that diverges the two enums would widen it.
const fn map_attestation_tier(tier: antigen_attestation::WitnessTier) -> WitnessTier {
    match tier {
        antigen_attestation::WitnessTier::None => WitnessTier::None,
        antigen_attestation::WitnessTier::Reachability => WitnessTier::Reachability,
        antigen_attestation::WitnessTier::Execution => WitnessTier::Execution,
        antigen_attestation::WitnessTier::FormalProof => WitnessTier::FormalProof,
    }
}

/// Map [`antigen_attestation::AuditHint`] to [`AuditHint`].
///
/// 1:1 mapping — every substrate-witness hint variant in the attestation
/// crate has a peer in [`AuditHint`]. The two enums are deliberately
/// duplicated so the runtime crate stays serde-stable for the on-disk
/// audit format while the attestation crate can evolve the substrate-
/// witness vocabulary independently.
///
/// rc.1 collapsed substrate hints into `NoneApplicable` /
/// `ExternalToolPrefixRecognized`, which made it impossible to read what
/// the substrate-witness pipeline actually found. rc.2 surfaces the real
/// state — the hint a CI gate or reviewer reads now names the case.
const fn map_attestation_audit_hint(hint: antigen_attestation::AuditHint) -> AuditHint {
    use antigen_attestation::AuditHint as AH;
    match hint {
        AH::DisciplineSidecarMissing => AuditHint::DisciplineSidecarMissing,
        AH::DisciplineSidecarSchemaInvalid => AuditHint::DisciplineSidecarSchemaInvalid,
        AH::DisciplinePredicateFailed => AuditHint::DisciplinePredicateFailed,
        AH::DisciplineSubstrateStale => AuditHint::DisciplineSubstrateStale,
        AH::DisciplineSubstrateDeltaChainNearCap => AuditHint::DisciplineSubstrateDeltaChainNearCap,
        AH::DisciplinePredicatePassedViaDeltaChain => {
            AuditHint::DisciplinePredicatePassedViaDeltaChain
        }
        AH::DisciplinePredicatePassedSubstrateCurrent => {
            AuditHint::DisciplinePredicatePassedSubstrateCurrent
        }
        AH::ToleranceVibesGrade => AuditHint::ToleranceVibesGrade,
        AH::ToleranceSidecarMissing => AuditHint::ToleranceSidecarMissing,
        AH::TolerancePredicateFailed => AuditHint::TolerancePredicateFailed,
        AH::TolerancePredicatePassedSubstrateCurrent => {
            AuditHint::TolerancePredicatePassedSubstrateCurrent
        }
        AH::DisciplineSidecarKindMismatchExpectedImmunityGotTolerance => {
            AuditHint::DisciplineSidecarKindMismatchExpectedImmunityGotTolerance
        }
        AH::ToleranceSidecarKindMismatchExpectedToleranceGotImmunity => {
            AuditHint::ToleranceSidecarKindMismatchExpectedToleranceGotImmunity
        }
        AH::DisciplineImmunityToleranceContradiction => {
            AuditHint::DisciplineImmunityToleranceContradiction
        }
    }
}

/// One entry in the function index — a single (path, kind) pair for a name.
#[derive(Debug, Clone)]
struct FunctionEntry {
    location: PathBuf,
    kind: WitnessKind,
}

/// Index of function name → all (file path, kind) pairs sharing that name.
///
/// W7 (A2) extends the flat name index to track *all* candidates for a name,
/// so `validate_witness` can detect ambiguity (ATK-A2-005). When more than one
/// function shares a name, the witness resolves to `WitnessStatus::Ambiguous`
/// rather than silently picking whichever was indexed last.
///
/// Cross-cutting limitations remaining for A3+:
/// - Module-qualified paths (`crate::foo::bar` parsing) require module-graph
///   resolution; for v0.1 we detect ambiguity and require the user to qualify
///   the witness (e.g., rename one of the conflicting functions, or use a
///   path that is unique).
/// - Functions inside `impl` blocks (method names, not free functions);
///   currently recorded with the same shape — matching is name-only.
type FunctionIndex = std::collections::HashMap<String, Vec<FunctionEntry>>;

fn collect_function_index(root: &Path) -> FunctionIndex {
    use syn::visit::Visit;
    use walkdir::WalkDir;

    let exclusions = ["target", ".git", "node_modules"];
    let mut index = FunctionIndex::new();

    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                !exclusions.iter().any(|x| *x == name)
            } else {
                true
            }
        })
    {
        let Ok(entry) = entry else { continue };
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(entry.path()) else {
            continue;
        };

        if let Ok(file) = syn::parse_file(&content) {
            let mut visitor = FunctionIndexVisitor {
                file_path: entry.path().to_path_buf(),
                source: &content,
                index: &mut index,
            };
            visitor.visit_file(&file);
        }
    }

    index
}

struct FunctionIndexVisitor<'a> {
    file_path: PathBuf,
    /// Source text of the file being walked. Carried for symmetry with
    /// `scan::ScanVisitor` and for future span-anchored diagnostics; the
    /// pre-W5 textual `source.contains("proptest!")` sentinel was removed
    /// when `visit_macro` took over proptest classification.
    #[allow(
        dead_code,
        reason = "reserved for span-anchored diagnostic work \
        that mirrors scan::ScanVisitor::source"
    )]
    source: &'a str,
    index: &'a mut FunctionIndex,
}

impl FunctionIndexVisitor<'_> {
    /// Classify a function by its own attributes.
    ///
    /// W5 (sweep A2): the prior heuristic — `self.source.contains("proptest!")`
    /// — over-classified every function in any file mentioning the string
    /// `proptest!` (including doc comments) as `WitnessKind::Proptest`.
    /// Replaced by structural detection: `visit_macro` registers
    /// proptest-internal function names with `WitnessKind::Proptest` directly.
    ///
    /// W7 (sweep A2): distinguish `#[test] #[ignore]` from a running `#[test]`.
    /// Per ADR-005 Amendment 3 and ATK-A2-012, an ignored test is weaker
    /// evidence than a runnable test — `cargo test` skips it by default.
    /// We tag it as `WitnessKind::IgnoredTest` so the audit can emit the
    /// `TestAttributePresentIgnoreSkipped` hint.
    fn detect_kind(attrs: &[syn::Attribute]) -> WitnessKind {
        let has_test = attrs.iter().any(|a| a.path().is_ident("test"));
        let has_ignore = attrs.iter().any(|a| a.path().is_ident("ignore"));
        match (has_test, has_ignore) {
            (true, true) => WitnessKind::IgnoredTest,
            (true, false) => WitnessKind::Test,
            (false, _) => WitnessKind::Function,
        }
    }
}

/// Extract top-level `fn IDENT` names from a `proptest! { ... }` macro body.
///
/// `proptest!` is a function-like macro that takes a sequence of test-shaped
/// declarations:
///
/// ```ignore
/// proptest! {
///     #[test]
///     fn name(args in strategy) { body }
///     ...
/// }
/// ```
///
/// The body's tokens contain `fn IDENT` at the top level for each test;
/// nested function definitions live inside `Group` tokens (the body block of
/// each fn) which a top-level token-iterator does not descend into. So a
/// linear walk that yields `name` whenever it sees `fn` followed by an
/// identifier captures exactly the proptest test names — no more, no less.
///
/// Why not parse with `syn` directly? `proptest!`'s grammar (`fn name(args
/// in strategy)`) is not a valid Rust function signature: the `in` keyword
/// inside the parameter list is custom syntax. `syn::ItemFn::parse` rejects
/// the body. The token walk below is grammar-aware enough for our purpose
/// (extracting names) without committing to parsing the strategy expressions.
fn extract_proptest_fn_names(tokens: &proc_macro2::TokenStream) -> Vec<String> {
    use proc_macro2::TokenTree;
    let mut names = Vec::new();
    let mut iter = tokens.clone().into_iter();
    while let Some(tt) = iter.next() {
        if let TokenTree::Ident(i) = &tt {
            if i == "fn" {
                if let Some(TokenTree::Ident(name)) = iter.next() {
                    names.push(name.to_string());
                }
            }
        }
    }
    names
}

/// Whether a macro path's last segment is `name`. Mirrors the
/// `attr_is`-style test in `scan.rs`: matches both `#[proptest!(...)]`-style
/// bare names and `proptest::proptest!(...)` path-qualified forms.
fn macro_path_last_is(path: &syn::Path, name: &str) -> bool {
    path.segments.last().is_some_and(|s| s.ident == name)
}

impl FunctionIndexVisitor<'_> {
    /// Push a candidate entry for `name`. W7: every distinct (location, kind)
    /// pair is preserved so `validate_witness` can detect ambiguity. The
    /// pre-W7 behavior (silent first-wins) is the bug ATK-A2-005 named.
    fn push(&mut self, name: String, kind: WitnessKind) {
        self.index.entry(name).or_default().push(FunctionEntry {
            location: self.file_path.clone(),
            kind,
        });
    }
}

impl<'ast> syn::visit::Visit<'ast> for FunctionIndexVisitor<'_> {
    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        let name = item.sig.ident.to_string();
        let kind = Self::detect_kind(&item.attrs);
        self.push(name, kind);
        syn::visit::visit_item_fn(self, item);
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        let name = item.sig.ident.to_string();
        let kind = Self::detect_kind(&item.attrs);
        self.push(name, kind);
        syn::visit::visit_impl_item_fn(self, item);
    }

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        // W5: structural proptest! detection. When the macro path's last
        // segment is `proptest`, walk its tokens for `fn IDENT` patterns
        // and register each name with `WitnessKind::Proptest`.
        if macro_path_last_is(&mac.path, "proptest") {
            for name in extract_proptest_fn_names(&mac.tokens) {
                self.push(name, WitnessKind::Proptest);
            }
        }
        syn::visit::visit_macro(self, mac);
    }
}

/// Determine the witness status for a single witness identifier string.
///
/// Resolution priority per ADR-013 + ADR-005 Amendment 3:
/// 1. Empty witness → `Missing`
/// 2. External-tool prefix (`clippy::`, `kani::`, ...) → `External`
/// 3. Phantom-type witness shape → `Resolved { PhantomType }`
/// 4. Workspace function lookup → `Resolved` / `Ambiguous` / `NotFound`
fn validate_witness(witness: &str, index: &FunctionIndex) -> WitnessStatus {
    // Normalize whitespace: the scan path records witnesses via ToTokens, which
    // emits spaced token form (`clippy :: no_panic_in_drop`, `PolarityProof :: < T > :: verified`).
    // Collapse all spacing around `::` and `<>` so every downstream detector
    // works on compact form regardless of source (hand-written or scan-path).
    let normalized_owned: String = {
        let collapsed = witness.split_whitespace().collect::<Vec<_>>().join(" ");
        collapsed
            .replace(" :: ", "::")
            .replace(":: ", "::")
            .replace(" ::", "::")
            .replace("< ", "<")
            .replace(" >", ">")
    };
    let trimmed = normalized_owned.trim();
    if trimmed.is_empty() {
        return WitnessStatus::Missing;
    }

    // Detect external-tool delegations.
    if let Some(tool) = detect_external_tool(trimmed) {
        return WitnessStatus::External {
            tool_hint: tool.to_string(),
        };
    }

    // Detect phantom-type witness shapes (ADR-013): `Path::<Args>::ctor` or
    // `Path::<Args>` or `Path` with trailing `()`. The shape recognition is
    // structural — we don't validate that the type exists.
    if let Some(phantom) = detect_phantom_type_witness(trimmed) {
        return WitnessStatus::Resolved {
            location: PathBuf::new(),
            witness_kind: phantom,
        };
    }

    // Resolve as a workspace-local function. The witness might be a path
    // (`module::function`); take the last segment as the function name.
    let function_name = trimmed
        .rsplit("::")
        .next()
        .unwrap_or(trimmed)
        .trim_end_matches("()")
        .trim();

    let candidates = index.get(function_name);
    let Some(candidates) = candidates else {
        return WitnessStatus::NotFound {
            reason: format!(
                "no function named `{function_name}` found in any .rs file under the scan root"
            ),
        };
    };

    match candidates.as_slice() {
        [] => WitnessStatus::NotFound {
            reason: format!(
                "no function named `{function_name}` found in any .rs file under the scan root"
            ),
        },
        [only] => WitnessStatus::Resolved {
            location: only.location.clone(),
            witness_kind: only.kind.clone(),
        },
        many => WitnessStatus::Ambiguous {
            candidates: many.iter().map(|e| e.location.clone()).collect(),
        },
    }
}

/// Recognize a phantom-type witness shape per ADR-013.
///
/// Matches: `Type`, `Type::ctor`, `Type::<Args>`, `Type::<Args>::ctor`,
/// optionally with trailing `()`. The `<Args>` group, when present, contains
/// comma-separated type parameters.
///
/// We deliberately accept *any* type-name shape (capital-leading identifier
/// path) here because v0.1's audit has no symbol table — we cannot tell
/// whether `PolarityProof` refers to a real type. The `audit_hint` carries
/// the warning to verify the constructor is sealed; that's the recognize-
/// and-warn discipline ADR-013 §OQ1 specifies.
///
/// Returns `None` when the witness looks more like a function path than a
/// phantom-type construction (lowercase final segment with no type-param
/// list and no trailing `()`-after-`::`-segment ambiguity). The heuristic:
/// if the path contains a `<...>` segment, OR the last segment starts with
/// an uppercase letter (typical Rust type-name convention) AND there are
/// no trailing `()` (which would indicate a function call), treat it as
/// a phantom-type witness candidate.
fn detect_phantom_type_witness(witness: &str) -> Option<WitnessKind> {
    // Input is pre-normalized by validate_witness — compact token spacing guaranteed.
    let trimmed = witness.trim().trim_end_matches("()").trim();
    let has_turbofish = trimmed.contains("::<");
    if !has_turbofish {
        // No turbofish = not a phantom-type witness shape we recognize. The
        // bare-type-name shape (`Foo`) is indistinguishable from a function
        // path at this layer; we let the function-index path handle it.
        return None;
    }

    // Split into pre-turbofish, type-params, post-turbofish-ctor.
    let (before, after) = trimmed.split_once("::<")?;
    let (params_raw, ctor_part) = after.split_once('>')?;

    // Guard: nested generics like `Foo::<Option<Bar>, Baz>::new` make
    // split_once('>') fire at the inner `>`, leaving params_raw with
    // unbalanced `<`. Return None rather than emit FormalProof for a
    // garbled parse — let it fall through to function-index (NotFound).
    let open_count = params_raw.chars().filter(|&c| c == '<').count();
    if open_count > 0 {
        return None;
    }

    let proof_type = before.trim().to_string();
    let type_params: Vec<String> = params_raw
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    // Strip any remaining closing `>`s and the `::` separator left by
    // split_once('>') on nested-generic inputs like `Foo::<Bar<Baz>>::new`.
    let constructor = ctor_part
        .trim_start_matches(['>', ':'])
        .trim()
        .trim_end_matches("()")
        .trim();
    let constructor = if constructor.is_empty() {
        None
    } else {
        Some(constructor.to_string())
    };

    Some(WitnessKind::PhantomType {
        proof_type,
        type_params,
        constructor,
    })
}

/// Detect whether the witness references an external tool we recognize.
fn detect_external_tool(witness: &str) -> Option<&'static str> {
    let lower = witness.to_ascii_lowercase();
    if lower.starts_with("clippy::") || lower.contains("clippy_") {
        Some("clippy")
    } else if lower.starts_with("kani::") || lower.contains("kani_proof") {
        Some("kani")
    } else if lower.starts_with("prusti::") {
        Some("prusti")
    } else if lower.starts_with("creusot::") {
        Some("creusot")
    } else if lower.starts_with("verus::") {
        Some("verus")
    } else if lower.starts_with("mutants::") {
        Some("cargo-mutants")
    } else {
        None
    }
}

// ============================================================================
// Supply-Chain Defense Family audit (ADR-025)
// ============================================================================

/// One result of evaluating a supply-chain witness leaf against a workspace.
///
/// Each entry carries the source-side identity (file + line where the
/// `#[immune(X, requires = <supply-chain-leaf>)]` was declared), the
/// antigen type it backs, and the [`AuditHint`] the evaluation produced.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyChainAudit {
    /// The antigen type the leaf is backing (`ContentHashMismatch`,
    /// `UnpinnedDependency`, etc.). Mirrors `Immunity::antigen_type`.
    pub antigen_type: String,
    /// Source file the immunity declaration lives in.
    pub file: PathBuf,
    /// Line number of the immunity attribute.
    pub line: usize,
    /// Crate name the leaf targeted (extracted from leaf args).
    pub crate_name: String,
    /// Crate version the leaf targeted (if applicable). Empty for
    /// leaves that don't take a version (e.g., `dep_pinned`).
    pub version: String,
    /// The audit hint the evaluation produced.
    pub hint: AuditHint,
    /// Optional structured detail (e.g., for `ContentHashMismatch`,
    /// the recorded vs current hash strings).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Aggregate result of [`audit_supply_chain`].
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SupplyChainAuditReport {
    /// Per-immunity audit entries (one per supply-chain leaf encountered).
    pub audits: Vec<SupplyChainAudit>,
    /// Count of entries whose hint denotes the leaf evaluation passed.
    pub pass_count: usize,
    /// Count of entries whose hint denotes a failure (mismatch,
    /// malformed sidecar, missing attestation, rubber-stamp, etc.).
    pub fail_count: usize,
}

impl SupplyChainAuditReport {
    /// True when no failure hints were emitted.
    #[must_use]
    pub const fn all_pass(&self) -> bool {
        self.fail_count == 0
    }
}

/// Audit supply-chain substrate-witness leaves across a scan report.
///
/// Walks every [`crate::scan::Immunity`] in the report. When the immunity
/// has a `requires_predicate` containing one or more of the five
/// supply-chain leaves (`dep_pinned`, `dep_attested`,
/// `maintainer_unchanged`, `content_hash_matches`, `sandbox_clean`), the
/// audit evaluates each leaf via
/// [`crate::supply_chain::evaluate`] against `workspace_root` and emits a
/// [`SupplyChainAudit`] entry per leaf.
///
/// **Why the standard `audit()` doesn't do this**: the standard
/// substrate-witness pipeline returns `false` for the supply-chain leaves
/// (honest-tier-naming per ADR-005 Amendment 2). The supply-chain audit
/// is the dedicated evaluator that knows how to drive
/// `evaluate_content_hash_matches` against `Cargo.lock` +
/// `.attest/supply-chain/content-hash/`, etc. Callers SHOULD invoke
/// both `audit()` and `audit_supply_chain()` for full coverage.
#[must_use]
pub fn audit_supply_chain(report: &ScanReport, workspace_root: &Path) -> SupplyChainAuditReport {
    let mut audits: Vec<SupplyChainAudit> = Vec::new();

    for immunity in &report.immunities {
        let Some(json) = &immunity.requires_predicate else {
            continue;
        };
        let Ok(predicate) = serde_json::from_str::<antigen_attestation::Predicate>(json) else {
            continue;
        };

        // Evaluate the predicate tree as a whole so combinator semantics
        // hold: a Mismatch inside `any_of([match_X, no_attest_Y])` does
        // NOT surface if `match_X` passes (ATK-SC-AUDIT-1). The
        // evaluator returns the per-leaf audit entries that should be
        // surfaced after combinator-aware pruning.
        let entries = eval_supply_chain_predicate(
            &predicate,
            workspace_root,
            &immunity.antigen_type,
            &immunity.file,
            immunity.line,
        );
        audits.extend(entries.entries);
    }

    let mut pass_count = 0usize;
    let mut fail_count = 0usize;
    for a in &audits {
        if is_supply_chain_pass_hint(&a.hint) {
            pass_count += 1;
        } else {
            fail_count += 1;
        }
    }

    SupplyChainAuditReport {
        audits,
        pass_count,
        fail_count,
    }
}

/// Result of evaluating a sub-predicate over supply-chain leaves.
///
/// The combinator-aware evaluator returns both the boolean predicate
/// outcome AND the per-leaf audit entries that should be surfaced.
/// Per ATK-SC-AUDIT-1: leaf-fail entries inside a satisfied `any_of`
/// MUST NOT surface — the sibling pass discharges them.
struct SupplyChainEval {
    /// Whether the sub-predicate evaluated to logical-true.
    passed: bool,
    /// Per-leaf audit entries to surface for this sub-tree, AFTER
    /// combinator-aware pruning. Per ATK-SC-AUDIT-1 / the `any_of`-
    /// discharge rule: only entries that contribute to the
    /// load-bearing answer should appear here.
    entries: Vec<SupplyChainAudit>,
}

/// Combinator-aware evaluator over supply-chain leaves.
///
/// Per ATK-SC-AUDIT-1: a satisfied `any_of` discharges its losing
/// children's audit hints. The naive "collect every leaf and emit
/// every fail" approach surfaces false positives when a sibling
/// branch passes.
fn eval_supply_chain_predicate(
    pred: &antigen_attestation::Predicate,
    workspace_root: &Path,
    antigen_type: &str,
    file: &Path,
    line: usize,
) -> SupplyChainEval {
    use antigen_attestation::Predicate;
    match pred {
        Predicate::Leaf(l) => {
            // Per-leaf evaluation. Pass entries through unconditionally
            // at the leaf level; combinator parents prune.
            let entry = audit_supply_chain_leaf(l, workspace_root, antigen_type, file, line);
            let passed = entry
                .as_ref()
                .is_some_and(|e| is_supply_chain_pass_hint(&e.hint));
            // Non-supply-chain leaves return None — treat them as
            // logically-true for the supply-chain sub-evaluation (the
            // standard substrate-witness audit handles them). This
            // means a `requires = all_of([content_hash_matches(...),
            // ratified_doc(...)])` still surfaces the supply-chain
            // half's verdict correctly.
            let logical_passed = entry.as_ref().is_none_or(|_| passed);
            SupplyChainEval {
                passed: logical_passed,
                entries: entry.into_iter().collect(),
            }
        }
        Predicate::AllOf { children } => {
            let mut entries = Vec::new();
            let mut all_pass = true;
            for c in children {
                let sub = eval_supply_chain_predicate(c, workspace_root, antigen_type, file, line);
                if !sub.passed {
                    all_pass = false;
                }
                entries.extend(sub.entries);
            }
            SupplyChainEval {
                passed: all_pass,
                entries,
            }
        }
        Predicate::AnyOf { children } => {
            // Per ATK-SC-AUDIT-1: evaluate each child; if ANY passes,
            // discharge the others' fail entries (keep only the
            // passing entries for documentation). If none pass,
            // surface every child's failure entries.
            let mut pass_entries = Vec::new();
            let mut fail_entries = Vec::new();
            let mut any_pass = false;
            for c in children {
                let sub = eval_supply_chain_predicate(c, workspace_root, antigen_type, file, line);
                if sub.passed {
                    any_pass = true;
                    pass_entries.extend(sub.entries);
                } else {
                    fail_entries.extend(sub.entries);
                }
            }
            let entries = if any_pass { pass_entries } else { fail_entries };
            SupplyChainEval {
                passed: any_pass,
                entries,
            }
        }
        Predicate::Not { child } => {
            // `not(P)` — the supply-chain audit cannot meaningfully
            // surface "the failed leaf" because the failure is the
            // intended outcome. We invert `passed` and DROP the inner
            // entries (they describe what we WANTED to fail; surfacing
            // them as hints would be misleading). v0.2 supports `not`
            // structurally but emits no documentary entries from the
            // negated sub-tree.
            let sub = eval_supply_chain_predicate(child, workspace_root, antigen_type, file, line);
            SupplyChainEval {
                passed: !sub.passed,
                entries: Vec::new(),
            }
        }
    }
}

/// Evaluate a single supply-chain leaf and produce a [`SupplyChainAudit`]
/// entry. Returns `None` for non-supply-chain leaves.
fn audit_supply_chain_leaf(
    leaf: &antigen_attestation::Leaf,
    workspace_root: &Path,
    antigen_type: &str,
    file: &Path,
    line: usize,
) -> Option<SupplyChainAudit> {
    use antigen_attestation::Leaf;

    let (crate_name, version, hint, detail) = match leaf {
        Leaf::DepPinned { crate_name } => {
            eval_dep_pinned_to_hint(workspace_root, crate_name.as_deref())
        }
        Leaf::DepAttested {
            crate_name,
            version,
            exact_version,
            ..
        } => eval_dep_attested_to_hint(workspace_root, crate_name, version, *exact_version),
        Leaf::MaintainerUnchanged {
            crate_name,
            since_version,
        } => eval_maintainer_unchanged_to_hint(workspace_root, crate_name, since_version),
        Leaf::ContentHashMatches {
            crate_name,
            version,
        } => eval_content_hash_matches_to_hint(workspace_root, crate_name, version),
        Leaf::SandboxClean {
            crate_name,
            sandbox_kind,
        } => eval_sandbox_clean_to_hint(crate_name, sandbox_kind),
        // Non-supply-chain leaves: not our pipeline's responsibility.
        Leaf::RatifiedDoc { .. }
        | Leaf::Signers { .. }
        | Leaf::SignedTrailer { .. }
        | Leaf::OraclesComplete { .. }
        | Leaf::FreshWithinDays { .. } => return None,
    };

    Some(SupplyChainAudit {
        antigen_type: antigen_type.to_string(),
        file: file.to_path_buf(),
        line,
        crate_name,
        version,
        hint,
        detail,
    })
}

/// Return `(crate, version, hint, detail)` for `dep_pinned`.
fn eval_dep_pinned_to_hint(
    workspace_root: &Path,
    crate_name: Option<&str>,
) -> (String, String, AuditHint, Option<String>) {
    use crate::supply_chain::{evaluate, witness::DepPinnedState};
    let state = evaluate::evaluate_dep_pinned(workspace_root, crate_name);
    let (hint, detail) = match &state {
        DepPinnedState::AllPinned => (AuditHint::FunctionResolves, None),
        DepPinnedState::Unpinned { unpinned_deps } => (
            AuditHint::UnpinnedDependency,
            Some(format!("unpinned: {unpinned_deps:?}")),
        ),
        DepPinnedState::NotInManifest { crate_name: cn } => (
            AuditHint::UnpinnedDependency,
            Some(format!("crate not in manifest: {cn}")),
        ),
    };
    (
        crate_name.map_or_else(|| "*".to_string(), str::to_string),
        String::new(),
        hint,
        detail,
    )
}

/// Return `(crate, version, hint, detail)` for `dep_attested`.
fn eval_dep_attested_to_hint(
    workspace_root: &Path,
    crate_name: &str,
    version: &str,
    exact_version: bool,
) -> (String, String, AuditHint, Option<String>) {
    use crate::supply_chain::{evaluate, witness::DepAttestedState};
    let state = evaluate::evaluate_dep_attested(workspace_root, crate_name, version, exact_version);
    let (hint, detail) = match &state {
        DepAttestedState::Attested { .. } => (AuditHint::FunctionResolves, None),
        DepAttestedState::AttestedWithoutReviewableArtifact => {
            (AuditHint::DepAttestWithoutReviewableArtifact, None)
        }
        DepAttestedState::SidecarMissing => (AuditHint::UnattestedDependencyInclusion, None),
        DepAttestedState::SidecarMalformed { error } => (
            AuditHint::UnattestedDependencyInclusion,
            Some(format!("sidecar malformed: {error}")),
        ),
        DepAttestedState::AttestationStale {
            attested_version,
            requested_version,
        } => (
            AuditHint::DepAttestationStale,
            Some(format!(
                "attested: {attested_version}; requested: {requested_version}"
            )),
        ),
    };
    (crate_name.to_string(), version.to_string(), hint, detail)
}

/// Return `(crate, since_version, hint, detail)` for `maintainer_unchanged`.
fn eval_maintainer_unchanged_to_hint(
    workspace_root: &Path,
    crate_name: &str,
    since_version: &str,
) -> (String, String, AuditHint, Option<String>) {
    use crate::supply_chain::{evaluate, witness::MaintainerState};
    let state = evaluate::evaluate_maintainer_unchanged(workspace_root, crate_name, since_version);
    let (hint, detail) = match &state {
        MaintainerState::Unchanged => (AuditHint::FunctionResolves, None),
        MaintainerState::Changed { added, removed } => (
            AuditHint::MaintainerChangeWithoutReattestation,
            Some(format!("added: {added:?}; removed: {removed:?}")),
        ),
        MaintainerState::SnapshotMissing => (
            AuditHint::MaintainerChangeWithoutReattestation,
            Some("snapshot missing".to_string()),
        ),
        MaintainerState::CratesIoQueryUnavailable => (AuditHint::CratesIoMetadataQueryFailed, None),
    };
    (
        crate_name.to_string(),
        since_version.to_string(),
        hint,
        detail,
    )
}

/// Return `(crate, version, hint, detail)` for `content_hash_matches`.
fn eval_content_hash_matches_to_hint(
    workspace_root: &Path,
    crate_name: &str,
    version: &str,
) -> (String, String, AuditHint, Option<String>) {
    use crate::supply_chain::{evaluate, witness::ContentHashState};
    let state = evaluate::evaluate_content_hash_matches(workspace_root, crate_name, version);
    let (hint, detail) = match &state {
        ContentHashState::Matches => (AuditHint::FunctionResolves, None),
        ContentHashState::Mismatch { recorded, current } => (
            AuditHint::ContentHashMismatch,
            Some(format!("recorded: {recorded}; current: {current}")),
        ),
        ContentHashState::NoAttestation => (AuditHint::ContentHashNoAttestation, None),
        ContentHashState::CrateNotInLockfile { crate_name: cn } => (
            AuditHint::ContentHashNoAttestation,
            Some(format!("crate not in Cargo.lock: {cn}")),
        ),
        ContentHashState::SidecarMalformed { error } => (
            AuditHint::ContentHashSidecarMalformed,
            Some(format!("malformed: {error}")),
        ),
    };
    (crate_name.to_string(), version.to_string(), hint, detail)
}

/// Return `(crate, version, hint, detail)` for `sandbox_clean`. v0.2:
/// tooling not available — emit the awareness hint.
fn eval_sandbox_clean_to_hint(
    crate_name: &str,
    sandbox_kind: &str,
) -> (String, String, AuditHint, Option<String>) {
    let hint = if sandbox_kind == "proc-macro" {
        AuditHint::UnsandboxedProcMacro
    } else {
        AuditHint::UnsandboxedBuildScript
    };
    (
        crate_name.to_string(),
        String::new(),
        hint,
        Some(format!(
            "v0.2 sandbox tooling not yet available; kind={sandbox_kind}"
        )),
    )
}

/// Distinguish supply-chain pass hints from fail hints. `FunctionResolves`
/// is the borrowed-from-standard-audit "predicate passed" hint that the
/// supply-chain audit emits for clean evaluations.
const fn is_supply_chain_pass_hint(hint: &AuditHint) -> bool {
    matches!(hint, AuditHint::FunctionResolves)
}

// ============================================================================
// Convergent-Evidence Family audit (ADR-024)
// ============================================================================

/// Default workspace floor for `#[clonal]` iterations. Configurable via
/// `[package.metadata.antigen.clonal_iterations_floor]` in a future
/// amendment.
pub const CLONAL_ITERATIONS_DEFAULT_FLOOR: u64 = 100;

/// Default workspace floor for `#[igg]` historical span (days).
pub const IGG_HISTORICAL_SPAN_DEFAULT_FLOOR: u64 = 30;

/// One result of auditing a convergent-evidence declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergentEvidenceAudit {
    /// The original declaration from the scan.
    pub declaration: crate::scan::ConvergentEvidence,
    /// The hint(s) the audit emitted for this declaration. A single
    /// declaration may surface multiple hints (e.g., `#[diagnostic]`
    /// can be both class-collapsed AND modality-insufficient).
    pub hints: Vec<AuditHint>,
}

/// Aggregate convergent-evidence audit report.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConvergentEvidenceAuditReport {
    /// Per-declaration audit results.
    pub audits: Vec<ConvergentEvidenceAudit>,
    /// Count of declarations whose hint set is empty (clean).
    pub clean_count: usize,
    /// Count of declarations whose hint set is non-empty (concerns
    /// surfaced).
    pub concern_count: usize,
}

impl ConvergentEvidenceAuditReport {
    /// True when no concerns were surfaced.
    #[must_use]
    pub const fn all_clean(&self) -> bool {
        self.concern_count == 0
    }
}

/// Audit convergent-evidence declarations across a scan report.
///
/// Walks every [`crate::scan::ConvergentEvidence`] in the report and
/// produces [`ConvergentEvidenceAudit`] entries surfacing the relevant
/// audit hints per ADR-024 §Audit-hint-vocabulary.
#[must_use]
pub fn audit_convergent_evidence(report: &ScanReport) -> ConvergentEvidenceAuditReport {
    let known_antigen_names: std::collections::HashSet<&str> = report
        .antigens
        .iter()
        .map(|a| a.type_name.as_str())
        .collect();

    let mut audits: Vec<ConvergentEvidenceAudit> = Vec::new();

    for decl in &report.convergent_evidences {
        let hints = evaluate_convergent_evidence_hints(decl, &known_antigen_names);
        audits.push(ConvergentEvidenceAudit {
            declaration: decl.clone(),
            hints,
        });
    }

    let mut clean_count = 0usize;
    let mut concern_count = 0usize;
    for a in &audits {
        if a.hints.is_empty() {
            clean_count += 1;
        } else {
            concern_count += 1;
        }
    }

    ConvergentEvidenceAuditReport {
        audits,
        clean_count,
        concern_count,
    }
}

fn evaluate_convergent_evidence_hints(
    decl: &crate::scan::ConvergentEvidence,
    known_antigen_names: &std::collections::HashSet<&str>,
) -> Vec<AuditHint> {
    use crate::scan::ConvergentEvidenceKind;

    let mut hints = Vec::new();
    match decl.kind {
        ConvergentEvidenceKind::Diagnostic => {
            if decl.modality_classes.is_empty() {
                hints.push(AuditHint::DiagnosticModalitiesEmpty);
                return hints;
            }
            let distinct: std::collections::HashSet<&str> =
                decl.modality_classes.iter().map(String::as_str).collect();
            // Class-collapse: many entries, all same class (per C1)
            if distinct.len() == 1 && decl.modality_classes.len() > 1 {
                hints.push(AuditHint::DiagnosticModalitiesClassCollapsed);
            }
            if let Some(min) = decl.min_independent {
                if u64::try_from(distinct.len()).unwrap_or(u64::MAX) < min {
                    hints.push(AuditHint::DiagnosticModalityInsufficient);
                }
            }
        }
        ConvergentEvidenceKind::Clonal => {
            // Fixed-seed in scan output: the proc-macro rejects this at
            // parse time, but the scan walks raw source — pre-cap source
            // can surface here. Surface the hint explicitly.
            if matches!(decl.seed_kind.as_deref(), Some("Fixed")) {
                hints.push(AuditHint::ClonalFixedSeedDetected);
            }
            if let Some(iters) = decl.iterations {
                if iters < CLONAL_ITERATIONS_DEFAULT_FLOOR {
                    hints.push(AuditHint::ClonalIterationsBelowThreshold);
                }
            }
        }
        ConvergentEvidenceKind::Igg => {
            if let Some(span) = decl.historical_span {
                if span < IGG_HISTORICAL_SPAN_DEFAULT_FLOOR {
                    hints.push(AuditHint::IggSpanTooShort);
                }
            }
            // Per ATK-CE-3-B: count UNIQUE witnesses, not raw count.
            // The same identity signing twice doesn't add reattestation
            // independence — the discipline is about independent re-
            // verification, not raw signature count. Raw-count check
            // (`witnesses.len() >= min_re`) is misleading because
            // duplicate identities inflate the apparent count.
            let unique_count: std::collections::HashSet<&str> =
                decl.witnesses.iter().map(String::as_str).collect();
            if let Some(min_re) = decl.min_reattestations {
                if u64::try_from(unique_count.len()).unwrap_or(u64::MAX) < min_re {
                    hints.push(AuditHint::IggReattestationsInsufficient);
                }
            }
            // Identity-collapse: best-effort at scan time — if the
            // recorded witnesses all collapse to one identity, surface
            // the warning. Real signer-identity tracking is v0.3+.
            if decl.witnesses.len() > 1 && unique_count.len() == 1 {
                hints.push(AuditHint::IggIdentityCollapseWarning);
            }
        }
        ConvergentEvidenceKind::Crossreactive => {
            for fp in &decl.fingerprints {
                if !known_antigen_names.contains(fp.as_str()) {
                    hints.push(AuditHint::CrossreactiveFingerprintUnresolved);
                    break;
                }
            }
        }
        ConvergentEvidenceKind::Polyclonal
        | ConvergentEvidenceKind::Monoclonal
        | ConvergentEvidenceKind::Adcc => {
            // v0.2: marker primitives. Lineage-count enforcement
            // (polyclonal) and mechanism-pairing detection (adcc)
            // require co-located witness sites; deferred to v0.3+ when
            // the scan layer cross-links convergent declarations with
            // their on-item witness companions. monoclonal is
            // documentary by definition. v0.2 emits no automatic
            // concerns for any of the three.
        }
    }
    hints
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_clippy_external_tool() {
        assert_eq!(
            detect_external_tool("clippy::no_panic_in_drop"),
            Some("clippy")
        );
    }

    #[test]
    fn detect_kani_external_tool() {
        assert_eq!(
            detect_external_tool("kani::proof_drop_safety"),
            Some("kani")
        );
    }

    #[test]
    fn detect_no_tool_for_local_function() {
        assert_eq!(detect_external_tool("safe_type_drop_no_panic_test"), None);
    }

    #[test]
    fn validate_witness_strips_path_prefix() {
        let mut idx = FunctionIndex::new();
        idx.insert(
            "my_test".to_string(),
            vec![FunctionEntry {
                location: PathBuf::from("src/lib.rs"),
                kind: WitnessKind::Test,
            }],
        );

        let status = validate_witness("module::path::my_test", &idx);
        assert!(matches!(status, WitnessStatus::Resolved { .. }));
    }

    #[test]
    fn validate_witness_reports_missing_when_empty() {
        let idx = FunctionIndex::new();
        let status = validate_witness("", &idx);
        assert_eq!(status, WitnessStatus::Missing);
    }

    #[test]
    fn validate_witness_reports_not_found_for_unknown() {
        let idx = FunctionIndex::new();
        let status = validate_witness("nonexistent_test", &idx);
        assert!(matches!(status, WitnessStatus::NotFound { .. }));
    }

    // ========================================================================
    // W5 — structural proptest! witness detection.
    //
    // Pre-W5, `detect_kind` did `self.source.contains("proptest!")` as a
    // sentinel — if the source string contained that text anywhere, every
    // function in the file was tagged `WitnessKind::Proptest`. Doc comments
    // mentioning the macro for explanatory purposes triggered the same
    // over-classification.
    //
    // W5 lifts this to structural detection via `visit_macro` + token-walking
    // the macro body for `fn IDENT` patterns. These tests are the contract
    // pinning the W5 behavior without needing a full filesystem fixture.
    // ========================================================================

    /// Run the function-index walk against an in-memory source string.
    /// Mirrors what `collect_function_index` does per-file but without
    /// touching disk — gives the W5 unit tests a tight feedback loop.
    fn index_from_str(source: &str) -> FunctionIndex {
        use syn::visit::Visit;
        let file = syn::parse_file(source).expect("source must parse");
        let mut index = FunctionIndex::new();
        let mut visitor = FunctionIndexVisitor {
            file_path: PathBuf::from("<test>.rs"),
            source,
            index: &mut index,
        };
        visitor.visit_file(&file);
        index
    }

    /// Helper for tests that expect a single index entry for a name.
    /// Panics with a clear message if the name is unindexed or ambiguous.
    fn unique_kind(idx: &FunctionIndex, name: &str) -> WitnessKind {
        let entries = idx.get(name).unwrap_or_else(|| panic!("{name} indexed"));
        assert_eq!(
            entries.len(),
            1,
            "expected single index entry for {name}, got {entries:?}",
        );
        entries[0].kind.clone()
    }

    #[test]
    fn w5_proptest_inner_fns_are_classified_proptest() {
        let src = r"
            proptest! {
                #[test]
                fn first_proptest(x in 0u32..100) {
                    assert!(x < 100);
                }

                #[test]
                fn second_proptest(x in 0u32..100, y in 0u32..100) {
                    assert!(x + y < 200);
                }
            }
        ";
        let idx = index_from_str(src);
        assert_eq!(unique_kind(&idx, "first_proptest"), WitnessKind::Proptest);
        assert_eq!(unique_kind(&idx, "second_proptest"), WitnessKind::Proptest);
    }

    #[test]
    fn w5_proptest_path_qualified_macro_is_recognized() {
        // The fixture canonical form is `proptest::proptest!`, matching how
        // the `proptest` crate is typically imported. The W5 helper
        // `macro_path_last_is` checks the LAST segment, so any path ending
        // in `proptest` matches.
        let src = r"
            proptest::proptest! {
                #[test]
                fn qualified_form_proptest(x in 0u32..100) {
                    assert!(x < 100);
                }
            }
        ";
        let idx = index_from_str(src);
        assert_eq!(
            unique_kind(&idx, "qualified_form_proptest"),
            WitnessKind::Proptest,
        );
    }

    #[test]
    fn w5_test_function_outside_proptest_is_classified_test() {
        // A regular `#[test]` outside any proptest! block must remain
        // `WitnessKind::Test`. The pre-W5 sentinel would have over-classified
        // this as Proptest if the file contained the string `proptest!`
        // anywhere; this test exercises the negative case directly.
        let src = r"
            // Doc-style comment mentioning proptest! for explanation purposes.
            // Pre-W5 this string in the source was sufficient to flag every
            // function in the file as Proptest. W5 must not regress to that.
            #[test]
            fn plain_test() {
                assert_eq!(2 + 2, 4);
            }

            proptest! {
                #[test]
                fn proptest_one(x in 0u32..10) {
                    assert!(x < 10);
                }
            }
        ";
        let idx = index_from_str(src);
        assert_eq!(
            unique_kind(&idx, "plain_test"),
            WitnessKind::Test,
            "plain_test outside proptest! must be Test, not Proptest, even when \
             the same file contains a proptest! invocation",
        );
        assert_eq!(unique_kind(&idx, "proptest_one"), WitnessKind::Proptest);
    }

    #[test]
    fn w5_doc_comment_mentioning_proptest_does_not_over_classify() {
        // The exact regression the pre-W5 textual sentinel had: a doc
        // comment containing the literal string `proptest!` would tag
        // every function in the file as Proptest. W5's structural detection
        // only fires on actual macro invocations, so this `#[test]` stays Test.
        let src = r"
            /// This function has nothing to do with proptest! — the macro
            /// is named here only for documentation.
            #[test]
            fn doc_comment_only_test() {
                assert!(true);
            }
        ";
        let idx = index_from_str(src);
        assert_eq!(
            unique_kind(&idx, "doc_comment_only_test"),
            WitnessKind::Test,
            "doc-comment mention must not trigger Proptest",
        );
    }

    #[test]
    fn w5_plain_function_is_classified_function() {
        let src = r"
            fn no_attribute_function() {}
        ";
        let idx = index_from_str(src);
        assert_eq!(
            unique_kind(&idx, "no_attribute_function"),
            WitnessKind::Function,
        );
    }

    #[test]
    fn w5_extract_proptest_fn_names_skips_nested() {
        // Nested function definitions inside a fn body live in a Group token;
        // the top-level token walk should not descend into them. This locks
        // the "nested fn doesn't get registered as a proptest test" invariant.
        use proc_macro2::TokenStream;
        let tokens: TokenStream = r"
            #[test]
            fn outer(x in 0u32..10) {
                fn nested_helper() {}
                assert!(x < 10);
            }
        "
        .parse()
        .unwrap();
        let names = extract_proptest_fn_names(&tokens);
        assert_eq!(names, vec!["outer".to_string()]);
    }

    #[test]
    fn w5_macro_path_last_is_handles_qualified_paths() {
        let bare: syn::Path = syn::parse_str("proptest").unwrap();
        let qualified: syn::Path = syn::parse_str("proptest::proptest").unwrap();
        let unrelated: syn::Path = syn::parse_str("other_crate::other_macro").unwrap();
        assert!(macro_path_last_is(&bare, "proptest"));
        assert!(macro_path_last_is(&qualified, "proptest"));
        assert!(!macro_path_last_is(&unrelated, "proptest"));
    }

    #[test]
    fn detect_phantom_nested_generic_returns_none() {
        // `Witnessed::<Option<MyType>, MyWitness>::try_new` has a nested `<>`
        // inside the type-param region. split_once('>') fires at the inner `>`,
        // producing malformed fields. The balanced-bracket guard must return None
        // so audit falls through to function-index (NotFound), not FormalProof.
        assert_eq!(
            detect_phantom_type_witness("Witnessed::<Option<MyType>, MyWitness>::try_new"),
            None,
        );
        // Simple non-nested shape must still work.
        assert!(matches!(
            detect_phantom_type_witness("PolarityProof::<FrameTranslation>::verified"),
            Some(WitnessKind::PhantomType { .. }),
        ));
    }
}
