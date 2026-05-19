//! Substrate-witness predicate evaluator (ADR-019 §M5).
//!
//! Given a [`Predicate`] + a [`Ratification`] sidecar + an evaluation
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
use crate::schema::{ItemRatification, Signer};
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
        return Ok(EvaluatedPredicate {
            witness_tier: WitnessTier::None,
            audit_hint: AuditHint::DisciplinePredicateFailed,
            evidence_kind: EvidenceKind::SubstrateState,
            signature_strength: None,
        });
    }

    // 3. Predicate passed. Derive tier + hint from the sidecar's signer
    //    state: stale signers; delta-chain-near-cap; via-delta-chain;
    //    all-fresh. The state machine is the M5 table for immunity.
    Ok(classify_passed_predicate(item, current_fingerprint, ctx))
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
        } => eval_signers(required, roles, *against, item, current_fingerprint),
        Leaf::SignedTrailer { key, role, count } => eval_signed_trailer(
            key,
            role.as_deref(),
            *count,
            item_source_file,
            &item.item_path,
            ctx,
        ),
        Leaf::OraclesComplete { files } => eval_oracles_complete(files, ctx),
        Leaf::FreshWithinDays { days } => eval_fresh_within_days(*days, item, ctx),
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
    item: &ItemRatification,
    current_fingerprint: &str,
) -> bool {
    for needed in required {
        let candidates: Vec<&Signer> = item.signers.iter().filter(|s| s.name == *needed).collect();
        if candidates.is_empty() {
            return false;
        }
        // Apply currency policy.
        let any_current_enough = match against {
            SignerCurrency::Current => candidates
                .iter()
                .any(|s| s.signed_against_fingerprint == current_fingerprint),
            SignerCurrency::Any => true,
        };
        if !any_current_enough {
            return false;
        }
        // Apply role assertion (if specified for this name).
        if let Some(expected_role) = roles.get(needed) {
            let any_role_match = candidates
                .iter()
                .any(|s| s.role.as_deref() == Some(expected_role.as_str()));
            if !any_role_match {
                return false;
            }
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

fn eval_oracles_complete<C: EvaluationContext>(files: &[std::path::PathBuf], ctx: &C) -> bool {
    files.iter().all(|p| {
        ctx.read_oracle(p)
            .is_some_and(|content| parse_oracle_status(&content).as_deref() == Some("complete"))
    })
}

fn eval_fresh_within_days<C: EvaluationContext>(
    days: u32,
    item: &ItemRatification,
    ctx: &C,
) -> bool {
    // Most recent signer's date OR `fresh_through` — whichever is later.
    let latest_signer = item.signers.iter().map(|s| s.date).max();
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
/// into the audit hint per ADR-019 §M5 (immunity table).
fn classify_passed_predicate<C: EvaluationContext>(
    item: &ItemRatification,
    current_fingerprint: &str,
    ctx: &C,
) -> EvaluatedPredicate {
    // No signers at all = predicate passed via non-signer leaves only
    // (e.g., `ratified_doc + oracles_complete + fresh_within_days` with
    // no `signers` leaf). The result is Execution-tier substrate-current
    // because all checked leaves passed.
    if item.signers.is_empty() {
        return EvaluatedPredicate {
            witness_tier: WitnessTier::Execution,
            audit_hint: AuditHint::DisciplinePredicatePassedSubstrateCurrent,
            evidence_kind: EvidenceKind::SubstrateState,
            signature_strength: Some(SignatureStrength::GitTrust),
        };
    }

    // Some signers exist. Detect stale signers (signed against a
    // non-current fingerprint).
    let stale_count = item
        .signers
        .iter()
        .filter(|s| s.signed_against_fingerprint != current_fingerprint)
        .count();
    if stale_count > 0 {
        return EvaluatedPredicate {
            witness_tier: WitnessTier::Reachability,
            audit_hint: AuditHint::DisciplineSubstrateStale,
            evidence_kind: EvidenceKind::SubstrateState,
            signature_strength: Some(SignatureStrength::GitTrust),
        };
    }

    // All signers current. Check delta-chain state.
    let cap = ctx.delta_chain_cap();
    let max_chain_depth = item
        .signers
        .iter()
        .map(|s| s.basis.chain_depth())
        .max()
        .unwrap_or(0);
    let has_delta = item.signers.iter().any(|s| s.basis.is_delta());

    if max_chain_depth >= cap.saturating_sub(1) && max_chain_depth > 0 {
        return EvaluatedPredicate {
            witness_tier: WitnessTier::Execution,
            audit_hint: AuditHint::DisciplineSubstrateDeltaChainNearCap,
            evidence_kind: EvidenceKind::SubstrateState,
            signature_strength: Some(SignatureStrength::GitTrust),
        };
    }

    if has_delta {
        return EvaluatedPredicate {
            witness_tier: WitnessTier::Execution,
            audit_hint: AuditHint::DisciplinePredicatePassedViaDeltaChain,
            evidence_kind: EvidenceKind::SubstrateState,
            signature_strength: Some(SignatureStrength::GitTrust),
        };
    }

    // All signers current and all Fresh — strongest v0.1 state.
    EvaluatedPredicate {
        witness_tier: WitnessTier::Execution,
        audit_hint: AuditHint::DisciplinePredicatePassedSubstrateCurrent,
        evidence_kind: EvidenceKind::SubstrateState,
        signature_strength: Some(SignatureStrength::GitTrust),
    }
}

// --- Frontmatter / oracle parsing helpers --------------------------------

/// Parse `version: X.Y.Z` from the leading YAML frontmatter of a doc.
/// Returns None if no frontmatter, no version field, or non-string value.
fn parse_frontmatter_version(content: &str) -> Option<String> {
    let stripped = content.strip_prefix("---\n")?;
    let end = stripped.find("\n---\n")?;
    let frontmatter = &stripped[..end];
    for line in frontmatter.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("version:") {
            return Some(rest.trim().trim_matches('"').trim_matches('\'').to_string());
        }
    }
    None
}

/// Parse `status: <value>` from an oracle file's YAML frontmatter.
fn parse_oracle_status(content: &str) -> Option<String> {
    let stripped = content.strip_prefix("---\n")?;
    let end = stripped.find("\n---\n")?;
    let frontmatter = &stripped[..end];
    for line in frontmatter.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("status:") {
            return Some(rest.trim().trim_matches('"').trim_matches('\'').to_string());
        }
    }
    None
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
        AntigenIdentifier, DocRef, ItemRatification, OracleRef, Ratification, RatificationKind,
        SchemaVersion, Signer, SignerBasis,
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
            signature: None,
        };
        let item = item_with(vec![signer]);
        let pred = Predicate::leaf(Leaf::Signers {
            required: vec!["alice".to_string()],
            roles: BTreeMap::new(),
            against: SignerCurrency::Current,
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
            signature: None,
        };
        let item = item_with(vec![signer]);
        let pred = Predicate::leaf(Leaf::Signers {
            required: vec!["alice".to_string()],
            roles: BTreeMap::new(),
            against: SignerCurrency::Current,
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
                oracles: vec![OracleRef {
                    path: PathBuf::from("o.md"),
                    status: None,
                }],
                fresh_through: None,
                extensions: BTreeMap::new(),
            }],
        };
    }
}
