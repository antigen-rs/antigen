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
    /// Per-leaf evaluation outcomes (Finding 7), in evaluation order. Lets
    /// `attest check` / `audit` render which leaf passed/failed and why —
    /// `expected X, found Y` — instead of an opaque tree-level
    /// `DisciplinePredicateFailed`. Empty for the infrastructure-failure
    /// constructors (`sidecar_missing` etc.) where no predicate walk occurred.
    pub leaf_outcomes: Vec<LeafOutcome>,
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
            leaf_outcomes: Vec::new(),
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
            leaf_outcomes: Vec::new(),
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
            leaf_outcomes: Vec::new(),
        }
    }
}

/// The outcome of evaluating a single predicate **leaf**, carrying both the
/// pass/fail verdict and a human-readable reason (Finding 7).
///
/// The reason is always populated — on PASS it states what was satisfied, on
/// FAIL it states expected-vs-found — so a failing compound predicate can be
/// explained leaf-by-leaf instead of collapsing to an opaque
/// `DisciplinePredicateFailed`. Debugging the name-vs-role confusion (Finding
/// 5) previously required reading the evaluator source; with this the audit
/// says `signers(required=["camp-maintainer"]): FAIL — no signer named
/// "camp-maintainer" (found names: ["Claude"])`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LeafOutcome {
    /// Short leaf label, e.g. `signers(required=["alice"])` or
    /// `fresh_within_days(90)`. Identifies which leaf this outcome is for.
    pub label: String,
    /// Whether the leaf passed.
    pub passed: bool,
    /// Human-readable reason. On FAIL: expected-vs-found. On PASS: what was
    /// satisfied. Never empty.
    pub reason: String,
    /// Whether this leaf was actually *evaluated* here. `true` for every leaf
    /// the standard evaluator runs. `false` for supply-chain leaves
    /// (`dep_pinned`, `dep_attested`, etc.) which the standard path does NOT
    /// evaluate — they're driven by `cargo antigen verify`. A not-evaluated
    /// leaf has `passed: false` (honest-tier-naming: it did not pass HERE), but
    /// the renderer must show `NOT-EVALUATED`, not `FAIL`, so an adopter does
    /// not mistake "the check was deferred" for "the check ran and failed"
    /// (ATK-eval-leaf-not-evaluated).
    ///
    /// CARDINALITY WATCH-ITEM (aristotle): this `bool` collapses three logical
    /// eval-states into two — `evaluated-here` (pass/fail), `not-here-but-
    /// elsewhere` (supply-chain → `cargo antigen verify`), and
    /// `not-evaluable-anywhere` (a genuine gap). At v0.2 every non-evaluated
    /// leaf is state-2 (its elsewhere is `verify`), so `false` unambiguously
    /// means "deferred elsewhere" and the bool is honest. If a leaf ever becomes
    /// un-evaluable by ANY layer (state-3), the bool would force it to
    /// masquerade as deferred-elsewhere (a `VecCardinalityMasqueradingAsSet`
    /// shadow) — that is the trigger to upgrade this field to a 3-state enum,
    /// not to add another `false` case.
    #[serde(default = "default_evaluated")]
    pub evaluated: bool,
}

/// Backward-compat default for [`LeafOutcome::evaluated`] on pre-this-field
/// serialized reports: assume a recorded leaf was evaluated.
const fn default_evaluated() -> bool {
    true
}

/// The evaluation of a predicate (sub)tree, mirroring [`Predicate`]'s shape so
/// that per-leaf diagnostics retain their `all_of` / `any_of` / `not` context
/// (Finding 7).
///
/// This is the **single** evaluation path: [`Self::passed`] is the boolean the
/// tier/hint classification consumes, and [`Self::leaf_outcomes`] is the
/// per-leaf diagnostic stream rendered by `attest check` / `audit`. There is no
/// separate "explain pass" — the verdict and the explanation are produced by
/// the same walk, so they cannot drift (a `RatifiedSpecDriftFromImpl` shape
/// that a parallel explain-pass would have risked).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalNode {
    /// A leaf evaluation with its outcome.
    Leaf(LeafOutcome),
    /// `all_of` — passes iff every child passes.
    AllOf(Vec<Self>),
    /// `any_of` — passes iff at least one child passes.
    AnyOf(Vec<Self>),
    /// `not` — passes iff the child does not.
    Not(Box<Self>),
}

/// Three-state composite verdict, distinguishing a genuine evaluation failure
/// from a deferred (not-yet-evaluated) leaf.
///
/// A deferred leaf (`passed: false, evaluated: false`) inside an `all_of`
/// must NOT map to `Failed` — the check was not run, not run-and-failed.
/// Collapsing Indeterminate → Failed produces `DisciplinePredicateFailed`
/// when the real state is "supply-chain audit needed here, not run yet."
/// That is a `SilentSemanticMismatchAtTrustBoundary` instance (V2 void from
/// ADR-029 Phase 8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositeVerdict {
    /// Every evaluated leaf passed; no deferred leaves present.
    Passed,
    /// At least one leaf evaluated to false (genuinely failed).
    Failed,
    /// No leaf evaluated to false, but ≥1 leaf was deferred (not evaluated
    /// by this evaluator — e.g. supply-chain leaves on the standard path).
    Indeterminate,
}

