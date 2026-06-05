//! Numeric-Truncation-Overflow family — the admitting-specimen.
//!
//! The affinity-pair exhibit (ADR-039 §C worth-multiplier) for
//! [`antigen::stdlib::numeric_truncation::SizeOfInElementCount`]: a raw copy
//! co-located with `size_of` (binds) + a copy with an explicit element count
//! (spared).
//!
//! Run:
//!
//! ```sh
//! cargo run --example numeric_truncation --package antigen
//! cargo run --bin cargo-antigen -- antigen scan --root antigen/examples
//! ```
//!
//! Note: both siblings are `#[presents]`-marked, so audit lists both — the safe
//! sibling is spared by the *fingerprint* (it doesn't bind), not hidden from the
//! console. To *read* the bind/spare side by side, see the guard tests
//! `antigen/tests/stdlib_family_fingerprints.rs`
//! (`size_of_in_count_binds_copy_with_size_of` beside
//! `size_of_in_count_spares_its_own_fix_so_demote_not_drop`).
//!
//! ## BIOSAFETY NOTE
//!
//! The real `ptr::copy_nonoverlapping` is `unsafe`, and the workspace forbids
//! `unsafe` (`-F unsafe-code`). So this exhibit uses a **safe toy** function
//! *named* `copy_nonoverlapping` — the fingerprint anchors on the call *token*,
//! so the foot-cannon call-shape is exhibited faithfully without real unsafe. In
//! production the real `ptr::copy_nonoverlapping` is the named site.

use antigen::{antigen, presents};

/// A raw-memory copy co-located with a `size_of` — the byte-count-where-element-
/// count-expected foot-cannon.
#[antigen(
    name = "size-of-in-element-count",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"all_of([body_calls("copy_nonoverlapping"), body_calls("size_of")])"#,
    family = "numeric-truncation-overflow",
    summary = "A raw-memory copy (copy_nonoverlapping) co-located with size_of — the byte-count-where-element-count foot-cannon (n * size_of as a count arg → OOB).",
    references = ["https://rust-lang.github.io/rust-clippy/master/index.html#size_of_in_element_count"],
)]
pub struct SizeOfInElementCount;

/// Safe toy stand-in for `ptr::copy_nonoverlapping` — same call token, safe body
/// (it just records the requested count), so the foot-cannon call-shape can be
/// exhibited under `-F unsafe-code`.
fn copy_nonoverlapping(_src: &[u8], dst: &mut Vec<u8>, count: usize) {
    dst.resize(count.min(64), 0);
}

/// BAD (the bind): the count arg is `n * size_of::<u32>()` — a **byte** count
/// where `copy_nonoverlapping` wants an **element** count, so this over-copies by
/// `sizeof(u32)` → out-of-bounds in real code.
///
/// `body_calls("copy_nonoverlapping")` matches AND `body_calls("size_of")` matches
/// → the `all_of` **binds**.
#[presents(SizeOfInElementCount)]
fn copy_bad(src: &[u8], dst: &mut Vec<u8>, n: usize) {
    copy_nonoverlapping(src, dst, n * std::mem::size_of::<u32>());
}

/// GOOD (the spare): the count arg is a plain element count `n` — no `size_of`
/// multiplier.
///
/// `body_calls("copy_nonoverlapping")` matches but `body_calls("size_of")` does
/// NOT → the `all_of` is **spared**. The absence of the `size_of` multiplier is
/// the difference between the foot-cannon and the correct call.
#[presents(SizeOfInElementCount)]
fn copy_good(src: &[u8], dst: &mut Vec<u8>, n: usize) {
    copy_nonoverlapping(src, dst, n);
}

fn main() {
    println!("antigen numeric-truncation example: see source for the affinity-pair.");
    println!(
        "Both siblings are #[presents]-marked, so audit lists both; the element-count path is spared by the FINGERPRINT (it doesn't bind). To read the bind/spare side by side, see antigen/tests/stdlib_family_fingerprints.rs."
    );

    let src = [1u8, 2, 3, 4];
    let mut dst = Vec::new();
    copy_bad(&src, &mut dst, 4);
    copy_good(&src, &mut dst, 4);
}
