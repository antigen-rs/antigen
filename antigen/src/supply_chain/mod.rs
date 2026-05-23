//! # Supply-Chain Defense Family — runtime substrate (ADR-025)
//!
//! This module hosts the runtime machinery that backs the
//! Supply-Chain Defense Family antigens declared in
//! [`crate::stdlib::supply_chain`]. The stdlib module declares the
//! *structural memory* (antigen unit-structs + biology cognates +
//! documentation); this module hosts the *active discipline* — sidecar
//! schemas, witness-leaf types, the Cargo manifest reader, and the
//! evaluation functions that the `cargo antigen verify` CLI subfamily
//! and the `audit_supply_chain` audit entry point drive.
//!
//! ## Sub-modules
//!
//! - [`crate::supply_chain::schema`] — JSON sidecar types persisted under
//!   `.attest/supply-chain/` (`DepAttestation`, `ContentHashRecord`,
//!   `MaintainerSnapshot`, `ReviewScope`, `SandboxKind`).
//! - [`crate::supply_chain::witness`] — substrate-witness leaf state types
//!   (`DepPinnedState`, `DepAttestedState`, `ContentHashState`,
//!   `MaintainerState`, `SandboxState`) — what each leaf evaluates to.
//! - [`crate::supply_chain::manifest`] — Cargo.toml reader; identifies pinned
//!   vs unpinned deps without pulling a full toml parser into the dependency
//!   tree (uses a minimal hand-rolled scanner — see ADR-002 Amendment 2:
//!   "compete where antigen cohesion serves").
//! - [`crate::supply_chain::evaluate`] — drives the witness leaves against a
//!   workspace root + the recorded sidecars and produces the per-antigen
//!   audit hints.
//!
//! ## What is NOT here
//!
//! - Live crates.io API integration — deferred to v0.3+ per ADR-025
//!   tooling-phase 2.
//! - Sandbox execution of build.rs / proc-macros — deferred to v0.4+
//!   per tooling-phase 3.
//! - Behavioral fingerprinting + Sigstore/SLSA federation — deferred to
//!   v0.5+ per tooling-phase 4.
//!
//! Each `_stub` evaluator in [`crate::supply_chain::evaluate`] returns the
//! "v0.X+" audit hint that names the limitation explicitly per ADR-005
//! Amendment 2 (honest-tier-naming requires explicit limitation surfacing).

pub mod evaluate;
pub mod manifest;
pub mod schema;
pub mod witness;
