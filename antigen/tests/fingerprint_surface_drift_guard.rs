//! # Fingerprint-Surface Drift Guard — closes the `ParallelStateTrackersDiverge`
//! class in antigen's own test infrastructure.
//!
//! ## The class this guards (a self-applying instance)
//!
//! A stdlib member's fingerprint string is **hand-duplicated** across surfaces:
//! the member declaration (`src/stdlib/<family>.rs`), and — for members that ship
//! a demonstrating **example** (`examples/<family>.rs`) or a **scan fixture**
//! (`tests/fixtures/<family>/lib.rs`) — a *copy* of the same string. The
//! `stdlib_family_fingerprints.rs` affinity-pair tests carry yet another copy as a
//! `const`.
//!
//! Those copies were "kept in sync" by a **comment** ("the fingerprint string
//! asserted here is the same shape the member declares"), not by an assertion.
//! That is exactly antigen's own `ParallelStateTrackersDiverge` failure-class —
//! parallel trackers of one truth with nothing forcing them to agree — *in
//! antigen's own tests*. The beta.2 seal demonstrated the risk concretely: when
//! the `from_slice` / `zeroed` / `set_len` arms were dropped from the member
//! fingerprints, the fixture and example copies silently kept the old arms (the
//! member-fp != copy-fp drift), and **every existing test stayed green** because
//! each surface only checks its own copy.
//!
//! ## What this test does (the structural-equality assertion the comment lacked)
//!
//! For each member that has a duplicated copy, it reads BOTH source files as text,
//! extracts the `fingerprint = r#"..."#` string for that member by its `name`, and
//! asserts the two strings **parse to structurally-equal `Fingerprint`s**
//! (`Fingerprint: PartialEq`). A drift between any member and its copy fails here —
//! loud, not silent.
//!
//! This is the interim, syntactic enforcement. The single-source-of-truth refactor
//! (examples/fixtures importing the member's fingerprint rather than copying it) is
//! the v0.4 structural fix — chartered, see `v2/drift-guard-unenforced`.

use std::fs;
use std::path::Path;

use antigen_fingerprint::Fingerprint;

/// Read a workspace-relative file (relative to the `antigen` crate manifest dir).
fn read(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// Extract the `fingerprint = r#"..."#` string belonging to the `#[antigen(...)]`
/// (or `#[presents]`-adjacent `#[antigen]`) whose `name = "<member>"` appears
/// nearest *before* it. Returns the raw DSL string (without the `r#"`/`"#`).
///
/// The files use single-line `name = "..."` and `fingerprint = r#"..."#,` forms
/// (verified across all four surfaces), so a name-anchored forward scan to the
/// next `fingerprint = r#"` is unambiguous.
fn fingerprint_for(src: &str, member_name: &str) -> String {
    let name_marker = format!("name = \"{member_name}\"");
    let name_at = src
        .find(&name_marker)
        .unwrap_or_else(|| panic!("member name `{member_name}` not found"));
    let after = &src[name_at..];
    let fp_open = after
        .find("fingerprint = r#\"")
        .unwrap_or_else(|| panic!("no fingerprint after name `{member_name}`"))
        + "fingerprint = r#\"".len();
    let rest = &after[fp_open..];
    let fp_close = rest
        .find("\"#")
        .unwrap_or_else(|| panic!("unterminated fingerprint for `{member_name}`"));
    rest[..fp_close].to_string()
}

/// Assert two surfaces declare a structurally-equal fingerprint for one member.
fn assert_surfaces_agree(member: &str, file_a: &str, file_b: &str) {
    let fp_a = Fingerprint::parse(&fingerprint_for(&read(file_a), member))
        .unwrap_or_else(|e| panic!("{file_a} fingerprint for `{member}` must parse: {e}"));
    let fp_b = Fingerprint::parse(&fingerprint_for(&read(file_b), member))
        .unwrap_or_else(|e| panic!("{file_b} fingerprint for `{member}` must parse: {e}"));
    assert_eq!(
        fp_a, fp_b,
        "DRIFT: `{member}` fingerprint differs between {file_a} and {file_b} — \
         a duplicated surface fell out of sync with the member (the \
         ParallelStateTrackersDiverge class). Update the copy to match the member."
    );
}

// ── deserialization: member ↔ example ───────────────────────────────────────
#[test]
fn unbounded_deserialization_member_matches_example() {
    assert_surfaces_agree(
        "unbounded-deserialization",
        "src/stdlib/deserialization.rs",
        "examples/deserialization.rs",
    );
}

#[test]
fn deserialize_without_deny_member_matches_example() {
    assert_surfaces_agree(
        "deserialize-without-deny-unknown-fields",
        "src/stdlib/deserialization.rs",
        "examples/deserialization.rs",
    );
}

// ── unsafe-soundness: member ↔ scan fixture ─────────────────────────────────
#[test]
fn uninit_assumed_init_member_matches_fixture() {
    assert_surfaces_agree(
        "uninit-memory-assumed-init",
        "src/stdlib/unsafe_soundness.rs",
        "tests/fixtures/family_unsafe_soundness/lib.rs",
    );
}

#[test]
fn transmute_mismatch_member_matches_fixture() {
    assert_surfaces_agree(
        "transmute-size-or-lifetime-mismatch",
        "src/stdlib/unsafe_soundness.rs",
        "tests/fixtures/family_unsafe_soundness/lib.rs",
    );
}

#[test]
fn from_utf8_unchecked_member_matches_fixture() {
    assert_surfaces_agree(
        "unvalidated-from-utf8-unchecked",
        "src/stdlib/unsafe_soundness.rs",
        "tests/fixtures/family_unsafe_soundness/lib.rs",
    );
}
