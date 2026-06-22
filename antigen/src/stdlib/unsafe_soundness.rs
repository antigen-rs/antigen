//! # Unsafe-Soundness-Boundary Family — stdlib antigens
//!
//! Soundness holes reachable from safe-looking code — the `unsafe`-primitive
//! call-shapes where a wrong invariant is **Undefined Behavior**, not a panic.
//! RUSTSEC's informational=unsound advisories + rustc's deny-by-default unsafe
//! lints (`mutable_transmutes`, `invalid_from_utf8_unchecked`) are the prior-art.
//!
//! Biology cognate: the **breached self/non-self membrane** — `unsafe` is the
//! explicit "I'm crossing the safety membrane, trust me." A wrong `unsafe`
//! invariant is a forged MHC marker (a foreign cell passing as self). This rhymes
//! hard with the Mucosal-Boundary family (trust-boundary) — it is mucosal-boundary
//! applied to the *memory-safety* membrane (the new-family-vs-deep-tier question
//! is a judgment call; the members are the same either way).
//!
//! ## Antigen-category (ADR-028)
//!
//! `FunctionalCorrectness`: each `unsafe` primitive, used with a wrong invariant,
//! produces a wrong *effect* — Undefined Behavior (memory corruption, an invalid
//! value, a UB string), the soundness the safe/unsafe boundary exists to keep.
//!
//! ## Why these are NAMED call-tells (effective-codomain)
//!
//! Every needle is a **rare/std-specific** unsafe primitive (`transmute`,
//! `assume_init`, `from_utf8_unchecked`, …) — a domain type will not have a method
//! by that name, so the needle alone restricts the codomain to the defect
//! population (the self-anchor rule). The *presence* of the call is current-scanner;
//! the precise size/lifetime/validity check is the v0.4 semantic tier.

use crate::antigen;

// ============================================================================
// 1. TransmuteSizeOrLifetimeMismatch
// ============================================================================

/// A `mem::transmute` / `transmute_copy` call — the most dangerous single
/// function in Rust; a size/lifetime/mutability mismatch is Undefined Behavior.
///
/// **Where in the wild:** rustc `mutable_transmutes` (deny-by-default — `&T → &mut T`
/// is UB), `wrong_transmute`; clippy `unsound_collection_transmute`,
/// `transmute_null_to_fn`. Transmute reinterprets bytes with no check — wrong
/// layout, a shortened lifetime, or an added `mut` is instant UB.
///
/// **Tell:** a call to `transmute` / `transmute_copy` —
/// `any_of([body_calls("transmute"), body_calls("transmute_copy")])`. The
/// *presence* is the named tell (transmute is `mem`-specific — a rare self-anchor
/// no domain type collides with); the precise size/lifetime/mutability check is
/// the v0.4 semantic tier.
///
/// **Tier:** **named** — `transmute` is rare/std-specific; its presence is itself
/// a strong correctness signal (rustc/clippy deny it).
///
/// **Witness:** a documented layout guarantee (`#[repr(...)]`) + a miri run, OR
/// the transmute is replaced by a checked conversion.
///
/// **Category:** `FunctionalCorrectness` — a layout/lifetime-mismatched transmute
/// produces a wrong *effect* (UB).
#[antigen(
    name = "transmute-size-or-lifetime-mismatch",
    category = AntigenCategory::FunctionalCorrectness,
    provenance = Provenance::Constructable,
    presentation = Presentation::Passive,
    fingerprint = r#"any_of([body_calls("transmute"), body_calls("transmute_copy")])"#,
    family = "unsafe-soundness",
    summary = "A mem::transmute / transmute_copy call — a size/lifetime/mutability mismatch is UB (rustc mutable_transmutes deny-by-default). Named (transmute is rare/std-specific); the precise layout check is v0.4 semantic.",
    references = [
        "https://doc.rust-lang.org/std/mem/fn.transmute.html",
        "ADR-040",
    ]
)]
pub struct TransmuteSizeOrLifetimeMismatch;

// ============================================================================
// 2. UninitMemoryAssumedInit
// ============================================================================

