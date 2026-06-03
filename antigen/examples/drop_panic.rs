//! Drop-and-Panic-Discipline family — the admitting-specimen.
//!
//! The affinity-pair exhibit (ADR-039 §C worth-multiplier) for
//! [`antigen::stdlib::drop_panic::PanicInDrop`]: a real `Drop` impl with a
//! `.unwrap()` panic source (binds) + a panic-free `Drop` sibling (spared) + an
//! inherent impl with a method merely *named* `drop` (spared by the
//! `impl_of_trait("Drop")` precision — the v2 the shipped `PanickingInDrop`
//! couldn't express).
//!
//! Run:
//!
//! ```sh
//! cargo run --example drop_panic --package antigen
//! ```
//!
//! Scan to see the pair separate:
//!
//! ```sh
//! cargo run --bin cargo-antigen -- antigen scan --root antigen/examples
//! ```
//!
//! ## BIOSAFETY NOTE
//!
//! The "bad" `Drop` impl really can panic — kept intentional for the exhibit.

use antigen::{antigen, presents};

/// A real `Drop` impl whose body reaches a panic source — a process-abort risk
/// during unwinding.
#[antigen(
    name = "panic-in-drop",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"all_of([item = impl, impl_of_trait("Drop"), any_of([body_calls("unwrap"), body_calls("expect"), body_contains_macro("panic"), body_contains_macro("unreachable"), body_contains_macro("todo"), body_contains_macro("unimplemented")])])"#,
    family = "drop-and-panic-discipline",
    summary = "A real Drop impl whose body reaches a panic source (.unwrap()/.expect() or panic!/...). Panic-during-unwind aborts the process.",
    references = ["https://doc.rust-lang.org/std/ops/trait.Drop.html#panics"],
)]
pub struct PanicInDrop;

/// A resource handle whose teardown flushes — and could panic.
struct PanickyGuard {
    pending: Option<String>,
}

/// BAD (the bind): a real `Drop` impl that `.unwrap()`s in teardown. If the
/// pending value is `None` during an in-flight unwind, this panics → process
/// abort. This is the call-shaped panic the macro-only `PanickingInDrop` misses.
///
/// `impl_of_trait("Drop")` matches AND `body_calls("unwrap")` matches → **binds**.
#[presents(PanicInDrop)]
impl Drop for PanickyGuard {
    fn drop(&mut self) {
        // BAD: unwrap() in Drop — panics if `pending` is None.
        let _flushed = self.pending.take().unwrap();
    }
}

/// A resource handle whose teardown is panic-free.
struct SafeGuard {
    pending: Option<String>,
}

/// GOOD (the spare): a real `Drop` impl with a panic-free teardown.
///
/// `impl_of_trait("Drop")` matches but `any_of([panic-sources])` does NOT →
/// **spared**.
#[presents(PanicInDrop)]
impl Drop for SafeGuard {
    fn drop(&mut self) {
        // GOOD: handle the Option instead of unwrapping.
        if let Some(_pending) = self.pending.take() {
            // flush safely
        }
    }
}

/// A type with an inherent method merely *named* `drop` — NOT the `Drop` trait.
struct NotReallyDrop {
    pending: Option<String>,
}

impl NotReallyDrop {
    /// SPARE (the v2 precision): an inherent `drop` that `.unwrap()`s. The shipped
    /// `item = impl`-only `PanickingInDrop` would over-fire here; `PanicInDrop`'s
    /// `impl_of_trait("Drop")` anchor correctly spares it — this is NOT the `Drop`
    /// trait, so a panic here is an ordinary method panic, not an unwind-abort.
    fn drop(&mut self) {
        let _ = self.pending.take().unwrap();
    }
}

fn main() {
    println!(
        "antigen drop-panic example: see source for the affinity-pair + the inherent-drop spare."
    );
    println!(
        "Run `cargo run --bin cargo-antigen -- antigen scan` to see the bad Drop flagged, the safe Drop + inherent drop spared."
    );

    // Exercise the safe paths; the bad guard is constructed with a value so its
    // unwrap does not panic in this run (its danger is the None-during-unwind edge).
    let g = PanickyGuard {
        pending: Some("data".to_string()),
    };
    drop(g);
    let s = SafeGuard { pending: None };
    drop(s);
    let mut n = NotReallyDrop {
        pending: Some("x".to_string()),
    };
    n.drop();
}
