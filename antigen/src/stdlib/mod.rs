//! # antigen-stdlib
//!
//! Curated, ratified stdlib antigens shipped by the antigen project as
//! structural memory of known failure-classes. Per ADR-022 (stdlib-vs-
//! extension two-disciplines): stdlib growth is **research-driven** and
//! deliberately comprehensive (not recognition-driven like adopter
//! extensions).
//!
//! Sub-modules group antigens by family:
//!
//! - [`supply_chain`] — the Supply-Chain Defense Family (ADR-025),
//!   11 antigens covering the 2026+ dependency-boundary threat landscape.
//!   Biology cognate: Distributed-Boundary Innate-Immunity.
//! - [`vcs_info_loss`](crate::stdlib::vcs_info_loss) — the
//!   VCS-Information-Loss Family (ADR-026), 11 antigens covering
//!   modern-git-workflow failure modes that erase load-bearing history.
//!   Central cognate: Immune amnesia (measles).
//! - [`recurrent`](crate::stdlib::recurrent) — the Recurrent-Emergence
//!   Family (ADR-024 §Family 2), canonical failure-classes that re-emerge
//!   across project lifetimes (MSRV-creep, gitignore-drift, lockfile-churn).
//! - [`mucosal`](crate::stdlib::mucosal) — the Mucosal Boundary Family
//!   (ADR-027 + Amendment 1), trust-boundary failure-classes (undefended
//!   boundary, mis-delegated defense, stale tolerance).
//! - [`agentic_coordination`](crate::stdlib::agentic_coordination) — the
//!   Agentic-Coordination Failure-Class Family, failure-classes that emerge
//!   specifically in multi-session / multi-agent / human-LLM-collab workflows.
//!   Biology cognate: immunological memory loss during session gap.
//! - [`crypto_misuse`](crate::stdlib::crypto_misuse) — the Crypto-Misuse Family
//!   (beta.2 voyage). **CHARTERED, no shipped member yet:** the flagship
//!   non-constant-time secret comparison is a real failure-class but has no honest
//!   call-only fingerprint (a verify-anchor anti-aligns with the safe path; the
//!   real defect is `==` on a secret, needing the deferred name + operator leaves).
//!   See the module doc for the full reasoning + graduation path.
//! - [`deserialization`](crate::stdlib::deserialization) — the
//!   Deserialization-Trust-Boundary Family (beta.2 voyage), the deep tier of
//!   Mucosal-Boundary: untrusted bytes crossing into typed structs without the
//!   tight-junction (`deny_unknown_fields` absent → silent field drop;
//!   unbounded streaming `from_reader` → DoS). Biology cognate: gut mucosa.
//! - [`time_ordering`](crate::stdlib::time_ordering) — the
//!   Time-and-Ordering-Hazards Family (beta.2 voyage): the silent-in-tests /
//!   panic-in-prod clock footgun (`SystemTime::duration_since().unwrap()` panics
//!   on backwards-clock, never in tests). Biology cognate: circadian /
//!   signaling-timing failure.
//! - [`drop_panic`](crate::stdlib::drop_panic) — the Drop-and-Panic-Discipline
//!   Family (beta.2 voyage): a real `Drop` impl (`impl_of_trait("Drop")`) whose
//!   body reaches a panic source — panic-during-unwind aborts the process.
//!   Biology cognate: apoptosis gone wrong (teardown that triggers a cascade).
//! - [`resource_lifecycle`](crate::stdlib::resource_lifecycle) — the
//!   Resource-Lifecycle-Leak Family (beta.2 voyage): an explicit-leak primitive
//!   (`mem::forget` / `Box::leak` / `Vec::leak`) skips `Drop`. The sibling of
//!   `drop_panic` on the Drop-Lifecycle axis (drop never-fires vs
//!   fires-but-explodes). Biology cognate: failure of apoptosis.
//! - [`panic_on_index`](crate::stdlib::panic_on_index) — the Panic-on-Index
//!   Family (beta.2 voyage): `get_unchecked` / `get_unchecked_mut` skip the
//!   bounds check → out-of-bounds is Undefined Behavior. Biology cognate:
//!   proprioception / spinal-reflex failure.
//! - [`async_soundness`](crate::stdlib::async_soundness) — the Async-Soundness
//!   Family (beta.2 voyage): a hand-written `unsafe impl Send/Sync` asserts
//!   cross-thread safety the compiler cannot check (~40% of unsound advisories).
//!   Biology cognate: a mislabeled self/non-self marker at the thread boundary.
//! - [`numeric_truncation`](crate::stdlib::numeric_truncation) — the
//!   Numeric-Truncation-Overflow Family (beta.2 voyage): the `size_of`-in-element-
//!   count foot-cannon (a byte count where an element count is expected → OOB),
//!   shipped at the **suspected** tier (the call-co-presence fires on the
//!   idiomatic-correct byte-copy too, so it's demoted from named; its own fix is
//!   spared, so demote-not-drop; graduation is type-aware). Biology cognate:
//!   silent mutation.
//! - [`unsafe_soundness`](crate::stdlib::unsafe_soundness) — the
//!   Unsafe-Soundness-Boundary Family (beta.2 voyage): the `unsafe`-primitive
//!   call-shapes where a wrong invariant is UB, not a panic (`transmute`,
//!   `assume_init`, `from_utf8_unchecked`). Biology cognate: the breached
//!   self/non-self membrane (a forged MHC marker).
//! - [`dogfood`](crate::stdlib::dogfood) — antigen-internal dogfood antigens,
//!   failure-classes observed directly in antigen's own development and
//!   coordination substrate (v0.2 completion arc, 2026-05-24).
//!
//! ## How adopters use these
//!
//! Stdlib antigens are imported by type name and referenced from `#[presents]`
//! / `#[immune]` / `#[antigen_tolerance]` / deferred-defense sites the same
//! way as locally-declared antigens. Example:
//!
//! ```ignore
//! use antigen::stdlib::supply_chain::ContentHashMismatch;
//! use antigen::{presents, immune};
//!
//! #[presents(ContentHashMismatch)]
//! #[immune(
//!     ContentHashMismatch,
//!     requires = and(
//!         doc("content-hash:left-pad@1.3.0", "matches"),
//!         signed_trailer("Reviewable-Artifact"),
//!     ),
//! )]
//! fn use_left_pad_safely() { /* ... */ }
//! ```
//!
//! The active discipline for stdlib supply-chain antigens lives in the
//! `cargo antigen verify` CLI subfamily + the substrate-witness pipeline,
//! NOT in source-walking fingerprint matching.

pub mod agentic_coordination;
pub mod async_soundness;
/// The bundled stdlib catalog (v0.4 E0).
///
/// The compile-in projection of the flagship stdlib fingerprints, so a consumer
/// crate gets antigen's default repertoire without antigen's source on disk.
/// Closes the zero-hits-cliff.
pub mod catalog;
pub mod crypto_misuse;
pub mod deserialization;
pub mod dogfood;
pub mod drop_panic;
pub mod mucosal;
pub mod numeric_truncation;
pub mod panic_on_index;
pub mod recurrent;
pub mod resource_lifecycle;
pub mod supply_chain;
pub mod time_ordering;
pub mod unsafe_soundness;
pub mod vcs_info_loss;
