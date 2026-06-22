// Scan fixture for the P0a ordering-regression guard (TEST 3). Parsed-as-text.
//
// Item A (#[dread], body X) immediately followed by item B (#[dread], body Y!=X)
// at module top level, in SOURCE ORDER. The guard: B's marked-unknown digest must
// equal structural_digest(B), NOT structural_digest(A).
//
// This is a regression fence against a NAIVE fix that captures
// `current_item_digest` too early (before the per-item recompute), which would
// make B silently inherit A's stale digest. The current visit-order is already
// correct (current_item_digest is recomputed per item before
// each check_attrs) — so this test should PASS once the seam is wired, and it must
// STAY green under any future refactor of the capture point.

#[dread(trigger = "item A: the first marked shape; its digest must not bleed onto the next item")]
struct ItemA {
    only_field_a: String,
}

#[dread(trigger = "item B: a distinct shape immediately after A; must carry its OWN digest, not A's")]
struct ItemB {
    field_one: u64,
    field_two: bool,
    field_three: Vec<u8>,
}
