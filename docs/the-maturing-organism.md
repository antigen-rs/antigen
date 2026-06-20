# The Maturing Organism

> antigen's headline capability in this release is the set of things it structurally
> refuses to do. This page is the one story that makes that sentence true.

You can read antigen's releases as an organism growing up. It learned to **recognize**
a failure by shape. It learned to **prove** a defense with a witness, not a claim. It
learned to **generate** a new recognition by generalizing from examples — and to screen
that generation against clean code in the same step, because a generator without a
self-screen is autoimmunity. Then it learned to **remember**: a failure-class stopped
being a catalog entry and became a thing with a history.

This release is the one where it learns to be *wrong about its memory — safely*.

That is what "maturing" means here, and it is not a metaphor reaching for weight.
Maturity is not knowing more. It is knowing what you cannot yet see, and refusing to act
past that horizon. A young system asserts. A mature one says "I can't tell yet, and here
is exactly when I'll be able to" — and means it structurally, in a way a later edit can't
quietly undo.

---

## The goal, before any organ: a conscience about forgetting

Lead with the goal, because every part derives from it. antigen's job is to make sure a
failure-class you once cared about is still defended at every site it could appear. The
nightmare — the one antigen exists to cut through — is a defense that's still needed but
silently gone: a stale guard, forgotten while the threat it caught is still live. Call it
by antigen's own internal name for it: `RoutingTableStale`, the routing table that says a
path is safe long after it stopped being safe.

Now give antigen a memory, and a new danger appears that earlier releases couldn't have:
the system can now be *wrong about its own memory*. It could decide a class is obsolete
and throw it away — and if it's wrong, it has re-introduced its own nightmare **from
inside the organ built to fight it.** A wrong forget is the one mistake that turns the
tool into the noise.

So v0.6's goal is not "antigen got a drift detector." The goal is **antigen got a
conscience about forgetting** — and the drift detector is one of its senses. Every organ
below is a faculty in service of that conscience: senses that gather evidence, a
classifier that integrates it, a ladder that acts reversibly-first, a gate that makes the
one irreversible act structurally rare, and a human-route that catches the cases the
machine can't decide.

Hold that frame and you can derive the anatomy. Meet the organs as a list and you'll
memorize parts.

---

## The arc: sense → classify → act, with a gate on the irreversible

v0.6 is a **reflex arc**. Three stations:

1. **Sense** — read a class's current state from two different channels.
2. **Classify** — fuse those channels into one verdict about the class.
3. **Act** — do something to the class, on a ladder ordered from reversible to
   irreversible.

The earlier releases built the *afferent* half of a different arc — the Learning-Core
that *makes* a class: mark a cluster of sites, anti-unify them into a draft, gate the
draft against clean code. That arc generates a class. v0.6 builds the half that lets a
generated class *live*: accumulate a history, mature, sense whether it's still earning
its keep, and be curated — eventually forgotten — when it isn't.

The whole arc, as one picture:

```
   git history ──► STOCK (life_record) ◄── the autobiography every sensor reads
                      │ score_trajectory()        is_retired() (a fold, not a flag)
                      ▼
        ┌─────────────┴──────────────┐
        ▼                            ▼
   SENSE (silent)              SENSE (loud)            + witness axis
   reader.rs                   adwin.rs                  (is this class
   "is the shape gone?"        "is the trajectory         still defended
   → SilentStatus              drifting down?"            by a live test?")
                               → DriftVerdict
        └─────────────┬──────────────┘──────────────────────┘
                      ▼
              CLASSIFY (discriminator)
              fuse the channels → ONE ClassVerdict
              ── if ANY channel is blind ⇒ RouteToHuman ──
                      ▼
              ACT (curate) — the moral center
              Keep · Hold · RouteToHuman · ReArm · FORGET
              Forget reachable from Obsolete ALONE (type-gated)
                      ▼
              back to STOCK: a Retired tombstone, or a Drifted event
              ── or out to a human (the v0.7 seam)
```

The visual version of this — every organ, every edge, the two boundaries — lives in
[the v0.6 anatomy](the-v06-anatomy.md). What follows is the walk.

---

## The reservoir: a class is a history, not a snapshot

Every sensor reads from one place — the **life-record**, a class's append-only
autobiography. It is a typed stream of events (`Born`, `Matured`, `Scored`, `Drifted`,
`Ratified`, `Retired`), and its load-bearing property is the one that makes everything
downstream trustworthy:

> **Current state is *derived*, never stored.** "Is this class retired?" is a fold over
> the event stream (`any(Retired)`), not a flag kept in sync. The story is the events;
> the prose summary is a one-way projection, never an input.

This is why the record is drift-immune the way `.git` is: it stores *what happened*, not
*what is true now*. A flag can desync from reality. A history that only ever grows
cannot. A forget, when it comes, is not an erasure — it's a `Retired` event *pushed onto*
the stream. The class's death is part of its biography, readable forever.

