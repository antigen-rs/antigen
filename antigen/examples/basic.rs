//! Basic example demonstrating the antigen macros end-to-end.
//!
//! Run with:
//!
//! ```sh
//! cargo run --example basic --package antigen
//! ```
//!
//! Or, more interestingly, scan this example with cargo-antigen:
//!
//! ```sh
//! cargo run --bin cargo-antigen -- antigen scan
//! ```
//!
//! The scan should find:
//! - 1 antigen declaration (`PanickingInDrop`)
//! - 1 presentation (the `impl Drop` for `VulnerableType`)
//! - 1 immunity claim (the `impl Drop` for `SafeType`)
//!
//! And report 0 unaddressed presentations because each #[presents] has a
//! corresponding #[immune] nearby (or in this minimal example, the
//! presents-without-immune is intentional and would be flagged).

use antigen::{antigen, immune, presents};

/// Drop impls must not panic. Panic during Drop while another panic is
/// unwinding causes process abort.
#[antigen(
    name = "panicking-in-drop",
    family = "boundary-violation",
    fingerprint = "impl Drop with unwrap/expect/panic in body",
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

#[immune(
    PanickingInDrop,
    witness = safe_type_drop_no_panic_test,
    rationale = "SafeType::drop uses non-panicking accessors only; verified by test."
)]
impl Drop for SafeType {
    fn drop(&mut self) {
        // GOOD: no unwrap, no expect, no panic.
        if let Some(_d) = self.data.as_ref() {
            // do something safe
        }
    }
}

/// Witness: proves `SafeType::drop` does not panic on any state.
#[allow(dead_code)]
fn safe_type_drop_no_panic_test() {
    let s = SafeType { data: None };
    drop(s);

    let s = SafeType {
        data: Some(String::from("hello")),
    };
    drop(s);
}

fn main() {
    println!("antigen example: see source for #[antigen], #[presents], #[immune] usage.");
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
