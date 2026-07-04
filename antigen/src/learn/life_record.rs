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

use crate::learn::affinity::Affinity;

/// A **typed** directional claim about a class's affinity trajectory — the
/// structural half of a hand-authored narration ([`LifeEvent::Narrated`]).
///
/// The claimed direction is a `Trend`, never free prose: the coherence-check
/// ([`LifeRecord::check_story_coherence`]) compares this *typed* claim against the
/// *typed* trajectory ([`LifeRecord::score_trajectory`]) — a structural
/// `Trend`-vs-`Trend` compare, never an NL-read of human text. This is persona-c
/// REQ-4's witness-link made structural: the part the system reasons over is typed,
/// so a hand-authored story is *witnessed-against* the struct **by construction**
/// (not "independently authored, audited by guesswork"). The same "typed claim
/// re-validated against typed events" shape as the §3 current-state-derived
/// invariant and ADR-057's lethal-corner — the wave's central three-sites principle.
/// (Aristotle's build-wave ruling on `loops/fate-record-is-the-missing-stock`.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Trend {
    /// The author claims affinity is rising (recall/precision improving run-over-run).
    Improving,
    /// The author claims affinity is falling (going autoimmune / losing recall).
    Declining,
    /// The author claims affinity is holding steady.
    Stable,
}

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
/// Mechanical events carry no payload (they are facts the gate produced). The
/// payload-bearing variants — [`LifeEvent::Ratified`] (the leaf-payload exception:
/// a free-text human `why` AT a typed leaf, ADR-020) and [`LifeEvent::Scored`] (the
/// affinity 2-vector at one maturation moment) — keep prose/data AT a typed leaf,
/// never as the record's structure.
///
/// `Eq` is NOT derived (only `PartialEq`): [`LifeEvent::Scored`] carries an
/// [`Affinity`] whose `f64` fields are `PartialEq` but not `Eq`. Nothing keys a
/// `HashMap`/`HashSet` on `LifeEvent` (verified), so the equality the tests use
/// (`assert_eq!`, needing only `PartialEq`) is unaffected — the score trajectory
/// is the reason, and `PartialEq` is what the record actually needs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LifeEvent {
    /// The class was first proposed/minted (anti-unified from a defect cluster).
    Born,
    /// An affinity-maturation pass tightened the class (a mechanical milestone).
    Matured,
    /// A maturation pass measured the class's [`Affinity`] (recall, precision) — the
    /// score-trajectory point (P1b). Recorded run-over-run so the trajectory can be
    /// read as a *stock* (the homeostasis loops sense it dropping). Carries the
    /// `Affinity` 2-vector directly (not parallel `f64`s) — one type, the same the
    /// maturation engine emits.
    Scored(Affinity),
    /// A **hand-authored** narration (persona-c REQ-4, the hand-authored WRITE-seam).
    ///
    /// `claimed` is a TYPED [`Trend`] — the structural directional claim the
    /// coherence-check reasons over (`Trend`-vs-trajectory, never an NL-read).
    /// `note` is the opaque ADR-020 leaf-payload: free human prose, rendered but
    /// NEVER parsed (same exception as [`LifeEvent::Ratified`]'s `why`). This keeps
    /// the witnessed-against part typed and the prose where ADR-020 allows it — a
    /// "maturing well" note whose `claimed: Improving` contradicts a declining
    /// trajectory is flagged STRUCTURALLY by [`LifeRecord::check_story_coherence`].
    Narrated {
        /// The TYPED directional claim — reasoned over structurally (no NL-parse).
        claimed: Trend,
        /// Opaque human prose (ADR-020 leaf-payload) — rendered, never parsed.
        note: String,
    },
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
///
/// `Eq` is not derived (it holds `Vec<LifeEvent>`, and [`LifeEvent`] is `PartialEq`
/// only — see its note).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    ///
    /// **Merge-order-invariant** (the committed-in-tree multi-writer property): this
    /// is an *existence* fold (`any(Retired)`), which is commutative — two branches
    /// appending to the same class, git-merged in EITHER interleaving, derive the
    /// same retired-state. There is no un-retire/`ReArmed` event, so retirement is
    /// monotone (once any branch retires, the merged stream is retired). An
    /// order-SENSITIVE derivation (a future `ReArmed` toggling against `Retired`)
    /// would NOT be merge-safe without a per-event total-order key — see the
    /// multi-writer seam on [`trajectory_direction`](Self::trajectory_direction),
    /// the one current order-sensitive read.
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
                LifeEvent::Scored(affinity) => format!(
                    "  • scored — affinity recall={:.2} precision={:.2}",
                    affinity.recall, affinity.precision
                ),
                LifeEvent::Narrated { claimed, note } => {
                    let dir = match claimed {
                        Trend::Improving => "improving",
                        Trend::Declining => "declining",
                        Trend::Stable => "stable",
                    };
                    format!("  • narrated [{dir}] — {note}")
                },
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

    /// The class's **affinity trajectory** — every [`LifeEvent::Scored`] affinity in
    /// append order (P1b). A *derived* structural read of the event stream (never a
    /// stored field), so it cannot drift from the events. This is the stock the v0.6
    /// homeostasis loops sense: a falling tail means the class is going autoimmune /
    /// losing recall run-over-run.
    #[must_use]
    pub fn score_trajectory(&self) -> Vec<Affinity> {
        self.events
            .iter()
            .filter_map(|e| match e {
                LifeEvent::Scored(affinity) => Some(*affinity),
                _ => None,
            })
            .collect()
    }

    /// The **derived** direction of the affinity trajectory — `None` when there are
    /// fewer than two score-points (no direction to read), else the [`Trend`] from
    /// the first scored point to the last. A structural fold over the
    /// [`Scored`](LifeEvent::Scored) events (never stored), so it cannot drift.
    ///
    /// Direction on the Pareto 2-vector: `Improving` iff the last point dominates the
    /// first (both recall and precision ≥, at least one strictly >), `Declining` iff
    /// the first dominates the last, `Stable` otherwise (equal, or a mixed
    /// trade-off neither dominates — honestly *not* a clean improvement, so not
    /// claimable as `Improving`).
    ///
    /// # ⚠ MULTI-WRITER SEAM (the committed-in-tree merge-order gap)
    ///
    /// Unlike [`is_retired`](Self::is_retired) (a commutative existence fold), this is
    /// an **order-sensitive** read: it folds first-vs-last over the `Scored` events in
    /// *append order*. When the life-record is committed-in-tree and shared (CI /
    /// multi-dev — which ADR-059's committed-in-tree makes day-one), two branches each
    /// appending `Scored` points are git-merged in SOME interleaving, and a
    /// first-vs-last read can depend on that interleaving — non-deterministic derived
    /// direction. (It also misses interior craters, 0.9→0.2→0.9 reading `Stable` —
    /// the same limitation noted for the ADWIN facet.) The robust fix, when the
    /// multi-writer regime arrives, is a per-`Scored`-event **total-order key that
    /// survives merge** — NOT file-append position — folded by a commutative
    /// reduction. The cheapest git-native key (uses the `.git` substrate ADR-059
    /// already leans on) is the event's commit-hash + commit-timestamp. DEFERRED for
    /// turn-zero single-writer; this is the seam to close before the score-trajectory
    /// reads are trusted under concurrent merge. (Surfaced by the build-adversarial's
    /// merge-order design-stress; the order-key shape is a design-Q routed to
    /// aristotle.)
    #[must_use]
    pub fn trajectory_direction(&self) -> Option<Trend> {
        let traj = self.score_trajectory();
        if traj.len() < 2 {
            return None;
        }
        let first = traj.first()?;
        let last = traj.last()?;
        let last_dominates = last.recall >= first.recall
            && last.precision >= first.precision
            && (last.recall > first.recall || last.precision > first.precision);
        let first_dominates = first.recall >= last.recall
            && first.precision >= last.precision
            && (first.recall > last.recall || first.precision > last.precision);
        Some(if last_dominates {
            Trend::Improving
        } else if first_dominates {
            Trend::Declining
        } else {
            Trend::Stable
        })
    }

    /// Coherence-check a **typed** hand-authored claim against the **typed**
    /// trajectory (persona-c REQ-4 — the hand-authored WRITE-seam).
    ///
    /// Returns `Some(StoryDivergence)` iff `claimed` contradicts the actual
    /// [`trajectory_direction`](Self::trajectory_direction) — e.g. a `Narrated`
    /// claim of [`Trend::Improving`] while the [`Scored`](LifeEvent::Scored) events
    /// say [`Trend::Declining`]. **A structural `Trend`-vs-`Trend` compare — never an
    /// NL-read of prose** (the prose lives in `Narrated.note` as an opaque ADR-020
    /// leaf and is never inspected here). This is the witness-against half of REQ-4:
    /// a hand-authored story cannot silently diverge from its struct, because the
    /// directional claim it carries is typed and re-validated against the typed
    /// events (the same shape as the §3 current-state-derived invariant and ADR-057's
    /// lethal-corner). `None` when the trajectory has no direction yet (< 2 score
    /// points) — nothing to contradict.
    ///
    /// What counts as a divergence (the asymmetry is deliberate — it tracks which
    /// way a hand-authored claim could HIDE A DROP, the failure REQ-4 exists to
    /// catch):
    /// - `Improving` ↔ `Declining` (either order) — direct contradiction.
    /// - `Stable` claimed while actually `Declining` — a *dishonest hedge*: "holding
    ///   steady / fine" over a real decline is downside-hiding drift, exactly the
    ///   story-vs-struct lie REQ-4 must surface. (Found by the adversarial's
    ///   no-self-witness re-attack: a `Stable` hedge was the gap a pure
    ///   `Improving↔Declining` opposition left open.)
    ///
    /// NOT flagged (benign): a matching claim; `Stable`-while-`Improving` (an honest
    /// under-claim — hides nothing); `Improving`/`Declining`-while-`Stable` (claiming
    /// a direction the trajectory doesn't yet show — over-claim, but not a
    /// drop-hiding lie; out of REQ-4's downside-hiding scope).
    #[must_use]
    pub fn check_story_coherence(&self, claimed: Trend) -> Option<StoryDivergence> {
        let actual = self.trajectory_direction()?;
        let opposed = matches!(
            (claimed, actual),
            (Trend::Improving | Trend::Stable, Trend::Declining)
                | (Trend::Declining, Trend::Improving)
        );
        opposed.then_some(StoryDivergence { claimed, actual })
    }
}