impl EvalNode {
    /// Three-state verdict for this (sub)tree. Prefer this over [`Self::passed`]
    /// whenever the caller needs to distinguish genuine failure from deferral.
    #[must_use]
    pub fn verdict(&self) -> CompositeVerdict {
        match self {
            Self::Leaf(o) => {
                if o.evaluated {
                    if o.passed {
                        CompositeVerdict::Passed
                    } else {
                        CompositeVerdict::Failed
                    }
                } else {
                    // Deferred leaf: not evaluated here, not a failure.
                    CompositeVerdict::Indeterminate
                }
            }
            Self::AllOf(children) => {
                let mut any_indeterminate = false;
                for c in children {
                    match c.verdict() {
                        CompositeVerdict::Failed => return CompositeVerdict::Failed,
                        CompositeVerdict::Indeterminate => any_indeterminate = true,
                        CompositeVerdict::Passed => {}
                    }
                }
                if any_indeterminate {
                    CompositeVerdict::Indeterminate
                } else {
                    CompositeVerdict::Passed
                }
            }
            Self::AnyOf(children) => {
                let mut any_indeterminate = false;
                for c in children {
                    match c.verdict() {
                        CompositeVerdict::Passed => return CompositeVerdict::Passed,
                        CompositeVerdict::Indeterminate => any_indeterminate = true,
                        CompositeVerdict::Failed => {}
                    }
                }
                if any_indeterminate {
                    CompositeVerdict::Indeterminate
                } else {
                    CompositeVerdict::Failed
                }
            }
            Self::Not(child) => match child.verdict() {
                CompositeVerdict::Passed => CompositeVerdict::Failed,
                CompositeVerdict::Failed => CompositeVerdict::Passed,
                // not(deferred) is still indeterminate — we don't know what
                // the deferred leaf would have returned.
                CompositeVerdict::Indeterminate => CompositeVerdict::Indeterminate,
            },
        }
    }

    /// Boolean verdict for this (sub)tree. Back-compat wrapper around
    /// [`Self::verdict`]: returns `true` only when verdict is `Passed`.
    /// Callers that need to distinguish `Failed` from `Indeterminate` must
    /// use `verdict()` directly.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.verdict() == CompositeVerdict::Passed
    }

    /// Flatten the tree to its leaf outcomes in evaluation order, for
    /// rendering. Composites (`all_of`/`any_of`/`not`) contribute their
    /// children's leaves; the verdict of a composite is derivable from its
    /// leaves plus the combinator, so the renderer can reconstruct context.
    #[must_use]
    pub fn leaf_outcomes(&self) -> Vec<LeafOutcome> {
        let mut out = Vec::new();
        self.collect_leaves(&mut out);
        out
    }

    fn collect_leaves(&self, out: &mut Vec<LeafOutcome>) {
        match self {
            Self::Leaf(o) => out.push(o.clone()),
            Self::AllOf(children) | Self::AnyOf(children) => {
                for c in children {
                    c.collect_leaves(out);
                }
            }
            Self::Not(child) => child.collect_leaves(out),
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
    //    returns an EvalNode tree carrying both the pass/fail verdict and
    //    per-leaf reasons (Finding 7: single eval path, no separate explain-pass).
    //    Axis classification happens after we know the verdict + per-signer state.
    let predicate_node = eval_pred(predicate, item, current_fingerprint, item_source_file, ctx);
    let leaf_outcomes = predicate_node.leaf_outcomes();

    match predicate_node.verdict() {
        CompositeVerdict::Failed => {
            let failed_hint = match kind {
                RatificationKind::Tolerance => AuditHint::TolerancePredicateFailed,
                RatificationKind::Immunity => AuditHint::DisciplinePredicateFailed,
            };
            return Ok(EvaluatedPredicate {
                witness_tier: WitnessTier::None,
                audit_hint: failed_hint,
                evidence_kind: EvidenceKind::SubstrateState,
                signature_strength: None,
                leaf_outcomes,
            });
        }
        CompositeVerdict::Indeterminate => {
            return Ok(EvaluatedPredicate {
                witness_tier: WitnessTier::None,
                audit_hint: AuditHint::DisciplinePredicateDeferred,
                evidence_kind: EvidenceKind::SubstrateState,
                signature_strength: None,
                leaf_outcomes,
            });
        }
        CompositeVerdict::Passed => {} // fall through to classify_passed_predicate
    }

    // 3. Predicate passed. Derive tier + hint from the sidecar's signer
    //    state: stale signers; delta-chain-near-cap; via-delta-chain;
    //    all-fresh. The M5 table differs between immunity and tolerance.
    let mut passed = classify_passed_predicate(item, current_fingerprint, kind, ctx);
    passed.leaf_outcomes = leaf_outcomes;
    Ok(passed)
}

/// Recursive predicate evaluation. Returns an [`EvalNode`] tree carrying both
/// the pass/fail verdict (via [`EvalNode::passed`]) and per-leaf reasons (via
/// [`EvalNode::leaf_outcomes`]). Axis classification happens in the caller after
/// we know the verdict plus the per-signer state (Finding 7: single eval path,
/// no separate explain-pass).
fn eval_pred<C: EvaluationContext>(
    p: &Predicate,
    item: &ItemRatification,
    current_fingerprint: &str,
    item_source_file: &Path,
    ctx: &C,
) -> EvalNode {
    match p {
        Predicate::Leaf(leaf) => EvalNode::Leaf(eval_leaf(
            leaf,
            item,
            current_fingerprint,
            item_source_file,
            ctx,
        )),
        Predicate::AllOf { children } => EvalNode::AllOf(
            children
                .iter()
                .map(|c| eval_pred(c, item, current_fingerprint, item_source_file, ctx))
                .collect(),
        ),
        Predicate::AnyOf { children } => EvalNode::AnyOf(
            children
                .iter()
                .map(|c| eval_pred(c, item, current_fingerprint, item_source_file, ctx))
                .collect(),
        ),
        Predicate::Not { child } => EvalNode::Not(Box::new(eval_pred(
            child,
            item,
            current_fingerprint,
            item_source_file,
            ctx,
        ))),
    }
}

fn eval_leaf<C: EvaluationContext>(
    leaf: &Leaf,
    item: &ItemRatification,
    current_fingerprint: &str,
    item_source_file: &Path,
    ctx: &C,
) -> LeafOutcome {
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
        | Leaf::SandboxClean { .. } => LeafOutcome {
            label: "supply-chain-leaf (not-evaluated)".to_string(),
            passed: false,
            evaluated: false,
            reason: "supply-chain leaves are not evaluated by the standard \
                     predicate evaluator; drive `cargo antigen verify` (the \
                     supply-chain audit layer) instead. Not a failure — the \
                     check was deferred, not run (ATK-eval-leaf-not-evaluated)."
                .to_string(),
        },
    }
}

