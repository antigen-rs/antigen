//! Ratification schema — the serde-derived on-disk shape for
//! `.attest/<Antigen>.json` sidecars.
//!
//! Single source of truth: the audit reads this; `cargo antigen attest sign`
//! and `cargo antigen tolerate sign` write this; future editor extensions
//! validate against this. Schema is intentionally serde-derived (not
//! hand-rolled JSON parsing) so the format is documented by the type
//! definition itself.
//!
//! ## Layout convention (ADR-019 M3)
//!
//! Sidecars live at `<file-stem>.attest/<AntigenName>.json` adjacent to the
//! source file. For `src/numerics.rs` carrying `SignedZeroDiscipline` on
//! `sinh`, the sidecar lives at
//! `src/numerics.attest/SignedZeroDiscipline.json`. Granularity follows
//! the antigen's `scope` field — site / file / package / workspace.

use std::{collections::BTreeMap, path::PathBuf};

use crate::tier::SignatureStrength;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Schema version tag.
///
/// Bumped via ADR amendment when the on-disk format changes
/// incompatibly. Audit refuses sidecars with newer schema versions than
/// it understands (ratchet-asymmetry per ADR-019 §Decision).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SchemaVersion {
    /// Initial v0.1 shape. Adds `RatificationKind`, `EvidenceKind`-aware
    /// audit hints, `SignerBasis` with anti-laundering safeguards.
    V1,
}

/// Immunity-vs-tolerance discriminator.
///
/// The struct itself is isomorphic between the two (per ADR-019 M3 — the
/// same `Ratification` serves both via `RatificationKind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RatificationKind {
    /// Backs `#[immune(X, requires = ...)]` — discipline-witness for an
    /// immunity claim.
    Immunity,
    /// Backs `#[antigen_tolerance(X, sidecar = true, requires = ...)]` —
    /// discipline-witness for an opt-in tolerance ratification (closes
    /// ADR-011's vibes-grade gap).
    Tolerance,
}

/// Identifier for the antigen this ratification attests to.
///
/// Uses the declaration-site canonical name (ADR-017) so the sidecar can
/// be validated against the antigen's declared scope + fingerprint
/// without needing to enumerate cross-crate trust.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AntigenIdentifier {
    /// Canonical name from the `#[antigen(name = ...)]` declaration.
    pub name: String,
    /// Defining-crate ident if known (cross-crate ratifications).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub defined_in: Option<String>,
}

/// Top-level sidecar shape. One `Ratification` per `<file>.attest/<Antigen>.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ratification {
    /// Format version. Audit refuses unknown future versions.
    pub schema_version: SchemaVersion,
    /// Immunity vs tolerance (ADR-019 §Decision).
    pub kind: RatificationKind,
    /// Which antigen this sidecar ratifies.
    pub antigen: AntigenIdentifier,
    /// Source file the sidecar attests to (the file containing the
    /// presented item). Path is recorded relative to workspace root for
    /// portability; audit resolves against the audit's `--root`.
    pub source_file: PathBuf,
    /// Per-item entries. Each declared `#[immune]` / `#[antigen_tolerance]`
    /// site in `source_file` gets its own [`ItemRatification`] (granularity
    /// per ADR-019 M3 scope-field).
    pub items: Vec<ItemRatification>,
}

/// Per-presented-item ratification entry. Multiple `ItemRatification`s can
/// share a `Ratification` when several items in one file present the same
/// antigen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemRatification {
    /// Item identity (e.g., `"sinh"` or `"MyType::method"`). Matches the
    /// scan-side item identifier per ADR-018 (item-identity v0.1).
    pub item_path: String,
    /// Fingerprint of the item AS OF THE LAST FRESH SIGNATURE. Currency
    /// check: audit re-computes the fingerprint at audit time; if it
    /// differs, the audit reports `discipline-substrate-stale`.
    pub current_fingerprint: String,
    /// Optional reference to a discipline doc (used by `ratified_doc(...)`
    /// leaf without an explicit `path`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc_ref: Option<DocRef>,
    /// All signers who have attested at this item-site. `signers(required =
    /// [...])` leaf checks names against this list. Order is preservation-
    /// order (append-only ratchet); audit cares about set membership +
    /// freshness, not order.
    #[serde(default)]
    pub signers: Vec<Signer>,
    /// Oracle artifact-class references used by `oracles_complete(...)` leaf.
    ///
    /// Per ADR-021 §D3 (ratified 2026-05-20): elements are full `Oracle`
    /// artifacts (state machine + stewardship + provenance), not the old
    /// `OracleRef { path, status }` struct. Old-shape sidecars are accepted
    /// via two-pass deserialization with an `oracle-ref-needs-migration` audit
    /// hint (additive-only schema evolution per ADR-021 §D2).
    #[serde(default, deserialize_with = "deserialize_oracles_with_legacy_fallback")]
    pub oracles: Vec<Oracle>,
    /// Optional expiry date for the whole entry. `fresh_within_days(...)`
    /// leaf computes against this OR the latest signer date — whichever is
    /// later.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fresh_through: Option<NaiveDate>,
    /// Open extension slot for ADR-019 §Posture "open integration surface."
    /// v0.2+ amendments use this; v0.1 leaves it empty by convention.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// A reference to a discipline doc.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocRef {
    /// Path to the doc, relative to workspace root.
    pub path: PathBuf,
    /// Minimum version of the doc accepted. Read from the doc's YAML
    /// frontmatter `version` field. Audit fails if doc version < min.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_version: Option<String>,
    /// Optional anchor within the doc (e.g., a section heading slug).
    /// Audit fails if anchor isn't present in the doc.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<String>,
}

/// A typed reference to an oracle artifact (ADR-021 §D1).
///
/// All variants behave identically to the audit (R1 content-blindness):
/// the audit validates structural well-formedness (URL parses, DOI matches
/// format) plus completion marker plus version-pin. NO network calls.
///
/// The `Other` variant is the open-extension surface for unanticipated
/// reference kinds (additive-only schema evolution per §D2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OracleRef {
    /// Local workspace file reference (e.g., a methodology doc, oracle
    /// completion JSON). `status_field` + `expected_status` retained for
    /// migration compatibility with the v0.0 shape (one-shot oracle files
    /// with frontmatter `status: complete`).
    LocalFile {
        /// Path to the file, relative to workspace root.
        path: PathBuf,
        /// Optional name of the YAML frontmatter field carrying status
        /// (defaults to `status`).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        status_field: Option<String>,
        /// Optional value the status field must hold for the reference to
        /// be considered satisfied (defaults to `complete`).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expected_status: Option<String>,
    },
    /// HTTP/HTTPS URL reference (e.g., a public spec, RFC, dataset link).
    /// Audit checks URL parses; no network fetch.
    Url {
        /// The URL string.
        url: String,
        /// Optional human-readable label.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
    },
    /// DOI reference (academic / standards literature).
    /// Audit checks DOI format; no resolver call.
    Doi {
        /// DOI string (e.g., `10.1145/3593856`).
        doi: String,
        /// Optional section/page anchor within the cited work.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        section: Option<String>,
    },
    /// arXiv reference. Audit checks arxiv ID format; no fetch.
    Arxiv {
        /// arXiv identifier (e.g., `2401.12345` or `cs.AI/0501001`).
        arxiv_id: String,
        /// Optional section anchor.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        section: Option<String>,
    },
    /// GitHub issue reference (e.g., upstream bug or design conversation).
    GitHubIssue {
        /// Repository `owner/repo` identifier.
        repo: String,
        /// Issue number.
        issue: u32,
    },
    /// Escape hatch for unanticipated reference kinds (additive-only
    /// extension surface per ADR-021 §D2).
    Other {
        /// Caller-supplied sub-kind discriminator (free-text). Renamed
        /// from `kind` to avoid clashing with serde's outer tag.
        subkind: String,
        /// Caller-supplied reference string.
        reference: String,
        /// Optional human-readable label.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
    },
}

/// Oracle as artifact-class (ADR-021 §D3 Model B).
///
/// An oracle is not a typed pointer carrying a completion marker — it is a
/// **structurally distinguished artifact-class** with its own state machine,
/// dedicated stewards, provenance, and lifecycle tracking. Per Tekgy R2:
/// without lifecycle structure, discipline degrades to convention.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Oracle {
    /// Unique oracle identifier within the workspace (caller-chosen
    /// identifier; typically slug-style: `higham-2002-section-6-3`).
    pub id: String,
    /// Typed pointer to the underlying reference (file, URL, DOI, ...).
    pub reference: OracleRef,
    /// Current lifecycle state.
    pub state: OracleState,
    /// Stewards authorized to transition the oracle's state.
    ///
    /// Minimum 2 required at oracle creation (ATK-021-13 succession
    /// mitigation). Append-only in v0.1 (F28-R1): stewards may be added
    /// but never removed in the same sidecar version. Steward removal is
    /// a v0.2 extension that requires explicit quorum semantics.
    pub stewards: Vec<Steward>,
    /// Who created this oracle and when.
    pub created: Provenance,
    /// Version pinning (immutable record of what version was
    /// authoritative at oracle creation / last `complete` transition).
    pub version: OracleVersion,
    /// State transition history (append-only).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transitions: Vec<StateTransition>,
    /// Open extension slot for v0.2+ amendments
    /// (ADR-021 §D2 additive-only schema).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// Oracle steward — authorized to maintain the oracle (state transitions,
