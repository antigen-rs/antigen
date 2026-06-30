//! ATK-FRAME-DIGEST-TIER — the identity digest is collision-RESISTANT (BLAKE3), not FNV.
//!
//! ## The claim this defends (ADR-070 §4.3, §6; ADR-067 §A.2)
//! Two distinct digests, security-tiered: `IdentityDigest` (BLAKE3, signing — "FNV-1a is
//! engineer-collidable, FORBIDDEN for identity") vs `ShapeDigest` (FNV-1a, clustering/backdate).
//! Using FNV for identity makes the stroma engineer-collidable — a security-class SILENT failure.
//! The load-bearing, falsifiable property: distinct item preimages get distinct `IdentityDigest`s,
//! and `IdentityDigest` is the 32-byte collision-resistant type, NEVER the FNV `String` shape digest.
//!
//! ## Born-red status
//! `#[ignore]` — `IdentityDigest::of_tokens` is `todo!()`. De-ignore on fill.
//!
//! ## Teeth (the negative control)
//! `nc_*` proves the test targets the IDENTITY tier only: the `ShapeDigest` (FNV) is ALLOWED to be
//! name-insensitive (two same-shape, different-name items share a shape digest) — the very property
//! that disqualifies it for identity. The NC shows the ATK is not a blanket "all digests must differ".

use antigen_stroma::node::digest::{IdentityDigest, ShapeDigest};

// ATK-FRAME-DIGEST-TIER (born-red): distinct preimages => distinct collision-resistant IdentityDigest.
#[test]
fn atk_frame_digest_tier_distinct_items_distinct_identity_digest() {
    // Two items whose canonical tokens differ — even by one byte — must get distinct identity digests.
    let a = IdentityDigest::of_tokens(b"struct Foo { a: u8 }");
    let b = IdentityDigest::of_tokens(b"struct Foo { a: u16 }");

    assert_ne!(
        a, b,
        "ATK-FRAME-DIGEST-TIER: two distinct items produced the SAME IdentityDigest — \
         the signing digest is not behaving as a collision-resistant hash over the preimage."
    );

    // The structural pin: IdentityDigest is the 32-byte BLAKE3 tier, NOT the FNV String shape tier.
    // (A builder who routed identity through FNV would be unable to satisfy the [u8; 32] type — this
    // assertion makes the tier visible at the type level even before a true collision is constructed.)
    let IdentityDigest(bytes) = a;
    assert_eq!(
        bytes.len(),
        32,
        "ATK-FRAME-DIGEST-TIER: IdentityDigest is not the 32-byte (BLAKE3-class) signing tier."
    );
}

// ATK-FRAME-DIGEST-TIER (born-red, determinism): the signing digest is a PURE function of the
// preimage — same tokens, same digest. (Tamper-evidence requires reproducibility.)
#[test]
fn atk_frame_digest_tier_is_deterministic() {
    let a = IdentityDigest::of_tokens(b"fn quux() {}");
    let b = IdentityDigest::of_tokens(b"fn quux() {}");
    assert_eq!(
        a, b,
        "ATK-FRAME-DIGEST-TIER: the signing digest is non-deterministic — identical preimages \
         hashed to different digests. A non-reproducible signing digest cannot be tamper-evidence."
    );
}

// NEGATIVE CONTROL (teeth): the ShapeDigest (FNV) is name-INSENSITIVE — two items with the same
// structure but different names MAY share a shape digest. This is the property that disqualifies FNV
// for identity but is CORRECT for clustering/backdate. The NC proves the ATK targets the identity
// tier, not a blanket "every digest distinguishes everything".
#[test]
fn nc_frame_shape_digest_is_name_insensitive() {
    // Same structure, different name. The shape digest strips the name (ADR-070 §4.3) so these MAY
    // collide. If a builder accidentally made ShapeDigest name-SENSITIVE, this NC fails and signals
    // the shape tier drifted toward the identity tier (losing its clustering purpose).
    let foo = ShapeDigest::of_item("struct Foo { a: u8 }");
    let bar = ShapeDigest::of_item("struct Bar { a: u8 }");
    assert_eq!(
        foo, bar,
        "NC: ShapeDigest distinguished two same-shape items by name — the shape tier became \
         name-sensitive and lost its clustering/backdate role (it is now mis-tiered toward identity)."
    );
}
