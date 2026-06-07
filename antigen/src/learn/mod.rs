//! The learning core (v0.4, ADR-045) — the affinity-maturation arm.
//!
//! Two halves that MUST co-ship (the single safety-tangle on the v0.4 chart):
//!
//! - **B — self-tolerance / the spare-clean gate**
//!   ([`self_tolerance`](crate::learn::self_tolerance)): the SELECTOR. Given a
//!   draft fingerprint and a corpus of *clean* code, it rejects any draft that
//!   matches a clean item (negative selection). This is the safety floor.
//! - **C — PROPOSE** (the anti-unify generator; sequenced after B): produces a
//!   draft fingerprint from a cluster of structurally-similar marked sites.
//!
//! **C must NEVER promote a draft without B green.** An ungoverned generator
//! over-generalizes (naive least-general-generalization drops the discriminating
//! leaves and collapses to "any Drop impl", which matches a CLEAN Drop sibling) —
//! the generator's own output IS the false positive. Shipping the generator
//! without the selector ships **autoimmunity**: antigen flagging clean code. B is
//! the negative-selection gate (the thymus) that makes the generator safe.

pub mod self_tolerance;
