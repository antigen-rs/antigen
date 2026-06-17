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

use std::path::{Path, PathBuf};

use antigen::scan::scan_workspace;

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
// A3 hardening: #[descended_from] on a non-type item surfaces as parse_failure
//
// Per ADR-013, #[descended_from] applies to antigen-type declarations (unit
// struct + class enum). Silent no-op on a "wrong" item kind (function, impl,
// trait, etc.) would hide user error. The visitor's guard must surface the
// situation explicitly.
// ============================================================================

#[test]
fn a3_descended_from_on_function_is_parse_failure_not_silent() {
    let fixture_root = fixture("atk_a3_descended_from_on_fn");
    let scan = scan_workspace(&fixture_root, None).expect("scan must complete");

    // No lineage edge is recorded for the misuse case — the function is
    // not a valid lineage child.
    assert_eq!(
        scan.lineage_edges.len(),
        0,
        "no lineage edge should be recorded for #[descended_from] on a function"
    );

    let descended_from_failures: Vec<_> = scan
        .parse_failures
        .iter()
        .filter(|f| f.error.contains("#[descended_from]"))
        .collect();
    assert_eq!(
        descended_from_failures.len(),
        1,
        "exactly one parse_failure should surface for the misuse, got: {:?}",
        scan.parse_failures
    );
    let err = &descended_from_failures[0].error;
    assert!(
        err.contains("type declaration") || err.contains("struct") || err.contains("enum"),
        "parse_failure must explain the type-declaration constraint, got: {err}"
    );
}

// ============================================================================
// A3 hardening: an acyclic 3-node lineage chain produces no failures.
//
// Negative-control sibling to ATK-A3-002 — cycle detection must not produce
// false positives on legitimate inheritance chains. Grandchild -> Child ->
// Parent is the canonical "memory cell descended from progenitor" lineage.
// ============================================================================

#[test]
fn a3_acyclic_three_node_chain_records_edges_without_failure() {
    let fixture_root = fixture("atk_a3_acyclic_chain");
    let scan = scan_workspace(&fixture_root, None).expect("scan must complete");

    assert_eq!(
        scan.lineage_edges.len(),
        2,
        "fixture has Grandchild -> Child and Child -> Parent",
    );
    assert_eq!(scan.antigens.len(), 3, "fixture declares three antigens");

    let cycle_failures: Vec<_> = scan
        .parse_failures
        .iter()
        .filter(|f| f.error.contains("cycle") || f.error.contains("maximum depth"))
        .collect();
    assert!(
        cycle_failures.is_empty(),
        "acyclic chain must produce no cycle/depth failures, got: {cycle_failures:?}"
    );
    assert!(
        scan.orphaned_lineage_edges().is_empty(),
        "all parents are declared in the same scan, so no orphans"
    );
}

// ============================================================================
// A3 hardening: a single-edge self-loop is the most degenerate cycle.
//
// Solo #[descended_from(Solo)] — one edge, one node. Cycle detection must
// catch this without looping or hanging. The unit test in scan.rs covers
// `detect_lineage_failures` directly; this test covers the full
// scan_workspace integration path.
// ============================================================================

