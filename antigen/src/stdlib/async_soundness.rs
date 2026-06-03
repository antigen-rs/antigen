//! # Async-Soundness Family — stdlib antigens (beta.2 voyage)
//!
//! Concurrency-boundary footguns. The build-now member is the **unsafe Send/Sync**
//! form: a hand-written `unsafe impl Send for T` / `unsafe impl Sync for T`
//! asserts cross-thread safety the compiler cannot check — ~40% of unsound
//! RUSTSEC advisories root here (raw pointers, `*mut`, interior non-`Sync`).
//!
//! Biology cognate: the **innate barrier of the concurrency boundary** — an
//! `unsafe impl Send/Sync` is a mislabeled self/non-self marker (declaring "safe
//! to cross the thread boundary" without the receptor that proves it).
//!
//! ## Antigen-category (ADR-028)
//!
//! `FunctionalCorrectness`: an *unsound* `unsafe impl Send/Sync` produces a wrong
//! *effect* (a data race / UB across threads) — the soundness the auto-trait
//! rules exist to guarantee.
//!
//! ## Scope (honest defect-slice)
//!
//! `LockHeldAcrossAwait` (liveness of a typed binding across a suspension point)
//! is a new control-flow grammar dimension → charter. `BlockingCallInAsyncFn`
//! (`is_async` + a heuristic blocking-API name-list) is build-now at the
//! suspected tier — a candidate for the next wave. `SpawnedFutureNotAwaited`
//! (`let _ = spawn()` binding-tell) → charter. This family ships the clean,
//! named `unsafe impl Send/Sync` member now.

use crate::antigen;

// ============================================================================
// 1. UnsafeSendSync
// ============================================================================

/// A hand-written `unsafe impl Send for T` / `unsafe impl Sync for T` — an
/// author-asserted cross-thread-safety the compiler cannot verify.
///
/// **Where in the wild:** RUSTSEC has a steady stream of soundness advisories
/// rooted in a wrong `unsafe impl Send/Sync` (raw pointers, `*mut`, interior
/// non-`Sync`). "Some mutex crates implement `Send` for their `MutexGuard`s …
/// compiles, deadlocks" is this exact class biting.
///
/// **Tell:** an `unsafe impl` of the `Send` or `Sync` trait —
/// `all_of([item = impl, is_unsafe, any_of([impl_of_trait("Send"),
/// impl_of_trait("Sync")])])`. A pure impl-presence + `unsafe`-qualifier tell
/// (the shipped `is_unsafe` G1 leaf reads `unsafe` on the impl; `impl_of_trait`
/// G3 reads the trait). Syntactic.
///
/// **Tier:** **named/confident** — a hand-written `unsafe impl Send/Sync` is an
/// explicit soundness assertion; RUSTSEC-backed (~40% of unsound advisories).
///
/// **Witness:** a documented safety argument (a `// SAFETY:` comment the sensor
/// layer reads), OR a kani proof of the `Send`/`Sync` invariant.
///
/// **Category:** `FunctionalCorrectness` — an unsound `unsafe impl Send/Sync`
/// produces a wrong *effect* (a data race / UB across threads).
#[antigen(
    name = "unsafe-send-sync",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"all_of([item = impl, is_unsafe, any_of([impl_of_trait("Send"), impl_of_trait("Sync")])])"#,
    family = "async-soundness",
    summary = "A hand-written unsafe impl Send/Sync asserts cross-thread safety the compiler cannot check — ~40% of unsound RUSTSEC advisories root here. Named tier; witness = a documented SAFETY argument / kani proof.",
    references = [
        "https://doc.rust-lang.org/nomicon/send-and-sync.html",
        "ADR-040",
    ]
)]
pub struct UnsafeSendSync;
