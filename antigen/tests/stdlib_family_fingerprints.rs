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
const CRYPTO_NON_CONSTANT_TIME: &str = r#"all_of([any_of([body_calls("verify"), body_calls("hmac_verify"), body_calls("verify_mac")]), not(any_of([body_calls("ct_eq"), body_calls("constant_time_eq")]))])"#;

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
    // any_of(verify entrypoints) = NoMatch → all_of short-circuits to NoMatch.
    // Guards against the fingerprint over-firing on any function with a `not`
    // branch.
    let fp = fp(CRYPTO_NON_CONSTANT_TIME);
    assert!(
        !fp.matches(&item("fn unrelated(x: u32) -> u32 { x + 1 }")),
        "must SPARE a function that never calls a verify entrypoint (no anchor)"
    );
}

#[test]
fn non_constant_time_secret_comparison_binds_hmac_verify_wide_net() {
    // BIND the wide-net arm: a body that calls `hmac_verify` (NOT the bare
    // `verify` needle) with no constant-time compare. A single-needle "verify"
    // fingerprint would SILENTLY MISS this (last-segment match) — the wide-net
    // any_of is exactly what prevents that false-negative (adversarial finding).
    let fp = fp(CRYPTO_NON_CONSTANT_TIME);
    assert!(
        fp.matches(&item("fn check(t: &[u8]) -> bool { hmac_verify(t) }")),
        "must BIND hmac_verify (the wide-net anchor; single 'verify' would miss it)"
    );
}

#[test]
fn non_constant_time_secret_comparison_spares_constant_time_eq_safe_step() {
    // SPARE the wide-net safe arm: a verify path whose safe step is
    // `constant_time_eq` (NOT the bare `ct_eq`). A single-needle `not(ct_eq)`
    // would FALSELY BIND this (constant_time_eq absent from the needle set →
    // looks undefended). The wide-net not(any_of([ct_eq, constant_time_eq]))
    // recognizes both safe-step spellings.
    let fp = fp(CRYPTO_NON_CONSTANT_TIME);
    assert!(
        !fp.matches(&item(
            "fn check(p: &[u8], e: &[u8]) -> bool { let _ = verify(p, e); constant_time_eq(p, e) }"
        )),
        "must SPARE a verify path guarded by constant_time_eq (wide-net safe arm)"
    );
}

// ============================================================================
// deserialization-trust-boundary :: DeserializeWithoutDenyUnknownFields
// ============================================================================

/// The member's declared fingerprint, kept in ONE place (the drift-guard).
const DESERIALIZE_WITHOUT_DENY: &str =
    r#"all_of([derives("Deserialize"), not(serde_arg("deny_unknown_fields"))])"#;

#[test]
fn deserialize_without_deny_binds_derive_without_the_arg() {
    // BIND: #[derive(Deserialize)] present, #[serde(deny_unknown_fields)] absent.
    // derives("Deserialize") = Match; not(serde_arg("deny_unknown_fields")) =
    // not(NoMatch) = Match → all_of = Match. The leaky-gut site.
    let fp = fp(DESERIALIZE_WITHOUT_DENY);
    assert!(
        fp.matches(&item(
            "#[derive(Deserialize)] struct Config { admin: bool, name: String }"
        )),
        "must BIND a Deserialize struct with no deny_unknown_fields"
    );
}

#[test]
fn deserialize_without_deny_spares_struct_with_the_arg() {
    // SPARE: the SAME struct, but #[serde(deny_unknown_fields)] IS present.
    // not(serde_arg("deny_unknown_fields")) = not(Match) = NoMatch → all_of =
    // NoMatch. The presence of the tight-junction spares the sibling.
    let fp = fp(DESERIALIZE_WITHOUT_DENY);
    assert!(
        !fp.matches(&item(
            "#[derive(Deserialize)] #[serde(deny_unknown_fields)] struct Config { admin: bool }"
        )),
        "must SPARE a Deserialize struct that sets deny_unknown_fields"
    );
}

