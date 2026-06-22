//! Antigen-Category audit (ADR-028 — G1/G2/G3 enforcement at scan/audit time).
//!
//! Extracted from the former monolithic `audit.rs` per ADR-036 (the scan/audit
//! orchestration decomposition). A pure detector module: a fn of `&ScanReport`
//! returning its own report; no detector calls another (single-conductor
//! invariant, ADR-036). API-invisible: re-exported from the `audit` root via
//! `pub use`.

use serde::{Deserialize, Serialize};

use super::AuditHint;
use crate::scan::ScanReport;

// ============================================================================
// Antigen-Category audit (ADR-028 — G1 scan-time-only enforcement)
// ============================================================================

/// Per-declaration antigen-category audit result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryAudit {
    /// The antigen declaration's `type_name` (for cross-referencing).
    pub antigen_type: String,
    /// Source file path.
    pub file: std::path::PathBuf,
    /// Line number.
    pub line: usize,
    /// Hints surfaced for this declaration (may be empty = clean).
    pub hints: Vec<AuditHint>,
}

/// Aggregate antigen-category audit report.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CategoryAuditReport {
    /// Per-declaration audit results (only declarations with concerns are
    /// recorded; clean declarations are counted but not listed).
    pub audits: Vec<CategoryAudit>,
    /// Count of antigen declarations with an explicit (non-empty) category.
    pub explicit_count: usize,
    /// Count of antigen declarations with an absent (empty) category — each
    /// surfaced the `antigen-category-defaulted-implicit-functional` hint.
    pub defaulted_count: usize,
    /// Count of explicit-category declarations whose category is NOT backed by
    /// an immunity of the matching witness type (G2 cross-check) — each
    /// surfaced the `antigen-category-claim-inconsistent-with-predicate-type`
    /// hint.
    #[serde(default)]
    pub mismatch_count: usize,
}

impl CategoryAuditReport {
    /// True when every antigen declaration carries an explicit category.
    #[must_use]
    pub const fn all_explicit(&self) -> bool {
        self.defaulted_count == 0
    }

    /// True when no explicit-category declaration has a category↔witness-type
    /// mismatch (G2 cross-check is clean).
    #[must_use]
    pub const fn no_category_witness_mismatch(&self) -> bool {
        self.mismatch_count == 0
    }

    /// True when no `SubstrateAlignment` antigen carries a silence-witness
    /// shape-mismatch advisory (neither no-witness nor wrong-tier). Derived
    /// from the audits rather than a counter: the no-witness hint is an
    /// orthogonal gap G2 does not count, and the wrong-tier hint rides
    /// alongside a G2 mismatch already counted by `mismatch_count`, so a
    /// dedicated scan keeps the silence-witness signal independent of the
    /// G2 count.
    #[must_use]
    pub fn no_silence_witness_mismatch(&self) -> bool {
        !self.audits.iter().any(|ca| {
            ca.hints
                .contains(&AuditHint::AntigenWitnessShapeMismatchForSilenceNoWitness)
                || ca
                    .hints
                    .contains(&AuditHint::AntigenWitnessShapeMismatchForSilenceWrongTier)
        })
    }
}

