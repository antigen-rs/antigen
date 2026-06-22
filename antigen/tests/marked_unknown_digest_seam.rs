//! P0a — the marked-unknown digest seam (the keystone PROPOSE-slice input
//! precondition; ADR-045 Amd-1).
//!
//! THE FAILING TESTS THAT DEFINE DONE. Write RED first; green = the seam is real.
//!
//! As-shipped at the HEAD these were authored against, every marked-unknown's
//! `structural_digest` is `""` and its `cluster_key` is `"dread@"` (the
//! over-merge) — `antigen/src/scan/types.rs:739` emits
//! `structural_digest: String::new()` and `:733` `cluster_key_of("", marker)`.
//! The PROPOSE-slice's "group marked-unknowns by `structural_digest`" therefore
//! collapses every dread mark into ONE bucket regardless of structure, which
//! anti-unifies unrelated ASTs into a hole-that-matches-everything (autoimmunity
//! the B-governor exists to prevent). These tests pin the fix: the enclosing
//! item's `structural_digest` rides through to the emitted Finding.
//!
//! The fix direction (design not prescription): add a
//! `structural_digest` field to `MarkedUnknown`, populate it from
//! `self.current_item_digest` at the push in `extract_marked_unknown`
//! (`parse.rs:481`; the digest is already in scope — set per item before
//! `check_attrs`), and have `to_finding` use it for BOTH
//! `structural_digest` and `cluster_key_of(digest, marker)`.

use std::path::{Path, PathBuf};

use antigen::scan::{MarkedUnknown, scan_workspace};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

/// Collect the marked-unknowns from a fixture scan, keyed by the marked item's
/// declaration so a test can name them by source order independently of the
/// scanner's internal visit order. We key by `line` (stable per fixture) and by
/// trigger-substring for human-legible assertions.
fn scan_marks(fixture_name: &str) -> Vec<MarkedUnknown> {
    let scan = scan_workspace(&fixture(fixture_name), None).expect("scan completes");
    scan.marked_unknowns
}

/// Find the single mark whose trigger contains `needle`.
fn mark_by_trigger<'a>(marks: &'a [MarkedUnknown], needle: &str) -> &'a MarkedUnknown {
    marks
        .iter()
        .find(|m| m.trigger.contains(needle))
        .unwrap_or_else(|| {
            panic!("no marked-unknown with trigger containing {needle:?}; got: {marks:?}")
        })
}

/// The `structural_digest` the emitted Finding carries for a given mark.
/// The **identity** digest (name+code-sensitive `structural_digest`). Diff-native
/// DETECT keys on this; two items differing only in NAME differ here.
fn finding_identity_digest(m: &MarkedUnknown) -> String {
    m.to_finding(0).structural_digest
}

/// The **clustering** digest the PROPOSE-slice groups on: the name-INSENSITIVE
/// `shape_digest`.
///
/// Two structurally-identical items with DIFFERENT names share this (so they
/// anti-unify into one draft). As-shipped before the seam it was `""`; the
/// marker-pollution bug (`dread`/`aura`/`red_flag` missing from
/// `ANTIGEN_OWNED_ATTRS`) made it differ on trigger text — both now fixed.
fn finding_shape_digest(m: &MarkedUnknown) -> String {
    m.to_finding(0).shape_digest
}

/// The `cluster_key` the emitted Finding carries. As-shipped this is `"dread@"`
/// for every dread mark (the over-merge). Once wired it is `"dread@<digest>"`.
fn finding_cluster_key(m: &MarkedUnknown) -> String {
    m.to_finding(0).cluster_key
}

// ===========================================================================
// TEST 1 — the digest seam carries the enclosing item's structure.
// ===========================================================================

