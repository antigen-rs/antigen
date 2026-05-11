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
#[ignore = "A3 pre-implementation contract; canonical_path cross-crate orphan \
             collision — remove ignore when canonical_path lands on AntigenDeclaration"]
fn atk_a3_006_orphan_edge_canonical_path_false_resolution() {
    // Contract: a lineage edge pointing to `crate_b::Foo` must NOT be
    // resolved as non-orphaned because `crate_a::Foo` (different canonical
    // path) is in the scan. The orphan query must use canonical_path equality
    // when both sides carry it.
    //
    // TODO(adversarial): implement when canonical_path: Option<String> lands
    // on AntigenDeclaration, LineageEdge, and orphaned_lineage_edges() uses
    // canonical-path-aware comparison.
    panic!("A3 pre-implementation contract");
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
#[ignore = "A3 pre-implementation contract; ADR-018 duplicate edge dedup \
             diagnostic channel — remove ignore when propagation pass lands (A3 D1.5+)"]
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
// ADR-017 §"How the path is computed" intentionally collapses two versions of
// the same crate to the same canonical_path string (crate name only, no
// version). If `foo v1.0` declares `PanickingInDrop` with fingerprint F1, and
// `foo v2.0` (allowed in same dep tree under semver major splits) declares
// `PanickingInDrop` with a breaking fingerprint change F2, the scanner sees:
//
//   AntigenDeclaration { type_name: "PanickingInDrop", canonical_path: Some("foo") }  // v1
//   AntigenDeclaration { type_name: "PanickingInDrop", canonical_path: Some("foo") }  // v2
//
// A user's `#[immune(PanickingInDrop, witness = my_test)]` validated against F1
// addresses the v2 presentation (same canonical_path, same type_name) even
// though the witness was never re-validated against F2's changed contract.
// Audit shows "addressed" — silent wrong answer.
//
// The ADR says "audit MAY surface version-divergence as a diagnostic." The
// enforcement review flags this as too weak — "MAY" leaves a known silent
// failure mode without committed detection. The failure is:
//   1. Two AntigenDeclarations for the same canonical_path + type_name with
//      DIFFERENT fingerprints — version divergence.
//   2. An immunity that was validated against one version addresses the other.
//   3. No diagnostic emitted.
//
// Contract: when the scan report contains two antigen declarations with the
// same (canonical_path, type_name) but different fingerprints, the audit MUST
// emit a diagnostic naming the divergence and flagging associated immunities
// for re-attestation.
//
// Alternatively (if deferred): the limitation must be explicitly named as an
// out-of-scope known failure with an adoption-pressure trigger (ADR-006
// threshold: three independent instances). "MAY" is not sufficient — it
// implies optionality where the structural consequence is a silent wrong answer.
//
// Biological cognate (naturalist, A3): this is ANTIGENIC DRIFT, not memory-waning.
//   - Memory-waning: claim-side decay. Target stays stable; the claim ages (titer
//     wanes). Maps to ADR-016 `verified_at` + re-attestation ("re-validate witness").
//   - Antigenic drift: target-side movement. The antigen has changed shape; the
//     claim references a stable-but-now-stale fingerprint version. Maps to this
//     scenario ("re-recognize against new fingerprint").
//
// This distinction predicts different A4+ audit messages:
//   - Memory-waning  → "your witness is stale; re-validate"
//   - Antigenic drift → "the antigen you referenced has changed shape; re-recognize"
//
// The A4+ ATK to file: audit emitting memory-waning language for an antigenic-drift
// failure is a silent category error — the user re-runs the same witness (which passes)
// and believes the problem is solved, but the fingerprint mismatch remains unaddressed.
//
// See: adversarial enforcement-review campsite entry for full analysis.
// See: ATK-A3-010 for the audit-message category-error contract (A4+ scope).
//
// Status: #[ignore] until ADR-017 Enforcement is amended to commit to
// either detection or explicit deferral with a named trigger.
// ============================================================================

#[test]
#[ignore = "A3 pre-implementation contract; multi-version fingerprint divergence \
             silent wrong immunity — remove ignore when ADR-017 version-collision \
             enforcement is resolved (either detection or explicit deferral)"]
fn atk_a3_009_multi_version_fingerprint_divergence_addressed_without_revalidation() {
    // Contract: two antigen declarations for `foo::PanickingInDrop` (same
    // canonical_path = "foo", same type_name, different fingerprints from v1 vs
    // v2) with an immunity validated against only one version must NOT silently
    // show "addressed" for the other version's presentation without a diagnostic.
    //
    // The scenario requires a ScanReport with:
    //   - Two AntigenDeclarations: same canonical_path + type_name, different fingerprints
    //   - One Immunity referencing the antigen (canonical_path = "foo")
    //   - Two Presentations for the same antigen (from v1 and v2 scans)
    //
    // Expected (per corrected enforcement):
    //   - Either a parse_failure or audit diagnostic flagging version divergence
    //   - OR explicit out-of-scope limitation documented in ScanReport.known_limitations
    //     so callers can surface the gap rather than silently suppress it
    //
    // TODO(adversarial): implement when ADR-017 Enforcement §version-collision
    // is resolved to MUST or explicit-deferred-with-trigger.
    panic!("A3 pre-implementation contract");
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
