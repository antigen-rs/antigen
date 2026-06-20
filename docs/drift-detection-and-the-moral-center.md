# Drift Detection and the Moral Center

> A drift detector that fires zero, and *says why*, is not a broken detector. At
> antigen's scale it is the only correct one — and the reason why is the most important
> idea in this release.

A learned defense can go out of date. The shape of a bug shifts, the surrounding code
moves, and a fingerprint that used to catch a real failure-family starts catching
something subtly different — or nothing at all. A mature system needs to *feel* that
shift. But feeling it is the easy half. The hard half — the half that decides whether
antigen helps you or quietly poisons you — is knowing **when you cannot feel it yet**, and
refusing to act as if you can.

This page teaches the drift sense (ADWIN) and the structure that sits on top of it (the
conservatism-JOIN), from first principles. By the end, "antigen will never auto-forget a
live defense on a signal it can't read" should read not as a slogan but as a thing you can
point at in the types.

---

## What drift looks like to antigen

A class earns an **affinity** score each time it matures — a 2-vector `(recall,
precision)`. Recall is *bind-tight*: of the defect cluster, how much did the fingerprint
catch? Precision is *spare-clean*: of the clean code, how much did it correctly leave
alone? Over a class's life, those scores form a **trajectory** — a short time-series the
life-record keeps in order.

Drift, concretely, is a **downward change-point** in that trajectory: a moment after which
the scores are systematically worse than before. A recall drop means the defect mutated
past the fingerprint — evasion. A precision drop means the fingerprint started flagging
clean code — autoimmunity creeping in. These are different failures, so the detector runs
**per axis and ORs the result** — it never averages recall and precision into one number,
because a scalar would let a crater in one axis hide behind health in the other.

```rust
// antigen/src/learn/adwin.rs — the verdict the detector emits
pub enum DriftVerdict {
    Drift        { cut_index: usize, axis: DriftAxis, observed_diff: f64, /* … */ },
    NoDrift      { tightest_margin: f64 },
    UnderPowered { eps_cut: f64, max_observable: f64 },
}
```

The mechanism underneath is ADWIN — the field-standard streaming change-detector
(Bifet-Gavaldà, 2007). One detail of antigen's build is worth stating because it follows
from the life-record: antigen's ADWIN is **batch-pure.** It's a pure derivation over the
materialized trajectory, not a `&mut self` state-store that accumulates as scores arrive.
A stateful detector would keep its own running summary — a second copy of the truth that
can fall out of step with the append-only record. A pure fold over the record cannot
desync, because it owns no state to desync.

---

## `UnderPowered` is the spine, not the corner case

Here is the idea everything turns on.

A statistical change is detectable only when you have enough observations to tell signal
from noise. Below that threshold, detection isn't *hard* — it's **impossible**. No
detector, however clever, can distinguish a real downward shift from random wobble when
there aren't enough points to establish the wobble's size. This isn't a limitation of
ADWIN; it's a property of the question.

At antigen's current scale, a class has matured a handful of times — call it `n ≈ 4–8`.
At that scale the detectable-change threshold is *larger than the entire measurable range*
of the score (`2·ε_cut > 1.0`). A correct detector **cannot** fire, because the smallest
change it could honestly call "real" is bigger than any change the bounded score can
exhibit. So the honest verdict for essentially every class today is `UnderPowered`.

A lesser detector would round that off to "no drift detected" — a green check. antigen
refuses, and the refusal is the whole point:

> **Silence has two causes — "no drift" and "can't see drift" — and they are different
> verdicts.** `NoDrift` means *I looked and the trajectory is stable*. `UnderPowered`
> means *I cannot look yet; there isn't enough history*. Collapsing them into one boolean
> is precisely the silent-miscalibration antigen exists to catch.

This is the invariant the code calls **INV-ADWIN-1: `UnderPowered` is never suppressed
into `NoDrift`.** There is no wildcard arm anywhere in the detector that smears the two
together. And `UnderPowered` carries its own remedy — `eps_cut` and `max_observable`,
plus `n*` on demand: the exact number of observations at which the bound becomes
satisfiable. The detector doesn't just say "I can't see." It says **"I can't see *yet*,
and here is exactly when I will."**

The most rewarding thing to grasp about this organ is that it *reorganizes its own
behavior by scale, with no code change*. The same detector that returns `UnderPowered` on
every class today will fire correctly the moment trajectories grow past `n*`. It is not a
stub waiting to be replaced. It is the finished organ, behaving honestly at the scale it's
actually run at. A detector that fires zero and tells you why is not half a feature — it's
a whole one, scoped to the truth.

---

## The moral center: why a forget is the one act that needs a conscience

Every other organ senses or classifies. **Curate is the first that *acts*** — and one of
its actions, `Forget`, discards a failure-class. That makes `Forget` categorically
different from everything else antigen does.

A scan that misses is recoverable: you mark the site and rescan. A draft that's too broad
is caught by the self-screen before it's ever promoted. But a *wrong forget* throws away a
working defense, silently, and re-introduces antigen's founding nightmare — the stale guard
that says a path is safe long after it stopped being — **from inside the organ built to
fight exactly that.** Get curate wrong and antigen becomes the noise it exists to cut
through.

So the curator's defining property is not what it forgets. It is **what it is structurally
incapable of forgetting**, and that property is built from two structures.

### The reversible-first ladder

Curate's actions are ordered from the one that spends nothing to the one that can't be
undone, and the ordering *is* the morality:

| Action | For | Reversible? |
|---|---|---|
| **Keep** | `WellDefended` | nothing changes |
| **Hold** | `Dormant` | reversible; discards nothing |
| **RouteToHuman** | `RouteToHuman` | reversible; the conservative default |
| **ReArm** | `Evaded` | reversible; records a drift, discards nothing |
| **Forget** | `Obsolete` | **irreversible** — the only discarding action |