Teach this before any sensor, because the sensors are folds over this stream. A reader
who holds "events, not claims" can derive why drift-detection is a pure read of the
trajectory rather than a stateful counter that could fall out of step.

---

## The two senses: one streamless, one loud — and one that announces its own blindness

A class can be sensed two ways, because two kinds of class exist.

**The silent sense** (`reader.rs`) reads a class that emits no signal over time —
antigen's founding population, the bug nobody noticed. With no trajectory to watch, it
still splits the one cell curation needs most, by reading two already-shipped primitives
(is the shape still present? did a near-miss appear?):

- **Obsolete** — the shape is gone, no near-miss appeared, *and* the class was capable of
  noticing a near-miss if one had. A safe-to-forget *candidate*.
- **Dormant** — the shape is present but nothing trips it. Alive; keep.
- **Evading** — a near-miss appeared: the defect mutated just past the fingerprint. The
  red-queen signal, checked first so it's never masked.
- **Indeterminate** — the shape is gone, but the class *couldn't* have noticed a
  near-miss (too few discriminating conjuncts), so "gone" and "evaded" are
  indistinguishable. This one routes to a human. It does not guess.

**The loud sense** (`adwin.rs`) reads a class that *does* emit a trajectory — a stream of
affinity scores over time — and watches it for a downward change-point. This is the
drift-detector, and it carries the single most important idea in v0.6:

> **"I can't see drift yet" is a first-class answer, not a missing one.**

A statistical change is detectable only above a power threshold. Below it, detection is
*mathematically impossible* — not hard, impossible. At antigen's current scale, where a
class has matured a handful of times, that threshold is unreachable: a *correct* detector
**cannot** fire. So the detector's default verdict today is `UnderPowered` — and its
entire v0.6 value is that it says so honestly, and tells you exactly when it *will* be
able to see (`n*`, computed from the bound). A detector that fires zero and *says why* is
the correct organ. It is the same organ that fires correctly once trajectories lengthen,
with no code change.

The invariant that makes this real: **silence has two distinct causes — "no drift" and
"can't see" — and they never collapse into each other.** A bare boolean that smeared them
together would be exactly the silent-miscalibration antigen exists to catch. Drift-detection
and its moral consequences get their own page: [drift-detection and the moral
center](drift-detection-and-the-moral-center.md).

---

## The integration: where the witness changes the verdict

The two senses, plus a third signal — **the witness axis** (does a live test still defend
this class?) — converge in the `discriminator`, which fuses them into one `ClassVerdict`.
Two things happen here that decide everything downstream.

**The witness override.** A class whose *shape is gone* — the silent sense would call it
obsolete — but which still carries a **live witness** is `WellDefended`, not `Obsolete`.
The witness is the plausible *reason* the shape is gone: the guard held, so the defect
never recurred. Forgetting it would discard a *working* immunity. On the silence axis
alone, `WellDefended` and `Obsolete` are identical; the witness is the single input that
separates a defense that's done its job from a defense that's dead.

**The conservatism-JOIN.** Before the fused verdict can ever be `Obsolete` — the one
auto-forgettable cell — every channel must be able to see:

> **If any channel is blind — the loud sense `UnderPowered`, or the silent sense
> `Indeterminate`, or a garbage non-finite drift signal — the verdict is `RouteToHuman`,
> regardless of what the other channel says. A blind channel cannot endorse an
> irreversible forget.**

This is why, at v0.6's scale — where the loud sense is `UnderPowered` on every class by
default — the system **literally cannot auto-forget anything.** The honest behavior is
"the drift sense sees nothing yet; here's the streamless read; route the undecidable to a
person." Not a fabricated signal. The conservatism-JOIN is the highest-leverage safety
structure in the release, and it gets taught in full on its own page.

---

## The act: a ladder ordered reversible-first

`curate` is the only organ that *acts*. It maps a verdict to an action, and the actions
form a ladder whose ordering **is** the morality — from the action that spends nothing to
the one that can't be undone:

1. **Keep** (`WellDefended`) — the null action. The class is doing its job.
2. **Hold** (`Dormant`) — keep an alive-but-unwalked class. Reversible; discards nothing.
3. **RouteToHuman** (`RouteToHuman`) — escalate the undecidable. The conservative default.
4. **ReArm** (`Evaded`) — the red-queen response to active evasion: broaden the
   fingerprint, record a drift, discard nothing.
5. **Forget** (`Obsolete`) — the only irreversible, discarding action. The last rung,
   reachable from `Obsolete` and nothing else.

The gate is type-enforced, not convention. `Forget` is emitted only when the verdict's
`is_auto_forgettable` is true — and that is true for `Obsolete` alone:

```rust
// antigen/src/learn/curate.rs
match verdict {
    ClassVerdict::WellDefended => CurationAction::Keep,
    ClassVerdict::Dormant      => CurationAction::Hold,
    ClassVerdict::Evaded       => CurationAction::ReArm,
    ClassVerdict::RouteToHuman => CurationAction::RouteToHuman,
    // the single discarding exit — gated
    ClassVerdict::Obsolete => {
        if verdict.is_auto_forgettable() {
            CurationAction::Forget
        } else {
            CurationAction::RouteToHuman // unreachable while the contract holds; conservative if not
        }
    }
}
```

