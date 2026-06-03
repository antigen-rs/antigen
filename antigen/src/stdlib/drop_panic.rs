//! # Drop-and-Panic-Discipline Family — stdlib antigens (beta.2 voyage)
//!
//! Teardown footguns. The canonical one: a `Drop::drop` body that can panic. A
//! panic during `Drop` *while another panic is unwinding* aborts the process
//! (`panic=unwind`), and the destructor's own cleanup is skipped → leaked
//! resources even on the unwinding path. std got this wrong repeatedly.
//!
//! Biology cognate: **apoptosis gone wrong** — programmed cell death (`Drop`)
//! that itself triggers a catastrophic cascade (double-panic → abort) instead
//! of clean teardown.
//!
//! ## Antigen-category (ADR-028)
//!
//! `FunctionalCorrectness`: a panic in `Drop` produces a wrong *effect* (a
//! process abort / leaked resources during unwinding), not a wrong
//! representation.
//!
//! ## Relationship to the shipped `PanickingInDrop` example
//!
//! The `basic` example ships a `PanickingInDrop` antigen whose fingerprint is
//! `item = impl, any_of([body_contains_macro(...)])` — it over-fires on
//! non-`Drop` impls (its own comment flags "no operator for *this impl is for
//! the Drop trait* — that's a v2 enhancement") and misses `.unwrap()`-shaped
//! panics (macro-only). This member is that v2: `impl_of_trait("Drop")` for the
//! real-`Drop` precision + `body_calls` for the call-shaped panic coverage. The
//! example's shipped fingerprint is tightened in lock-step (the v2 tightening,
//! CHANGELOG'd).

use crate::antigen;

// ============================================================================
// 1. PanicInDrop
// ============================================================================

/// A `Drop::drop` body that contains a reachable panic source — a process-abort
/// risk during unwinding.
///
/// **Where in the wild:** the canonical teardown footgun. A panic in `Drop`
/// during an in-flight unwind aborts the process; cleanup in the destructor is
/// then skipped → leaked resources even on `panic=unwind`.
///
/// **Tell:** an `impl Drop for T` (the real `Drop` trait, via `impl_of_trait("Drop")`
/// — NOT merely an inherent impl with a method named `drop`) whose body reaches a
/// panic source: a call-shaped `.unwrap()` / `.expect()` (`body_calls`) OR a
/// macro-shaped `panic!` / `unreachable!` / `todo!` / `unimplemented!`
/// (`body_contains_macro`). Covering both shapes is the point: a macro-only tell
/// (the shipped `PanickingInDrop`) silently misses the `.unwrap()` form, which is
/// the more common teardown panic.
///
/// **Tier:** **named** — the `impl_of_trait("Drop")` + reachable-panic-source
/// tell is precise, and the double-panic-on-unwind class is documented std
/// behaviour.
///
/// **Witness:** the drop body is panic-free, OR the risky op is wrapped to catch/
/// log, OR `std::thread::panicking()` is checked before the risky op (so the
/// destructor stays quiet during an in-flight unwind).
///
/// **Category:** `FunctionalCorrectness` — a panic in `Drop` produces a wrong
/// *effect* (a process abort / skipped cleanup during unwinding).
#[antigen(
    name = "panic-in-drop",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"all_of([item = impl, impl_of_trait("Drop"), any_of([body_calls("unwrap"), body_calls("expect"), body_contains_macro("panic"), body_contains_macro("unreachable"), body_contains_macro("todo"), body_contains_macro("unimplemented")])])"#,
    family = "drop-and-panic-discipline",
    summary = "A real Drop impl (impl_of_trait Drop) whose body reaches a panic source — .unwrap()/.expect() OR panic!/unreachable!/todo!/unimplemented!. Panic-during-unwind aborts the process. Covers BOTH call-shaped and macro-shaped panics (the shipped macro-only PanickingInDrop misses .unwrap()).",
    references = [
        "https://doc.rust-lang.org/std/ops/trait.Drop.html#panics",
        "ADR-040",
    ]
)]
pub struct PanicInDrop;
