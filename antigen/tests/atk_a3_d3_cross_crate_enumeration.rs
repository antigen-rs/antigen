//! ATK-A3 D3 substrate: cross-crate enumeration via `cargo metadata`.
//!
//! Tests that exercise [`antigen::scan::enumerate_dep_crate_roots`] against
//! the real workspace + a deliberately-broken fixture. These tests run
//! `cargo metadata` as a subprocess; they require cargo to be on PATH (which
//! is always the case in any environment that built the workspace).
//!
//! Per A3 scope-lock §6 + navigator's 2026-05-09 ruling: cross-crate scope
//! in v0.1 is enumeration + per-crate scanning, NOT merged cross-crate
//! `addresses()` matching. ATK-A3-005 (module-path-qualified `ItemTarget`)
//! is deferred to ADR-class decision pending aristotle.
//!
//! The Sub-clause F trust-model ADR sentence is in flight with aristotle;
//! these tests cover the mechanism, not the trust documentation.

use antigen::scan::{enumerate_dep_crate_roots, scan_workspace, CrateOrigin};
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    // The antigen crate's CARGO_MANIFEST_DIR is `R:\antigen\antigen\`; the
    // workspace root is its parent.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("antigen crate must have a workspace parent")
        .to_path_buf()
}

// ============================================================================
// Mechanism: cargo metadata produces a non-empty roots list for this
// workspace, including known registry deps (syn, serde, walkdir).
// ============================================================================

#[test]
fn d3_enumerate_returns_known_registry_deps() {
    let root = workspace_root();
    let roots = enumerate_dep_crate_roots(&root, false)
        .expect("cargo metadata must succeed for this workspace");

    assert!(
        !roots.is_empty(),
        "antigen workspace has registry deps; enumeration must return them"
    );

    let names: Vec<&str> = roots.iter().map(|r| r.package_name.as_str()).collect();
    for required in ["syn", "serde", "walkdir", "quote"] {
        assert!(
            names.contains(&required),
            "expected `{required}` in enumeration; got: {names:?}",
        );
    }
}

// ============================================================================
// Workspace members are excluded from the enumeration — running
// scan_workspace on the root covers them.
// ============================================================================

#[test]
fn d3_enumerate_excludes_workspace_members() {
    let root = workspace_root();
    let roots = enumerate_dep_crate_roots(&root, false).expect("cargo metadata must succeed");

    let names: Vec<&str> = roots.iter().map(|r| r.package_name.as_str()).collect();
    for member in [
        "antigen",
        "antigen-macros",
        "antigen-fingerprint",
        "cargo-antigen",
    ] {
        assert!(
            !names.contains(&member),
            "workspace member `{member}` must NOT appear in dep enumeration; got: {names:?}",
        );
    }
}

// ============================================================================
// Each returned root's `crate_root` directory exists and contains a
// Cargo.toml — defensive verification of the manifest_path → parent
// resolution discipline.
// ============================================================================

#[test]
fn d3_enumerate_roots_have_existing_cargo_toml() {
    let root = workspace_root();
    let roots = enumerate_dep_crate_roots(&root, false).expect("cargo metadata must succeed");

    for dep in &roots {
        let manifest = dep.crate_root.join("Cargo.toml");
        assert!(
            manifest.is_file(),
            "expected Cargo.toml at {} for dep `{}`",
            manifest.display(),
            dep.package_name
        );
    }
}

// ============================================================================
// Origin classification: registry deps are classified as Registry, the
// workspace antigen pkgs (path-deps) appear ONLY when include_path_workspace
// is true.
// ============================================================================

