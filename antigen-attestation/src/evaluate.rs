//! Substrate-witness predicate evaluator (ADR-019 §M5).
//!
//! Given a [`Predicate`] + a [`crate::schema::Ratification`] sidecar + an evaluation
//! context (substrate IO + audit-time clock + workspace config), produces
//! a [`(WitnessTier, AuditHint, EvidenceKind, Option<SignatureStrength>)`]
//! per the state-mapping table in ADR-019 §M5.
//!
//! ## Substrate-vs-machinery unification asymmetry
//!
//! Per ADR-019 §Decision + aristotle F1 + adversarial T5-R: substrate-
//! witness evaluation shares **discipline**-level unification with
//! cross-crate witness evaluation (both follow tier-honesty; both have
//! `SubstrateState` evidence kind; both cap at `Execution`). They do NOT
//! share **machinery** — the parsers, IO surfaces, attack surfaces, and
//! recovery semantics are distinct.
//!
//! **DO NOT share parser code between this module and
//! `antigen::scan`'s Rust-AST path.** A future maintainer might read
//! "discipline unification" and try to share — that would silently
//! mis-classify a JSON-shaped payload as Rust-AST-shaped (or vice
//! versa). The adversarial precision test in
//! `antigen/tests/atk_a3_unification_guardrail.rs` (when integrated) FAILS
//! if shared parsing is wired up.
//!
//! ## Pure-logic core; IO at the edges
//!
//! The evaluation logic is pure given an [`EvaluationContext`] trait
//! implementation. The default `FilesystemContext` does real IO; tests
//! supply in-memory contexts. This keeps the evaluator unit-testable
//! without filesystem fixtures.

use std::path::Path;

use chrono::NaiveDate;

use crate::predicate::{Leaf, Predicate, SignerCurrency};
use crate::schema::{ItemRatification, RatificationKind, Signer};
use crate::tier::{AuditHint, EvidenceKind, SignatureStrength, WitnessTier};

/// The IO + clock surface the evaluator depends on. Implementations
/// supply real-disk readers in production; tests supply in-memory
/// equivalents.
pub trait EvaluationContext {
    /// Current date for `fresh_within_days` evaluation. Audit invocations
    /// supply `chrono::Local::now().date_naive()`; tests inject deterministic
    /// dates.
    fn today(&self) -> NaiveDate;

    /// Read a doc file. Returns the file content. None on missing file
    /// or read error (predicate evaluation treats both as "missing").
    fn read_doc(&self, path: &Path) -> Option<String>;

    /// Read an oracle file. Returns its content. None on missing/error.
    fn read_oracle(&self, path: &Path) -> Option<String>;

    /// Query git log on the file containing the item; return the
    /// concatenated trailer entries from `git interpret-trailers` for
    /// commits touching this item. Returns empty vec when no commits
    /// match or git is unavailable.
    ///
    /// Each returned entry is one rendered trailer line, e.g.,
    /// `"Discipline-Verified-By: alice <alice@example.com>"`.
    fn read_git_trailers(&self, item_source_file: &Path, item_path: &str) -> Vec<String>;

    /// The configured chain-depth cap for delta entries (per workspace
    /// `[package.metadata.antigen.attestation]` `delta_chain_cap`;
    /// default 3).
    fn delta_chain_cap(&self) -> u32 {
        crate::schema::DEFAULT_DELTA_CHAIN_CAP
    }
}

/// The result of evaluating a substrate-witness predicate at one
/// presented item-site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluatedPredicate {
    /// Strength of evidence the substrate provided (per the state-
    /// mapping table in ADR-019 §M5).
    pub witness_tier: WitnessTier,
    /// Per-case audit-hint disambiguation.
    pub audit_hint: AuditHint,
    /// Always `EvidenceKind::SubstrateState` for substrate-witness
    /// predicates (the kind axis is constant for this evaluator;
    /// other evidence kinds come from other evaluators).
    pub evidence_kind: EvidenceKind,
    /// Strength of the signer-identity binding. `Some(GitTrust)` for
    /// v0.1; `None` when no signers exist (e.g., predicate failed
    /// before signer-check).
    pub signature_strength: Option<SignatureStrength>,
}

impl EvaluatedPredicate {
    /// Build the "sidecar missing" result. Used by the audit when no
    /// sidecar file exists for an antigen.
    #[must_use]
    pub const fn sidecar_missing() -> Self {
        Self {
            witness_tier: WitnessTier::None,
            audit_hint: AuditHint::DisciplineSidecarMissing,
            evidence_kind: EvidenceKind::SubstrateState,
            signature_strength: None,
        }
    }

    /// Build the "sidecar schema invalid" result.
    #[must_use]
    pub const fn sidecar_schema_invalid() -> Self {
        Self {
            witness_tier: WitnessTier::None,
            audit_hint: AuditHint::DisciplineSidecarSchemaInvalid,
            evidence_kind: EvidenceKind::SubstrateState,
            signature_strength: None,
        }
    }

    /// Build the "tolerance vibes-grade" result for `#[antigen_tolerance]`
    /// without `sidecar = true` opt-in. `EvidenceKind` is `None` here (not
    /// `SubstrateState`) to surface that no substrate was consulted at all.
    #[must_use]
    pub const fn tolerance_vibes_grade() -> Self {
        Self {
            witness_tier: WitnessTier::None,
            audit_hint: AuditHint::ToleranceVibesGrade,
            evidence_kind: EvidenceKind::None,
            signature_strength: None,
        }
    }
}

/// Errors that can arise during predicate evaluation. Distinct from
/// "predicate failed" — these are infrastructure failures (corrupt
/// sidecar; impossible context state).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvaluationError {
    /// Predicate references an item-path not present in the sidecar.
    /// The audit treats this as "sidecar exists but doesn't cover the
    /// item" — equivalent to sidecar-missing for that item.
    ItemNotFoundInSidecar {
        /// Item path that was looked up but not found.
        item_path: String,
    },
}

impl std::fmt::Display for EvaluationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ItemNotFoundInSidecar { item_path } => {
                write!(f, "item `{item_path}` not found in sidecar")
            }
        }
    }
}

impl std::error::Error for EvaluationError {}

/// Evaluate a substrate-witness predicate against a sidecar item entry
/// and return the audit-axes result.
///
/// `current_fingerprint` is the fingerprint of the item as recomputed by
/// the audit (not the sidecar's stored value). The function uses both:
/// the stored fingerprint feeds signer-currency comparison; the audit-
/// recomputed value feeds drift detection.
///
/// # Errors
///
/// Currently never returns Err — all paths produce an `EvaluatedPredicate`
/// (the error type is reserved for future infrastructure failure modes
/// like corrupt sidecar binary content). Callers should still match on
/// Result so future error variants don't silently change behavior.
pub fn evaluate_predicate<C: EvaluationContext>(
    predicate: &Predicate,
    item: &ItemRatification,
    current_fingerprint: &str,
    item_source_file: &Path,
    ctx: &C,
) -> Result<EvaluatedPredicate, EvaluationError> {
    evaluate_predicate_with_kind(
        predicate,
        item,
        current_fingerprint,
        item_source_file,
        RatificationKind::Immunity,
        ctx,
    )
}

/// Kind-aware variant of [`evaluate_predicate`].
///
/// Pass the sidecar's `RatificationKind` so tolerance sidecars emit
/// tolerance-specific audit hints (`TolerancePredicatePassedSubstrateCurrent`,
/// `TolerancePredicateFailed`) instead of the immunity equivalents.
///
/// # Errors
///
/// Currently never returns Err (see [`evaluate_predicate`]).
pub fn evaluate_predicate_with_kind<C: EvaluationContext>(
    predicate: &Predicate,
    item: &ItemRatification,
    current_fingerprint: &str,
    item_source_file: &Path,
    kind: RatificationKind,
    ctx: &C,
) -> Result<EvaluatedPredicate, EvaluationError> {
    // 1. Validate the predicate is well-formed (no zero-leaf compositions
    //    snuck through deserialization).
    if predicate.validate().is_err() {
        return Ok(EvaluatedPredicate::sidecar_schema_invalid());
    }

    // 2. Recursively evaluate the predicate. The recursive evaluator
    //    returns just bool (predicate-pass result); axis derivation
    //    happens after we know the boolean result + the per-signer
    //    delta-chain + freshness state of the sidecar.
    let predicate_passed = eval_pred(predicate, item, current_fingerprint, item_source_file, ctx);

    if !predicate_passed {
        let failed_hint = match kind {
            RatificationKind::Tolerance => AuditHint::TolerancePredicateFailed,
            RatificationKind::Immunity => AuditHint::DisciplinePredicateFailed,
        };
        return Ok(EvaluatedPredicate {
            witness_tier: WitnessTier::None,
            audit_hint: failed_hint,
            evidence_kind: EvidenceKind::SubstrateState,
            signature_strength: None,
        });
    }

    // 3. Predicate passed. Derive tier + hint from the sidecar's signer
    //    state: stale signers; delta-chain-near-cap; via-delta-chain;
    //    all-fresh. The M5 table differs between immunity and tolerance.
    Ok(classify_passed_predicate(
        item,
        current_fingerprint,
        kind,
        ctx,
    ))
}

/// Recursive predicate evaluation. Returns `true` if the predicate
/// passes against the sidecar + IO context. Pure-bool result; axis
/// classification happens in the caller after we know the bool plus
/// the per-signer state.
fn eval_pred<C: EvaluationContext>(
    p: &Predicate,
    item: &ItemRatification,
    current_fingerprint: &str,
    item_source_file: &Path,
    ctx: &C,
) -> bool {
    match p {
        Predicate::Leaf(leaf) => eval_leaf(leaf, item, current_fingerprint, item_source_file, ctx),
        Predicate::AllOf { children } => children
            .iter()
            .all(|c| eval_pred(c, item, current_fingerprint, item_source_file, ctx)),
        Predicate::AnyOf { children } => children
            .iter()
            .any(|c| eval_pred(c, item, current_fingerprint, item_source_file, ctx)),
        Predicate::Not { child } => {
            !eval_pred(child, item, current_fingerprint, item_source_file, ctx)
        }
    }
}

