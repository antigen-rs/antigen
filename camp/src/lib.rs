//! Camp — team coordination substrate for the antigen project.
//!
//! This crate uses antigen primitives (`#[antigen]`, `#[immune]`, Oracle
//! artifact-class, substrate-witness predicates) to coordinate the antigen
//! project's own team. Each campsite is a Rust module declaring an Oracle
//! with required signers + lifecycle state; `cargo antigen audit --root camp`
//! tells you the full team state.
//!
//! ## Why camp exists
//!
//! See `~/.claude/skills/camp/SKILL.md` for the design rationale. Short
//! version: the old SQLite-backed campsite skill accumulated friction under
//! Opus 4.7 + current harness (agents naturally substrate-checked rather
//! than CLI-logged status). Rather than patch the friction, camp redesigns
//! the coordination layer on top of antigen — the substrate-check pattern
//! agents already use IS the pattern antigen formalizes.
//!
//! ## Status query
//!
//! Quick:
//!
//! ```sh
//! cd camp
//! cargo antigen audit
//! ```
//!
//! Full readiness check (CI-gate-style; nonzero exit if any campsite open):
//!
//! ```sh
//! cd camp
//! cargo antigen audit --strict
//! ```
//!
//! ## Adding a new campsite
//!
//! See `~/.claude/skills/camp/SKILL.md` § "Adding a new campsite". The
//! shape is: create a new module in `src/campsites/`, declare the antigen
//! it presents + the immunity predicate (required signers), declare the
//! Oracle if it has lifecycle state, then commit.
//!
//! ## Why camp is canonical dogfood
//!
//! Antigen's first internal adopter is camp. Every primitive in
//! antigen-attestation (Ratification, Predicate, Signer, Oracle, state
//! machine, signature_strength tiers) is exercised here. External adopters
//! looking for "how do I actually use antigen for team coordination?" can
//! read this crate as the canonical reference.

pub mod antigens;
pub mod campsites;
pub mod roles;
