//! # Antigen
//!
//! Structural memory of failure-classes for Rust. Make implicit immunity explicit.
//!
//! This crate is in the **design phase**. The published `0.0.1` version reserves the
//! crate name and signals intent. Real macros, witness types, and structural
//! recognition primitives are under active design — see
//! <https://github.com/antigen-rs/antigen> for the design documents and progress.
//!
//! ## What antigen IS (intended)
//!
//! - A vocabulary of **failure-class declarations** (`#[antigen]`, `#[presents]`,
//!   `#[immune]`, `#[descended_from]`) that carry the memory of structural failure
//!   patterns *with the code itself*, not in human memory or commit history.
//! - A composition system: immunity propagates through derivation, calls, and trait
//!   implementations. Lessons learned about one type structurally inoculate
//!   structurally-similar types.
//! - A cargo extension (`cargo antigen`) for scanning, vaccinating, and auditing.
//!
//! ## What antigen is NOT
//!
//! - A documentation system. Documentation is itself vulnerable to drift; the antigen
//!   declarations live in the type system and are checked by tooling.
//! - A replacement for tests, lints, deprecations, or formal verification. Antigen
//!   composes existing Rust ecosystem tools into a coherent immune-system surface.
//! - A logic-bug catcher. Antigen catches *failure-classes that have been named*; it
//!   does not detect novel logic errors.
//!
//! See the workspace `docs/expedition/` directory for full design intent and API shape.
