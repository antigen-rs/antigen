//! # Digest Strip-List Completeness Guard
//!
//! Closes a `RatifiedSpecDriftFromImpl` / `ParallelStateTrackersDiverge` instance
//! in antigen's own infrastructure: the digest's `ANTIGEN_OWNED_ATTRS` list (the
//! attribute names stripped before hashing, so toggling an antigen attr never
//! invalidates a signed-against digest) and the set of `#[proc_macro_attribute]`s
//! antigen actually defines are **two parallel trackers of one truth** — "which
//! attributes does antigen own?" — that nothing forced to agree.
//!
//! ## The drift this guards (lived)
//!
//! A comment in `digest.rs` once claimed "all 26 antigen macro names are now in
//! `ANTIGEN_OWNED_ATTRS`." When the clinical-medicine family (`#[panel]`,
//! `#[rx]`, `#[triage]`, …) and `#[antigen_generates]` were added to
//! `antigen-macros`, nobody added them to the strip-list — and every existing
//! test stayed green, because the existing digest tests only spot-check three
//! representative families. Result: applying e.g. `#[panel]` to a signed item
//! silently changed its structural digest (the source attribute is hashed
//! because it is NOT stripped), invalidating the signature — exactly the
//! attestation-insensitivity invariant the strip-list exists to uphold.
//!
//! ## What this test does (the assertion the comment lacked)
//!
//! Reads BOTH surfaces as text — (1) `ANTIGEN_OWNED_ATTRS` in `src/digest.rs`
//! (this crate), and (2) every `#[proc_macro_attribute]`-annotated `pub fn` in
//! `../antigen-macros/src/lib.rs` — and asserts set-equality (modulo one
//! documented exception). A new macro added without a strip-list entry — or a
//! strip-list entry with no backing macro — fails here, loud, not silent. This
//! is the structural fix: the cross-surface invariant is now ENFORCED, not
//! commented.
//!
//! ## The one documented exception: `immune`
//!
//! `immune` is in the strip-list but is NOT a live `#[proc_macro_attribute]` — the
//! `#[immune]` macro was removed (ADR-029). It is RETAINED in the strip-list so
//! that the digest of an item in a not-yet-migrated dependency crate (which still
//! carries `#[immune]` in source) stays stable. Stripping it is name-based and
//! needs no live macro. The guard hard-codes this as the single allowed
//! strip-list-only name; if a second such case ever appears, it must be added
//! here deliberately (with a reason), never silently.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// Strip-list names that are intentionally NOT backed by a live
/// `#[proc_macro_attribute]`. See the module docs for `immune`.
const RETAINED_NON_MACRO_NAMES: &[&str] = &["immune"];

