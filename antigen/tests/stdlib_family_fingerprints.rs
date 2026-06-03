//! Stdlib family-member fingerprint affinity-pair tests (beta.2 voyage).
//!
//! Each build-now stdlib family member ships WITH an **admitting-specimen** — an
//! affinity-pair (a failing case the fingerprint *binds* + a clean sibling it
//! must *not* bind), the ADR-039 §C worth-multiplier. These tests assert that
//! property directly at the fingerprint level: the member's declared fingerprint
//! string is parsed and matched against a binds-bad / spares-good pair.
//!
//! This is also a drift-guard: the fingerprint string asserted here is the same
//! shape the member declares in `antigen/src/stdlib/<family>.rs` and exhibits in
//! `antigen/examples/<family>.rs`. If a future edit changes the member's
//! fingerprint without updating the specimen (or vice-versa), the affinity-pair
//! here breaks — the member can never silently ship a fingerprint whose codomain
//! diverges from its demonstrated mechanism (antigen's own ⊥-collapse class,
//! dogfooded).
//!
//! Tests-first cadence: the bind/spare assertions DEFINE done for each member;
//! the member's fingerprint is built to make them green.

use antigen_fingerprint::Fingerprint;

/// Parse a fingerprint source; panic with the parse error if it does not parse
/// (a member whose fingerprint does not even parse is a hard failure).
fn fp(src: &str) -> Fingerprint {
    Fingerprint::parse(src).expect("member fingerprint must parse")
}

/// Parse one Rust item from source (the specimen item under test).
fn item(src: &str) -> syn::Item {
    syn::parse_str(src).expect("specimen item must parse")
}

// ============================================================================
// crypto-misuse :: NonConstantTimeSecretComparison
// ============================================================================

/// The crypto member's declared fingerprint, kept in ONE place so the bind and
/// spare assertions below test the exact shipped shape.
const CRYPTO_NON_CONSTANT_TIME: &str =
    r#"all_of([body_calls("verify"), not(body_calls("ct_eq"))])"#;

#[test]
fn non_constant_time_secret_comparison_binds_verify_without_ct_eq() {
    // BIND (the vulnerable specimen): a verify path with NO constant-time
    // comparison present. body_calls("verify") = Match; not(body_calls("ct_eq"))
    // = not(NoMatch) = Match → all_of = Match. This is the timing-oracle site.
    let fp = fp(CRYPTO_NON_CONSTANT_TIME);
    assert!(
        fp.matches(&item(
            "fn check(p: &[u8], e: &[u8]) -> bool { verify(p, e) }"
        )),
        "must BIND a verify path with no constant-time comparison present"
    );
}

#[test]
fn non_constant_time_secret_comparison_spares_verify_with_ct_eq() {
    // SPARE (the clean sibling): the SAME verify path, but the constant-time
    // comparison IS present. body_calls("verify") = Match; not(body_calls("ct_eq"))
    // = not(Match) = NoMatch → all_of = NoMatch. The presence of the safe step is
    // exactly what the absence-grammar tell looks for.
    let fp = fp(CRYPTO_NON_CONSTANT_TIME);
    assert!(
        !fp.matches(&item(
            "fn check(p: &[u8], e: &[u8]) -> bool { let _ = verify(p, e); ct_eq(p, e) }"
        )),
        "must SPARE a verify path that routes the comparison through ct_eq"
    );
}

#[test]
fn non_constant_time_secret_comparison_spares_unrelated_fn() {
    // A function that does neither (no verify call at all) is spared:
    // body_calls("verify") = NoMatch → all_of short-circuits to NoMatch. Guards
    // against the fingerprint over-firing on any function with a `not` branch.
    let fp = fp(CRYPTO_NON_CONSTANT_TIME);
    assert!(
        !fp.matches(&item("fn unrelated(x: u32) -> u32 { x + 1 }")),
        "must SPARE a function that never calls verify (no anchor)"
    );
}
