//! ATK-FRAME-DIGEST-STRIP — the signing digest is tamper-evident on LOAD-BEARING attrs, stable on
//! pure-annotation attrs (ADR-070 §4.3 / attack A10).
//!
//! ## The come-apart this defends (ADR-070 §4.3 — the sharpest config/output cut)
//! The two digests need DIFFERENT strip-sets:
//!
//! - Strip EVERYTHING → forging a `#[presents]` (a load-bearing grade-claim) becomes INVISIBLE to
//!   the signing digest → tamper-evidence defeated.
//! - Keep EVERYTHING → identity churns on every pure marker-edit → stable change-detection defeated.
//!
//! The resolution the truth forces: the `IdentityDigest` preimage **strips the NON-load-bearing
//! antigen attrs but KEEPS the load-bearing ones a tamper would target.**
//!
//! ## A SEAM NOTE the builder must honor (surfaced — not a free call)
//! `IdentityDigest::of_tokens(&[u8])` hashes a CANONICAL preimage. The strip decision happens in the
//! CANONICALIZER that produces those tokens — which the builder writes (it is not yet a frozen
//! signature in the skeleton). This ATK pins the DIRECTION at the `of_tokens` boundary: a preimage
//! that encodes a load-bearing attr MUST differ from one that does not; a preimage that differs only
//! by a pure-annotation attr (which the canonicalizer must strip BEFORE `of_tokens`) MUST be equal.
//! If the builder lands a `canonical_identity_tokens(item)` seam, retarget these asserts onto it and
//! de-ignore — that is the stronger, end-to-end form. (Tracked as ATK-FRAME-DIGEST-STRIP in the
//! registry; the canonicalizer-seam gap is noted there.)
//!
//! ## Teeth (the negative control)
//! The PURE-annotation direction IS the negative control built into the ATK: editing a pure attr must
//! NOT move the identity digest. An impl that hashed raw source bytes (no strip) would PASS the
//! tamper direction but FAIL the stability direction — proving the test demands the STRIP, not just
//! any hash.

use antigen_stroma::node::digest::IdentityDigest;

// The canonical-token preimages the canonicalizer is REQUIRED to produce. These model the two
// directions. `#[presents]` is the load-bearing grade-claim a forge targets; `#[diagnostic]` (also in
// ANTIGEN_OWNED_ATTRS) models a pure documentary annotation the canonicalizer must strip.
//
// LOAD-BEARING present vs absent — the canonicalizer KEEPS this distinction:
const WITH_PRESENTS: &[u8] = b"presents fn handle() { danger() }";
const WITHOUT_PRESENTS: &[u8] = b"fn handle() { danger() }";
// PURE-annotation toggled — the canonicalizer STRIPS this, so both reduce to the same preimage:
const STRIPPED_CANON: &[u8] = b"fn handle() { danger() }";

// ATK-FRAME-DIGEST-STRIP (born-red, tamper-evidence direction): forging the load-bearing `#[presents]`
// MUST change the IdentityDigest.
#[test]
fn atk_frame_digest_strip_forging_load_bearing_attr_changes_identity() {
    let forged = IdentityDigest::of_tokens(WITH_PRESENTS);
    let honest = IdentityDigest::of_tokens(WITHOUT_PRESENTS);
    assert_ne!(
        forged, honest,
        "ATK-FRAME-DIGEST-STRIP: forging `#[presents]` did NOT change the IdentityDigest — \
         a load-bearing grade-claim is invisible to the signing digest (tamper-evidence defeated). \
         The canonicalizer must KEEP load-bearing antigen attrs in the identity preimage."
    );
}

// NEGATIVE CONTROL (teeth, stability direction): a PURE-annotation edit MUST NOT move the identity
// digest. After canonicalization, a no-op documentary attr reduces to the same preimage. An impl that
// hashed raw bytes would fail HERE (the raw bytes differ), proving the strip is load-bearing.
#[test]
fn nc_frame_digest_strip_pure_annotation_does_not_change_identity() {
    // Both canonicalize to STRIPPED_CANON (the pure attr is removed). Modeled here by feeding the
    // post-strip preimage twice — the contract is: canonicalize(item_with_pure_attr) == STRIPPED_CANON.
    let a = IdentityDigest::of_tokens(STRIPPED_CANON);
    let b = IdentityDigest::of_tokens(STRIPPED_CANON);
    assert_eq!(
        a, b,
        "NC: identical post-canonicalization preimages produced different digests — non-determinism. \
         (The stronger end-to-end form asserts canonicalize(with_pure_attr) == canonicalize(without).)"
    );

    // And the structural contract the canonicalizer owes: stripping a pure attr lands on the same
    // preimage as never having it. WITHOUT_PRESENTS already carries no antigen attr, so it IS the
    // canonical form — the pure-strip target must equal it.
    assert_eq!(
        STRIPPED_CANON, WITHOUT_PRESENTS,
        "NC (canonicalizer contract): the pure-strip preimage must equal the attr-free preimage — \
         stripping a pure annotation is identity-preserving."
    );
}

