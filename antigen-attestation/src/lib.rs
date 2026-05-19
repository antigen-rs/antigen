//! Ratification schema + substrate-witness predicate evaluator for antigen
//! discipline-witnesses (ADR-019 — Substrate-witness predicate family).
//!
//! This crate ships the on-disk schema for `.attest/*.json` sidecars, the
//! closed combinator grammar for substrate-witness predicates, and the
//! evaluator that `antigen::audit` invokes when an `#[immune]` or
//! `#[antigen_tolerance]` claim carries a `requires = ...` substrate
//! predicate.
//!
//! ## Three coupled pieces (per ADR-019 M1, ADR-007 anti-YAGNI)
//!
//! - **Predicate language** — closed combinator grammar (`all_of`,
//!   `any_of`, `not`) over a sealed set of leaf primitives. See
//!   [`Predicate`] + [`Leaf`].
//! - **Ratification schema** — serde-derived single source of truth for
//!   `.attest/<Antigen>.json` sidecars. Covers both immunity and tolerance
//!   ratifications via [`RatificationKind`] discriminator. See
//!   [`Ratification`].
//! - **Evaluator** — reads sidecars + git log + named docs; returns
//!   `(WitnessTier, AuditHint, EvidenceKind, Option<SignatureStrength>)`
//!   per the state-mapping table in ADR-019 M5. See
//!   [`evaluate::evaluate_predicate`].
//!
//! ## Tier-honesty (ADR-005 Amendment 3, extended)
//!
//! The substrate-witness primitive ships at [`EvidenceKind::SubstrateState`].
//! Per-kind ceilings:
//! - [`EvidenceKind::TypeSystemProof`] → reaches [`WitnessTier::FormalProof`]
//! - [`EvidenceKind::Behavioral`] → reaches [`WitnessTier::Execution`]
//! - [`EvidenceKind::SubstrateState`] → reaches [`WitnessTier::Execution`];
//!   **cannot reach** [`WitnessTier::FormalProof`]
//!
//! The ratchet-asymmetry property (ADR-019 §Decision): audit reports the
//! lower bound of verification work; promotions require evidence;
//! downgrades are automatic when evidence falters (fingerprint drift,
//! expiry, signer removal, chain-depth cap hit).
//!
//! ## Anti-laundering safeguards on delta-attestation
//!
//! See [`SignerBasis::DeltaFrom`] for the three layered safeguards
//! (chain-depth cap; cumulative-fingerprint tracking; required non-empty
//! rationale). Removing any one re-opens the laundering surface; the
//! three together close it (ADR-019 §Decision + adversarial T2-R).

#![forbid(unsafe_code)]

pub mod evaluate;
pub mod predicate;
pub mod schema;
pub mod tier;

pub use evaluate::{EvaluatedPredicate, EvaluationError};
pub use predicate::{Combinator, Leaf, Predicate, PredicateParseError};
pub use schema::{
    AntigenIdentifier, DocRef, ItemRatification, OracleRef, Ratification, RatificationKind,
    SchemaVersion, Signature, Signer, SignerBasis,
};
pub use tier::{AuditHint, EvidenceKind, SignatureStrength, WitnessTier};
