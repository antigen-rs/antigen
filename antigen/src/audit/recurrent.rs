//! Recurrent-Emergence audit (ADR-024 §Family 2).
//!
//! Extracted from the former monolithic `audit.rs` per ADR-036 (the scan/audit
//! orchestration decomposition). A pure detector module: a fn of `&ScanReport`
//! returning its own report; no detector calls another (single-conductor
//! invariant, ADR-036). API-invisible: re-exported from the `audit` root via
//! `pub use`. `is_version_tag` is a helper the audit test module exercises
//! directly; the `audit` root re-exports it under `#[cfg(test)]` only (test
//! reach, not public API) so the test module's `use super::*` keeps resolving
//! it after the move.

use antigen_macros::presents;
use serde::{Deserialize, Serialize};

use super::AuditHint;
use crate::scan::ScanReport;

// ============================================================================
// Recurrent-Emergence audit (ADR-024 §Family 2)
// ============================================================================

/// Per-declaration recurrent-emergence audit result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurrentAudit {
    /// The original declaration from the scan.
    pub declaration: crate::scan::RecurrentDeclaration,
    /// Hints surfaced for this declaration (may be empty = clean).
    pub hints: Vec<AuditHint>,
}

/// Aggregate recurrent-emergence audit report.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecurrentAuditReport {
    /// Per-declaration audit results.
    pub audits: Vec<RecurrentAudit>,
    /// Count of declarations whose hint set is empty (clean).
    pub clean_count: usize,
    /// Count of declarations whose hint set is non-empty (concerns surfaced).
    pub concern_count: usize,
}

impl RecurrentAuditReport {
    /// True when no concerns were surfaced.
    #[must_use]
    pub const fn all_clean(&self) -> bool {
        self.concern_count == 0
    }
}

/// Default review horizon (days) past which a `#[chronic]` `since` date
/// surfaces `chronic-signal-past-review-date`. One year — chronic states
/// older than this warrant explicit re-review per ADR-024 recurrent
/// discipline. Configurable via workspace config in v0.3+.
const CHRONIC_REVIEW_HORIZON_DAYS: i64 = 365;

/// Audit recurrent-emergence declarations across a scan report (ADR-024
/// §Family 2). Walks every [`crate::scan::RecurrentDeclaration`] and
/// surfaces the relevant hints per the recurrent audit taxonomy.
#[must_use]
pub fn audit_recurrent(report: &ScanReport) -> RecurrentAuditReport {
    // Antigen-type names that have downstream action (an #[immune] or
    // #[presents] referencing them). Used for the recurrence-anchor
    // threshold-reached-no-action check.
    let acted_on: std::collections::HashSet<&str> = report
        .immunities
        .iter()
        .map(|i| i.antigen_type.as_str())
        .chain(report.presentations.iter().map(|p| p.antigen_type.as_str()))
        .collect();

    // Antigen-type names declared by #[itch] entries. Used to check that
    // #[recurrence_anchor] has upstream noticing preconditions (ATK-RECURRENT-2).
    let itch_antigen_types: std::collections::HashSet<&str> = report
        .recurrent_declarations
        .iter()
        .filter(|d| d.kind == crate::scan::RecurrentKind::Itch)
        .filter_map(|d| d.antigen_type.as_deref())
        .collect();

    // Direct child→parent edges (bare names), the substrate for the
    // lineage-aware from_itches check (ADR-024 Amendment 3). The
    // noticing-precondition is class-specific BUT lineage-aware: noticing an
    // ANCESTOR class is legitimate upstream evidence for committing to track a
    // descendant (inheritance is provenance — ADR-018 Amd1; parent-recurrence
    // is evidence the lineage recurs). Built once here; `ancestors_of` walks it
    // transitively per anchor.
    let parent_of: std::collections::HashMap<&str, Vec<&str>> = {
        let mut m: std::collections::HashMap<&str, Vec<&str>> = std::collections::HashMap::new();
        for e in &report.lineage_edges {
            m.entry(e.child.as_str())
                .or_default()
                .push(e.parent.as_str());
        }
        m
    };

    let mut audits: Vec<RecurrentAudit> = Vec::new();
    for decl in &report.recurrent_declarations {
        let hints = evaluate_recurrent_hints(decl, &acted_on, &itch_antigen_types, &parent_of);
        audits.push(RecurrentAudit {
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

    RecurrentAuditReport {
        audits,
        clean_count,
        concern_count,
    }
}

/// Heuristic: is `s` a recognizable version tag (vs. a calendar date or
/// garbage)? Per ATK-RECURRENT-4a the chronic `since` field tolerates
/// version anchors but flags unparseable strings. A version tag is an
/// optional leading `v`/`V` followed by at least one dot-separated numeric
/// component (e.g. `v0.2.0`, `1.4`, `2.0.0-rc.1`). Pre-release/build
/// suffixes after the numeric core are allowed.
pub fn is_version_tag(s: &str) -> bool {
    let had_v_prefix = s.starts_with(['v', 'V']);
    let core = if had_v_prefix { &s[1..] } else { s };
    // The numeric core runs until the first `-`/`+` (where a pre-release or
    // build suffix like `-rc.1` or `+build` begins).
    let numeric_core: &str = core.split(['-', '+']).next().unwrap_or("");
    if numeric_core.is_empty() {
        return false;
    }
    // Every dot-separated component of the numeric core must be all digits.
    let mut component_count = 0usize;
    for part in numeric_core.split('.') {
        if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
            return false;
        }
        component_count += 1;
    }
    // A version tag is either `v`-prefixed (e.g. `v3`, `v0.2.0`) OR has at
    // least major.minor structure (≥2 dot-separated numeric components).
    // A bare single integer like `"3"` is ambiguous garbage, not a version.
    had_v_prefix || component_count >= 2
}

/// Transitive ancestor set of `antigen` over the `child → parent` lineage map
/// (ADR-024 Amendment 3 lineage-aware `from_itches`). Walks every
/// `#[descended_from]` chain upward, cycle-guarded (a malformed cyclic lineage
/// must not loop here — cycle detection is the scanner's job, but this walk is
/// defensively bounded by the `visited` set). The anchor's OWN type is NOT
/// included (the caller checks self-match separately); only strict ancestors.
fn ancestors_of<'a>(
    antigen: &'a str,
    parent_of: &std::collections::HashMap<&'a str, Vec<&'a str>>,
) -> std::collections::HashSet<&'a str> {
    let mut acc: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut stack: Vec<&str> = parent_of.get(antigen).cloned().unwrap_or_default();
    while let Some(parent) = stack.pop() {
        if acc.insert(parent) {
            if let Some(grandparents) = parent_of.get(parent) {
                stack.extend(grandparents.iter().copied());
            }
        }
    }
    acc
}

