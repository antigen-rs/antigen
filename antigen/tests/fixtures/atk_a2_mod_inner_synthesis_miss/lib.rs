// Fixture: synthesis-pass KNOWN LIMITATION — items inside inline `mod` blocks
// are not visited.
//
// synthesis_pass() iterates `parsed_file.items` (the top-level syn::File items)
// and calls item_kind_and_target() on each. For a syn::Item::Mod, the function
// currently returns None — the mod is skipped, and its inner items are NEVER
// evaluated against fingerprints.
//
// The fingerprint `name = matches("INNER_*")` would match INNER_StructSite if
// synthesis descended into the mod block. It does not — items inside an inline
// mod are currently invisible to fingerprint synthesis.
//
// This is a DOCUMENTED limitation (scan.rs synthesis_pass comment: "Only
// top-level items are checked; descent into impl methods and trait methods
// deferred to W6b/A3.") — the same "deferred" applies to mod-inner items.
//
// The top-level struct OUTER_StructSite (outside any mod) SHOULD get 1 match.
// The mod-inner INNER_StructSite should get 0 matches.
// Total expected: 1.

#[antigen(name = "inner-mod-miss", fingerprint = "name = matches(\"INNER_*\")")]
pub struct InnerModMiss;

// Top-level struct — not inside any mod. Does NOT match the fingerprint
// (name does not start with INNER_).
pub struct OUTER_StructSite;

pub mod inner {
    // Inside a mod — would match `name = matches("INNER_*")` if synthesis
    // descended into mod blocks. Currently invisible to synthesis.
    pub struct INNER_StructSite;
}