fn eval_ratified_doc<C: EvaluationContext>(
    explicit_path: Option<&Path>,
    min_version: Option<&str>,
    anchor: Option<&str>,
    sibling_json: bool,
    item: &ItemRatification,
    ctx: &C,
) -> LeafOutcome {
    let label = "ratified_doc".to_string();
    let fail = |reason: String| LeafOutcome {
        label: label.clone(),
        passed: false,
        evaluated: true,
        reason,
    };

    // Resolve the doc path: prefer explicit `path`, fall back to item's
    // `doc_ref.path` (one indirection).
    let path =
        match explicit_path {
            Some(p) => p.to_path_buf(),
            None => match &item.doc_ref {
                Some(dr) => dr.path.clone(),
                None => return fail(
                    "no doc to check — neither an explicit `path` nor an item `doc_ref` was set"
                        .to_string(),
                ),
            },
        };

    let Some(content) = ctx.read_doc(&path) else {
        return fail(format!("doc not found or unreadable: `{}`", path.display()));
    };

    // Frontmatter version check (if min_version is requested).
    if let Some(min) = min_version {
        let Some(found_version) = parse_frontmatter_version(&content) else {
            return fail(format!(
                "doc `{}` has no parseable frontmatter version (required min_version `{min}`)",
                path.display()
            ));
        };
        if compare_versions(&found_version, min) == std::cmp::Ordering::Less {
            return fail(format!(
                "doc `{}` version `{found_version}` is below required min_version `{min}`",
                path.display()
            ));
        }
    }

    // Anchor check (if requested). Simple substring check for now; a
    // future amendment can sharpen to YAML-frontmatter-aware or
    // markdown-heading-slug-aware checking.
    if let Some(a) = anchor {
        if !content.contains(a) {
            return fail(format!(
                "doc `{}` does not contain required anchor `{a}`",
                path.display()
            ));
        }
    }

    // Sibling JSON check (if requested).
    if sibling_json {
        let sibling = sibling_json_path(&path);
        if ctx.read_doc(&sibling).is_none() {
            return fail(format!(
                "required sibling JSON `{}` not found next to doc `{}`",
                sibling.display(),
                path.display()
            ));
        }
    }

    LeafOutcome {
        label,
        passed: true,
        evaluated: true,
        reason: format!("doc `{}` satisfied all checks", path.display()),
    }
}

