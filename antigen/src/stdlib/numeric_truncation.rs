//! # Numeric-Truncation-Overflow Family — stdlib antigens (beta.2 voyage)
//!
//! Silent numeric-corruption classes. The build-now member is the famous
//! **`size_of`-in-element-count** foot-cannon: passing a *byte* count where an
//! *element* count is expected (`n * size_of::<T>()` as the count arg to a
//! raw-memory copy) copies `N * sizeof` elements instead of `N` → out-of-bounds
//! read/write → UB.
//!
//! Biology cognate: **silent mutation** — a base-pair flip that produces a
//! still-folding protein (compiles, runs, returns *a* value) but the wrong one;
//! no immediate symptom, corruption propagates.
//!
//! ## Antigen-category (ADR-028)
//!
//! `FunctionalCorrectness`: the miscounted raw copy produces a wrong *effect*
//! (an out-of-bounds read/write / UB).
//!
//! ## Scope (honest defect-slice)
//!
//! `LossyNumericCast` (`as`-cast type-signature-shape) and the arithmetic-overflow
//! / float-equality members are operator-shaped tells (no shipped leaf) →
//! charter. This family ships the clean **call-co-presence** `size_of`-in-count
//! member now.

use crate::antigen;

// ============================================================================
// 1. SizeOfInElementCount
// ============================================================================

/// A raw-memory copy (`copy_nonoverlapping` / `copy`) co-located with a
/// `size_of` — the byte-count-where-element-count-expected foot-cannon.
///
/// **Where in the wild:** `ptr::copy_nonoverlapping(src, dst, n * size_of::<T>())`
/// — the count arg of `copy_nonoverlapping` is in **elements**, not bytes, so
/// multiplying by `size_of` over-copies by a factor of `sizeof(T)` → OOB. clippy
/// has a correctness lint (`size_of_in_element_count`) for exactly this; it
/// recurs and the harm is memory corruption / UB.
///
/// **Tell (and its honest tier):** the coarse **co-presence** form —
/// `all_of([body_calls("copy_nonoverlapping"), body_calls("size_of")])` (a
/// raw-memory copy call AND a `size_of` in the same body). The precise "`size_of`
/// in the *count* arg position, not in a divisor" needs arg-position
/// introspection (`G2`-extended → charter). Unlike a generic co-occurrence, this
/// stays **named**: the co-presence of a raw-memory copy *and* `size_of` is
/// itself a strong, clippy-confirmed correctness signal (raw copies are rare and
/// `size_of`-near-a-raw-copy is the documented foot-cannon shape). The clean
/// sibling (a `copy_nonoverlapping` with an explicit element count and no
/// `size_of`) is spared.
///
/// **Witness:** the count is an element count (no `size_of` multiplier), OR a
/// `// SAFETY:` argument that the byte/element units are correct, OR miri.
///
/// **Category:** `FunctionalCorrectness` — the miscounted copy produces a wrong
/// *effect* (an out-of-bounds read/write / UB).
#[antigen(
    name = "size-of-in-element-count",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"all_of([body_calls("copy_nonoverlapping"), body_calls("size_of")])"#,
    family = "numeric-truncation-overflow",
    summary = "A raw-memory copy (copy_nonoverlapping) co-located with size_of — the byte-count-where-element-count-expected foot-cannon (n * size_of as a count arg copies N*sizeof elements → OOB). Named (clippy correctness); the precise arg-position check is G2-extended (charter).",
    references = [
        "https://rust-lang.github.io/rust-clippy/master/index.html#size_of_in_element_count",
        "ADR-040",
    ]
)]
pub struct SizeOfInElementCount;
