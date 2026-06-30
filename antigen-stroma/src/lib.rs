//! # antigen-stroma — the read-write-constitute coordinate frame + base node-set
//!
//! The stroma is the **base** every antigen organ snaps to: a salsa-clocked relational base of
//! collision-free, cfg-aware, tier-honest nodes, read through a 3-axis coordinate frame.
//!
//! ## The organizing rule — CONFIG/OUTPUT split (ADR-067 §F.13)
//! Separate the authored/stable half from the derived/changing half; never let one word name both;
//! **lower-never-corroborates-up.** This crate encodes the rule structurally, not just in docs:
//! - [`node::digest::IdentityDigest`] (BLAKE3, signing, stable) vs [`node::digest::ShapeDigest`]
//!   (FNV, recomputable) — two distinct types, never conflated.
//! - [`node::locator::Locator`] (stable salsa key) vs [`node::node::Node`]'s changing digest fields.
//! - [`config_output::Config`] vs [`config_output::Output`] at every API boundary.
//!
//! ## The build epochs (epoch ≠ crate — both live here)
//! - **FRAME epoch** (this skeleton): the read CONTRACT + constituted BASE + query STUBS.
//! - **ENGINE epoch** (deferred, SAME crate): the ascent closure bodies + 4 semirings + condensation
//!   + degraded-mode population + CLASS-2. Every `todo!("engine epoch")` is an engine fill-point.
//!
//! ## Scope-line (co-captain-ruled)
//! `write ∈ constitute` on the compose base — there is NO sovereign write verb here. Sovereign
//! generation lives in `antigen-fingerprint`. The frame ships the tier-honesty (a syntactic read
//! cannot construct a `presents`-grade verdict), not the resolved-tier population.

#![forbid(unsafe_code)]
// `node::node` is intentional: the `node` module groups the identity/salsa-shape types, and `Node`
// (the salsa `#[input]` storage handle) lives in its own `node.rs` to keep it type-distinct from the
// `Locator` (the salsa key) and `StromaNodeId` (the semantic identity) — the locator/identity split
// is the deepest cut of the frame (§4). The path `node::node::{Node, Revision, Contract}` is pinned
// public API the born-red ATK suite depends on; renaming would break that contract for a style nit.
#![allow(clippy::module_inception)]

pub mod config_output;
pub mod db;

pub mod base;
pub mod constitute;
pub mod deferred;
pub mod fidelity;
pub mod node;
pub mod read;
pub mod scip;
pub mod write;

pub use db::StromaDb;
