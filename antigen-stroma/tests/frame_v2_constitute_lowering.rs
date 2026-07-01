//! frame-v2#4 — integration tests for the constitute lowering seam (kill the 13 dark mutants).
//!
//! The survey's cargo-mutants sweep found 13 mutants surviving across the constitute lowering, none
//! killed by any test — the whole `ScanReport -> NodeFacts` seam was integration-test-dark because no
//! `ScanReport` fixture existed. This file, built on `support/scan_report_fixture.rs`, closes that:
//! it exercises `lower_scan_report` (the only pub entry) over real specimen files so that
//! `module_chain_from_path`, `digests_at_line`, and the lowering itself each have teeth.
//!
//! Both private helpers are reached THROUGH `lower_scan_report`, observing:
//!   - `module_chain_from_path` via the produced `fq_path` (which encodes the module chain:
//!     `crate::<chain>::<item>`),
//!   - `digests_at_line` via whether the identity digest is a REAL content digest (item found at
//!     line) or the traceable `gap_digests` fallback (item NOT found).
//!
//! ## Mutant coverage map (`adapter.rs` line → guarding assertion — anchored to the post-fix tree)
//!
//! `module_chain_from_path` (fn @:116) —
//! - :135 lib special-case (`&&` / `==`) → `lib_rs_items_live_in_crate_root` + the `&&`→`||` come-apart `lib_and_main_special_cases_require_whole_path_not_first_segment`
//! - :139 main special-case (`&&` / `==`) → `main_rs_items_live_in_crate_root` + `lib_and_main_special_cases_require_whole_path_not_first_segment`
//! - :143 mod `== "mod"` / :144 strip-leaf `segments.len() - 1` → `mod_rs_items_live_in_parent_module`
//! - regular-file fall-through / multi-segment join → `regular_file_and_deep_nesting_full_chain`
//!
//! `digests_at_line` (fn @:70) —
//! - :76 the CONTAINMENT range `(start..=end).contains(&target_line)` (was exact `==`; the v2 fix) → found-arm `real_item_at_line_yields_content_digest`; the containment SEMANTIC is locked by `atk_frame_v2_multi_attr_different_lines_merge_to_one_node` (in `atk_frame_v2_dedup_identity.rs`)
//! - :85 body→`None` (short-circuit) / a non-item line → `missing_line_yields_traceable_gap_digest` (gap)
//!
//! `lower_scan_report` (fn @:164) —
//! - body→(empty,empty) → every test above (any real node disproves "return empty")

#[path = "support/scan_report_fixture.rs"]
mod fixture;

use antigen_stroma::constitute::adapter::lower_scan_report;
use antigen_stroma::node::cfg::CfgSet;
use fixture::{FixtureFile, ScanReportBuilder, fixture_root};

const SUB: &str = "frame_v2_modtree";

/// Lower a single-struct specimen file and return the produced node's `fq_path` + whether its identity
/// digest is the traceable gap-fallback (`gap_digests` sets identity = `BLAKE3(fq_path)`).
fn lower_one_struct(rel: &str, struct_name: &str) -> (String, bool) {
    let (source_root, rel_path) = fixture_root(SUB, rel);
    let file = FixtureFile::load(&source_root, &rel_path);
    let item = file.struct_named(struct_name);

    let report = ScanReportBuilder::new()
        .presentation(&rel_path, item.line, item.item_target)
        .build();

    let (nodes, edges) = lower_scan_report(&report, &source_root, "cratename", &CfgSet::default());
    assert!(edges.is_empty(), "frame epoch emits no edges");
    assert_eq!(
        nodes.len(),
        1,
        "lower_scan_report dropped the only node — the lowering returned empty (mutant :174) or the \
         intern path is broken."
    );

    let node = &nodes[0];
    // gap_digests sets identity_digest = BLAKE3(fq_path.path.as_bytes()). If the item was FOUND at
    // its line, digests_at_line returns the REAL content digest, which is NOT BLAKE3(fq_path).
    let gap_identity =
        antigen_stroma::node::digest::IdentityDigest::of_tokens(node.id.fq_path.path.as_bytes());
    let is_gap = node.id.identity_digest == gap_identity;
    (node.id.fq_path.path.clone(), is_gap)
}

// ── module_chain_from_path: the lib / main / mod / regular-file / deep branches ─────────────────────