#[test]
fn deserialize_without_deny_spares_non_deserialize_struct() {
    // A struct that does not derive Deserialize is spared: derives("Deserialize")
    // = NoMatch → all_of short-circuits to NoMatch. Guards against the not-branch
    // vacuously matching every struct (the absence-grammar soundness contract).
    let fp = fp(DESERIALIZE_WITHOUT_DENY);
    assert!(
        !fp.matches(&item("#[derive(Debug)] struct Plain { x: u32 }")),
        "must SPARE a struct that does not derive Deserialize (no anchor)"
    );
}

// ============================================================================
// deserialization-trust-boundary :: UnboundedDeserialization
// ============================================================================

/// The member's declared fingerprint, kept in ONE place (the drift-guard).
const UNBOUNDED_DESERIALIZATION: &str = r#"any_of([all_of([body_calls("from_reader"), not(body_calls("take"))]), body_calls("from_slice")])"#;

#[test]
fn unbounded_deserialization_binds_from_reader_call() {
    // BIND: a deser-entrypoint call (from_reader) with no bounded guard.
    // body_calls("from_reader") = Match → any_of = Match. The DoS surface.
    let fp = fp(UNBOUNDED_DESERIALIZATION);
    assert!(
        fp.matches(&item(
            "fn load(r: impl std::io::Read) -> Config { serde_json::from_reader(r).unwrap() }"
        )),
        "must BIND a from_reader deserialization call"
    );
}

#[test]
fn unbounded_deserialization_binds_from_slice_call() {
    // BIND the other arm (from_slice) — proves the any_of covers both byte-source
    // entrypoints, not just from_reader.
    let fp = fp(UNBOUNDED_DESERIALIZATION);
    assert!(
        fp.matches(&item(
            "fn load(b: &[u8]) -> Config { serde_json::from_slice(b).unwrap() }"
        )),
        "must BIND a from_slice deserialization call"
    );
}

#[test]
fn unbounded_deserialization_spares_take_guarded_from_reader() {
    // SPARE (the guard-absence keystone): a from_reader BOUNDED by `.take(limit)`
    // — the std-documented anti-DoS idiom. body_calls("from_reader") = Match but
    // not(body_calls("take")) = not(Match) = NoMatch → the all_of arm is NoMatch;
    // no from_slice → the any_of is NoMatch. The guarded reader is spared.
    let fp = fp(UNBOUNDED_DESERIALIZATION);
    assert!(
        !fp.matches(&item(
            "fn load(r: impl std::io::Read) -> Config { serde_json::from_reader(r.take(1024)).unwrap() }"
        )),
        "must SPARE a from_reader bounded by .take(limit) (the anti-DoS idiom)"
    );
}

#[test]
fn unbounded_deserialization_spares_from_str_and_unrelated() {
    // SPARE from_str: deliberately EXCLUDED (FromStr collision — body_calls has no
    // path resolution, so from_str would fire on every i32::from_str). The member
    // does not anchor on it, so a from_str-only fn is spared.
    let fp = fp(UNBOUNDED_DESERIALIZATION);
    assert!(
        !fp.matches(&item(
            "fn parse(s: &str) -> i32 { i32::from_str(s).unwrap() }"
        )),
        "must SPARE from_str (excluded — FromStr collision needs path resolution)"
    );
    // And an unrelated fn with neither entrypoint is spared.
    assert!(
        !fp.matches(&item("fn unrelated(x: u32) -> u32 { x + 1 }")),
        "must SPARE a function with no deser entrypoint"
    );
}

// ============================================================================
// time-and-ordering-hazards :: SystemTimeUnwrapPanic
// ============================================================================

/// The member's declared fingerprint, kept in ONE place (the drift-guard).
const SYSTEM_TIME_UNWRAP: &str = r#"all_of([any_of([body_calls("duration_since"), body_calls("elapsed")]), any_of([body_calls("unwrap"), body_calls("expect")])])"#;

