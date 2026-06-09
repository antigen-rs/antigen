//! Deferred-Defense Family audit (ADR-023).
//!
//! Extracted from the former monolithic `audit.rs` per ADR-036 (the scan/audit
//! orchestration decomposition). A pure detector module: a fn of `&ScanReport`
//! (plus a stale-grace-days knob) returning its own report; no detector calls
//! another (single-conductor invariant, ADR-036). API-invisible: re-exported
//! from the `audit` root via `pub use`.

use serde::{Deserialize, Serialize};

use super::AuditHint;

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
            },
            AuditHint::AnergyCostimulationNotArrived
            | AuditHint::ImmunosuppressExpired
            | AuditHint::PoxpartyOutcomePending
            | AuditHint::OrientPendingActionRequired => {
                expired_count += 1;
            },
            AuditHint::AnergyStale | AuditHint::ImmunosuppressDurationCapExceeded => {
                // Cap-exceeded is the most-overstayed immunosuppress state — it
                // outlived its own declared hard cap; classify as stale (escalate)
                // alongside anergy-stale, not merely expired.
                stale_count += 1;
            },
            _ => {},
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
            // `#[anergy]` REQUIRES `until` (ADR-023: anergy without a time-bound
            // degrades to silent tolerance). Mirror the Orient arm: match on the
            // PRESENCE of `until` first so a present-but-malformed date does NOT
            // collapse into the absent-date grace path. Previously `unwrap_or("")`
            // made `until=None` and `until=Some("not-a-date")` indistinguishable —
            // both parsed to `None` and fell to AnergyActive, so a typo'd deadline
            // silently granted permanent active status with zero diagnostic.
            match decl.until.as_deref() {
                // Absent `until`: only arises for hand-built/legacy scan records
                // (the macro parse-gate requires a valid `until`). Legacy grace path.
                None | Some("") => AuditHint::AnergyActive,
                Some(s) => match parse_iso_date(s) {
                    Some(until) if until >= today => AuditHint::AnergyActive,
                    // Genuine past date → tally by staleness against the grace window.
                    Some(until) => {
                        let days_past = (today - until).num_days();
                        if days_past > stale_grace_days {
                            AuditHint::AnergyStale
                        } else {
                            AuditHint::AnergyCostimulationNotArrived
                        }
                    },
                    // Present-but-unparseable (typo like "2026-13-99", "soon"): an
                    // INTENDED deadline that resolves to nothing → unresolved
                    // co-stimulation, not a grace. Escalate (not silently active).
                    None => AuditHint::AnergyCostimulationNotArrived,
                },
            }
        },
        DeferredDefenseKind::Immunosuppress => {
            // Duration-cap enforcement (ADR-023): `#[immunosuppress(since = D,
            // duration_cap = N)]` is capped at N days from D. Once `since + cap`
            // is in the past, the suppression has overstayed its hard cap —
            // emit ImmunosuppressDurationCapExceeded. This is checked FIRST
            // because an exceeded cap is the loudest state (a suppression that
            // outlived its own declared limit). Previously `since`/`duration_cap`
            // lived only as unparsed `see[]` string tags, so this hint had zero
            // emission sites; they are now typed fields the audit can read.
            if let (Some(since), Some(cap)) = (decl.since.as_deref(), decl.duration_cap) {
                if let Some(since_date) = parse_iso_date(since) {
                    let elapsed = (today - since_date).num_days();
                    if let Ok(cap_days) = i64::try_from(cap) {
                        if elapsed > cap_days {
                            return AuditHint::ImmunosuppressDurationCapExceeded;
                        }
                    }
                }
            }
            // `until` is REQUIRED (ADR-023). Same Orient-style split: present-but-
            // malformed must escalate, not silently stay Active. Previously a typo
            // like "2026/01/01" (slash format, looks like a past date to a human but
            // fails ISO parse) collapsed via `unwrap_or("")` to ImmunosuppressActive.
            match decl.until.as_deref() {
                // Absent `until`: hand-built/legacy only (macro requires it). Grace.
                None | Some("") => AuditHint::ImmunosuppressActive,
                Some(s) => match parse_iso_date(s) {
                    Some(until) if until >= today => AuditHint::ImmunosuppressActive,
                    // Past date OR present-but-unparseable: the suppression's declared
                    // expiry arrived (or was a typo that resolves to nothing). Either
                    // way it has outlived its intended bound → Expired, not Active.
                    _ => AuditHint::ImmunosuppressExpired,
                },
            }
        },
        DeferredDefenseKind::Poxparty => {
            // `until` is REQUIRED (ADR-023). Same Orient-style split: present-but-
            // malformed must escalate, not silently stay Active. Previously a typo
            // like "soon" collapsed via `unwrap_or("")` to PoxpartyActive forever.
            match decl.until.as_deref() {
                // Absent `until`: hand-built/legacy only (macro requires it). Grace.
                None | Some("") => AuditHint::PoxpartyActive,
                Some(s) => match parse_iso_date(s) {
                    Some(until) if until >= today => AuditHint::PoxpartyActive,
                    // Past date OR present-but-unparseable: the isolation window's
                    // declared horizon arrived (or was a typo) → the outcome is due
                    // for recording, not silently active. Escalate.
                    _ => AuditHint::PoxpartyOutcomePending,
                },
            }
        },
        DeferredDefenseKind::Orient => {
            // ADR-023: `#[orient]` REQUIRES `until` (the orientation-period
            // horizon). The audit OBSERVES that date: once it has passed, the
            // orientation period elapsed without the failure-class being
            // resolved — surface OrientPendingActionRequired loudly rather than
            // perpetually reporting OrientActive (a deferred defense whose
            // deadline arrived must escalate, not stay silently green).
            // Match on the PRESENCE of `until` first, so a present-but-malformed
            // date does NOT collapse into the absent-date grace path. Previously
            // `unwrap_or("")` made `until=None` and `until=Some("not-a-date")`
            // indistinguishable — both fell to OrientActive, so a typo'd deadline
            // silently granted permanent green with zero diagnostic.
            match decl.until.as_deref() {
                // Absent `until`: only arises for hand-built/legacy scan records
                // (the macro parse-gate requires a valid `until`). Legacy grace
                // path — don't fabricate an escalation.
                None | Some("") => AuditHint::OrientActive,
                // Future deadline → still active. Everything else for a PRESENT
                // `until` escalates: a past date is an elapsed orientation, and a
                // present-but-unparseable date (typo like "2026-13-99", "2099/01/01",
                // or "soon") is an INTENDED-but-broken deadline that resolves to
                // nothing. Both are unresolved orientations needing action — not a
                // grace. (Only an ABSENT until takes the legacy grace path above.)
                Some(s) => match parse_iso_date(s) {
                    Some(until) if until >= today => AuditHint::OrientActive,
                    _ => AuditHint::OrientPendingActionRequired,
                },
            }
        },
    }
}

/// Parse an ISO-8601 date string for audit-time UTC comparison.
/// Returns `None` if the string is not a valid date.
fn parse_iso_date(s: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}