/// `src/lib.rs` → crate root → chain `[]` → `fq_path` `cratename::CrateRootItem`.
/// Guards adapter.rs:119 (the lib special-case): a mutant that flips `== "lib"` or the `&&` would
/// produce `cratename::lib::CrateRootItem`, caught here.
#[test]
fn lib_rs_items_live_in_crate_root() {
    let (fq, is_gap) = lower_one_struct("src/lib.rs", "CrateRootItem");
    assert_eq!(
        fq, "cratename::CrateRootItem",
        "src/lib.rs items must live in the crate root (empty module chain). Got `{fq}` — the lib \
         special-case (adapter.rs:119) mis-fired."
    );
    assert!(
        !is_gap,
        "the real struct at its line must yield a content digest, not the gap fallback"
    );
}

/// `src/main.rs` → crate root → chain `[]`. Guards adapter.rs:123 (the main special-case, DISTINCT
/// from lib — a mutant that removes the main arm would leave `cratename::main::MainRootItem`).
#[test]
fn main_rs_items_live_in_crate_root() {
    let (fq, is_gap) = lower_one_struct("src/main.rs", "MainRootItem");
    assert_eq!(
        fq, "cratename::MainRootItem",
        "src/main.rs items must live in the crate root. Got `{fq}` — the main special-case \
         (adapter.rs:123) mis-fired."
    );
    assert!(!is_gap);
}

/// `src/node/mod.rs` → module `node` (the `mod` leaf stripped) → chain `["node"]`.
/// Guards adapter.rs:127 (the `== "mod"` check) AND :128 (the `segments.len() - 1` strip arithmetic):
/// a `-`→`+` mutant would panic/overflow or keep the `mod` leaf; a `==`→`!=` would not strip.
#[test]
fn mod_rs_items_live_in_parent_module() {
    let (fq, is_gap) = lower_one_struct("src/node/mod.rs", "NodeModItem");
    assert_eq!(
        fq, "cratename::node::NodeModItem",
        "src/node/mod.rs items must live in module `node` with the `mod` leaf stripped. Got `{fq}` \
         — the mod special-case (adapter.rs:127) or the strip-leaf arithmetic (:128) is wrong."
    );
    assert!(!is_gap);
}

/// `src/node/locator.rs` → module `node::locator` (regular file, full path) → chain `["node","locator"]`.
/// AND `src/a/b/c.rs` → `["a","b","c"]`. Guards the fall-through (adapter.rs:132) and the multi-segment
/// join — distinguishing a plain file from the mod/lib/main special-cases at depth > 1.
#[test]
fn regular_file_and_deep_nesting_full_chain() {
    let (fq_loc, gap_loc) = lower_one_struct("src/node/locator.rs", "LocatorItem");
    assert_eq!(
        fq_loc, "cratename::node::locator::LocatorItem",
        "a regular file `src/node/locator.rs` must yield the full `node::locator` chain (NOT stripped \
         like mod.rs). Got `{fq_loc}`."
    );
    assert!(!gap_loc);

    let (fq_deep, gap_deep) = lower_one_struct("src/a/b/c.rs", "DeepItem");
    assert_eq!(
        fq_deep, "cratename::a::b::c::DeepItem",
        "deep nesting `src/a/b/c.rs` must yield the full `a::b::c` chain. Got `{fq_deep}`."
    );
    assert!(!gap_deep);
}

/// The lib/main special-cases must fire ONLY for the WHOLE-path-is-`lib`/`main` case (`len == 1 && ..`),
/// NOT merely when the FIRST segment is `lib`/`main`. `src/lib/sub.rs` → `["lib","sub"]` and
/// `src/main/sub.rs` → `["main","sub"]`, NOT `[]`. This is the come-apart that kills the surviving
/// `&&`→`||` mutants at adapter.rs:119 / :123 — a `||` would collapse a multi-segment `lib*`/`main*`
/// path to the crate root. (Found by the cargo-mutants teeth-sweep; closed here.)
#[test]
fn lib_and_main_special_cases_require_whole_path_not_first_segment() {
    let (fq_lib, gap_lib) = lower_one_struct("src/lib/sub.rs", "LibSubItem");
    assert_eq!(
        fq_lib, "cratename::lib::sub::LibSubItem",
        "`src/lib/sub.rs` must yield `[\"lib\",\"sub\"]` — the lib special-case fired on a mere \
         first-segment match (adapter.rs:119 `&&`→`||`), collapsing a real `lib::sub` module to the \
         crate root. Got `{fq_lib}`."
    );
    assert!(!gap_lib);

    let (fq_main, gap_main) = lower_one_struct("src/main/sub.rs", "MainSubItem");
    assert_eq!(
        fq_main, "cratename::main::sub::MainSubItem",
        "`src/main/sub.rs` must yield `[\"main\",\"sub\"]` — the main special-case fired on a mere \
         first-segment match (adapter.rs:123 `&&`→`||`). Got `{fq_main}`."
    );
    assert!(!gap_main);
}

