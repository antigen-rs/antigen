//! The life-record — antigen's first persistent **append-only** substrate, the
//! organism's autobiography (ADR-059, the v0.6 taproot STOCK).
//!
//! # Why this exists (the missing stock)
//!
//! Before v0.6, [`propose`](crate::learn::propose::propose) was a pure function:
//! `(cluster, clean_corpus) -> Result<PromotedDraft, _>`, persisting nothing. A
//! class's affinity was a *flow* — a fresh score each call — with no *reservoir* to
//! accumulate it. Every v0.6 homeostasis loop (autoimmunity-pruner, forgetting-curve,
//! red-queen evasion) reads a per-class history as its afferent signal, and none can
//! close without a place that history lives. The life-record IS that reservoir.
//!
//! A trajectory only accumulates **forward** from when you start recording — you
//! cannot back-fill a time series. So the record-hook is a turn-zero install or a
//! permanent blind hole across the period the organism was first alive (the same
//! shape as the v0.5 loop-terminator that had to be built in turn one).
//!
//! # The contract (ADR-058 C1 + ADR-059 identity-amendment)
//!
//! 1. **The story is a TYPED EVENT-STREAM, not free text.** The authoritative form
//!    is an append-only sequence of typed [`LifeEvent`]s ([`LifeEvent::Born`],
//!    [`LifeEvent::Matured`], [`LifeEvent::Ratified`], …). A cold *agent* and a
//!    *human* must BOTH re-inherit the history without one NL-parsing the other's
//!    prose — so prose ([`LifeRecord::render`]) is a one-way *projection* of the
//!    events, never an input. (The two-readers seam, ADR-058, dissolves *by
//!    construction*: the typed stream is the agent-native form; prose is a
//!    rendering.) The [`LifeEvent::Ratified`] `why` is the **leaf-payload
//!    exception** (ADR-020): free human text is allowed *as a value at a typed
//!    leaf* (opaque attestation), never as the *structure* of the record.
//! 2. **Append-only.** [`append`](LifeRecord::append) is the sole mutator — there is
//!    no `set`/`delete`/`mutate`. A forget is a *pushed event*, not an erasure
//!    (antigen's first mutable substrate is mutable only by append).
//! 3. **Tombstone-not-silence.** Retiring a class appends a [`LifeEvent::Retired`]
//!    that *remains readable* in history — the cold agent's only signal a dead end
//!    was already walked. A record dropped on forget would silence the dead end.
//! 4. **Current-state is DERIVED, never stored** (the identity-amendment). The
//!    record stores *events* (what happened); "is this class retired?" is a
//!    [`fold`](LifeRecord::is_retired) over the stream, never a stored `is_retired`
//!    flag that must be kept in sync. Storing events (not claims) is what makes the
//!    record drift-immune like `.git`.
//!
//! Lives **committed in-tree** (ADR-059 lean): the record travels *with* the code,
//! like the bundled stdlib catalog — so a fresh agent in any checkout inherits the
//! same body. A gitignored per-machine cache would be un-shareable and break
//! persona-c's cross-context inheritance.

use serde::{Deserialize, Serialize};

/// Who performed a lifecycle action — read **structurally** by the agent, never by
/// NL-parsing prose.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Actor {
    /// A named human ratifier (the leaf-payload name is opaque attestation).
    Human(String),
    /// A named agent/session that acted (e.g. a CI run, a maturation pass).
    Agent(String),
    /// The system itself (an automatic, un-attributed action).
    System,
}

/// One typed event in a class's life. The story IS the ordered stream of these —
/// prose is a projection ([`LifeRecord::render`]), never a source.
///
/// Mechanical events carry no payload (they are facts the gate produced). The one
/// payload-bearing variant, [`LifeEvent::Ratified`], carries the leaf-payload
/// exception: a free-text human `why` AT a typed leaf (ADR-020), traversable
/// without parsing it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifeEvent {
    /// The class was first proposed/minted (anti-unified from a defect cluster).
    Born,
    /// An affinity-maturation pass tightened the class (a mechanical milestone).
    Matured,
    /// The class fired against a real defect site (it did its job).
    Fired,
    /// The class's per-class rate-stream drifted (a change-detector flagged it).
    Drifted,
    /// A human ratified the class into a standing defense.
    ///
    /// `who` is read structurally; `why` is the leaf-payload exception — opaque
    /// human text AS a value, never the record's structure.
    Ratified {
        /// The ratifier — matched structurally (`Actor::Human("alice")`), no NL-parse.
        who: Actor,
        /// Free human attestation text AT a typed leaf (ADR-020 opaque payload).
        why: String,
    },
    /// The class was retired/forgotten — a **tombstone** that persists in history
    /// (tombstone-not-silence): the cold agent's signal this dead end was walked.
    Retired,
}

