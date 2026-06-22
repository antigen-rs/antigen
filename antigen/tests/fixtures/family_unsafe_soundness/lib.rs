// Admitting-specimen for the unsafe-soundness family (beta.2).
//
// SCAN FIXTURE (not a compiled example) BY NECESSITY: every member's tell is an
// `unsafe` primitive (transmute / assume_init / from_utf8_unchecked), and the
// workspace sets `unsafe_code = "forbid"` (an un-overridable forbid), so these
// cannot live in a compiled crate. The scanner reads source as TEXT, so the
// affinity-pairs live here where the real unsafe primitives can be exhibited.
//
// Each member: a BAD path the fingerprint binds + a GOOD sibling it spares.

// --- Member 1: TransmuteSizeOrLifetimeMismatch ---
#[antigen(
    name = "transmute-size-or-lifetime-mismatch",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"any_of([body_calls("transmute"), body_calls("transmute_copy")])"#,
    family = "unsafe-soundness",
    summary = "A mem::transmute / transmute_copy call — a size/lifetime/mutability mismatch is UB.",
    references = ["https://doc.rust-lang.org/std/mem/fn.transmute.html"]
)]
struct TransmuteSizeOrLifetimeMismatch;

// BAD (the bind): transmutes `&T` to `&mut T` — UB (rustc mutable_transmutes).
#[presents(TransmuteSizeOrLifetimeMismatch)]
fn alias_mut(r: &u8) -> &mut u8 {
    unsafe { std::mem::transmute(r) }
}

// GOOD (the spare): a checked `as` cast — no transmute.
#[presents(TransmuteSizeOrLifetimeMismatch)]
fn widen(x: u8) -> u32 {
    x as u32
}

// --- Member 2: UninitMemoryAssumedInit ---
#[antigen(
    name = "uninit-memory-assumed-init",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"any_of([body_calls("assume_init"), body_calls("uninitialized")])"#,
    family = "unsafe-soundness",
    summary = "Reading uninitialized memory as initialized — MaybeUninit::assume_init / mem::uninitialized. UB. (zeroed dropped — fires on safe bytemuck::zeroed; set_len permanent-suspected — ADR-039 §C Amd-1.)",
    references = ["https://doc.rust-lang.org/std/mem/union.MaybeUninit.html#method.assume_init"]
)]
struct UninitMemoryAssumedInit;

// BAD (the bind): reads MaybeUninit memory before initializing it — UB.
#[presents(UninitMemoryAssumedInit)]
fn read_uninit() -> u32 {
    let m: std::mem::MaybeUninit<u32> = std::mem::MaybeUninit::uninit();
    unsafe { m.assume_init() }
}

// GOOD (the spare): a fully-initialized value — no uninit primitive.
#[presents(UninitMemoryAssumedInit)]
fn read_init() -> u32 {
    0
}

// --- Member 3: UnvalidatedFromUtf8Unchecked ---
#[antigen(
    name = "unvalidated-from-utf8-unchecked",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"any_of([body_calls("from_utf8_unchecked"), body_calls("from_utf8_unchecked_mut")])"#,
    family = "unsafe-soundness",
    summary = "str::from_utf8_unchecked on non-validated bytes — a UB str (rustc invalid_from_utf8_unchecked).",
    references = ["https://doc.rust-lang.org/std/str/fn.from_utf8_unchecked.html"]
)]
struct UnvalidatedFromUtf8Unchecked;

// BAD (the bind): builds a `str` from unvalidated bytes — UB if not valid UTF-8.
#[presents(UnvalidatedFromUtf8Unchecked)]
fn as_str_unchecked(b: &[u8]) -> &str {
    unsafe { std::str::from_utf8_unchecked(b) }
}

// GOOD (the spare): the CHECKED from_utf8 (validates, returns Result).
#[presents(UnvalidatedFromUtf8Unchecked)]
fn as_str_checked(b: &[u8]) -> &str {
    std::str::from_utf8(b).unwrap_or("")
}
