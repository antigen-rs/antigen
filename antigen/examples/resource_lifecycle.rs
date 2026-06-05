//! Resource-Lifecycle-Leak family — the admitting-specimen.
//!
//! The affinity-pair exhibit (ADR-039 §C worth-multiplier) for
//! [`antigen::stdlib::resource_lifecycle::DeliberateLeakNotDocumented`]: a
//! `mem::forget` leak (binds) + an ordinary scope-drop sibling (spared).
//!
//! Run:
//!
//! ```sh
//! cargo run --example resource_lifecycle --package antigen
//! cargo run --bin cargo-antigen -- antigen scan --root antigen/examples
//! ```
//!
//! Note: both siblings are `#[presents]`-marked, so audit lists both — the safe
//! scope-drop sibling is spared by the *fingerprint* (it doesn't bind), not hidden
//! from the console. To *read* the bind/spare side by side, see the guard tests
//! `antigen/tests/stdlib_family_fingerprints.rs`
//! (`deliberate_leak_binds_mem_forget` beside `deliberate_leak_spares_ordinary_drop`).
//!
//! ## BIOSAFETY NOTE
//!
//! The `mem::forget` below leaks a `String` on purpose to exhibit the call-shape.

// The leak/use exhibit fns are trivially const-eligible to clippy's nursery
// `missing_const_for_fn`, but marking a runtime-leak exhibit `const` would be
// misleading — these are demonstration sites, not const utilities.
#![allow(clippy::missing_const_for_fn)]

use antigen::{antigen, presents};

/// A call to an explicit-leak primitive (`mem::forget` / `Box::leak` /
/// `Vec::leak`) — `Drop` is deliberately skipped; the witness is the documented
/// rationale.
#[antigen(
    name = "deliberate-leak-not-documented",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"any_of([body_calls("forget"), body_calls("leak")])"#,
    family = "resource-lifecycle-leak",
    summary = "A call to an explicit-leak primitive (mem::forget / Box::leak / Vec::leak) — Drop is deliberately skipped; the witness is the documented rationale.",
    references = ["https://doc.rust-lang.org/std/mem/fn.forget.html"],
)]
pub struct DeliberateLeakNotDocumented;

/// BAD (the bind): `mem::forget`s a heap value — its `Drop` never runs, so the
/// allocation leaks. No rationale doc accompanies it (the witness this antigen
/// asks for).
///
/// `body_calls("forget")` matches → the `any_of` **binds**.
#[presents(DeliberateLeakNotDocumented)]
fn leak_it(s: String) {
    std::mem::forget(s);
}

/// GOOD (the spare): an ordinary use that lets the value drop at end of scope —
/// no leak primitive called.
///
/// Neither `forget` nor `leak` is called → **spared**.
#[presents(DeliberateLeakNotDocumented)]
fn use_it(s: String) -> usize {
    s.len()
    // `s` drops here, normally.
}

fn main() {
    println!("antigen resource-lifecycle example: see source for the affinity-pair.");
    println!(
        "Both siblings are #[presents]-marked, so audit lists both; the ordinary drop is spared by the FINGERPRINT (it doesn't bind). To read the bind/spare side by side, see antigen/tests/stdlib_family_fingerprints.rs."
    );

    // Note: leak_it intentionally leaks; we call use_it on a fresh String.
    let _ = use_it(String::from("kept"));
    leak_it(String::from("leaked-on-purpose"));
}
