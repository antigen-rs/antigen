//! Multi-crate (member-aware) scan — integration coverage for the v0.3
//! cornerstone [`antigen::scan::scan_workspace_multi_crate`] and its
//! member-enumeration substrate
//! [`antigen::scan::enumerate_workspace_member_roots`].
//!
//! Two test surfaces:
//!
//! 1. **Real workspace** — `enumerate_workspace_member_roots` against the
//!    antigen workspace itself must return all five members with their
//!    canonical paths, and a member-aware scan must attribute each declaration
//!    to its owning member crate (distinct `canonical_path`s). This is the dual
//!    of `atk_a3_d3_cross_crate_enumeration.rs`, which covers the *dep*
//!    enumerator (members EXCLUDED); here members are the whole point.
//!
//! 2. **Synthetic 2-member workspace** built in a tempdir — proves the heart of
//!    the cornerstone: a `#[descended_from(Parent)]` in member A resolves to a
//!    `Parent` antigen declared in member B, with both endpoints stamped to
//!    their respective members. The scanner is a purely *syntactic* walker
//!    (`syn::parse_file` + attribute-name matching), so the fixture crates need
//!    no `antigen` dependency — the attributes are read as source text. This
//!    keeps the fixture hermetic (no network, no lock resolution): `cargo
//!    metadata --no-deps` resolves the workspace members from the manifests
//!    alone.
//!
//! These tests run `cargo metadata` as a subprocess; they require cargo on PATH
//! (always true in any environment that built the workspace).

use std::io::Write;
use std::path::{Path, PathBuf};

use antigen::scan::{enumerate_workspace_member_roots, scan_workspace_multi_crate};

fn workspace_root() -> PathBuf {
    // The antigen crate's CARGO_MANIFEST_DIR is `<workspace>/antigen/`; the
    // workspace root is its parent.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("antigen crate must have a workspace parent")
        .to_path_buf()
}

// ============================================================================
// Real workspace: member enumeration returns every member (the dual of dep
// enumeration, which excludes them).
// ============================================================================

#[test]
fn enumerate_returns_all_workspace_members() {
    let root = workspace_root();
    let members = enumerate_workspace_member_roots(&root)
        .expect("cargo metadata --no-deps must succeed for this workspace");

    let names: Vec<&str> = members.iter().map(|m| m.package_name.as_str()).collect();
    for expected in [
        "antigen",
        "antigen-macros",
        "antigen-fingerprint",
        "antigen-attestation",
        "cargo-antigen",
    ] {
        assert!(
            names.contains(&expected),
            "expected member `{expected}` in enumeration; got: {names:?}",
        );
    }
    assert_eq!(
        members.len(),
        5,
        "antigen workspace has exactly five members; got: {names:?}",
    );
}

#[test]
fn enumerate_members_have_existing_cargo_toml_and_canonical_path() {
    let root = workspace_root();
    let members = enumerate_workspace_member_roots(&root).expect("cargo metadata must succeed");

    for m in &members {
        let manifest = m.crate_root.join("Cargo.toml");
        assert!(
            manifest.is_file(),
            "expected Cargo.toml at {} for member `{}`",
            manifest.display(),
            m.package_name
        );
        // Canonical path is `<name>@<version>` and is non-empty on both sides.
        let cp = m.canonical_path();
        assert!(
            cp.starts_with(&m.package_name) && cp.contains('@') && !cp.ends_with('@'),
            "member canonical path must be `<name>@<version>`, got: {cp}"
        );
    }
}

#[test]
fn member_aware_scan_attributes_declarations_to_their_member() {
    let root = workspace_root();
    let report = scan_workspace_multi_crate(&root).expect("member-aware scan of antigen workspace");

    // Every antigen declaration must carry a member canonical_path (the flat
    // scan would leave these `None`). The stdlib antigens live in the `antigen`
    // crate, so at least one must be stamped `antigen@<version>`.
    assert!(
        !report.antigens.is_empty(),
        "antigen workspace declares stdlib antigens; member-aware scan must find them"
    );
    assert!(
        report.antigens.iter().all(|a| a.canonical_path.is_some()),
        "member-aware scan must stamp EVERY antigen with its member canonical_path; \
         unstamped: {:?}",
        report
            .antigens
            .iter()
            .filter(|a| a.canonical_path.is_none())
            .map(|a| &a.type_name)
            .collect::<Vec<_>>()
    );
    let has_antigen_member = report.antigens.iter().any(|a| {
        a.canonical_path
            .as_deref()
            .is_some_and(|c| c.starts_with("antigen@"))
    });
    assert!(
        has_antigen_member,
        "stdlib antigens must be attributed to the `antigen@<version>` member"
    );
}