/// deprecation, retirement, revocation).
///
/// Stewards are categorically distinct from signers (ADR-021 §D3 + B-021-4
/// biology grounding: FDC stewardship vs B-cell attestation; different
/// cellular lineages). The math-researcher who attests against an oracle
/// is NOT necessarily its steward.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Steward {
    /// Steward name (git config `user.name` when CLI-recorded).
    pub name: String,
    /// Optional role tag (e.g., `"tech-lead"`, `"domain-authority"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// REQUIRED non-empty (ADR-005 Amendment 2 inheritance). WHY this
    /// person has steward authority — e.g., "appointed by tech-lead
    /// 2026-03-01; domain authority on floating-point semantics".
    pub authorization_basis: String,
}

/// Oracle lifecycle state machine (ADR-021 §D3).
///
/// Monotonic; backward transitions PROHIBITED (validated at parse time
/// and at CLI invocation time). The five-state distinction is load-
/// bearing for trust semantics: Retired preserves prior attestations at
/// Execution; Revoked with `invalidates_prior_attestations = true`
/// retroactively demotes them to Reachability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum OracleState {
    /// Not yet authoritatively established. Signers CANNOT attest against
    /// Draft oracles; `oracles_complete(...)` predicate leaves referencing
    /// a Draft oracle evaluate to false with audit hint
    /// `oracle-draft-blocks-attestation`.
    Draft,
    /// Authoritatively established. Signers may attest.
    Complete,
    /// Superseded by a successor oracle. Existing attestations honored at
    /// `Execution` tier. For INCORRECT oracles use
    /// `Revoked { invalidates_prior_attestations: true }` instead.
    Deprecated {
        /// Optional successor oracle id (free-text reference; may be empty
        /// for deprecations without a named successor).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        superseded_by: Option<String>,
        /// REQUIRED non-empty (Amendment 2): WHY deprecated.
        reason: String,
    },
    /// Permanently gone / superseded. All prior attestations honored at
    /// `Execution` tier. Use ONLY when oracle is gone for reasons OTHER
    /// THAN incorrectness. For incorrect oracles use `Revoked` with
    /// `invalidates_prior_attestations = true`.
    Retired {
        /// REQUIRED non-empty (Amendment 2): WHY retired.
        reason: String,
        /// Steward name (must appear in oracle's `stewards[*].name`).
        retired_by: String,
    },
    /// Oracle was compromised or incorrect.
    /// `invalidates_prior_attestations` controls retroactive demotion:
    /// - `false` — prior attestations preserved at `Execution` (steward
    ///   judges the oracle text was correct but is no longer authoritative)
    /// - `true` — prior attestations retroactively demoted to
    ///   `Reachability` (oracle was fundamentally wrong; attestations
    ///   based on it cannot stand)
    Revoked {
        /// REQUIRED non-empty (Amendment 2): WHY revoked.
        reason: String,
        /// Steward name (must appear in oracle's `stewards[*].name`).
        revoked_by: String,
        /// Whether to retroactively demote prior attestations to
        /// `Reachability` (true) or preserve them at `Execution` (false).
        invalidates_prior_attestations: bool,
    },
}

/// A single state-machine transition entry (append-only log).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateTransition {
    /// Source state name (`snake_case` discriminant, e.g., `"draft"`).
    pub from: String,
    /// Destination state name (`snake_case` discriminant).
    pub to: String,
    /// Steward name (must appear in oracle's `stewards[*].name`).
    pub authorized_by: String,
    /// Date of the transition.
    pub at: NaiveDate,
    /// REQUIRED non-empty (ADR-005 Amendment 2): WHY this transition.
    pub rationale: String,
}

/// Oracle version pinning — records what version/snapshot was
/// authoritative at oracle creation / last `complete` transition.
///
/// Parallel to `signed_against_fingerprint` for signers: both record
/// "what was true at the moment in time" so the audit can reason about
/// drift without reconstructing the past.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleVersion {
    /// Pinned version identifier (semver / commit-sha / date-string —
    /// caller's discretion; opaque to the audit).
    pub pinned: String,
    /// When the pin was recorded.
    pub pinned_at: NaiveDate,
}

/// Generic provenance record — who did this and when. Used by `Oracle`
/// for creation tracking; the same shape can extend to other artifact
/// classes in future ADRs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    /// Steward / signer name (git config `user.name`).
    pub recorded_by: String,
    /// Date of the action.
    pub at: NaiveDate,
}

/// Completion marker written at sign time by `attest oracle mark` (ADR-021
/// §D4 sign-time-validity).
///
/// Records "what was true at attestation time" so the audit can reason
/// about drift WITHOUT retroactively reinterpreting the past. Parallel to
/// `signed_against_fingerprint` in `Signer`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleCompletionMarker {
    /// Signer who reviewed this oracle and attested against it.
    pub marked_by: String,
    /// When the attestation was made.
    pub attested_at: NaiveDate,
    /// Version-pin: what version/snapshot was reviewed at attestation
    /// time (typically `Oracle::version.pinned`).
    pub version_pinned: String,
    /// Oracle state at the moment of attestation. Written by the CLI at
    /// sign time. Load-bearing for sign-time-validity (D4): without this
    /// field the audit cannot distinguish "signer attested when state
    /// was Complete; oracle later Deprecated" (hint, tier preserved)
    /// from "signer attested when state was already Deprecated"
    /// (different audit hint, same tier-impact rule).
    pub oracle_state_at_attestation: OracleState,
}

/// Legacy v0.0 oracle reference shape — `{ path, status }` — accepted by
/// [`deserialize_oracles_with_legacy_fallback`] for back-compatibility.
/// Lifted into a minimal `Oracle` value with `<legacy-import>` provenance
/// markers that the audit recognizes via the `oracle-ref-needs-migration`
/// hint.
#[derive(Deserialize)]
struct LegacyOracleRef {
    path: PathBuf,
    #[serde(default)]
    status: Option<String>,
}

/// Two-pass deserialization for `ItemRatification.oracles`.
///
/// Per ADR-021 §D2, the field shape changed from `Vec<OracleRef>` (v0.0
/// struct shape `{ path, status }`) to `Vec<Oracle>` (v0.1 artifact-class).
/// This is the LAST breaking shape change before the additive-only commit;
/// existing v0.0 sidecars must continue parsing so v0.1 audit can emit
/// the `oracle-ref-needs-migration` hint without forcing a hand-edit.
///
/// First-pass: attempt to deserialize as `Vec<Oracle>` (the new shape).
/// Second-pass: if first-pass fails, attempt to deserialize as legacy
/// `Vec<LegacyOracleRef>` and lift each entry into a minimal `Oracle` with
/// `OracleRef::LocalFile`, `OracleState::Draft`, and a synthetic steward
/// list. The audit reports the legacy origin via the migration hint.
fn deserialize_oracles_with_legacy_fallback<'de, D>(
    deserializer: D,
) -> Result<Vec<Oracle>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize as _;

    let value = serde_json::Value::deserialize(deserializer)?;

    // Empty / missing → empty vec (default for the field).
    if value.is_null() {
        return Ok(Vec::new());
    }

    // First-pass: try the new shape.
    if let Ok(oracles) = serde_json::from_value::<Vec<Oracle>>(value.clone()) {
        return Ok(oracles);
    }

    // Second-pass: legacy shape.
    let legacy: Vec<LegacyOracleRef> =
        serde_json::from_value(value).map_err(serde::de::Error::custom)?;

    Ok(legacy
        .into_iter()
        .map(|leg| Oracle {
            id: leg.path.display().to_string(),
            reference: OracleRef::LocalFile {
                path: leg.path,
                status_field: None,
                expected_status: leg.status,
            },
            state: OracleState::Draft,
            stewards: Vec::new(),
            created: Provenance {
                recorded_by: "<legacy-import>".to_string(),
                at: NaiveDate::from_ymd_opt(2026, 1, 1)
                    .expect("hardcoded legacy-import date is well-formed"),
            },
            version: OracleVersion {
                pinned: "<legacy-import>".to_string(),
                pinned_at: NaiveDate::from_ymd_opt(2026, 1, 1)
                    .expect("hardcoded legacy-import date is well-formed"),
            },
            transitions: Vec::new(),
            extensions: BTreeMap::new(),
        })
        .collect())
}