/// A flagged contradiction between a claimed and an actual trajectory direction.
///
/// Raised by [`LifeRecord::check_story_coherence`] when a hand-authored [`Trend`]
/// claim opposes the derived trajectory direction (persona-c REQ-4). Carries both
/// typed directions so the human review sees exactly the mismatch — no prose to parse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoryDivergence {
    /// What the hand-authored narration CLAIMED (the typed [`Trend`]).
    pub claimed: Trend,
    /// What the score-trajectory ACTUALLY did (derived from the events).
    pub actual: Trend,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trajectory_direction_reads_rising_falling_and_none() {
        // < 2 score points → no direction.
        let mut rec = LifeRecord::new("c");
        assert_eq!(rec.trajectory_direction(), None);
        rec.append(LifeEvent::Scored(Affinity::new(0.5, 0.5)));
        assert_eq!(
            rec.trajectory_direction(),
            None,
            "one point has no direction"
        );

        // Rising (last dominates first) → Improving.
        let mut up = LifeRecord::new("c");
        up.append(LifeEvent::Scored(Affinity::new(0.4, 0.4)));
        up.append(LifeEvent::Scored(Affinity::new(0.9, 0.8)));
        assert_eq!(up.trajectory_direction(), Some(Trend::Improving));

        // Falling (first dominates last) → Declining.
        let mut down = LifeRecord::new("c");
        down.append(LifeEvent::Scored(Affinity::new(0.9, 0.8)));
        down.append(LifeEvent::Scored(Affinity::new(0.5, 0.4)));
        assert_eq!(down.trajectory_direction(), Some(Trend::Declining));

        // Mixed trade-off (recall up, precision down) → Stable (neither dominates).
        let mut mixed = LifeRecord::new("c");
        mixed.append(LifeEvent::Scored(Affinity::new(0.5, 0.9)));
        mixed.append(LifeEvent::Scored(Affinity::new(0.9, 0.5)));
        assert_eq!(
            mixed.trajectory_direction(),
            Some(Trend::Stable),
            "a mixed trade-off neither dominates — honestly not a clean improvement"
        );
    }

    #[test]
    fn coherence_check_flags_only_opposed_claims() {
        let mut down = LifeRecord::new("c");
        down.append(LifeEvent::Scored(Affinity::new(0.9, 0.8)));
        down.append(LifeEvent::Scored(Affinity::new(0.5, 0.4))); // declining

        // Opposed claim (Improving vs actual Declining) → flagged.
        let d = down.check_story_coherence(Trend::Improving);
        assert_eq!(
            d,
            Some(StoryDivergence {
                claimed: Trend::Improving,
                actual: Trend::Declining,
            })
        );

        // Matching claim (Declining vs Declining) → not flagged.
        assert_eq!(down.check_story_coherence(Trend::Declining), None);

        // No trajectory yet → nothing to contradict.
        let empty = LifeRecord::new("c");
        assert_eq!(empty.check_story_coherence(Trend::Improving), None);
    }

    /// REGRESSION (adversarial no-self-witness re-attack): a `Stable` hedge over a
    /// REAL decline is a DISHONEST hedge that hides a drop — it MUST be flagged. (The
    /// pure `Improving↔Declining` opposition left this gap; the asymmetric fix closes
    /// the downside-hiding direction.)
    #[test]
    fn stable_claim_while_declining_is_flagged() {
        let mut down = LifeRecord::new("c");
        down.append(LifeEvent::Scored(Affinity::new(0.9, 0.8)));
        down.append(LifeEvent::Scored(Affinity::new(0.3, 0.2))); // real decline

        assert_eq!(
            down.check_story_coherence(Trend::Stable),
            Some(StoryDivergence {
                claimed: Trend::Stable,
                actual: Trend::Declining,
            }),
            "a Narrated claim of Stable ('holding steady / fine') over a real \
             DECLINING trajectory is a downside-hiding lie — the exact story-vs-struct \
             drift REQ-4 exists to catch. It must be flagged."
        );
    }

    /// And the benign side of the asymmetry: `Stable` over an IMPROVING trajectory is
    /// an honest under-claim (hides nothing) — NOT flagged.
    #[test]
    fn stable_claim_while_improving_is_not_flagged() {
        let mut up = LifeRecord::new("c");
        up.append(LifeEvent::Scored(Affinity::new(0.3, 0.2)));
        up.append(LifeEvent::Scored(Affinity::new(0.9, 0.8))); // improving

        assert_eq!(
            up.check_story_coherence(Trend::Stable),
            None,
            "Stable over Improving is a benign under-claim — it hides no drop, so it \
             is not a REQ-4 divergence (the asymmetry is deliberate)."
        );
    }

    #[test]
    fn score_trajectory_reads_scored_events_in_order() {
        let mut rec = LifeRecord::new("demo-class");
        rec.append(LifeEvent::Born);
        rec.append(LifeEvent::Scored(Affinity::new(0.9, 0.8)));
        rec.append(LifeEvent::Matured);
        rec.append(LifeEvent::Scored(Affinity::new(0.5, 0.4)));

        let traj = rec.score_trajectory();
        assert_eq!(
            traj.len(),
            2,
            "only the two Scored events are in the trajectory"
        );
        assert_eq!(traj[0], Affinity::new(0.9, 0.8));
        assert_eq!(traj[1], Affinity::new(0.5, 0.4));
        // The trajectory is DERIVED (a fold over events), never a stored field — it
        // cannot drift from the stream.
    }

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
