//! Mucosal Boundary audit (ADR-027 + Amendment 1).
//!
//! Extracted from the former monolithic `audit.rs` per ADR-036 (the scan/audit
//! orchestration decomposition). A pure detector module: a fn of `&ScanReport`
//! returning its own report; no detector calls another (single-conductor
//! invariant, ADR-036). API-invisible: re-exported from the `audit` root via
//! `pub use`.

use antigen_macros::{antigen_tolerance, presents};
use serde::{Deserialize, Serialize};

use super::AuditHint;
use crate::scan::ScanReport;

// ============================================================================
// Mucosal Boundary audit (ADR-027 + Amendment 1)
// ============================================================================

/// Per-declaration mucosal-boundary audit result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MucosalAudit {
    /// The original declaration from the scan.
    pub declaration: crate::scan::MucosalDeclaration,
    /// Hints surfaced for this declaration (may be empty = clean).
    pub hints: Vec<AuditHint>,
}

/// Aggregate mucosal-boundary audit report.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MucosalAuditReport {
    /// Per-declaration audit results.
    pub audits: Vec<MucosalAudit>,
    /// Count of declarations whose hint set is empty (clean).
    pub clean_count: usize,
    /// Count of declarations whose hint set is non-empty.
    pub concern_count: usize,
}

impl MucosalAuditReport {
    /// True when no concerns were surfaced.
    #[must_use]
    pub const fn all_clean(&self) -> bool {
        self.concern_count == 0
    }
}

/// Minimum rationale lengths per ADR-027 + Amendment 1 Change 6 (risk-
/// proportionate: tolerance is riskier than defense, so its floor is higher).
const MUCOSAL_RATIONALE_FLOOR: usize = 20;
const MUCOSAL_TOLERANT_RATIONALE_FLOOR: usize = 40;

