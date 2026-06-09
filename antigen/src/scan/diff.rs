//! diff-native DETECT (v0.4, ADR-046) — match a structural DELTA, not a snapshot.
//!
//! Antigen's fingerprint model is a SNAPSHOT predicate: it matches code as it is
//! at one commit. The most dangerous failure-classes live in the DELTA — the
//! change that REMOVED a bounds guard, MOVED a check past an await, loosened a
//! bound. A snapshot scan of the AFTER state is blind by construction: an absent
//! guard is indistinguishable from a never-needed one. The DIFF carries the
//! signal — "this item's structure changed between these two versions."
//!
//! This is the **DETECT** tier (do-now, cheap): compare two versions of the
//! source on the **scan path**, key each item by **(item-name,
//! [`structural_digest`](antigen_fingerprint::structural_digest))**, and set-diff.
//! An item whose `(name, digest)` pair differs (added / removed / structure
//! changed) is surfaced. Keyed on item-**name**, NOT file+line: a reorder shifts
//! every line but changes no item's name or structure, so a benign reorder
//! surfaces ZERO churn (the degenerate the line-keyed approach gets wrong).
//!
//! The CLASSIFY tier (say *a guard* was removed, not just *structure changed*)
//! and the LABEL tier (was it a REQUIRED guard? — Rice-undecidable, human/incident)
//! are do-later / out of scope here (ADR-046).
//!
//! # Claim-scope (ADR-044)
//!
//! **What this proves:** a structural DELETION/CHANGE/ADDITION of a named item
//! between two versions is detected reproducibly (decidable, machine). **What it
//! does NOT prove:** whether the changed structure was a REQUIRED guard (semantic,
//! undecidable — Rice). DETECT reports "this item's structure changed"; "this was
//! a guard regression" is the do-later CLASSIFY heuristic + the human LABEL.
//!
//! ## Why identity digest (not shape digest)
//!
//! diff-native keys on `structural_digest` — the **identity** (name+code-sensitive)
//! digest — precisely because a structure change MUST register. This is distinct
//! from the marked-unknown PROPOSE-slice's `shape_digest` (name-insensitive
//! clustering): there, two differently-named identical bodies should cluster;
//! here, any structural edit to a named item should surface. Two fields, two
//! honest meanings (ADR-045 Amd-2).

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

/// The kind of structural change DETECT observed for one item name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChangeKind {
    /// The item exists only in the AFTER version (newly introduced).
    Added,
    /// The item exists only in the BEFORE version (removed).
    Removed,
    /// The item exists in both but its `structural_digest` changed.
    Modified,
}

/// One detected structural change: the item's name + how it changed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemChange {
    /// The item's name (the stable diff key).
    pub name: String,
    /// Whether it was added, removed, or had its structure modified.
    pub kind: ChangeKind,
}

/// The result of a diff-native DETECT pass: the set of items whose structure
/// changed between two versions, plus the totals.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralDiff {
    /// The changed items (added / removed / modified), sorted by name.
    pub changes: Vec<ItemChange>,
}

impl StructuralDiff {
    /// `true` when no item's structure changed (the benign-motion case).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// The set of changed item NAMES (the core DETECT signal). Convenience for
    /// callers that only need "which items changed" without the change kind.
    #[must_use]
    pub fn changed_names(&self) -> BTreeSet<String> {
        self.changes.iter().map(|c| c.name.clone()).collect()
    }
}

/// An `item-name → structural_digest` map for one version of the source.
///
/// The projection a diff-native pass set-diffs. Keyed by name (stable across a
/// reorder); valued by the identity digest (stable across whitespace / position,
/// sensitive to real structural change).
pub type ItemDigestMap = BTreeMap<String, String>;

/// Build the [`ItemDigestMap`] for one parsed file's top-level items.
///
/// Every `HasAttributes` item kind with a nameable identity participates; the
/// item's name is its diff key and its [`structural_digest`](antigen_fingerprint::structural_digest)
/// the value. Item kinds without a single top-level name (impls, `use`, …) are
/// skipped — they have no stable diff key.
#[must_use]
pub fn item_digest_map(file: &syn::File) -> ItemDigestMap {
    let mut map = ItemDigestMap::new();
    for item in &file.items {
        if let Some((name, digest)) = named_item_digest(item) {
            map.insert(name, digest);
        }
    }
    map
}

/// Build the [`ItemDigestMap`] for a whole version made of several parsed files.
///
/// The multi-file scan-path case. Names collide across files only if the source
/// has genuinely-shadowing top-level idents; the last write wins (a degenerate
/// the per-file map already collapses).
#[must_use]
pub fn item_digest_map_multi(files: &[syn::File]) -> ItemDigestMap {
    let mut map = ItemDigestMap::new();
    for file in files {
        map.extend(item_digest_map(file));
    }
    map
}

/// Diff-native DETECT over two pre-built [`ItemDigestMap`]s: the set of items
/// added, removed, or whose structure changed.
///
/// An item present in both versions with an **identical** digest is NOT in the
/// diff (no phantom churn). This is the pure set-diff at the heart of the
/// modality; [`scan_diff_files`] is the convenience that parses source first.
#[must_use]
pub fn diff_item_digests(before: &ItemDigestMap, after: &ItemDigestMap) -> StructuralDiff {
    let mut changes = Vec::new();
    let all_names: BTreeSet<&String> = before.keys().chain(after.keys()).collect();
    for name in all_names {
        let kind = match (before.get(name), after.get(name)) {
            (Some(db), Some(da)) if db == da => continue, // unchanged structure
            (Some(_), Some(_)) => ChangeKind::Modified,
            (None, Some(_)) => ChangeKind::Added,
            (Some(_), None) => ChangeKind::Removed,
            (None, None) => unreachable!("name came from one of the two maps"),
        };
        changes.push(ItemChange {
            name: name.clone(),
            kind,
        });
    }
    StructuralDiff { changes }
}