/// Reading uninitialized memory as initialized — `MaybeUninit::assume_init` /
/// `mem::uninitialized` — UB.
///
/// **Where in the wild:** clippy `uninit_assumed_init`, `uninit_vec`;
/// `mem::uninitialized` is deprecated *because* it is almost always UB. Treating
/// uninitialized memory as a valid value is instant UB.
///
/// **Tell:** a call to `assume_init` / `uninitialized` —
/// `any_of([body_calls("assume_init"), body_calls("uninitialized")])`. Both are
/// rare/std-specific self-anchors with **no common safe namesake** (you do not
/// name a safe method `assume_init`), so they spare the namesake-clean case and
/// the effective codomain is the defect population. The "is `T` safely-uninit?"
/// check is the v0.4 semantic tier; the presence is current-scanner.
///
/// **Why `zeroed` was DROPPED and `set_len` is NOT here (ADR-039 §C Amendment 1,
/// the spares-namesake sub-test).** An earlier form carried two more arms —
/// `zeroed` and `set_len` — claimed "all rare/std-specific." They are not:
/// - **`zeroed` → DROP (every tier).** The `zeroed` last-segment fires on
///   `bytemuck::zeroed()` / `Zeroable::zeroed()` — the **recommended-safe**,
///   trait-gated replacement for `mem::zeroed`. A needle that flags the
///   recommended remediation is inadmissible at any tier (the clean-sibling
///   collision, like `from_slice`/`elapsed`), so the arm is dropped, not demoted.
/// - **`set_len` → DROPPED from this named member; its risky form is a v0.4
///   charter, NOT a beta.2 suspected member (member-creation is out of seal
///   scope).** `set_len` fires on any domain buffer/builder's `.set_len(n)`, not
///   only the unsafe `Vec::set_len`-on-uninit. There is **no AST-feasible
///   discriminator**: risky-vs-safe turns on the **receiver TYPE** (`Vec` vs a
///   domain buffer) AND the **arg value** (`new_len ≤ initialized`) — *neither* is
///   syntactic (`x.set_len(n)` exposes neither `x`'s type nor whether `n` is
///   in-bounds). So it can never re-earn named via a syntactic leaf; the honest
///   framing is **permanent-suspected** (re-examine only when type/value-aware
///   analysis is available). The beta.2 fix is to **drop it from this named
///   member** (done); a dedicated suspected `set_len` member is **charter** (v0.4),
///   and the recall hole — an unsafe `Vec::set_len`-on-uninit is not flagged here —
///   is documented, not silently absorbed.
///
/// **Tier:** **named** — `assume_init` / `uninitialized` are rare/std-specific
/// unsafe primitives with no safe namesake, clippy-correctness-backed.
///
/// **Witness:** a `// SAFETY:` proving full initialization before the read, OR
/// miri/kani.
///
/// **Category:** `FunctionalCorrectness` — reading uninit memory produces a wrong
/// *effect* (UB / an invalid value).
#[antigen(
    name = "uninit-memory-assumed-init",
    category = AntigenCategory::FunctionalCorrectness,
    provenance = Provenance::Constructable,
    presentation = Presentation::Passive,
    fingerprint = r#"any_of([body_calls("assume_init"), body_calls("uninitialized")])"#,
    family = "unsafe-soundness",
    summary = "Reading uninitialized memory as initialized — MaybeUninit::assume_init / mem::uninitialized. UB (clippy uninit_assumed_init/uninit_vec). Named (rare/std-specific, no safe namesake). zeroed DROPPED (fires on the safe bytemuck::zeroed — ADR-039 §C Amd-1); set_len DROPPED-from-named (no AST-feasible discriminator — risky-vs-safe is receiver-type AND arg-value, neither syntactic → permanent-suspected; a dedicated suspected member is v0.4 charter, the recall hole documented); the safely-uninit check is v0.4 semantic.",
    references = [
        "https://doc.rust-lang.org/std/mem/union.MaybeUninit.html#method.assume_init",
        "ADR-040",
    ]
)]
pub struct UninitMemoryAssumedInit;

// ============================================================================
// 3. UnvalidatedFromUtf8Unchecked
// ============================================================================

/// `str::from_utf8_unchecked` / `_mut` on non-validated bytes — a UB `str`.
///
/// **Where in the wild:** rustc `invalid_from_utf8_unchecked`. `from_utf8_unchecked`
/// skips the UTF-8 validity check; a `str` containing invalid UTF-8 is UB (every
/// downstream `str` operation may misbehave).
///
/// **Tell:** a call to `from_utf8_unchecked` / `from_utf8_unchecked_mut` —
/// `any_of([body_calls("from_utf8_unchecked"), body_calls("from_utf8_unchecked_mut")])`.
/// Rare/std-specific self-anchor; the "were the bytes validated?" check is v0.4
/// semantic.
///
/// **Tier:** **named** — a precise, rare/std-specific unsafe primitive
/// (rustc-lint-backed).
///
/// **Witness:** the bytes were validated (or are a known-UTF-8 constant), proved
/// by a `// SAFETY:` + a check / miri.
///
/// **Category:** `FunctionalCorrectness` — an unvalidated `from_utf8_unchecked`
/// produces a wrong *effect* (a UB `str`).
#[antigen(
    name = "unvalidated-from-utf8-unchecked",
    category = AntigenCategory::FunctionalCorrectness,
    provenance = Provenance::Constructable,
    presentation = Presentation::Passive,
    fingerprint = r#"any_of([body_calls("from_utf8_unchecked"), body_calls("from_utf8_unchecked_mut")])"#,
    family = "unsafe-soundness",
    summary = "str::from_utf8_unchecked on non-validated bytes — a UB str (rustc invalid_from_utf8_unchecked). Named (rare/std-specific); the bytes-validated check is v0.4 semantic.",
    references = [
        "https://doc.rust-lang.org/std/str/fn.from_utf8_unchecked.html",
        "ADR-040",
    ]
)]
pub struct UnvalidatedFromUtf8Unchecked;
