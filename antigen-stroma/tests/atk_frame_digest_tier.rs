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

use antigen_stroma::node::digest::{IdentityDigest, ShapeDigest, is_load_bearing_antigen_attr};

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

// ─────────────────────────────────────────────────────────────────────────────────────────────
// TEETH-HOLE v2#2 — frame-v2-shapedigest-name-insensitivity-untested-for-non-struct-items
//
// The NC above tests name-insensitivity for ONE item kind (struct). `ShapeDigest::of_item`
// (src/node/digest.rs) has a match with a SEPARATE ident-normalizing arm per ident-bearing kind:
// Struct / Enum / Union / Trait / Type / Const / Static / Fn. cargo-mutants proved 7 of those 8
// arms could be DELETED with the suite still green — deleting a kind's arm makes it fall through to
// the `_ =>` raw-DefaultHasher branch, which is NAME-SENSITIVE (it hashes the raw source string, no
// ident placeholder-normalization). So for every non-struct kind the clustering property was UNGUARDED.
//
// The property this test class defends (per ident-bearing kind):
//   (a) name-insensitivity — two structurally-identical items differing ONLY in name share a
//       ShapeDigest. (This is the direct teeth-check on each arm: a deleted arm falls through to the
//       name-SENSITIVE raw branch, giving distinct digests → RED.)
//   (b) shape-sensitivity — a genuine structural change DOES move the digest. Without (b), (a) could
//       pass vacuously against a digest that collapses everything to one value.
//
// TEETH PROOF (arm-deletion, negative control): deleting any of the 8 arms in `of_item` routes that
// kind through the raw-token `_` branch, which is name-sensitive → the (a) assertion for that kind
// fails RED. Verified by construction here; cargo-mutants confirms the full 8/8 sweep.
// ─────────────────────────────────────────────────────────────────────────────────────────────

// A structurally-identical pair (differing ONLY in the top-level name) plus a shape-changed variant,
// for one item kind. `same_a`/`same_b` MUST share a ShapeDigest (name-insensitive); `shifted` MUST
// differ from `same_a` (shape-sensitive).
struct ShapeCase {
    kind: &'static str,
    same_a: &'static str,
    same_b: &'static str,
    shifted: &'static str,
}

// One case per ident-bearing arm in `ShapeDigest::of_item`. Each `shifted` perturbs the STRUCTURE
// (a field type, an arm count, a signature) — never merely the name — so it exercises the
// shape-sensitivity half distinctly from the name-insensitivity half.
const SHAPE_CASES: &[ShapeCase] = &[
    ShapeCase {
        kind: "struct",
        same_a: "struct Foo { a: u8 }",
        same_b: "struct Bar { a: u8 }",
        shifted: "struct Foo { a: u16 }",
    },
    ShapeCase {
        kind: "enum",
        same_a: "enum Foo { A, B }",
        same_b: "enum Bar { A, B }",
        shifted: "enum Foo { A, B, C }",
    },
    ShapeCase {
        kind: "union",
        same_a: "union Foo { a: u8 }",
        same_b: "union Bar { a: u8 }",
        shifted: "union Foo { a: u16 }",
    },
    ShapeCase {
        kind: "trait",
        same_a: "trait Foo { fn m(&self); }",
        same_b: "trait Bar { fn m(&self); }",
        shifted: "trait Foo { fn m(&self, x: u8); }",
    },
    ShapeCase {
        kind: "type-alias",
        same_a: "type Foo = u8;",
        same_b: "type Bar = u8;",
        shifted: "type Foo = u16;",
    },
    ShapeCase {
        kind: "const",
        same_a: "const FOO: u8 = 1;",
        same_b: "const BAR: u8 = 1;",
        shifted: "const FOO: u16 = 1;",
    },
    ShapeCase {
        kind: "static",
        same_a: "static FOO: u8 = 1;",
        same_b: "static BAR: u8 = 1;",
        shifted: "static FOO: u16 = 1;",
    },
    ShapeCase {
        kind: "fn",
        same_a: "fn foo(x: u8) {}",
        same_b: "fn bar(x: u8) {}",
        shifted: "fn foo(x: u16) {}",
    },
];

// TEETH v2#2 (name-insensitivity, all 8 arms): for EVERY ident-bearing item kind, two structurally-
// identical items with different names share a ShapeDigest. A deleted arm falls through to the
// name-SENSITIVE raw branch → RED for that kind. One test kills all 7 previously-missed mutants.
#[test]
fn nc_frame_shape_digest_name_insensitive_across_all_item_kinds() {
    for case in SHAPE_CASES {
        let a = ShapeDigest::of_item(case.same_a);
        let b = ShapeDigest::of_item(case.same_b);
        assert_eq!(
            a,
            b,
            "NC v2#2 [{kind}]: ShapeDigest distinguished two same-shape {kind} items by name \
             (`{sa}` vs `{sb}`) — the `{kind}` arm in ShapeDigest::of_item is missing/broken and the \
             kind fell through to the name-SENSITIVE raw-token branch, losing its clustering role.",
            kind = case.kind,
            sa = case.same_a,
            sb = case.same_b,
        );
    }
}

