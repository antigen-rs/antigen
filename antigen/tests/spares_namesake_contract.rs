//! # Spares-Namesake Contract — the interim enforcement of ADR-039 §C Amendment 1.
//!
//! ## Why this file exists
//!
//! ADR-039 §C's affinity-pair requires a Constructable member to *spare a clean
//! sibling*. The beta.2 Geological Society seal found that the in-member
//! affinity-pairs (in `stdlib_family_fingerprints.rs`) only spared the
//! **trivially-absent** sibling (a *different* method name, or no call) — never the
//! **same-method NAMESAKE on a clean receiver**, which is the codomain the
//! receiver-agnostic `body_calls` leaf actually reaches. That gap let four named
//! members over-claim (a "passing test, wrong answer" — antigen's own founding
//! class, caught in antigen's own tests).
//!
//! The root fix (ratified) strengthens the affinity-pair to **spares-namesake**:
//! a named common-method arm must spare the same-method-name clean sibling. Until
//! the `provenance-earnedness-verifier` enforces that structurally, **these guards
//! are the contract** — each asserts a fixed member's named fingerprint does NOT
//! fire on its namesake-clean sibling (and that the honest core arm still does).
//!
//! ## Tier-faithful shapes (not one template)
//!
//! - **DROP** (`from_slice`, `zeroed`): the arm fires on the recommended-safe
//!   namesake (the *fix* / safe API) → inadmissible at every tier → dropped. Guard
//!   = the member SPARES the namesake; the honest core arm still BINDS.
//! - **DEMOTE-to-suspected** (`set_len`, `SizeOf`): the arm fires on *unrelated*
//!   correct siblings (un-correlated, not anti-correlated) → demote, label the
//!   recall holes. Guard = the named member SPARES the demoted needle; the labeled
//!   FP is documented as a known suspected-tier hole, not a named promise.
//! - **doc-disclose** (`duration_since`, already suspected): within-tier recall
//!   noise; guard = the labeled hole is pinned (fires, documented) + the member
//!   stays suspected.
//!
//! Each guard reads the member's **canonical post-fix fingerprint** as a string
//! literal and matches it against the namesake item. If a future edit re-adds a
//! dropped arm to a member, the corresponding `SPARES` guard goes red — loud, not
//! silent. (A structural member-fp == this-string assertion is the
//! `provenance-earnedness-verifier`'s job; see that charter.)

use antigen_fingerprint::Fingerprint;

/// Parse a member's canonical fingerprint string (must parse).
fn fp(src: &str) -> Fingerprint {
    Fingerprint::parse(src).expect("canonical member fingerprint must parse")
}

/// Parse one Rust item (the namesake sibling under test).
fn item(src: &str) -> syn::Item {
    syn::parse_str(src).expect("namesake item must parse")
}

// ============================================================================
// DROP — from_slice (UnboundedDeserialization, named)
//
// Canonical post-fix fingerprint (deserialization.rs): body_calls("from_reader").
// ============================================================================

/// The member's canonical post-fix fingerprint — KEEP IN SYNC with
/// `antigen/src/stdlib/deserialization.rs` :: `UnboundedDeserialization`.
const UNBOUNDED_DESERIALIZATION: &str = r#"body_calls("from_reader")"#;

#[test]
fn deser_binds_from_reader_honest_core() {
    // The honest named core SURVIVES the from_slice drop.
    assert!(
        fp(UNBOUNDED_DESERIALIZATION).matches(&item(
            "fn load(r: impl std::io::Read) -> Config { serde_json::from_reader(r).unwrap() }"
        )),
        "from_reader (the rare/std-specific streaming anchor) must still BIND"
    );
}