#[test]
fn member_aware_scan_records_complete_coverage() {
    let root = workspace_root();
    let report = scan_workspace_multi_crate(&root).expect("member-aware scan of antigen workspace");

    let coverage = report
        .scan_coverage
        .as_ref()
        .expect("member-aware scan must populate scan_coverage");

    // All five members enumerated AND scanned ⇒ no ignorance frontier.
    assert_eq!(
        coverage.enumerated_members.len(),
        5,
        "all five workspace members must be enumerated; got: {:?}",
        coverage.enumerated_members
    );
    assert!(
        coverage.is_complete(),
        "a full --workspace scan of the antigen workspace must have complete coverage \
         (empty ignorance frontier); unscanned: {:?}",
        coverage.unscanned_members()
    );
    assert_eq!(
        coverage.enumerated_members, coverage.scanned_members,
        "enumerated and scanned member sets must match for a complete scan"
    );
}

// ============================================================================
// Synthetic 2-member workspace: cross-member `#[descended_from]` resolution.
// ============================================================================

/// Build a hermetic 2-member Cargo workspace in `dir`:
///   - member `mc_parent` declares antigen `SharedParent`
///   - member `mc_child` declares `Descendant` with
///     `#[descended_from(SharedParent)]` and `#[presents(...)]` is irrelevant
///
/// No `antigen` dependency — the scanner reads attributes syntactically.
fn write_two_member_workspace(dir: &Path) {
    // Workspace manifest.
    let ws_manifest = dir.join("Cargo.toml");
    let mut f = std::fs::File::create(&ws_manifest).expect("create workspace Cargo.toml");
    write!(
        f,
        r#"[workspace]
resolver = "2"
members = ["mc_parent", "mc_child"]
"#
    )
    .expect("write workspace manifest");
    drop(f);

    // Parent member.
    let parent_dir = dir.join("mc_parent");
    std::fs::create_dir_all(parent_dir.join("src")).expect("mkdir mc_parent/src");
    let mut pm = std::fs::File::create(parent_dir.join("Cargo.toml")).expect("parent manifest");
    write!(
        pm,
        r#"[package]
name = "mc_parent"
version = "0.1.0"
edition = "2021"
"#
    )
    .expect("write parent manifest");
    drop(pm);
    let mut plib =
        std::fs::File::create(parent_dir.join("src").join("lib.rs")).expect("parent lib");
    write!(
        plib,
        r#"//! parent member
#[antigen(
    name = "shared-parent",
    summary = "a parent antigen declared in mc_parent",
)]
pub struct SharedParent;

#[presents(SharedParent)]
pub fn parent_vulnerable_site() {{}}
"#
    )
    .expect("write parent lib.rs");
    drop(plib);

    // Child member: descends from the parent in the OTHER crate.
    let child_dir = dir.join("mc_child");
    std::fs::create_dir_all(child_dir.join("src")).expect("mkdir mc_child/src");
    let mut cm = std::fs::File::create(child_dir.join("Cargo.toml")).expect("child manifest");
    write!(
        cm,
        r#"[package]
name = "mc_child"
version = "0.2.0"
edition = "2021"
"#
    )
    .expect("write child manifest");
    drop(cm);
    let mut clib = std::fs::File::create(child_dir.join("src").join("lib.rs")).expect("child lib");
    write!(
        clib,
        r#"//! child member
#[antigen(
    name = "descendant",
    summary = "descends from SharedParent declared in mc_parent",
)]
#[descended_from(SharedParent)]
pub struct Descendant;

// Cross-member reference: a #[presents(SharedParent)] whose antigen is declared
// in the OTHER member (mc_parent). This is the Layer-2 resolution target — its
// canonical_path must re-resolve to mc_parent@, NOT mc_child@ where it lives.
#[presents(SharedParent)]
pub fn child_site_presenting_parents_antigen() {{}}

// Cross-member defense: a #[defended_by(SharedParent)] in mc_child must address
// (defend) the SharedParent presents-sites once both sides re-resolve to
// mc_parent@ — the heart of cross-crate addresses() (ADR-017 Amendment 1).
#[cfg(test)]
#[defended_by(SharedParent)]
fn child_test_defends_parents_antigen() {{}}
"#
    )
    .expect("write child lib.rs");
    drop(clib);
}

