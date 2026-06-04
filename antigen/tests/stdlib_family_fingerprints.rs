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
// crypto-misuse :: NonConstantTimeSecretComparison — CHARTERED (no test)
//
// The crypto-misuse flagship is chartered, NOT shipped (aristotle's beta.2 notary
// ruling): no honest call-only fingerprint exists for it. A verify-entrypoint
// anchor + not(ct_eq) ANTI-ALIGNS with the defect — it fires on the SAFE path
// (`ring::hmac::verify` is constant-time internally; verify/hmac_verify are the
// names of the safe operation), and the real defect (a hand-rolled `==` on a
// secret, GHSA-q7pg-9pr4-mrp2) has no distinctive call — it needs the deferred
// `security_sensitive_name` name-leaf + the `==` operator-leaf. So there is no
// affinity-pair to test here: the member graduates when those leaves land. See
// `stdlib/crypto_misuse.rs` (the charter doc) for the full reasoning.
// ============================================================================

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
/// `from_slice` was DROPPED (ADR-039 §C Amd-1, spares-namesake): a slice is a
/// bounded source, so `from_slice` fires on the bounded-slice FIX and on safe
/// constructors — flagging the fix is inadmissible at any tier.
const UNBOUNDED_DESERIALIZATION: &str = r#"body_calls("from_reader")"#;

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
fn unbounded_deserialization_spares_from_slice_namesake() {
    // SPARES the `from_slice` NAMESAKE (ADR-039 §C Amd-1 regression guard — the
    // arm that was DROPPED). `from_slice` was a breadth-arm that over-claimed at
    // named: a slice is a *bounded* source, so the call is NOT an unbounded vector
    // — and the bare last-segment fires on the bounded-slice FIX itself plus
    // ubiquitous safe constructors. After the drop, the member must SPARE every
    // `from_slice` namesake: the deser-shaped one (`serde_json::from_slice`),
    // antigen's OWN bounded use (`from_slice(&output.stdout)`), and the safe ctors
    // (`GenericArray::from_slice`). If any future edit re-adds the arm, these fire
    // and this guard goes red.
    let fp = fp(UNBOUNDED_DESERIALIZATION);
    assert!(
        !fp.matches(&item(
            "fn load(b: &[u8]) -> Config { serde_json::from_slice(b).unwrap() }"
        )),
        "must SPARE serde_json::from_slice (bounded source — the dropped breadth-arm)"
    );
    assert!(
        !fp.matches(&item(
            "fn meta(out: &Output) -> Value { serde_json::from_slice(&out.stdout).unwrap() }"
        )),
        "must SPARE antigen's own bounded from_slice(&stdout) (the masterclass false-positive)"
    );
    assert!(
        !fp.matches(&item(
            "fn key(b: &[u8]) -> GenericArray<u8, U32> { GenericArray::from_slice(b).clone() }"
        )),
        "must SPARE the ubiquitous safe ctor GenericArray::from_slice"
    );
}

#[test]
fn unbounded_deserialization_fires_on_take_guarded_reader_witness_spares_at_audit() {
    // The take-guarded from_reader must STILL FIRE on the fingerprint (the risky
    // SURFACE — a streaming deser — is genuinely present); the `.take(limit)`
    // defense is proved by the WITNESS at audit, not fingerprint-spared. A
    // `not(body_calls("take"))` guard would instead silently suppress this finding
    // whenever an UNRELATED Iterator::take appeared — a silent false-negative that
    // breaks the named tier's high-confidence promise. So at named, the surface
    // fires and the witness spares (the surface-flag / witness-proof split).
    let fp = fp(UNBOUNDED_DESERIALIZATION);
    assert!(
        fp.matches(&item(
            "fn load(r: impl std::io::Read) -> Config { serde_json::from_reader(r.take(1024)).unwrap() }"
        )),
        "must FIRE on a take-guarded from_reader (surface present); the witness spares at audit, not the fingerprint"
    );
}

