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
//! - [`vcs_info_loss`] — the VCS-Information-Loss Family (ADR-026),
//!   11 antigens covering modern-git-workflow failure modes that erase
//!   load-bearing history. Central cognate: Immune amnesia (measles).
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

pub mod supply_chain;
pub mod vcs_info_loss;