A future edit that tried to forget a `WellDefended` or `Evaded` class would have to
*delete the gate* to do it. The unsafe path does not exist to be reached by accident, and
a test pins it across every verdict:

```text
$ cargo test -p antigen --test atk_curate_forget_path
test atk_curate2_evading_never_reaches_forget ... ok
test atk_curate2_indeterminate_never_reaches_forget ... ok
test atk_curate3_double_retire_corrupts_autobiography ... ok
... 19 passed; 0 failed
```

The module's load-bearing property is not what it forgets — it is what it is
*structurally incapable* of forgetting.

---

## The co-native inverse: a learned class a human can read

One organ sits off the sense→act arc, on the boundary between the machine and the person:
the **serializer**, which turns a learned fingerprint back into DSL text — the exact
inverse of the parser that reads DSL text into a fingerprint.

It matters because of *which* rendering it is. A fingerprint can be printed three ways
(JSON, debug, DSL), but the DSL is the **privileged** one: it is the only rendering that
is also the parser's *input grammar*. The same text a human reads is the text the parser
consumes is the text the `#[antigen(…)]` macro compiles. Round-trip exactness —
`parse(serialize(fp)) == fp` — *is* co-nativeness: a machine-proposed draft becomes
human-ratifiable with no translation layer in between.

Completeness here is a compiler guarantee, not a test's hope. The constraint alphabet is
closed, so the serializer's match is exhaustive with no wildcard arm — add a new operator
and the serializer fails to compile until you write its case. (A `_` arm would re-open the
silent-variant-drop class; it's a rejectable change.)

---

## Honest scope — where v0.6's knowing stops

These are the boundaries of what is true today. Each is a fact about the present, not a
promise about later, and stating them plainly is part of the product.

- **The organs are a library; the end-to-end loop is not wired.** Every efferent organ
  above is a tested, composable `antigen::learn::*` API. What does *not* ship is a `cargo
  antigen` verb that drives sense → classify → act from end to end. The one wired verb is
  `propose` (the afferent half). The organs are real; the verb that runs them in sequence
  is the next release's frontier.

- **The affinity score is not a probability.** It's a 2-vector — `(recall, precision)` —
  deliberately with no total order, because the two trade off and a single number would
  hide the choice. Calibrating it into a probability is later work. The anti-scalar shape
  *is* the honesty: it refuses to report one confident number it hasn't earned. (See
  [drift-detection and the moral center](drift-detection-and-the-moral-center.md) for why
  the 2-vector is the right shape.)

- **The strange loop is unfired.** `propose` routes a draft to a human; it does not
  promote it. The honest sentence is *"antigen anti-unifies a draft and routes it to a
  human to ratify"* — never *"antigen immunized itself."* The system curating its own
  immune memory end-to-end is the next release; this one keeps the human in the loop on
  purpose, so trust can accrue before the loop closes.

- **The drift sense fires zero at this scale, by design.** `UnderPowered` is the default,
  not a bug. "Can't see drift yet, here's when I will" is the honest behavior and the v0.6
  value.

That last point about keeping the human in the loop is not modesty. It's the mechanism. A
machine that proposes honestly, shows the trade-off instead of an opaque score, and routes
what it can't decide is a machine a person can learn to trust — and that earned trust is
the thing the next release's autonomy will spend. Honest scope isn't a hedge against the
present. It's an investment in the future the roadmap describes.

---

## What to carry away

A reader who leaves with **"antigen learned to forget *carefully*"** has the system. A
reader who leaves with "antigen got several new modules" has the parts.

The three new senses refuse to answer for three different reasons — the drift sense
because the statistical power is absent, the affinity score because the order genuinely
doesn't exist, the curator because the action is irreversible while a channel is blind.
The reasons differ. The shape is the same: in each one, *the unavailable answer is a
first-class value* — a peer of the real answers, never a degenerate case of one — so the
type-checker forces every caller to reckon with it. That rhyme, repeating across three
unrelated mechanisms at the same small scale, is not a coincidence. It is what maturity
looks like in a type system: the can't-answer made unfakeable.

---

## See also

- [drift-detection and the moral center](drift-detection-and-the-moral-center.md) — the
  drift sense and the conservatism-JOIN, taught from first principles
- [the v0.6 anatomy](the-v06-anatomy.md) — every organ and edge in one visual
- [when not to use antigen](when-not-to-use-antigen.md) — the honest boundary of the tool
- [the immune-system guide, Chapter 11](the-immune-system-a-programmers-guide.md) — the
  efferent arc told as biology
- [the learning loop](the-learning-loop.md) — the afferent half: how a class is born
- [concepts](concepts.md) — the architectural concepts behind the story
