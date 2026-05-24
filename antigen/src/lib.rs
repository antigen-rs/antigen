//! # Antigen
//!
//! Structural memory of failure-classes for Rust. Make implicit immunity explicit.
//!
//! This crate is the main library entry point for the antigen ecosystem. It
//! re-exports the five attribute macros from [`antigen-macros`](antigen_macros)
//! and provides the [`scan`] and [`audit`] modules consumed by `cargo-antigen`
//! and available for custom integrations (CI harnesses, IDE plugins,
//! programmatic audit tooling).
//!
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
//!     fingerprint = r#"
//!         item = impl,
//!         any_of([
//!             body_contains_macro("panic"),
//!             body_contains_macro("unreachable"),
//!         ])
//!     "#,
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
//!     fingerprint = r#"item = impl, body_contains_macro("panic")"#,
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
//! #[immune(
//!     PanickingInDrop,
//!     witness = no_panic_in_drop_test,
//!     rationale = "SafeType::drop uses non-panicking accessors only."
//! )]
//! impl Drop for SafeType { ... }
//! ```
//!
//! Then run `cargo antigen scan` to find unaddressed presentations and
//! `cargo antigen audit` to validate witness identifiers. See the
//! [`examples/`](https://github.com/antigen-rs/antigen/tree/main/antigen/examples)
//! directory for five worked examples covering each macro plus inheritance,
//! tolerance, and phantom-type witnesses.
//!
//! ## Macros (re-exported from `antigen-macros`)
//!
//! - [`#[antigen(...)]`](macro@antigen) — declare a named failure-class with a
//!   structural fingerprint (ADR-001, ADR-010)
//! - [`#[presents(...)]`](macro@presents) — mark code as exhibiting a
//!   failure-class's structural pattern (vulnerability declaration)
//! - [`#[immune(...)]`](macro@immune) — declare immunity with a witness
//!   reference (test, proptest, phantom-type proof, or external-tool delegation)
//! - [`#[descended_from(...)]`](macro@descended_from) — propagate antigen
//!   markers through an inheritance chain (ADR-013, ADR-018 §propagation)
//! - [`#[antigen_tolerance(...)]`](macro@antigen_tolerance) — document an
//!   intentional opt-out with rationale on the page (ADR-011)
//!
//! ### Deferred-Defense Family (ADR-023)
//!
//! - [`#[anergy(...)]`](macro@anergy) — deferred-but-muted posture; `until`
//!   REQUIRED; aging escalation via `anergy-active` / `anergy-stale` hints
//! - [`#[immunosuppress(...)]`](macro@immunosuppress) — surgical silencing
//!   with hard duration cap enforced at parse time (A4)
//! - [`#[poxparty(...)]`](macro@poxparty) — intentional exposure with
//!   structural compile-time isolation (A3; `antigen-poxparty` feature)
//! - [`#[orient(...)]`](macro@orient) — see-also context; lightest-weight
//!   deferred-defense primitive; all fields optional
//!
//! ## Modules
//!
//! - [`scan`] — workspace scanner. `scan_workspace()` walks `.rs` files,
//!   extracts antigen-related attributes, builds [`scan::ScanReport`].
//!   Includes the propagation walk (ADR-018) for `#[descended_from]` chains
//!   and the cross-crate enumeration path (ADR-017) via
//!   `enumerate_dep_crate_roots()`.
//! - [`audit`] — witness validation. `audit()` consumes a [`scan::ScanReport`]
//!   and validates each immunity's witness identifier against the workspace's
//!   function index, producing per-witness [`audit::WitnessStatus`] +
//!   [`audit::WitnessTier`] classifications (ADR-001 Amendment 1, ADR-013).
//!
//! ## What this crate IS
//!
//! - The five core attribute macros (re-exported from `antigen-macros`)
//! - The [`scan`] module: scanner library + propagation walk + cross-crate
//!   enumeration
//! - The [`audit`] module: witness resolution + tier classification
//! - Future: `witness` module with phantom-type witness templates
//! - Future: `stdlib` feature flag re-exporting `antigen-stdlib`'s seed antigens
//!
//! ## What this crate is NOT
//!
//! - Not a documentation system. Documentation drifts; antigen declarations
//!   are checked by tooling.
//! - Not a replacement for tests, lints, deprecations, or formal verification.
//!   Antigen *composes* existing Rust ecosystem tools into a coherent
//!   immune-system surface (ADR-002).
//! - Not a logic-bug catcher. Antigen catches *failure-classes that have been
//!   named*; it does not detect novel logic errors.
//!
//! See the [`docs/expedition/`](https://github.com/antigen-rs/antigen/tree/main/docs/expedition)
//! directory in the repository for the full design intent.

