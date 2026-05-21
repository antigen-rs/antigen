//! Campsites — units of team coordination work for the antigen project.
//!
//! Each module is one campsite. The module name (timestamp-prefix +
//! semantic-slug) is the campsite's stable identifier. Each module
//! declares:
//!
//! 1. A struct (e.g., `pub struct BuildCamp;`) — typed handle for the
//!    campsite + carrying the doc-comment that describes the work
//! 2. A `pub fn done()` function with `#[immune(CampsiteOpen, requires = ...)]`
//!    — the discipline-claim that names required signers
//! 3. Optionally: an Oracle declaration via the camp/.antigen/oracles/
//!    folder for full Draft→Complete→Deprecated lifecycle (not all
//!    campsites need this; only ones with meaningful state transitions)
//!
//! Import order in this file is journey-order: active/in-flight first,
//! pending later. Compiler enforces nothing; convention only.
//!
//! ## First campsites (in journey-order)
//!
//! `build_camp` — the recursive seed. Camp's first campsite is "build
//! camp itself." Required signers: tekgy + team-lead. When this
//! attests as done, camp exists as canonical antigen-dogfood-example.
//! After this, the team's other in-flight work gets its own campsites
//! (Layer 1 source-dogfood, Layer 4 ADR-as-Oracle, tambear-discipline
//! expansion, etc.).

pub mod build_camp;