fn eval_signers(
    required: &[String],
    roles: &std::collections::BTreeMap<String, String>,
    against: SignerCurrency,
    signature_allow: &[crate::tier::SignatureStrength],
    _signature_prefer: Option<crate::tier::SignatureStrength>,
    item: &ItemRatification,
    current_fingerprint: &str,
) -> LeafOutcome {
    let label = format!("signers(required={required:?})");
    let present_names: Vec<&str> = item.signers.iter().map(|s| s.name.as_str()).collect();
    let fail = |reason: String| LeafOutcome {
        label: label.clone(),
        passed: false,
        evaluated: true,
        reason,
    };

    for needed in required {
        let candidates: Vec<&Signer> = item.signers.iter().filter(|s| s.name == *needed).collect();
        if candidates.is_empty() {
            // The Finding-5 case: `required` is matched against signer NAME, not
            // role. If `needed` happens to be a role one of the present signers
            // holds, say so explicitly — that is the exact confusion that cost a
            // 20-minute source dive.
            let role_held_by_someone = item
                .signers
                .iter()
                .any(|s| s.role.as_deref() == Some(needed.as_str()));
            let hint = if role_held_by_someone {
                format!(
                    " — note: `{needed}` is a signer ROLE here, not a name; \
                     `required=[...]` matches signer NAMES. Did you mean \
                     `roles={{<name> = \"{needed}\"}}`?"
                )
            } else {
                String::new()
            };
            return fail(format!(
                "no signer named `{needed}` (found names: {present_names:?}){hint}"
            ));
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
            // NFA-21 discipline: strength is checked only against CURRENT-fp entries —
            // a stale high-tier entry must not satisfy the allow-list (ATK-NFA-24).
            let strength_ok = signature_allow.is_empty()
                || (s.signed_against_fingerprint == current_fingerprint
                    && signature_allow.contains(&s.strength));
            currency_ok && role_ok && strength_ok
        });
        if !any_candidate_satisfies {
            // Build a specific reason: which sub-constraint failed for `needed`?
            // Report the first unmet axis across the candidate entries.
            let mut reasons: Vec<String> = Vec::new();
            if matches!(against, SignerCurrency::Current)
                && !candidates
                    .iter()
                    .any(|s| s.signed_against_fingerprint == current_fingerprint)
            {
                reasons.push(format!(
                    "no entry for `{needed}` signed against the current fingerprint \
                     (against=\"current\"; current=`{current_fingerprint}`)"
                ));
            }
            if let Some(r) = expected_role {
                if !candidates
                    .iter()
                    .any(|s| s.role.as_deref() == Some(r.as_str()))
                {
                    let found_roles: Vec<Option<&str>> =
                        candidates.iter().map(|s| s.role.as_deref()).collect();
                    reasons.push(format!(
                        "`{needed}` is required to hold role `{r}` but its entries' \
                         roles are {found_roles:?}"
                    ));
                }
            }
            if !signature_allow.is_empty()
                && !candidates
                    .iter()
                    .any(|s| signature_allow.contains(&s.strength))
            {
                let found_strengths: Vec<_> = candidates.iter().map(|s| s.strength).collect();
                reasons.push(format!(
                    "`{needed}` signed at strengths {found_strengths:?} but \
                     signature_allow requires one of {signature_allow:?}"
                ));
            }
            let detail = if reasons.is_empty() {
                "no single entry satisfied currency + role + strength together \
                 (NFA-13: must be the SAME entry)"
                    .to_string()
            } else {
                reasons.join("; ")
            };
            return fail(format!(
                "signer `{needed}` present but unsatisfied: {detail}"
            ));
        }
    }
    LeafOutcome {
        label,
        passed: true,
        evaluated: true,
        reason: format!("all required signers satisfied: {required:?}"),
    }
}

fn eval_signed_trailer<C: EvaluationContext>(
    key: &str,
    role: Option<&str>,
    count: u32,
    item_source_file: &Path,
    item_path: &str,
    ctx: &C,
) -> LeafOutcome {
    let label = format!("signed_trailer(key={key:?})");
    let trailers = ctx.read_git_trailers(item_source_file, item_path);
    let mut hits: u32 = 0;
    for line in &trailers {
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
    }
    let role_clause = role.map_or(String::new(), |r| format!(" with role `{r}`"));
    if hits >= count {
        LeafOutcome {
            label,
            passed: true,
            evaluated: true,
            reason: format!("found {hits} matching `{key}` trailer(s){role_clause} (need {count})"),
        }
    } else {
        LeafOutcome {
            label,
            passed: false,
            evaluated: true,
            reason: format!(
                "found {hits} matching `{key}` trailer(s){role_clause} across \
                 {} git trailer line(s); need {count}",
                trailers.len()
            ),
        }
    }
}

fn eval_oracles_complete<C: EvaluationContext>(
    files: &[std::path::PathBuf],
    item: &ItemRatification,
    ctx: &C,
) -> LeafOutcome {
    use crate::schema::{OracleRef, OracleState};
    let label = "oracles_complete".to_string();
    let check = |p: &std::path::PathBuf| -> Result<(), String> {
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
            return Err(format!(
                "oracle `{}` is Draft or Revoked(invalidates=true) — blocks attestation",
                p.display()
            ));
        }

        if ctx
            .read_oracle(p)
            .is_some_and(|content| parse_oracle_status(&content).as_deref() == Some("complete"))
        {
            Ok(())
        } else {
            Err(format!(
                "oracle `{}` is missing or its status is not `complete`",
                p.display()
            ))
        }
    };

    for p in files {
        if let Err(reason) = check(p) {
            return LeafOutcome {
                label,
                passed: false,
                evaluated: true,
                reason,
            };
        }
    }
    LeafOutcome {
        label,
        passed: true,
        evaluated: true,
        reason: format!("all {} oracle(s) complete", files.len()),
    }
}