/// Audit mucosal-boundary declarations across a scan report (ADR-027 +
/// Amendment 1).
///
/// Implements the Change-5 three-tier delegate diagnosis via set-membership
/// kind-matching against the handler functions' `#[mucosal]` declarations.
///
/// **Residual risk**: `handler_kinds` is built from intra-crate `#[mucosal]`
/// declarations only. Cross-crate handlers are not in the index; delegates
/// pointing at them will false-positive as `MucosalDisciplineDelegateTargetMissing`.
/// See [`crate::stdlib::agentic_coordination::DelegateCrossCrateResolutionGap`].
/// Structural fix is v0.3+ scope (multi-crate scan pass).
#[must_use]
#[presents(DelegateCrossCrateResolutionGap)]
#[antigen_tolerance(
    DelegateCrossCrateResolutionGap,
    rationale = "Accepted v0.2 limitation: handler_kinds is built from intra-crate #[mucosal] \
                 declarations only, so a #[mucosal_delegate] pointing at a cross-crate handler \
                 false-positives as MucosalDisciplineDelegateTargetMissing. The structural fix is a \
                 multi-crate scan pass (v0.3+ scope, same boundary as --include-deps cross-crate \
                 addressing). Until then the false-positive is the conservative failure (flags rather \
                 than silently trusts an unresolvable delegate).",
    until = "v0.3"
)]
pub fn audit_mucosal(report: &ScanReport) -> MucosalAuditReport {
    use crate::scan::{ItemTarget, MucosalKindTag};

    // Build handler-function → set-of-mucosal-kinds index from every
    // `#[mucosal]` declaration sitting on a function. The delegate
    // kind-matching (Change 5c) is set-membership against this index;
    // hybrid handlers (multiple `#[mucosal(kind = X)]`) contribute multiple
    // kinds to their function's set.
    let mut handler_kinds: std::collections::HashMap<&str, std::collections::HashSet<&str>> =
        std::collections::HashMap::new();
    // Track which source files each bare function name appears in, so we can detect
    // same-name ambiguity (findings/mucosal-same-name-fn-collision). A bare fn name
    // that appears in more than one file is ambiguous: delegates pointing to it by
    // bare name alone cannot unambiguously identify the target.
    let mut handler_files: std::collections::HashMap<
        &str,
        std::collections::HashSet<&std::path::Path>,
    > = std::collections::HashMap::new();
    for decl in &report.mucosal_declarations {
        if decl.tag == MucosalKindTag::Mucosal {
            if let ItemTarget::Fn(fn_name) = &decl.item_target {
                if let Some(kind) = decl.boundary_kind.as_deref() {
                    handler_kinds
                        .entry(fn_name.as_str())
                        .or_default()
                        .insert(kind);
                }
                handler_files
                    .entry(fn_name.as_str())
                    .or_default()
                    .insert(decl.file.as_path());
            }
        }
    }
    // A name is ambiguous when it maps to 2+ distinct source files.
    let ambiguous_names: std::collections::HashSet<&str> = handler_files
        .iter()
        .filter(|(_, files)| files.len() > 1)
        .map(|(name, _)| *name)
        .collect();

    let mut audits: Vec<MucosalAudit> = Vec::new();
    for decl in &report.mucosal_declarations {
        let hints = evaluate_mucosal_hints(decl, &handler_kinds, &ambiguous_names);
        audits.push(MucosalAudit {
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

    MucosalAuditReport {
        audits,
        clean_count,
        concern_count,
    }
}

fn evaluate_mucosal_hints(
    decl: &crate::scan::MucosalDeclaration,
    handler_kinds: &std::collections::HashMap<&str, std::collections::HashSet<&str>>,
    ambiguous_names: &std::collections::HashSet<&str>,
) -> Vec<AuditHint> {
    use crate::scan::MucosalKindTag;

    let mut hints = Vec::new();
    match decl.tag {
        MucosalKindTag::Mucosal => {
            if decl.boundary_kind.is_none() {
                hints.push(AuditHint::MucosalKindMismatch);
            }
            if decl
                .rationale
                .as_deref()
                .is_none_or(|r| r.len() < MUCOSAL_RATIONALE_FLOOR)
            {
                hints.push(AuditHint::MucosalRationaleInsufficient);
            }
        }
        MucosalKindTag::MucosalDelegate => {
            if decl.boundary_kind.is_none() {
                hints.push(AuditHint::MucosalKindMismatch);
            }
            // Change 5 three-tier diagnosis on the delegate handler.
            match decl.handled_by.as_deref() {
                None => hints.push(AuditHint::MucosalDisciplineDelegateTargetMissing),
                Some(handler) => {
                    // Ambiguity check (findings/mucosal-same-name-fn-collision):
                    // If the bare handler name matches multiple source files, the
                    // delegate is ambiguous and must be flagged BEFORE attempting
                    // kind-resolution — the merged kind-set union would silently
                    // grant the wrong file's kinds. Only emitted for resolved-but-
                    // ambiguous names (handler in ambiguous_names), NOT for missing
                    // targets (Tier 1 below catches those).
                    if ambiguous_names.contains(handler) {
                        hints.push(AuditHint::MucosalDisciplineDelegateTargetAmbiguous);
                    } else {
                        match handler_kinds.get(handler) {
                            // Tier 1: handler doesn't resolve to any #[mucosal]-fn.
                            None => hints.push(AuditHint::MucosalDisciplineDelegateTargetMissing),
                            Some(kinds) if kinds.is_empty() => {
                                // Tier 2: resolves but carries no mucosal kind.
                                hints.push(AuditHint::MucosalDisciplineDelegateTargetNotMucosal);
                            }
                            Some(kinds) => {
                                // Tier 3: set-membership kind match (NOT exact-equality).
                                let matches = decl
                                    .boundary_kind
                                    .as_deref()
                                    .is_some_and(|b| kinds.contains(b));
                                if !matches {
                                    hints.push(
                                        AuditHint::MucosalDisciplineDelegateTargetKindMismatch,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        MucosalKindTag::MucosalTolerant => {
            if decl.boundary_kind.is_none() {
                hints.push(AuditHint::MucosalKindMismatch);
            }
            if decl
                .rationale
                .as_deref()
                .is_none_or(|r| r.len() < MUCOSAL_TOLERANT_RATIONALE_FLOOR)
            {
                hints.push(AuditHint::MucosalTolerantRationaleInsufficient);
            }
            if decl.accepts.as_deref().is_none_or(|a| a.trim().is_empty()) {
                hints.push(AuditHint::MucosalTolerantAcceptsEmpty);
            }
            if decl.reviewed_by.is_none() {
                hints.push(AuditHint::MucosalTolerantWithoutReviewer);
            }
            // Past-review-date: only when `until` parses as an ISO date.
            if let Some(until) = decl.until.as_deref() {
                if let Ok(until_date) = chrono::NaiveDate::parse_from_str(until, "%Y-%m-%d") {
                    if chrono::Utc::now().date_naive() > until_date {
                        hints.push(AuditHint::MucosalTolerantPastReviewDate);
                    }
                }
            }
        }
    }
    hints
}
