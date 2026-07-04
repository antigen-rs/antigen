//! diff-native DETECT (ADR-046) — THE ACCEPTANCE GATE for the do-now half.
//!
//! Campsite-adjacent: `dream/the-antigen-that-spans-the-diff-not-the-snapshot`
//! (the DETECT slice is do-now; CLASSIFY/LABEL are do-later/human). STREAM-DIFF
//! in the Pioneers baton §2.
//!
//! THE SPEC (briefing §2 diff-native DETECT):
//!   - a fn with a bounds guard at HEAD~1 and the guard removed at HEAD: the
//!     `(name, structural_digest)` set-diff surfaces "validate changed structure";
//!   - a benign line-shift (reorder/insert, no structural change) does NOT surface
//!     phantom churn (THE ADVERSARIAL'S UN-RUN DEGENERATE — built here).
//!
//! This gate pins the CORE diff-native algorithm against the SHIPPED public
//! primitive `antigen_fingerprint::structural_digest`, keyed by item NAME (not
//! file+line — the pathmaker-verified identity choice that sidesteps the
//! Finding-line-identity gap; a reorder shifts every line but no item's name or
//! structure). The set-diff = symmetric difference of `(name, digest)` pairs.
//!
//! The public scan-path diff entrypoint (a `cargo antigen review <diff>` /
//! `scan_diff(before, after)` surface) is do-now but unbuilt; when it ships it
//! must satisfy THIS behavior. The algorithm gate below is the falsifiable
//! definition of done it builds toward.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

fn fixture_src(name: &str) -> String {
    let p: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
        .join("lib.rs");
    std::fs::read_to_string(p).expect("fixture lib.rs readable")
}

/// Build the `item-name → structural_digest` map for a fixture, the way a
/// scan-path diff would pull it from the syn AST. Keyed by NAME (stable across a
/// reorder), valued by the structure-keyed digest (stable across whitespace /
/// position). Only the item kinds the fixtures use are modelled here; the real
/// scan surface covers all `HasAttributes` kinds.
fn name_to_digest(fixture: &str) -> HashMap<String, String> {
    let src = fixture_src(fixture);
    let file = syn::parse_file(&src).expect("fixture parses");
    let mut map = HashMap::new();
    for item in &file.items {
        if let syn::Item::Fn(f) = item {
            let name = f.sig.ident.to_string();
            let digest = antigen_fingerprint::structural_digest(f);
            map.insert(name, digest);
        }
    }
    map
}

/// The diff-native DETECT primitive: the set of item NAMES whose
/// `(name, structural_digest)` pair differs between two versions — i.e. items
/// that were added, removed, or whose STRUCTURE changed. An item present in both
/// with an identical digest is NOT in the diff (no phantom churn).
fn structurally_changed_items(before: &str, after: &str) -> BTreeSet<String> {
    let b = name_to_digest(before);
    let a = name_to_digest(after);

    let mut changed = BTreeSet::new();
    // names in either version
    let all_names: BTreeSet<&String> = b.keys().chain(a.keys()).collect();
    for name in all_names {
        match (b.get(name), a.get(name)) {
            (Some(db), Some(da)) if db == da => { /* unchanged structure — no churn */ },
            _ => {
                changed.insert(name.clone());
            },
        }
    }
    changed
}

// ===========================================================================
// (1) A guard removal surfaces. `validate` loses its bounds guard between
//     before/after → its structure changed → it IS in the diff. `helper` and
//     `process` are byte-identical → NOT in the diff.
// ===========================================================================

#[test]
fn guard_removal_surfaces_as_a_structural_change() {
    let changed = structurally_changed_items("diff_native_guard_before", "diff_native_guard_after");

    assert!(
        changed.contains("validate"),
        "removing `validate`'s bounds guard changes its structure — the \
         (name, digest) set-diff MUST surface `validate`. This is the \
         guard-removal-on-a-PR blind-spot a snapshot scan cannot see. changed = {changed:?}"
    );
    assert!(
        !changed.contains("helper"),
        "`helper` is byte-identical across before/after — it must NOT appear in the \
         diff. changed = {changed:?}"
    );
    assert!(
        !changed.contains("process"),
        "`process` is unchanged — it must NOT appear in the diff. changed = {changed:?}"
    );
    // Exactly one item changed: validate.
    assert_eq!(
        changed,
        BTreeSet::from(["validate".to_string()]),
        "exactly one item (`validate`) changed structure; got {changed:?}"
    );
}

// ===========================================================================
// (2) THE DEGENERATE — a benign line-shift surfaces NOTHING (no phantom churn).
//     The reorder fixture has the SAME items with the SAME structure, just moved
//     around with blank lines + comments inserted. A snapshot-blind tool keyed on
//     file+line would flag every item as "changed"; the diff-native modality,
//     keyed on (name, structure), must flag NONE.
// ===========================================================================