/// Diff-native DETECT over two versions of source code (each a single file's
/// text). Parses both, builds the `(name, digest)` maps, and set-diffs.
///
/// This is the file-level convenience the `cargo antigen review <diff>` surface
/// rides; the multi-file scan-path variant composes [`item_digest_map_multi`]
/// with [`diff_item_digests`].
///
/// # Errors
///
/// Returns a [`syn::Error`] if either source fails to parse as a Rust file.
pub fn scan_diff_files(before_src: &str, after_src: &str) -> syn::Result<StructuralDiff> {
    let before = item_digest_map(&syn::parse_file(before_src)?);
    let after = item_digest_map(&syn::parse_file(after_src)?);
    Ok(diff_item_digests(&before, &after))
}

/// Extract `(name, structural_digest)` for a top-level item that has a single
/// nameable identity. Returns `None` for kinds without one (impls, `use`,
/// extern-crate, foreign-mod, macro invocations).
fn named_item_digest(item: &syn::Item) -> Option<(String, String)> {
    use antigen_fingerprint::structural_digest;
    match item {
        syn::Item::Fn(i) => Some((i.sig.ident.to_string(), structural_digest(i))),
        syn::Item::Struct(i) => Some((i.ident.to_string(), structural_digest(i))),
        syn::Item::Enum(i) => Some((i.ident.to_string(), structural_digest(i))),
        syn::Item::Union(i) => Some((i.ident.to_string(), structural_digest(i))),
        syn::Item::Trait(i) => Some((i.ident.to_string(), structural_digest(i))),
        syn::Item::Type(i) => Some((i.ident.to_string(), structural_digest(i))),
        syn::Item::Const(i) => Some((i.ident.to_string(), structural_digest(i))),
        syn::Item::Static(i) => Some((i.ident.to_string(), structural_digest(i))),
        syn::Item::Mod(i) => Some((i.ident.to_string(), structural_digest(i))),
        syn::Item::TraitAlias(i) => Some((i.ident.to_string(), structural_digest(i))),
        // No single top-level name → no stable diff key.
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GUARD_BEFORE: &str = r"
        pub fn validate(i: usize, len: usize) -> bool {
            if i >= len { return false; }
            true
        }
        pub fn helper(x: u8) -> u8 { x.wrapping_add(1) }
    ";

    const GUARD_AFTER_REMOVED: &str = r"
        pub fn validate(i: usize, len: usize) -> bool {
            true
        }
        pub fn helper(x: u8) -> u8 { x.wrapping_add(1) }
    ";

    const REORDERED_BENIGN: &str = r"
        // a comment inserted above

        pub fn helper(x: u8) -> u8 { x.wrapping_add(1) }


        pub fn validate(i: usize, len: usize) -> bool {
            if i >= len { return false; }
            true
        }
    ";

    #[test]
    fn guard_removal_surfaces_the_modified_item() {
        let diff = scan_diff_files(GUARD_BEFORE, GUARD_AFTER_REMOVED).unwrap();
        assert_eq!(
            diff.changed_names(),
            BTreeSet::from(["validate".to_string()])
        );
        let change = &diff.changes[0];
        assert_eq!(change.name, "validate");
        assert_eq!(change.kind, ChangeKind::Modified);
    }

    #[test]
    fn benign_reorder_surfaces_nothing() {
        // Same items, same structure, moved + comment inserted → ZERO churn.
        let diff = scan_diff_files(GUARD_BEFORE, REORDERED_BENIGN).unwrap();
        assert!(
            diff.is_empty(),
            "a benign reorder must surface no structural change; got {:?}",
            diff.changes
        );
    }

    #[test]
    fn added_and_removed_items_are_classified() {
        let before = "pub fn a() {} pub fn b() {}";
        let after = "pub fn a() {} pub fn c() {}";
        let diff = scan_diff_files(before, after).unwrap();
        let mut kinds: Vec<(String, ChangeKind)> = diff
            .changes
            .iter()
            .map(|c| (c.name.clone(), c.kind))
            .collect();
        kinds.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(
            kinds,
            vec![
                ("b".to_string(), ChangeKind::Removed),
                ("c".to_string(), ChangeKind::Added),
            ]
        );
    }

    #[test]
    fn discriminates_real_change_from_benign_motion() {
        let real = scan_diff_files(GUARD_BEFORE, GUARD_AFTER_REMOVED).unwrap();
        let benign = scan_diff_files(GUARD_BEFORE, REORDERED_BENIGN).unwrap();
        assert!(!real.is_empty());
        assert!(benign.is_empty());
        assert_ne!(real, benign);
    }

    #[test]
    fn struct_field_change_surfaces() {
        let before = "pub struct S { a: u8 }";
        let after = "pub struct S { a: u8, b: u16 }";
        let diff = scan_diff_files(before, after).unwrap();
        assert_eq!(diff.changed_names(), BTreeSet::from(["S".to_string()]));
        assert_eq!(diff.changes[0].kind, ChangeKind::Modified);
    }

    #[test]
    fn serializes_co_natively() {
        let diff = scan_diff_files(GUARD_BEFORE, GUARD_AFTER_REMOVED).unwrap();
        let json = serde_json::to_string(&diff).expect("serializes");
        assert!(json.contains("\"kind\":\"modified\""));
        let back: StructuralDiff = serde_json::from_str(&json).unwrap();
        assert_eq!(back, diff);
    }
}