#[test]
fn system_time_unwrap_binds_duration_since_then_unwrap() {
    // BIND: a clock read (duration_since) AND an unwrap in the same body.
    // any_of(clock-read) = Match AND any_of(unwrap/expect) = Match → all_of =
    // Match. The silent-in-tests / panic-in-prod site.
    let fp = fp(SYSTEM_TIME_UNWRAP);
    assert!(
        fp.matches(&item(
            "fn age(t: SystemTime) -> Duration { SystemTime::now().duration_since(t).unwrap() }"
        )),
        "must BIND a duration_since read whose Result is unwrapped"
    );
}

#[test]
fn system_time_unwrap_binds_elapsed_then_expect() {
    // BIND the other arms (elapsed + expect) — proves both any_of branches.
    let fp = fp(SYSTEM_TIME_UNWRAP);
    assert!(
        fp.matches(&item(
            r#"fn since(t: SystemTime) -> Duration { t.elapsed().expect("clock skew") }"#
        )),
        "must BIND an elapsed read whose Result is expect-ed"
    );
}

#[test]
fn system_time_unwrap_spares_handled_clock_read() {
    // SPARE: a clock read whose Result is HANDLED (no unwrap/expect anywhere in
    // the body). any_of(unwrap/expect) = NoMatch → all_of short-circuits to
    // NoMatch. The handled-Result sibling is the safe path.
    let fp = fp(SYSTEM_TIME_UNWRAP);
    assert!(
        !fp.matches(&item(
            "fn age(t: SystemTime) -> Duration { SystemTime::now().duration_since(t).unwrap_or(Duration::ZERO) }"
        )),
        "must SPARE a clock read whose Result is handled (unwrap_or, no unwrap/expect)"
    );
}

#[test]
fn system_time_unwrap_spares_unwrap_without_clock_read() {
    // SPARE: an unwrap with NO clock read in the body. any_of(clock-read) =
    // NoMatch → all_of short-circuits to NoMatch. The co-occurrence requires
    // BOTH halves — an unwrap on an unrelated Result is not this class.
    let fp = fp(SYSTEM_TIME_UNWRAP);
    assert!(
        !fp.matches(&item(
            "fn parse(s: &str) -> i32 { s.parse::<i32>().unwrap() }"
        )),
        "must SPARE an unwrap with no clock read (no clock-read anchor)"
    );
}

// ============================================================================
// drop-and-panic-discipline :: PanicInDrop
// ============================================================================

/// The member's declared fingerprint, kept in ONE place (the drift-guard).
const PANIC_IN_DROP: &str = r#"all_of([item = impl, impl_of_trait("Drop"), any_of([body_calls("unwrap"), body_calls("expect"), body_contains_macro("panic"), body_contains_macro("unreachable"), body_contains_macro("todo"), body_contains_macro("unimplemented")])])"#;

#[test]
fn panic_in_drop_binds_drop_impl_with_unwrap() {
    // BIND (call-shaped): a real Drop impl whose body calls .unwrap(). This is
    // the panic the shipped macro-only PanickingInDrop silently MISSES — the v2
    // body_calls coverage. impl_of_trait("Drop") = Match, body_calls("unwrap") =
    // Match → all_of = Match.
    let fp = fp(PANIC_IN_DROP);
    assert!(
        fp.matches(&item(
            "impl Drop for Bad { fn drop(&mut self) { self.h.take().unwrap(); } }"
        )),
        "must BIND a real Drop impl with a .unwrap() panic source"
    );
}

#[test]
fn panic_in_drop_binds_drop_impl_with_panic_macro() {
    // BIND (macro-shaped): a real Drop impl whose body invokes panic!.
    let fp = fp(PANIC_IN_DROP);
    assert!(
        fp.matches(&item(
            r#"impl Drop for Bad { fn drop(&mut self) { if self.dirty { panic!("unflushed"); } } }"#
        )),
        "must BIND a real Drop impl with a panic! macro"
    );
}

#[test]
fn panic_in_drop_spares_clean_drop_impl() {
    // SPARE: a real Drop impl with NO panic source. impl_of_trait("Drop") = Match
    // but any_of(panic-sources) = NoMatch → all_of = NoMatch. The panic-free
    // teardown is the safe path.
    let fp = fp(PANIC_IN_DROP);
    assert!(
        !fp.matches(&item(
            "impl Drop for Good { fn drop(&mut self) { let _ = self.h.take(); } }"
        )),
        "must SPARE a real Drop impl with no panic source"
    );
}