#[test]
fn marked_unknown_finding_carries_the_enclosing_item_digest() {
    // Fixture: Alpha (#[dread], body A), Beta (#[dread], body B != A),
    // Gamma (#[dread], body STRUCTURALLY IDENTICAL to Alpha).
    let marks = scan_marks("marked_unknown_digest_seam");
    assert_eq!(
        marks.len(),
        3,
        "fixture has exactly three #[dread] marks; got: {marks:?}"
    );

    let alpha = mark_by_trigger(
        &marks,
        "drops the lock guard before the buffer flush; ordering",
    );
    let beta = mark_by_trigger(&marks, "re-enters the connection pool");
    let gamma = mark_by_trigger(&marks, "same shape as Alpha");

    // The PROPOSE-slice clusters on the name-INSENSITIVE SHAPE digest (the
    // two-field distinction, ADR-045 Amd-2): identical-shape items must
    // share it so anti-unification has a cluster to generalize.
    let s_alpha = finding_shape_digest(alpha);
    let s_beta = finding_shape_digest(beta);
    let s_gamma = finding_shape_digest(gamma);

    // (1) NON-EMPTY for every marked-unknown — the core failure of the
    //     as-shipped seam (all three carried "").
    for (label, d) in [("alpha", &s_alpha), ("beta", &s_beta), ("gamma", &s_gamma)] {
        assert!(
            !d.is_empty(),
            "marked-unknown {label} must carry a NON-EMPTY shape_digest (the \
             enclosing item's body shape); as-shipped this was \"\" — the bug. \
             digest = {d:?}"
        );
    }

    // (2) structurally-IDENTICAL enclosing items share the SHAPE digest …
    //     Alpha and Gamma differ in NAME and in #[dread] trigger text, but have
    //     identical body shape — so the name-insensitive, marker-stripped shape
    //     digest MUST be equal (this is the regression fence for BOTH the
    //     marker-pollution bug AND the name-sensitivity axis).
    assert_eq!(
        s_alpha, s_gamma,
        "Alpha and Gamma are structurally identical (differing only in name + \
         #[dread] trigger text) — their SHAPE digests MUST be equal so PROPOSE \
         clusters them together. If unequal, either the marker payload still \
         pollutes the shape digest (dread/aura/red_flag must be in \
         ANTIGEN_OWNED_ATTRS) or the name leaked in (shape digest must normalize \
         the ident). alpha={s_alpha:?} gamma={s_gamma:?}"
    );

    // (3) … and a structurally DIFFERENT enclosing item has a different shape.
    assert_ne!(
        s_alpha, s_beta,
        "Alpha and Beta are structurally DIFFERENT structs — their shape digests \
         must differ so PROPOSE does not over-merge them. alpha={s_alpha:?} beta={s_beta:?}"
    );

    // (4) THE TWO-FIELD DISTINCTION (ADR-045 Amd-2): the IDENTITY digest is
    //     name-SENSITIVE, so Alpha and Gamma (different names) must DIFFER on it
    //     even though they share a shape. This pins that the two fields carry
    //     DIFFERENT meanings — collapsing them would break either diff-native
    //     DETECT (needs identity) or PROPOSE clustering (needs shape).
    assert_ne!(
        finding_identity_digest(alpha),
        finding_identity_digest(gamma),
        "Alpha and Gamma have DIFFERENT names → their IDENTITY (structural_digest) \
         must differ, even though their SHAPE digests match. If these are equal, \
         the identity digest has been collapsed into the shape digest — diff-native \
         DETECT (which keys on identity) would then miss a rename."
    );
}

// ===========================================================================
// TEST 2 — PROPOSE groups by structure, not by marker-kind.
// ===========================================================================

#[test]
fn propose_slice_groups_by_structure_not_by_marker_kind() {
    // Group the marked-unknowns by cluster_key (what the PROPOSE-slice keys on).
    // The identical pair (Alpha, Gamma) shares one key; Beta is separate → at
    // least TWO distinct keys among the three dread marks.
    let marks = scan_marks("marked_unknown_digest_seam");

    let keys: Vec<String> = marks.iter().map(finding_cluster_key).collect();
    let distinct: std::collections::BTreeSet<&String> = keys.iter().collect();

    assert!(
        distinct.len() >= 2,
        "PROPOSE must see AT LEAST TWO distinct cluster_keys among these dread marks \
         (the structurally-identical Alpha/Gamma share one; the distinct Beta is \
         another). As-shipped there is exactly ONE key \"dread@\" — the over-merge \
         that anti-unifies unrelated ASTs into a match-everything fingerprint. \
         keys = {keys:?}"
    );

    // And the over-merge sentinel must be gone: no key may be the bare
    // marker-only "dread@" (empty-digest) form.
    for k in &keys {
        assert_ne!(
            k, "dread@",
            "the bare marker-only cluster_key \"dread@\" (empty digest) is the \
             over-merge bug — every dread mark collapsed into one bucket. keys = {keys:?}"
        );
    }
}

