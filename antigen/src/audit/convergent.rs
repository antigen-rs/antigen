//! Convergent-Evidence Family audit (ADR-024).
//!
//! Extracted from the former monolithic `audit.rs` per ADR-036 (the scan/audit
//! orchestration decomposition). A pure detector module: a fn of `&ScanReport`
//! returning its own report; no detector calls another (single-conductor
//! invariant, ADR-036). API-invisible: re-exported from the `audit` root via
//! `pub use`.

use serde::{Deserialize, Serialize};

use super::{AuditHint, CLONAL_ITERATIONS_DEFAULT_FLOOR, IGG_HISTORICAL_SPAN_DEFAULT_FLOOR};
use crate::scan::ScanReport;

// ============================================================================
// Convergent-Evidence Family audit (ADR-024)
// ============================================================================

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
                if min == 0 {
                    // A zero threshold is semantically null: it can never fire
                    // DiagnosticModalityInsufficient regardless of how many (or
                    // few) independent classes exist. Surface the misconfiguration
                    // explicitly rather than silently accepting it. (ATK-CE-5)
                    hints.push(AuditHint::DiagnosticMinIndependentZero);
                } else if u64::try_from(distinct.len()).unwrap_or(u64::MAX) < min {
                    hints.push(AuditHint::DiagnosticModalityInsufficient);
                }
            }
        },
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
        },
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
                if min_re == 0 {
                    // Zero reattestations required is a null threshold — it can
                    // never fire IggReattestationsInsufficient. Surface the
                    // misconfiguration explicitly. (ATK-CE-5)
                    hints.push(AuditHint::IggMinReattestationsZero);
                } else if u64::try_from(unique_count.len()).unwrap_or(u64::MAX) < min_re {
                    hints.push(AuditHint::IggReattestationsInsufficient);
                }
            }
            // Identity-collapse: best-effort at scan time — if the
            // recorded witnesses all collapse to one identity, surface
            // the warning. Real signer-identity tracking is v0.3+.
            if decl.witnesses.len() > 1 && unique_count.len() == 1 {
                hints.push(AuditHint::IggIdentityCollapseWarning);
            }
        },
        ConvergentEvidenceKind::Crossreactive => {
            for fp in &decl.fingerprints {
                if !known_antigen_names.contains(fp.as_str()) {
                    hints.push(AuditHint::CrossreactiveFingerprintUnresolved);
                    break;
                }
            }
        },
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
        },
    }
    hints
}
