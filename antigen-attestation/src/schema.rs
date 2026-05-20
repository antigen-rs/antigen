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
//! the antigen's [`scope` field](Self::source_file) — site / file / package /
//! workspace.

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
    /// Oracle-file references used by `oracles_complete(...)` leaf.
    #[serde(default)]
    pub oracles: Vec<OracleRef>,
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

/// A reference to an oracle-completion file used by `oracles_complete(...)`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleRef {
    /// Path to the oracle file, relative to workspace root.
    pub path: PathBuf,
    /// Optional expected status (default: `"complete"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
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
}

impl std::fmt::Display for ValidationError {
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
        }
        Ok(())
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
}