/// Evaluate hints for a single recurrent declaration.
///
/// **ATK-RECURRENT-2 fix (dd51d4b)**: this function now checks BOTH the
/// upstream precondition (`itch_antigen_types` contains anchor's antigen type)
/// AND the downstream action (`acted_on` contains the antigen type). See
/// [`crate::stdlib::dogfood::AuditHintWithNoUpstreamPreconditionCheck`].
///
/// **ADR-024 Amendment 3 (class-specific, lineage-aware `from_itches`)**: a
/// `from_itches` entry satisfies the noticing-precondition iff it names the
/// anchor's OWN `antigen_type` (or a lineage ANCESTOR of it) AND that class has
/// a scan-resident `#[itch]`. A pure cross-class reference (an unrelated class,
/// even one with its own itch) carries ZERO precondition-evidence for this
/// anchor — noticing `AntigenY` tells you nothing about whether `AntigenX`
/// recurred. The prior global membership test silently widened the precondition
/// to "does the workspace contain any itch at all", the vacuous-guard shape;
/// this realigns the impl with the audit-hint doc's already-stated intent
/// ("the same antigen type" — a `RatifiedSpecDriftFromImpl` fix, not a new
/// design choice). The lineage exception is the one legitimate
/// "cross-class" case and is intra-lineage, not cross-class: inheritance is
/// provenance (ADR-018 Amd1), so parent-recurrence is evidence the descended
/// lineage recurs.
// ADR-029 migration: this fn `#[presents]` AuditHintWithNoUpstreamPreconditionCheck
// (it once emitted the hint without checking the upstream precondition). The
// integration test `atk_recurrent_2_recurrence_anchor_without_matching_itch_emits_hint`
// (tests/atk_recurrent_adversarial.rs) declares it defends the class via
// `#[defended_by]`; the audit cross-references and observes the verdict.
#[presents(AuditHintWithNoUpstreamPreconditionCheck)]
fn evaluate_recurrent_hints(
    decl: &crate::scan::RecurrentDeclaration,
    acted_on: &std::collections::HashSet<&str>,
    itch_antigen_types: &std::collections::HashSet<&str>,
    parent_of: &std::collections::HashMap<&str, Vec<&str>>,
) -> Vec<AuditHint> {
    use crate::scan::RecurrentKind;

    let mut hints = Vec::new();
    match decl.kind {
        RecurrentKind::Itch => {
            if decl.antigen_type.is_none() {
                hints.push(AuditHint::ItchNoticedNotAnchored);
            }
        },
        RecurrentKind::RecurrenceAnchor => {
            // Anchor has no upstream itch preconditions — temporal progression
            // (itch → anchor → crystallize) bypassed (ATK-RECURRENT-2).
            //
            // Two bypass vectors (ATK-RECURRENT-7 adds the second):
            //   (a) from_itches is empty AND the anchor's antigen type has no
            //       corresponding #[itch] in the scan — temporal progression skipped.
            //   (b) from_itches is non-empty but ALL listed itches are phantom
            //       references — they name itch types that have no #[itch] declaration
            //       in the scan. A non-empty phantom list bypassed the is_empty() guard
            //       while providing zero real precondition evidence. We now validate
            //       that from_itches entries actually resolve to scan-resident itches.
            if let Some(antigen) = decl.antigen_type.as_deref() {
                // ADR-024 Amendment 3: a from_itches entry is valid ONLY when it
                // names the anchor's own class OR a lineage ancestor of it, AND
                // that class has a scan-resident #[itch]. A pure cross-class
                // reference is a phantom for THIS anchor (no precondition
                // evidence), exactly as ATK-RECURRENT-7 treats non-scan-resident
                // phantoms — the class-scoped test below subsumes the old global
                // membership test (an entry must still be itch-resident, but now
                // it must additionally be in-lineage).
                let ancestors = ancestors_of(antigen, parent_of);
                let in_lineage = |itch: &str| itch == antigen || ancestors.contains(itch);
                let has_valid_from_itches = !decl.from_itches.is_empty()
                    && decl.from_itches.iter().any(|itch| {
                        in_lineage(itch.as_str()) && itch_antigen_types.contains(itch.as_str())
                    });
                let has_implicit_itch = itch_antigen_types.contains(antigen);

                if !has_valid_from_itches && !has_implicit_itch {
                    // No real itch precondition: either no from_itches + no implicit
                    // itch, OR from_itches is entirely phantom references that don't
                    // resolve to any scan-resident #[itch] declaration. (ATK-RECURRENT-7)
                    hints.push(AuditHint::RecurrenceAnchorNoItchPrecondition);
                }
                // Anchor crossed threshold but nothing downstream addresses it.
                if !acted_on.contains(antigen) {
                    hints.push(AuditHint::RecurrenceThresholdReachedNoAction);
                }
            }
        },
        RecurrentKind::Crystallize => {
            // A crystallization with neither a formal antigen NOR source
            // itches crystallized nothing into anything.
            if decl.antigen_type.is_none() && decl.from_itches.is_empty() {
                hints.push(AuditHint::CrystallizeWithoutAntigen);
            }
        },
        RecurrentKind::Chronic => {
            if decl.managed_by.is_none() {
                hints.push(AuditHint::ChronicSignalUnmanaged);
            }
            // Three-path `since` resolution per ATK-RECURRENT-4a:
            //   (1) ISO-8601 date → enforce the review horizon
            //       (past-horizon → past-review-date hint).
            //   (2) version tag (e.g. "v0.2.0", "1.4.3-rc.1") → tolerate;
            //       the chronic state is anchored to a release, not a
            //       calendar, so no date check applies.
            //   (3) neither → `since` is a malformed anchor; emit
            //       chronic-since-not-a-date.
            if let Some(since) = decl.since.as_deref() {
                if let Ok(since_date) = chrono::NaiveDate::parse_from_str(since, "%Y-%m-%d") {
                    let age = chrono::Utc::now().date_naive() - since_date;
                    if age.num_days() > CHRONIC_REVIEW_HORIZON_DAYS {
                        hints.push(AuditHint::ChronicSignalPastReviewDate);
                    }
                } else if !is_version_tag(since) {
                    hints.push(AuditHint::ChronicSinceNotADate);
                }
            }
        },
        RecurrentKind::Saturate => {
            if decl.contributing_to.is_none() {
                hints.push(AuditHint::SaturateNoAnchor);
            }
        },
        RecurrentKind::Strand => {
            if decl.anchored_by.is_empty() {
                hints.push(AuditHint::StrandNoAnchors);
            }
        },
    }
    hints
}
