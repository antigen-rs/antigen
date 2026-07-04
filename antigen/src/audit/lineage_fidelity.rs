//! Lineage-fidelity audit (`DescendedFromFingerprintDivergence`) — ADVISORY.
//!
//! Extracted from the former monolithic `audit.rs` per ADR-036 (the scan/audit
//! orchestration decomposition). One pure detector module: a fn of `&ScanReport`
//! returning its own report. No detector calls another; stop-authority stays
//! with the orchestrator (the single-conductor invariant, ADR-036 §The
//! out-of-band invariant). API-invisible: re-exported from the `audit` root via
//! `pub use`.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::AuditHint;
use crate::scan::ScanReport;

// ============================================================================
// Lineage-fidelity audit (DescendedFromFingerprintDivergence) — ADVISORY
//
// scientist severity ruling 2026-05-27: advisory for v0.3, hard-fail deferred to
// a future ADR. For each `#[descended_from(Parent)]` edge, check whether the
// CHILD antigen's structural fingerprint *refines* the PARENT's (child.matches ⊆
// parent.matches — the child is at-least-as-specific). Emits an advisory hint on
// the conservative, statically-decidable NON-refinement cases only (no false
// positives). Biology: MHC restriction / negative selection.
// ============================================================================

/// One lineage edge's fidelity verdict (advisory).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageFidelityAudit {
    /// Child antigen type name (bears `#[descended_from]`).
    pub child: String,
    /// Parent antigen type name (the `#[descended_from]` argument).
    pub parent: String,
    /// Source file of the `#[descended_from]` edge.
    pub file: PathBuf,
    /// Line of the `#[descended_from]` edge.
    pub line: usize,
    /// Advisory hint — `DescendedFromFingerprintDivergence` when the child's
    /// fingerprint is detectably NOT a refinement of the parent's; the
    /// human-readable reason is in `detail`.
    pub hint: AuditHint,
    /// Why the divergence was detected (e.g. "child item-kind `enum` differs
    /// from parent `struct`").
    pub detail: String,
}

/// Aggregate lineage-fidelity audit report.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LineageFidelityAuditReport {
    /// Edges whose child fingerprint is detectably not a refinement (advisory).
    pub divergences: Vec<LineageFidelityAudit>,
}

/// Audit `#[descended_from]` lineage fidelity (ADVISORY, scientist 2026-05-27).
///
/// Only flags edges where BOTH endpoints have a parseable fingerprint AND the
/// child is detectably NOT a refinement of the parent. Edges where either
/// fingerprint is absent (verify-only antigens, ADR-009 Amendment 1) or
/// unparseable are skipped — the refinement question is undefined there, and an
/// advisory must not produce false positives. Orphaned/dangling edges (missing
/// parent/child declaration) are a separate concern (`orphaned_lineage_edges` /
/// `dangling_child_lineage_edges`) — they are not re-flagged here.
#[must_use]
pub fn audit_lineage_fidelity(report: &ScanReport) -> LineageFidelityAuditReport {
    use std::collections::HashMap;

    // Index declarations by (type_name, canonical_path) → parsed fingerprint
    // (skip absent/unparseable). Keying by the bare type_name alone collides
    // cross-crate: two antigens named "Foo" from different crates would let
    // `.collect()` silently keep an arbitrary one, so a lineage lookup could
    // resolve to the WRONG crate's "Foo" and fire a non-deterministic spurious
    // divergence (ATK-LF-3). The full antigen identity is (type_name,
    // canonical_path) per ADR-017 — same cross-crate discipline as the G2
    // immunity/defense loops (ATK-G2-24). Intra-workspace declarations carry
    // `canonical_path = None`, so `(name, None)` keys match the `None`-pathed
    // lineage edges they pair with (backward-compat).
    let fingerprints: HashMap<(&str, Option<&str>), antigen_fingerprint::Fingerprint> = report
        .antigens
        .iter()
        .filter_map(|a| {
            let raw = a.fingerprint.as_deref()?;
            antigen_fingerprint::Fingerprint::parse(raw)
                .ok()
                .map(|fp| ((a.type_name.as_str(), a.canonical_path.as_deref()), fp))
        })
        .collect();

    let mut divergences = Vec::new();
    for edge in &report.lineage_edges {
        // Resolve each endpoint by its OWN canonical path (the edge carries
        // child/parent canonical paths independently — a cross-crate child can
        // descend from an intra-workspace parent or vice-versa).
        let (Some(child_fp), Some(parent_fp)) = (
            fingerprints.get(&(edge.child.as_str(), edge.child_canonical_path.as_deref())),
            fingerprints.get(&(edge.parent.as_str(), edge.parent_canonical_path.as_deref())),
        ) else {
            // One or both endpoints lack a parseable fingerprint — refinement is
            // undefined; advisory stays silent (no false positive).
            continue;
        };
        if let Some(detail) = fingerprint_nonrefinement_reason(child_fp, parent_fp) {
            divergences.push(LineageFidelityAudit {
                child: edge.child.clone(),
                parent: edge.parent.clone(),
                file: edge.file.clone(),
                line: edge.line,
                hint: AuditHint::DescendedFromFingerprintDivergence,
                detail,
            });
        }
    }
    LineageFidelityAuditReport { divergences }
}