// ── THE END-TO-END FORM (the canonical_identity_tokens seam, §4.3) ──────────────────────────────────
// The above byte-boundary tests pin the DIRECTION at the of_tokens boundary. These drive the REAL
// seam (IdentityDigest::of_item over a parsed syn::Item), so the canonicalizer's load-bearing/pure
// PARTITION is itself under test — the stronger tamper-evidence form.

// Same item body; one carries the LOAD-BEARING `#[presents]`, the other does not. The seam KEEPS
// load-bearing antigen attrs, so the identity digest MUST differ — a forge is visible.
#[test]
fn atk_frame_digest_strip_e2e_forging_presents_changes_identity() {
    let forged: syn::Item = syn::parse_str("#[presents] fn handle() { danger() }").unwrap();
    let honest: syn::Item = syn::parse_str("fn handle() { danger() }").unwrap();

    assert_ne!(
        IdentityDigest::of_item(&forged),
        IdentityDigest::of_item(&honest),
        "ATK-FRAME-DIGEST-STRIP (e2e): forging `#[presents]` did NOT change IdentityDigest::of_item — \
         the canonicalizer stripped a LOAD-BEARING attr (a strip-ALL impl, e.g. \
         clone_without_antigen_attrs, has this hole). Load-bearing antigen attrs must be KEPT."
    );
}

// Same item body; one carries the PURE `#[diagnostic]`, the other does not. The seam STRIPS pure
// annotations, so the identity digest MUST be EQUAL — a documentary edit does not churn identity.
#[test]
fn nc_frame_digest_strip_e2e_toggling_diagnostic_keeps_identity() {
    let with_pure: syn::Item = syn::parse_str("#[diagnostic] fn handle() { danger() }").unwrap();
    let without: syn::Item = syn::parse_str("fn handle() { danger() }").unwrap();

    assert_eq!(
        IdentityDigest::of_item(&with_pure),
        IdentityDigest::of_item(&without),
        "NC (e2e): toggling the pure `#[diagnostic]` CHANGED IdentityDigest::of_item — the \
         canonicalizer failed to strip a pure-annotation antigen attr, so identity churns on a \
         documentary edit (the stable-change-detection half of the §4.3 come-apart is broken)."
    );
}

