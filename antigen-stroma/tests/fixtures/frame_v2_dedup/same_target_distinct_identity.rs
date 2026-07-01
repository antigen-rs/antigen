// SPECIMEN (frame-v2#1 · frame-v2-dedup-key-collapses-same-itemtarget)
//
// Two `impl Foo` blocks in ONE file. Both scan to the SAME `ItemTarget`
// (`Impl { trait_path: None, target_type: "Foo" }`) — the `NodeKey`'s
// `format!("{item_target:?}")` renders them identically — yet they have
// DISTINCT bodies, so `canonical_identity_tokens` yields DISTINCT
// `IdentityDigest`s. Under the current `(file, ItemTarget)` dedup the second
// block is silently dropped BEFORE its identity is ever consulted (§4.1:
// ItemTarget is NOT identity). Under the fixed identity-keyed dedup, both
// survive as distinct `StromaNodeId`s.
//
// This file is a static FIXTURE — it lives under tests/fixtures/ and is NOT
// compiled as part of the crate. It is READ by `digests_at_line` via
// std::fs::read_to_string + syn::parse_file; the two impl blocks must sit at
// stable line numbers the fixture builder pins (asserted in-test, not hard-coded
// here, so an edit that shifts the lines fails loud rather than silently mis-targeting).

struct Foo;

impl Foo {
    fn alpha(&self) -> u8 {
        1
    }
}

impl Foo {
    fn beta(&self) -> u8 {
        2
    }
}
