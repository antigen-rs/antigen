//! ATK-FRAME-V2-DEDUP-001 — the constitute dedup must key on IDENTITY, not `(file, ItemTarget)`.
//!
//! ## The defect (frame-v2#1 · frame-v2-dedup-key-collapses-same-itemtarget)
//!
//! `constitute/adapter.rs`'s `NodeKey` deduplicates on `(file, format!("{item_target:?}"))`. Two
//! items in ONE file that share an `ItemTarget` but have DISTINCT bodies (→ distinct
//! `IdentityDigest`s, → distinct `StromaNodeId`s) collide on that key. The second is dropped at the
//! `node_map.contains_key(&key) → return` early-gate **before its `identity_digest` is ever
//! computed**. That is exactly what §4.1 forbids: `ItemTarget` is NOT identity. Two `impl Foo` blocks
//! in one module are the canonical case — the collision the whole frame exists to close, re-imported
//! at the synchronization node.
//!
//! ## The come-apart (born-red)
//!
//! `atk_frame_v2_dedup_two_impls_same_target_both_survive` builds a `ScanReport` with two records
//! pointing at the two REAL `impl Foo` blocks in the fixture (same `ItemTarget`, distinct lines →
//! distinct bodies → distinct identity). It asserts BOTH survive as DISTINCT `StromaNodeId`s through
//! `lower_scan_report`.
//!   - RED against the current `(file, ItemTarget)` dedup (2nd silently dropped → 1 node).
//!   - GREEN against the fixed identity-keyed dedup (2 distinct nodes).
//!
//! ## The negative control (teeth)
//!
//! `nc_two_genuinely_identical_records_dedup_to_one` points TWO records at the SAME line (same
//! `ItemTarget` AND same identity digest — a true duplicate). It must dedup to ONE node under BOTH
//! the current and the fixed key. This proves the fix collapses genuine duplicates, not
//! distinct-same-target items — without it, "key on identity" could be satisfied by never deduping at
//! all (a vacuous pass).
//!
//! ## The teeth-proof of record
//!
//! The born-red was FORCE-RUN against `eb1156b` (the certified frame, current dedup) and FAILED with
//! `1 != 2` — recorded in the ATK registry. The builder's identity-keyed fix turns it GREEN; that
//! red→green IS the verification of record for this v2 patch.

#[path = "support/scan_report_fixture.rs"]
mod fixture;

use std::collections::HashSet;

use antigen_stroma::constitute::adapter::lower_scan_report;
use antigen_stroma::node::cfg::CfgSet;
use antigen_stroma::node::id::StromaNodeId;
use fixture::{FixtureFile, ScanReportBuilder, fixture_path};

const FIXTURE_REL: &str = "frame_v2_dedup/same_target_distinct_identity.rs";

/// Load the two-impl fixture and pin its shape: exactly two inherent `impl Foo` blocks, at DISTINCT
/// lines. This guards the specimen — if an edit collapses or duplicates the blocks, the test that
/// relies on "two distinct-body same-target items" fails loud here rather than silently going vacuous.
fn located_impls() -> (
    std::path::PathBuf,
    std::path::PathBuf,
    Vec<fixture::LocatedItem>,
) {
    let (source_root, rel) = fixture_path(FIXTURE_REL);
    let file = FixtureFile::load(&source_root, &rel);
    let impls = file.inherent_impls_of("Foo");
    assert_eq!(
        impls.len(),
        2,
        "specimen invariant broken: expected exactly two `impl Foo` blocks in {FIXTURE_REL}, found {}",
        impls.len()
    );
    assert_ne!(
        impls[0].line, impls[1].line,
        "specimen invariant broken: the two `impl Foo` blocks must sit at distinct lines"
    );
    (source_root, rel, impls)
}

/// THE BORN-RED (ATK-FRAME-V2-DEDUP-001): two items sharing an `ItemTarget` but with distinct
/// identity digests must BOTH survive `lower_scan_report` as DISTINCT `StromaNodeId`s.
#[test]
fn atk_frame_v2_dedup_two_impls_same_target_both_survive() {
    let (source_root, rel, impls) = located_impls();

    // Two records, one per real impl block. Same ItemTarget (`Impl { None, "Foo" }`), distinct lines
    // → distinct bodies → distinct IdentityDigest. Under the current key these collide on
    // (file, ItemTarget) and the second is dropped.
    let report = ScanReportBuilder::new()
        .presentation(&rel, impls[0].line, impls[0].item_target.clone())
        .presentation(&rel, impls[1].line, impls[1].item_target.clone())
        .build();

    let (nodes, edges) =
        lower_scan_report(&report, &source_root, "fixture_crate", &CfgSet::default());

    assert!(edges.is_empty(), "frame epoch emits no edges");

    let ids: HashSet<StromaNodeId> = nodes.iter().map(|n| n.id.clone()).collect();
    assert_eq!(
        nodes.len(),
        2,
        "ATK-FRAME-V2-DEDUP-001: two impl-Foo blocks sharing an ItemTarget but with DISTINCT bodies \
         collapsed to {} node(s) — the second was dropped at the (file, ItemTarget) dedup gate \
         BEFORE its identity_digest was consulted (§4.1: ItemTarget is NOT identity). FIX: key the \
         dedup on the identity-digest / StromaNodeId, not (file, ItemTarget).",
        nodes.len()
    );
    assert_eq!(
        ids.len(),
        2,
        "ATK-FRAME-V2-DEDUP-001: the two surviving nodes are not DISTINCT StromaNodeIds — they must \
         differ by identity_digest (same fq_path, distinct body)."
    );

    // Both share the SAME fq_path (same module + same item name `Foo`) and differ ONLY by
    // identity_digest — the precise come-apart §4.1 protects.
    let fq_paths: HashSet<String> = ids.iter().map(|id| id.fq_path.path.clone()).collect();
    assert_eq!(
        fq_paths.len(),
        1,
        "the two impl-Foo nodes should share ONE fq_path and differ only by identity_digest; \
         distinct fq_paths would mean the come-apart is not the identity-only one §4.1 targets."
    );
    let digests: HashSet<_> = ids.iter().map(|id| id.identity_digest.clone()).collect();
    assert_eq!(
        digests.len(),
        2,
        "the two impl-Foo nodes must carry DISTINCT identity_digests (distinct bodies)."
    );
}

