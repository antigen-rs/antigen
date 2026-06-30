//! ATK-FRAME-DIGEST-STRIP-E2E — the §4.3 come-apart, settled EXECUTABLY against the real path.
//!
//! ## The disagreement this adjudicates (the frame's last gate before survey)
//! The builder found `lower_scan_report` uses `clone_without_antigen_attrs()`, which strips ALL
//! antigen attrs (incl. load-bearing `#[presents]`/`#[defended_by]`) and worried the IDENTITY digest
//! inherits that strip → forging `#[presents]` would be invisible (tamper-evidence hole). The
//! build-scout called it "not a real gap." The prose can't settle it; this test does.
//!
//! ## The §4.3 come-apart, both directions, against the REAL end-to-end path
//! `IdentityDigest::of_item(&syn::Item)` is the actual identity-digest constructor the constitute
//! adapter routes through (`extract_item_at_line` → `canonical_identity_tokens`). This ATK exercises
//! THAT path — not the `of_tokens` byte-boundary the weaker DIGEST-STRIP test reached.
//!   - LOAD-BEARING attr change (`#[presents]`) → identity digests MUST DIFFER (forging detectable).
//!   - PURE/documentary attr change (`#[diagnostic]`) → identity digests MUST MATCH (pure attr stripped).
//!
//! ## The verdict protocol (test-architect settles it)
//! RED on the load-bearing case (forging `#[presents]` leaves the digest equal) ⇒ the hole is REAL ⇒
//! the builder fixes the identity path to route through `canonical_identity_tokens` (keep load-bearing).
//! GREEN ⇒ no hole, the identity path already keeps load-bearing attrs ⇒ the frame's tamper-evidence
//! holds. Either way the executable answer beats the disagreement. NOT `#[ignore]`d — it runs NOW
//! against the current build, because settling the disagreement is the whole point.

use antigen_stroma::node::digest::IdentityDigest;

fn item(src: &str) -> syn::Item {
    syn::parse_str(src).expect("fixture must parse as a syn::Item")
}

// THE ATK (load-bearing direction): forging a load-bearing `#[presents]` MUST change the identity
// digest. If RED, the strip is over-broad and a forged grade-claim is invisible — the tamper-evidence
// hole the builder flagged.
#[test]
fn atk_frame_digest_strip_e2e_forging_presents_changes_identity() {
    let honest = item("fn handle() { danger() }");
    let forged = item("#[presents(\"x\")] fn handle() { danger() }");

    let d_honest = IdentityDigest::of_item(&honest);
    let d_forged = IdentityDigest::of_item(&forged);

    assert_ne!(
        d_honest, d_forged,
        "ATK-FRAME-DIGEST-STRIP-E2E: forging `#[presents]` did NOT change the IdentityDigest — \
         the identity path strips load-bearing antigen attrs (the tamper-evidence hole is REAL). \
         FIX: route identity through `canonical_identity_tokens` (strip PURE attrs only, KEEP \
         load-bearing). The build-scout's 'not a real gap' is refuted by this RED."
    );
}

// ALSO load-bearing: `#[defended_by]` (the ADR-029 code-tier witness — a forge target).
#[test]
fn atk_frame_digest_strip_e2e_forging_defended_by_changes_identity() {
    let honest = item("fn t() {}");
    let forged = item("#[defended_by(\"x\")] fn t() {}");
    assert_ne!(
        IdentityDigest::of_item(&honest),
        IdentityDigest::of_item(&forged),
        "ATK-FRAME-DIGEST-STRIP-E2E: forging `#[defended_by]` did NOT change identity — a witness \
         mark a tamper would target is invisible to the signing digest."
    );
}

// NEGATIVE CONTROL (teeth, stability direction): toggling a PURE/documentary attr (`#[diagnostic]`,
// in PURE_ANTIGEN_ATTRS) MUST NOT change the identity digest. If this FAILED, the strip would be too
// NARROW (identity churns on every pure annotation, defeating stable change-detection). This is the
// other arm of the come-apart — and it's what makes the ATK above non-vacuous (an impl that hashed
// raw bytes would pass the ATK but FAIL here).
#[test]
fn nc_frame_digest_strip_e2e_pure_annotation_does_not_change_identity() {
    let plain = item("fn handle() { danger() }");
    let annotated = item("#[diagnostic] fn handle() { danger() }");

    assert_eq!(
        IdentityDigest::of_item(&plain),
        IdentityDigest::of_item(&annotated),
        "NC: toggling the pure documentary `#[diagnostic]` CHANGED the IdentityDigest — the strip is \
         too narrow (identity churns on a pure annotation, defeating stable change-detection). \
         `#[diagnostic]` must be in the pure-strip set."
    );
}

// NEGATIVE CONTROL (teeth, the wrapper attr): the bare `#[antigen]` wrapper is pure (a container, not
// a claim) — toggling it must NOT change identity.
#[test]
fn nc_frame_digest_strip_e2e_bare_antigen_wrapper_does_not_change_identity() {
    let plain = item("struct S { a: u8 }");
    let wrapped = item("#[antigen] struct S { a: u8 }");
    assert_eq!(
        IdentityDigest::of_item(&plain),
        IdentityDigest::of_item(&wrapped),
        "NC: the bare `#[antigen]` wrapper changed identity — a container attr is being treated as \
         load-bearing (over-narrow strip)."
    );
}
