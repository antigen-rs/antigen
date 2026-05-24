//! # Mucosal Boundary Family — stdlib antigens (ADR-027 + Amendment 1)
//!
//! Canonical trust-boundary failure-classes — the data-flow surfaces where a
//! system meets the outside world and the boundary discipline is absent or
//! mis-applied. Adopters mark boundary functions with `#[mucosal]` /
//! `#[mucosal_delegate]` / `#[mucosal_tolerant]`; these stdlib `#[antigen]`
//! declarations name the failure-classes that an undefended-or-mis-defended
//! boundary presents.
//!
//! ## Antigen-category (ADR-028)
//!
//! All mucosal stdlib antigens are `SubstrateAlignment`: the representation
//! of "this boundary is defended" (the presence/absence/kind of a
//! `#[mucosal]` declaration + its handler chain) diverges from the actual
//! defense state. The witness checks the declaration substrate — is the
//! boundary declared, does the delegate resolve, does the handler's kind
//! match — not runtime sanitization correctness (ADR-027: sanitization
//! presence ≠ correctness; the discipline fires at the boundary regardless).
//!
//! ## Biology grounding (per ADR-027 — NON-NEGOTIABLE)
//!
//! Biology grounds the TIER-CLAIM (mucosal surfaces are a distinct immune
//! tier with selective permeability) + the four functional disciplines, NOT
//! per-variant tissue mapping. The `MucosalKind` taxonomy is
//! software-engineering scope-selection by data-flow type. These antigens
//! are the failure-classes the boundary primitives defend against; they are
//! not per-variant biology-grounded.
//!
//! ## How these antigens are evaluated
//!
//! `cargo antigen mucosal-map` walks the scan report's mucosal declarations
//! and runs `audit_mucosal` (incl. the Change-5 three-tier delegate
//! kind-matching diagnosis). The `fingerprint` uses the uniform
//! `doc_contains("ADR-027")` form.

use crate::antigen;

// ============================================================================
// 1. UndefendedTrustBoundary
// ============================================================================

/// A data-flow boundary admitting external input with no boundary declaration.
///
/// The boundary carries no `#[mucosal]` / `#[mucosal_tolerant]` declaration —
/// neither actively defended nor intentionally tolerated; it is simply
/// undecided (the third response state per ADR-027 Amendment 1 Change 6).
///
/// **The failure mode**: unknown boundaries are where attacks land. A
/// function that receives caller-supplied data without a boundary
/// declaration leaves the audit unable to distinguish "defended elsewhere"
/// from "nobody thought about this." `cargo antigen mucosal-map --undefended`
/// surfaces these.
///
/// **Category**: `SubstrateAlignment` — the absence of a `#[mucosal]`
/// declaration diverges from the actual presence of a trust boundary.
#[antigen(
    name = "undefended-trust-boundary",
    category = AntigenCategory::SubstrateAlignment,
    fingerprint = r#"doc_contains("ADR-027")"#,
    family = "mucosal-boundary",
    summary = "A data-flow boundary admitting external input with no #[mucosal]/#[mucosal_tolerant] declaration — undecided, neither defended nor intentionally tolerated.",
    references = ["ADR-027", "ADR-027#Amendment-1"]
)]
pub struct UndefendedTrustBoundary;

// ============================================================================
// 2. DelegatedDefenseWithoutMatchingHandler
// ============================================================================

/// A `#[mucosal_delegate]` whose handler lacks a matching `#[mucosal(kind)]`.
///
/// The `handled_by` target does not carry a `#[mucosal(kind = X)]` matching
/// the delegated boundary kind — the delegation falsely attests defense that
/// the handler does not actually provide (the Change-5 split-defense problem).
///
/// **The failure mode**: `#[mucosal_delegate(boundary = UserInput,
/// handled_by = sanitize_db)]` passes a naive "does the handler exist?"
/// check even when `sanitize_db` only carries `#[mucosal(kind =
/// DatabaseQuery)]`. The audit's three-tier diagnosis (Change 5) catches
/// this via set-membership kind-matching, emitting
/// `mucosal-discipline-delegate-target-kind-mismatch`.
///
/// **Category**: `SubstrateAlignment` — the delegation claims the boundary
/// is handled, but the handler's declared kind-set diverges from the
/// delegated kind.
#[antigen(
    name = "delegated-defense-without-matching-handler",
    category = AntigenCategory::SubstrateAlignment,
    fingerprint = r#"doc_contains("ADR-027")"#,
    family = "mucosal-boundary",
    summary = "A #[mucosal_delegate] whose handler lacks a #[mucosal(kind=X)] matching the delegated boundary kind — falsely attested defense (Change-5 split-defense).",
    references = ["ADR-027", "ADR-027#Amendment-1-Change-5"]
)]
pub struct DelegatedDefenseWithoutMatchingHandler;

// ============================================================================
// 3. ToleratedBoundaryWithoutReview
// ============================================================================

/// A `#[mucosal_tolerant]` boundary whose tolerance has gone stale or unowned.
///
/// The review deadline (`until`) has passed without re-attestation, or the
/// declaration carries no `reviewed_by` — an intentional-tolerance decision
/// that has gone stale or was never owned.
///
/// **The failure mode**: active tolerance (deliberately permitting
/// unauthenticated input) is the riskier boundary state, so ADR-027
/// Amendment 1 raises its rationale floor to ≥40 chars and adds a review
/// cadence. A tolerant boundary past its `until` date, or with no reviewer,
/// is tolerance that nobody is accountable for — the IBD-analog of immune
/// tolerance gone unchecked.
///
/// **Category**: `SubstrateAlignment` — the tolerance declaration's review
/// state diverges from a current, owned tolerance decision.
#[antigen(
    name = "tolerated-boundary-without-review",
    category = AntigenCategory::SubstrateAlignment,
    fingerprint = r#"doc_contains("ADR-027")"#,
    family = "mucosal-boundary",
    summary = "A #[mucosal_tolerant] boundary past its review deadline or with no reviewed_by — intentional tolerance gone stale or unowned.",
    references = ["ADR-027", "ADR-027#Amendment-1-Change-6"]
)]
pub struct ToleratedBoundaryWithoutReview;