/// NEGATIVE CONTROL: two GENUINELY-identical records (same `ItemTarget` AND same line → same identity)
/// must dedup to ONE node under both keys. Proves the fix collapses true duplicates and the born-red
/// above isn't satisfiable by simply never deduping.
#[test]
fn nc_two_genuinely_identical_records_dedup_to_one() {
    let (source_root, rel, impls) = located_impls();

    // BOTH records point at the SAME impl block (same file, same line, same ItemTarget) → identical
    // identity digest → a true duplicate.
    let report = ScanReportBuilder::new()
        .presentation(&rel, impls[0].line, impls[0].item_target.clone())
        .presentation(&rel, impls[0].line, impls[0].item_target.clone())
        .build();

    let (nodes, _edges) =
        lower_scan_report(&report, &source_root, "fixture_crate", &CfgSet::default());

    assert_eq!(
        nodes.len(),
        1,
        "NC: two genuinely-identical records (same file/line/ItemTarget → same identity) must dedup \
         to ONE node. {} nodes means the dedup over-splits true duplicates.",
        nodes.len()
    );
}

const MULTI_ATTR_REL: &str = "frame_v2_dedup/multi_attr_different_lines.rs";

/// BORN-RED (locks the `digests_at_line` CONTAINMENT semantic) — ONE item carrying two antigen attrs
/// on DIFFERENT lines produces two records (at line N and line N+1, both INSIDE the item's `syn` span)
/// that must merge to exactly ONE node.
///
/// This is REQUIRED, not decorative: the v2#1 dedup fix (key on identity) UNMASKED a latent
/// `digests_at_line` bug. Under the old exact-span-start matching (`== target_line`), the record on the
/// SECOND attr line (N+1) matched no item's span-start → fell back to `gap_digests` → got a DISTINCT
/// `identity_digest` → and, because the dedup now keys on identity, SPLIT into a spurious second node.
/// The fix matches by CONTAINMENT (`span.start().line ..= span.end().line`), so both attr-lines resolve
/// to the SAME containing item → SAME `IdentityDigest` → dedup to ONE node.
///
///   - RED against exact-span-start matching (2 nodes: N real + N+1 gap).
///   - GREEN under containment matching (1 node: both resolve to the same item).
///
/// `same_item_across_two_vecs_merges_to_one_node` (the SAME-line case) does NOT guard this — exact-match
/// already passes it. Only the DIFFERENT-line case locks containment; without it a silent revert to
/// exact-match passes every other test. (`#[presents]`@N + `#[immune]`@N+1 is routine in antigen's own
/// dogfooding, which is why this had to close before the engine builds on these facts.)
#[test]
fn atk_frame_v2_multi_attr_different_lines_merge_to_one_node() {
    let (source_root, rel) = fixture_path(MULTI_ATTR_REL);
    let file = FixtureFile::load(&source_root, &rel);
    let (start, end, target) = file.struct_span("TwoAttrItem");

    // Two attr-sites on DIFFERENT lines of ONE item: the span-start line N and the NEXT line N+1.
    // Both must fall inside the item's span `[start ..= end]` (the specimen guarantees a multi-line span).
    let second_attr_line = start + 1;
    assert!(
        second_attr_line <= end,
        "specimen setup: the second attr line ({second_attr_line}) must fall inside the item span \
         [{start}..={end}] — the whole containment premise"
    );
    assert_ne!(
        start, second_attr_line,
        "the two records must sit on DIFFERENT lines (else this is the same-line case, which \
         exact-match already passes and which does NOT guard containment)"
    );

    let report = ScanReportBuilder::new()
        .presentation(&rel, start, target.clone())
        .presentation(&rel, second_attr_line, target)
        .build();

    let (nodes, _edges) =
        lower_scan_report(&report, &source_root, "fixture_crate", &CfgSet::default());

    assert_eq!(
        nodes.len(),
        1,
        "ATK (containment): ONE item with attrs on two DIFFERENT lines (N={start}, N+1={second_attr_line}, \
         both inside span [{start}..={end}]) split into {} nodes. The N+1 record missed the item under \
         exact-span-start matching → gap-digest → a spurious second node (unmasked by the identity-keyed \
         dedup). FIX: match `digests_at_line` by CONTAINMENT (span.start().line ..= span.end().line), so \
         both attr-lines resolve to the same containing item → same IdentityDigest → one node.",
        nodes.len()
    );
}