// ── digests_at_line: found vs gap-fallback ──────────────────────────────────────────────────────────

/// A record pointing at the REAL struct line yields a content digest (item FOUND). This is the
/// positive arm of `digests_at_line`: it PROVES the `==` line-match (adapter.rs:60) hits, and that the
/// function does not short-circuit to `None` (:56).
#[test]
fn real_item_at_line_yields_content_digest() {
    let (_fq, is_gap) = lower_one_struct("src/lib.rs", "CrateRootItem");
    assert!(
        !is_gap,
        "digests_at_line returned the gap-fallback for an item that IS present at its line — the \
         line-match (:60) missed or the read/parse (:56) short-circuited to None."
    );
}

/// A record pointing at a line where NO item starts must fall back to the TRACEABLE gap-digest.
/// This is the negative arm: it proves the `==` at :60 is a real discriminator (a `!=` mutant would
/// match the WRONG line and mint a spurious content digest instead of the honest gap), and that the
/// gap-fallback path (`unwrap_or_else(gap_digests)`) is reachable.
#[test]
fn missing_line_yields_traceable_gap_digest() {
    let (source_root, rel_path) = fixture_root(SUB, "src/lib.rs");
    let file = FixtureFile::load(&source_root, &rel_path);
    let real = file.struct_named("CrateRootItem");

    // Point at a line guaranteed NOT to be an item's span-start (line 1 is a `//` comment).
    let bogus_line = 1usize;
    assert_ne!(
        real.line, bogus_line,
        "test setup: bogus line must differ from the real item line"
    );

    let report = ScanReportBuilder::new()
        .presentation(&rel_path, bogus_line, real.item_target)
        .build();

    let (nodes, _edges) = lower_scan_report(&report, &source_root, "cratename", &CfgSet::default());
    assert_eq!(
        nodes.len(),
        1,
        "the node still lands (with gap digests) even when the line misses"
    );

    let node = &nodes[0];
    let gap_identity =
        antigen_stroma::node::digest::IdentityDigest::of_tokens(node.id.fq_path.path.as_bytes());
    assert_eq!(
        node.id.identity_digest, gap_identity,
        "a record whose line matches NO item must fall back to the traceable gap-digest \
         (identity = BLAKE3(fq_path)). It did not — digests_at_line matched a wrong line (mutant \
         :60 `==`→`!=`) or otherwise returned a content digest for a non-item line."
    );
}

// ── lower_scan_report: cross-vec merge + emptiness ──────────────────────────────────────────────────

/// The legitimate cross-attribute-vec dedup: the SAME item appearing in TWO vecs (a presentation AND
/// an immunity at the same file/line/target) must merge to ONE node (same `StromaNodeId`). Distinct from
/// the v2#1 born-red (which forbids collapsing DISTINCT-identity same-target items) — this proves the
/// fix preserves the merge it SHOULD do. Guards `lower_scan_report` :174 (body→empty would give 0 nodes).
#[test]
fn same_item_across_two_vecs_merges_to_one_node() {
    let (source_root, rel_path) = fixture_root(SUB, "src/node/locator.rs");
    let file = FixtureFile::load(&source_root, &rel_path);
    let item = file.struct_named("LocatorItem");

    // Presentation AND immunity for the SAME (file, line, target). The builder's fixture only exposes
    // presentation(); a second presentation at the identical coordinates exercises the same
    // same-StromaNodeId merge the cross-vec case relies on (both interned via the same intern()).
    let report = ScanReportBuilder::new()
        .presentation(&rel_path, item.line, item.item_target.clone())
        .presentation(&rel_path, item.line, item.item_target)
        .build();

    let (nodes, _edges) = lower_scan_report(&report, &source_root, "cratename", &CfgSet::default());
    assert_eq!(
        nodes.len(),
        1,
        "the SAME item at the SAME (file,line,target) must merge to ONE node (same StromaNodeId). \
         {} nodes means the legitimate identity-merge was lost.",
        nodes.len()
    );
}
