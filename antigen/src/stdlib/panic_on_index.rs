//! # Panic-on-Index Family — stdlib antigens (beta.2 voyage)
//!
//! Out-of-bounds access classes. The build-now member is the **unsafe** form:
//! `get_unchecked` / `get_unchecked_mut` skip the bounds check, so an
//! out-of-bounds index is **Undefined Behavior**, not a panic — a soundness
//! hole, not merely a `DoS`.
//!
//! Biology cognate: **proprioception / spinal-reflex failure** — acting past the
//! body's valid range without the protective reflex (the bounds check) that
//! normally fires.
//!
//! ## Antigen-category (ADR-028)
//!
//! `FunctionalCorrectness`: `get_unchecked` on an out-of-bounds index produces a
//! wrong *effect* (UB — memory unsafety), the soundness failure the bounds check
//! exists to prevent.
//!
//! ## Scope (honest defect-slice)
//!
//! The **panic** form (`expr[i]` indexing a `Vec`/slice with an input-derived
//! index — `UncheckedIndexOnDynamicCollection`) is an Index-*operator* tell, not
//! a call leaf; it is charter-deferred to the operator-leaf increment. The
//! deref-coercion compile-vs-runtime gem (`(&arr)[OOB]` compiles where
//! `arr[OOB]` does not) is a specimen-garden exhibit, not a fingerprint member.
//! This family ships the clean **call-shaped** `get_unchecked` member now.

use crate::antigen;

// ============================================================================
// 1. GetUncheckedWithoutProof
// ============================================================================

/// A call to `get_unchecked` / `get_unchecked_mut` — the unchecked indexing
/// escape hatch whose out-of-bounds case is Undefined Behavior.
///
/// **Where in the wild:** `slice::get_unchecked` / `get_unchecked_mut` skip the
/// bounds check for performance; an out-of-bounds index is **UB** (not a panic),
/// so a wrong index is a *soundness* hole — silent memory corruption, not a
/// clean crash. Belongs to both this family and the unsafe-soundness boundary.
///
/// **Tell:** a call to `get_unchecked` / `get_unchecked_mut` —
/// `any_of([body_calls("get_unchecked"), body_calls("get_unchecked_mut")])`. A
/// clean call-shape: both are slice/`Vec`-specific method names with no stdlib
/// collision.
///
/// **Tier:** **named** — the call-tell is precise and the class (UB on OOB) is
/// real and miri-catchable.
///
/// **Witness:** a `// SAFETY:` comment proving the index is in-bounds + a miri
/// run, OR the checked `.get(i)` with a handled `None`.
///
/// **Category:** `FunctionalCorrectness` — `get_unchecked` on an out-of-bounds
/// index produces a wrong *effect* (UB / memory unsafety).
#[antigen(
    name = "get-unchecked-without-proof",
    category = AntigenCategory::FunctionalCorrectness,
    provenance = Provenance::Constructable,
    presentation = Presentation::Passive,
    fingerprint = r#"any_of([body_calls("get_unchecked"), body_calls("get_unchecked_mut")])"#,
    family = "panic-on-index",
    summary = "A call to get_unchecked / get_unchecked_mut — the unchecked-indexing escape hatch whose out-of-bounds case is Undefined Behavior (a soundness hole, not a panic). Named tier; witness = a SAFETY proof of in-bounds + miri.",
    references = [
        "https://doc.rust-lang.org/std/primitive.slice.html#method.get_unchecked",
        "ADR-040",
    ]
)]
pub struct GetUncheckedWithoutProof;