// TEETH v2#2 (shape-sensitivity, the non-vacuity guard): a genuine STRUCTURAL change DOES move the
// ShapeDigest. Without this, the name-insensitivity assertion could pass against a digest that
// collapses every input to one value (which would ALSO be name-insensitive, but useless as a
// clustering key). This pins that ShapeDigest still discriminates on structure.
#[test]
fn nc_frame_shape_digest_shape_sensitive_across_all_item_kinds() {
    for case in SHAPE_CASES {
        let base = ShapeDigest::of_item(case.same_a);
        let shifted = ShapeDigest::of_item(case.shifted);
        assert_ne!(
            base,
            shifted,
            "NC v2#2 [{kind}]: ShapeDigest collapsed a genuine structural change \
             (`{sa}` vs `{sh}`) to the SAME digest — the shape tier lost structural discrimination, \
             so the name-insensitivity assertion above would pass vacuously.",
            kind = case.kind,
            sa = case.same_a,
            sh = case.shifted,
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// TEETH-HOLE v2#3 — frame-v2-is-load-bearing-public-helper-has-no-teeth
//
// `is_load_bearing_antigen_attr` (pub, src/node/digest.rs) is a PUBLIC API surface — an organ
// reasoning about tamper-surface asks "is this attr security-load-bearing?" through it. cargo-mutants
// proved BOTH whole-body replacements (-> true, -> false) survived: it had ZERO regression coverage.
//
// SUBTLETY (from the detail-auditor + adversarial): is_load_bearing and is_pure are COMPLEMENTS only
// WITHIN the antigen-owned universe (the partition_guard proves PURE ⊎ LOAD == OWNED). Over a
// NON-antigen attr BOTH return false. So the contract is a TRIPLE, not a single case:
//   (a) a load-bearing antigen attr (#[presents]) → true
//   (b) a pure antigen attr (#[diagnostic])       → false
//   (c) a non-antigen attr (#[derive(Debug)])     → false
//
// TEETH PROOF (body-replacement, negative control): the `-> true` mutant fails on (b)/(c); the
// `-> false` mutant fails on (a). The triple kills both missed mutants.
// ─────────────────────────────────────────────────────────────────────────────────────────────

// Parse a single `#[..]`-attributed item and return its first top-level attribute — the honest way
// to obtain a real `syn::Attribute` (the same last-segment path an organ would hand the helper).
fn first_attr(attributed_item_src: &str) -> syn::Attribute {
    let item: syn::Item =
        syn::parse_str(attributed_item_src).expect("fixture must parse as an attributed syn::Item");
    let attrs = match item {
        syn::Item::Struct(it) => it.attrs,
        syn::Item::Fn(it) => it.attrs,
        other => panic!("fixture item kind carries no attrs surface for this test: {other:?}"),
    };
    attrs
        .into_iter()
        .next()
        .expect("fixture must carry at least one top-level attribute")
}

// TEETH v2#3: pin the (load-bearing → true, pure → false, non-antigen → false) contract of the public
// helper. The `-> true` body-mutant fails on the pure/non-antigen cases; the `-> false` body-mutant
// fails on the load-bearing case. Both previously-missed mutants now go RED.
#[test]
fn atk_is_load_bearing_antigen_attr_contract_triple() {
    // (a) load-bearing antigen attr — a forgeable grade-claim the identity digest KEEPS → true.
    let load_bearing = first_attr("#[presents] struct S;");
    assert!(
        is_load_bearing_antigen_attr(&load_bearing),
        "v2#3(a): is_load_bearing_antigen_attr(#[presents]) was false — a load-bearing antigen attr \
         (a forgeable claim the identity digest must KEEP) was mis-classified as not-load-bearing. \
         A `-> false` body-mutant would produce exactly this failure."
    );

    // (b) pure antigen attr — a documentary annotation the identity digest STRIPS → false.
    let pure = first_attr("#[diagnostic] struct S;");
    assert!(
        !is_load_bearing_antigen_attr(&pure),
        "v2#3(b): is_load_bearing_antigen_attr(#[diagnostic]) was true — a PURE (documentary) antigen \
         attr was mis-classified as load-bearing. A `-> true` body-mutant would produce this failure."
    );

    // (c) non-antigen attr — outside the antigen-owned universe → false (NOT load-bearing).
    // is_pure and is_load_bearing are complements only WITHIN the owned set; outside it BOTH are false.
    let non_antigen = first_attr("#[derive(Debug)] struct S;");
    assert!(
        !is_load_bearing_antigen_attr(&non_antigen),
        "v2#3(c): is_load_bearing_antigen_attr(#[derive(Debug)]) was true — a NON-antigen attr was \
         claimed as security-load-bearing. Load-bearing is scoped to ANTIGEN_OWNED_ATTRS; a \
         `-> true` body-mutant would produce this failure."
    );
}
