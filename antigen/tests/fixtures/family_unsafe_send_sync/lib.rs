// Admitting-specimen for `UnsafeSendSync` (async-soundness family, beta.2 voyage).
//
// This is a SCAN FIXTURE, not a compiled example, BY NECESSITY: the workspace
// sets `unsafe_code = "forbid"` (a forbid an inner #[allow] cannot override), so
// a real `unsafe impl Send` cannot live in a compiled crate. The scanner reads
// source as text (it does not compile it), so the affinity-pair lives here where
// the real `unsafe impl Send` can be exhibited faithfully.
//
// The affinity-pair:
//   BIND  — `unsafe impl Send for RawHandle` (the author-asserted cross-thread
//           safety the compiler cannot check).
//   SPARE — `impl Clone for RawHandle` (a safe impl of an ordinary trait).
//
// Fingerprint:
//   all_of([item = impl, is_unsafe, any_of([impl_of_trait("Send"), impl_of_trait("Sync")])])

// The antigen declaration (so this fixture is a self-contained scan target — the
// scanner builds its fingerprint catalog from declarations in the scanned tree).
#[antigen(
    name = "unsafe-send-sync",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"all_of([item = impl, is_unsafe, any_of([impl_of_trait("Send"), impl_of_trait("Sync")])])"#,
    family = "async-soundness",
    summary = "A hand-written unsafe impl Send/Sync asserts cross-thread safety the compiler cannot check.",
    references = ["https://doc.rust-lang.org/nomicon/send-and-sync.html"]
)]
struct UnsafeSendSync;

// A type wrapping a raw pointer — not auto-Send (the *mut makes it !Send).
struct RawHandle {
    ptr: *mut u8,
}

// BAD (the bind): a hand-written `unsafe impl Send`. The author asserts it is
// safe to move `RawHandle` across threads — a soundness claim the compiler does
// NOT verify. If the pointee is not actually thread-safe, this is UB.
#[presents(UnsafeSendSync)]
unsafe impl Send for RawHandle {}

// GOOD (the spare): a safe impl of an ordinary trait. is_unsafe = NoMatch, and it
// is not Send/Sync — the fingerprint spares it on both counts.
impl Clone for RawHandle {
    fn clone(&self) -> Self {
        RawHandle { ptr: self.ptr }
    }
}