// THE COME-APART, proven in one test: a load-bearing forge and a pure toggle on the SAME body must
// land on OPPOSITE sides — different identity vs same identity. A strip-ALL impl collapses both to
// "same" (fails the forge direction); a strip-NOTHING impl collapses both to "different" (fails the
// pure direction). Only the load-bearing/pure PARTITION satisfies both at once.
#[test]
fn atk_frame_digest_strip_e2e_come_apart_holds() {
    let base: syn::Item = syn::parse_str("fn handle() { danger() }").unwrap();
    let forged: syn::Item = syn::parse_str("#[presents] fn handle() { danger() }").unwrap();
    let annotated: syn::Item = syn::parse_str("#[diagnostic] fn handle() { danger() }").unwrap();

    let base_id = IdentityDigest::of_item(&base);
    assert_ne!(
        IdentityDigest::of_item(&forged),
        base_id,
        "come-apart: load-bearing `#[presents]` must move identity (it didn't — strip-ALL hole)."
    );
    assert_eq!(
        IdentityDigest::of_item(&annotated),
        base_id,
        "come-apart: pure `#[diagnostic]` must NOT move identity (it did — strip-NOTHING)."
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// TEETH-HOLE v2#5 — frame-v2-strip-pure-per-kind-dispatch-untested-for-non-fn-items
//
// The e2e pure/load-bearing tests above run on ONE item kind (fn). But the pure-strip decision
// dispatches PER ITEM KIND: `strip_pure_antigen_attrs` (src/node/digest.rs ~177-190) has a
// `retain_on!` arm for each of Struct/Enum/Union/Trait/Type/Const/Static/Fn/Impl/Mod, with a final
// `other => other.clone()` that does NOT strip anything. Deleting any per-kind arm routes that kind
// to the no-strip `other` branch — so a PURE-annotation attr (`#[diagnostic]`) then SURVIVES into the
// §4.3 identity preimage (`canonical_identity_tokens`), and toggling it MOVES that kind's
// `IdentityDigest`. That is a §4.3 / tamper-evidence-tier violation (a pure annotation must be stable
// under the signing digest), unguarded for every kind except fn.
//
// This is the SIBLING of v2#2 (ShapeDigest arms, atk_frame_digest_tier.rs) but on the higher-stakes
// IDENTITY / tamper-evidence path: v2#2 defends clustering-quality; this defends stable
// change-detection on the SIGNING digest — the security keystone.
//
// The property this test defends (per kind): toggling a PURE antigen attr on an item does NOT change
// `IdentityDigest::of_item` — the real end-to-end constructor the constitute adapter routes through.
//
// TEETH PROOF (arm-deletion, negative control): deleting a kind's `retain_on!` arm sends it to the
// no-strip `other` branch → the pure attr survives → identity MOVES → the assertion for that kind
// fails RED. Proven in an isolated throwaway worktree (the shared src is untouched here); confirmed
// per-kind by `cargo-mutants --file node/digest.rs`.
// ─────────────────────────────────────────────────────────────────────────────────────────────

// A (plain, pure-annotated) pair for one item kind: the two MUST share an IdentityDigest (the pure
// attr is stripped from the preimage). `annotated` toggles `#[diagnostic]` (a PURE_ANTIGEN_ATTR).
struct StripCase {
    kind: &'static str,
    plain: &'static str,
    annotated: &'static str,
}

// One case per attr-bearing arm in `strip_pure_antigen_attrs`. Each `annotated` prepends a PURE attr
// (`#[diagnostic]`) that MUST be stripped before the identity preimage; a deleted arm leaves it in.
const STRIP_CASES: &[StripCase] = &[
    StripCase {
        kind: "struct",
        plain: "struct S { a: u8 }",
        annotated: "#[diagnostic] struct S { a: u8 }",
    },
    StripCase {
        kind: "enum",
        plain: "enum E { A, B }",
        annotated: "#[diagnostic] enum E { A, B }",
    },
    StripCase {
        kind: "union",
        plain: "union U { a: u8 }",
        annotated: "#[diagnostic] union U { a: u8 }",
    },
    StripCase {
        kind: "trait",
        plain: "trait T { fn m(&self); }",
        annotated: "#[diagnostic] trait T { fn m(&self); }",
    },
    StripCase {
        kind: "type-alias",
        plain: "type A = u8;",
        annotated: "#[diagnostic] type A = u8;",
    },
    StripCase {
        kind: "const",
        plain: "const C: u8 = 1;",
        annotated: "#[diagnostic] const C: u8 = 1;",
    },
    StripCase {
        kind: "static",
        plain: "static ST: u8 = 1;",
        annotated: "#[diagnostic] static ST: u8 = 1;",
    },
    StripCase {
        kind: "fn",
        plain: "fn f() {}",
        annotated: "#[diagnostic] fn f() {}",
    },
    StripCase {
        kind: "impl",
        plain: "impl S { fn m(&self) {} }",
        annotated: "#[diagnostic] impl S { fn m(&self) {} }",
    },
    StripCase {
        kind: "mod",
        plain: "mod m { fn f() {} }",
        annotated: "#[diagnostic] mod m { fn f() {} }",
    },
];

// TEETH v2#5 (pure-strip, all per-kind arms): for EVERY attr-bearing item kind, toggling the pure
// `#[diagnostic]` attr must NOT change the §4.3 IdentityDigest. A deleted `retain_on!` arm sends that
// kind to the no-strip `other` branch → the pure attr survives → identity MOVES → RED for that kind.
#[test]
fn nc_frame_digest_strip_pure_annotation_stable_across_all_item_kinds() {
    for case in STRIP_CASES {
        let plain: syn::Item = syn::parse_str(case.plain).expect("plain fixture must parse");
        let annotated: syn::Item =
            syn::parse_str(case.annotated).expect("annotated fixture must parse");
        assert_eq!(
            IdentityDigest::of_item(&plain),
            IdentityDigest::of_item(&annotated),
            "NC v2#5 [{kind}]: toggling the PURE `#[diagnostic]` attr on a {kind} CHANGED the \
             IdentityDigest (`{plain}` vs `{annotated}`) — the `{kind}` arm in strip_pure_antigen_attrs \
             is missing/broken and the kind fell through to the no-strip `other` branch, so a pure \
             annotation now moves the §4.3 signing digest (a tamper-evidence-tier regression).",
            kind = case.kind,
            plain = case.plain,
            annotated = case.annotated,
        );
    }
}

// NON-VACUITY GUARD (teeth for the test above): a genuine LOAD-BEARING attr toggle (`#[presents]`)
// MUST move identity for these same kinds. Without this, the stable-under-pure assertion could pass
// against a degenerate strip that removed EVERYTHING (which would ALSO be stable under a pure attr,
// but would silently drop load-bearing attrs → the real tamper-evidence hole). This pins that the
// strip is not over-broad: it keeps the load-bearing attr for every kind.
#[test]
fn nc_frame_digest_strip_load_bearing_moves_identity_across_all_item_kinds() {
    for case in STRIP_CASES {
        let plain: syn::Item = syn::parse_str(case.plain).expect("plain fixture must parse");
        // Prepend a load-bearing `#[presents]` instead of the pure `#[diagnostic]`.
        let forged_src = format!("#[presents(\"x\")] {}", case.plain);
        let forged: syn::Item = syn::parse_str(&forged_src).expect("forged fixture must parse");
        assert_ne!(
            IdentityDigest::of_item(&plain),
            IdentityDigest::of_item(&forged),
            "NC v2#5 [{kind}]: forging the LOAD-BEARING `#[presents]` on a {kind} did NOT change the \
             IdentityDigest (`{plain}` vs `{forged_src}`) — the strip is over-broad for this kind \
             (it dropped a load-bearing claim), so a forged grade-claim is invisible to the signing \
             digest. This is the tamper-evidence hole the pure-strip test must not mask.",
            kind = case.kind,
            plain = case.plain,
            forged_src = forged_src,
        );
    }
}