/// Read a file relative to THIS crate's manifest dir.
fn read_rel(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// Extract the string literals inside the `ANTIGEN_OWNED_ATTRS` array in
/// `src/digest.rs`. The array is `const ANTIGEN_OWNED_ATTRS: &[&str] = &[ ... ];`
/// with one `"name",` per entry (comments interspersed). We slice the array body
/// and pull every double-quoted token.
fn strip_list_names() -> BTreeSet<String> {
    let src = read_rel("src/digest.rs");
    let decl = "const ANTIGEN_OWNED_ATTRS: &[&str] = &[";
    let start = src
        .find(decl)
        .unwrap_or_else(|| panic!("`{decl}` not found in src/digest.rs"))
        + decl.len();
    let rest = &src[start..];
    let end = rest
        .find("];")
        .unwrap_or_else(|| panic!("unterminated ANTIGEN_OWNED_ATTRS array"));
    let body = &rest[..end];

    // Pull every `"..."` literal. Line-comments (`// ...`) never contain a `"`
    // in this array (verified), so a naive quote-scan is unambiguous here; to be
    // safe we still skip `//`-comment lines before scanning.
    let mut names = BTreeSet::new();
    for line in body.lines() {
        let code = line.split("//").next().unwrap_or("");
        let mut chars = code.char_indices().peekable();
        while let Some((i, c)) = chars.next() {
            if c == '"' {
                let after = &code[i + 1..];
                let close = after
                    .find('"')
                    .expect("string literal in ANTIGEN_OWNED_ATTRS must terminate on its line");
                names.insert(after[..close].to_string());
                // advance past the closing quote
                while let Some(&(j, _)) = chars.peek() {
                    if j > i + 1 + close {
                        break;
                    }
                    chars.next();
                }
            }
        }
    }
    names
}

/// Extract every `#[proc_macro_attribute]`-annotated `pub fn <name>` from
/// `../antigen-macros/src/lib.rs`. The attribute and the `pub fn` line are
/// adjacent (the `pub fn` is on the next non-doc line after the attribute), so we
/// scan for `#[proc_macro_attribute]` and read the next `pub fn <name>`.
fn proc_macro_attribute_names() -> BTreeSet<String> {
    let src = read_rel("../antigen-macros/src/lib.rs");
    let lines: Vec<&str> = src.lines().collect();
    let mut names = BTreeSet::new();
    for (i, line) in lines.iter().enumerate() {
        if line.trim() == "#[proc_macro_attribute]" {
            // Find the next `pub fn <name>(` line.
            let fn_line = lines[i + 1..]
                .iter()
                .find(|l| l.trim_start().starts_with("pub fn "))
                .unwrap_or_else(|| panic!("no `pub fn` after #[proc_macro_attribute] at line {i}"));
            let after_fn = fn_line.trim_start().strip_prefix("pub fn ").unwrap();
            let name: String = after_fn
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            assert!(
                !name.is_empty(),
                "could not parse macro name from `{fn_line}`"
            );
            names.insert(name);
        }
    }
    assert!(
        !names.is_empty(),
        "found zero #[proc_macro_attribute]s — parser is broken, not the macros crate"
    );
    names
}

#[test]
fn strip_list_covers_every_antigen_owned_attribute() {
    let strip = strip_list_names();
    let macros = proc_macro_attribute_names();
    let retained: BTreeSet<String> = RETAINED_NON_MACRO_NAMES
        .iter()
        .map(|s| (*s).to_string())
        .collect();

    // Every defined macro must appear in the strip-list — else toggling that attr
    // on a signed item silently changes its digest.
    let missing_from_strip: Vec<&String> = macros.difference(&strip).collect();
    assert!(
        missing_from_strip.is_empty(),
        "ANTIGEN_OWNED_ATTRS is MISSING {} antigen-owned attribute(s): {missing_from_strip:?}\n\
         Each #[proc_macro_attribute] in antigen-macros/src/lib.rs must be in \
         ANTIGEN_OWNED_ATTRS (antigen-fingerprint/src/digest.rs), or applying it to a \
         signed item silently invalidates the signature (attestation-insensitivity).",
        missing_from_strip.len(),
    );

    // Every strip-list name must be backed by a live macro — EXCEPT the documented
    // retained-legacy names (`immune`). A strip-list entry with no macro and no
    // retained-reason is dead config (or a typo).
    let unbacked: Vec<&String> = strip
        .difference(&macros)
        .filter(|n| !retained.contains(*n))
        .collect();
    assert!(
        unbacked.is_empty(),
        "ANTIGEN_OWNED_ATTRS has {} name(s) with no backing #[proc_macro_attribute] and no \
         retained-legacy reason: {unbacked:?}\n\
         Either the macro was removed (add the name to RETAINED_NON_MACRO_NAMES with a reason, \
         like `immune`), or the entry is a typo.",
        unbacked.len(),
    );

    // Belt-and-suspenders: the exact set identity, so the two surfaces are locked
    // together and the count can't drift behind a matched add+remove.
    let expected: BTreeSet<String> = macros.union(&retained).cloned().collect();
    assert_eq!(
        strip, expected,
        "ANTIGEN_OWNED_ATTRS must equal {{ all #[proc_macro_attribute] names }} ∪ \
         {{ retained-legacy names }}. Update the strip-list (digest.rs) and/or \
         RETAINED_NON_MACRO_NAMES to match.",
    );
}
