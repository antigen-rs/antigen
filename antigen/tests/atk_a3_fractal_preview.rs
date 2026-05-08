//! ATK-A3 fractal-prediction pre-implementation contracts.
//!
//! Filed during A2 adversarial sweep based on naturalist's fractal prediction:
//! "wherever antigen does recognition work, there will be a structural-variant
//! blind spot." Both sites here are A3 scope (cross-crate scan +
//! `#[descended_from]` propagation). All tests are `#[ignore]` until A3 ships.
//!
//! The fractal pattern: the recognizer catches one structural variant and
//! silently misses others. Same shape at every tier the project has examined:
//! - Events tier: `UlpDistanceRolledByHand` (inline vs multi-statement body)
//! - Coordination tier: ratification signals (outbox vs substrate state)
//! - Implementation tier: `attr.path().is_ident()` (bare vs path-qualified)
//! - This file: recognition-work in A3 (cross-crate + lineage walking)
//!
//! See: campsites/antigen-design/20260508120021-.../naturalist/20260508-tier-confusion-roam.md
//! and the-fractal-was-the-architecture.md garden entry.

// All tests in this file are #[ignore] pre-implementation contracts.
// Fixtures and scan_workspace will be needed when A3 ships.

// ============================================================================
// ATK-A3-001: cross-crate witness resolution — re-export structural variant
//
// A3 will implement cross-crate witness resolution. The fractal prediction:
// A3's initial implementation will handle the direct-dependency case and
// silently miss witnesses accessed through re-exports.
//
// Concrete case: crate A re-exports `pub use crate_b::test_fn`. A user writes
// `witness = crate_a::test_fn`. A3's resolver walks crate_a's items, finds
// the re-export marker, but the actual function body lives in crate_b. If the
// resolver only indexes items that are *defined* in a crate (not re-exported),
// the witness silently resolves as NotFound even though the function exists
// and is accessible.
//
// Impact: every library that tidies its public API with re-exports (common
// Rust practice) will produce false NotFound results for witnesses referencing
// those re-exported functions.
//
// Status: #[ignore] until A3 ships cross-crate witness resolution.
// ============================================================================

#[test]
#[ignore = "A3 pre-implementation contract; cross-crate witness re-export variant — file when A3 ships"]
fn atk_a3_001_reexported_witness_resolves_correctly() {
    // Contract: witness = crate_a::test_fn where test_fn is re-exported from
    // crate_b must resolve to the function's actual definition in crate_b,
    // not produce NotFound because the indexer only finds defined-here items.
    //
    // TODO(adversarial): write multi-crate fixture when A3 ships cross-crate scan.
    panic!("A3 pre-implementation contract");
}

// ============================================================================
// ATK-A3-002: #[descended_from] circular chain — infinite loop risk
//
// When A3 implements lineage walking for #[descended_from], a circular chain
// (fn A descended from fn B descended from fn A) must be detected and surfaced
// as an error, not cause the scan to loop indefinitely or stack-overflow.
//
// This is the highest-severity structural-variant miss: it's not a silent
// wrong-answer but a hang or crash. Circular chains can occur:
//   - Accidentally (copy-paste error in attribute arguments)
//   - Through refactoring (A descended from B, then B is refactored to
//     descend from A without removing A's declaration)
//   - Through multi-hop chains that aren't visually obvious in a large codebase
//
// The fix is standard: cycle detection via a visited set during the lineage walk.
// The contract: if a circular #[descended_from] chain exists, scan must either:
//   (a) report a parse_failure explaining the cycle, or
//   (b) terminate the walk at the cycle point and report it
// NOT hang, crash, or silently produce a partial result.
//
// Status: #[ignore] until A3 ships #[descended_from] propagation.
// ============================================================================

#[test]
#[ignore = "A3 pre-implementation contract; #[descended_from] cycle detection — remove ignore when A3 ships lineage walking"]
fn atk_a3_002_circular_descended_from_chain_is_detected_not_infinite_loop() {
    // Contract: a workspace containing:
    //   fn_a: #[descended_from(fn_b)]
    //   fn_b: #[descended_from(fn_a)]
    // must produce a parse_failure or diagnostic, not hang indefinitely.
    //
    // TODO(adversarial): write fixture with circular chain when A3 ships.
    // This is the crash variant of the structural-variant blind spot — not a
    // wrong answer but a catastrophic failure mode. File immediately as A3
    // opens to ensure cycle detection is in scope from the start.
    panic!("A3 pre-implementation contract — cycle detection is a safety requirement, not optional");
}

// ============================================================================
// ATK-A3-003: #[descended_from] pointing at removed/renamed parent
//
// When a parent function is renamed or removed, a #[descended_from] marker
// pointing at the old name becomes a stale reference. A3's lineage walker
// must detect this and surface it as a broken chain, not silently drop
// the inheritance (producing a false "no presentations inherited" result).
//
// This is the stale-tolerance analog at the lineage layer (same shape as
// ATK-A2-009 stale tolerance orphans, but for inheritance chains rather than
// explicit tolerance markers).
//
// Biology analog: B-cell lineage whose progenitor no longer exists —
// the clonal line is orphaned. The memory cells still carry the antigen-
// specificity, but there's no parent to verify divergence against.
//
// Status: #[ignore] until A3 ships #[descended_from] propagation.
// ============================================================================

#[test]
#[ignore = "A3 pre-implementation contract; stale #[descended_from] reference — remove ignore when A3 ships lineage walking"]
fn atk_a3_003_stale_descended_from_reference_is_flagged_not_silently_dropped() {
    // Contract: #[descended_from(nonexistent_parent)] must produce a
    // diagnostic (parse_failure or scan warning), not silently succeed with
    // "no inherited presentations" — which would be indistinguishable from
    // a valid function that happens to have no presentations.
    //
    // The key silent-failure mode: if the lineage walker returns "chain
    // resolved to nothing" without distinguishing "parent doesn't exist" from
    // "parent exists and has no presentations," the developer cannot know
    // whether their inheritance chain is working.
    //
    // TODO(adversarial): write fixture when A3 ships lineage walking.
    panic!("A3 pre-implementation contract");
}

// ============================================================================
// ATK-A3-004: cross-crate witness via proc-macro generated impl
//
// A witness function that lives inside a proc-macro generated impl block
// (e.g., from #[derive(Debug)]) is not directly visible in the source AST.
// A3's cross-crate resolver, which walks source files, cannot find it.
//
// This is the #[antigen_generates] problem (ADR-014) applied to witness
// resolution. The fractal: the resolver recognizes hand-written source items
// and silently misses proc-macro generated items.
//
// Impact: any codebase using derive macros (nearly all Rust code) may have
// witnesses in generated impls that A3 cannot find. The audit reports NotFound
// for legitimate witnesses.
//
// Note: ADR-014 defers #[antigen_generates] to A3/A4. This ATK is the
// witness-resolution side of the same problem. They should be addressed
// together.
//
// Status: #[ignore] until A3 ships cross-crate witness resolution.
// ============================================================================

#[test]
#[ignore = "A3 pre-implementation contract; proc-macro generated witness — remove ignore when A3 ships cross-crate witness resolution"]
fn atk_a3_004_proc_macro_generated_witness_is_handled_not_silently_missing() {
    // Contract: witness = MyType::fmt (where fmt is generated by #[derive(Debug)])
    // must either resolve correctly OR produce a clear diagnostic explaining
    // that generated impls are not yet resolvable — not silently NotFound.
    //
    // TODO(adversarial): write fixture when A3 ships.
    panic!("A3 pre-implementation contract");
}
