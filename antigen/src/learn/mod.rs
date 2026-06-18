//! The learning core (v0.4, ADR-045) — the affinity-maturation arm.
//!
//! Two halves that MUST co-ship (the single safety-tangle on the v0.4 chart):
//!
//! - **B — self-tolerance / the spare-clean gate**
//!   ([`self_tolerance`](crate::learn::self_tolerance)): the SELECTOR. Given a
//!   draft fingerprint and a corpus of *clean* code, it rejects any draft that
//!   matches a clean item (negative selection). This is the safety floor.
//! - **C — PROPOSE** (the anti-unify generator; sequenced after B —
//!   [`propose`](crate::learn::propose)): anti-unifies a cluster of
//!   structurally-similar marked sites into a DRAFT fingerprint (TO DISJUNCTION,
//!   not the naive drop-leaves collapse), then promotes it ONLY through B.
//!
//! **C must NEVER promote a draft without B green.** An ungoverned generator
//! over-generalizes (naive least-general-generalization drops the discriminating
//! leaves and collapses to "any Drop impl", which matches a CLEAN Drop sibling) —
//! the generator's own output IS the false positive. Shipping the generator
//! without the selector ships **autoimmunity**: antigen flagging clean code. B is
//! the negative-selection gate (the thymus) that makes the generator safe. The
//! C ══ B co-ship is enforced structurally:
//! [`propose`](crate::learn::propose::propose) is the only path to a promotable
//! fingerprint, and it routes every draft through
//! [`promote_if_safe`](crate::learn::self_tolerance::promote_if_safe). The raw
//! [`anti_unify`](crate::learn::propose::anti_unify) draft is a HYPOTHESIS
//! (ADR-044, observe-don't-declare) — a ratifiable suggestion, never an
//! auto-asserted class.

pub mod adwin;
pub mod affinity;
pub mod curate;
pub mod discriminator;
pub mod life_record;
pub mod maturation;
pub mod propose;
pub mod reader;
pub mod self_tolerance;
pub mod szz;
