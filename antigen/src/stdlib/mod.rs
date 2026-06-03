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
//!   (beta.2 voyage), the RUSTSEC `crypto-failure` category seen developer-side:
//!   misuse of a present defense (non-constant-time secret comparison). Tells
//!   are call-anchored + safe-step-ABSENCE. Biology cognate: using the immune
//!   machinery wrong (the timing leak as a non-constant-time self/non-self read).
//! - [`deserialization`](crate::stdlib::deserialization) — the
//!   Deserialization-Trust-Boundary Family (beta.2 voyage), the deep tier of
//!   Mucosal-Boundary: untrusted bytes crossing into typed structs without the
//!   tight-junction (`deny_unknown_fields` absent → silent field drop;
//!   unbounded `from_reader`/`from_slice` → DoS). Biology cognate: gut mucosa.
//! - [`time_ordering`](crate::stdlib::time_ordering) — the
//!   Time-and-Ordering-Hazards Family (beta.2 voyage): the silent-in-tests /
//!   panic-in-prod clock footgun (`SystemTime::duration_since().unwrap()` panics
//!   on backwards-clock, never in tests). Biology cognate: circadian /
//!   signaling-timing failure.
//! - [`drop_panic`](crate::stdlib::drop_panic) — the Drop-and-Panic-Discipline
//!   Family (beta.2 voyage): a real `Drop` impl (`impl_of_trait("Drop")`) whose
//!   body reaches a panic source — panic-during-unwind aborts the process.
//!   Biology cognate: apoptosis gone wrong (teardown that triggers a cascade).
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
pub mod crypto_misuse;
pub mod deserialization;
pub mod dogfood;
pub mod drop_panic;
pub mod mucosal;
pub mod recurrent;
pub mod supply_chain;
pub mod time_ordering;
pub mod vcs_info_loss;
