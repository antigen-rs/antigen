//! The learning core — the affinity-maturation arm (v0.4 onward).
//!
//! # The v0.4 spine: PROPOSE governed by self-tolerance (ADR-045)
//!
//! Two halves that MUST co-ship (the single safety-tangle):
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

//! # The v0.6 maturing-organism arc (ADR-059..065)
//!
//! v0.4/v0.5 shipped the PROPOSE→gate spine above (the *afferent* arc: mark a cluster,
//! anti-unify a draft, gate it). v0.6 builds the organs that let a *learned* class
//! mature and be curated over its life — all **library-complete** (typed, tested,
//! composable); the live CLI curation loop that drives them end-to-end is v0.7 (no
//! production caller wires these into a `cargo antigen` verb yet):
//!
//! - **STOCK** — [`life_record`](crate::learn::life_record): the append-only
//!   autobiography (ADR-059), the persistent trajectory every other v0.6 organ reads.
//! - **MATURE** — [`affinity`](crate::learn::affinity) (the (recall, precision) height,
//!   ADR-061) + [`maturation`](crate::learn::maturation) (the germinal-center engine
//!   that climbs it).
//! - **READER** — [`reader`](crate::learn::reader): the drift/obsolescence sensor (the
//!   silent-core facet).
//! - **DISCRIMINATOR** — [`discriminator`](crate::learn::discriminator)'s
//!   `fused_classify`: the shared classifier spine that fuses the sensors into one
//!   per-class `ClassVerdict`.
//! - **ADWIN** — [`adwin`](crate::learn::adwin): the honest-blind batch drift-detector
//!   (ADR-065), the loud-class half of the decay-trigger.
//! - **CURATE** — [`curate`](crate::learn::curate): the moral-center efferent
//!   decision-layer (the forget-gate; the conservative default holds when any channel
//!   is blind, ADR-057).

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
