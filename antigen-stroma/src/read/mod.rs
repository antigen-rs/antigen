//! STEP 1 — the read CONTRACT (frozen FIRST). The base constitutes *against* this shape.
//!
//! The read-frame is 3 axes (ADR-069 §B): SOURCE × PERSPECTIVE × POLARITY, with a structurally-
//! present-but-empty T3 (mir-only) slot. The contract's whole job is **tier-honesty**: a lower tier
//! can NEVER corroborate up (ADR-069 §A), enforced in [`tier`] so a syntactic read literally cannot
//! construct a `presents`-grade verdict.
//!
//! Freeze this module before constituting the base — the base references these types, and reworking
//! a contract the base already depends on is the stall the build-order exists to prevent.

pub mod answer;
pub mod coord;
pub mod query;
pub mod tier;

pub use answer::{SnapshotHandle, TieredAnswer};
pub use coord::{Perspective, Polarity, ReadCoord};
pub use tier::{
    DetectionGrade, PresentsVerdict, ResolutionTier, corroborate, corroborate_presents,
};