fn eval_fresh_within_days<C: EvaluationContext>(
    days: u32,
    item: &ItemRatification,
    current_fingerprint: &str,
    ctx: &C,
) -> LeafOutcome {
    let label = format!("fresh_within_days({days})");
    // Most recent CURRENT-fingerprint signer's date OR `fresh_through` — whichever is later.
    // NFA-21: stale-fingerprint signer dates must not satisfy a freshness check — a signer
    // who attested against fp-old months ago and never re-attested should not count as fresh.
    let latest_signer = item
        .signers
        .iter()
        .filter(|s| s.signed_against_fingerprint == current_fingerprint)
        .map(|s| s.date)
        .max();
    // ATK-FT-1/2: `fresh_through` EXTENDS a real current-fingerprint attestation's
    // freshness window; it does NOT SUBSTITUTE for one. Without at least one
    // current-fp signer, `fresh_through` is an unwitnessed date a sidecar writer
    // can set to `today` to bypass review entirely (the temporal forged-freshness
    // class, sibling of the S4 frame-expiry bypass ATK-PRES-13). So `fresh_through`
    // counts ONLY when a current-fp signer anchors it; with no such signer the leaf
    // fails as "freshness not met" — a clean rejection, never SubstrateStale
    // (which would falsely read "reviewed but old").
    let candidate = match (latest_signer, item.fresh_through) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (Some(a), None) => Some(a),
        // No current-fp signer: `fresh_through` alone cannot anchor freshness.
        (None, _) => None,
    };
    let Some(latest) = candidate else {
        let reason = if item.fresh_through.is_some() {
            "`fresh_through` is set but NO current-fingerprint signer anchors it — \
             an unwitnessed freshness date cannot satisfy the gate (ATK-FT-1/2: \
             fresh_through extends a real attestation, it does not substitute for one)"
                .to_string()
        } else {
            "no current-fingerprint signer date and no `fresh_through` — \
             nothing to measure freshness against (NFA-21: stale-fingerprint \
             dates are excluded)"
                .to_string()
        };
        return LeafOutcome {
            label,
            passed: false,
            evaluated: true,
            reason,
        };
    };
    let today = ctx.today();
    let diff_days = (today - latest).num_days();
    let fresh = diff_days >= 0 && i128::from(diff_days) <= i128::from(days);
    LeafOutcome {
        label,
        passed: fresh,
        evaluated: true,
        reason: if fresh {
            format!("latest attestation {latest} is {diff_days} day(s) old (≤ {days})")
        } else if diff_days < 0 {
            format!("latest attestation {latest} is in the future relative to {today}")
        } else {
            format!("latest attestation {latest} is {diff_days} day(s) old (> {days})")
        },
    }
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
            leaf_outcomes: Vec::new(),
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
            leaf_outcomes: Vec::new(),
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
            leaf_outcomes: Vec::new(),
        };
    }

    if has_delta {
        return EvaluatedPredicate {
            witness_tier: WitnessTier::Execution,
            audit_hint: AuditHint::DisciplinePredicatePassedViaDeltaChain,
            evidence_kind: EvidenceKind::SubstrateState,
            signature_strength: Some(min_strength),
            leaf_outcomes: Vec::new(),
        };
    }

    // All signers current and all Fresh — strongest v0.1 state.
    EvaluatedPredicate {
        witness_tier: WitnessTier::Execution,
        audit_hint: passed_hint,
        evidence_kind: EvidenceKind::SubstrateState,
        signature_strength: Some(min_strength),
        leaf_outcomes: Vec::new(),
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

    // Finding 7: a failed predicate must carry a per-leaf reason, and the
    // name-vs-role confusion (Finding 5) must be named explicitly rather than
    // collapsing to an opaque DisciplinePredicateFailed.
    #[test]
    fn signers_leaf_failure_reports_per_leaf_reason() {
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
        assert_eq!(r.leaf_outcomes.len(), 1, "one signers leaf → one outcome");
        let leaf = &r.leaf_outcomes[0];
        assert!(!leaf.passed);
        assert!(
            leaf.reason.contains("no signer named `bob`") && leaf.reason.contains("alice"),
            "leaf reason must state expected-vs-found: {}",
            leaf.reason
        );
    }

    #[test]
    fn signers_leaf_failure_names_the_role_vs_name_confusion() {
        // Finding 5: alice holds the ROLE "math-researcher"; an adopter who puts
        // the role into `required` (instead of `roles={alice="math-researcher"}`)
        // must get a reason that points at the confusion, not silence.
        let mut alice = alice_fresh(sample_date(), "fp-current");
        alice.role = Some("math-researcher".to_string());
        let item = item_with(vec![alice]);
        let pred = Predicate::leaf(Leaf::Signers {
            required: vec!["math-researcher".to_string()],
            roles: BTreeMap::new(),
            against: SignerCurrency::Current,
            signature_allow: vec![],
            signature_prefer: None,
        });
        let ctx = TestContext::new(sample_date());
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        let leaf = &r.leaf_outcomes[0];
        assert!(!leaf.passed);
        assert!(
            leaf.reason.contains("ROLE") && leaf.reason.contains("roles="),
            "leaf reason must name the name-vs-role confusion and suggest roles={{...}}: {}",
            leaf.reason
        );
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
        // GAP CLOSED (adversarial NFA-23, fixed via ATK-FT-1/2): `eval_fresh_within_days`
        // used to accept `item.fresh_through` as an anchor date even when the sidecar had
        // NO signers — a sidecar with `signers = []` but `fresh_through = today` satisfied
        // the freshness leaf, so nobody had attested yet the item appeared "fresh." That
        // was the temporal forged-freshness bypass.
        //
        // The fix: `fresh_through` EXTENDS a real current-fingerprint attestation's
        // freshness window; it does NOT SUBSTITUTE for one. With no current-fp signer the
        // freshness leaf now FAILS (DisciplinePredicateFailed) — a clean rejection.
        //
        // This test now asserts the CORRECTED behavior (it formerly documented the gap).
        let mut item = item_with(vec![]); // NO signers
                                          // fresh_through = today, but no signer anchors it.
        item.fresh_through = Some(sample_date());
        let pred = Predicate::leaf(Leaf::FreshWithinDays { days: 60 });
        let ctx = TestContext::new(sample_date()); // today = 2026-05-19
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        // CORRECTED: unanchored fresh_through no longer satisfies freshness.
        assert_eq!(
            r.witness_tier,
            WitnessTier::None,
            "NFA-23 (closed by ATK-FT-1/2): fresh_through with no signers must NOT satisfy \
             the freshness leaf — an unwitnessed date cannot anchor freshness; got {:?}",
            r.witness_tier
        );
        assert_eq!(
            r.audit_hint,
            AuditHint::DisciplinePredicateFailed,
            "unanchored fresh_through ⇒ a clean freshness failure, not a pass"
        );
    }

    #[test]
    fn signature_allow_enforced_against_current_fp_entry_strength_nfa24() {
        // NFA-24 FIX VERIFICATION: `eval_signers` with `against = SignerCurrency::Any`
        // and a non-empty `signature_allow` list must enforce strength against
        // CURRENT-fp entries only. A stale high-tier entry must not satisfy the
        // allow-list when the signer's current-fp entry is below it.
        //
        // Scenario: alice has a stale GitTrust entry (fp-old) and a current TextStamp
        // entry (fp-current); signature_allow=[GitTrust, CryptoSigned]. With the fix,
        // the stale GitTrust entry must NOT satisfy the allow-list — only the current
        // TextStamp entry counts, which is below the allow-list, so the predicate FAILS.
        //
        // This test was updated when ATK-NFA-24 was fixed; the prior version documented
        // the wrong (pre-fix) behavior of WitnessTier::Execution.
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
        // CORRECT BEHAVIOR (post-fix): the allow-list is enforced against current-fp
        // entries only. Alice's current-fp entry is TextStamp (below the allow-list) —
        // the predicate FAILS (WitnessTier::None). Her stale GitTrust entry at fp-old
        // does not satisfy the allow-list because it is not current-fp.
        assert_eq!(
            r.witness_tier,
            WitnessTier::None,
            "NFA-24 fix: against=Any + signature_allow must enforce strength against \
             current-fp entries only; alice's current-fp is TextStamp (below allow-list) \
             so predicate must fail; got {:?}",
            r.witness_tier
        );
    }

    /// ATK-NFA-24: `against=Any` + `signature_allow` must enforce strength against
    /// current-fingerprint entries, not let a stale high-tier entry bypass the
    /// allow-list when the signer's current-fp entry is below it.
    ///
    /// CONTRACT (FAILING until fixed): when alice's CURRENT attestation is
    /// `TextStamp` (below `[GitTrust, CryptoSigned]`) and her STALE entry is
    /// `GitTrust`, `against=Any` must NOT let the stale entry satisfy the
    /// allow-list. The predicate must FAIL (`WitnessTier::None`).
    ///
    /// FALSIFICATION: run without the fix → `WitnessTier::Execution` (wrong).
    /// Run with the fix → `WitnessTier::None` (correct, test passes).
    ///
    /// FIX DIRECTION: in `eval_signers`, when `signature_allow` is non-empty,
    /// check strength only against entries that are current-fp
    /// (`signed_against_fingerprint == current_fp`). A stale entry at a higher
    /// tier must not satisfy the allow-list constraint.
    #[test]
    fn atk_nfa24_signature_allow_against_any_must_enforce_against_current_fp_only() {
        let alice_stale_git = Signer {
            name: "alice".to_string(),
            role: None,
            date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            signed_against_fingerprint: "fp-old".to_string(), // stale — must NOT satisfy allow-list
            basis: crate::schema::SignerBasis::Fresh { reasoning: None },
            strength: SignatureStrength::GitTrust, // passes allow-list but is STALE
            signature: None,
        };
        let alice_current_text = Signer {
            name: "alice".to_string(),
            role: None,
            date: sample_date(),
            signed_against_fingerprint: "fp-current".to_string(), // current
            basis: crate::schema::SignerBasis::Fresh { reasoning: None },
            strength: SignatureStrength::TextStamp, // BELOW allow-list — this is alice's current commitment
            signature: None,
        };
        let item = item_with(vec![alice_stale_git, alice_current_text]);
        let pred = Predicate::leaf(Leaf::Signers {
            required: vec!["alice".to_string()],
            roles: BTreeMap::new(),
            against: SignerCurrency::Any,
            signature_allow: vec![SignatureStrength::GitTrust, SignatureStrength::CryptoSigned],
            signature_prefer: None,
        });
        let ctx = TestContext::new(sample_date());
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        // CORRECT BEHAVIOR: the allow-list must be satisfied by a CURRENT-fp entry.
        // Alice's current-fp entry is TextStamp (below the allow-list) — the predicate
        // must FAIL regardless of her stale GitTrust entry. The stale entry's higher
        // tier reflects a past commitment, not her current one.
        assert_eq!(
            r.witness_tier,
            WitnessTier::None,
            "ATK-NFA-24: against=Any + signature_allow must enforce strength against current-fp \
             entries only. Alice's current-fp attestation is TextStamp (below [GitTrust, \
             CryptoSigned]); her stale GitTrust entry must NOT satisfy the allow-list. \
             Got {:?}; expected WitnessTier::None. \
             Fix: in eval_signers, gate signature_allow check on current-fp currency.",
            r.witness_tier
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

    // ========================================================================
    // ATK-eval-leaf-not-evaluated: supply-chain leaves in standard evaluator
    //
    // Supply-chain leaves (dep_pinned, dep_attested, etc.) cannot be evaluated
    // by the standard predicate evaluator — they require Cargo.lock + sidecar
    // reading that only `cargo antigen audit --supply-chain` provides.
    //
    // FIXED (option b): `evaluated: bool` field added to `LeafOutcome`; supply-chain
    // leaves set `evaluated=false` so the CLI renders NOT-EVALUATED, not FAIL.
    // Campsite findings/eval-leaf-not-evaluated-arm is closed.
    // ========================================================================

    #[test]
    fn atk_eval_leaf_supply_chain_leaf_is_not_evaluated_not_failed() {
        use crate::predicate::{Leaf, Predicate};
        // dep_pinned() is a supply-chain leaf — cannot be evaluated by standard evaluator.
        let pred = Predicate::Leaf(Leaf::DepPinned { crate_name: None });
        let item = item_with(vec![]);
        let ctx = TestContext::new(sample_date());

        let r = evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx)
            .expect("evaluation must not error");

        // The predicate should report "not evaluated" — not "failed".
        // At the leaf_outcomes level, the supply-chain leaf outcome must be
        // distinguishable from a genuine evaluation failure.
        assert_eq!(r.leaf_outcomes.len(), 1, "one leaf outcome expected");
        let leaf = &r.leaf_outcomes[0];

        // The leaf must have evaluated=false so the CLI can render NOT-EVALUATED
        // rather than FAIL. A passed=false + evaluated=false pair is the honest
        // representation: the check was deferred, not run-and-failed.
        assert!(
            !leaf.evaluated,
            "ATK-eval-leaf-not-evaluated: supply-chain leaf (dep_pinned) must have \
             evaluated=false so the CLI distinguishes 'not evaluated here' from a \
             genuine failure. Current: label={:?}, passed={}, evaluated={}. \
             The `passed: false, evaluated: true` combination implies the check ran \
             and failed, but it never ran — only the supply-chain audit pass can \
             evaluate dep_pinned leaves. \
             Fix: set `evaluated: false` in the supply-chain leaf arm of eval_leaf(). \
             See campsite findings/eval-leaf-not-evaluated-arm.",
            leaf.label, leaf.passed, leaf.evaluated,
        );
    }

    #[test]
    fn composite_verdict_indeterminate_on_supply_chain_leaf() {
        // A pure supply-chain leaf must produce Indeterminate, not Failed.
        // This is the behavioral lock for e58627d5 — the bool-layer dishonesty
        // that collapsed Indeterminate into Failed at the EvaluatedPredicate level.
        use crate::predicate::{Leaf, Predicate};
        let pred = Predicate::Leaf(Leaf::DepPinned { crate_name: None });
        let item = item_with(vec![]);
        let ctx = TestContext::new(sample_date());

        let node = eval_pred(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx);
        assert_eq!(
            node.verdict(),
            CompositeVerdict::Indeterminate,
            "e58627d5 lock: supply-chain leaf must yield Indeterminate, not Failed"
        );
    }

    #[test]
    fn evaluate_predicate_emits_deferred_hint_on_supply_chain_leaf() {
        // evaluate_predicate_with_kind must emit DisciplinePredicateDeferred (not
        // DisciplinePredicateFailed) when the composite verdict is Indeterminate.
        // Behavioral lock for the e58627d5 call-site fix.
        use crate::predicate::{Leaf, Predicate};
        let pred = Predicate::Leaf(Leaf::DepPinned { crate_name: None });
        let item = item_with(vec![]);
        let ctx = TestContext::new(sample_date());

        let r = evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx)
            .expect("evaluation must not error");

        assert_eq!(
            r.audit_hint,
            AuditHint::DisciplinePredicateDeferred,
            "e58627d5 lock: supply-chain-only predicate must emit DisciplinePredicateDeferred, \
             not DisciplinePredicateFailed. Got: {:?}",
            r.audit_hint,
        );
        assert_eq!(
            r.witness_tier,
            WitnessTier::None,
            "deferred predicate must not claim any witness tier"
        );
    }

    #[test]
    fn all_of_mixed_supply_chain_and_substrate_is_indeterminate() {
        // AllOf(dep_pinned, Signers[alice]) with alice in item → Indeterminate.
        // Confirms Indeterminate propagates through AllOf correctly (not short-circuited
        // to Failed, not promoted to Passed).
        use crate::predicate::{Leaf, Predicate, SignerCurrency};

        let item = item_with(vec![alice_fresh(sample_date(), "fp-current")]);
        let pred = Predicate::AllOf {
            children: vec![
                Predicate::Leaf(Leaf::DepPinned { crate_name: None }),
                Predicate::Leaf(Leaf::Signers {
                    required: vec!["alice".to_string()],
                    roles: BTreeMap::new(),
                    against: SignerCurrency::Current,
                    signature_allow: vec![],
                    signature_prefer: None,
                }),
            ],
        };
        let ctx = TestContext::new(sample_date());

        let node = eval_pred(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx);
        assert_eq!(
            node.verdict(),
            CompositeVerdict::Indeterminate,
            "AllOf(supply-chain-deferred, substrate-passed) must be Indeterminate"
        );
    }

    #[test]
    fn ratified_doc_anchor_vs_min_version_affordance_trap_nfa27() {
        // SILENT WRONG-PASS (adversarial NFA-27): `AffordanceTrapInAttestationDSL`.
        //
        // `ratified_doc(path=..., min_version=..., anchor=...)` has three optional
        // `String` slots with distinct semantics: `min_version` does a structured
        // frontmatter version comparison, `anchor` does a plain substring search.
        //
        // An author who means "require version >= 1.0" and writes
        //   `ratified_doc(path = "docs/d.md", anchor = "1.0")`
        // instead of
        //   `ratified_doc(path = "docs/d.md", min_version = "1.0")`
        //
        // gets the WRONG predicate silently accepted by the parser (both are valid
        // keyword arguments with `String` values). At evaluation time, `anchor = "1.0"`
        // performs `content.contains("1.0")`. If the document contains "1.0" anywhere
        // — as a heading, version string, example, or anywhere else — the predicate
        // PASSES, even if the document's structured frontmatter version is 0.5 (below
        // the intended floor). This is a false PASS: the author believed they were
        // enforcing a version floor; the predicate enforces nothing meaningful.
        //
        // This is the strictest form of the affordance trap: not a fail (which would
        // be caught) but a wrong GREEN that confirms "all is well" when it isn't.
        //
        // CONCRETE CASE: doc has frontmatter `version: 0.5` (below intended 1.0 floor)
        // but the body contains "See also v1.0 changelog" — anchor = "1.0" substring
        // matches "v1.0", predicate passes with Execution tier. The min_version check
        // would correctly fail (0.5 < 1.0).
        //
        // FIX DIRECTION (per AffordanceTrapInAttestationDSL R5): newtype-wrapped slots
        // — `MinVersion("1.0")` vs `Anchor("1.0")` — so wrong-slot binding is a
        // type error, not a runtime surprise. Alternatively, a semantic-alias validator
        // that flags `anchor` values matching semver patterns and suggests `min_version`.
        //
        // This test documents the current behavior (false PASS) as a regression anchor.
        // It PASSES under the current code — proving the trap is live. A correct fix
        // would introduce a parse-time or validate-time guard that catches this class
        // of wrong-slot binding.
        let item = item_with(vec![]);
        let pred = Predicate::leaf(Leaf::RatifiedDoc {
            path: Some(PathBuf::from("docs/d.md")),
            // WRONG SLOT: author intends a version floor but uses anchor.
            min_version: None,
            anchor: Some("1.0".to_string()),
            sibling_json: false,
        });
        // Doc has structured version 0.5 (below intended floor 1.0), but body
        // contains "1.0" as a substring in a changelog reference.
        // min_version = "1.0" would FAIL (0.5 < 1.0).
        // anchor = "1.0" PASSES (substring found in body).
        let doc_content = "---\nversion: 0.5\n---\n# Discipline\nSee also v1.0 changelog.";
        let ctx = TestContext::new(sample_date()).with_doc("docs/d.md", doc_content);
        let r =
            evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        // CURRENT BEHAVIOR: predicate passes (Execution) — the anchor "1.0" is found
        // in the body. This is a FALSE PASS: the document version is 0.5, below the
        // intended 1.0 floor. The author believed they were enforcing a version floor
        // but the wrong slot produces a vacuous success.
        //
        // A CORRECT predicate would be `min_version = "1.0"`, which would fail here
        // (doc version 0.5 < required 1.0).
        //
        // NFA-27 DOCUMENTS: wrong-slot binding (anchor vs min_version) silently produces
        // a false-green audit result. The parser must not accept this ambiguity silently.
        assert_eq!(
            r.witness_tier,
            WitnessTier::Execution,
            "NFA-27 affordance trap: anchor = \"1.0\" on a doc with version 0.5 silently \
             passes because \"1.0\" appears in the body text. A correct min_version check \
             would fail (0.5 < 1.0). This documents the live AffordanceTrapInAttestationDSL \
             failure class — wrong-slot binding produces false-green audit result."
        );
    }
}
