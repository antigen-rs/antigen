//! Basic example demonstrating the antigen macros end-to-end.
//!
//! Run with:
//!
//! ```sh
//! cargo run --example basic --package antigen
//! ```
//!
//! Or, more interestingly, scan the examples directory with cargo-antigen:
//!
//! ```sh
//! cargo run --bin cargo-antigen -- antigen scan --root antigen/examples
//! ```
//!
//! The scan will find declarations from all five example files together.
//! For `basic.rs` specifically:
//! - 1 antigen declaration (`PanickingInDrop`) — declared in this file
//! - 1 explicit presentation (`#[presents(PanickingInDrop)]` on the `impl Drop` for `VulnerableType`)
//! - 1 defended presentation (`#[presents(PanickingInDrop)]` + `#[defended_by(PanickingInDrop)]` on the test)
//! - 1 unaddressed presentation — the deliberate `#[presents]` on `VulnerableType` with
//!   no witness
//!
//! ADR-029: immunity is observed by audit (via `#[defended_by]` on tests), not declared
//! by `#[immune]` on the code site. `#[presents]` marks the vulnerability; `#[defended_by]`
//! on the test declares the test's intent toward that failure-class.
//!
//! Other example files contribute their own declarations to the scan total.
//! See the other files in this directory for `descended_from`, `antigen_tolerance`,
//! and phantom-type witness examples.

use antigen::{antigen, defended_by, presents};

/// Drop impls must not panic. Panic during Drop while another panic is
/// unwinding causes process abort.
// Canonical seed antigen per ADR-010 Amendment 3 Clause C. The fingerprint
// matches real `Drop` impls (via `impl_of_trait("Drop")`, ADR-040) whose bodies
// contain a panic-shaped macro. The `impl_of_trait("Drop")` anchor is the v2
// tightening (beta.2 voyage): the v1 grammar had no operator for "this impl is
// for the Drop trait", so it over-fired on every non-`Drop` impl with a panic
// macro; now it only fires on the real Drop trait, narrowing the codomain to the
// actual failure-class. (For the broader stdlib member that ALSO covers
// call-shaped `.unwrap()`/`.expect()` panics, see
// `antigen::stdlib::drop_panic::PanicInDrop`.)
#[antigen(
    name = "panicking-in-drop",
    family = "boundary-violation",
    fingerprint = r#"
        item = impl,
        impl_of_trait("Drop"),
        any_of([
            body_contains_macro("panic"),
            body_contains_macro("unreachable"),
            body_contains_macro("todo"),
            body_contains_macro("unimplemented")
        ])
    "#,
    summary = "Drop impls must not panic; panic-during-unwind causes process abort.",
    references = [
        "https://doc.rust-lang.org/std/ops/trait.Drop.html#panics",
    ],
)]
pub struct PanickingInDrop;

/// A type that demonstrates the failure-class — its `Drop` impl could panic.
pub struct VulnerableType {
    /// Inner data; could be `None`.
    pub data: Option<String>,
}

#[presents(PanickingInDrop)]
impl Drop for VulnerableType {
    fn drop(&mut self) {
        // BAD: unwrap() in Drop. This is the failure-class the antigen names.
        // In real code this would be flagged. Here we keep it intentional for
        // demonstration; the scan will report this as an unaddressed presentation.
        let _val = self.data.as_ref().unwrap_or(&String::new()).clone();
    }
}

/// A safe alternative whose `Drop` impl is provably panic-free.
pub struct SafeType {
    /// Inner data; could be `None`.
    pub data: Option<String>,
}

/// ADR-029: mark the site with `#[presents]`; the test declares its intent with
/// `#[defended_by]`. Immunity is observed by audit — not declared at the code site.
#[presents(PanickingInDrop)]
impl Drop for SafeType {
    fn drop(&mut self) {
        // GOOD: no unwrap, no expect, no panic.
        if let Some(_d) = self.data.as_ref() {
            // do something safe
        }
    }
}

/// Witness: proves `SafeType::drop` does not panic on any state.
/// `#[defended_by]` declares this test's intent toward `PanickingInDrop`;
/// audit observes whether the circuit is wired (this test covers the site).
#[allow(dead_code)]
#[defended_by(PanickingInDrop)]
fn safe_type_drop_no_panic_test() {
    let s = SafeType { data: None };
    drop(s);

    let s = SafeType {
        data: Some(String::from("hello")),
    };
    drop(s);
}

fn main() {
    println!("antigen example: see source for #[antigen], #[presents], #[defended_by] usage.");
    println!("Run `cargo run --bin cargo-antigen -- antigen scan` to see them detected.");

    // Exercise both types so the example is functional.
    let v = VulnerableType {
        data: Some("data".to_string()),
    };
    drop(v);

    let s = SafeType { data: None };
    drop(s);

    safe_type_drop_no_panic_test();
}
