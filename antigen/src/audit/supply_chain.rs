//! Supply-Chain Defense Family audit (ADR-025).
//!
//! Extracted from the former monolithic `audit.rs` per ADR-036 (the scan/audit
//! orchestration decomposition). A pure detector module: a fn of `&ScanReport`
//! (plus the workspace root, for sidecar/manifest reads) returning its own
//! report; no detector calls another (single-conductor invariant, ADR-036).
//! API-invisible: re-exported from the `audit` root via `pub use`.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::AuditHint;
use crate::scan::ScanReport;

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

        // ATK-SC-7: serde's derived Deserialize does NOT call Predicate::validate(),
        // so a hand-crafted JSON like `{"kind":"all_of","children":[]}` bypasses the
        // ZeroLeafComposition guard and evaluates vacuously to passed=true with no
        // leaves. Validate after deserialization and emit a diagnostic rather than
        // silently proceeding to a vacuous evaluation.
        if predicate.validate().is_err() {
            audits.push(SupplyChainAudit {
                antigen_type: immunity.antigen_type.clone(),
                hint: AuditHint::MalformedRequiresPredicate,
                file: immunity.file.clone(),
                line: immunity.line,
                crate_name: String::new(),
                version: String::new(),
                detail: Some(
                    "requires_predicate deserialized but failed structural validation \
                     (e.g., empty combinator — vacuous pass with no leaves)"
                        .to_string(),
                ),
            });
            continue;
        }

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
        },
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
        },
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
        },
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
        },
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
        },
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
        },
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
