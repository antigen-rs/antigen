//! # Antigen
//!
//! Structural memory of failure-classes for Rust. Make implicit immunity explicit.
//!
//! This crate provides the macros and scanning library for the antigen ecosystem.
//! See <https://github.com/antigen-rs/antigen> for the project documentation,
//! design substrate, and origin story.
//!
//! ## Quick start
//!
//! Antigen meets you at any discipline level (per ADR-009 — adoption gradient).
//! Layer 1 — the minimum-viable form — has just `name` and `fingerprint`:
//!
//! ```ignore
//! use antigen::antigen;
//!
//! #[antigen(
//!     name = "panicking-in-drop",
//!     fingerprint = "impl Drop with unwrap/expect/panic in body",
//! )]
//! pub struct PanickingInDrop;
//! ```
//!
//! Layer 2 enriches with `family`, `summary`, and `references`:
//!
//! ```ignore
//! #[antigen(
//!     name = "panicking-in-drop",
//!     family = "boundary-violation",
//!     fingerprint = "impl Drop with unwrap/expect/panic in body",
//!     summary = "Drop impls must not panic; panic-during-unwind aborts the process.",
//!     references = ["https://doc.rust-lang.org/std/ops/trait.Drop.html#panics"],
//! )]
//! pub struct PanickingInDrop;
//! ```
//!
//! Mark code as vulnerable:
//!
//! ```ignore
//! use antigen::presents;
//!
//! #[presents(PanickingInDrop)]
//! impl Drop for MyType { ... }
//! ```
//!
//! Declare immunity with a witness:
//!
//! ```ignore
//! use antigen::immune;
//!
//! #[immune(PanickingInDrop, witness = no_panic_in_drop_test)]
//! impl Drop for SafeType { ... }
//! ```
//!
//! Then run `cargo antigen scan` to find unaddressed presentations across your
//! codebase.
//!
//! ## What this crate IS (intended)
//!
//! - The four core attribute macros (re-exported from `antigen-macros`)
//! - The [`scan`] module: scanning library used by `cargo-antigen` and consumable
//!   directly for custom integrations
//! - Future: `witness` module with phantom-type witness templates
//! - Future: `stdlib` feature flag re-exporting `antigen-stdlib`'s seed antigens
//!
//! ## What this crate is NOT
//!
//! - Not a documentation system. Documentation drifts; antigen declarations are
//!   checked by tooling.
//! - Not a replacement for tests, lints, deprecations, or formal verification.
//!   Antigen *composes* existing Rust ecosystem tools into a coherent immune-system
//!   surface.
//! - Not a logic-bug catcher. Antigen catches *failure-classes that have been
//!   named*; it does not detect novel logic errors.
//!
//! See the [`docs/expedition/`](https://github.com/antigen-rs/antigen/tree/main/docs/expedition)
//! directory in the repository for the full design intent.

// Re-export the proc-macros from antigen-macros so users can `use antigen::antigen`,
// `use antigen::presents`, etc.
pub use antigen_macros::{antigen, antigen_tolerance, descended_from, immune, presents};

pub mod audit;
pub mod scan;