/// A single signer entry.
///
/// Each named-signer who has attested at an item-site appears once in
/// `ItemRatification::signers` per attestation event (Fresh = new
/// attestation; `DeltaFrom` = carry-forward from a prior signature with
/// rationale).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signer {
    /// Signer name (git config `user.name`).
    pub name: String,
    /// Optional role tag (e.g., `"math-researcher"`, `"reviewer"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Date of this attestation (signer's local date at sign time).
    pub date: NaiveDate,
    /// Fingerprint of the item AT THE TIME this signer signed. Drift
    /// detection: if audit's recomputed fingerprint != this value, the
    /// signer's attestation is stale.
    pub signed_against_fingerprint: String,
    /// Basis: fresh attestation, or carry-forward delta (with anti-
    /// laundering safeguards). See [`SignerBasis`].
    pub basis: SignerBasis,
    /// Identity-binding strength of this signer's attestation.
    ///
    /// `TextStamp` — name + timestamp only; no external identity verification;
    /// used by LLM agents or non-git-configured reviewers.
    /// `GitTrust` — identity bound to `git config user.name + user.email`;
    /// v0.1 default for human reviewers.
    /// `CryptoSigned` — identity bound cryptographically; requires `signature`
    /// field; v0.4+ activation path.
    ///
    /// Defaults to `GitTrust` on deserialization for backward compatibility
    /// with sidecars written before this field existed. New sidecars MUST
    /// record the actual strength used at sign time.
    #[serde(default = "SignatureStrength::default_git_trust")]
    pub strength: SignatureStrength,
    /// Optional cryptographic signature (v0.4+; DSSE-PAE-encoded).
    /// `None` in v0.1 (git-trust basis only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<Signature>,
}

/// Whether a signer entry is a fresh attestation or a carry-forward delta
/// from a prior attestation. Delta entries carry anti-laundering safeguards
/// per ADR-019 §Decision + adversarial T2-R.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SignerBasis {
    /// Fresh attestation — signer reviewed the current item state and
    /// signed against the current fingerprint. The `attest sign` CLI is
    /// the only write-path that produces a `Fresh` basis (R-A4).
    ///
    /// `reasoning` carries optional free-text describing what the signer
    /// checked (per observer NB004 — closes the tambear-inline-rationale
    /// gap; the inline `doc_attested(rationale = ...)` shape from
    /// `tambear-adoption-log` carried rationale at the call site for ALL
    /// signing acts; v3 sidecar Fresh basis lacked the equivalent).
    /// Optional in v0.1; `cargo antigen attest sign` may require it via
    /// workspace config (`require_fresh_reasoning = true`).
    Fresh {
        /// Optional free-text describing what was checked at sign time
        /// (e.g., "audited under Sub-pattern 5.11; dispatch shoulder
        /// verified Chebyshev-optimal at x=5"). Recorded for audit-trail
        /// visibility; not enforced as a predicate constraint.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reasoning: Option<String>,
    },
    /// Carry-forward from a prior fingerprint. The signer reviewed the
    /// diff from `prior_fingerprint` to the current state and decided the
    /// change is invariant-preserving from the prior attestation's
    /// standpoint. The three safeguards on this variant CLOSE the
    /// laundering surface: chain-depth cap; cumulative-fingerprint
    /// tracking; non-empty rationale.
    DeltaFrom {
        /// The fingerprint this delta is rooted against (the immediately
        /// previous signature for this signer at this item).
        prior_fingerprint: String,
        /// The fingerprint of the LAST `Fresh`-basis signature for this
        /// signer at this item. Audit verifies the cumulative diff (root
        /// → current) hasn't exceeded the workspace's configured
        /// cumulative-diff threshold (default: 200 lines or 25% of item,
        /// whichever smaller).
        ///
        /// Anti-laundering safeguard #2: prevents slow drift across many
        /// small deltas from accumulating into a substantive cumulative
        /// change without explicit Fresh re-attestation.
        cumulative_root_fingerprint: String,
        /// How many deltas since the last `Fresh` basis (inclusive of
        /// this entry). Audit enforces `chain_depth <= cap` (default cap:
        /// 3, configurable per workspace).
        ///
        /// Anti-laundering safeguard #1: prevents long laundering chains.
        chain_depth: u32,
        /// Why the signer believes the delta is invariant-preserving.
        /// Schema rejects empty/whitespace-only strings at parse time.
        ///
        /// Anti-laundering safeguard #3: forces explicit rationale on
        /// every delta entry; rubber-stamp deltas are impossible without
        /// at least typed justification.
        rationale: String,
    },
}

impl SignerBasis {
    /// `true` if the basis carries fresh review of the current state.
    #[must_use]
    pub const fn is_fresh(&self) -> bool {
        matches!(self, Self::Fresh { .. })
    }

    /// `true` if the basis is a carry-forward delta.
    #[must_use]
    pub const fn is_delta(&self) -> bool {
        matches!(self, Self::DeltaFrom { .. })
    }

    /// Chain depth (0 for `Fresh`; ≥1 for `DeltaFrom` variants).
    #[must_use]
    pub const fn chain_depth(&self) -> u32 {
        match self {
            Self::Fresh { .. } => 0,
            Self::DeltaFrom { chain_depth, .. } => *chain_depth,
        }
    }

    /// The optional reasoning carried by a `Fresh` basis (None for
    /// `DeltaFrom` bases — they use `rationale` instead, which is
    /// required-non-empty).
    #[must_use]
    pub fn fresh_reasoning(&self) -> Option<&str> {
        match self {
            Self::Fresh { reasoning } => reasoning.as_deref(),
            Self::DeltaFrom { .. } => None,
        }
    }
}

/// Cryptographic signature envelope (v0.4+ activation per ADR-019 §Decision).
///
/// In v0.1, no signers carry this — git-trust is the only signature basis.
/// The field is reserved on the schema so v0.4+ DSSE/Sigstore activation
/// doesn't require an incompatible schema bump.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
    /// Signature scheme identifier (e.g., `"dsse-pae-v1"`,
    /// `"sigstore-bundle-v0.2"`).
    pub scheme: String,
    /// Opaque signature payload, scheme-specific encoding.
    pub payload: String,
    /// Optional transparency-log reference (Sigstore tlog entry, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tlog_ref: Option<String>,
}

