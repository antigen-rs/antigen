//! Panic-on-Index family — the admitting-specimen.
//!
//! The affinity-pair exhibit (ADR-039 §C worth-multiplier) for
//! [`antigen::stdlib::panic_on_index::GetUncheckedWithoutProof`]: an unchecked
//! `get_unchecked` (binds) + a checked `.get(i)` sibling (spared).
//!
//! Run:
//!
//! ```sh
//! cargo run --example panic_on_index --package antigen
//! cargo run --bin cargo-antigen -- antigen scan --root antigen/examples
//! ```
//!
//! Note: both siblings are `#[presents]`-marked, so audit lists both — the safe
//! `.get(i)` sibling is spared by the *fingerprint* (it doesn't bind), not hidden
//! from the console. To *read* the bind/spare side by side, see the guard tests
//! `antigen/tests/stdlib_family_fingerprints.rs`
//! (`get_unchecked_binds_unchecked_index` beside `get_unchecked_spares_checked_get`).
//!
//! ## BIOSAFETY NOTE
//!
//! The workspace forbids `unsafe` (`-F unsafe-code`), and a *real* `get_unchecked`
//! is `unsafe`. So this exhibit uses a **safe toy collection** with a method
//! *named* `get_unchecked` — the fingerprint anchors on the call *token*
//! (`body_calls` matches by method/last-segment identifier), so the call-shape is
//! faithfully exhibited without invoking real unchecked indexing. In production
//! the real `slice::get_unchecked` is the named site; the toy stands in for it.

use antigen::{antigen, presents};

/// A call to `get_unchecked` / `get_unchecked_mut` — the unchecked indexing
/// escape hatch whose out-of-bounds case is Undefined Behavior.
#[antigen(
    name = "get-unchecked-without-proof",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"any_of([body_calls("get_unchecked"), body_calls("get_unchecked_mut")])"#,
    family = "panic-on-index",
    summary = "A call to get_unchecked / get_unchecked_mut — the unchecked-indexing escape hatch whose out-of-bounds case is Undefined Behavior.",
    references = ["https://doc.rust-lang.org/std/primitive.slice.html#method.get_unchecked"],
)]
pub struct GetUncheckedWithoutProof;

/// A safe toy collection with a method *named* `get_unchecked` — stands in for
/// the real `slice::get_unchecked` so the call-shape can be exhibited under the
/// workspace's `-F unsafe-code` lint (the real method is `unsafe`).
struct ToyBuf {
    data: Vec<u8>,
}

impl ToyBuf {
    /// Stand-in for the real (unsafe) `slice::get_unchecked` — same call token,
    /// safe body (it just bounds-checks internally for the exhibit).
    fn get_unchecked(&self, i: usize) -> u8 {
        self.data[i % self.data.len().max(1)]
    }
}

/// BAD (the bind): reads through a `get_unchecked` call — in real code this is
/// the unsafe escape hatch where out-of-bounds is UB (not a panic). The call-shape
/// is exactly what the antigen names.
///
/// `body_calls("get_unchecked")` matches → the `any_of` **binds**.
#[presents(GetUncheckedWithoutProof)]
fn first_unchecked(buf: &ToyBuf, i: usize) -> u8 {
    buf.get_unchecked(i)
}

/// GOOD (the spare): the checked `.get(i)` — a bounds-checked read that returns
/// `None` instead of risking UB.
///
/// Neither `get_unchecked` nor `get_unchecked_mut` is called → **spared**.
#[presents(GetUncheckedWithoutProof)]
fn first_checked(v: &[u8], i: usize) -> Option<u8> {
    v.get(i).copied()
}

fn main() {
    println!("antigen panic-on-index example: see source for the affinity-pair.");
    println!(
        "Both siblings are #[presents]-marked, so audit lists both; the checked .get path is spared by the FINGERPRINT (it doesn't bind). To read the bind/spare side by side, see antigen/tests/stdlib_family_fingerprints.rs."
    );

    let buf = ToyBuf {
        data: vec![1u8, 2, 3],
    };
    let _ = first_unchecked(&buf, 1);
    let _ = first_checked(&buf.data, 1);
}
