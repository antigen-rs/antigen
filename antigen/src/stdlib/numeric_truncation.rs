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
/// **Tell:** the coarse **co-presence** form —
/// `all_of([body_calls("copy_nonoverlapping"), body_calls("size_of")])` (a
/// raw-memory copy call AND a `size_of` in the same body).
///
/// **Tier:** **suspected** (ADR-039 §C Amendment 1) — the co-presence *correlates*
/// with the dangerous region (an unsafe raw copy near a `size_of`) but **cannot
/// pinpoint** the defect, so it fires on idiomatic-correct both-calls code and a
/// `named` tier ("if it doesn't fire you're covered") could not carry that. The
/// member's own anti-correlated **fix** — `copy_nonoverlapping(s, d, n)` with an
/// element count and **no** `size_of` — *is* spared (the `all_of` co-anchor needs
/// both calls), which is why it is **demoted, not dropped**. But two CORRECT
/// both-calls siblings still fire (un-correlated, not anti-correlated): a copy by
/// element count whose body separately computes `size_of` for a bounds check, and
/// the legitimate single-element byte-copy `copy_nonoverlapping(p, q,
/// size_of::<u32>())` on `*u8` pointers. Those firings are honest labeled-recall
/// noise at suspected.
///
/// **Graduation to named is TYPE-AWARE, not a near-term syntactic leaf (honest —
/// do not over-promise an operator-leaf).** Pinpointing the defect needs **both**
/// arg-position introspection (`size_of::<T>()` *in the count argument*) **and**
/// the **pointee type** of the copy — because the arg-structure leaf alone is
/// *insufficient*: the correct `*mut u8` byte-buffer idiom
/// (`copy(dst: *mut u8, n * size_of::<T>())`) carries the very same `n * size_of`
/// shape and would still false-positive; sparing it requires knowing the
/// destination is `*u8` (a byte buffer), which is **resolved-type** information not
/// available at macro/scan time. So this graduates only at the **v0.4 type-aware
/// tier** (arg-position AND pointee-type), never at a syntactic operator-leaf.
///
/// **Witness:** the count is an element count (no `size_of` multiplier), OR a
/// `// SAFETY:` argument that the byte/element units are correct, OR miri.
///
/// **Category:** `FunctionalCorrectness` — the miscounted copy produces a wrong
/// *effect* (an out-of-bounds read/write / UB).
#[antigen(
    name = "size-of-in-element-count",
    category = AntigenCategory::FunctionalCorrectness,
    provenance = Provenance::Constructable,
    presentation = Presentation::Passive,
    fingerprint = r#"all_of([body_calls("copy_nonoverlapping"), body_calls("size_of")])"#,
    family = "numeric-truncation-overflow",
    summary = "A raw-memory copy (copy_nonoverlapping) co-located with size_of — the byte-count-where-element-count-expected foot-cannon (n * size_of as a count arg copies N*sizeof elements → OOB). SUSPECTED tier (ADR-039 §C Amd-1): the co-presence fires on idiomatic-correct both-calls code (a byte-buffer copy, a separate-bounds size_of), so it's a region-correlator demoted from named — but its own fix copy(n) (no size_of) IS spared, so demote not drop. Graduation to named is TYPE-AWARE (arg-position AND pointee-type), NOT a syntactic operator-leaf — the correct *u8 byte-copy still FPs without the pointee type.",
    references = [
        "https://rust-lang.github.io/rust-clippy/master/index.html#size_of_in_element_count",
        "ADR-040",
    ]
)]
pub struct SizeOfInElementCount;