/// Conservative, statically-decidable NON-refinement detector (scientist
/// refinement note 2026-05-27). Returns `Some(reason)` when the child is
/// detectably NOT a refinement of the parent, `None` otherwise (including all
/// undecidable cases — the advisory errs toward silence, never a false positive).
///
/// A child fingerprint refines its parent's when `child.matches ⊆ parent.matches`.
/// We detect two unambiguous violations of that on the TOP-LEVEL constraints:
/// - **item-kind**: the parent pins `item = <K>` and the child pins a DIFFERENT
///   `item = <K'>` → disjoint match sets → not a refinement.
/// - **`doc_contains`**: the parent requires `doc_contains(s)` but NO child
///   `doc_contains` substring contains `s` → an item can match the child without
///   matching the parent → not a refinement.
///
/// `name = matches(glob)` containment is deferred (the harder case scout/scientist
/// flagged); glob-subset is not attempted here, so a glob mismatch is NOT flagged
/// (silence, not a false positive). Nested combinators (`all_of` / `any_of` /
/// `not`) are not descended into for v0.3 — only top-level constraints compared.
fn fingerprint_nonrefinement_reason(
    child: &antigen_fingerprint::Fingerprint,
    parent: &antigen_fingerprint::Fingerprint,
) -> Option<String> {
    // (1) item-kind divergence: parent pins one kind, child pins a different one
    //     OR child has NO item-kind constraint while parent has a definite one.
    //
    // Delegate to `Fingerprint::node_kind()` (antigen_fingerprint/src/lib.rs:383),
    // which descends into `AllOf` via `Constraint::node_kind_hint` — so a
    // fingerprint like `all_of(item = struct, doc_contains("error"))` correctly
    // reports its item-kind (previously `child_top_item_kind` only inspected
    // the top-level Vec and silently missed the nested kind, ATK-LF-1).
    // `node_kind()` returns `None` for `any_of` over item kinds — the
    // widening-via-any_of case is genuinely undecidable at static kind matching
    // and the advisory correctly errs toward silence there (ATK-LF-5 pins this
    // as a known limitation, not a regression).
    //
    // ATK-LF-6: child with `node_kind() = None` is UNCONDITIONALLY BROADER in
    // the item dimension when the parent has a definite kind. Unlike ATK-LF-5
    // (any_of is undecidable), this case IS decidable: parent=Some(Struct) +
    // child=None means child matches ALL item kinds, including non-struct items
    // the parent would not. That is a widening (not a refinement) that can be
    // statically detected. Flag it rather than silently skipping the check.
    match (parent.node_kind(), child.node_kind()) {
        (Some(pk), Some(ck)) if pk != ck => {
            return Some(format!(
                "child `item = {ck:?}` differs from parent `item = {pk:?}` \
                 — disjoint item kinds cannot be a refinement"
            ));
        },
        (Some(pk), None) => {
            // Parent has a definite item kind; child has NO item constraint —
            // child unconditionally matches a broader set of items than parent.
            // This is not a refinement. (ATK-LF-6)
            return Some(format!(
                "parent constrains `item = {pk:?}` but child has no item-kind \
                 constraint — child matches all item kinds and is broader, not a refinement"
            ));
        },
        _ => {},
    }

    // (2) doc_contains divergence: a parent-required substring that no child
    // doc_contains contains. (If the child contains a SUPERSTRING of the parent's
    // needle, that IS a refinement — child requires more, matches a subset.)
    //
    // BOTH SIDES descend into `AllOf` ONLY (conjunctive children — every AllOf
    // child applies, so a `doc_contains` nested inside AllOf is required just
    // like a top-level one). Do NOT descend into `AnyOf` (disjunctive — a
    // parent `any_of([doc_contains("A"), doc_contains("B")])` requires "A" OR
    // "B", not both; treating the AnyOf arms as required would false-positive
    // on a child satisfying only one arm — ATK-LF-4). Do NOT descend into
    // `Not` (negative requirement, not a substring this advisory can check).
    // ATK-LF-2 pinned that the top-level-only iteration missed nested AllOf
    // requirements; ATK-LF-4 pins that the fix must NOT over-descend into
    // AnyOf.
    let child_docs = collect_doc_contains_allof_only(&child.constraints);
    let parent_docs = collect_doc_contains_allof_only(&parent.constraints);
    for parent_needle in &parent_docs {
        let covered = child_docs.iter().any(|cd| cd.contains(parent_needle));
        if !covered {
            return Some(format!(
                "parent requires `doc_contains({parent_needle:?})` but no child \
                 `doc_contains` includes it — child can match where parent does not"
            ));
        }
    }

    None
}

/// Recursively collect all `DocContains` substring requirements from a
/// constraint list, descending into `AllOf` children only.
///
/// `AllOf` is conjunctive — every child applies — so a `doc_contains` nested
/// inside `AllOf` is a required substring just like a top-level `DocContains`.
/// `AnyOf` is disjunctive: descending into it would over-constrain (treating
/// alternatives as required), turning a valid refinement into a false-positive
/// divergence (ATK-LF-4). `Not` is a negative requirement the advisory does
/// not model. So the descent is `AllOf`-only.
fn collect_doc_contains_allof_only(constraints: &[antigen_fingerprint::Constraint]) -> Vec<&str> {
    use antigen_fingerprint::Constraint;
    let mut out = Vec::new();
    for c in constraints {
        match c {
            Constraint::DocContains(s) => out.push(s.as_str()),
            Constraint::AllOf(children) => {
                out.extend(collect_doc_contains_allof_only(children));
            },
            _ => {}, // AnyOf / Not / other leaves: do not descend.
        }
    }
    out
}