fn eval_leaf<C: EvaluationContext>(
    leaf: &Leaf,
    item: &ItemRatification,
    current_fingerprint: &str,
    item_source_file: &Path,
    ctx: &C,
) -> bool {
    match leaf {
        Leaf::RatifiedDoc {
            path,
            min_version,
            anchor,
            sibling_json,
        } => eval_ratified_doc(
            path.as_deref(),
            min_version.as_deref(),
            anchor.as_deref(),
            *sibling_json,
            item,
            ctx,
        ),
        Leaf::Signers {
            required,
            roles,
            against,
            signature_allow,
            signature_prefer,
        } => eval_signers(
            required,
            roles,
            *against,
            signature_allow,
            *signature_prefer,
            item,
            current_fingerprint,
        ),
        Leaf::SignedTrailer { key, role, count } => eval_signed_trailer(
            key,
            role.as_deref(),
            *count,
            item_source_file,
            &item.item_path,
            ctx,
        ),
        Leaf::OraclesComplete { files } => eval_oracles_complete(files, item, ctx),
        Leaf::FreshWithinDays { days } => {
            eval_fresh_within_days(*days, item, current_fingerprint, ctx)
        }
        // Supply-chain leaf types (ADR-025). These cannot be evaluated by the
        // standard predicate evaluator — they require reading Cargo.lock, dep-
        // attestation sidecars, or (v0.4+) sandbox execution. The standard
        // `audit()` pipeline skips them; callers must drive `audit_supply_chain()`
        // in `antigen::supply_chain::evaluate` separately.
        //
        // Per ADR-005 Amendment 2 (honest-tier-naming): returning `false` here
        // is correct — these leaves do NOT pass in the standard path, and the
        // supply-chain audit layer is responsible for their evaluation. An
        // implicit-pass (`true`) would be worse than an implicit-fail.
        Leaf::DepPinned { .. }
        | Leaf::DepAttested { .. }
        | Leaf::MaintainerUnchanged { .. }
        | Leaf::ContentHashMatches { .. }
        | Leaf::SandboxClean { .. } => false,
    }
}

fn eval_ratified_doc<C: EvaluationContext>(
    explicit_path: Option<&Path>,
    min_version: Option<&str>,
    anchor: Option<&str>,
    sibling_json: bool,
    item: &ItemRatification,
    ctx: &C,
) -> bool {
    // Resolve the doc path: prefer explicit `path`, fall back to item's
    // `doc_ref.path` (one indirection).
    let path = match explicit_path {
        Some(p) => p.to_path_buf(),
        None => match &item.doc_ref {
            Some(dr) => dr.path.clone(),
            None => return false, // no doc to check
        },
    };

    let Some(content) = ctx.read_doc(&path) else {
        return false;
    };

    // Frontmatter version check (if min_version is requested).
    if let Some(min) = min_version {
        let Some(found_version) = parse_frontmatter_version(&content) else {
            return false;
        };
        if compare_versions(&found_version, min) == std::cmp::Ordering::Less {
            return false;
        }
    }

    // Anchor check (if requested). Simple substring check for now; a
    // future amendment can sharpen to YAML-frontmatter-aware or
    // markdown-heading-slug-aware checking.
    if let Some(a) = anchor {
        if !content.contains(a) {
            return false;
        }
    }

    // Sibling JSON check (if requested).
    if sibling_json {
        let sibling = sibling_json_path(&path);
        if ctx.read_doc(&sibling).is_none() {
            return false;
        }
    }

    true
}

fn eval_signers(
    required: &[String],
    roles: &std::collections::BTreeMap<String, String>,
    against: SignerCurrency,
    signature_allow: &[crate::tier::SignatureStrength],
    _signature_prefer: Option<crate::tier::SignatureStrength>,
    item: &ItemRatification,
    current_fingerprint: &str,
) -> bool {
    for needed in required {
        let candidates: Vec<&Signer> = item.signers.iter().filter(|s| s.name == *needed).collect();
        if candidates.is_empty() {
            return false;
        }
        // Currency and role must be satisfied by the SAME candidate entry (NFA-13).
        // Checking them independently allows a signer with the right role on a stale
        // entry AND a current entry without the role to pass both checks separately
        // — but no single entry satisfies both, so the predicate would be falsely
        // reporting that the required signer-as-role has signed against the current
        // fingerprint.
        let expected_role = roles.get(needed);
        let any_candidate_satisfies = candidates.iter().any(|s| {
            let currency_ok = match against {
                SignerCurrency::Current => s.signed_against_fingerprint == current_fingerprint,
                SignerCurrency::Any => true,
            };
            let role_ok = expected_role.is_none_or(|r| s.role.as_deref() == Some(r.as_str()));
            // If signature_allow is non-empty, the signer's strength must be in the list.
            let strength_ok = signature_allow.is_empty() || signature_allow.contains(&s.strength);
            currency_ok && role_ok && strength_ok
        });
        if !any_candidate_satisfies {
            return false;
        }
    }
    true
}

fn eval_signed_trailer<C: EvaluationContext>(
    key: &str,
    role: Option<&str>,
    count: u32,
    item_source_file: &Path,
    item_path: &str,
    ctx: &C,
) -> bool {
    let trailers = ctx.read_git_trailers(item_source_file, item_path);
    let mut hits: u32 = 0;
    for line in trailers {
        // Trailer format: `"Key: value"`. Match by key prefix.
        let Some((found_key, value)) = line.split_once(':') else {
            continue;
        };
        if found_key.trim() != key {
            continue;
        }
        if let Some(expected_role) = role {
            // Role tag convention: trailer value contains a `[role=...]`
            // or `(role=...)` marker. Simple substring check for v0.1;
            // future amendment can formalize.
            let role_marker_paren = format!("(role={expected_role})");
            let role_marker_bracket = format!("[role={expected_role}]");
            if !value.contains(&role_marker_paren) && !value.contains(&role_marker_bracket) {
                continue;
            }
        }
        hits = hits.saturating_add(1);
        if hits >= count {
            return true;
        }
    }
    hits >= count
}

fn eval_oracles_complete<C: EvaluationContext>(
    files: &[std::path::PathBuf],
    item: &ItemRatification,
    ctx: &C,
) -> bool {
    use crate::schema::{OracleRef, OracleState};
    files.iter().all(|p| {
        // NFA-26 fix: check Oracle artifact-class state before falling back to
        // file-content check. If a matching Oracle entry exists in the sidecar,
        // its state governs the evaluation — Draft blocks attestation,
        // Revoked{invalidates=true} blocks attestation. Only Complete passes.
        // For Deprecated, Retired, Revoked{false}: pass but the audit-hint layer
        // will surface the lifecycle event (hint channel is a v0.2 extension;
        // for v0.1 these non-blocking states fall through to the file check).
        let oracle_state_blocks = item
            .oracles
            .iter()
            .find(|o| matches!(&o.reference, OracleRef::LocalFile { path, .. } if path == p))
            .is_some_and(|o| {
                matches!(
                    o.state,
                    OracleState::Draft
                        | OracleState::Revoked {
                            invalidates_prior_attestations: true,
                            ..
                        }
                )
            });

        if oracle_state_blocks {
            return false;
        }

        ctx.read_oracle(p)
            .is_some_and(|content| parse_oracle_status(&content).as_deref() == Some("complete"))
    })
}

fn eval_fresh_within_days<C: EvaluationContext>(
    days: u32,
    item: &ItemRatification,
    current_fingerprint: &str,
    ctx: &C,
) -> bool {
    // Most recent CURRENT-fingerprint signer's date OR `fresh_through` — whichever is later.
    // NFA-21: stale-fingerprint signer dates must not satisfy a freshness check — a signer
    // who attested against fp-old months ago and never re-attested should not count as fresh.
    let latest_signer = item
        .signers
        .iter()
        .filter(|s| s.signed_against_fingerprint == current_fingerprint)
        .map(|s| s.date)
        .max();
    let candidate = match (latest_signer, item.fresh_through) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    };
    let Some(latest) = candidate else {
        return false;
    };
    let today = ctx.today();
    let diff_days = (today - latest).num_days();
    diff_days >= 0 && i128::from(diff_days) <= i128::from(days)
}