#[test]
fn unbounded_deserialization_spares_from_str_and_unrelated() {
    // SPARE from_str: deliberately EXCLUDED (FromStr collision — body_calls has no
    // path resolution, so from_str would fire on every i32::from_str). The member
    // anchors only on from_reader, so a from_str-only fn is spared.
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
const SYSTEM_TIME_UNWRAP: &str = r#"all_of([body_calls("duration_since"), any_of([body_calls("unwrap"), body_calls("expect")])])"#;

#[test]
fn system_time_unwrap_binds_duration_since_then_unwrap() {
    // BIND: a duration_since read AND an unwrap in the same body.
    // body_calls("duration_since") = Match AND any_of(unwrap/expect) = Match →
    // all_of = Match. The silent-in-tests / panic-in-prod site.
    let fp = fp(SYSTEM_TIME_UNWRAP);
    assert!(
        fp.matches(&item(
            "fn age(t: SystemTime) -> Duration { SystemTime::now().duration_since(t).unwrap() }"
        )),
        "must BIND a duration_since read whose Result is unwrapped"
    );
}

#[test]
fn system_time_unwrap_binds_duration_since_then_expect() {
    // BIND the expect arm — proves both any_of(unwrap/expect) branches.
    let fp = fp(SYSTEM_TIME_UNWRAP);
    assert!(
        fp.matches(&item(
            r#"fn age(a: SystemTime, b: SystemTime) -> Duration { a.duration_since(b).expect("skew") }"#
        )),
        "must BIND a duration_since read whose Result is expect-ed"
    );
}

#[test]
fn system_time_unwrap_spares_instant_elapsed_clean_sibling() {
    // SPARE (the clean-sibling rule): `Instant::now().elapsed()` is the textbook
    // "use Instant instead of SystemTime" FIX — the member's own clean sibling.
    // `elapsed` is NOT in the anchor (it would fire on this anti-correlated safe
    // case), so even with an unrelated unwrap in the body, the duration_since
    // anchor is absent → all_of = NoMatch. The fix is not flagged.
    let fp = fp(SYSTEM_TIME_UNWRAP);
    assert!(
        !fp.matches(&item(
            "fn timed(m: &Map) -> u8 { let _d = Instant::now().elapsed(); m.get(0).unwrap() }"
        )),
        "must SPARE Instant::now().elapsed() (the clean sibling / recommended fix)"
    );
}

#[test]
fn system_time_unwrap_fires_on_instant_duration_since_disclosed_fp() {
    // The DISCLOSED namesake FP (ADR-039 §C Amd-1, the why-suspected guard):
    // `duration_since` is ALSO the infallible `Instant::duration_since` (returns
    // Duration, no Result). So the co-occurrence fires on a body that calls
    // `instant_a.duration_since(instant_b)` AND unwraps something UNRELATED — a
    // false positive on the Instant path, separable only by the receiver TYPE
    // (SystemTime vs Instant), which scan cannot resolve. This is EXACTLY why the
    // member is suspected, not named: a receiver-type-only discriminator is not an
    // AST-feasible leaf, so this firing is honest within-tier recall noise at
    // suspected (a named tier could not carry it). Pins the disclosure so a future
    // promotion-to-named without resolving the receiver type goes red here.
    let fp = fp(SYSTEM_TIME_UNWRAP);
    assert!(
        fp.matches(&item(
            "fn d(a: Instant, b: Instant, m: &Map) -> u8 { let _x = a.duration_since(b); m.get(0).unwrap() }"
        )),
        "fires on infallible Instant::duration_since + unrelated unwrap (disclosed suspected-tier FP)"
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

// ============================================================================
// async-soundness :: UnsafeSendSync
// ============================================================================

/// The member's declared fingerprint, kept in ONE place (the drift-guard).
const UNSAFE_SEND_SYNC: &str =
    r#"all_of([item = impl, is_unsafe, any_of([impl_of_trait("Send"), impl_of_trait("Sync")])])"#;

#[test]
fn unsafe_send_sync_binds_unsafe_impl_send() {
    // BIND: a hand-written `unsafe impl Send for T`. item=impl Match, is_unsafe
    // Match (the impl carries `unsafe`), impl_of_trait("Send") Match → all_of =
    // Match. The author-asserted cross-thread-safety site.
    let fp = fp(UNSAFE_SEND_SYNC);
    assert!(
        fp.matches(&item("unsafe impl Send for Wrapper {}")),
        "must BIND a hand-written unsafe impl Send"
    );
}

#[test]
fn unsafe_send_sync_binds_unsafe_impl_sync() {
    // BIND the Sync arm.
    let fp = fp(UNSAFE_SEND_SYNC);
    assert!(
        fp.matches(&item("unsafe impl Sync for Wrapper {}")),
        "must BIND a hand-written unsafe impl Sync"
    );
}

#[test]
fn unsafe_send_sync_spares_safe_impl_of_other_trait() {
    // SPARE: a SAFE impl (not `unsafe`) of an ordinary trait. is_unsafe = NoMatch
    // → all_of short-circuits to NoMatch. Only the *unsafe* assertion is the tell.
    let fp = fp(UNSAFE_SEND_SYNC);
    assert!(
        !fp.matches(&item(
            "impl Clone for Wrapper { fn clone(&self) -> Self { Wrapper } }"
        )),
        "must SPARE a safe impl of an ordinary trait (not unsafe)"
    );
}

#[test]
fn unsafe_send_sync_spares_unsafe_impl_of_other_unsafe_trait() {
    // SPARE: an `unsafe impl` of a DIFFERENT unsafe trait (not Send/Sync).
    // is_unsafe = Match but any_of([Send, Sync]) = NoMatch → all_of = NoMatch.
    // The trait identity (not just the unsafe-ness) is load-bearing.
    let fp = fp(UNSAFE_SEND_SYNC);
    assert!(
        !fp.matches(&item("unsafe impl MyUnsafeTrait for Wrapper {}")),
        "must SPARE an unsafe impl of a non-Send/Sync trait"
    );
}

#[test]
fn unsafe_send_sync_spares_safe_impl_send() {
    // SPARE (the is_unsafe discriminator): a SAFE `impl Send for T` (no `unsafe`
    // keyword). is_unsafe = NoMatch → all_of short-circuits to NoMatch. This is
    // what makes is_unsafe load-bearing: only the AUTHOR-ASSERTED (unsafe) form is
    // the soundness tell; an auto-derived/blanket safe Send is not the class. (A
    // bare `impl Send` is unusual in real code but valid syn — the test pins that
    // the `unsafe` qualifier, not merely the Send trait, is required.)
    let fp = fp(UNSAFE_SEND_SYNC);
    assert!(
        !fp.matches(&item("impl Send for Wrapper {}")),
        "must SPARE a safe (non-unsafe) impl Send — the unsafe qualifier is the tell"
    );
}

// ============================================================================
// numeric-truncation-overflow :: SizeOfInElementCount (SUSPECTED tier)
// ============================================================================

/// The member's declared fingerprint, kept in ONE place (the drift-guard). The
/// fingerprint is UNCHANGED by the demote (the fix is the TIER, named → suspected,
/// ADR-039 §C Amd-1): the co-presence correlates with the defect REGION but can't
/// pinpoint it, so it ships soft (suspected), not loud (named).
const SIZE_OF_IN_COUNT: &str =
    r#"all_of([body_calls("copy_nonoverlapping"), body_calls("size_of")])"#;

#[test]
fn size_of_in_count_binds_copy_with_size_of() {
    // BIND: a raw copy co-located with size_of — the byte-count-where-element-
    // count foot-cannon. body_calls("copy_nonoverlapping") = Match AND
    // body_calls("size_of") = Match → all_of = Match. The member is useful at
    // suspected: it correlates with the defect region.
    let fp = fp(SIZE_OF_IN_COUNT);
    assert!(
        fp.matches(&item(
            "fn copy(src: *const u8, dst: *mut u8, n: usize) { unsafe { std::ptr::copy_nonoverlapping(src, dst, n * std::mem::size_of::<u32>()) } }"
        )),
        "must BIND a copy_nonoverlapping co-located with size_of (suspected = correlates)"
    );
}

#[test]
fn size_of_in_count_spares_its_own_fix_so_demote_not_drop() {
    // SPARES the member's own anti-correlated FIX: a copy_nonoverlapping with an
    // element count and NO size_of (drop the spurious multiplier). The all_of
    // co-anchor needs BOTH calls, so the fix (no size_of) → all_of = NoMatch. The
    // fix being spared is WHY this is DEMOTE-to-suspected, not DROP (ADR-039 §C
    // Amd-1): an arm that fires on its own fix would be DROP (cf. from_slice); this
    // one is un-correlated (fires on benign siblings) not anti-correlated.
    let fp = fp(SIZE_OF_IN_COUNT);
    assert!(
        !fp.matches(&item(
            "fn copy(src: *const u8, dst: *mut u8, n: usize) { unsafe { std::ptr::copy_nonoverlapping(src, dst, n) } }"
        )),
        "must SPARE the fix (element count, no size_of) — spared-fix is why DEMOTE not DROP"
    );
    // SPARE: a bare size_of with no raw copy — the co-presence requires BOTH.
    assert!(
        !fp.matches(&item("fn sz() -> usize { std::mem::size_of::<u64>() }")),
        "must SPARE a bare size_of with no raw copy (no copy anchor)"
    );
}

#[test]
fn size_of_in_count_fires_on_correct_both_calls_the_why_suspected_guard() {
    // The WHY-SUSPECTED guard (ADR-039 §C Amd-1): this member is SUSPECTED, not
    // named, because the co-presence FIRES on CORRECT both-calls code the AST can't
    // separate from the defect — the discriminator is the `* size_of` count-arg
    // position AND the pointee type, neither syntactic (type-aware → v0.4). These
    // firings are honest labeled-recall noise at suspected (a named tier could not
    // carry them). When the type-aware leaf lands, these become SPARES and the
    // member promotes.
    let fp = fp(SIZE_OF_IN_COUNT);
    // (a) element count, size_of computed for a SEPARATE bounds check — correct.
    assert!(
        fp.matches(&item(
            "fn c<T>(s:*const T,d:*mut T,count:usize){let _b=count*std::mem::size_of::<T>();unsafe{std::ptr::copy_nonoverlapping(s,d,count)}}"
        )),
        "fires on correct element-count + separate size_of bound (suspected-tier recall noise)"
    );
    // (b) single-element byte copy: count = size_of bytes on *u8 ptrs — correct,
    //     and spared only by the pointee type (*u8 = byte buffer), not the AST.
    assert!(
        fp.matches(&item(
            "fn c(s:*const u8,d:*mut u8){unsafe{std::ptr::copy_nonoverlapping(s,d,std::mem::size_of::<u32>())}}"
        )),
        "fires on the correct single-element byte copy (suspected-tier recall noise; pointee-type-spared)"
    );
}

// ============================================================================
// async-soundness :: UnsafeSendSync — SCAN-FIXTURE specimen
// ============================================================================
//
// UnsafeSendSync's admitting-specimen cannot be a compiled example: the workspace
// sets `unsafe_code = "forbid"` (an inner #[allow] cannot override a forbid), so a
// real `unsafe impl Send` cannot live in a compiled crate. The scanner reads
// source as TEXT (it does not compile it), so the affinity-pair lives as a scan
// fixture. This test scans that fixture end-to-end and asserts the member binds
// the `unsafe impl Send` and spares the safe `impl Clone`.

#[test]
fn unsafe_send_sync_scan_fixture_binds_unsafe_impl_spares_safe_impl() {
    use antigen::scan::scan_workspace;
    use std::path::Path;

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("family_unsafe_send_sync");
    let scan = scan_workspace(&fixture, None).expect("fixture scans");

    // BIND: the `unsafe impl Send for RawHandle` must be a fingerprint-match for
    // UnsafeSendSync.
    let bound = scan
        .presentations
        .iter()
        .any(|p| p.antigen_type == "UnsafeSendSync");
    assert!(
        bound,
        "scan must bind UnsafeSendSync on the unsafe impl Send; got: {:?}",
        scan.presentations
            .iter()
            .map(|p| &p.antigen_type)
            .collect::<Vec<_>>()
    );

    // SPARE: there must be exactly ONE UnsafeSendSync site (the unsafe impl Send),
    // NOT two — the safe `impl Clone` must be spared. (The explicit #[presents] on
    // the unsafe impl is one site; the fingerprint must not ALSO fire on Clone.)
    let unsafe_sites = scan
        .presentations
        .iter()
        .filter(|p| p.antigen_type == "UnsafeSendSync")
        .count();
    assert_eq!(
        unsafe_sites, 1,
        "exactly one UnsafeSendSync site (the unsafe impl Send); the safe impl Clone must be spared"
    );
}

// ============================================================================
// unsafe-soundness :: TransmuteSizeOrLifetimeMismatch
// ============================================================================

/// The member's declared fingerprint, kept in ONE place (the drift-guard).
const TRANSMUTE_MISMATCH: &str =
    r#"any_of([body_calls("transmute"), body_calls("transmute_copy")])"#;

#[test]
fn transmute_mismatch_binds_transmute_call_spares_safe_cast() {
    let fp = fp(TRANSMUTE_MISMATCH);
    // BIND: a transmute call (the rare/std-specific self-anchor).
    assert!(
        fp.matches(&item(
            "fn cast(p: *const u8) -> *mut u8 { unsafe { std::mem::transmute(p) } }"
        )),
        "must BIND a transmute call"
    );
    // SPARE: a checked cast with no transmute — body_calls(transmute/transmute_copy)
    // = NoMatch → any_of = NoMatch.
    assert!(
        !fp.matches(&item("fn cast(x: u32) -> i32 { x as i32 }")),
        "must SPARE a checked `as` cast (no transmute)"
    );
}

// ============================================================================
// unsafe-soundness :: UninitMemoryAssumedInit
// ============================================================================

/// The member's declared fingerprint, kept in ONE place (the drift-guard).
/// `zeroed` + `set_len` were DROPPED (ADR-039 §C Amd-1, spares-namesake): `zeroed`
/// fires on the safe `bytemuck::zeroed` (the recommended replacement); `set_len`
/// fires on any domain buffer's `.set_len` (receiver-type-only discriminator,
/// permanent-suspected). Only the no-safe-namesake arms stay named.
const UNINIT_ASSUMED_INIT: &str =
    r#"any_of([body_calls("assume_init"), body_calls("uninitialized")])"#;

#[test]
fn uninit_assumed_init_binds_assume_init_spares_initialized() {
    let fp = fp(UNINIT_ASSUMED_INIT);
    // BIND: an assume_init call.
    assert!(
        fp.matches(&item(
            "fn make() -> u8 { let m = MaybeUninit::uninit(); unsafe { m.assume_init() } }"
        )),
        "must BIND an assume_init call"
    );
    // BIND the uninitialized arm.
    assert!(
        fp.matches(&item(
            "fn make() -> u8 { unsafe { std::mem::uninitialized() } }"
        )),
        "must BIND a mem::uninitialized call"
    );
    // SPARE: a fully-initialized construction with no uninit primitive.
    assert!(
        !fp.matches(&item("fn make() -> u8 { 0u8 }")),
        "must SPARE a fully-initialized construction"
    );
}

#[test]
fn uninit_assumed_init_spares_zeroed_and_setlen_namesakes() {
    // SPARES the dropped breadth-arm NAMESAKES (ADR-039 §C Amd-1 regression guard).
    // `zeroed` was DROPPED: it fires on the SAFE recommended replacement
    // `bytemuck::zeroed()` / `Zeroable::zeroed()` — flagging the fix is inadmissible
    // at any tier. `set_len` was DROPPED: it fires on any domain buffer's
    // `.set_len(n)`, separable only by receiver TYPE (not scan-resolvable) →
    // permanent-suspected, not in this named member. If a future edit re-adds
    // either arm, these fire and the guard goes red.
    let fp = fp(UNINIT_ASSUMED_INIT);
    assert!(
        !fp.matches(&item("fn z() -> T { bytemuck::zeroed() }")),
        "must SPARE the safe bytemuck::zeroed (the recommended replacement — dropped arm)"
    );
    assert!(
        !fp.matches(&item(
            "fn grow(v: &mut Vec<u8>, n: usize) { unsafe { v.set_len(n) } }"
        )),
        "must SPARE set_len (receiver-type-only discriminator → permanent-suspected, dropped arm)"
    );
    assert!(
        !fp.matches(&item("fn fit(b: &mut MyBuf, n: usize) { b.set_len(n); }")),
        "must SPARE a domain buffer's set_len (the benign namesake the named arm over-fired on)"
    );
}

// ============================================================================
// unsafe-soundness :: UnvalidatedFromUtf8Unchecked
// ============================================================================

/// The member's declared fingerprint, kept in ONE place (the drift-guard).
const FROM_UTF8_UNCHECKED: &str =
    r#"any_of([body_calls("from_utf8_unchecked"), body_calls("from_utf8_unchecked_mut")])"#;

#[test]
fn from_utf8_unchecked_binds_unchecked_spares_checked() {
    let fp = fp(FROM_UTF8_UNCHECKED);
    // BIND: a from_utf8_unchecked call.
    assert!(
        fp.matches(&item(
            "fn s(b: &[u8]) -> &str { unsafe { std::str::from_utf8_unchecked(b) } }"
        )),
        "must BIND a from_utf8_unchecked call"
    );
    // SPARE: the CHECKED from_utf8 (returns Result) — a different last-segment.
    assert!(
        !fp.matches(&item(
            "fn s(b: &[u8]) -> &str { std::str::from_utf8(b).unwrap() }"
        )),
        "must SPARE the checked from_utf8 (different method)"
    );
}

// ============================================================================
// unsafe-soundness — SCAN-FIXTURE specimen (the three members end-to-end)
// ============================================================================
//
// The unsafe-soundness specimens cannot be compiled examples (every tell is an
// `unsafe` primitive, and the workspace forbids unsafe). The scan fixture carries
// the real primitives as text. This test scans it and asserts all three members
// bind their `unsafe` site (each marked `#[presents]`).

#[test]
fn unsafe_soundness_scan_fixture_binds_all_three_members() {
    use antigen::scan::scan_workspace;
    use std::path::Path;

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("family_unsafe_soundness");
    let scan = scan_workspace(&fixture, None).expect("fixture scans");

    for member in [
        "TransmuteSizeOrLifetimeMismatch",
        "UninitMemoryAssumedInit",
        "UnvalidatedFromUtf8Unchecked",
    ] {
        let bound = scan.presentations.iter().any(|p| p.antigen_type == member);
        assert!(
            bound,
            "scan must bind {member} on its unsafe-primitive site; got: {:?}",
            scan.presentations
                .iter()
                .map(|p| &p.antigen_type)
                .collect::<Vec<_>>()
        );
    }
}