#[test]
fn deser_spares_from_slice_namesakes() {
    // SPARES-NAMESAKE: from_slice was DROPPED — a slice is a *bounded* source, so
    // the call fires on the bounded-slice FIX and on ubiquitous safe constructors.
    let fp = fp(UNBOUNDED_DESERIALIZATION);
    for (src, why) in [
        (
            "fn load(b: &[u8]) -> Config { serde_json::from_slice(b).unwrap() }",
            "serde_json::from_slice (deser-shaped, but bounded — not an unbounded vector)",
        ),
        (
            "fn meta(out: &Output) -> Value { serde_json::from_slice(&out.stdout).unwrap() }",
            "antigen's OWN from_slice(&stdout) — the masterclass false-positive (scan/multi_crate.rs)",
        ),
        (
            "fn key(b: &[u8]) -> GenericArray<u8, U32> { GenericArray::from_slice(b).clone() }",
            "GenericArray::from_slice — ubiquitous safe crypto ctor",
        ),
        (
            "fn pk(raw: &[u8]) -> Pubkey { Pubkey::from_slice(raw).expect(\"32\") }",
            "Pubkey::from_slice — k256/solana fixed-size key parse",
        ),
    ] {
        assert!(
            !fp.matches(&item(src)),
            "must SPARE the from_slice namesake: {why}"
        );
    }
}

// ============================================================================
// DROP — zeroed (UninitMemoryAssumedInit, named)
// DEMOTE — set_len (out of this named member, into a suspected home)
//
// Canonical post-fix fingerprint (unsafe_soundness.rs):
//   any_of([body_calls("assume_init"), body_calls("uninitialized")]).
// ============================================================================

/// KEEP IN SYNC with `antigen/src/stdlib/unsafe_soundness.rs` ::
/// `UninitMemoryAssumedInit`.
const UNINIT_ASSUMED_INIT: &str =
    r#"any_of([body_calls("assume_init"), body_calls("uninitialized")])"#;

#[test]
fn uninit_binds_rare_primitives_honest_core() {
    let fp = fp(UNINIT_ASSUMED_INIT);
    assert!(
        fp.matches(&item(
            "fn m() -> u8 { let u = MaybeUninit::uninit(); unsafe { u.assume_init() } }"
        )),
        "assume_init (rare/std-specific, no safe namesake) must still BIND"
    );
    assert!(
        fp.matches(&item(
            "fn m() -> u8 { unsafe { std::mem::uninitialized() } }"
        )),
        "uninitialized must still BIND"
    );
}

#[test]
fn uninit_spares_zeroed_namesake_dropped() {
    // DROP: `zeroed` fires on bytemuck::zeroed() — the SAFE Zeroable-gated
    // replacement for mem::zeroed (the recommended remediation). Inadmissible at
    // any tier → dropped from the named member.
    let fp = fp(UNINIT_ASSUMED_INIT);
    assert!(
        !fp.matches(&item("fn m() -> Header { bytemuck::zeroed() }")),
        "must SPARE bytemuck::zeroed (the SAFE recommended zero-init) — zeroed DROPPED"
    );
    assert!(
        !fp.matches(&item("fn r(b: &mut Buffer) { b.zeroed(); }")),
        "must SPARE a benign .zeroed() method"
    );
}

#[test]
fn uninit_spares_set_len_demoted_out_of_named() {
    // DEMOTE: `set_len` fires on any domain buffer/builder's .set_len(n); the only
    // discriminator is the receiver TYPE (Vec vs domain), not scan-resolvable. So
    // it is carved OUT of this named member into a permanent-suspected home. The
    // named member must spare BOTH the benign setter AND (deliberately) Vec::set_len.
    let fp = fp(UNINIT_ASSUMED_INIT);
    assert!(
        !fp.matches(&item("fn b(h: &mut Header, n: usize) { h.set_len(n); }")),
        "must SPARE a benign domain header.set_len() from the named member"
    );
    assert!(
        !fp.matches(&item(
            "fn g(v: &mut Vec<u8>, n: usize) { unsafe { v.set_len(n); } }"
        )),
        "Vec::set_len is deliberately NOT caught by THIS named member (moved to the suspected home)"
    );
}

// ============================================================================
// DEMOTE — SizeOf (SizeOfInElementCount, named → suspected)
//
// Canonical fingerprint (numeric_truncation.rs, UNCHANGED string — the change is
// the TIER, named → suspected): all_of([copy_nonoverlapping, size_of]).
// At suspected, the byte-buffer / separate-size_of FPs are LABELED recall holes,
// not named over-claims. The defect still fires (the member is useful at suspected).
// ============================================================================

/// KEEP IN SYNC with `antigen/src/stdlib/numeric_truncation.rs` ::
/// `SizeOfInElementCount` (fingerprint string unchanged; the fix is the TIER).
const SIZE_OF_IN_COUNT: &str =
    r#"all_of([body_calls("copy_nonoverlapping"), body_calls("size_of")])"#;