#[test]
fn synthetic_workspace_stamps_each_member_distinctly() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    write_two_member_workspace(dir.path());

    let report = scan_workspace_multi_crate(dir.path())
        .expect("member-aware scan of synthetic 2-member workspace");

    let parent = report
        .antigens
        .iter()
        .find(|a| a.type_name == "SharedParent")
        .expect("SharedParent must be discovered");
    let child = report
        .antigens
        .iter()
        .find(|a| a.type_name == "Descendant")
        .expect("Descendant must be discovered");

    assert_eq!(
        parent.canonical_path.as_deref(),
        Some("mc_parent@0.1.0"),
        "parent antigen must be stamped to its own member"
    );
    assert_eq!(
        child.canonical_path.as_deref(),
        Some("mc_child@0.2.0"),
        "child antigen must be stamped to its own member — distinct from the parent's"
    );
}

#[test]
fn synthetic_workspace_resolves_cross_member_descended_from() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    write_two_member_workspace(dir.path());

    let report = scan_workspace_multi_crate(dir.path())
        .expect("member-aware scan of synthetic 2-member workspace");

    // Exactly one lineage edge: Descendant -> SharedParent, crossing members.
    let edge = report
        .lineage_edges
        .iter()
        .find(|e| e.child == "Descendant" && e.parent == "SharedParent")
        .expect("cross-member lineage edge must be collected");

    assert_eq!(
        edge.child_canonical_path.as_deref(),
        Some("mc_child@0.2.0"),
        "edge child endpoint = the member bearing #[descended_from]"
    );
    assert_eq!(
        edge.parent_canonical_path.as_deref(),
        Some("mc_parent@0.1.0"),
        "edge parent endpoint must RE-RESOLVE to the member declaring SharedParent \
         (the heart of cross-crate #[descended_from])"
    );

    // The edge must NOT be orphaned — the parent is resolvable in the merged set.
    assert!(
        report.orphaned_lineage_edges().is_empty(),
        "re-resolved cross-member edge must not be flagged orphaned; orphans: {:?}",
        report.orphaned_lineage_edges()
    );

    // And the parent's #[presents(SharedParent)] site must have propagated to the
    // descendant as an inherited presentation (ADR-018 propagation over the
    // cross-member edge). Per ADR-018 §"The synthesis algorithm", the inherited
    // presentation lands at the descendant's *declaration site* (the mc_child
    // lib.rs file) but PRESERVES the ancestor's identity — `antigen_type =
    // SharedParent`, `canonical_path = mc_parent@0.1.0` — with a provenance
    // `inherited_from` chain naming SharedParent@mc_parent. That this propagated
    // AT ALL across the member boundary is the cross-crate-lineage win.
    let child_lib = dir.path().join("mc_child").join("src").join("lib.rs");
    let inherited = report.presentations.iter().find(|p| {
        p.antigen_type == "SharedParent" && p.inherited_from.is_some() && p.file == child_lib
    });
    let inherited = inherited.unwrap_or_else(|| {
        panic!(
            "cross-member lineage propagation must attach an inherited SharedParent \
             presentation at the mc_child site; presentations: {:?}",
            report
                .presentations
                .iter()
                .map(|p| (
                    &p.antigen_type,
                    p.canonical_path.as_deref(),
                    p.file.file_name(),
                    p.inherited_from.is_some()
                ))
                .collect::<Vec<_>>()
        )
    });
    assert_eq!(
        inherited.canonical_path.as_deref(),
        Some("mc_parent@0.1.0"),
        "inherited presentation preserves the ancestor's canonical identity (ADR-018)"
    );
    let provenance = inherited
        .inherited_from
        .as_ref()
        .expect("inherited presentation carries provenance");
    assert!(
        provenance.iter().any(|pe| pe.antigen_type == "SharedParent"
            && pe.canonical_path.as_deref() == Some("mc_parent@0.1.0")),
        "provenance chain must name SharedParent@mc_parent as the source"
    );
}

// ============================================================================
// Layer 2: cross-crate addresses() resolution (ADR-017 Amendment 1, ATK-A3-005).
// A reference record (presents / defended_by) whose antigen is declared in a
// DIFFERENT member must re-resolve its canonical_path to the declaring member,
// so cross-member addresses() matches instead of failing on a mis-stamped path.
// ============================================================================