#[test]
fn a3_self_loop_lineage_detected_through_scan_workspace() {
    let fixture_root = fixture("atk_a3_self_loop_lineage");
    let scan = scan_workspace(&fixture_root, None).expect("scan must complete, not hang");

    let cycle_failures: Vec<_> = scan
        .parse_failures
        .iter()
        .filter(|f| f.error.contains("cycle"))
        .collect();
    assert_eq!(
        cycle_failures.len(),
        1,
        "self-loop must produce exactly one cycle failure, got: {:?}",
        scan.parse_failures
    );
    assert!(
        cycle_failures[0].error.contains("Solo -> Solo"),
        "self-loop error must render `Solo -> Solo` chain, got: {}",
        cycle_failures[0].error
    );
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
fn atk_a3_005_cross_crate_name_collision_not_suppressed_by_same_name_immunity() {
    // Contract (now GREEN, D1.5 commit 4): per ADR-017 §addresses()
    // semantics, an `#[immune(X)]` on `crate_a::MyType` must NOT suppress
    // an unaddressed presentation for `crate_b::MyType`, even if both
    // have `ItemTarget::Struct("MyType")`. The fix is canonical_path
    // tuple identity, not module-path-qualified ItemTarget.
    //
    // The fix shipped: `addresses_for_immunity()` requires
    // `i.canonical_path == p.canonical_path` AND `locus_matches(...)`
    // returns false when canonical_paths differ. So the crate-a immunity
    // never addresses the crate-b presentation.
    //
    // Constructed-scan-report fixture (avoids the multi-crate fixture
    // overhead): one Presentation with canonical_path = "crate_b@1.0.0",
    // one Immunity with canonical_path = "crate_a@1.0.0", both with
    // ItemTarget::Struct("MyType"). Verify the immunity does not
    // suppress the presentation.
    use std::path::PathBuf;

    use antigen::scan::{Immunity, ItemTarget, Presentation, ScanReport};

    let mut report = ScanReport::default();
    // crate_b's presentation (unmarked vulnerability)
    report.presentations.push(Presentation {
        antigen_type: "PanickingInDrop".to_string(),
        file: PathBuf::from("crate_b/lib.rs"),
        line: 5,
        item_kind: "struct".to_string(),
        item_target: ItemTarget::Struct("MyType".to_string()),
        match_kind: antigen::scan::MatchKind::ExplicitMarker,
        canonical_path: Some("crate_b@1.0.0".to_string()),
        inherited_from: None,
        structural_fingerprint: String::new(),
        requires_predicate: None,
        proof: None,
    });
    // crate_a's immunity (different canonical_path — different identity)
    report.immunities.push(Immunity {
        antigen_type: "PanickingInDrop".to_string(),
        witness: "my_test".to_string(),
        file: PathBuf::from("crate_a/lib.rs"),
        line: 8,
        item_kind: "struct".to_string(),
        item_target: ItemTarget::Struct("MyType".to_string()),
        canonical_path: Some("crate_a@1.0.0".to_string()),
        requires_predicate: None,
        structural_fingerprint: String::new(),
    });

    let unaddressed = report.unaddressed_presentations();
    assert_eq!(
        unaddressed.len(),
        1,
        "crate_a's immunity must NOT suppress crate_b's presentation \
         (different canonical_path); got: {unaddressed:?}"
    );
    assert_eq!(
        unaddressed[0].presentation.canonical_path.as_deref(),
        Some("crate_b@1.0.0"),
        "the unaddressed presentation must be the crate_b one"
    );
}

// ============================================================================
// ATK-A3-006: orphaned_lineage_edges cross-crate false-resolution via
// bare-name collision when canonical_path lands
//
// The current `orphaned_lineage_edges()` builds a `known` set using bare
// `type_name` strings (e.g., "Foo"). When canonical_path lands on
// AntigenDeclaration (Approach 3-revised — `canonical_path: Option<String>`),
// two antigens from different crates may both have `type_name = "Foo"`:
//   - crate_a: AntigenDeclaration { type_name: "Foo", canonical_path: Some("crate_a::Foo") }
//   - crate_b: AntigenDeclaration { type_name: "Foo", canonical_path: Some("crate_b::Foo") }
//
// A lineage edge pointing to `crate_b::Foo` (orphaned because crate_b isn't
// in the local scan) would NOT appear as an orphan because `crate_a::Foo`'s
// bare type_name "Foo" is in the known set. The orphan check silently
// resolves the wrong antigen across crate boundaries.
//
// The fix: `orphaned_lineage_edges()` must use canonical_path for comparison
// when both the edge and the antigen carry canonical_path (Some/Some case),
// falling back to bare name only when canonical_path is None on either side
// (intra-workspace / pre-cross-crate case).
//
// This is the orphan-detection analog of ATK-A3-005 (immunity false-positive
// via naive file-guard removal). Same structural shape: bare-name equality
// across crate boundaries produces false-resolution.
//
// Status: #[ignore] until canonical_path lands on AntigenDeclaration (A3 D3+).
// ============================================================================

#[test]
fn atk_a3_006_orphan_edge_canonical_path_false_resolution() {
    // Contract (now GREEN, D1.5 commit 3): a lineage edge pointing to
    // `foo@2.0.0::Foo` must NOT be resolved as non-orphaned because
    // `foo@1.0.0::Foo` (different canonical_path) is in the scan. The
    // orphan query uses (type_name, canonical_path) tuple comparison per
    // ADR-017 + ADR-018 §Enforcement.
    use std::path::PathBuf;

    use antigen::scan::{AntigenDeclaration, LineageEdge, ScanReport};

    let mut report = ScanReport::default();
    // crate_a has `Foo` declared
    report.antigens.push(AntigenDeclaration {
        name: "foo".to_string(),
        type_name: "Foo".to_string(),
        file: PathBuf::from("crate_a/lib.rs"),
        line: 1,
        family: None,
        summary: None,
        fingerprint: None,
        canonical_path: Some("foo@1.0.0".to_string()),
        category: Vec::new(),
        provenance: None,
        presentation: None,
    });
    // Lineage edge points to `foo@2.0.0::Foo` — different canonical_path
    // than the declared antigen. Must surface as orphan.
    report.lineage_edges.push(LineageEdge {
        child: "Bar".to_string(),
        parent: "Foo".to_string(),
        file: PathBuf::from("workspace/lib.rs"),
        line: 5,
        parent_canonical_path: Some("foo@2.0.0".to_string()),
        child_canonical_path: None,
    });

    let orphans = report.orphaned_lineage_edges();
    assert_eq!(
        orphans.len(),
        1,
        "edge pointing at `foo@2.0.0::Foo` must be orphan even though \
         `foo@1.0.0::Foo` is declared (different canonical_path); \
         got: {orphans:?}"
    );
    assert_eq!(orphans[0].parent, "Foo");
    assert_eq!(
        orphans[0].parent_canonical_path.as_deref(),
        Some("foo@2.0.0")
    );
}

// ============================================================================
// ATK-A3-007: fake registry path bypasses cross-crate trust delegation
//
// REFRAMED (post-ADR-017 ratification, 2026-05-09): the trust boundary is at
// `enumerate_dep_crate_roots`, not at `scan_workspace`. The original contract
// targeted the wrong layer.
//
// Attack: create a directory at
//   `.cargo/registry/src/<fake-index>/implausible-crate-9.9.9/`
// containing a valid-looking `lib.rs` with `#[antigen]` declarations.
// This directory passes the registry layout check but is NOT listed in the
// workspace's Cargo.toml dependencies and therefore NOT reachable from
// `cargo metadata`.
//
// The attack is distinct from cargo's supply-chain checksum verification —
// cargo verifies the integrity of packages it FETCHED; it says nothing about
// files planted manually in the registry directory afterward.
//
// Impact: if a caller bypasses `enumerate_dep_crate_roots` and passes the
// fake path directly to `scan_workspace`, injected antigens appear as real
// cross-crate declarations. Immunity claims against them suppress legitimate
// unaddressed-presentation warnings.
//
// Correct trust boundary (ADR-017 ratified): `enumerate_dep_crate_roots` is
// the ONLY public mechanism that returns registry paths for scanning.
// `scan_workspace` is a plain directory scanner — it has no trust logic and
// is not the right layer for rejection. Trust enforcement lives entirely in
// the enumeration layer.
//
// ADR-017 explicitly states: "Do not add alternative path-discovery mechanisms
// that bypass `enumerate_dep_crate_roots` — doing so bypasses the sub-clause F
// delegation and requires a separate trust argument. ATK-A3-007 enforces this
// discipline."
//
// Correct contract: call `enumerate_dep_crate_roots` against a workspace where
// a fake crate directory exists at `.cargo/registry/src/<index>/fake-9.9.9/`
// but is NOT listed in the workspace's Cargo.toml dependencies. Assert the fake
// path does NOT appear in the returned `Vec<DepCrateRoot>`. Trust is enforced
// at enumeration time, not scan time.
//
// Status: #[ignore] — `enumerate_dep_crate_roots` is live (A3 D3 shipped) but
// the real-workspace fixture for this specific fake-path case is not yet built.
// ============================================================================

#[test]
#[ignore = "A3 pre-implementation contract; fake registry path must not appear in \
             enumerate_dep_crate_roots output — trust boundary is enumeration, not scan; \
             remove ignore when fake-registry fixture is built"]
fn atk_a3_007_fake_registry_path_not_in_cargo_metadata_is_rejected() {
    // Contract: call `enumerate_dep_crate_roots` against a workspace that does
    // NOT list `fake-crate` as a dependency. A directory planted at
    // `.cargo/registry/src/<index>/fake-9.9.9/` MUST NOT appear in the returned
    // Vec<DepCrateRoot>. The fake path is excluded at enumeration time because
    // it is not reachable from `cargo metadata`'s resolution graph.
    //
    // The test does NOT call scan_workspace on the fake path — that would bypass
    // the trust boundary. This contract specifically exercises `enumerate_dep_crate_roots`
    // as the sub-clause F delegation point (ADR-017).
    //
    // TODO(adversarial): build a fixture workspace with a planted fake registry
    // directory and assert enumerate_dep_crate_roots excludes it.
    //
    // See ADR-017 §"The trust delegation" for the two-precondition model.
    // See also: git grep "enumerate_dep_crate_roots" antigen/ for the implementation.
    panic!("A3 pre-implementation contract — trust boundary is enumerate_dep_crate_roots");
}

// ============================================================================
// ATK-A3-008: ADR-018 duplicate edge silent swallow in synthesis pass
//
// ADR-018 §"Edge-level dedup" ratifies: "before the propagation walk, edges
// are deduped by (child, parent, child_canonical_path, parent_canonical_path)
// tuple." This means BUG-A3-001's duplicate edges get collapsed silently at
// the synthesis pass — no failure is emitted to the user.
//
// But the unit test `atk_a3_dup_duplicate_lineage_edge_is_diagnosed_not_silent`
// (scan::tests, currently FAILING) asserts a diagnostic SHOULD be emitted.
//
// This pre-implementation contract records the design question:
//   Should duplicate #[descended_from] be:
//   (a) Silently deduped (ADR-018 synthesis pass dedup — no user feedback), OR
//   (b) Diagnosed with a warning/parse_failure before dedup (explicit elevation
//       per ADR-004)?
//
// The fractal prediction: wherever dedup is silent, a user who writes the
// attribute twice accidentally receives no feedback. The sub-clause F argument:
// the user's intent (two distinct declarations) is not what they get (one
// silently collapsed). This is the same shape as stale-tolerance orphans —
// structural incoherence that surfaces via orphan query, not parse_failure.
//
// Contract: duplicate (child, parent) edges must produce at least one visible
// diagnostic — either a parse_failure or inclusion in a future
// `duplicate_lineage_edges()` query. Silent dedup is not acceptable because it
// makes the user's structural error invisible.
//
// Note: if ADR-018 is amended to emit a diagnostic for duplicates at dedup time,
// the existing FAILING test in scan::tests should be updated to match the
// actual diagnostic channel (parse_failure vs a new query method).
//
// Status: #[ignore] until ADR-018 is ratified and propagation pass lands.
// ============================================================================

#[test]
#[ignore = "spirit satisfied by `dedupe_lineage_edges` parse_failures channel; \
             literal assertion targets a different diagnostic path; defer to \
             A4+ if channel unification is needed."]
fn atk_a3_008_duplicate_edge_adr018_synthesis_dedup_emits_diagnostic() {
    // Contract: duplicate #[descended_from(B)] on struct A, after ADR-018
    // edge-level dedup collapses the two edges to one, must emit at least one
    // visible diagnostic explaining the collapse. Scan report's duplicate_lineage
    // count should be queryable.
    //
    // TODO(adversarial): implement after ADR-018 ratification defines the
    // diagnostic channel for edge-level dedup discards.
    panic!("A3 pre-implementation contract");
}

// ============================================================================
// ATK-A3-009: multi-version same-crate antigen fingerprint divergence — silent
// wrong immunity address
//
// DISPOSED (post-ADR-017 ratification, 2026-05-09): the original attack surface
// is structurally eliminated by the ratified "name@version" canonical_path format.
//
// Original scenario (no longer possible):
//   The draft had canonical_path = crate name only (e.g., Some("foo")).
//   Two versions of the same antigen would both produce canonical_path = Some("foo"),
//   causing an immunity validated against v1 to silently satisfy a v2 presentation.
//
// Why it's eliminated:
//   ADR-017 ratified canonical_path = "name@version" (e.g., "foo@1.0.0"). With this
//   format, "foo@1.0.0" ≠ "foo@1.1.0" at the identity level. The scanner produces two
//   distinct AntigenDeclaration identities; an immunity against one canonical_path does
//   not silently address presentations from the other. The silent match cannot occur.
//
// Residual risk (deferred — not a contract here):
//   Same name@version string from two different registries (e.g., crates.io foo@1.0.0
//   vs alt-registry foo@1.0.0). canonical_path would be identical; antigen identities
//   would collide. This is ADR-017 Open Question 1 (documented limitation). No
//   separate contract is needed here — the deferred status lives in ADR-017 with its
//   own adoption-pressure trigger.
//
// Biological cognate (naturalist, A3): this failure class is ANTIGENIC DRIFT, not
// memory-waning. The drift-vs-waning audit message distinction is the live adversarial
// concern for A4+; see ATK-A3-010 for that contract.
//   - Memory-waning: claim-side decay. Target stable; titer wanes. Maps to ADR-016
//     `verified_at` + re-attestation. User action: re-run same witness.
//   - Antigenic drift: target-side movement. Antigen shape changed; claim is
//     stable-but-stale-by-reference. User action: re-recognize vulnerability shape
//     before re-running witness.
//
// The test body is kept (panic) rather than deleted — a dormant contract is better
// documentation than a gap in the numbering. The ignore reason explains the disposition.
// ============================================================================

#[test]
#[ignore = "ATK-A3-009 disposed: attack surface eliminated by ADR-017 name@version format \
             (foo@1.0.0 ≠ foo@1.1.0 at identity level); residual alt-registry collision \
             deferred to ADR-017 Open Question 1; drift-vs-waning angle in ATK-A3-010"]
fn atk_a3_009_multi_version_fingerprint_divergence_addressed_without_revalidation() {
    // Original contract: two antigen declarations with same (canonical_path, type_name)
    // but different fingerprints — immunity against one silently addresses the other.
    // This scenario cannot arise under the ratified name@version canonical_path format.
    //
    // Kept as documentation. See comment block above for full disposition.
    panic!("ATK-A3-009 disposed — see comment block");
}

// ============================================================================
// ATK-A3-010: audit message category error — drift language vs waning language
//
// A4+ pre-implementation contract. Gated on A4-A5 behavioral re-validation tier.
//
// Two failure modes produce "witness may be stale" audit hints, but require
// DIFFERENT user actions:
//
//   Memory-waning (ADR-016, `verified_at` staleness):
//     The antigen fingerprint is UNCHANGED. The user's witness worked when
//     validated; time has passed. The user action: re-run the same witness.
//     Audit message shape: "witness verified_at <date>; re-validate against
//     current codebase."
//
//   Antigenic drift (this contract):
//     The antigen fingerprint has CHANGED (version divergence or deliberate
//     antigen amendment). The user's witness was correct for the old fingerprint
//     shape; it may not be correct for the new shape. The user action: re-examine
//     whether the witness covers the new fingerprint — re-running the same witness
//     is not sufficient.
//     Audit message shape: "antigen fingerprint has changed since witness validation;
//     re-recognize vulnerability shape before re-running witness."
//
// The adversarial catch: an A4 implementation that emits memory-waning language
// ("re-validate your witness") for an antigenic-drift scenario gives the user
// a plausible-but-wrong action. The user re-runs the witness, it passes (because
// the witness was written correctly for the OLD fingerprint and still compiles),
// and believes the immunity is restored — but the fingerprint mismatch between
// old witness scope and new antigen shape remains unaddressed. Silent wrong answer
// with confirmed green output.
//
// Contract: when the audit detects fingerprint divergence (two versions of same
// antigen with different fingerprints + an immunity validated against one version),
// the audit diagnostic MUST use drift language ("antigen shape changed" or
// equivalent), NOT memory-waning language ("re-validate witness"). The distinction
// MUST appear in the AuditHint text so callers can key off it programmatically.
//
// Detection: fixture with two fingerprint-divergent declarations + an immunity
// whose witness was validated against the old fingerprint. Audit output is
// string-matched for drift-specific language. A test that asserts the ABSENCE
// of memory-waning language ("re-validate", "stale witness") in the drift case.
//
// Status: #[ignore] until A4-A5 behavioral re-validation tier lands.
// ============================================================================

#[test]
#[ignore = "A4+ pre-implementation contract; audit drift-vs-waning message category \
             error — remove ignore when A4-A5 behavioral re-validation tier ships"]
fn atk_a3_010_antigenic_drift_audit_message_must_not_use_memory_waning_language() {
    // Contract: a fingerprint-divergence scenario (ATK-A3-009 substrate) MUST
    // produce audit language that tells the user to RE-RECOGNIZE the vulnerability
    // shape, not merely to re-run their existing witness.
    //
    // The failure to catch: user re-runs witness, it passes (witness still compiles
    // against new fingerprint), audit shows green, fingerprint mismatch silently
    // remains. This is the antigenic-drift silent-green: a plausible passing action
    // that does not address the actual structural change.
    //
    // TODO(adversarial): implement when A4-A5 ships behavioral re-validation +
    // AuditHint text differentiation. Requires:
    //   - ATK-A3-009 fixture (fingerprint divergence + immunity against old version)
    //   - Audit AuditHint carrying a category field (Drift vs Waning) or equivalent
    //     textual distinction the test can assert against.
    panic!("A4+ pre-implementation contract — antigenic-drift audit message category");
}

// ============================================================================
// ATK-A3-011: cross-crate witness tier overstated — theatrical dependency witness
//             reported as ExecutionVerified when consuming workspace cannot run it
//
// A4+ pre-implementation contract. Gated on A4-A5 behavioral re-validation tier
// and cross-crate witness resolution (A3 D3+).
//
// Attack vector: a dependency crate ships `#[immune(X, witness = test::stub_test)]`
// where `stub_test` technically passes (resolves, compiles, produces no assertion
// failure) but does not exercise antigen X's specific failure mode. The consuming
// workspace calls `cargo antigen audit`; the audit resolves the witness to the
// function in the dependency's source, reports `ExecutionVerified` tier, and
// marks the site as addressed.
//
// The gap: the consuming workspace CANNOT execute the dependency's test. It can
// confirm the function exists (source walk) and compiles (cargo check), but it
// cannot run it. Reporting `ExecutionVerified` when the test hasn't been executed
// in the consumer's CI violates ADR-005 Amendment 3 audit-tier-honesty.
//
// The correct tier for cross-crate witnesses where the function lives in a
// dependency's test suite: `ExternalUnvalidated` — same as `witness = kani::...`
// when kani isn't in the local CI pipeline. The consumer asserts existence, not
// execution.
//
// The theatrical-witness amplification: a dependency that ships a witness designed
// to pass without exercising the failure mode (e.g., `fn stub_test() {}`) will
// receive `ExecutionVerified` in every consuming workspace's audit. The failure
// mode is invisible at every layer because the audit reports tier from source
// resolution alone.
//
// Contract: `cargo antigen audit` MUST report `ExternalUnvalidated` (not
// `ExecutionVerified`) for any witness whose function definition lives in a
// dependency crate's test suite, unless the consuming workspace has a mechanism
// to execute that test locally (e.g., it is a dev-dependency re-run). The tier
// distinction MUST appear in the AuditReport output and be grounded in whether
// the consuming workspace's CI pipeline actually ran the witness.
//
// ADR-005 Amendment 3 amendment candidate: the cross-crate witness tier rule
// should be named explicitly in the amendment text before v0.1.0-rc.1 ships.
//
// Status: #[ignore] until A4-A5 cross-crate witness resolution + tier computation
// lands.
// ============================================================================

#[test]
#[ignore = "A4+ pre-implementation contract; cross-crate witness reported as \
             ExecutionVerified when consumer cannot run it — theatrical-witness \
             attack vector; should be ExternalUnvalidated; remove ignore when \
             A4-A5 cross-crate witness tier computation lands"]
fn atk_a3_011_cross_crate_witness_reported_executed_when_consumer_cannot_run_it() {
    // Contract: a ScanReport containing an Immunity whose witness function lives
    // in a dependency crate's test suite MUST produce WitnessTier::ExternalUnvalidated
    // in the audit output, not WitnessTier::ExecutionVerified.
    //
    // The fixture requires:
    //   - A dependency crate with `#[immune(X, witness = test::stub_test)]` where
    //     stub_test is defined in the dependency's test suite.
    //   - A consuming workspace that scans --include-deps and audits.
    //   - Assertion: the Immunity's WitnessTier in the audit output is ExternalUnvalidated.
    //   - Anti-assertion: the tier is NOT ExecutionVerified (the consuming workspace
    //     did not execute the test).
    //
    // TODO(adversarial): implement when A4-A5 cross-crate witness resolution lands.
    // See also: campsite `20260510-multi-component-threat-model.md` §C6 for full attack.
    panic!("A4+ pre-implementation contract — cross-crate witness tier");
}

// ============================================================================
// ATK-A3-012: proc-macro generated immunity lacks source annotation —
//             injected claims indistinguishable from hand-written claims
//
// A4+ pre-implementation contract. Gated on ADR-014 (#[antigen_generates]) landing.
//
// Attack vector: a compromised or malicious proc-macro dependency generates
// `#[immune(X, witness = test::stub)]` declarations on items the user did not
// intend to mark as immune. The proc-macro runs at compile time; its output is
// syntactically indistinguishable from a developer-written `#[immune]` attribute.
// `cargo antigen scan` collects the generated Immunity records. Audit reports the
// sites as addressed. Real vulnerability sites are suppressed.
//
// The gap: the current ScanReport does not record the *source* of each declaration
// (hand-written vs. `#[antigen_generates]`-produced vs. fingerprint-engine-inferred).
// A consumer reading the audit output cannot distinguish a developer's deliberate
// `#[immune]` from a proc-macro's injected immunity claim.
//
// This is structurally different from the C6 supply-chain attack (which targets
// a crate's published declarations). The proc-macro injection attack operates
// inside the user's own build pipeline — the compromised macro is a build
// dependency, and its output appears in the user's own workspace scan.
//
// Contract: `cargo antigen scan` MUST record a `declaration_source` field on
// Immunity (and Presentation) records indicating how the declaration was produced:
//   - `HandWritten` — the attribute appears in source as written by a developer
//   - `GeneratedByMacro(macro_name)` — produced by an `#[antigen_generates]` invocation
//   - `FingerprintInferred` — inferred by the fingerprint engine, no source attribute
//
// `cargo antigen audit` MUST surface `GeneratedByMacro` source in its output,
// distinguishing generated immunity claims from hand-written ones. Consumers
// can apply appropriate scrutiny to macro-generated claims.
//
// Status: #[ignore] until ADR-014 (`#[antigen_generates]`) lands and
// ScanReport gains declaration_source field.
// ============================================================================

#[test]
#[ignore = "A4+ pre-implementation contract; proc-macro generated immunity has no \
             source annotation — injected claims indistinguishable from hand-written; \
             ScanReport needs declaration_source field; remove ignore when ADR-014 \
             (#[antigen_generates]) lands"]
fn atk_a3_012_proc_macro_generated_immunity_lacks_source_annotation() {
    // Contract: an Immunity produced by an `#[antigen_generates]` invocation MUST
    // carry declaration_source = GeneratedByMacro("macro_name") in the ScanReport.
    // An Immunity written directly by a developer MUST carry declaration_source = HandWritten.
    // The audit output MUST distinguish the two.
    //
    // The adversarial fixture requires:
    //   - A proc-macro that generates `#[immune(X)]` on an item via #[antigen_generates].
    //   - A scan of the workspace containing the generated attribute.
    //   - Assertion: the Immunity record has declaration_source = GeneratedByMacro.
    //   - Anti-assertion: a hand-written `#[immune(X)]` on the same item type produces
    //     declaration_source = HandWritten, not GeneratedByMacro.
    //
    // TODO(adversarial): implement when ADR-014 lands and scan.rs tracks declaration source.
    // See also: campsite `20260510-multi-component-threat-model.md` §C1 for full attack.
    panic!("A4+ pre-implementation contract — proc-macro generated immunity source");
}

// ============================================================================
// ATK-A3-013: diamond ProvenanceEntry set-union loses path structure —
//             A4-A5 witness re-validation uses wrong ancestor's witness
//
// A4+ pre-implementation contract. Gated on A4-A5 behavioral re-validation tier.
//
// Background: ADR-018 Option C (ProvenanceEntry with canonical_path) is ratified.
// Diamond dedup merges `inherited_from` by set-union of ProvenanceEntry tuples.
// This correctly deduplicates presentations. But the merge loses path structure:
// which diamond path contributed which ancestor is not preserved.
//
// Attack scenario:
//
//   Antigen hierarchy:
//     GrandParent (canonical_path: Some("gp-crate@1.0.0"))
//     ├── Parent1 (canonical_path: Some("p1-crate@1.0.0"))
//     │     witness for GrandParent: test::thorough_test  (covers all cases)
//     └── Parent2 (canonical_path: Some("p2-crate@1.0.0"))
//           witness for GrandParent: test::stub_test       (passes, covers nothing)
//     └── Child (inherits from both Parent1 and Parent2)
//
//   After diamond dedup:
//     Child.inherited_from = Some([
//       ProvenanceEntry { antigen_type: "GrandParent", canonical_path: Some("gp-crate@1.0.0") }
//     ])
//
//   Both Parent1's thorough_test and Parent2's stub_test are ancestors of the
//   GrandParent ProvenanceEntry. A4-A5 behavioral re-validation must evaluate
//   which witness applies to Child's inherited GrandParent presentation.
//
// The gap: if A4-A5 implementation arbitrarily picks one ancestor's witness
// (e.g., first in lineage_edges traversal order), it may pick stub_test.
// Child's GrandParent immunity is reported as re-validated against stub_test —
// the weaker witness. The thorough_test path is silently discarded.
//
// The correct behavior: A4-A5 witness re-validation against diamond descendants
// MUST traverse the full `lineage_edges` substrate to reconstruct path structure,
// not rely solely on the merged ProvenanceEntry set. Both ancestor witnesses must
// be evaluated; the weaker tier (ExternalUnvalidated < ExecutionVerified) governs.
//
// Contract: for a diamond inheritance scenario where two paths to the same
// ProvenanceEntry carry witnesses of different tiers, the audit MUST report the
// LOWER (more conservative) tier, not the higher one. The implementation MUST
// use `lineage_edges` for path reconstruction, not just `inherited_from`.
//
// Status: #[ignore] until A4-A5 behavioral re-validation tier lands.
// ============================================================================

#[test]
#[ignore = "A4+ pre-implementation contract; diamond ProvenanceEntry set-union loses \
             path structure — A4-A5 re-validation may pick weaker ancestor witness, \
             must use lineage_edges not inherited_from alone; remove ignore when \
             A4-A5 behavioral re-validation tier lands"]
fn atk_a3_013_diamond_provenance_set_union_loses_path_witness_structure() {
    // Contract: diamond inheritance where two paths to the same ProvenanceEntry
    // carry different witnesses — the audit MUST evaluate all ancestor witnesses
    // and report the most conservative (weakest) tier, not arbitrarily pick one.
    //
    // Fixture requires:
    //   - GrandParent antigen with two intermediate parents, each with a different
    //     witness (one thorough, one stub/theatrical).
    //   - Child inheriting from both parents (diamond shape).
    //   - After synthesis, Child's inherited_from has one ProvenanceEntry for GrandParent.
    //   - A4-A5 audit: assert the reported tier is the weaker witness's tier, NOT
    //     the stronger one. Assert that lineage_edges traversal is used, not
    //     inherited_from alone.
    //
    // TODO(adversarial): implement when A4-A5 behavioral re-validation lands.
    // See also: campsite `20260510-multi-component-threat-model.md` §C5 for full attack.
    panic!("A4+ pre-implementation contract — diamond ProvenanceEntry path loss");
}

// ============================================================================
// ATK-A3-014: reference tier annotation absent — LLM hallucinated references
//             indistinguishable from validated references in audit output
//
// A4+ pre-implementation contract. Gated on `cargo antigen audit` reference
// validation feature.
//
// Attack vector: LLM collaborators that help write antigen declarations
// hallucinate plausible-looking but nonexistent references. The hallucinations
// are calibrated to look real — RFC numbers in plausible ranges, blog URLs with
// correct domain and path structure, CVE IDs in correct formats. All reliably
// return 404 or redirect when fetched. The current v0.1 model stores references
// as opaque strings; audit does not validate them. Hallucinated and real references
// are structurally indistinguishable in the scan report.
//
// LLM double-trust amplification: the same LLM that hallucinated a reference
// will trust it in a future session (reading `references = ["url:https://..."]`
// and treating it as ground truth). The circular trust: LLM generates, then
// validates against its own generation. The structural vocabulary makes this
// trust feel warranted — the reference is "in the code."
//
// The shared-reference-cluster risk: multiple related antigens reference the same
// external source. When that source is a hallucination (or later compromised via
// domain expiry/squatting), a single resolution failure undermines the rationale
// for a whole cluster of failure classes simultaneously.
//
// Contract: `cargo antigen audit` MUST produce a tier annotation per reference
// entry in the `references = [...]` field:
//   - `ValidatedReference` — URL resolves with expected content type (200 OK, HTML)
//   - `UnvalidatedReference` — URL not checked (audit run without --validate-refs)
//   - `DeadReference` — URL returns 4xx/5xx or does not parse as a valid URL
//
// The audit MUST surface `DeadReference` as a warning. Antigens with only
// `DeadReference` entries have no validated lived-context grounding.
//
// Shared-reference cluster detection: when N antigens share the same external
// reference URL, audit SHOULD surface the cluster size as metadata — "N antigens
// share this reference; single-point failure risk."
//
// Status: #[ignore] until `cargo antigen audit --validate-refs` feature lands.
// ============================================================================

#[test]
#[ignore = "A4+ pre-implementation contract; reference tier annotation absent — \
             LLM hallucinated references (reliably 404) indistinguishable from \
             real ones; audit needs ValidatedReference/DeadReference tier per entry; \
             remove ignore when cargo antigen audit --validate-refs lands"]
fn atk_a3_014_hallucinated_references_indistinguishable_from_validated() {
    // Contract: an antigen declaration with a `references = [...]` entry that
    // returns 404 when fetched MUST produce a DeadReference tier annotation in
    // the audit output. A real reference that resolves MUST produce ValidatedReference.
    // The two MUST be distinguishable in audit output.
    //
    // Fixture requires:
    //   - An antigen with a `references = ["url:https://invalid.example.invalid/fake"]`
    //     entry that reliably does not resolve.
    //   - A scan + audit run with --validate-refs flag.
    //   - Assertion: the audit output for this antigen includes a DeadReference tier
    //     annotation for the entry.
    //   - Shared-cluster fixture: N antigens sharing one dead reference URL → audit
    //     surfaces cluster size N in its output for that reference.
    //
    // TODO(adversarial): implement when cargo antigen audit gains --validate-refs.
    // See also: campsite `20260510-multi-component-threat-model.md` §C4 for full attack.
    panic!("A4+ pre-implementation contract — reference tier annotation");
}

// ============================================================================
// ATK-A3-015: seam-tier consistency witness — oracle built from wrong implementation
//
// Encounter-candidate seam-tier framing (tambear math-researcher, single instance).
// Gated: seam-tier antigen vocabulary ships (post-A3, taxonomy ADR territory).
//
// Background: seam-tier antigens (composition-time failures between two correct
// implementations — e.g., ExpKernelState's `(1 + expm1_r) << k` diverging from
// standalone exp.rs at the reconstruction seam) use cross-implementation
// consistency tests as their canonical witness shape. This is structurally
// different from type-tier witnesses (phantom-type, compile-time) and introduces
// witness-specific attack surfaces type-system witnesses don't have.
//
// Attack vector: the consistency test compares impl_A(x) against impl_B(x).
// If impl_A is chosen as the oracle and is itself wrong, the test confirms
// wrong-matches-wrong and passes. WitnessTier reports ExecutionVerified.
// Audit shows immunity. The seam-tier failure is not covered.
//
// Why this doesn't apply to type-system witnesses: a phantom-type witness
// fails at compile time if the type contract isn't satisfied — it cannot
// confirm a wrong answer. A consistency test can confirm consistent-but-wrong
// if both implementations share the same systematic error.
//
// The specific mathematical shape: Taylor-vs-Remez implementations of exp()
// may agree at easy inputs (small |x|, integer-adjacent values) and diverge
// at precision boundaries (catastrophic cancellation range, denormal inputs).
// A consistency test that samples from easy inputs confirms agreement at the
// wrong domain; the seam failure lives in the hard domain.
//
// Contract: a seam-tier antigen's consistency witness MUST specify the
// oracle implementation explicitly and provide rationale for why that
// implementation is authoritative. Audit MUST surface "oracle: impl_A" as
// a field in the witness record. An undifferentiated "both agree" witness
// (no oracle designation) MUST produce ExternalUnvalidated tier, not
// ExecutionVerified — the consistency proves agreement, not correctness.
//
// Status: #[ignore] gated on seam-tier vocabulary ADR.
// ============================================================================

#[test]
#[ignore = "Encounter-candidate pre-impl contract; seam-tier consistency witness \
             oracle built from wrong implementation — consistent-but-wrong passes \
             audit; oracle must be designated and rationale-grounded; remove ignore \
             when seam-tier antigen vocabulary ADR ships"]
fn atk_a3_015_seam_tier_consistency_witness_wrong_oracle_passes_audit() {
    // Contract: a cross-implementation consistency test where both implementations
    // share a systematic error MUST NOT produce ExecutionVerified tier. The audit
    // MUST require oracle designation; undifferentiated consistency (no oracle)
    // MUST yield ExternalUnvalidated.
    //
    // Fixture: two exp() implementations that agree on easy inputs but both
    // carry the same systematic bias on hard inputs. Consistency test at easy
    // inputs passes. Audit reports ExecutionVerified. The seam-tier failure
    // (precision boundary divergence) is uncovered.
    //
    // Anti-fixture: an explicitly designated oracle implementation with rationale
    // ("reference implementation per IEEE 754 conformance suite") produces
    // ExecutionVerified because the oracle claim is grounded.
    //
    // TODO(adversarial): implement when seam-tier antigen vocabulary ships.
    // See deferred-substrate V16 §seam-tier and campsite 20260510-encounters-attack-surface.md.
    panic!("Encounter-candidate pre-impl contract — seam-tier oracle wrong");
}

// ============================================================================
// ATK-A3-016: seam-tier antigen declared at wrong seam —
//             witness probes composition point that doesn't cover the failure
//
// Encounter-candidate seam-tier framing. Gated: seam-tier vocabulary ships.
//
// Attack vector (the ATK-A3-007 wrong-trust-layer pattern at the declaration
// level): a seam-tier antigen is declared at a seam that LOOKS compositional
// but doesn't actually cover where the failure lives. The consistency witness
// probes impl_A ∘ impl_B at the declared seam; the actual failure lives at
// impl_B ∘ impl_A (different composition order) or at impl_A ∘ impl_B ∘ impl_C
// (N-ary composition not covered by binary test).
//
// Concrete mathematical instance: ExpKernelState's reconstruction seam is
// `(1 + expm1_r) << k`. A consistency test that probes `exp.rs` API output
// vs. `ExpKernelState.reconstruct()` output covers the API-level seam.
// But if the actual divergence is inside the kernel's intermediate reduction
// step (not at the public API), the consistency test at the API seam is blind
// to it — it sees the final outputs, which may agree due to cancellation of
// two errors in opposite directions.
//
// This is distinct from ATK-A3-015 (wrong oracle): here the oracle is correct
// and the test is honest, but it's testing the wrong composition boundary.
// The antigen declaration is at the wrong seam.
//
// Contract: seam-tier antigen declarations MUST specify the composition
// boundary explicitly — which two (or N) implementations are being composed,
// at what interface, in what order. The consistency witness MUST probe the
// SAME composition boundary named in the antigen declaration, not a higher-
// or lower-level abstraction of it. Audit MUST verify that the witness's
// composition point matches the antigen's declared seam.
//
// Status: #[ignore] gated on seam-tier vocabulary ADR.
// ============================================================================

#[test]
#[ignore = "Encounter-candidate pre-impl contract; seam-tier antigen at wrong seam — \
             witness covers different composition boundary than where failure lives; \
             wrong-trust-layer pattern at declaration level; remove ignore when \
             seam-tier antigen vocabulary ADR ships"]
fn atk_a3_016_seam_tier_antigen_declared_at_wrong_composition_boundary() {
    // Contract: a seam-tier antigen where the consistency witness probes a
    // different composition boundary than the failure-class lives at MUST be
    // surfaced by audit as "seam mismatch: antigen declares boundary X; witness
    // probes boundary Y." The immunity claim MUST NOT be honored across mismatched
    // seams.
    //
    // Fixture: antigen declares seam at `exp_kernel ∘ exp_api`; witness tests
    // `exp_api` output against reference. The kernel's internal intermediate
    // seam (where divergence actually occurs) is never probed.
    //
    // TODO(adversarial): implement when seam-tier vocabulary ships.
    panic!("Encounter-candidate pre-impl contract — seam-tier wrong composition boundary");
}

// ============================================================================
// ATK-A3-017: type-tier / seam-tier mis-classification to evade witness strength
//
// Encounter-candidate seam-tier framing. Gated: seam-tier vocabulary ships.
//
// Attack vector: the two antigen classes carry different witness strength
// guarantees. Type-tier failures caught by phantom-type witnesses have compile-
// time enforcement — the type system is the verifier; the witness either
// satisfies the type contract everywhere or nowhere. Seam-tier failures caught
// by consistency tests have runtime, input-coverage-dependent enforcement.
//
// Mis-classification as tier downgrade:
//   A type-tier failure (catchable at compile time via phantom-type witness)
//   is mis-declared as seam-tier. The consistency-test witness required for
//   seam-tier is weaker: runtime, input-selected, oracle-dependent. The team
//   has traded a compile-time guarantee for a test-coverage-dependent one.
//   No audit signal fires — the antigen is classified, the witness matches
//   the classification, audit passes.
//
// Mis-classification as false upgrade (reverse direction):
//   A seam-tier failure (lives at composition boundary; no type-level encoding)
//   is declared as type-tier with a phantom-type witness. The phantom-type
//   witness satisfies the type constraint without covering the composition
//   boundary — theatrical at the seam. The type system cannot express the
//   seam failure; the witness is structurally incapable of covering it.
//   This failure mode is HARDER to exploit maliciously (it's structurally
//   incoherent) but EASIER to do by mistake when a developer doesn't understand
//   the tier distinction.
//
// Contract: the audit MUST validate that the witness shape is CONSISTENT with
// the antigen's declared tier:
//   - type-tier antigen + consistency-test witness → audit warning: "type-tier
//     antigens should use compile-time witnesses; consistency test provides
//     weaker guarantees"
//   - seam-tier antigen + phantom-type witness → audit error: "phantom-type
//     witnesses cannot cover composition-boundary failures; witness is
//     structurally mismatched to antigen tier"
//
// Status: #[ignore] gated on seam-tier vocabulary ADR.
// ============================================================================

#[test]
#[ignore = "Encounter-candidate pre-impl contract; type-tier/seam-tier mis-classification \
             evades witness strength requirements — type-tier to seam-tier is silent \
             downgrade from compile-time to runtime guarantee; remove ignore when \
             seam-tier antigen vocabulary ADR ships"]
fn atk_a3_017_tier_misclassification_evades_witness_strength_requirement() {
    // Contract: audit MUST check witness shape consistency with antigen tier.
    // type-tier + consistency-test → warning (downgrade from compile-time guarantee).
    // seam-tier + phantom-type → error (structurally incapable of covering seam).
    //
    // Fixture A (downgrade): antigen declared type-tier; witness is a cross-
    // implementation consistency test. Audit emits warning citing weaker coverage.
    //
    // Fixture B (structural mismatch): antigen declared seam-tier; witness is
    // phantom-type. Audit emits error: phantom-type cannot cover composition boundary.
    //
    // TODO(adversarial): implement when seam-tier vocabulary ships.
    panic!("Encounter-candidate pre-impl contract — tier mis-classification witness strength");
}

// ============================================================================
// ATK-A3-018: retire-to-documentation as premature dismissal of real encounter-candidates
//
// Meta-level attack on the encounters discipline itself.
// Gated: encounters tier ratification (aristotle Phase 1-8, process.md Q7).
//
// Background: the encounters-proposal adds a third encounter disposition:
// retire-to-documentation — the encounter becomes usage docs / adoption guide
// rather than vocabulary extension. This is correct and necessary (not every
// pattern warrants vocabulary growth).
//
// Attack vector: retire-to-documentation is self-reported and requires no
// structural verification. An encounter-candidate that someone finds
// inconvenient to champion (requires difficult Phase 1-8, challenges existing
// vocabulary, has no obvious owner) can be retired-to-documentation with
// rationale "this is just a usage pattern" — even when it is a genuine
// vocabulary-extension candidate.
//
// The adversarial shape: the retire-to-documentation disposition provides
// legitimate cover for premature closure. It is structurally indistinguishable
// from correct retirement (the rationale can always be written plausibly).
// A pattern that should have promoted to V0+1 candidate is buried in usage
// docs; no second instance ever accumulates because the pattern-as-pattern
// is no longer watched.
//
// Note: this is not about bad faith. The failure mode can be fully honest —
// a developer genuinely believes "this is just a usage pattern" and retires
// it, when in fact the pattern has vocabulary-extension potential not yet
// visible from one instance. The premature retirement is a category error,
// not malice.
//
// Required guard (not currently in encounters-proposal):
//   1. Retire-to-documentation requires a second opinion from a team member
//      who did NOT file the original encounter — prevents the original filer
//      from self-retiring without external check.
//   2. Retired encounters are not deleted; they are marked retired-by(name)
//      with rationale AND held in the register for one additional "revisit
//      window" (e.g., one A-sweep). If a new instance of the pattern surfaces
//      during the revisit window, the retirement is reversed automatically.
//   3. The retirement rationale must specify the usage-docs artifact the
//      encounter produces — "retire to docs/usage-patterns.md §seam-tier"
//      not just "retire to documentation" in the abstract. An encounter with
//      no concrete retirement artifact has not actually been retired.
//
// Contract (process-level, not code-level): encounters-proposal ratification
// MUST include the three guards above before retire-to-documentation disposition
// is operationally enabled. Without them, the third disposition is an evasion
// surface for real candidates.
//
// Status: #[ignore] gated on encounters-tier ratification.
// ============================================================================

#[test]
#[ignore = "Meta-level ATK on encounters discipline; retire-to-documentation lacks \
             structural guard against premature dismissal — three guards required: \
             second-opinion, revisit-window, concrete-artifact; remove ignore when \
             encounters-tier ratification (aristotle Phase 1-8) addresses this"]
fn atk_a3_018_retire_to_documentation_is_evasion_surface_without_guards() {
    // This is a process-level contract, not a code contract. It cannot be
    // activated by an implementation change alone — it activates when the
    // encounters-tier ratification document (process.md Q7 sub-section or
    // encounters.md) specifies the three guards:
    //   1. Second-opinion requirement for retirement decisions
    //   2. Revisit-window during which new instances reverse retirement
    //   3. Concrete retirement artifact required (not abstract "to docs")
    //
    // The test body is intentionally inert — the "test" is the process
    // discipline, not a code assertion. When the three guards land in the
    // ratified encounters-tier document, this contract is closed.
    //
    // TODO(adversarial): close when encounters ratification addresses guards.
    // See campsite `20260510-encounters-attack-surface.md` §Failure mode 5 for
    // the full lifecycle analysis.
    panic!("Meta-level process contract — retire-to-documentation guards");
}

// NOTE: ATK-A3-019 (audit human-readable FormalProof/Reachability tier label)
// lived here historically but had to shell out to the binary via a NESTED
// `cargo run --bin cargo-antigen` (this `antigen` crate does not depend on
// `cargo-antigen`, so `CARGO_BIN_EXE_cargo-antigen` is not injected here). That
// nested `cargo run` raced the workspace target lock under `cargo test
// --workspace` (cold-run flake). It now lives at
// `cargo-antigen/tests/atk_a3_audit_tier_label_cli.rs`, where it uses the
// lock-free `env!("CARGO_BIN_EXE_cargo-antigen")` idiom shared by its ~20
// binary-shelling siblings. Same assertion, same corpus, zero behavior change.