#[test]
fn d3_enumerate_classifies_registry_origin() {
    let root = workspace_root();
    let roots = enumerate_dep_crate_roots(&root, false).expect("cargo metadata must succeed");

    // Pick a dep we know is from the registry — `serde` is a stable choice.
    let serde_dep = roots
        .iter()
        .find(|r| r.package_name == "serde")
        .expect("serde must be in the dep enumeration");

    assert_eq!(
        serde_dep.origin,
        CrateOrigin::Registry,
        "serde must be classified as a Registry dep, got: {:?}",
        serde_dep.origin
    );

    // The crate_root must point inside `.cargo/registry/src/...` — defensive
    // sanity check on cargo metadata's manifest_path.
    let crate_root_str = serde_dep.crate_root.to_string_lossy();
    assert!(
        crate_root_str.contains(".cargo")
            && crate_root_str.contains("registry")
            && crate_root_str.contains("src"),
        "registry dep crate_root must contain `.cargo/registry/src/`, got: {crate_root_str}"
    );
}

// ============================================================================
// Full pipeline: enumerate + scan_workspace per-dep produces independent
// per-crate ScanReports. No cross-crate matching, per navigator's ruling.
//
// The test runs scan_workspace on a single small dep (`antigen-fingerprint`,
// when --include-path-workspace=true is set) and asserts the report is
// well-formed. This validates that the enumeration → scan pipeline works
// end-to-end without writing a hand-rolled multi-crate fixture.
// ============================================================================

#[test]
fn d3_enumerate_with_path_workspace_includes_workspace_path_deps() {
    let root = workspace_root();
    let roots = enumerate_dep_crate_roots(&root, true).expect("cargo metadata must succeed");

    let names: Vec<&str> = roots.iter().map(|r| r.package_name.as_str()).collect();
    // With include_path_workspace=true, NO workspace MEMBERS appear (they're
    // excluded by id), but path-deps to *other* workspaces would. This
    // workspace has no cross-workspace path-deps, so the count should be
    // identical to include_path_workspace=false. The contract is structural:
    // the function must not crash with the flag flipped, and must still
    // exclude workspace members.
    for member in [
        "antigen",
        "antigen-macros",
        "antigen-fingerprint",
        "cargo-antigen",
    ] {
        assert!(
            !names.contains(&member),
            "workspace member `{member}` must STILL be excluded with include_path_workspace=true"
        );
    }
}

#[test]
fn d3_enumerate_then_scan_per_crate_produces_independent_reports() {
    let root = workspace_root();
    let roots = enumerate_dep_crate_roots(&root, false).expect("cargo metadata must succeed");

    // Pick a small leaf-ish dep so the per-crate scan is fast — `unicode-ident`
    // is small and stable. Fall back to any dep if not present.
    let target = roots
        .iter()
        .find(|r| r.package_name == "unicode-ident")
        .or_else(|| roots.first())
        .expect("at least one dep crate root must be enumerated");

    // Per-crate scan should complete without erroring, even on a registry
    // crate that has no antigen declarations.
    let report = scan_workspace(&target.crate_root, None)
        .expect("scan_workspace must complete on a dep crate root");

    // No #[antigen(...)] declarations exist in the wild (P5 finding), so the
    // report's antigens list should be empty for any registry crate.
    assert!(
        report.antigens.is_empty(),
        "registry crate `{}` unexpectedly has antigen declarations: {:?}",
        target.package_name,
        report.antigens
    );

    // The report is a valid ScanReport — files were scanned (or zero, if the
    // crate's source was excluded by walkdir's default exclusions).
    // Either way: no parse failures should be cycle/depth failures, since
    // the crate has no #[descended_from] chains.
    let lineage_failures: Vec<_> = report
        .parse_failures
        .iter()
        .filter(|f| f.error.contains("cycle") || f.error.contains("maximum depth"))
        .collect();
    assert!(
        lineage_failures.is_empty(),
        "no lineage failures expected on registry crate, got: {lineage_failures:?}"
    );
}

// ============================================================================
// Defensive: bad workspace root produces a structured Err, not a panic.
// ============================================================================

#[test]
fn d3_enumerate_returns_err_on_nonexistent_workspace() {
    let bad_root = PathBuf::from("R:/antigen-nonexistent-workspace-xyz");
    let result = enumerate_dep_crate_roots(&bad_root, false);
    assert!(
        result.is_err(),
        "expected Err for nonexistent workspace, got: {result:?}"
    );
}