#[test]
fn cross_member_presents_resolves_to_declaring_member() {
    // mc_child's `#[presents(SharedParent)]` site lives in mc_child but addresses
    // an antigen DECLARED in mc_parent. Per-member stamping first puts
    // mc_child@0.2.0 on it (where it lives); Layer-2 resolution must re-stamp it
    // to mc_parent@0.1.0 (where SharedParent is declared) — the contract that
    // Presentation::canonical_path is the *antigen's* site, not the site's.
    let dir = tempfile::TempDir::new().expect("tempdir");
    write_two_member_workspace(dir.path());
    let report = scan_workspace_multi_crate(dir.path()).expect("member-aware scan");

    let child_lib = dir.path().join("mc_child").join("src").join("lib.rs");
    // The EXPLICIT cross-member presents-site (not the inherited one).
    let cross = report
        .presentations
        .iter()
        .find(|p| {
            p.antigen_type == "SharedParent" && p.file == child_lib && p.inherited_from.is_none()
        })
        .unwrap_or_else(|| {
            panic!(
                "the explicit cross-member #[presents(SharedParent)] in mc_child must be \
                 found; presentations: {:?}",
                report
                    .presentations
                    .iter()
                    .map(|p| (
                        &p.antigen_type,
                        p.canonical_path.as_deref(),
                        p.file.file_name(),
                        p.inherited_from.is_some()
                    ))
                    .collect::<Vec<_>>()
            )
        });
    assert_eq!(
        cross.canonical_path.as_deref(),
        Some("mc_parent@0.1.0"),
        "Layer-2: a cross-member presents-site must re-resolve to the member that \
         DECLARES the antigen (mc_parent), not the member it lives in (mc_child)"
    );
}

#[test]
fn cross_member_defended_by_resolves_and_addresses() {
    // The crux of cross-crate addresses(): mc_child's #[defended_by(SharedParent)]
    // must, after Layer-2 re-stamping, DEFEND the SharedParent presents-sites that
    // also resolve to mc_parent@. Before Layer 2, the defense was keyed mc_child@,
    // the antigen mc_parent@, and defense_addresses() (canonical-path-keyed) would
    // NOT match — the DelegateCrossCrateResolutionGap. After, they match.
    let dir = tempfile::TempDir::new().expect("tempdir");
    write_two_member_workspace(dir.path());
    let report = scan_workspace_multi_crate(dir.path()).expect("member-aware scan");

    let defense = report
        .defenses
        .iter()
        .find(|d| d.antigen_type == "SharedParent")
        .expect("the cross-member #[defended_by(SharedParent)] must be collected");
    assert_eq!(
        defense.canonical_path.as_deref(),
        Some("mc_parent@0.1.0"),
        "Layer-2: a cross-member defense must re-resolve to the antigen's declaring \
         member so its (antigen_type, canonical_path) key matches the presents-sites"
    );

    // Now the audit-side invariant: with both the defense and the parent's own
    // #[presents(SharedParent)] resolved to mc_parent@, the defense ADDRESSES that
    // presents-site. unaddressed_presentations() must NOT list the parent's
    // explicit SharedParent site (it is now cross-crate-defended).
    let parent_lib = dir.path().join("mc_parent").join("src").join("lib.rs");
    let unaddressed = report.unaddressed_presentations();
    assert!(
        !unaddressed.iter().any(|up| {
            up.presentation.antigen_type == "SharedParent"
                && up.presentation.file == parent_lib
                && up.presentation.inherited_from.is_none()
        }),
        "the parent's explicit SharedParent presents-site must be ADDRESSED by the \
         cross-member defense (DelegateCrossCrateResolutionGap closed); unaddressed: {:?}",
        unaddressed
            .iter()
            .map(|up| (
                &up.presentation.antigen_type,
                up.presentation.canonical_path.as_deref()
            ))
            .collect::<Vec<_>>()
    );
}

// ============================================================================
// Defensive: a bad workspace root produces a structured Err, not a panic.
// ============================================================================

#[test]
fn member_enumeration_errs_on_nonexistent_workspace() {
    let bad = PathBuf::from("/nonexistent-antigen-ws-mc-xyz");
    assert!(
        enumerate_workspace_member_roots(&bad).is_err(),
        "nonexistent workspace must yield Err, not panic"
    );
}