// ===========================================================================
// TEST 3 — ordering-regression guard (defends against a naive capture-too-early
// fix; the current visit-order is already correct, keep this fence).
// ===========================================================================

#[test]
fn ordering_regression_guard() {
    // Item A immediately precedes Item B in source; A and B have DISTINCT bodies.
    // B's mark must carry B's OWN digest, not A's stale digest.
    let marks = scan_marks("marked_unknown_digest_ordering");
    assert_eq!(marks.len(), 2, "fixture has two marks; got: {marks:?}");

    let a = mark_by_trigger(&marks, "item A:");
    let b = mark_by_trigger(&marks, "item B:");

    let d_a = finding_shape_digest(a);
    let d_b = finding_shape_digest(b);

    assert!(
        !d_a.is_empty() && !d_b.is_empty(),
        "both marks must carry a non-empty digest; a={d_a:?} b={d_b:?}"
    );
    assert_ne!(
        d_a, d_b,
        "Item B's digest must be B's OWN (it is a structurally distinct item), \
         NOT inherited from the immediately-preceding Item A. A naive \
         capture-too-early fix would make B carry A's stale digest. \
         a={d_a:?} b={d_b:?}"
    );
}

// ===========================================================================
// TEST 4 — a mark on a non-canonical-item position (enum variant). Per the
// refinement: the variant shares the enclosing ENUM's digest (a
// principled stand-in), which must be NON-EMPTY — it must NOT silently
// re-collapse into the empty "dread@" bucket.
// ===========================================================================

#[test]
fn mark_on_a_non_item_position_uses_a_principled_nonempty_digest() {
    // Two #[dread]-marked enum variants of ONE enum.
    let marks = scan_marks("marked_unknown_digest_nonitem");
    assert_eq!(
        marks.len(),
        2,
        "fixture has two variant-level #[dread] marks; got: {marks:?}"
    );

    for m in &marks {
        let d = finding_shape_digest(m);
        // PRINCIPLED, NON-EMPTY: the enclosing enum's digest stands in for the
        // variant (parse.rs design comment). The empty-string sentinel "" is the
        // over-merge bug and is forbidden here too.
        assert!(
            !d.is_empty(),
            "a #[dread] on an enum VARIANT must carry the enclosing enum's \
             structural_digest (a principled stand-in) — NOT the empty \"\" \
             sentinel that re-collapses into the \"dread@\" bucket. mark = {m:?}, \
             digest = {d:?}"
        );
        assert_ne!(
            finding_cluster_key(m),
            "dread@",
            "variant marks must not carry the bare empty-digest \"dread@\" cluster_key"
        );
    }

    // Documented spec consequence (an open question, NOT
    // a failure assertion): under the enum-granular stand-in, two distinct marked
    // variants of ONE enum share the enclosing enum's digest → they cluster
    // together. If the PROPOSE-slice later needs per-variant discrimination, this
    // is the seam to revisit. We assert the SHARED-key consequence so a future
    // change that introduces per-variant digests trips this fence and forces a
    // deliberate spec decision rather than a silent behavior drift.
    let keys: std::collections::BTreeSet<String> = marks.iter().map(finding_cluster_key).collect();
    assert_eq!(
        keys.len(),
        1,
        "the enum-granular stand-in means both variant marks share the enclosing \
         enum's digest → ONE cluster_key. If this becomes >1, per-variant digests \
         were introduced — a deliberate spec change, not a silent \
         drift. keys = {keys:?}"
    );
}
