//! The read CONTRACT — the typed query surface the base is constituted against.
//!
//! The read-frame is 3 axes (ADR-069 §B): SOURCE × PERSPECTIVE × POLARITY, with a structurally-
//! present-but-empty T3 (mir-only) slot. The contract's whole job is **tier-honesty**: a lower tier
//! can NEVER corroborate up (ADR-069 §A), enforced in [`tier`] so a syntactic read literally cannot
//! construct a `presents`-grade verdict.

pub mod answer;
pub mod coord;
pub mod query;
pub mod tier;

pub use answer::{SnapshotHandle, TieredAnswer};
pub use coord::{Perspective, Polarity, ReadCoord};
pub use tier::{
    DetectionGrade, PresentsVerdict, ResolutionTier, corroborate, corroborate_presents,
};
