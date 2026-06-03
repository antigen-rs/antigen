//! # Resource-Lifecycle-Leak Family — stdlib antigens (beta.2 voyage)
//!
//! Leaks: resources whose `Drop` never fires. The build-now member is the
//! **explicit-leak-primitive** form — `mem::forget` / `Box::leak` / `Vec::leak`
//! deliberately skip `Drop`; legitimate for `'static` upgrades but a silent leak
//! if misused, so the call is a flag whose witness is the *documented rationale*.
//!
//! Biology cognate: **failure of apoptosis / efferocytosis** — cells that should
//! die and be cleared (dropped) instead persist (senescent-cell accumulation).
//! `mem::forget` = explicitly suppressing the death signal.
//!
//! ## Sibling-family note (scout)
//!
//! This family and `drop_panic` are two halves of one Drop-Lifecycle axis:
//! `drop_panic` = drop fires-but-explodes; `resource_lifecycle` = drop
//! never-fires. They are NOT merged (distinct remedies — panic-free teardown vs
//! document-the-leak) but the kinship is real, recorded for the naturalist.
//!
//! ## Antigen-category (ADR-028)
//!
//! `FunctionalCorrectness`: `forget`/`leak` skip `Drop`, producing a wrong
//! *effect* (a leaked resource), or a deliberate ownership trick that needs
//! documenting.
//!
//! ## Scope (honest defect-slice)
//!
//! The "without rationale doc" half (a `// SAFETY:` / doc-absence check) is a
//! sensor-layer refinement (doc-substrate-alignment) and is charter-deferred;
//! the build-now member is the leak-*call* presence. The `RcCycleWithoutWeak`
//! (relational cycle-detection) and `GuardOrHandleImmediatelyDropped`
//! (`let _ = lock()` binding-tell) members are charter-deferred.

use crate::antigen;

// ============================================================================
// 1. DeliberateLeakNotDocumented
// ============================================================================

/// A call to an explicit-leak primitive (`mem::forget` / `Box::leak` /
/// `Vec::leak`) — `Drop` is deliberately skipped; the witness is the documented
/// rationale.
///
/// **Where in the wild:** "Don't use `std::mem::forget` unnecessarily."
/// `Box::leak` is legitimate for `'static` upgrades but silently leaks if
/// misused; `mem::forget` skips `Drop` (can leak resources OR be a deliberate
/// ownership trick). The call is a flag whose *witness* is the stated reason.
///
/// **Tell:** a call to `forget` / `leak` —
/// `any_of([body_calls("forget"), body_calls("leak")])` (`mem::forget` →
/// last-segment `forget`; `Box::leak` / `Vec::leak` → last-segment `leak`). A
/// user function also named `forget` / `leak` is the honest last-segment
/// false-positive the dial carries.
///
/// **Tier:** **named** on the call-presence — `forget` / `leak` are explicit,
/// intentional-leak primitives; their presence IS the flag.
///
/// **Witness:** a documented rationale (`Box::leak` for a known-`'static`
/// singleton is fine, *if said*), OR the resource is not actually leaked.
///
/// **Category:** `FunctionalCorrectness` — `forget`/`leak` skip `Drop`, leaking
/// the resource (a wrong effect) unless deliberate-and-documented.
#[antigen(
    name = "deliberate-leak-not-documented",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"any_of([body_calls("forget"), body_calls("leak")])"#,
    family = "resource-lifecycle-leak",
    summary = "A call to an explicit-leak primitive (mem::forget / Box::leak / Vec::leak) — Drop is deliberately skipped; the witness is the documented rationale. Named on call-presence; the doc-absence refinement is sensor-layer (charter).",
    references = [
        "https://doc.rust-lang.org/std/mem/fn.forget.html",
        "ADR-040",
    ]
)]
pub struct DeliberateLeakNotDocumented;