/// A class's append-only life-record — the typed event-stream + a pure projection
/// to prose. Keyed by the class's kebab-case `#[antigen(name = "...")]` identifier.
///
/// The ONLY mutator is [`append`](Self::append). Current state (e.g.
/// [`is_retired`](Self::is_retired)) is always *derived* by folding
/// [`events`](Self::events) — never stored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifeRecord {
    /// The class this record belongs to (its kebab-case antigen name).
    class: String,
    /// The append-only event stream — the authoritative story (prose is derived).
    events: Vec<LifeEvent>,
}

impl LifeRecord {
    /// Open a fresh, empty life-record for `class`. Events are appended as the
    /// class lives (the first is normally [`LifeEvent::Born`]).
    #[must_use]
    pub fn new(class: &str) -> Self {
        Self {
            class: class.to_owned(),
            events: Vec::new(),
        }
    }

    /// The class this record belongs to.
    #[must_use]
    pub fn class(&self) -> &str {
        &self.class
    }

    /// Append a lifecycle event. **The sole mutator** — history only grows; earlier
    /// events are never altered or removed (append-only, ADR-059). A forget is a
    /// pushed [`LifeEvent::Retired`], not an erasure.
    pub fn append(&mut self, event: LifeEvent) {
        self.events.push(event);
    }

    /// Read the event stream — the authoritative story, consumed by *matching typed
    /// events*, never by NL-parsing prose (the ADR-058 C1 agent-native read).
    #[must_use]
    pub fn events(&self) -> &[LifeEvent] {
        &self.events
    }

    /// **Derived** current state: is this class retired? A fold over the stream
    /// (ADR-059) — antigen never *stores* the claim "class K is retired" (that would
    /// drift); it stores the [`LifeEvent::Retired`] *event* and derives the state.
    #[must_use]
    pub fn is_retired(&self) -> bool {
        self.events.iter().any(|e| matches!(e, LifeEvent::Retired))
    }

    /// Render the typed stream to human prose — a **pure one-way projection** of the
    /// events (ADR-058 / persona-c REQ-4). Identical event streams render identical
    /// prose; the prose carries no information not in the events, and there is no
    /// path to author prose independently of the stream (so story-vs-struct drift —
    /// antigen's own nightmare — cannot arise inside its own record).
    #[must_use]
    pub fn render(&self) -> String {
        let mut out = format!("Life of `{}`:\n", self.class);
        for event in &self.events {
            let line = match event {
                LifeEvent::Born => "  • born — proposed from a defect cluster".to_owned(),
                LifeEvent::Matured => "  • matured — affinity tightened".to_owned(),
                LifeEvent::Fired => "  • fired — flagged a real defect site".to_owned(),
                LifeEvent::Drifted => "  • drifted — its rate-stream shifted".to_owned(),
                LifeEvent::Ratified { who, why } => {
                    let by = match who {
                        Actor::Human(name) => format!("human {name}"),
                        Actor::Agent(name) => format!("agent {name}"),
                        Actor::System => "the system".to_owned(),
                    };
                    format!("  • ratified by {by} — {why}")
                },
                LifeEvent::Retired => {
                    "  • retired — tombstone; this dead end was walked".to_owned()
                },
            };
            out.push_str(&line);
            out.push('\n');
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_only_history_only_grows() {
        let mut rec = LifeRecord::new("demo-class");
        rec.append(LifeEvent::Born);
        assert_eq!(rec.events().len(), 1);
        rec.append(LifeEvent::Matured);
        assert_eq!(rec.events().len(), 2);
        assert!(matches!(rec.events()[0], LifeEvent::Born));
    }

    #[test]
    fn current_state_derives_from_events() {
        let mut rec = LifeRecord::new("demo-class");
        rec.append(LifeEvent::Born);
        assert!(!rec.is_retired());
        rec.append(LifeEvent::Retired);
        assert!(rec.is_retired());
    }

    #[test]
    fn render_is_a_pure_projection() {
        let mut a = LifeRecord::new("demo-class");
        a.append(LifeEvent::Born);
        a.append(LifeEvent::Matured);
        let mut b = LifeRecord::new("demo-class");
        b.append(LifeEvent::Born);
        b.append(LifeEvent::Matured);
        assert_eq!(a.render(), b.render());
    }

    #[test]
    fn ratified_who_is_read_structurally() {
        let mut rec = LifeRecord::new("demo-class");
        rec.append(LifeEvent::Ratified {
            who: Actor::Human("alice".into()),
            why: "bit us twice — worth a standing defense".into(),
        });
        let ratifier = rec.events().iter().find_map(|e| match e {
            LifeEvent::Ratified { who, .. } => Some(who.clone()),
            _ => None,
        });
        assert_eq!(ratifier, Some(Actor::Human("alice".into())));
    }
}