/// Given a predicate that passed, classify the sidecar signer state
/// into the audit hint per ADR-019 §M5 (immunity or tolerance table).
fn classify_passed_predicate<C: EvaluationContext>(
    item: &ItemRatification,
    current_fingerprint: &str,
    kind: RatificationKind,
    ctx: &C,
) -> EvaluatedPredicate {
    let passed_hint = match kind {
        RatificationKind::Tolerance => AuditHint::TolerancePredicatePassedSubstrateCurrent,
        RatificationKind::Immunity => AuditHint::DisciplinePredicatePassedSubstrateCurrent,
    };
    // No signers at all = predicate passed via non-signer leaves only
    // (e.g., `ratified_doc + oracles_complete + fresh_within_days` with
    // no `signers` leaf). The result is Execution-tier substrate-current
    // because all checked leaves passed, but there is NO identity
    // binding — no signer exists to bind identity to, so signature_strength
    // must be None, not Some(GitTrust).
    if item.signers.is_empty() {
        return EvaluatedPredicate {
            witness_tier: WitnessTier::Execution,
            audit_hint: passed_hint,
            evidence_kind: EvidenceKind::SubstrateState,
            signature_strength: None,
        };
    }

    // Weakest-link: strength is the minimum across CURRENT-fingerprint signers only
    // (NFA-19: historical entries from prior fingerprints must not pull down the
    // strength of a fresh re-attestation at the current fingerprint).
    let min_strength = item
        .signers
        .iter()
        .filter(|s| s.signed_against_fingerprint == current_fingerprint)
        .map(|s| s.strength)
        .min()
        .unwrap_or(SignatureStrength::GitTrust);

    // Stale detection: a NAME is stale iff ALL of its entries are against a
    // non-current fingerprint (NFA-18: sidecars are append-only; a re-attested
    // signer has both a historical stale entry AND a fresh current entry; counting
    // stale ROWS instead of stale NAMES falsely marks them stale).
    let all_names: std::collections::BTreeSet<&str> =
        item.signers.iter().map(|s| s.name.as_str()).collect();
    let stale_count = all_names
        .iter()
        .filter(|&&name| {
            !item
                .signers
                .iter()
                .any(|s| s.name == name && s.signed_against_fingerprint == current_fingerprint)
        })
        .count();
    if stale_count > 0 {
        return EvaluatedPredicate {
            witness_tier: WitnessTier::Reachability,
            audit_hint: AuditHint::DisciplineSubstrateStale,
            evidence_kind: EvidenceKind::SubstrateState,
            signature_strength: Some(min_strength),
        };
    }

    // All signer NAMES have a current entry. Check delta-chain state among
    // current-fingerprint entries only (NFA-20: historical delta entries must
    // not contaminate the delta-chain classification of a fresh re-attestation).
    let cap = ctx.delta_chain_cap();
    let max_chain_depth = item
        .signers
        .iter()
        .filter(|s| s.signed_against_fingerprint == current_fingerprint)
        .map(|s| s.basis.chain_depth())
        .max()
        .unwrap_or(0);
    let has_delta = item
        .signers
        .iter()
        .filter(|s| s.signed_against_fingerprint == current_fingerprint)
        .any(|s| s.basis.is_delta());

    if max_chain_depth >= cap.saturating_sub(1) && max_chain_depth > 0 {
        return EvaluatedPredicate {
            witness_tier: WitnessTier::Execution,
            audit_hint: AuditHint::DisciplineSubstrateDeltaChainNearCap,
            evidence_kind: EvidenceKind::SubstrateState,
            signature_strength: Some(min_strength),
        };
    }

    if has_delta {
        return EvaluatedPredicate {
            witness_tier: WitnessTier::Execution,
            audit_hint: AuditHint::DisciplinePredicatePassedViaDeltaChain,
            evidence_kind: EvidenceKind::SubstrateState,
            signature_strength: Some(min_strength),
        };
    }

    // All signers current and all Fresh — strongest v0.1 state.
    EvaluatedPredicate {
        witness_tier: WitnessTier::Execution,
        audit_hint: passed_hint,
        evidence_kind: EvidenceKind::SubstrateState,
        signature_strength: Some(min_strength),
    }
}

// --- Frontmatter / oracle parsing helpers --------------------------------

/// Parse `version: X.Y.Z` from the leading YAML frontmatter of a doc.
/// Returns None if no frontmatter, no version field, or non-string value.
fn parse_frontmatter_version(content: &str) -> Option<String> {
    parse_frontmatter_field(content, "version:")
}

/// Parse a named field from YAML frontmatter, tolerating both LF and CRLF.
fn parse_frontmatter_field(content: &str, field_prefix: &str) -> Option<String> {
    // Normalize CRLF → LF so Windows-authored files are not silently rejected.
    let normalized: std::borrow::Cow<str> = if content.contains('\r') {
        std::borrow::Cow::Owned(content.replace("\r\n", "\n").replace('\r', "\n"))
    } else {
        std::borrow::Cow::Borrowed(content)
    };
    let stripped = normalized.strip_prefix("---\n")?;
    // Accept both `\n---\n` (frontmatter followed by body) and `\n---` at EOF
    // (NFA-25: docs without a trailing newline after the closing delimiter).
    let end = stripped
        .find("\n---\n")
        .or_else(|| stripped.strip_suffix("\n---").map(str::len))?;
    let frontmatter = &stripped[..end];
    for line in frontmatter.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix(field_prefix) {
            return Some(rest.trim().trim_matches('"').trim_matches('\'').to_string());
        }
    }
    None
}

/// Parse `status: <value>` from an oracle file's YAML frontmatter.
fn parse_oracle_status(content: &str) -> Option<String> {
    parse_frontmatter_field(content, "status:")
}

/// Lexicographic comparison of semver-shaped version strings. Splits on
/// `.` and compares numerically per-component; falls back to string
/// comparison for non-numeric components. Sufficient for the v0.1
/// `frontmatter version: X.Y.Z` shape; a future amendment can swap in
/// a full semver crate when needed.
fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let parse = |s: &str| -> Vec<(u64, String)> {
        s.split('.')
            .map(|c| {
                let trimmed = c.trim();
                let n = trimmed.parse::<u64>().unwrap_or(0);
                (n, trimmed.to_string())
            })
            .collect()
    };
    let av = parse(a);
    let bv = parse(b);
    av.cmp(&bv)
}