/// Errors that can occur when validating a `Ratification` post-parse.
///
/// `serde_json::from_str` catches structural / type errors; this layer
/// catches semantic invariants that serde alone cannot express
/// (non-empty rationale; chain-depth cap; sane date ranges).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// A `SignerBasis::DeltaFrom` carried an empty or whitespace-only
    /// `rationale` (anti-laundering safeguard #3).
    EmptyDeltaRationale {
        /// Item path the offending signer is recorded under.
        item_path: String,
        /// The offending signer's name.
        signer_name: String,
    },
    /// A `SignerBasis::DeltaFrom` exceeded the configured chain-depth
    /// cap. Anti-laundering safeguard #1.
    ChainDepthExceeded {
        /// Item path the offending signer is recorded under.
        item_path: String,
        /// The offending signer's name.
        signer_name: String,
        /// Observed chain depth.
        chain_depth: u32,
        /// Configured cap.
        cap: u32,
    },
    /// A `SignerBasis::DeltaFrom` declared `chain_depth = 0`, which is
    /// reserved for `Fresh` bases. Schema invariant.
    ZeroDeltaChainDepth {
        /// Item path the offending signer is recorded under.
        item_path: String,
        /// The offending signer's name.
        signer_name: String,
    },
    /// A `SignerBasis::DeltaFrom` carried a `rationale` shorter than the
    /// configured minimum (default 20 chars). Per adversarial T2R-B:
    /// non-empty alone permits rubber-stamp rationales — minimum-length
    /// enforcement keeps the field carrying actual signal.
    RationaleTooShort {
        /// Item path the offending signer is recorded under.
        item_path: String,
        /// The offending signer's name.
        signer_name: String,
        /// Observed rationale character count.
        actual_chars: usize,
        /// Configured minimum.
        min_chars: usize,
    },
    /// A `SignerBasis::DeltaFrom` at `chain_depth=1` carries a
    /// `cumulative_root_fingerprint` that differs from its `prior_fingerprint`.
    /// At depth 1 these must be identical: the cumulative root IS the prior
    /// (the last Fresh signature before this delta). A discrepancy means the
    /// sidecar anchors cumulative-diff tracking (anti-laundering safeguard #2)
    /// at a fingerprint that does not exist in this chain — indicating a
    /// construction error or tamper.
    InconsistentCumulativeRoot {
        /// Item path the offending signer is recorded under.
        item_path: String,
        /// The offending signer's name.
        signer_name: String,
        /// The `prior_fingerprint` declared by this signer.
        prior_fingerprint: String,
        /// The `cumulative_root_fingerprint` that does not match `prior_fingerprint`.
        cumulative_root_fingerprint: String,
    },
    /// A signer declared `strength = CryptoSigned` but carries no `signature`
    /// field. `CryptoSigned` requires a DSSE-PAE-encoded signature envelope
    /// (v0.4+ activation); claiming the tier without the payload is tier
    /// inflation with no cryptographic backing (NFA-17).
    StrengthSignatureMismatch {
        /// Item path the offending signer is recorded under.
        item_path: String,
        /// The offending signer's name.
        signer_name: String,
    },
    /// A workspace-configured value for an antigen-attestation knob is
    /// out of the project-enforced hard-floor bounds. Per adversarial
    /// T2R-C: workspaces can tighten anti-laundering caps but cannot
    /// loosen them beyond a hardcoded floor.
    WorkspaceConfigOutOfBounds {
        /// Which config key violates the floor (e.g., `"delta_chain_cap"`).
        key: &'static str,
        /// The offending value.
        value: u64,
        /// Hard minimum permitted.
        min: u64,
        /// Hard maximum permitted (or `u64::MAX` if unbounded above).
        max: u64,
    },
    /// An `Oracle` was created or persisted with fewer than two stewards.
    /// Per ADR-021 ATK-021-13: a single-steward oracle has no succession
    /// path — if the lone steward leaves, the oracle becomes orphaned
    /// (no one can authorize transitions, deprecate, or revoke).
    OracleRequiresMinimumTwoStewards {
        /// Oracle id (`Oracle::id`).
        oracle_id: String,
        /// Observed steward count.
        observed: usize,
    },
    /// An `Oracle.stewards[*].authorization_basis` is empty or whitespace-only.
    /// Per ADR-005 Amendment 2: WHY-records are required, not optional.
    OracleStewardAuthorizationBasisEmpty {
        /// Oracle id (`Oracle::id`).
        oracle_id: String,
        /// Offending steward name.
        steward_name: String,
    },
    /// A `StateTransition.rationale` is empty or whitespace-only. Per
    /// ADR-005 Amendment 2 + ADR-021 §D3: every state transition MUST
    /// carry a non-empty rationale.
    OracleTransitionRationaleEmpty {
        /// Oracle id (`Oracle::id`).
        oracle_id: String,
        /// 1-based index of the offending transition entry.
        transition_index: usize,
    },
    /// A `StateTransition.authorized_by` names a steward not present in
    /// the oracle's `stewards[*].name`. State transitions must be
    /// authorized by a declared steward; orphaned authorizations are
    /// rejected at parse time (ATK-021-15).
    OracleTransitionAuthorNotSteward {
        /// Oracle id (`Oracle::id`).
        oracle_id: String,
        /// 1-based index of the offending transition entry.
        transition_index: usize,
        /// The `authorized_by` value that doesn't match any steward.
        authorized_by: String,
    },
    /// Oracle transition dates are not chronologically monotonic. The
    /// transitions log is append-only and time-ordered; out-of-order
    /// entries indicate tamper or construction error.
    OracleTransitionsNotMonotonic {
        /// Oracle id (`Oracle::id`).
        oracle_id: String,
        /// 1-based index of the offending transition entry (the one
        /// whose date is older than the previous entry's).
        transition_index: usize,
    },
    /// An `Oracle.transitions` entry declares a `from` state that doesn't
    /// match the oracle's prior state in the log. State transitions must
    /// form a valid chain over the monotonic state machine.
    OracleTransitionFromMismatch {
        /// Oracle id (`Oracle::id`).
        oracle_id: String,
        /// 1-based index of the offending transition entry.
        transition_index: usize,
        /// The expected `from` state (the prior log entry's `to` or the
        /// oracle's initial state if this is the first transition).
        expected_from: String,
        /// The offending entry's `from` value.
        actual_from: String,
    },
    /// An oracle completion predicate (defining when an oracle reaches
    /// `Complete` state) references another oracle via `oracles_complete`.
    /// Per ADR-021 ATK-021-18: cross-oracle references in completion
    /// predicates create circular dependency deadlocks. Rejected at scaffold
    /// time.
    OracleCompletionPredicateContainsOracleRef {
        /// Oracle id whose completion predicate is being scaffolded.
        oracle_id: String,
    },
    /// A `StateTransition` was authorized at a strength weaker than the
    /// project-required minimum (`GitTrust` per ATK-021-15). `TextStamp`-
    /// authorized transitions are rejected at the CLI; audits flag pre-
    /// existing transitions written before this safeguard with the
    /// `oracle-transition-auth-insufficient` hint.
    OracleTransitionAuthInsufficient {
        /// Oracle id (`Oracle::id`).
        oracle_id: String,
        /// 1-based index of the offending transition entry.
        transition_index: usize,
    },
}