#[test]
fn sizeof_binds_the_defect_still_useful_at_suspected() {
    // The element-count foot-cannon (n * size_of on a *mut T) still FIRES — the
    // member remains useful at suspected (it correlates with the defect).
    assert!(
        fp(SIZE_OF_IN_COUNT).matches(&item(
            "unsafe fn bad<T: Copy>(s: *const T, d: *mut T, n: usize) { std::ptr::copy_nonoverlapping(s, d, n * std::mem::size_of::<T>()); }"
        )),
        "the element-count defect must still fire (suspected = correlates)"
    );
}

#[test]
fn sizeof_labeled_recall_holes_at_suspected() {
    // LABELED holes (the reason it is SUSPECTED not named): the co-presence cannot
    // see arg-position or pointee type, so it ALSO fires on un-correlated correct
    // siblings. At suspected these are documented recall holes, NOT named breaks.
    // Pinning them here keeps the demotion honest: if the member is ever promoted
    // to named while these still fire, that promotion is an over-claim.
    let fp = fp(SIZE_OF_IN_COUNT);
    assert!(
        fp.matches(&item(
            "unsafe fn to_bytes<T: Copy>(s: *const T, d: *mut u8, n: usize) { std::ptr::copy_nonoverlapping(s as *const u8, d, n * std::mem::size_of::<T>()); }"
        )),
        "LABELED HOLE: the byte-buffer form (correct on *mut u8) fires — a known suspected-tier recall hole, NOT a named promise"
    );
    assert!(
        fp.matches(&item(
            "unsafe fn grow<T>(o: *const T, nw: *mut T, len: usize) { let _b = len.checked_mul(std::mem::size_of::<T>()).unwrap(); std::ptr::copy_nonoverlapping(o, nw, len); }"
        )),
        "LABELED HOLE: the Vec-grow idiom (size_of for alloc, copy by len) fires — known suspected-tier recall hole"
    );
    // The TRUE fix-of-the-defect (drop the spurious multiplier) IS spared:
    assert!(
        !fp.matches(&item(
            "unsafe fn good<T: Copy>(s: *const T, d: *mut T, n: usize) { std::ptr::copy_nonoverlapping(s, d, n); }"
        )),
        "the true fix copy(n) (no size_of) is SPARED — the member spares its own fix (un-correlated, so DEMOTE not DROP)"
    );
}

// ============================================================================
// DOC-DISCLOSE — duration_since (SystemTimeUnwrapPanic, already suspected)
//
// Canonical fingerprint (time_ordering.rs, UNCHANGED): the fix is the DOC
// disclosing the Instant::duration_since collision. The labeled hole is pinned here.
// ============================================================================

/// KEEP IN SYNC with `antigen/src/stdlib/time_ordering.rs` ::
/// `SystemTimeUnwrapPanic` (fingerprint + suspected tier unchanged; fix is doc).
const SYSTEM_TIME_UNWRAP: &str = r#"all_of([body_calls("duration_since"), any_of([body_calls("unwrap"), body_calls("expect")])])"#;

#[test]
fn systime_binds_the_defect_core() {
    assert!(
        fp(SYSTEM_TIME_UNWRAP).matches(&item(
            "fn age(t: SystemTime) -> Duration { SystemTime::now().duration_since(t).unwrap() }"
        )),
        "the SystemTime::duration_since().unwrap() defect must fire (core)"
    );
}

#[test]
fn systime_instant_duration_since_is_labeled_recall_hole() {
    // SUSPECTED-tier honesty: duration_since exists on BOTH SystemTime (fallible,
    // the defect) AND Instant (infallible, the fix). The co-occurrence fires on
    // Instant::duration_since + an unrelated unwrap — a WITHIN-TIER recall hole at
    // suspected (NOT a named break, unlike the dropped arms). This pins it as
    // KNOWN/labeled and guards against silent promotion to named while the Instant
    // collision is unresolved (that promotion would make this FP an over-claim).
    assert!(
        fp(SYSTEM_TIME_UNWRAP).matches(&item(
            "fn measure(start: Instant, c: &C) -> Duration { let d = Instant::now().duration_since(start); let _ = c.v.unwrap(); d }"
        )),
        "LABELED HOLE: Instant::duration_since + unrelated unwrap fires — known suspected-tier recall hole, doc-disclosed"
    );
}
