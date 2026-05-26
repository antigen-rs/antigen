// Fixture: synthesis-pass SILENT MISS on const items.
//
// synthesis_pass() calls item_kind_and_target(syn_item) for each top-level item.
// For syn::Item::Const the match arm is `_ => None`, causing `continue` — the
// item is NEVER evaluated against any declared fingerprint.
//
// Consequence: a fingerprint without an `item = <kind>` pin (e.g. only
// `name = matches("SENTINEL_*")`) should fire for BOTH struct and const items
// matching the glob. It fires for the struct (ItemTarget::Struct goes through
// item_kind_and_target), but NOT for the const (silently skipped).
//
// This is the ParallelStateTrackersDiverge anti-pattern at the scanner's own
// design level: Pass 1 (attribute scanning) has visit_item_const; Pass 2
// (fingerprint synthesis) lacks the matching arm.

#[antigen(name = "sentinel-silent-miss", fingerprint = "name = matches(\"SENTINEL_*\")")]
pub struct SentinelSilentMiss;

// Matches the fingerprint — struct items ARE evaluated in synthesis.
pub struct SENTINEL_StructSite;

// Matches the fingerprint by name but is NEVER evaluated — item_kind_and_target
// returns None for syn::Item::Const, so synthesis silently skips it.
pub const SENTINEL_CONST_SITE: usize = 42;