/// Convention: `path/to/doc.md` → `path/to/doc.attest.json`.
fn sibling_json_path(doc: &Path) -> std::path::PathBuf {
    let mut p = doc.to_path_buf();
    let new_ext = doc.extension().and_then(|e| e.to_str()).map_or_else(
        || "attest.json".to_string(),
        |ext| format!("{ext}.attest.json"),
    );
    p.set_extension(new_ext);
    p
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::predicate::SignerCurrency;
    use crate::schema::{
        AntigenIdentifier, DocRef, ItemRatification, Oracle, OracleRef, OracleState, OracleVersion,
        Provenance, Ratification, RatificationKind, SchemaVersion, Signer, SignerBasis,
    };
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    /// In-memory evaluation context for unit tests. Caller seeds maps
    /// for docs / oracles / trailers and a fixed `today` date.
    struct TestContext {
        today: NaiveDate,
        docs: BTreeMap<PathBuf, String>,
        oracles: BTreeMap<PathBuf, String>,
        trailers: BTreeMap<(PathBuf, String), Vec<String>>,
        cap: u32,
    }

    impl TestContext {
        fn new(today: NaiveDate) -> Self {
            Self {
                today,
                docs: BTreeMap::new(),
                oracles: BTreeMap::new(),
                trailers: BTreeMap::new(),
                cap: 3,
            }
        }
        fn with_doc(mut self, path: &str, content: &str) -> Self {
            self.docs.insert(PathBuf::from(path), content.to_string());
            self
        }
        fn with_oracle(mut self, path: &str, content: &str) -> Self {
            self.oracles
                .insert(PathBuf::from(path), content.to_string());
            self
        }
        fn with_trailers(mut self, file: &str, item: &str, trailers: Vec<&str>) -> Self {
            self.trailers.insert(
                (PathBuf::from(file), item.to_string()),
                trailers.into_iter().map(String::from).collect(),
            );
            self
        }
    }

    impl EvaluationContext for TestContext {
        fn today(&self) -> NaiveDate {
            self.today
        }
        fn read_doc(&self, path: &Path) -> Option<String> {
            self.docs.get(path).cloned()
        }
        fn read_oracle(&self, path: &Path) -> Option<String> {
            self.oracles.get(path).cloned()
        }
        fn read_git_trailers(&self, item_source_file: &Path, item_path: &str) -> Vec<String> {
            self.trailers
                .get(&(item_source_file.to_path_buf(), item_path.to_string()))
                .cloned()
                .unwrap_or_default()
        }
        fn delta_chain_cap(&self) -> u32 {
            self.cap
        }
    }

    fn sample_date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 5, 19).unwrap()
    }

    fn item_with(signers: Vec<Signer>) -> ItemRatification {
        ItemRatification {
            item_path: "sinh".to_string(),
            current_fingerprint: "fp-current".to_string(),
            doc_ref: None,
            signers,
            oracles: vec![],
            fresh_through: None,
            extensions: BTreeMap::new(),
        }
    }

    fn alice_fresh(date: NaiveDate, fp: &str) -> Signer {
        Signer {
            name: "alice".to_string(),
            role: None,
            date,
            signed_against_fingerprint: fp.to_string(),
            basis: crate::schema::SignerBasis::Fresh { reasoning: None },
            strength: SignatureStrength::GitTrust,
            signature: None,
        }
    }

    #[test]
    fn signers_leaf_passes_when_required_present_and_current() {
        let item = item_with(vec![alice_fresh(sample_date(), "fp-current")]);
        let pred = Predicate::leaf(Leaf::Signers {
            required: vec!["alice".to_string()],
            roles: BTreeMap::new(),
            against: SignerCurrency::Current,
            signature_allow: vec![],
            signature_prefer: None,
        });
        let ctx = TestContext::new(sample_date());
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        assert_eq!(r.witness_tier, WitnessTier::Execution);
        assert_eq!(
            r.audit_hint,
            AuditHint::DisciplinePredicatePassedSubstrateCurrent
        );
    }

    #[test]
    fn signers_leaf_fails_when_required_missing() {
        let item = item_with(vec![alice_fresh(sample_date(), "fp-current")]);
        let pred = Predicate::leaf(Leaf::Signers {
            required: vec!["bob".to_string()],
            roles: BTreeMap::new(),
            against: SignerCurrency::Current,
            signature_allow: vec![],
            signature_prefer: None,
        });
        let ctx = TestContext::new(sample_date());
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        assert_eq!(r.witness_tier, WitnessTier::None);
        assert_eq!(r.audit_hint, AuditHint::DisciplinePredicateFailed);
    }

    #[test]
    fn signers_leaf_reports_stale_when_signature_against_old_fingerprint() {
        let item = item_with(vec![alice_fresh(sample_date(), "fp-OLD")]);
        let pred = Predicate::leaf(Leaf::Signers {
            required: vec!["alice".to_string()],
            roles: BTreeMap::new(),
            against: SignerCurrency::Any,
            signature_allow: vec![],
            signature_prefer: None,
        });
        let ctx = TestContext::new(sample_date());
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        // Predicate passes (against=Any), but signers are stale.
        assert_eq!(r.witness_tier, WitnessTier::Reachability);
        assert_eq!(r.audit_hint, AuditHint::DisciplineSubstrateStale);
    }

    #[test]
    fn fresh_within_days_passes_when_signer_recent() {
        let item = item_with(vec![alice_fresh(sample_date(), "fp-current")]);
        let pred = Predicate::leaf(Leaf::FreshWithinDays { days: 30 });
        let ctx = TestContext::new(sample_date());
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        assert_eq!(r.witness_tier, WitnessTier::Execution);
    }

    #[test]
    fn fresh_within_days_fails_when_signer_too_old() {
        let old_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let item = item_with(vec![alice_fresh(old_date, "fp-current")]);
        let pred = Predicate::leaf(Leaf::FreshWithinDays { days: 30 });
        let ctx = TestContext::new(sample_date());
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        assert_eq!(r.witness_tier, WitnessTier::None);
        assert_eq!(r.audit_hint, AuditHint::DisciplinePredicateFailed);
    }

    #[test]
    fn delta_chain_near_cap_emits_hint() {
        let signer = Signer {
            name: "alice".to_string(),
            role: None,
            date: sample_date(),
            signed_against_fingerprint: "fp-current".to_string(),
            basis: SignerBasis::DeltaFrom {
                prior_fingerprint: "fp-prior".to_string(),
                cumulative_root_fingerprint: "fp-root".to_string(),
                chain_depth: 2, // cap=3 → 2 is cap-1; near-cap fires
                rationale: "reviewed diff against prior; invariant-preserving".to_string(),
            },
            strength: SignatureStrength::GitTrust,
            signature: None,
        };
        let item = item_with(vec![signer]);
        let pred = Predicate::leaf(Leaf::Signers {
            required: vec!["alice".to_string()],
            roles: BTreeMap::new(),
            against: SignerCurrency::Current,
            signature_allow: vec![],
            signature_prefer: None,
        });
        let ctx = TestContext::new(sample_date());
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        assert_eq!(r.witness_tier, WitnessTier::Execution);
        assert_eq!(
            r.audit_hint,
            AuditHint::DisciplineSubstrateDeltaChainNearCap
        );
    }

    #[test]
    fn delta_chain_below_near_cap_emits_via_delta_hint() {
        let signer = Signer {
            name: "alice".to_string(),
            role: None,
            date: sample_date(),
            signed_against_fingerprint: "fp-current".to_string(),
            basis: SignerBasis::DeltaFrom {
                prior_fingerprint: "fp-prior".to_string(),
                cumulative_root_fingerprint: "fp-root".to_string(),
                chain_depth: 1,
                rationale: "reviewed diff against prior; invariant-preserving".to_string(),
            },
            strength: SignatureStrength::GitTrust,
            signature: None,
        };
        let item = item_with(vec![signer]);
        let pred = Predicate::leaf(Leaf::Signers {
            required: vec!["alice".to_string()],
            roles: BTreeMap::new(),
            against: SignerCurrency::Current,
            signature_allow: vec![],
            signature_prefer: None,
        });
        let ctx = TestContext::new(sample_date());
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        assert_eq!(r.witness_tier, WitnessTier::Execution);
        assert_eq!(
            r.audit_hint,
            AuditHint::DisciplinePredicatePassedViaDeltaChain
        );
    }

    #[test]
    fn signed_trailer_passes_when_trailer_present() {
        let item = item_with(vec![]);
        let pred = Predicate::leaf(Leaf::SignedTrailer {
            key: "Discipline-Verified-By".to_string(),
            role: None,
            count: 1,
        });
        let ctx = TestContext::new(sample_date()).with_trailers(
            "src/test.rs",
            "sinh",
            vec!["Discipline-Verified-By: alice <alice@example.com>"],
        );
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        // Signer-list empty but predicate passed via trailer; classification
        // reports Execution + substrate-current.
        assert_eq!(r.witness_tier, WitnessTier::Execution);
    }

    #[test]
    fn signed_trailer_requires_count_matches() {
        let item = item_with(vec![]);
        let pred = Predicate::leaf(Leaf::SignedTrailer {
            key: "Discipline-Verified-By".to_string(),
            role: None,
            count: 2,
        });
        let ctx = TestContext::new(sample_date()).with_trailers(
            "src/test.rs",
            "sinh",
            vec!["Discipline-Verified-By: alice <alice@example.com>"],
        );
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        assert_eq!(r.witness_tier, WitnessTier::None);
    }

    #[test]
    fn signed_trailer_role_filter_works() {
        let item = item_with(vec![]);
        let pred = Predicate::leaf(Leaf::SignedTrailer {
            key: "Discipline-Verified-By".to_string(),
            role: Some("math-researcher".to_string()),
            count: 1,
        });
        // Trailer without role tag — should fail
        let ctx_no_role = TestContext::new(sample_date()).with_trailers(
            "src/test.rs",
            "sinh",
            vec!["Discipline-Verified-By: alice <alice@example.com>"],
        );
        let r1 = evaluate_predicate(
            &pred,
            &item,
            "fp-current",
            Path::new("src/test.rs"),
            &ctx_no_role,
        )
        .unwrap();
        assert_eq!(r1.witness_tier, WitnessTier::None);
        // Trailer with role tag — should pass
        let ctx_with_role = TestContext::new(sample_date()).with_trailers(
            "src/test.rs",
            "sinh",
            vec!["Discipline-Verified-By: alice [role=math-researcher]"],
        );
        let r2 = evaluate_predicate(
            &pred,
            &item,
            "fp-current",
            Path::new("src/test.rs"),
            &ctx_with_role,
        )
        .unwrap();
        assert_eq!(r2.witness_tier, WitnessTier::Execution);
    }

    #[test]
    fn oracles_complete_passes_when_all_complete() {
        let item = item_with(vec![]);
        let pred = Predicate::leaf(Leaf::OraclesComplete {
            files: vec![PathBuf::from("docs/oracles/o.md")],
        });
        let ctx = TestContext::new(sample_date())
            .with_oracle("docs/oracles/o.md", "---\nstatus: complete\n---\nbody");
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        assert_eq!(r.witness_tier, WitnessTier::Execution);
    }

    #[test]
    fn oracles_complete_fails_when_any_incomplete() {
        let item = item_with(vec![]);
        let pred = Predicate::leaf(Leaf::OraclesComplete {
            files: vec![
                PathBuf::from("docs/oracles/a.md"),
                PathBuf::from("docs/oracles/b.md"),
            ],
        });
        let ctx = TestContext::new(sample_date())
            .with_oracle("docs/oracles/a.md", "---\nstatus: complete\n---\nbody")
            .with_oracle("docs/oracles/b.md", "---\nstatus: in_progress\n---\nbody");
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        assert_eq!(r.witness_tier, WitnessTier::None);
    }

    #[test]
    fn ratified_doc_passes_with_explicit_path() {
        let item = item_with(vec![]);
        let pred = Predicate::leaf(Leaf::RatifiedDoc {
            path: Some(PathBuf::from("docs/sinh.md")),
            min_version: Some("1.0".to_string()),
            anchor: None,
            sibling_json: false,
        });
        let ctx = TestContext::new(sample_date())
            .with_doc("docs/sinh.md", "---\nversion: 1.0\n---\nbody");
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        assert_eq!(r.witness_tier, WitnessTier::Execution);
    }

    #[test]
    fn ratified_doc_fails_when_version_too_low() {
        let item = item_with(vec![]);
        let pred = Predicate::leaf(Leaf::RatifiedDoc {
            path: Some(PathBuf::from("docs/sinh.md")),
            min_version: Some("2.0".to_string()),
            anchor: None,
            sibling_json: false,
        });
        let ctx = TestContext::new(sample_date())
            .with_doc("docs/sinh.md", "---\nversion: 1.0\n---\nbody");
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        assert_eq!(r.witness_tier, WitnessTier::None);
    }

    #[test]
    fn ratified_doc_uses_item_doc_ref_when_path_omitted() {
        let mut item = item_with(vec![]);
        item.doc_ref = Some(DocRef {
            path: PathBuf::from("docs/from-item.md"),
            min_version: None,
            anchor: None,
        });
        let pred = Predicate::leaf(Leaf::RatifiedDoc {
            path: None,
            min_version: None,
            anchor: None,
            sibling_json: false,
        });
        let ctx = TestContext::new(sample_date())
            .with_doc("docs/from-item.md", "body without frontmatter");
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        assert_eq!(r.witness_tier, WitnessTier::Execution);
    }

    #[test]
    fn all_of_short_circuits_on_first_fail() {
        let item = item_with(vec![alice_fresh(sample_date(), "fp-current")]);
        let pred = Predicate::all_of(vec![
            Predicate::leaf(Leaf::Signers {
                required: vec!["alice".to_string()],
                roles: BTreeMap::new(),
                against: SignerCurrency::Current,
                signature_allow: vec![],
                signature_prefer: None,
            }),
            Predicate::leaf(Leaf::OraclesComplete {
                files: vec![PathBuf::from("nonexistent.md")],
            }),
        ])
        .unwrap();
        let ctx = TestContext::new(sample_date());
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        // OraclesComplete leaf fails → all_of fails → DisciplinePredicateFailed
        assert_eq!(r.witness_tier, WitnessTier::None);
        assert_eq!(r.audit_hint, AuditHint::DisciplinePredicateFailed);
    }

    #[test]
    fn any_of_passes_when_one_branch_passes() {
        let item = item_with(vec![alice_fresh(sample_date(), "fp-current")]);
        let pred = Predicate::any_of(vec![
            Predicate::leaf(Leaf::OraclesComplete {
                files: vec![PathBuf::from("nonexistent.md")],
            }),
            Predicate::leaf(Leaf::Signers {
                required: vec!["alice".to_string()],
                roles: BTreeMap::new(),
                against: SignerCurrency::Current,
                signature_allow: vec![],
                signature_prefer: None,
            }),
        ])
        .unwrap();
        let ctx = TestContext::new(sample_date());
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        assert_eq!(r.witness_tier, WitnessTier::Execution);
    }

    #[test]
    fn not_inverts_child_result() {
        let item = item_with(vec![]);
        let pred = Predicate::not(Predicate::leaf(Leaf::Signers {
            required: vec!["alice".to_string()],
            roles: BTreeMap::new(),
            against: SignerCurrency::Current,
            signature_allow: vec![],
            signature_prefer: None,
        }));
        let ctx = TestContext::new(sample_date());
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        // Signers required=alice fails (no signers); not() inverts to pass.
        assert_eq!(r.witness_tier, WitnessTier::Execution);
    }

    #[test]
    fn no_signers_no_signers_required_passes_substrate_current() {
        let item = item_with(vec![]);
        let pred = Predicate::leaf(Leaf::FreshWithinDays { days: 365 });
        // FreshWithinDays with no signers/no fresh_through fails because
        // there's no anchor date. Wrap in `not` to make the predicate pass.
        let pred = Predicate::not(pred);
        let ctx = TestContext::new(sample_date());
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        assert_eq!(r.witness_tier, WitnessTier::Execution);
        assert_eq!(
            r.audit_hint,
            AuditHint::DisciplinePredicatePassedSubstrateCurrent
        );
    }

    #[test]
    fn unused_ratification_is_silent() {
        // Constructing a full Ratification + serializing is covered in
        // schema tests; this test just confirms the type compiles in
        // evaluate.rs's test module.
        let _r = Ratification {
            schema_version: SchemaVersion::V1,
            kind: RatificationKind::Immunity,
            antigen: AntigenIdentifier {
                name: "X".to_string(),
                defined_in: None,
            },
            source_file: PathBuf::from("src/test.rs"),
            items: vec![ItemRatification {
                item_path: "y".to_string(),
                current_fingerprint: "fp".to_string(),
                doc_ref: None,
                signers: vec![],
                oracles: vec![Oracle {
                    id: "o.md".to_string(),
                    reference: OracleRef::LocalFile {
                        path: PathBuf::from("o.md"),
                        status_field: None,
                        expected_status: None,
                    },
                    state: OracleState::Complete,
                    stewards: vec![],
                    created: Provenance {
                        recorded_by: "test".to_string(),
                        at: sample_date(),
                    },
                    version: OracleVersion {
                        pinned: "v0".to_string(),
                        pinned_at: sample_date(),
                    },
                    transitions: vec![],
                    extensions: BTreeMap::new(),
                }],
                fresh_through: None,
                extensions: BTreeMap::new(),
            }],
        };
    }

    #[test]
    fn oracle_crlf_line_endings_not_silently_rejected_nfa6() {
        // BUG REGRESSION TEST (adversarial NFA-6): oracle files authored on
        // Windows use CRLF (\r\n) line endings. `parse_oracle_status` uses
        // `strip_prefix("---\n")` which fails on `---\r\n`, causing the oracle
        // to be treated as missing/incomplete even when status IS "complete".
        //
        // This is a SILENT FAILURE — the predicate reports "failed" with no
        // indication that CRLF normalization is the cause. Indistinguishable
        // from a genuinely incomplete oracle at the audit layer.
        //
        // This test FAILS against the buggy code.
        let item = item_with(vec![]);
        let pred = Predicate::leaf(Leaf::OraclesComplete {
            files: vec![PathBuf::from("docs/oracles/o.md")],
        });
        // Oracle content with CRLF line endings (as written on Windows).
        let crlf_oracle = "---\r\nstatus: complete\r\n---\r\nbody";
        let ctx = TestContext::new(sample_date()).with_oracle("docs/oracles/o.md", crlf_oracle);
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        // An oracle with status: complete (even CRLF-encoded) must pass the leaf.
        assert_eq!(
            r.witness_tier,
            WitnessTier::Execution,
            "CRLF-encoded oracle with status: complete must not be silently rejected"
        );
    }

    #[test]
    fn tolerance_stale_signer_emits_immunity_stale_hint_v02_gap() {
        // DOCUMENTED LIMITATION (v0.2): `classify_passed_predicate` emits the
        // IMMUNITY intermediate-state hints (`DisciplineSubstrateStale`,
        // `DisciplineSubstrateDeltaChainNearCap`, `DisciplinePredicatePassedViaDeltaChain`)
        // even for TOLERANCE sidecars. There are no `Tolerance*` equivalents for
        // these intermediate states in the `AuditHint` enum.
        //
        // The kind-awareness from NFA-10 (fix: evaluate_predicate_with_kind) only
        // reaches the passed/failed terminal hints, not the intermediate classification
        // hints. A tolerance sidecar with a stale signer gets the immunity stale hint.
        //
        // Fix direction (v0.2): Add `ToleranceSubstrateStale`, `ToleranceDeltaChainNearCap`,
        // `TolerancePredicatePassedViaDeltaChain` variants to AuditHint and thread
        // `kind` through `classify_passed_predicate`'s intermediate-state branches.
        //
        // This test DOCUMENTS the current behavior so any future fix is immediately
        // visible (the assertion will start failing when the hint is made tolerance-aware,
        // serving as a prompt to update all callers).
        let alice_stale = Signer {
            name: "alice".to_string(),
            role: None,
            date: sample_date(),
            signed_against_fingerprint: "fp-OLD".to_string(),
            basis: SignerBasis::Fresh { reasoning: None },
            strength: SignatureStrength::GitTrust,
            signature: None,
        };
        let item = item_with(vec![alice_stale]);
        let pred = Predicate::leaf(Leaf::Signers {
            required: vec!["alice".to_string()],
            roles: BTreeMap::new(),
            against: SignerCurrency::Any,
            signature_allow: vec![],
            signature_prefer: None,
        });
        let ctx = TestContext::new(sample_date());
        // Evaluate as TOLERANCE kind.
        let r = evaluate_predicate_with_kind(
            &pred,
            &item,
            "fp-current",
            Path::new("src/test.rs"),
            RatificationKind::Tolerance,
            &ctx,
        )
        .unwrap();
        // Predicate passes (against=Any), all signers stale → classify fires stale branch.
        // CURRENT BEHAVIOR: emits DisciplineSubstrateStale (immunity hint) not a tolerance
        // equivalent. This documents the v0.2 gap.
        assert_eq!(r.witness_tier, WitnessTier::Reachability);
        assert_eq!(
            r.audit_hint,
            AuditHint::DisciplineSubstrateStale,
            "v0.2 gap: tolerance sidecar with stale signer emits immunity stale hint; \
             no ToleranceSubstrateStale variant exists yet"
        );
    }

    #[test]
    fn signers_role_currency_must_be_joint_not_independent_nfa13() {
        // BUG REGRESSION TEST (adversarial NFA-13): `eval_signers` checks
        // currency (against=Current) and role separately across the candidates
        // list. Two independent `any(...)` checks: "does any alice have current
        // fingerprint?" AND "does any alice have role=reviewer?". If alice has
        // TWO entries — one current without the role, one stale with the role —
        // both checks pass independently, so the predicate reports PASS. But no
        // single alice entry is BOTH current AND has the required role. The predicate
        // should FAIL: the signer-as-reviewer has not signed against the current
        // fingerprint.
        //
        // This is a SILENT FAILURE: the predicate reports Execution-tier pass
        // and implies "alice-as-reviewer has signed against current fingerprint"
        // when in reality alice-as-reviewer only signed against a stale fingerprint.
        //
        // FIX DIRECTION: the role and currency filters must be evaluated jointly
        // on each candidate, not independently across the set.
        //
        // This test FAILS against the current code.
        let alice_current_no_role = Signer {
            name: "alice".to_string(),
            role: None, // no role on the current signature
            date: sample_date(),
            signed_against_fingerprint: "fp-current".to_string(),
            basis: crate::schema::SignerBasis::Fresh { reasoning: None },
            strength: SignatureStrength::GitTrust,
            signature: None,
        };
        let alice_stale_with_role = Signer {
            name: "alice".to_string(),
            role: Some("reviewer".to_string()), // role on the STALE signature
            date: sample_date(),
            signed_against_fingerprint: "fp-STALE".to_string(),
            basis: crate::schema::SignerBasis::Fresh { reasoning: None },
            strength: SignatureStrength::GitTrust,
            signature: None,
        };
        let item = item_with(vec![alice_current_no_role, alice_stale_with_role]);

        let mut roles = BTreeMap::new();
        roles.insert("alice".to_string(), "reviewer".to_string());
        let pred = Predicate::leaf(Leaf::Signers {
            required: vec!["alice".to_string()],
            roles,
            against: SignerCurrency::Current,
            signature_allow: vec![],
            signature_prefer: None,
        });

        let ctx = TestContext::new(sample_date());
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        // MUST FAIL: no single alice entry is both current AND has role=reviewer.
        // alice-as-reviewer only signed against fp-STALE.
        assert_eq!(
            r.witness_tier,
            WitnessTier::None,
            "NFA-13: signers leaf must fail when currency and role are satisfied by \
             DIFFERENT entries; alice-as-reviewer signed stale, not current"
        );
    }

    #[test]
    fn empty_min_version_string_vacuously_passes_version_check_nfa15() {
        // BUG REGRESSION TEST (adversarial NFA-15): same class as NFA-14.
        // `min_version: Some("")` enters the version check branch but `compare_versions`
        // parses "" as `[(0, "")]` — the lowest possible version representation.
        // Any document with a non-empty version field compares Greater than "" and passes.
        // The check "document version >= ''" is vacuously true for any versioned doc.
        //
        // This is part of the convergent "empty/zero parameter vacuous bypass" class
        // (NFA-7 empty Signers.required; NFA-8 empty OraclesComplete.files;
        // NFA-9 SignedTrailer count=0; NFA-14 empty anchor).
        //
        // FIX DIRECTION: `Predicate::validate()` should reject `Leaf::RatifiedDoc`
        // entries where `min_version == Some("")`. Add a new `PredicateParseError::EmptyMinVersion`
        // variant and the corresponding check in the walk phase.
        //
        // This test FAILS until the fix is applied.
        let pred = Predicate::leaf(Leaf::RatifiedDoc {
            path: Some(PathBuf::from("docs/sinh.md")),
            min_version: Some(String::new()), // empty — vacuously passes version check
            anchor: None,
            sibling_json: false,
        });
        let err = pred.validate();
        assert!(
            err.is_err(),
            "NFA-15: validate() must reject RatifiedDoc with empty min_version string; \
             compare_versions(any_version, '') is always Greater or Equal and provides \
             no version floor guarantee"
        );
    }

    #[test]
    fn empty_anchor_string_vacuously_bypasses_anchor_check_nfa14() {
        // BUG REGRESSION TEST (adversarial NFA-14): `eval_ratified_doc` performs
        // the anchor check via `content.contains(anchor)`. In Rust, `str::contains("")`
        // is always `true` for any string. An `anchor: Some("")` predicate therefore
        // vacuously passes for ANY doc content — even a doc with no meaningful
        // section anchors. This is a silent bypass of the anchor-presence requirement.
        //
        // The predicate claims "anchor '' is present in the doc" — which is
        // trivially true and carries no information. The correct behaviour is to
        // reject empty anchor strings at validation time (predicate.validate()).
        //
        // FIX DIRECTION: `Predicate::validate()` should reject `Leaf::RatifiedDoc`
        // entries where `anchor == Some("")`. Add a `PredicateParseError::EmptyAnchor`
        // variant and the corresponding check in the walk phase.
        //
        // This test FAILS against the current code because validate() does not
        // catch the empty anchor, and the evaluator then accepts it vacuously.
        let pred = Predicate::leaf(Leaf::RatifiedDoc {
            path: Some(PathBuf::from("docs/sinh.md")),
            min_version: None,
            anchor: Some(String::new()), // empty anchor — should be rejected
            sibling_json: false,
        });
        // This should fail validate() — empty anchor is a no-op that bypasses the check.
        let err = pred.validate();
        assert!(
            err.is_err(),
            "NFA-14: validate() must reject RatifiedDoc with empty anchor string; \
             content.contains('') is always true and provides no anchor guarantee"
        );
    }

    #[test]
    fn text_stamp_signer_silently_inflated_to_git_trust_nfa16() {
        // BUG REGRESSION TEST (adversarial NFA-16): `classify_passed_predicate`
        // hardcodes `signature_strength: Some(SignatureStrength::GitTrust)` for
        // all signer-present paths. The `Signer` schema carries no
        // `strength: SignatureStrength` field, so the evaluator cannot distinguish
        // a TextStamp signer (name + timestamp only; no identity verification) from
        // a GitTrust signer (git config identity; fingerprint pin).
        //
        // Attack: an LLM agent or non-git-configured human writes a sidecar with
        // a `Signer` entry (no `signature` field — TextStamp tier). The audit reports
        // `signature_strength: Some(GitTrust)` — inflated by one tier. A CI gate
        // requiring `min_signature_strength >= GitTrust` silently passes despite
        // the actual identity binding being TextStamp (minimal, unverifiable).
        //
        // This is a SILENT tier-honesty violation. The output looks correct but the
        // strength claim is wrong.
        //
        // ROOT CAUSE: Two-layer gap:
        // (1) `schema::Signer` has no `strength: SignatureStrength` field — there is
        //     no way to record which tier a signer used when writing the sidecar.
        // (2) `classify_passed_predicate` hardcodes `Some(SignatureStrength::GitTrust)`
        //     instead of reading per-signer strength and taking the minimum.
        //
        // FIX DIRECTION:
        // (1) Add `strength: SignatureStrength` field to `schema::Signer` (default
        //     `SignatureStrength::GitTrust` for backward compat with existing sidecars
        //     that were written before TextStamp existed).
        // (2) `classify_passed_predicate` reads each signer's `strength` field and
        //     reports `min(signer.strength for all signers)` in the result. Minimum
        //     is correct: the overall attestation strength is limited by the weakest
        //     individual signer's identity binding (weakest-link principle).
        //
        // This test FAILS against the current code because the evaluator returns
        // GitTrust regardless of what strength the signer carries.
        //
        // NOTE: Once (1) is fixed, the sidecar schema gains a new optional field with
        // `#[serde(default)]` so existing sidecars without the field default to
        // `SignatureStrength::GitTrust` (backward-compatible).
        let alice_text_stamp = Signer {
            name: "alice".to_string(),
            role: None,
            date: sample_date(),
            signed_against_fingerprint: "fp-current".to_string(),
            basis: crate::schema::SignerBasis::Fresh { reasoning: None },
            strength: SignatureStrength::TextStamp,
            signature: None,
        };
        let item = item_with(vec![alice_text_stamp]);
        let pred = Predicate::leaf(Leaf::Signers {
            required: vec!["alice".to_string()],
            roles: BTreeMap::new(),
            against: SignerCurrency::Current,
            signature_allow: vec![],
            signature_prefer: None,
        });
        let ctx = TestContext::new(sample_date());
        let r = evaluate_predicate_with_kind(
            &pred,
            &item,
            "fp-current",
            Path::new("src/test.rs"),
            RatificationKind::Immunity,
            &ctx,
        )
        .unwrap();
        // Predicate passes (alice is current + required).
        assert_eq!(r.witness_tier, WitnessTier::Execution);
        // BUG: currently reports GitTrust — should be TextStamp once schema
        // carries per-signer strength and evaluator reads it.
        // This assertion FAILS (returns Some(GitTrust)) until the fix is applied:
        assert_eq!(
            r.signature_strength,
            Some(SignatureStrength::TextStamp),
            "NFA-16: TextStamp signer must not be silently inflated to GitTrust in audit output; \
             CI gates requiring >= GitTrust would silently pass on TextStamp-only attestations"
        );
    }

    #[test]
    fn historical_signer_entry_does_not_trigger_false_stale_nfa18() {
        // BUG REGRESSION TEST (adversarial NFA-18): sidecars are append-only. When
        // a signer re-attests after the item fingerprint changes, the sidecar gains
        // a NEW entry (current fp) while keeping the OLD entry (prior fp). The old
        // stale-detection code counted stale ROWS rather than stale NAMES: any row
        // with a non-current fingerprint incremented stale_count, so a re-attested
        // signer who has BOTH a stale row AND a fresh row was (falsely) classified
        // as stale — downgrading the result from Execution to Reachability.
        //
        // FIX: a NAME is stale iff ALL of its entries are against non-current fp.
        // If at least one entry for that name is current, the name is NOT stale.
        //
        // This test FAILS against the buggy row-counting code.
        let alice_old = Signer {
            name: "alice".to_string(),
            role: None,
            date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            signed_against_fingerprint: "fp-old".to_string(),
            basis: crate::schema::SignerBasis::Fresh { reasoning: None },
            strength: SignatureStrength::GitTrust,
            signature: None,
        };
        let alice_new = Signer {
            name: "alice".to_string(),
            role: None,
            date: sample_date(),
            signed_against_fingerprint: "fp-current".to_string(),
            basis: crate::schema::SignerBasis::Fresh { reasoning: None },
            strength: SignatureStrength::GitTrust,
            signature: None,
        };
        // Sidecar has BOTH entries (append-only ratchet).
        let item = item_with(vec![alice_old, alice_new]);
        let pred = Predicate::leaf(Leaf::Signers {
            required: vec!["alice".to_string()],
            roles: BTreeMap::new(),
            against: SignerCurrency::Current,
            signature_allow: vec![],
            signature_prefer: None,
        });
        let ctx = TestContext::new(sample_date());
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        // Alice has a current entry — must NOT be classified as stale.
        assert_eq!(
            r.witness_tier,
            WitnessTier::Execution,
            "NFA-18: re-attested signer with historical stale row must not be classified stale; \
             append-only ratchet produces both rows; stale detection must be per NAME not per ROW"
        );
        assert_eq!(
            r.audit_hint,
            AuditHint::DisciplinePredicatePassedSubstrateCurrent
        );
    }

    #[test]
    fn historical_text_stamp_entry_does_not_pull_down_current_git_trust_strength_nfa19() {
        // BUG REGRESSION TEST (adversarial NFA-19): `min_strength` was computed
        // across ALL signer rows including historical stale entries. In an append-only
        // sidecar, a signer who attested at TextStamp tier against fp-old then
        // re-attested at GitTrust tier against fp-current leaves BOTH rows. The old
        // min_strength computation took the minimum across all rows — returning
        // TextStamp — even though the current attestation is GitTrust.
        //
        // FIX: min_strength is computed over CURRENT-fingerprint entries only.
        //
        // This test FAILS against the buggy all-rows min_strength code.
        let alice_old_text_stamp = Signer {
            name: "alice".to_string(),
            role: None,
            date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            signed_against_fingerprint: "fp-old".to_string(),
            basis: crate::schema::SignerBasis::Fresh { reasoning: None },
            strength: SignatureStrength::TextStamp, // historical: weaker tier
            signature: None,
        };
        let alice_new_git_trust = Signer {
            name: "alice".to_string(),
            role: None,
            date: sample_date(),
            signed_against_fingerprint: "fp-current".to_string(),
            basis: crate::schema::SignerBasis::Fresh { reasoning: None },
            strength: SignatureStrength::GitTrust, // current: stronger tier
            signature: None,
        };
        let item = item_with(vec![alice_old_text_stamp, alice_new_git_trust]);
        let pred = Predicate::leaf(Leaf::Signers {
            required: vec!["alice".to_string()],
            roles: BTreeMap::new(),
            against: SignerCurrency::Current,
            signature_allow: vec![],
            signature_prefer: None,
        });
        let ctx = TestContext::new(sample_date());
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        assert_eq!(r.witness_tier, WitnessTier::Execution);
        // Historical TextStamp entry must NOT pull down the current GitTrust strength.
        assert_eq!(
            r.signature_strength,
            Some(SignatureStrength::GitTrust),
            "NFA-19: historical TextStamp entry must not pull down current GitTrust \
             min_strength; min_strength must be computed over current-fp entries only"
        );
    }

    #[test]
    fn historical_delta_entry_does_not_contaminate_current_fresh_state_nfa20() {
        // BUG REGRESSION TEST (adversarial NFA-20): `max_chain_depth` and `has_delta`
        // were computed across ALL signer rows. A signer who originally attested via
        // a delta-chain (chain_depth >= near-cap) against fp-old then re-attested Fresh
        // against fp-current should be classified as Fresh — not DeltaChainNearCap.
        // But the old code found the historical delta row and emitted the delta hint.
        //
        // FIX: both delta-chain fields are computed over CURRENT-fingerprint entries only.
        //
        // This test FAILS against the buggy all-rows delta computation.
        let alice_old_delta = Signer {
            name: "alice".to_string(),
            role: None,
            date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            signed_against_fingerprint: "fp-old".to_string(),
            basis: crate::schema::SignerBasis::DeltaFrom {
                prior_fingerprint: "fp-root".to_string(),
                cumulative_root_fingerprint: "fp-root".to_string(),
                chain_depth: 2, // near-cap (cap=3 → depth 2 fires DeltaChainNearCap)
                rationale: "historical delta".to_string(),
            },
            strength: SignatureStrength::GitTrust,
            signature: None,
        };
        let alice_new_fresh = Signer {
            name: "alice".to_string(),
            role: None,
            date: sample_date(),
            signed_against_fingerprint: "fp-current".to_string(),
            basis: crate::schema::SignerBasis::Fresh { reasoning: None }, // re-attested Fresh
            strength: SignatureStrength::GitTrust,
            signature: None,
        };
        let item = item_with(vec![alice_old_delta, alice_new_fresh]);
        let pred = Predicate::leaf(Leaf::Signers {
            required: vec!["alice".to_string()],
            roles: BTreeMap::new(),
            against: SignerCurrency::Current,
            signature_allow: vec![],
            signature_prefer: None,
        });
        let ctx = TestContext::new(sample_date());
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        // Current attestation is Fresh — must NOT emit DeltaChainNearCap or ViaDeltaChain.
        assert_eq!(
            r.audit_hint,
            AuditHint::DisciplinePredicatePassedSubstrateCurrent,
            "NFA-20: historical delta entry must not contaminate fresh re-attestation; \
             delta classification must be computed over current-fp entries only"
        );
    }

    #[test]
    fn stale_signer_date_does_not_satisfy_fresh_within_days_standalone_nfa21() {
        // BUG REGRESSION TEST (adversarial NFA-21): `eval_fresh_within_days` used
        // `item.signers.iter().map(|s| s.date).max()` — taking the maximum date
        // across ALL signer entries including stale-fingerprint ones. A signer who
        // signed TODAY against fp-old (stale fingerprint) would satisfy a
        // `fresh_within_days(60)` check even though they have never attested against
        // the current fingerprint. This is a silent freshness bypass.
        //
        // FIX: only consider signer dates for entries whose
        // `signed_against_fingerprint == current_fingerprint`.
        //
        // This test FAILS against the buggy all-rows date computation.
        let stale_bob = Signer {
            name: "bob".to_string(),
            role: None,
            date: sample_date(), // today — but against fp-old (stale)
            signed_against_fingerprint: "fp-old".to_string(),
            basis: crate::schema::SignerBasis::Fresh { reasoning: None },
            strength: SignatureStrength::GitTrust,
            signature: None,
        };
        // No current-fp entry exists; fresh_through is also None.
        let item = item_with(vec![stale_bob]);
        let pred = Predicate::leaf(Leaf::FreshWithinDays { days: 60 });
        let ctx = TestContext::new(sample_date());
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        // Stale signer's date must NOT satisfy fresh_within_days.
        assert_eq!(
            r.witness_tier,
            WitnessTier::None,
            "NFA-21: signer date against stale fingerprint must not satisfy \
             fresh_within_days; only current-fp signer dates count"
        );
        assert_eq!(r.audit_hint, AuditHint::DisciplinePredicateFailed);
    }

    #[test]
    fn fresh_through_with_no_signers_at_all_bypasses_freshness_nfa23() {
        // DOCUMENTED GAP (adversarial NFA-23): `eval_fresh_within_days` accepts
        // `item.fresh_through` as an anchor date even when the sidecar has NO signers.
        // A sidecar with `signers = []` but `fresh_through` set to a recent date will
        // satisfy the freshness leaf — nobody has attested, but the item appears "fresh."
        //
        // Note: when signers ARE present but stale, `classify_passed_predicate` catches
        // them via the per-name stale detection and returns Reachability, not Execution.
        // The gap is strongest when there are NO signers at all and fresh_through is set
        // — the signer list is empty, classify returns Execution with no signers.
        //
        // Attack: create a sidecar with no signers and fresh_through = today. The item
        // is claimed "fresh" by the freshness leaf even though nobody has reviewed it.
        //
        // FIX DIRECTION (v0.2): `eval_fresh_within_days` should require at least one
        // signer entry to be meaningful — fresh_through alone (without a signer to
        // anchor the freshness claim) should not satisfy the leaf.
        //
        // This test DOCUMENTS the current behavior.
        let mut item = item_with(vec![]); // NO signers
                                          // fresh_through = today; the freshness check reads this as "signed on today".
        item.fresh_through = Some(sample_date());
        let pred = Predicate::leaf(Leaf::FreshWithinDays { days: 60 });
        let ctx = TestContext::new(sample_date()); // today = 2026-05-19
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        // CURRENT BEHAVIOR: fresh_through satisfies freshness with zero signers.
        // classify_passed_predicate sees empty signer list and returns Execution directly.
        // Documents the v0.2 gap — fresh_through with no signers is unanchored freshness.
        assert_eq!(
            r.witness_tier,
            WitnessTier::Execution,
            "NFA-23 documented gap: fresh_through with no signers satisfies freshness leaf; \
             nobody has reviewed the item but it appears 'fresh'; v0.2 fix: require signer co-presence"
        );
        assert_eq!(
            r.signature_strength, None,
            "no signers → signature_strength must be None"
        );
    }

    #[test]
    fn signature_allow_enforced_against_current_fp_entry_strength_nfa24() {
        // SILENT FAILURE CANDIDATE (adversarial NFA-24): `eval_signers` with
        // `against = SignerCurrency::Any` and a non-empty `signature_allow` list
        // should enforce strength against ALL candidate entries, but the
        // `any_candidate_satisfies` loop might match a STALE entry that happens to
        // pass the `signature_allow` constraint when the CURRENT entry does not —
        // or vice versa.
        //
        // Specifically: if `signature_allow = [GitTrust, CryptoSigned]` (disallow
        // TextStamp), and alice has a stale entry with GitTrust AND a current entry
        // with TextStamp (she downgraded her signing tier), `against = Any` means
        // the stale GitTrust entry satisfies the allow-list. The predicate passes
        // — but the current attestation is at TextStamp, which is below the allow-list.
        //
        // This is a SILENT FAILURE: the predicate passes because `against=Any`
        // lets the stale entry satisfy both currency AND strength — but the signer's
        // CURRENT commitment is TextStamp-only.
        //
        // FIX DIRECTION: `signature_allow` enforcement should be checked only against
        // entries that also satisfy the currency constraint. For `against=Current`,
        // this is already correct (currency_ok gates strength_ok). For `against=Any`,
        // the strength must be checked against current-fp entries specifically (or
        // the predicate must require `against=Current` when `signature_allow` is set).
        //
        // This test documents the current behavior.
        let alice_stale_git = Signer {
            name: "alice".to_string(),
            role: None,
            date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            signed_against_fingerprint: "fp-old".to_string(), // stale
            basis: crate::schema::SignerBasis::Fresh { reasoning: None },
            strength: SignatureStrength::GitTrust, // passes allow-list
            signature: None,
        };
        let alice_current_text = Signer {
            name: "alice".to_string(),
            role: None,
            date: sample_date(),
            signed_against_fingerprint: "fp-current".to_string(), // current
            basis: crate::schema::SignerBasis::Fresh { reasoning: None },
            strength: SignatureStrength::TextStamp, // BELOW allow-list
            signature: None,
        };
        let item = item_with(vec![alice_stale_git, alice_current_text]);
        let pred = Predicate::leaf(Leaf::Signers {
            required: vec!["alice".to_string()],
            roles: BTreeMap::new(),
            against: SignerCurrency::Any, // stale entries count for currency
            signature_allow: vec![SignatureStrength::GitTrust, SignatureStrength::CryptoSigned],
            signature_prefer: None,
        });
        let ctx = TestContext::new(sample_date());
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        // CURRENT BEHAVIOR: the stale GitTrust entry satisfies both currency (Any=always true)
        // and strength (in allow-list), so the predicate PASSES even though alice's
        // current-fp attestation is TextStamp (below the allow-list). alice HAS a
        // current-fp entry so stale_count=0 (per-name NFA-18 fix). Classification: Execution.
        //
        // The gap: `against=Any` + `signature_allow` is unsound — a stale entry at a
        // higher tier satisfies the allow-list while the signer's actual CURRENT tier is
        // below it. This should fail: the signer's current commitment is TextStamp-only.
        //
        // This documents the v0.2 gap — fix by requiring strength checks against
        // current-fp entries specifically when against=Any.
        assert_eq!(
            r.witness_tier,
            WitnessTier::Execution,
            "NFA-24 documented gap: against=Any + signature_allow lets stale GitTrust entry \
             satisfy allow-list when current-fp entry is TextStamp (below allow-list); \
             CORRECT behavior would be WitnessTier::None (predicate should fail)"
        );
    }

    #[test]
    fn doc_without_trailing_newline_after_closing_frontmatter_silently_fails_nfa25() {
        // SILENT FAILURE (adversarial NFA-25): `parse_frontmatter_field` uses
        // `stripped.find("\n---\n")` to locate the closing delimiter. If the doc
        // ends immediately after `---` with no trailing newline — which is a
        // common authoring pattern — the terminator `\n---\n` never matches and
        // the function returns None. The doc version/anchor is silently dropped,
        // and `RatifiedDoc` fails the version/anchor check even when the content
        // IS correct.
        //
        // Example: "---\nversion: 2.0\n---" (no trailing newline) → None.
        // Correct:  "---\nversion: 2.0\n---\n" (trailing newline) → Some("2.0").
        //
        // This is a SILENT FAILURE: the audit reports "doc fails version check"
        // with no indication that a missing trailing newline is the cause.
        //
        // FIX DIRECTION: also try `find("\n---")` at end-of-string (i.e., the
        // closing delimiter may be at EOF without a subsequent newline). A simple
        // fix: also check if the frontmatter ends with `\n---` at end-of-string.
        // Or normalize content by appending `\n` if absent before parsing.
        //
        // This test FAILS against the current code because the closing `---` has
        // no trailing newline and find("\n---\n") returns None.
        let item = item_with(vec![]);
        let pred = Predicate::leaf(Leaf::RatifiedDoc {
            path: Some(PathBuf::from("docs/sinh.md")),
            min_version: Some("1.0".to_string()),
            anchor: None,
            sibling_json: false,
        });
        // No trailing newline after the closing `---`.
        let no_trailing_nl = "---\nversion: 2.0\n---";
        let ctx = TestContext::new(sample_date()).with_doc("docs/sinh.md", no_trailing_nl);
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        // The version IS 2.0 (satisfies min 1.0), but the parser fails to find
        // the frontmatter due to the missing trailing newline. The predicate fails silently.
        assert_eq!(
            r.witness_tier,
            WitnessTier::Execution,
            "NFA-25: doc with version 2.0 and no trailing newline after closing --- \
             must not silently fail the version check; parser must handle EOF-terminated frontmatter"
        );
    }

    #[test]
    fn doc_crlf_line_endings_not_silently_rejected_nfa6b() {
        // Same CRLF silent failure in parse_frontmatter_version — a doc with
        // CRLF line endings has its version silently dropped, causing
        // RatifiedDoc to fail even when the version IS sufficient.
        let item = item_with(vec![]);
        let pred = Predicate::leaf(Leaf::RatifiedDoc {
            path: Some(PathBuf::from("docs/sinh.md")),
            min_version: Some("1.0".to_string()),
            anchor: None,
            sibling_json: false,
        });
        // Doc with CRLF — version: 2.0 is present but CRLF breaks parsing.
        let crlf_doc = "---\r\nversion: 2.0\r\n---\r\n# Sinh discipline\r\nbody";
        let ctx = TestContext::new(sample_date()).with_doc("docs/sinh.md", crlf_doc);
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        assert_eq!(
            r.witness_tier,
            WitnessTier::Execution,
            "CRLF-encoded doc with version: 2.0 must not be silently rejected"
        );
    }

    #[test]
    fn oracle_without_trailing_newline_not_silently_rejected_nfa25b() {
        // Same NFA-25 class as the doc variant above but for oracle files.
        // `parse_oracle_status` uses the same `parse_frontmatter_field` function.
        let item = item_with(vec![]);
        let pred = Predicate::leaf(Leaf::OraclesComplete {
            files: vec![PathBuf::from("docs/oracles/o.md")],
        });
        // Oracle with no trailing newline after closing `---`.
        let no_trailing_nl = "---\nstatus: complete\n---";
        let ctx = TestContext::new(sample_date()).with_oracle("docs/oracles/o.md", no_trailing_nl);
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        assert_eq!(
            r.witness_tier,
            WitnessTier::Execution,
            "NFA-25b: oracle with status: complete and no trailing newline after closing --- \
             must not be silently treated as missing/incomplete"
        );
    }

    #[test]
    fn oracle_state_machine_draft_blocks_oracles_complete_predicate_nfa26() {
        // DOCUMENTED GAP (evaluator NFA-26): `eval_oracles_complete` reads the
        // oracle file on disk and checks `status: complete` in the frontmatter.
        // It does NOT consult the Oracle artifact-class state machine in the sidecar.
        //
        // ADR-021 §D3: Draft oracles must block the `oracles_complete(...)` predicate.
        // But the evaluator bypasses the sidecar's `item.oracles[*].state` entirely —
        // the predicate evaluates against the raw file content, not the Oracle state.
        //
        // A sidecar that declares `oracle.state = Draft` with a corresponding file
        // that has `status: complete` in its frontmatter would PASS the predicate —
        // even though the oracle has not been authoritatively established (Draft state).
        //
        // This is a SILENT FAILURE: the Oracle state machine guard is unenforced
        // at evaluation time. The implementation gap is in `eval_oracles_complete`,
        // which needs to be updated to check `item.oracles[*].state` when a
        // matching Oracle artifact-class entry exists.
        //
        // FIX DIRECTION (task #60/61): `eval_oracles_complete` must be extended to
        // accept the `item` parameter and look up each file path in `item.oracles`
        // by matching `oracle.reference` against the file path. If a matching Oracle
        // is found and its state is NOT `Complete`, the leaf must return false
        // (with an appropriate audit hint: `oracle-draft-blocks-attestation`,
        // `oracle-deprecated`, `oracle-revoked`).
        //
        // This test DOCUMENTS the current behavior. It will start failing when the
        // fix is applied (the assertion will change from Execution to None).
        use crate::schema::{Oracle, OracleRef, OracleState, OracleVersion, Provenance, Steward};
        use std::collections::BTreeMap as BM;

        let draft_oracle = Oracle {
            id: "test-oracle".to_string(),
            reference: OracleRef::LocalFile {
                path: PathBuf::from("docs/oracles/o.md"),
                status_field: None,
                expected_status: None,
            },
            state: OracleState::Draft, // NOT yet Complete
            stewards: vec![
                Steward {
                    name: "alice".to_string(),
                    role: None,
                    authorization_basis: "domain authority".to_string(),
                },
                Steward {
                    name: "bob".to_string(),
                    role: None,
                    authorization_basis: "tech-lead".to_string(),
                },
            ],
            created: Provenance {
                recorded_by: "alice".to_string(),
                at: sample_date(),
            },
            version: OracleVersion {
                pinned: "v0".to_string(),
                pinned_at: sample_date(),
            },
            transitions: vec![],
            extensions: BM::new(),
        };
        let mut item = item_with(vec![]);
        item.oracles = vec![draft_oracle];

        let pred = Predicate::leaf(Leaf::OraclesComplete {
            files: vec![PathBuf::from("docs/oracles/o.md")],
        });
        // File exists and has status: complete — but oracle is in Draft state.
        let ctx = TestContext::new(sample_date())
            .with_oracle("docs/oracles/o.md", "---\nstatus: complete\n---\nbody");
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();

        // NFA-26 FIX: Draft oracle state now blocks the oracles_complete predicate
        // even when the underlying file has `status: complete`. The evaluator checks
        // Oracle artifact-class state before falling back to file-content evaluation.
        assert_eq!(
            r.witness_tier,
            WitnessTier::None,
            "NFA-26: Draft oracle must block oracles_complete predicate; \
             file content alone cannot satisfy the leaf when the oracle is in Draft state"
        );
        assert_eq!(
            r.audit_hint,
            AuditHint::DisciplinePredicateFailed,
            "NFA-26: Draft-blocked predicate must emit DisciplinePredicateFailed hint"
        );
    }

    #[test]
    fn oracle_revoked_invalidates_true_blocks_predicate_nfa26b() {
        // Revoked(invalidates=true) oracle must block — incorrect oracle,
        // prior attestations retroactively demoted.
        use crate::schema::{Oracle, OracleRef, OracleState, OracleVersion, Provenance, Steward};
        let revoked = Oracle {
            id: "revoked".to_string(),
            reference: OracleRef::LocalFile {
                path: PathBuf::from("docs/oracles/rev.md"),
                status_field: None,
                expected_status: None,
            },
            state: OracleState::Revoked {
                reason: "oracle discipline was incorrect".to_string(),
                revoked_by: "alice".to_string(),
                invalidates_prior_attestations: true,
            },
            stewards: vec![
                Steward {
                    name: "alice".to_string(),
                    role: None,
                    authorization_basis: "domain authority".to_string(),
                },
                Steward {
                    name: "bob".to_string(),
                    role: None,
                    authorization_basis: "tech-lead".to_string(),
                },
            ],
            created: Provenance {
                recorded_by: "alice".to_string(),
                at: sample_date(),
            },
            version: OracleVersion {
                pinned: "v0".to_string(),
                pinned_at: sample_date(),
            },
            transitions: vec![],
            extensions: std::collections::BTreeMap::new(),
        };
        let mut item = item_with(vec![]);
        item.oracles = vec![revoked];
        let pred = Predicate::leaf(Leaf::OraclesComplete {
            files: vec![PathBuf::from("docs/oracles/rev.md")],
        });
        let ctx = TestContext::new(sample_date())
            .with_oracle("docs/oracles/rev.md", "---\nstatus: complete\n---\nbody");
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        assert_eq!(
            r.witness_tier,
            WitnessTier::None,
            "NFA-26b: Revoked(invalidates=true) oracle must block oracles_complete predicate"
        );
    }

    #[test]
    fn oracle_deprecated_allows_predicate_to_pass_nfa26c() {
        // Deprecated oracle (superseded, not incorrect) passes — sign-time-validity:
        // prior attestations honored at Execution tier with a deprecation hint.
        use crate::schema::{Oracle, OracleRef, OracleState, OracleVersion, Provenance, Steward};
        let deprecated = Oracle {
            id: "deprecated".to_string(),
            reference: OracleRef::LocalFile {
                path: PathBuf::from("docs/oracles/dep.md"),
                status_field: None,
                expected_status: None,
            },
            state: OracleState::Deprecated {
                superseded_by: Some("new-oracle".to_string()),
                reason: "superseded by updated oracle".to_string(),
            },
            stewards: vec![
                Steward {
                    name: "alice".to_string(),
                    role: None,
                    authorization_basis: "domain authority".to_string(),
                },
                Steward {
                    name: "bob".to_string(),
                    role: None,
                    authorization_basis: "tech-lead".to_string(),
                },
            ],
            created: Provenance {
                recorded_by: "alice".to_string(),
                at: sample_date(),
            },
            version: OracleVersion {
                pinned: "v1".to_string(),
                pinned_at: sample_date(),
            },
            transitions: vec![],
            extensions: std::collections::BTreeMap::new(),
        };
        let mut item = item_with(vec![]);
        item.oracles = vec![deprecated];
        let pred = Predicate::leaf(Leaf::OraclesComplete {
            files: vec![PathBuf::from("docs/oracles/dep.md")],
        });
        let ctx = TestContext::new(sample_date())
            .with_oracle("docs/oracles/dep.md", "---\nstatus: complete\n---\nbody");
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        // Deprecated = superseded, not incorrect → predicate passes (sign-time-validity).
        assert_eq!(
            r.witness_tier,
            WitnessTier::Execution,
            "NFA-26c: Deprecated oracle (superseded) must allow predicate to pass"
        );
    }
}
