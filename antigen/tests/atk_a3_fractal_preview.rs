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

// Tests in this file are pre-implementation contracts. As each A3 deliverable
// ships, the corresponding test loses its #[ignore] and gains a real fixture-
// driven body. ATK-A3-002 (cycle detection) and ATK-A3-003 (stale lineage)
// were activated when scan-side cycle detection + lineage-edge collection
// landed (A3 deliverable 1 + 2).

use antigen::scan::scan_workspace;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

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
fn atk_a3_002_circular_descended_from_chain_is_detected_not_infinite_loop() {
    // Contract: a workspace with `Alpha #[descended_from(Beta)]` and
    // `Beta #[descended_from(Alpha)]` must produce a parse_failure with
    // the chain text — not hang indefinitely or stack-overflow. ADR-005
    // Amendment 3 (crash-resistance) — cycle detection is a hard entry
    // requirement, not optional.
    //
    // The fixture uses struct types because #[descended_from] applies to
    // antigen-type declarations (unit struct + class enum) per ADR-013;
    // applying it to functions is a parse_failure on its own (separate
    // from the cycle detection path under test here).
    let fixture_root = fixture("atk_a3_002_circular_lineage");
    let scan = scan_workspace(&fixture_root, None).expect("scan must complete, not hang");

    assert_eq!(
        scan.lineage_edges.len(),
        2,
        "fixture has two #[descended_from] declarations forming a cycle"
    );

    // Cycle must surface as a parse_failure (structural error channel —
    // scan cannot complete a propagation walk correctly). Other channels
    // (orphaned_lineage_edges) are for semantic warnings, not crashes.
    let cycle_failures: Vec<_> = scan
        .parse_failures
        .iter()
        .filter(|f| f.error.contains("cycle"))
        .collect();
    assert_eq!(
        cycle_failures.len(),
        1,
        "exactly one cycle should be reported (dedup across entry points), \
         got: {:?}",
        scan.parse_failures
    );

    // Chain text must be present in the error so the user can identify
    // which edges form the loop.
    let err = &cycle_failures[0].error;
    assert!(
        err.contains("Alpha") && err.contains("Beta"),
        "cycle error must name both nodes in the chain, got: {err}"
    );
    assert!(
        err.contains("->"),
        "cycle error must render the chain with `->` between nodes, got: {err}"
    );
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
fn atk_a3_003_stale_descended_from_reference_is_flagged_not_silently_dropped() {
    // Contract: #[descended_from(MissingParent)] where MissingParent is not
    // declared in the scanned workspace must surface via the
    // orphaned_lineage_edges() query method — parallel to orphaned_tolerances().
    //
    // Channel taxonomy (scope-lock §"Stale reference handling"):
    //   - parse_failures: structural errors that prevent correct scan
    //     completion (file IO, fingerprint parse errors, cycles)
    //   - orphaned_lineage_edges() / orphaned_tolerances(): semantic warnings,
    //     scan completed but a declaration references something no longer
    //     present. Caller decides severity.
    //
    // The silent-failure mode this test guards against: if a stale lineage
    // reference were silently dropped, propagation would resolve to "no
    // inherited presentations" — indistinguishable from a valid antigen
    // that genuinely has no parent presentations. The query method makes
    // the orphan visible without forcing scan failure.
    let fixture_root = fixture("atk_a3_003_stale_lineage");
    let scan = scan_workspace(&fixture_root, None).expect("scan must complete");

    assert_eq!(
        scan.lineage_edges.len(),
        1,
        "fixture has one #[descended_from] declaration"
    );
    assert_eq!(
        scan.antigens.len(),
        1,
        "fixture declares Child but NOT MissingParent — that's the orphan setup"
    );

    // Stale references must NOT appear as parse_failures (different channel).
    assert!(
        scan.parse_failures
            .iter()
            .all(|f| !f.error.contains("MissingParent")),
        "stale lineage references must not be reported as parse_failures, got: {:?}",
        scan.parse_failures
    );

    let orphans = scan.orphaned_lineage_edges();
    assert_eq!(
        orphans.len(),
        1,
        "stale lineage reference must surface as one orphan, got: {orphans:?}"
    );
    assert_eq!(orphans[0].child, "Child");
    assert_eq!(orphans[0].parent, "MissingParent");
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

// ============================================================================
// ATK-A3-005: cross-crate immunity false-positive — naive file-guard removal
//
// v0.1 `unaddressed_presentations` uses `i.file == p.file` as a proxy for
// "same item." The A3 cross-crate case requires relaxing this guard so that
// an `#[immune]` in `crate-a` can address a fingerprint-matched presentation
// in `crate-b`. But naive removal of the file guard would cause false-positive
// address matches across unrelated types:
//
//   crate-a/src/lib.rs: struct MyType (with #[immune(PanickingInDrop, ...)])
//   crate-b/src/lib.rs: struct MyType (with FingerprintMatch for PanickingInDrop)
//
// Both have `ItemTarget::Struct("MyType")`. Without the file guard, `addresses()`
// returns true — the immunity in crate-a suppresses the presentation in crate-b
// despite them being completely unrelated types that happen to share a name.
//
// The correct A3 fix is NOT to remove the file guard but to replace it with
// a module-path-aware identity: `ItemTarget` variants must carry crate-qualified
// paths (e.g., `crate_a::MyType` vs `crate_b::MyType`). The `i.file == p.file`
// guard becomes `i.module_path == p.module_path` where module_path is the
// fully-qualified crate + module path.
//
// This requires changing the `ItemTarget` representation — it's an ADR-class
// decision, not a one-line fix. File at A3 open to ensure it's in scope before
// implementation begins.
//
// Status: #[ignore] until A3 ships cross-crate identity model.
// ============================================================================

#[test]
#[ignore = "A3 pre-implementation contract; cross-crate name collision via naive file-guard removal — address when A3 designs cross-crate ItemTarget identity"]
fn atk_a3_005_cross_crate_name_collision_not_suppressed_by_same_name_immunity() {
    // Contract: an #[immune(X)] on `crate_a::MyType` must NOT suppress an
    // unaddressed presentation for `crate_b::MyType`, even if both have
    // ItemTarget::Struct("MyType"). The fix requires module-path-qualified
    // identity in ItemTarget, not just name equality.
    //
    // The failing scenario (naive cross-crate implementation):
    //   1. crate-a declares #[immune(PanickingInDrop, witness = my_test)] on MyType
    //   2. crate-b has a fingerprint-matched presentation on its own MyType
    //   3. A3 relaxes the `i.file == p.file` guard to allow cross-crate matching
    //   4. ItemTarget::Struct("MyType").addresses(ItemTarget::Struct("MyType")) = true
    //   5. The crate-b presentation is silently suppressed — false green
    //
    // TODO(adversarial): write multi-crate fixture when A3 ships cross-crate scan.
    panic!("A3 pre-implementation contract");
}
