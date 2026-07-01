// SPECIMEN (frame-v2#4 · digests_at_line CONTAINMENT semantic): ONE item carrying two antigen attrs
// on DIFFERENT source lines. `syn` folds outer attrs INTO the item span, so the item's span-start is
// the FIRST attr line and its span-end is the closing brace — the whole `[first_attr ..= close]` range.
// A scan emits a record per attr-site, so this item produces records at TWO distinct lines (the
// `#[presents]` line and the `#[immune]` line), both INSIDE the one span.
//
// Under exact span-start matching (`== target_line`), only the record on the span-start line resolves
// to the item; the other misses every item → gap-digest → (because the dedup keys on identity) SPLITS
// into a spurious SECOND node. Under CONTAINMENT matching (`start..=end`), both records resolve to the
// SAME containing item → SAME IdentityDigest → dedup to ONE node (the intended cross-attr merge).
//
// This is the come-apart that is BORN-RED against exact-match and GREEN under containment. It is
// routine in antigen's own dogfooding (`#[presents]` + `#[immune]` on the same item), which is why the
// dedup fix (key on identity) made closing it REQUIRED — without containment, every such item silently
// doubles.

struct Plain;

#[cfg_attr(all(), doc = "presents-marker-line")]
#[cfg_attr(all(), doc = "immune-marker-line")]
struct TwoAttrItem {
    field: u8,
}
