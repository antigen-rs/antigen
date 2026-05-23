// ATK-CE-4: polyclonal/monoclonal audit semantics check.
//
// Per ADR-024: #[polyclonal] and #[monoclonal] have no required args.
// The question: does audit_convergent_evidence() DO anything with them,
// or are they pure documentation markers?
//
// Per ADR-024 §Enforcement-Surface: NOT in the enforcement table.
// This means they ARE pure documentation at v0.2.
//
// The adversarial concern:
// - If pure documentation, adopters can mark any function #[polyclonal]
//   without providing any evidence of multiple lineages.
// - The audit hint `polyclonal-insufficient-lineages` is in the hint vocabulary
//   but is never emitted (because there's nothing to check against).
//
// This fixture confirms: both macros apply without args AND the test
// verifies whether audit emits anything for them (expected: nothing,
// because they're declarative markers at v0.2).
//
// If audit emits polyclonal-insufficient-lineages on this fixture
// WITHOUT any threshold being set, that's a false positive.
// If audit emits NOTHING, document that CE-4 is pure documentation.

use antigen::polyclonal;
use antigen::monoclonal;

/// ATK-CE-4-A: polyclonal with no evidence.
/// v0.2 — should this emit polyclonal-insufficient-lineages?
/// Per ADR enforcement table: NOT listed. Expected: no audit hint.
/// Document finding either way.
#[polyclonal]
pub fn atk_ce4_polyclonal_no_evidence() {}

/// ATK-CE-4-B: monoclonal with no evidence.
/// Same question. Expected: no audit hint (pure documentation marker).
#[monoclonal]
pub fn atk_ce4_monoclonal_no_evidence() {}

/// ATK-CE-4-C: adcc with no evidence.
/// Expected: no audit hint.
#[adcc]
pub fn atk_ce4_adcc_no_evidence() {}