Four of the five rungs discard nothing. `Forget` is the last rung, and it's reachable from
`Obsolete` and nothing else — enforced in code, not by convention:

```rust
// antigen/src/learn/curate.rs — Forget is gated, structurally
ClassVerdict::Obsolete => {
    if verdict.is_auto_forgettable() {
        CurationAction::Forget
    } else {
        CurationAction::RouteToHuman // unsure ⇒ route, never forget
    }
}
```

`is_auto_forgettable()` is true for `Obsolete` alone. An edit that wanted to forget any
other verdict would have to delete the gate to do it. The unsafe path doesn't exist to be
reached by accident, and a test pins it across every verdict:

```text
$ cargo test -p antigen --test atk_curate_forget_path
test atk_curate2_evading_never_reaches_forget ... ok
test atk_curate2_indeterminate_never_reaches_forget ... ok
... 19 passed; 0 failed
```

### The conservatism-JOIN

The ladder protects you *given* a verdict. But the deeper question is: how does a class
ever become `Obsolete` in the first place? That's the integration step — `fuse_channels`
in the discriminator — and it carries the keystone safety property:

> **Before the fused verdict can be `Obsolete`, every channel must be able to see. If the
> loud sense is `UnderPowered`, OR the silent sense is `Indeterminate`, OR a drift signal
> arrives as a garbage non-finite number — the verdict is `RouteToHuman`, regardless of
> what the other channel says. A blind channel cannot endorse an irreversible forget.**

Read that against `UnderPowered` being the *default* and the consequence falls out:

> **At v0.6's scale, where the loud sense is `UnderPowered` on every class, the system
> literally cannot auto-forget anything.**

This is not a bug or an unfinished edge. It is the moral center working exactly as
designed. The honest behavior is "the drift sense sees nothing yet — here's the streamless
read from the silent sense — and the undecidable cases go to a person." Not a fabricated
loud signal to fill the silence. The conservatism-JOIN converts "I cannot decide" from a
silent miscalibration into an explicit human escalation — and at the current scale, it
routes essentially every forget-eligible class to a human rather than acting.

There's a hardening layer worth naming because it closes a sharp edge: a `Drift` carrying a
non-finite `observed_diff` (a `±∞` that could otherwise clear a finite threshold and
*fabricate* a confident drift) is treated as garbage — blind — and routed to a human at the
fusion input. The blindness check guards the value, not just the verdict label.

### The witness override

One more cell decides more than its size suggests. A class whose *shape is gone* — the
silent sense alone would call it `Obsolete` — but which still carries a **live witness** (a
test that still defends it) is `WellDefended`, not `Obsolete`. The witness is the plausible
*reason* the shape is gone: the guard held, so the defect never recurred. Forgetting it
would throw away a *working* immunity — the exact `RoutingTableStale` failure, re-created by
the organ meant to prevent it. On the silence axis alone, `WellDefended` and `Obsolete` are
indistinguishable; the witness is the single input that tells a defense that finished its
job from a defense that died.

---

## The rhyme: refusing is a first-class answer

Step back and the drift sense and the moral center are two instances of one shape. In each,
*the unavailable answer is a peer of the real answers, not a degenerate case of one* — and
because it's a peer in the type, every caller is forced to reckon with it:

- `DriftVerdict::UnderPowered` is a peer of `Drift` and `NoDrift` — **not** a `NoDrift`
  with low confidence. "Can't see" is a first-class inhabitant of the verdict.
- `ClassVerdict::RouteToHuman` is a peer of `Obsolete` and `Evaded` — and the
  conservatism-JOIN makes it the *forced* inhabitant the instant a channel goes blind.
  "Undecidable" is a first-class inhabitant of the curation verdict.

The two refuse for different reasons — the detector because the statistical power is
absent, the curator because the action is irreversible while a channel is blind. The
reasons differ; the representation is the rhyme. That rhyme is what lets you trust the
tool: antigen's headline capability is the set of things it structurally refuses to do, and
the proof is that, in the types, refusing is a first-class answer rather than a missing one.

---

## Honest scope

- **The drift sense fires zero at this scale, by design.** `UnderPowered` is the default,
  not a defect. The value is the honest "can't see yet, here's `n*`."
- **These organs are a library, not a wired CLI loop.** The drift sense, the discriminator,
  and the curator are tested `antigen::learn::*` APIs. No `cargo antigen` verb drives them
  end-to-end; the one wired verb is `propose`. The sequenced loop is the next release.
- **An O(log n) detection path exists in the code but is not on the hot path.** The shipped
  `detect` scans all `n−1` candidate splits; the bucketed exponential-histogram leg is gated
  behind a larger `n` than v0.6 ever reaches, where the rigorous all-`n` floor governs
  instead. Don't read the histogram structure as live on the detection path today.
- **The moral center is a witnessed invariant, not unconditional perfection.** What's
  guaranteed is "never auto-forgets a live defense on a blind or garbage signal" — the
  conservatism-JOIN plus the type-gated forget exit — not "always curates correctly." The
  guarantee is structural, and it is scoped.

---

## See also

- [the maturing organism](the-maturing-organism.md) — the whole v0.6 arc this page lives in
- [the v0.6 anatomy](the-v06-anatomy.md) — the drift sense and the JOIN in one visual
- [the immune-system guide, Chapter 11](the-immune-system-a-programmers-guide.md) — the
  conservatism-JOIN as the tolerance checkpoint on memory, in biology
- [reading a verdict](reading-a-verdict.md) — how to read the audit output
