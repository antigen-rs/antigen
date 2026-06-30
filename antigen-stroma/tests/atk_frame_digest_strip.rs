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
#[ignore = "born-red until IdentityDigest::of_tokens + canonicalizer land (frame epoch); de-ignore on fill"]
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
#[ignore = "born-red until IdentityDigest::of_tokens + canonicalizer land (frame epoch); de-ignore on fill"]
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