/// Audit antigen-category coverage across a scan report (ADR-028).
///
/// Two checks, both at audit time:
///
/// **G1 (scan-time-only enforcement)**: emits
/// [`AuditHint::AntigenCategoryDefaultedImplicitFunctional`] for any
/// [`crate::scan::AntigenDeclaration`] whose `category` field is empty
/// (absent). This is the load-bearing signal that makes absent-category
/// VISIBLE in v0.2 — without it, the soft-default to `[FunctionalCorrectness]`
/// would be a silent false-green. v0.1/v0.2 discrimination + parse-time
/// hard-error are the v0.2.x migration-record slice; for v0.2 the hint fires
/// for ALL absent-category declarations (both carry-overs and new), since
/// both should migrate.
///
/// **G2 (category↔witness-type cross-check, per Amendment 2)**:
/// for each explicit-category declaration, joins the immunities addressing it
/// ([`crate::scan::Immunity::antigen_type`] == the declaration's `type_name`)
/// and emits [`AuditHint::AntigenCategoryClaimInconsistentWithPredicateType`]
/// when the declared category is not backed by an immunity of the matching
/// witness type. The witness-type is read structurally from each immunity:
/// `requires_predicate.is_some()` is a substrate-witness; a non-empty
/// `witness` is a code-witness. This check lives at audit time because the
/// antigen↔immunity join only exists once the scan report assembles — a
/// single `#[antigen]` cannot see its separately-declared `#[immune]`s at
/// macro-expand time. A declaration with NO immunities addressing it is not a
/// mismatch (the immunity coverage gap is a separate concern); the check only
/// fires when immunities exist but are of the wrong type for the category.
#[must_use]
pub fn audit_category(report: &ScanReport) -> CategoryAuditReport {
    use crate::category::AntigenCategory;

    let mut audits = Vec::new();
    let mut explicit_count = 0usize;
    let mut defaulted_count = 0usize;
    let mut mismatch_count = 0usize;

    for decl in &report.antigens {
        if decl.category.is_empty() {
            defaulted_count += 1;
            audits.push(CategoryAudit {
                antigen_type: decl.type_name.clone(),
                file: decl.file.clone(),
                line: decl.line,
                hints: vec![AuditHint::AntigenCategoryDefaultedImplicitFunctional],
            });
            continue;
        }

        explicit_count += 1;

        // G2 cross-check. Read the witness-types present across all immunities
        // addressing this antigen. An immunity is a substrate-witness when it
        // carries a `requires = <predicate>` (requires_predicate is Some); it
        // is a code-witness when it carries a non-empty `witness = <fn>`.
        //
        // Canonical-path-aware (same discipline as the `report.defenses` loop
        // below, and as `scan::defense_addresses`): a `#[immune(Foo)]` /
        // `#[presents(Foo)]` from a DIFFERENT crate must not satisfy this
        // crate's `Foo` cross-check. Without the guard, a dependency's
        // code-tier immunity for a same-bare-name antigen sets
        // `has_any_immunity`/`has_code_witness` on THIS crate's antigen — a
        // cross-crate overclaim (ATK-G2-24) that both fires a spurious G2
        // mismatch AND silences the silence-no-witness advisory for an antigen
        // that genuinely has no local witness (ATK-G2-25). An immunity with
        // `canonical_path = None` matches any (backward-compat, mirrors the
        // defense loop).
        let mut has_substrate_witness = false;
        let mut has_code_witness = false;
        let mut has_any_immunity = false;
        for imm in &report.immunities {
            if imm.antigen_type != decl.type_name {
                continue;
            }
            // Strict canonical-path equality (forward/shared-canonical-path-addresses-helper
            // ruling: None == None only; None ≠ Some). Previously `is_some() && ≠` let an
            // intra-workspace immunity (None) address ANY antigen including stamped dep
            // declarations — same class of wildcard that ATK-ADR029-23 fixed on the defense
            // side. Route through `scan::canonical_paths_match` — the single source of truth
            // for the canonical-path dimension of every "does X address antigen Y" check, so
            // this rule cannot drift independently of the scan-layer defense/tolerance sites.
            if !crate::scan::canonical_paths_match(
                imm.canonical_path.as_deref(),
                decl.canonical_path.as_deref(),
            ) {
                continue;
            }
            has_any_immunity = true;
            if imm.requires_predicate.is_some() {
                has_substrate_witness = true;
            }
            if !imm.witness.is_empty() {
                has_code_witness = true;
            }
        }
        // ADR-029: a `#[defended_by(X)]` registration is a CODE-TIER witness for
        // X — the migration target for `#[immune(X, witness=fn)]`. G2 must
        // consult `report.defenses` too, or every adopter who moves from
        // `#[immune]` to `#[defended_by]` silently bypasses this witness-type
        // cross-check (a SubstrateAlignment antigen defended ONLY by a code-tier
        // `#[defended_by]` would go unflagged — the wrong witness type for the
        // declared category). A matching defense counts as code-tier evidence
        // addressing this antigen, exactly as a `witness=` immunity does.
        //
        // Canonical-path-aware (mirrors `scan::defense_addresses`, but matched
        // against the declaration's canonical_path rather than a presentation's):
        // a `#[defended_by(Foo)]` from a DIFFERENT crate must not satisfy this
        // crate's `Foo` G2 check (ATK-G2-22 / ATK-ADR029-23 cross-crate
        // overclaim). The canonical-path dimension routes through
        // `scan::canonical_paths_match` (None matches None only; Some(x) matches
        // Some(x) only) — the single source of truth shared with the scan-layer
        // defense/tolerance/immunity sites. `stamp_canonical_path` runs
        // all-or-nothing per scan, so (None defense, Some decl) is always
        // cross-boundary and correctly fails the match.
        if report.defenses.iter().any(|d| {
            d.antigen_type == decl.type_name
                && crate::scan::canonical_paths_match(
                    d.canonical_path.as_deref(),
                    decl.canonical_path.as_deref(),
                )
        }) {
            has_any_immunity = true;
            has_code_witness = true;
        }

        let wants_substrate = decl.category.contains(&AntigenCategory::SubstrateAlignment);
        let wants_code = decl
            .category
            .contains(&AntigenCategory::FunctionalCorrectness);
        let is_hybrid = wants_substrate && wants_code;

        // No immunities/defenses addressing this antigen is not (for G2) a
        // category mismatch — it's an orthogonal coverage gap, so G2 bails.
        // But for a SubstrateAlignment antigen, no-witness-at-all IS the
        // silence-by-absence generator: SA failures are detected only by a
        // mechanism asserting the closure exists, and none is wired. Emit the
        // silence-no-witness advisory here (the gap G2 deliberately leaves),
        // then continue — there is no witness TYPE to cross-check.
        if !has_any_immunity {
            if wants_substrate {
                audits.push(CategoryAudit {
                    antigen_type: decl.type_name.clone(),
                    file: decl.file.clone(),
                    line: decl.line,
                    hints: vec![AuditHint::AntigenWitnessShapeMismatchForSilenceNoWitness],
                });
            }
            continue;
        }

        let substrate_satisfied = !wants_substrate || has_substrate_witness;
        let code_satisfied = !wants_code || has_code_witness;

        if substrate_satisfied && code_satisfied {
            continue;
        }

        // The emission is three-way:
        //   - hybrid [SA, FC] with exactly ONE axis witnessed → incomplete
        //     evidence (partial coverage, not a full violation)
        //   - hybrid with ZERO axes witnessed → claim-inconsistent (full
        //     structural violation, same as single-axis)
        //   - single-axis category with no matching witness → claim-inconsistent
        let hybrid_one_axis_witnessed = is_hybrid && (has_substrate_witness ^ has_code_witness);

        let hint = if hybrid_one_axis_witnessed {
            AuditHint::AntigenCategoryHybridIncompleteEvidence
        } else {
            AuditHint::AntigenCategoryClaimInconsistentWithPredicateType
        };

        let mut hints = vec![hint];

        // Silence-witness wrong-tier advisory. A SubstrateAlignment antigen
        // whose ONLY witnesses are code-tier (`witness = fn` / `#[defended_by]`
        // — has_code_witness) with NO substrate predicate (!has_substrate_witness)
        // is defended by a tier that detects behavioral, not substrate-alignment,
        // failures. Co-emitted with G2's primary hint (same root cause — a
        // witness-type mismatch) but carries the silence-generator framing and
        // the actionable "reach for a substrate predicate or bijection-parity
        // test" guidance G2's type-only verdict omits. Suppressed when a
        // substrate witness is also present (the code test is then supplementary)
        // — that is exactly the `!has_substrate_witness` guard. By design,
        // the wrong-weighting generator legitimately uses a code-tier
        // confidence test, so this is advisory: confirm the intended generator
        // before treating it as a mismatch.
        if wants_substrate && has_code_witness && !has_substrate_witness {
            hints.push(AuditHint::AntigenWitnessShapeMismatchForSilenceWrongTier);
        }

        mismatch_count += 1;
        audits.push(CategoryAudit {
            antigen_type: decl.type_name.clone(),
            file: decl.file.clone(),
            line: decl.line,
            hints,
        });
    }

    CategoryAuditReport {
        audits,
        explicit_count,
        defaulted_count,
        mismatch_count,
    }
}