// Re-export the proc-macros from antigen-macros so users can `use antigen::antigen`,
// `use antigen::presents`, etc.
pub use antigen_macros::{antigen, antigen_tolerance, descended_from, immune, presents};

// Deferred-Defense Family (ADR-023): loudness-as-discipline for intentional
// non-immunity. Four structurally distinct postures — anergy, immunosuppress,
// poxparty, orient — each with parse-time enforcement and aging escalation.
pub use antigen_macros::{anergy, immunosuppress, orient, poxparty};

// Rollback-as-Triage primitive (ADR-026 §Rollback-as-triage discipline):
// sibling to the deferred-defense family per aristotle's fixup-orient-dual-
// signature resolution. Names a triage decision + commit to rollback within
// a tight time-bound; clinical-medicine grounded.
pub use antigen_macros::triage_commit;

// Convergent-Evidence Family (ADR-024): seven primitives for backward-
// looking evidence aggregation plus the two public enums (WitnessClass +
// SeedKind) adopters supply as macro arguments.
pub use antigen_macros::{adcc, clonal, crossreactive, diagnostic, igg, monoclonal, polyclonal};
pub use convergent::{SeedKind, WitnessClass};

// Recurrent-Emergence Family (ADR-024 §Family 2 + scientist HOW-spec
// cf2a2317 + aristotle Reading-A pre-authorization 744471a3): six
// present-looking primitives for noticing-and-anchoring recurring
// failure-classes across substrates. Cognitive-organizational +
// immunology-proper + clinical-medicine dual-axis grounding.
pub use antigen_macros::{chronic, crystallize, itch, recurrence_anchor, saturate, strand};

pub mod audit;
pub mod scan;

/// Stdlib of curated, ratified antigen declarations.
///
/// Per ADR-022 (stdlib-vs-extension): stdlib growth is research-driven and
/// deliberately comprehensive. Adopters import these declarations the same
/// way they would locally-declared antigens.
///
/// See [`stdlib::supply_chain`] for the v0.2 Supply-Chain Defense Family
/// (ADR-025).
pub mod stdlib;

/// Public types for the Convergent-Evidence Family (ADR-024).
///
/// Hosts [`convergent::WitnessClass`] (independence-checking categories
/// for `#[diagnostic]`) and [`convergent::SeedKind`] (non-deterministic
/// seed enforcement for `#[clonal]`). These are first-class public types
/// — adopters use them in macro arguments.
pub mod convergent;

/// Runtime substrate for the Supply-Chain Defense Family (ADR-025).
///
/// Hosts the sidecar schema types (`DepAttestation`, `ContentHashRecord`,
/// `MaintainerSnapshot`, `ReviewScope`, `SandboxKind`), the substrate-
/// witness state types (`DepPinnedState`, `DepAttestedState`,
/// `ContentHashState`, `MaintainerState`, `SandboxState`), the minimal
/// Cargo-manifest reader, and the per-witness evaluator functions that
/// `cargo antigen verify` and the supply-chain audit pipeline drive.
pub mod supply_chain;

/// Public types for the VCS-Information-Loss Family (ADR-026).
///
/// Hosts [`vcs::TriageDecision`] (the 5-color triage classification for
/// `#[triage_commit]` declarations) and [`vcs::ServerSideEnforcementMode`]
/// (friction-only vs structural enforcement). The 11 stdlib antigens, the
/// substrate-witness evaluators, and the `cargo antigen vcs` CLI integration
/// land alongside this module as ADR-026 implementation completes.
pub mod vcs;
pub use vcs::{ServerSideEnforcementMode, TriageDecision};

/// Antigen-Category taxonomy (ADR-028).
///
/// Hosts [`AntigenCategory`] — the first-class two-variant enum
/// (`SubstrateAlignment` | `FunctionalCorrectness`) that qualifies every
/// antigen declaration and shapes witness requirements, audit routing, and
/// scan/audit CLI filtering.
pub mod category;
pub use category::AntigenCategory;