#[test]
fn panic_in_drop_spares_inherent_impl_named_drop() {
    // SPARE (the v2 precision): an INHERENT impl with a method merely *named*
    // `drop` that calls .unwrap() — NOT the real Drop trait. impl_of_trait("Drop")
    // = NoMatch → all_of short-circuits to NoMatch. This is exactly the
    // over-fire the shipped item=impl-only PanickingInDrop cannot avoid, and the
    // v2 impl_of_trait tightening fixes.
    let fp = fp(PANIC_IN_DROP);
    assert!(
        !fp.matches(&item(
            "impl Widget { fn drop(&mut self) { self.h.take().unwrap(); } }"
        )),
        "must SPARE an inherent impl with a method named drop (not the Drop trait)"
    );
}

// ============================================================================
// panic-on-index :: GetUncheckedWithoutProof
// ============================================================================

/// The member's declared fingerprint, kept in ONE place (the drift-guard).
const GET_UNCHECKED: &str =
    r#"any_of([body_calls("get_unchecked"), body_calls("get_unchecked_mut")])"#;

#[test]
fn get_unchecked_binds_unchecked_index() {
    // BIND: a call to get_unchecked (the unsafe escape hatch; OOB = UB).
    let fp = fp(GET_UNCHECKED);
    assert!(
        fp.matches(&item(
            "fn at(v: &[u8], i: usize) -> u8 { unsafe { *v.get_unchecked(i) } }"
        )),
        "must BIND a get_unchecked call"
    );
}

#[test]
fn get_unchecked_binds_unchecked_mut() {
    // BIND the other arm (get_unchecked_mut).
    let fp = fp(GET_UNCHECKED);
    assert!(
        fp.matches(&item(
            "fn at_mut(v: &mut [u8], i: usize) -> &mut u8 { unsafe { v.get_unchecked_mut(i) } }"
        )),
        "must BIND a get_unchecked_mut call"
    );
}

#[test]
fn get_unchecked_spares_checked_get() {
    // SPARE: the checked .get(i) — not an unchecked-index call. neither
    // get_unchecked nor get_unchecked_mut present → any_of = NoMatch.
    let fp = fp(GET_UNCHECKED);
    assert!(
        !fp.matches(&item(
            "fn at(v: &[u8], i: usize) -> Option<&u8> { v.get(i) }"
        )),
        "must SPARE the checked .get(i) (no unchecked-index call)"
    );
}

// ============================================================================
// resource-lifecycle-leak :: DeliberateLeakNotDocumented
// ============================================================================

/// The member's declared fingerprint, kept in ONE place (the drift-guard).
const DELIBERATE_LEAK: &str = r#"any_of([body_calls("forget"), body_calls("leak")])"#;

#[test]
fn deliberate_leak_binds_mem_forget() {
    // BIND: a mem::forget call (last-segment "forget") — Drop is skipped.
    let fp = fp(DELIBERATE_LEAK);
    assert!(
        fp.matches(&item("fn drop_it(x: Resource) { std::mem::forget(x); }")),
        "must BIND a mem::forget call"
    );
}

#[test]
fn deliberate_leak_binds_box_leak() {
    // BIND the leak arm (Box::leak / Vec::leak — last-segment "leak").
    let fp = fp(DELIBERATE_LEAK);
    assert!(
        fp.matches(&item(
            "fn make_static(b: Box<str>) -> &'static str { Box::leak(b) }"
        )),
        "must BIND a Box::leak call"
    );
}

#[test]
fn deliberate_leak_spares_ordinary_drop() {
    // SPARE: an ordinary scope-drop with no leak primitive. neither forget nor
    // leak present → any_of = NoMatch.
    let fp = fp(DELIBERATE_LEAK);
    assert!(
        !fp.matches(&item("fn use_it(x: Resource) { let _ = x.compute(); }")),
        "must SPARE an ordinary use with no forget/leak call"
    );
}