impl std::fmt::Display for ValidationError {
    // 13 variants, each with a precise multi-line error message; splitting
    // the match per-group would obscure the variant→message correspondence.
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyDeltaRationale {
                item_path,
                signer_name,
            } => write!(
                f,
                "signer `{signer_name}` at `{item_path}` has SignerBasis::DeltaFrom \
                 with empty rationale; non-empty rationale is required \
                 (ADR-019 anti-laundering safeguard #3)"
            ),
            Self::ChainDepthExceeded {
                item_path,
                signer_name,
                chain_depth,
                cap,
            } => write!(
                f,
                "signer `{signer_name}` at `{item_path}` has chain_depth {chain_depth} \
                 > cap {cap}; signer must do a Fresh re-attestation \
                 (ADR-019 anti-laundering safeguard #1)"
            ),
            Self::ZeroDeltaChainDepth {
                item_path,
                signer_name,
            } => write!(
                f,
                "signer `{signer_name}` at `{item_path}` has SignerBasis::DeltaFrom \
                 with chain_depth = 0; chain_depth must be >= 1 for delta entries \
                 (chain_depth = 0 is reserved for Fresh basis)"
            ),
            Self::RationaleTooShort {
                item_path,
                signer_name,
                actual_chars,
                min_chars,
            } => write!(
                f,
                "signer `{signer_name}` at `{item_path}` has SignerBasis::DeltaFrom \
                 with rationale of {actual_chars} chars; minimum is {min_chars} \
                 (ADR-019 anti-laundering safeguard #3 — prevents rubber-stamp \
                 rationales like 'ok' / 'fine' / 'reviewed')"
            ),
            Self::InconsistentCumulativeRoot {
                item_path,
                signer_name,
                prior_fingerprint,
                cumulative_root_fingerprint,
            } => write!(
                f,
                "signer `{signer_name}` at `{item_path}` has SignerBasis::DeltaFrom \
                 chain_depth=1 where cumulative_root_fingerprint `{cumulative_root_fingerprint}` \
                 != prior_fingerprint `{prior_fingerprint}`; at depth 1 these must be identical \
                 (ADR-019 anti-laundering safeguard #2 — cumulative root must be the prior Fresh)"
            ),
            Self::StrengthSignatureMismatch {
                item_path,
                signer_name,
            } => write!(
                f,
                "signer `{signer_name}` at `{item_path}` claims `strength = CryptoSigned` \
                 but carries no `signature` field; CryptoSigned requires a DSSE-PAE \
                 cryptographic signature envelope (NFA-17 — tier inflation without backing)"
            ),
            Self::WorkspaceConfigOutOfBounds {
                key,
                value,
                min,
                max,
            } if *max == u64::MAX => write!(
                f,
                "workspace config `{key} = {value}` violates hard floor: must be >= {min} \
                 (ADR-019 anti-laundering safeguard — workspaces cannot loosen below floor)"
            ),
            Self::WorkspaceConfigOutOfBounds {
                key,
                value,
                min,
                max,
            } => write!(
                f,
                "workspace config `{key} = {value}` out of bounds [{min}, {max}] \
                 (ADR-019 anti-laundering safeguard — workspaces cannot exceed \
                 project-enforced hard floor)"
            ),
            Self::OracleRequiresMinimumTwoStewards {
                oracle_id,
                observed,
            } => write!(
                f,
                "oracle `{oracle_id}` has {observed} steward(s); minimum 2 required \
                 (ADR-021 ATK-021-13 — succession mitigation)"
            ),
            Self::OracleStewardAuthorizationBasisEmpty {
                oracle_id,
                steward_name,
            } => write!(
                f,
                "oracle `{oracle_id}` steward `{steward_name}` has empty \
                 authorization_basis; non-empty WHY-record required \
                 (ADR-005 Amendment 2 inheritance per ADR-021 §D3)"
            ),
            Self::OracleTransitionRationaleEmpty {
                oracle_id,
                transition_index,
            } => write!(
                f,
                "oracle `{oracle_id}` transition #{transition_index} has empty \
                 rationale; non-empty WHY-record required for every state \
                 transition (ADR-005 Amendment 2 per ADR-021 §D3)"
            ),
            Self::OracleTransitionAuthorNotSteward {
                oracle_id,
                transition_index,
                authorized_by,
            } => write!(
                f,
                "oracle `{oracle_id}` transition #{transition_index} authorized_by \
                 `{authorized_by}` does not match any declared steward \
                 (ADR-021 ATK-021-15 — orphaned-authorization guard)"
            ),
            Self::OracleTransitionsNotMonotonic {
                oracle_id,
                transition_index,
            } => write!(
                f,
                "oracle `{oracle_id}` transition #{transition_index} date is \
                 older than the previous transition; the transitions log is \
                 append-only and time-ordered (ADR-021 §D3)"
            ),
            Self::OracleTransitionFromMismatch {
                oracle_id,
                transition_index,
                expected_from,
                actual_from,
            } => write!(
                f,
                "oracle `{oracle_id}` transition #{transition_index} declares \
                 from = `{actual_from}` but expected `{expected_from}` based on \
                 prior state in the chain (ADR-021 §D3 state-machine validity)"
            ),
            Self::OracleCompletionPredicateContainsOracleRef { oracle_id } => write!(
                f,
                "oracle `{oracle_id}` completion predicate references another \
                 oracle via oracles_complete; cross-oracle references in \
                 completion predicates create circular deadlock \
                 (ADR-021 ATK-021-18 — rejected at scaffold time)"
            ),
            Self::OracleTransitionAuthInsufficient {
                oracle_id,
                transition_index,
            } => write!(
                f,
                "oracle `{oracle_id}` transition #{transition_index} was \
                 authorized at strength below GitTrust minimum \
                 (ADR-021 ATK-021-15 — TextStamp-authorized transitions \
                 are rejected by the CLI)"
            ),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Default chain-depth cap per workspace (overridable in
/// `[package.metadata.antigen.attestation]` `delta_chain_cap`).
pub const DEFAULT_DELTA_CHAIN_CAP: u32 = 3;

/// Hard floor on chain-depth cap, NOT workspace-configurable.
///
/// Per adversarial T2R-C: workspaces can TIGHTEN the cap (set it lower than
/// the default) but cannot LOOSEN it beyond this floor. The CLI refuses
/// `[package.metadata.antigen.attestation] delta_chain_cap = N` when N > this
/// constant. Without a hard floor, a workspace TOML edit defeats the entire
/// anti-laundering safeguard.
pub const HARD_DELTA_CHAIN_CAP_MAX: u32 = 10;

/// Minimum hard floor on chain-depth cap. Workspaces cannot set the cap to
/// 0 (which would disable delta-chain enforcement entirely — per T2R-C the
/// CLI must refuse this).
pub const HARD_DELTA_CHAIN_CAP_MIN: u32 = 1;

/// Default minimum character count for `SignerBasis::DeltaFrom::rationale`.
///
/// Per adversarial T2R-B: non-empty alone permits rubber-stamp rationales
/// like `"ok"`, `"fine"`, `"reviewed"`. Schema enforces a minimum length so
/// the rationale carries actual signal. Workspaces can TIGHTEN via
/// `[package.metadata.antigen.attestation] delta_rationale_min_chars = N`
/// (subject to the hard floor below).
pub const DEFAULT_DELTA_RATIONALE_MIN_CHARS: usize = 20;

/// Minimum hard floor on rationale-length minimum. Workspaces cannot set
/// `delta_rationale_min_chars` below this; CLI refuses lower values.
pub const HARD_DELTA_RATIONALE_MIN_CHARS_FLOOR: usize = 10;

/// Validate that a workspace-configured `delta_chain_cap` value is within
/// the hard-floor bounds. Used by `cargo antigen` config-loading + by tests.
///
/// # Errors
///
/// Returns [`ValidationError::WorkspaceConfigOutOfBounds`] when the value
/// is below the minimum or above the maximum hard-floor bound.
pub fn validate_chain_cap(value: u32) -> Result<(), ValidationError> {
    if !(HARD_DELTA_CHAIN_CAP_MIN..=HARD_DELTA_CHAIN_CAP_MAX).contains(&value) {
        return Err(ValidationError::WorkspaceConfigOutOfBounds {
            key: "delta_chain_cap",
            value: u64::from(value),
            min: u64::from(HARD_DELTA_CHAIN_CAP_MIN),
            max: u64::from(HARD_DELTA_CHAIN_CAP_MAX),
        });
    }
    Ok(())
}

/// Validate that a workspace-configured `delta_rationale_min_chars` value is
/// within the hard-floor bounds.
///
/// # Errors
///
/// Returns [`ValidationError::WorkspaceConfigOutOfBounds`] when the value
/// is below the hard floor.
pub const fn validate_rationale_min_chars(value: usize) -> Result<(), ValidationError> {
    if value < HARD_DELTA_RATIONALE_MIN_CHARS_FLOOR {
        return Err(ValidationError::WorkspaceConfigOutOfBounds {
            key: "delta_rationale_min_chars",
            value: value as u64,
            min: HARD_DELTA_RATIONALE_MIN_CHARS_FLOOR as u64,
            max: u64::MAX,
        });
    }
    Ok(())
}

impl Ratification {
    /// Validate semantic invariants beyond what serde catches.
    ///
    /// Currently checks:
    /// - non-empty rationale on every `SignerBasis::DeltaFrom`
    /// - rationale length >= `rationale_min_chars` (default 20; T2R-B)
    /// - `chain_depth >= 1` on every `SignerBasis::DeltaFrom`
    /// - `chain_depth <= cap` on every `SignerBasis::DeltaFrom`
    /// - `strength == CryptoSigned` implies `signature.is_some()` (NFA-17)
    ///
    /// Returns the first failure encountered. Callers wanting all failures
    /// should walk `items` + `signers` directly. (For audit reporting, the
    /// "first failure" semantic is sufficient: a sidecar with any
    /// violation reports `discipline-sidecar-schema-invalid` and the audit
    /// shows the first violation as the actionable hint.)
    ///
    /// # Errors
    ///
    /// Returns the first [`ValidationError`] encountered.
    pub fn validate(&self, cap: u32, rationale_min_chars: usize) -> Result<(), ValidationError> {
        for item in &self.items {
            for signer in &item.signers {
                if let SignerBasis::DeltaFrom {
                    chain_depth,
                    rationale,
                    prior_fingerprint,
                    cumulative_root_fingerprint,
                } = &signer.basis
                {
                    // Structural invariants first (chain_depth = 0 is impossible
                    // for DeltaFrom; Fresh uses chain_depth = 0 via is_fresh()).
                    if *chain_depth == 0 {
                        return Err(ValidationError::ZeroDeltaChainDepth {
                            item_path: item.item_path.clone(),
                            signer_name: signer.name.clone(),
                        });
                    }
                    // Enforcement ratchet second (cap exceeded → must re-attest Fresh).
                    if *chain_depth > cap {
                        return Err(ValidationError::ChainDepthExceeded {
                            item_path: item.item_path.clone(),
                            signer_name: signer.name.clone(),
                            chain_depth: *chain_depth,
                            cap,
                        });
                    }
                    // Anti-laundering safeguard #2 consistency (NFA-12): at depth=1
                    // the cumulative root IS the prior (the immediately preceding
                    // Fresh signature). A discrepancy means the sidecar anchors its
                    // cumulative-diff tracking to a fingerprint that doesn't exist in
                    // this chain — a construction error or tamper indicator.
                    if *chain_depth == 1 && cumulative_root_fingerprint != prior_fingerprint {
                        return Err(ValidationError::InconsistentCumulativeRoot {
                            item_path: item.item_path.clone(),
                            signer_name: signer.name.clone(),
                            prior_fingerprint: prior_fingerprint.clone(),
                            cumulative_root_fingerprint: cumulative_root_fingerprint.clone(),
                        });
                    }
                    // Quality gates last (rationale content checks).
                    let trimmed = rationale.trim();
                    if trimmed.is_empty() {
                        return Err(ValidationError::EmptyDeltaRationale {
                            item_path: item.item_path.clone(),
                            signer_name: signer.name.clone(),
                        });
                    }
                    if trimmed.chars().count() < rationale_min_chars {
                        return Err(ValidationError::RationaleTooShort {
                            item_path: item.item_path.clone(),
                            signer_name: signer.name.clone(),
                            actual_chars: trimmed.chars().count(),
                            min_chars: rationale_min_chars,
                        });
                    }
                }
                // NFA-17: CryptoSigned tier requires the `signature` field to be present.
                // A signer claiming CryptoSigned without a cryptographic signature
                // envelope is tier inflation — the audit would report the max tier
                // without any cryptographic backing.
                if signer.strength == crate::tier::SignatureStrength::CryptoSigned
                    && signer.signature.is_none()
                {
                    return Err(ValidationError::StrengthSignatureMismatch {
                        item_path: item.item_path.clone(),
                        signer_name: signer.name.clone(),
                    });
                }
            }
            // ADR-021 §D3: validate every oracle attached to this item.
            for oracle in &item.oracles {
                oracle.validate()?;
            }
        }
        Ok(())
    }
}

impl Oracle {
    /// Validate ADR-021 §D3 invariants on this oracle.
    ///
    /// Catches: minimum-2-stewards (ATK-021-13); empty `authorization_basis`
    /// per steward (Amendment 2 inheritance); empty transition `rationale`
    /// (Amendment 2 per state transition); `authorized_by` ∈ `stewards[*].name`
    /// (ATK-021-15 orphaned-authorization guard); chronological monotonicity
    /// across the `transitions` log; well-formed state-machine `from`-chain.
    ///
    /// Legacy-import oracles (constructed via the two-pass deserialization
    /// fallback for v0.0 `OracleRef { path, status }` sidecars) carry an
    /// empty steward list and `<legacy-import>` provenance — these are
    /// intentionally exempted from the minimum-2-stewards check (the audit
    /// emits the `oracle-ref-needs-migration` hint instead). Operator
    /// remediation via `cargo antigen oracle declare --import-legacy ...`
    /// is the v0.1 forward path.
    ///
    /// # Errors
    ///
    /// Returns the first [`ValidationError`] encountered. Validation is
    /// short-circuit by design: a sidecar with any oracle invariant
    /// violation reports `discipline-sidecar-schema-invalid` and the audit
    /// surfaces the first violation as the actionable hint.
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Legacy-import sidecars carry `<legacy-import>` provenance + empty
        // steward list; they're rendered as legitimate Oracle values by the
        // two-pass deserialization but exempt from steward-minimum until
        // the operator runs `oracle declare --import-legacy`.
        let is_legacy_import =
            self.created.recorded_by == "<legacy-import>" && self.stewards.is_empty();

        if !is_legacy_import {
            // ATK-021-13: succession mitigation.
            if self.stewards.len() < 2 {
                return Err(ValidationError::OracleRequiresMinimumTwoStewards {
                    oracle_id: self.id.clone(),
                    observed: self.stewards.len(),
                });
            }
            // Amendment 2: every steward MUST carry a non-empty authorization_basis.
            for steward in &self.stewards {
                if steward.authorization_basis.trim().is_empty() {
                    return Err(ValidationError::OracleStewardAuthorizationBasisEmpty {
                        oracle_id: self.id.clone(),
                        steward_name: steward.name.clone(),
                    });
                }
            }
        }

        // Walk the transitions log.
        let steward_names: std::collections::BTreeSet<&str> =
            self.stewards.iter().map(|s| s.name.as_str()).collect();
        let mut prior_state_name: String =
            oracle_state_discriminant(&OracleState::Draft).to_string();
        let mut prior_date: Option<NaiveDate> = None;
        for (idx0, transition) in self.transitions.iter().enumerate() {
            let idx1 = idx0 + 1;

            // Amendment 2: every transition MUST carry a non-empty rationale.
            if transition.rationale.trim().is_empty() {
                return Err(ValidationError::OracleTransitionRationaleEmpty {
                    oracle_id: self.id.clone(),
                    transition_index: idx1,
                });
            }

            // ATK-021-15: authorized_by ∈ stewards[*].name.
            // Skip this check for legacy-import oracles (no stewards yet).
            if !is_legacy_import && !steward_names.contains(transition.authorized_by.as_str()) {
                return Err(ValidationError::OracleTransitionAuthorNotSteward {
                    oracle_id: self.id.clone(),
                    transition_index: idx1,
                    authorized_by: transition.authorized_by.clone(),
                });
            }

            // Chronological monotonicity.
            if let Some(prev_date) = prior_date {
                if transition.at < prev_date {
                    return Err(ValidationError::OracleTransitionsNotMonotonic {
                        oracle_id: self.id.clone(),
                        transition_index: idx1,
                    });
                }
            }
            prior_date = Some(transition.at);

            // State-machine validity: `from` must match the prior state in the chain.
            if transition.from != prior_state_name {
                return Err(ValidationError::OracleTransitionFromMismatch {
                    oracle_id: self.id.clone(),
                    transition_index: idx1,
                    expected_from: prior_state_name,
                    actual_from: transition.from.clone(),
                });
            }
            prior_state_name.clone_from(&transition.to);
        }

        Ok(())
    }
}

/// `snake_case` discriminant name for an `OracleState`. Matches the serde
/// `tag = "state", rename_all = "snake_case"` rendering used in
/// `StateTransition.from` / `to`.
const fn oracle_state_discriminant(state: &OracleState) -> &'static str {
    match state {
        OracleState::Draft => "draft",
        OracleState::Complete => "complete",
        OracleState::Deprecated { .. } => "deprecated",
        OracleState::Retired { .. } => "retired",
        OracleState::Revoked { .. } => "revoked",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signer_fresh(name: &str, date: NaiveDate, fp: &str) -> Signer {
        Signer {
            name: name.to_string(),
            role: None,
            date,
            signed_against_fingerprint: fp.to_string(),
            basis: SignerBasis::Fresh { reasoning: None },
            strength: crate::tier::SignatureStrength::GitTrust,
            signature: None,
        }
    }

    fn signer_delta(name: &str, date: NaiveDate, depth: u32, rationale: &str) -> Signer {
        // For depth=1, cumulative_root_fingerprint must equal prior_fingerprint
        // (NFA-12 invariant: at chain_depth=1 the root IS the prior).
        // For depth>1 the root differs from the immediate prior; use a distinct fixture value.
        let prior = "fp-prior".to_string();
        let cumulative_root = if depth == 1 {
            prior.clone()
        } else {
            "fp-root".to_string()
        };
        Signer {
            name: name.to_string(),
            role: None,
            date,
            signed_against_fingerprint: "fp-current".to_string(),
            basis: SignerBasis::DeltaFrom {
                prior_fingerprint: prior,
                cumulative_root_fingerprint: cumulative_root,
                chain_depth: depth,
                rationale: rationale.to_string(),
            },
            strength: crate::tier::SignatureStrength::GitTrust,
            signature: None,
        }
    }

    fn item_with_signers(item: &str, signers: Vec<Signer>) -> ItemRatification {
        ItemRatification {
            item_path: item.to_string(),
            current_fingerprint: "fp-current".to_string(),
            doc_ref: None,
            signers,
            oracles: vec![],
            fresh_through: None,
            extensions: BTreeMap::new(),
        }
    }

    fn ratification_with_items(items: Vec<ItemRatification>) -> Ratification {
        Ratification {
            schema_version: SchemaVersion::V1,
            kind: RatificationKind::Immunity,
            antigen: AntigenIdentifier {
                name: "TestAntigen".to_string(),
                defined_in: None,
            },
            source_file: PathBuf::from("src/test.rs"),
            items,
        }
    }

    #[test]
    fn fresh_signer_passes_validate() {
        let date = NaiveDate::from_ymd_opt(2026, 5, 19).unwrap();
        let item = item_with_signers("test_item", vec![signer_fresh("alice", date, "fp-current")]);
        let rat = ratification_with_items(vec![item]);
        assert!(rat
            .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
            .is_ok());
    }

    #[test]
    fn delta_with_empty_rationale_rejected() {
        let date = NaiveDate::from_ymd_opt(2026, 5, 19).unwrap();
        let item = item_with_signers("test_item", vec![signer_delta("alice", date, 1, "")]);
        let rat = ratification_with_items(vec![item]);
        let err = rat
            .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
            .unwrap_err();
        assert!(matches!(err, ValidationError::EmptyDeltaRationale { .. }));
    }

    #[test]
    fn delta_with_whitespace_only_rationale_rejected() {
        let date = NaiveDate::from_ymd_opt(2026, 5, 19).unwrap();
        let item = item_with_signers(
            "test_item",
            vec![signer_delta("alice", date, 1, "   \t\n  ")],
        );
        let rat = ratification_with_items(vec![item]);
        let err = rat
            .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
            .unwrap_err();
        assert!(matches!(err, ValidationError::EmptyDeltaRationale { .. }));
    }

    /// Test-fixture rationale text >= `DEFAULT_DELTA_RATIONALE_MIN_CHARS` so
    /// the rationale-too-short rule (T2R-B) doesn't fire in tests that
    /// aren't exercising it specifically.
    const VALID_DELTA_RATIONALE: &str = "reviewed diff against prior; invariant-preserving change";

    #[test]
    fn delta_with_zero_chain_depth_rejected() {
        let date = NaiveDate::from_ymd_opt(2026, 5, 19).unwrap();
        let item = item_with_signers(
            "test_item",
            vec![signer_delta("alice", date, 0, VALID_DELTA_RATIONALE)],
        );
        let rat = ratification_with_items(vec![item]);
        let err = rat
            .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
            .unwrap_err();
        assert!(matches!(err, ValidationError::ZeroDeltaChainDepth { .. }));
    }

    #[test]
    fn delta_exceeding_cap_rejected() {
        let date = NaiveDate::from_ymd_opt(2026, 5, 19).unwrap();
        let item = item_with_signers(
            "test_item",
            vec![signer_delta("alice", date, 4, VALID_DELTA_RATIONALE)],
        );
        let rat = ratification_with_items(vec![item]);
        let err = rat
            .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
            .unwrap_err();
        let ValidationError::ChainDepthExceeded {
            chain_depth, cap, ..
        } = err
        else {
            panic!("expected ChainDepthExceeded");
        };
        assert_eq!(chain_depth, 4);
        assert_eq!(cap, DEFAULT_DELTA_CHAIN_CAP);
    }

    #[test]
    fn delta_at_cap_accepted() {
        let date = NaiveDate::from_ymd_opt(2026, 5, 19).unwrap();
        let item = item_with_signers(
            "test_item",
            vec![signer_delta("alice", date, 3, VALID_DELTA_RATIONALE)],
        );
        let rat = ratification_with_items(vec![item]);
        assert!(rat
            .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
            .is_ok());
    }

    #[test]
    fn delta_with_rubber_stamp_rationale_rejected_t2r_b() {
        // Per adversarial T2R-B: "ok", "fine", "reviewed" all pass non-empty
        // but the minimum-length floor catches them.
        let date = NaiveDate::from_ymd_opt(2026, 5, 19).unwrap();
        for rubber_stamp in &["ok", "fine", "reviewed", "changes are safe"] {
            let item = item_with_signers(
                "test_item",
                vec![signer_delta("alice", date, 1, rubber_stamp)],
            );
            let rat = ratification_with_items(vec![item]);
            let err = rat
                .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
                .unwrap_err();
            let ValidationError::RationaleTooShort {
                actual_chars,
                min_chars,
                ..
            } = err
            else {
                panic!("expected RationaleTooShort for `{rubber_stamp}`");
            };
            assert_eq!(min_chars, DEFAULT_DELTA_RATIONALE_MIN_CHARS);
            assert!(actual_chars < DEFAULT_DELTA_RATIONALE_MIN_CHARS);
        }
    }

    #[test]
    fn validate_chain_cap_rejects_below_floor_t2r_c() {
        let err = validate_chain_cap(0).unwrap_err();
        assert!(matches!(
            err,
            ValidationError::WorkspaceConfigOutOfBounds {
                key: "delta_chain_cap",
                ..
            }
        ));
    }

    #[test]
    fn validate_chain_cap_rejects_above_floor_t2r_c() {
        let err = validate_chain_cap(999).unwrap_err();
        let ValidationError::WorkspaceConfigOutOfBounds { value, max, .. } = err else {
            panic!("expected WorkspaceConfigOutOfBounds");
        };
        assert_eq!(value, 999);
        assert_eq!(max, u64::from(HARD_DELTA_CHAIN_CAP_MAX));
    }

    #[test]
    fn validate_chain_cap_accepts_within_bounds() {
        assert!(validate_chain_cap(1).is_ok());
        assert!(validate_chain_cap(DEFAULT_DELTA_CHAIN_CAP).is_ok());
        assert!(validate_chain_cap(HARD_DELTA_CHAIN_CAP_MAX).is_ok());
    }

    #[test]
    fn validate_rationale_min_chars_rejects_below_floor() {
        let err = validate_rationale_min_chars(5).unwrap_err();
        assert!(matches!(
            err,
            ValidationError::WorkspaceConfigOutOfBounds {
                key: "delta_rationale_min_chars",
                ..
            }
        ));
    }

    #[test]
    fn validate_rationale_min_chars_accepts_floor_and_above() {
        assert!(validate_rationale_min_chars(HARD_DELTA_RATIONALE_MIN_CHARS_FLOOR).is_ok());
        assert!(validate_rationale_min_chars(DEFAULT_DELTA_RATIONALE_MIN_CHARS).is_ok());
        assert!(validate_rationale_min_chars(100).is_ok());
    }

    #[test]
    fn round_trip_via_serde_json() {
        let date = NaiveDate::from_ymd_opt(2026, 5, 19).unwrap();
        let item = item_with_signers(
            "test_item",
            vec![
                signer_fresh("alice", date, "fp-current"),
                signer_delta("bob", date, 2, VALID_DELTA_RATIONALE),
            ],
        );
        let rat = ratification_with_items(vec![item]);
        let json = serde_json::to_string_pretty(&rat).unwrap();
        let parsed: Ratification = serde_json::from_str(&json).unwrap();
        assert_eq!(rat, parsed);
    }

    #[test]
    fn signer_basis_chain_depth_zero_for_fresh() {
        let basis = SignerBasis::Fresh { reasoning: None };
        assert_eq!(basis.chain_depth(), 0);
        assert!(basis.is_fresh());
        assert!(!basis.is_delta());
    }

    #[test]
    fn signer_basis_fresh_carries_optional_reasoning() {
        let basis = SignerBasis::Fresh {
            reasoning: Some("audited under Sub-pattern 5.11".to_string()),
        };
        assert!(basis.is_fresh());
        let json = serde_json::to_string(&basis).unwrap();
        assert!(json.contains("\"reasoning\""));
        let parsed: SignerBasis = serde_json::from_str(&json).unwrap();
        assert_eq!(basis, parsed);
    }

    #[test]
    fn signer_basis_fresh_skips_reasoning_when_none() {
        let basis = SignerBasis::Fresh { reasoning: None };
        let json = serde_json::to_string(&basis).unwrap();
        // None reasoning skipped via skip_serializing_if; JSON has just the kind tag.
        assert!(!json.contains("reasoning"));
    }

    #[test]
    fn signer_basis_chain_depth_for_delta() {
        let basis = SignerBasis::DeltaFrom {
            prior_fingerprint: "p".to_string(),
            cumulative_root_fingerprint: "r".to_string(),
            chain_depth: 2,
            rationale: "x".to_string(),
        };
        assert_eq!(basis.chain_depth(), 2);
        assert!(!basis.is_fresh());
        assert!(basis.is_delta());
    }

    #[test]
    fn nfa_17_crypto_signed_without_signature_rejected() {
        // BUG REGRESSION TEST (NFA-17): `strength = CryptoSigned` without a
        // `signature` field is tier inflation — CryptoSigned requires a DSSE-PAE
        // envelope. validate() must reject this before it reaches the audit,
        // which would otherwise report the maximum tier with no cryptographic backing.
        let date = NaiveDate::from_ymd_opt(2026, 5, 19).unwrap();
        let signer = Signer {
            name: "alice".to_string(),
            role: None,
            date,
            signed_against_fingerprint: "fp-current".to_string(),
            basis: SignerBasis::Fresh { reasoning: None },
            strength: crate::tier::SignatureStrength::CryptoSigned,
            signature: None, // NFA-17: CryptoSigned with no envelope
        };
        let item = item_with_signers("test_item", vec![signer]);
        let rat = ratification_with_items(vec![item]);
        let err = rat
            .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
            .expect_err("CryptoSigned signer with no signature field must be rejected");
        assert!(
            matches!(err, ValidationError::StrengthSignatureMismatch { .. }),
            "expected StrengthSignatureMismatch, got {err:?}"
        );
        // Lock the variant fields.
        match err {
            ValidationError::StrengthSignatureMismatch {
                item_path,
                signer_name,
            } => {
                assert_eq!(item_path, "test_item");
                assert_eq!(signer_name, "alice");
            }
            other => panic!("expected StrengthSignatureMismatch, got {other:?}"),
        }
    }

    // =========================================================================
    // Oracle validation tests (ADR-021 §D3 Model B invariants)
    // =========================================================================

    fn sample_date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 5, 20).unwrap()
    }

    /// Build a minimal valid Oracle with two stewards. Tests adjust individual
    /// fields to exercise specific validation paths.
    fn valid_oracle(id: &str) -> Oracle {
        Oracle {
            id: id.to_string(),
            reference: OracleRef::LocalFile {
                path: std::path::PathBuf::from("docs/oracles/test.md"),
                status_field: None,
                expected_status: None,
            },
            state: OracleState::Complete,
            stewards: vec![
                Steward {
                    name: "alice".to_string(),
                    role: None,
                    authorization_basis: "domain authority on floating-point numerics".to_string(),
                },
                Steward {
                    name: "bob".to_string(),
                    role: None,
                    authorization_basis: "tech-lead appointed 2026-03-01".to_string(),
                },
            ],
            created: Provenance {
                recorded_by: "alice".to_string(),
                at: sample_date(),
            },
            version: OracleVersion {
                pinned: "2026-05-20".to_string(),
                pinned_at: sample_date(),
            },
            transitions: vec![],
            extensions: BTreeMap::new(),
        }
    }

    fn ratification_with_oracle(oracle: Oracle) -> Ratification {
        Ratification {
            schema_version: SchemaVersion::V1,
            kind: RatificationKind::Immunity,
            antigen: AntigenIdentifier {
                name: "TestAntigen".to_string(),
                defined_in: None,
            },
            source_file: std::path::PathBuf::from("src/test.rs"),
            items: vec![ItemRatification {
                item_path: "test_item".to_string(),
                current_fingerprint: "fp-current".to_string(),
                doc_ref: None,
                signers: vec![],
                oracles: vec![oracle],
                fresh_through: None,
                extensions: BTreeMap::new(),
            }],
        }
    }

    #[test]
    fn oracle_with_two_stewards_passes_validate_atk021_13() {
        // A well-formed oracle with >= 2 stewards must pass.
        let oracle = valid_oracle("test-oracle");
        let rat = ratification_with_oracle(oracle);
        assert!(
            rat.validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
                .is_ok(),
            "valid oracle with 2 stewards must pass validate()"
        );
    }

    #[test]
    fn oracle_with_one_steward_rejected_atk021_13() {
        // BUG REGRESSION TEST (adversarial ATK-021-13): single-steward oracles
        // must be rejected — if the lone steward leaves, the oracle becomes
        // permanently orphaned with no succession path.
        let mut oracle = valid_oracle("single-steward-oracle");
        oracle.stewards.truncate(1); // only alice, no bob
        let rat = ratification_with_oracle(oracle);
        let err = rat
            .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
            .expect_err("single-steward oracle must be rejected (ATK-021-13)");
        assert!(
            matches!(
                err,
                ValidationError::OracleRequiresMinimumTwoStewards { observed: 1, .. }
            ),
            "expected OracleRequiresMinimumTwoStewards, got {err:?}"
        );
    }

    #[test]
    fn oracle_with_zero_stewards_rejected_atk021_13() {
        let mut oracle = valid_oracle("no-steward-oracle");
        oracle.stewards.clear();
        let rat = ratification_with_oracle(oracle);
        let err = rat
            .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
            .expect_err("zero-steward oracle must be rejected");
        assert!(
            matches!(
                err,
                ValidationError::OracleRequiresMinimumTwoStewards { observed: 0, .. }
            ),
            "expected OracleRequiresMinimumTwoStewards, got {err:?}"
        );
    }

    #[test]
    fn oracle_steward_with_empty_authorization_basis_rejected() {
        // BUG REGRESSION TEST: ADR-005 Amendment 2 requires WHY-records.
        // An empty authorization_basis on a steward is a silent bypass of
        // the rationale-as-visibility discipline.
        let mut oracle = valid_oracle("empty-basis-oracle");
        oracle.stewards[0].authorization_basis = String::new(); // empty
        let rat = ratification_with_oracle(oracle);
        let err = rat
            .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
            .expect_err("steward with empty authorization_basis must be rejected");
        assert!(
            matches!(
                err,
                ValidationError::OracleStewardAuthorizationBasisEmpty { .. }
            ),
            "expected OracleStewardAuthorizationBasisEmpty, got {err:?}"
        );
    }

    #[test]
    fn oracle_transition_with_unauthorized_author_rejected_atk021_15() {
        // BUG REGRESSION TEST (adversarial ATK-021-15): state transitions must
        // be authorized by a declared steward. A non-steward author is a silent
        // forgery — at TextStamp level anyone can claim to be a steward.
        let mut oracle = valid_oracle("bad-transition-oracle");
        oracle.transitions.push(StateTransition {
            from: "draft".to_string(),
            to: "complete".to_string(),
            authorized_by: "carol-not-a-steward".to_string(), // not in stewards
            at: sample_date(),
            rationale: "oracle review complete; all examples verified".to_string(),
        });
        // Note: oracle.state is Complete but transitions log starts from Draft.
        // The from-chain check fires BEFORE the authorized_by check in this
        // configuration because draft→complete is valid as the first transition.
        // Actually both alice and bob are stewards; carol is not.
        let rat = ratification_with_oracle(oracle);
        let err = rat
            .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
            .expect_err("transition by non-steward must be rejected (ATK-021-15)");
        assert!(
            matches!(
                err,
                ValidationError::OracleTransitionAuthorNotSteward { .. }
            ),
            "expected OracleTransitionAuthorNotSteward, got {err:?}"
        );
    }

    #[test]
    fn oracle_transition_with_empty_rationale_rejected() {
        // ADR-005 Amendment 2: every state transition must carry a non-empty rationale.
        let mut oracle = valid_oracle("empty-rationale-transition-oracle");
        oracle.transitions.push(StateTransition {
            from: "draft".to_string(),
            to: "complete".to_string(),
            authorized_by: "alice".to_string(),
            at: sample_date(),
            rationale: String::new(), // empty — rejected
        });
        let rat = ratification_with_oracle(oracle);
        let err = rat
            .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
            .expect_err("transition with empty rationale must be rejected");
        assert!(
            matches!(err, ValidationError::OracleTransitionRationaleEmpty { .. }),
            "expected OracleTransitionRationaleEmpty, got {err:?}"
        );
    }

    #[test]
    fn oracle_transitions_out_of_chronological_order_rejected() {
        // Transitions must be chronologically monotonic (append-only log).
        // Out-of-order dates indicate tamper or construction error.
        let mut oracle = valid_oracle("chronology-oracle");
        let earlier = NaiveDate::from_ymd_opt(2026, 4, 1).unwrap();
        let later = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
        oracle.transitions.push(StateTransition {
            from: "draft".to_string(),
            to: "complete".to_string(),
            authorized_by: "alice".to_string(),
            at: later, // first transition is at a later date
            rationale: "oracle review complete".to_string(),
        });
        oracle.transitions.push(StateTransition {
            from: "complete".to_string(),
            to: "deprecated".to_string(),
            authorized_by: "bob".to_string(),
            at: earlier, // second transition is at an earlier date — violation
            rationale: "superseded by new oracle".to_string(),
        });
        let rat = ratification_with_oracle(oracle);
        let err = rat
            .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
            .expect_err("out-of-order transition dates must be rejected");
        assert!(
            matches!(err, ValidationError::OracleTransitionsNotMonotonic { .. }),
            "expected OracleTransitionsNotMonotonic, got {err:?}"
        );
    }

    #[test]
    fn oracle_transition_from_mismatch_rejected() {
        // State machine integrity: transition.from must match the prior state.
        // A mismatch means the log is inconsistent (tamper or construction error).
        let mut oracle = valid_oracle("from-mismatch-oracle");
        oracle.transitions.push(StateTransition {
            from: "complete".to_string(), // wrong: initial state is Draft
            to: "deprecated".to_string(),
            authorized_by: "alice".to_string(),
            at: sample_date(),
            rationale: "superseded".to_string(),
        });
        let rat = ratification_with_oracle(oracle);
        let err = rat
            .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
            .expect_err("transition.from mismatch must be rejected");
        assert!(
            matches!(err, ValidationError::OracleTransitionFromMismatch { .. }),
            "expected OracleTransitionFromMismatch, got {err:?}"
        );
    }

    #[test]
    fn legacy_import_oracle_exempt_from_steward_minimum() {
        // Two-pass deserialization produces legacy-import oracles with empty
        // steward lists. These must pass validate() without the minimum-2-stewards
        // check, emitting oracle-ref-needs-migration hint at audit time instead.
        let oracle = Oracle {
            id: "legacy-path".to_string(),
            reference: OracleRef::LocalFile {
                path: std::path::PathBuf::from("docs/oracle.md"),
                status_field: None,
                expected_status: Some("complete".to_string()),
            },
            state: OracleState::Draft,
            stewards: vec![],
            created: Provenance {
                recorded_by: "<legacy-import>".to_string(), // sentinel
                at: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            },
            version: OracleVersion {
                pinned: "<legacy-import>".to_string(),
                pinned_at: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            },
            transitions: vec![],
            extensions: BTreeMap::new(),
        };
        let rat = ratification_with_oracle(oracle);
        assert!(
            rat.validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
                .is_ok(),
            "legacy-import oracle with zero stewards must pass validate() \
             (migration hint emitted at audit time instead)"
        );
    }
}