#[test]
fn benign_reorder_surfaces_no_phantom_churn() {
    let changed =
        structurally_changed_items("diff_native_guard_before", "diff_native_benign_reorder");

    assert!(
        changed.is_empty(),
        "a benign reorder/insert (no item's structure changed) must surface ZERO \
         structural changes — phantom churn here means the diff keyed on line \
         position instead of (name, structure). changed = {changed:?}"
    );
}

// ===========================================================================
// (3) The two diffs are DISTINCT — guards the degenerate isn't passing because
//     the diff is a no-op for EVERYTHING (a diff that never surfaces anything
//     would pass (2) vacuously). The guard-removal diff is non-empty; the
//     benign-reorder diff is empty. Same `before` baseline, different `after`.
// ===========================================================================

#[test]
fn the_modality_discriminates_real_change_from_benign_motion() {
    let real = structurally_changed_items("diff_native_guard_before", "diff_native_guard_after");
    let benign =
        structurally_changed_items("diff_native_guard_before", "diff_native_benign_reorder");

    assert!(
        !real.is_empty(),
        "precondition: the guard-removal diff must be non-empty"
    );
    assert!(
        benign.is_empty(),
        "precondition: the benign-reorder diff must be empty"
    );
    assert_ne!(
        real, benign,
        "the modality must DISCRIMINATE a real structural change from benign motion \
         — if these two diffs were equal, the detector would be a constant (useless). \
         real = {real:?}, benign = {benign:?}"
    );
}

// ===========================================================================
// THE REAL ENTRYPOINT — `scan_diff_files` (now SHIPPED, scan/diff.rs). The gates
// above proved the algorithm against the raw primitive; these refute the public
// surface the `cargo antigen review <diff>` render rides.
// ===========================================================================

#[test]
fn scan_diff_files_surfaces_a_guard_removal_as_modified() {
    use antigen::scan::{ChangeKind, scan_diff_files};

    let before = fixture_src("diff_native_guard_before");
    let after = fixture_src("diff_native_guard_after");
    let diff = scan_diff_files(&before, &after).expect("both fixtures parse");

    let changed = diff.changed_names();
    assert!(
        changed.contains("validate"),
        "the shipped scan_diff_files must surface `validate` (guard removed) — \
         changed = {changed:?}"
    );
    assert!(
        !changed.contains("helper"),
        "an unchanged item must not surface — changed = {changed:?}"
    );
    // and the change KIND is Modified (present in both, structure changed).
    let validate_change = diff
        .changes
        .iter()
        .find(|c| c.name == "validate")
        .expect("validate change present");
    assert_eq!(
        validate_change.kind,
        ChangeKind::Modified,
        "a guard removed from an existing fn is a MODIFIED change (not Added/Removed)"
    );
}

#[test]
fn scan_diff_files_no_phantom_churn_on_a_benign_reorder() {
    use antigen::scan::scan_diff_files;

    let before = fixture_src("diff_native_guard_before");
    let reordered = fixture_src("diff_native_benign_reorder");
    let diff = scan_diff_files(&before, &reordered).expect("both parse");

    assert!(
        diff.is_empty(),
        "the shipped scan_diff_files must surface ZERO changes on a benign reorder \
         (no item's structure changed). changes = {:?}",
        diff.changes
    );
}

// ===========================================================================
// THE BLIND SPOT (pinned as a deliberate scope decision, not a silent miss).
// `named_item_digest` returns None for `impl` items — so a guard removed inside a
// METHOD (impl Foo { fn validate() }) is INVISIBLE to diff-native DETECT, because
// the impl has no single top-level name key. Most real guards live in methods, so
// this materially narrows DETECT's reach (the demand-side need #7 — "a PR that
// removes a guard is laundered" — is only PARTIALLY closed: free fns yes, methods
// no). This test PINS the current behavior so a future method-level-diff change is
// a conscious decision, and so the gap is visible (not discovered in production).
// ===========================================================================

#[test]
fn guard_removed_inside_an_impl_method_is_currently_invisible_a_known_blind_spot() {
    use antigen::scan::scan_diff_files;

    let before = r"
        pub struct Validator;
        impl Validator {
            pub fn check(&self, i: usize, len: usize) -> bool {
                if i >= len { return false; }
                true
            }
        }
    ";
    // The guard `if i >= len { return false; }` is REMOVED from the method.
    let after = r"
        pub struct Validator;
        impl Validator {
            pub fn check(&self, i: usize, len: usize) -> bool {
                let _ = (i, len);
                true
            }
        }
    ";
    let diff = scan_diff_files(before, after).expect("both parse");

    // CURRENT behavior: the impl is skipped (no top-level name), so the method
    // guard removal is NOT surfaced. `Validator` (the struct) is unchanged.
    assert!(
        diff.is_empty(),
        "KNOWN BLIND SPOT: a guard removed inside an impl METHOD is currently \
         invisible to diff-native DETECT (impls have no top-level name key, so \
         named_item_digest skips them). This test documents the gap — if it starts \
         FAILING (the diff is non-empty), method-level diff was implemented, which \
         is the DESIRED closure of demand-side need #7 for methods; update this \
         fence to assert the method change IS surfaced. changes = {:?}",
        diff.changes
    );
}
