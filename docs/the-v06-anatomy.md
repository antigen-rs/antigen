# The v0.6 Anatomy

> Every v0.6 organ, every edge, and the two boundaries that matter — in one picture, then
> the cells worth pausing on. The prose walk is [the maturing
> organism](the-maturing-organism.md); this page is the map you keep open beside it.

v0.6 is a reflex arc: **sense → classify → act**, with a conservative gate on the one
action that can't be undone. Hold that and every box below has a place to sit.

---

## The whole arc

```mermaid
flowchart TD
    GIT["git history<br/>(SZZ defect/fix corpus)"]
    STOCK["STOCK — life_record.rs<br/>append-only autobiography<br/>events: Born · Scored · Drifted · Retired<br/><i>state is derived, never stored</i>"]
    MATURE["MATURE — affinity.rs + maturation.rs<br/>climbs a draft toward the Pareto frontier<br/>→ a Scored(Affinity) event"]
    SILENT["SENSE (silent) — reader.rs<br/>shape gone? near-miss?<br/>→ SilentStatus"]
    LOUD["SENSE (loud) — adwin.rs<br/>trajectory drifting down?<br/>→ DriftVerdict"]
    WITNESS["witness axis<br/>is the class still<br/>defended by a live test?"]
    CLASSIFY["CLASSIFY — discriminator.rs<br/>fuse_channels → ONE ClassVerdict<br/><b>any channel blind ⇒ RouteToHuman</b>"]
    CURATE["ACT — curate.rs (the moral center)<br/>Keep · Hold · RouteToHuman · ReArm · FORGET<br/><b>Forget ⟸ Obsolete ALONE (type-gated)</b>"]
    HUMAN(["a human ratifier<br/>(the v0.7 seam)"])
    SERIAL["serialize.rs — Fingerprint → DSL<br/>the co-native inverse, off-arc<br/>parse ∘ serialize = identity"]

    GIT -->|recomputable seed| STOCK
    MATURE -->|Scored event| STOCK
    STOCK -->|score_trajectory| LOUD
    STOCK -.->|the draft to climb| MATURE
    SILENT --> CLASSIFY
    LOUD --> CLASSIFY
    WITNESS --> CLASSIFY
    CLASSIFY -->|verdict| CURATE
    CURATE -->|Forget → Retired tombstone<br/>ReArm → Drifted event| STOCK
    CURATE -->|RouteToHuman| HUMAN
    SERIAL -.->|makes a draft<br/>human-ratifiable| HUMAN

    style STOCK fill:#E8F4F8
    style CLASSIFY fill:#FFE8B5
    style CURATE fill:#FFE4B5
    style HUMAN fill:#FFFFE0
    style SERIAL fill:#F0F0F0
```

The two highlighted boxes are where the safety lives: the **JOIN** (classify) and the
**gate** (act). Everything else feeds them.

---

## The reservoir every sensor reads

The life-record is not a station on the arc — it's the substrate under it. Its one
load-bearing property is that **current state is a fold over events, never a stored flag**,
which is why it can't drift out of sync the way a cached summary would.

```mermaid
flowchart LR
    A["append(event)"] --> S[("event stream<br/>Born · Scored · Drifted · Retired")]
    S --> R1["is_retired()<br/>= any(Retired)<br/><i>commutative, merge-safe</i>"]
    S --> R2["score_trajectory()<br/>= the Scored affinities, in order<br/><i>the stock every drift-sense reads</i>"]
    S --> R3["render()<br/>a one-way prose projection<br/><i>output only — never an input</i>"]
    style S fill:#E8F4F8
```

A `Forget` is not an erasure — it's a `Retired` event *pushed onto* the stream. The class's
death is part of its biography, readable forever.

---

## The verdict → action ladder (and the gate)

The discriminator emits one `ClassVerdict`; the curator maps it to one `CurationAction`.
The mapping is total and deterministic, and its ordering — reversible exits before the one
irreversible exit — *is* the morality.

```mermaid
flowchart TD
    WD["WellDefended"] --> KEEP["Keep<br/><i>null action</i>"]
    DOR["Dormant"] --> HOLD["Hold<br/><i>reversible</i>"]
    EVA["Evaded"] --> REARM["ReArm<br/><i>reversible — records a drift</i>"]
    RTH["RouteToHuman"] --> ROUTE["RouteToHuman<br/><i>reversible — the conservative default</i>"]
    OBS["Obsolete"] --> GATE{"is_auto_forgettable?<br/>true for Obsolete ALONE"}
    GATE -->|yes| FORGET["Forget<br/><b>the only irreversible exit</b>"]
    GATE -->|no| ROUTE

    style FORGET fill:#FFB6B6
    style GATE fill:#FFE4B5
    style ROUTE fill:#FFFFE0
```

The gate is type-enforced, not convention. An edit that wanted to forget any verdict but
`Obsolete` would have to *delete the gate* to do it — and a test pins that across every
verdict:

```text
$ cargo test -p antigen --test atk_curate_forget_path
test atk_curate2_evading_never_reaches_forget ... ok
test atk_curate2_indeterminate_never_reaches_forget ... ok
test atk_curate5_reversible_actions_never_retire ... ok
... 19 passed; 0 failed
```

---

## The conservatism-JOIN (the keystone cell)

This is the highest-leverage structure in the release: the rule that decides whether a
class can ever *reach* `Obsolete`. Before the fused verdict can be the one auto-forgettable
cell, **every channel must be able to see.**

```mermaid
flowchart TD
    L["loud sense (ADWIN)"] --> J{fuse_channels}
    SI["silent sense (reader)"] --> J
    W["witness axis"] --> J

    J -->|loud = UnderPowered| BLIND
    J -->|silent = Indeterminate| BLIND
    J -->|drift = non-finite garbage| BLIND
    BLIND["a channel is BLIND"] --> RTH["RouteToHuman<br/><i>a blind channel cannot<br/>endorse an irreversible forget</i>"]

    J -->|shape gone, no witness,<br/>both channels see| OBS["Obsolete<br/>→ forget-eligible"]
    J -->|shape gone BUT live witness| WD["WellDefended<br/><i>the witness override —<br/>a working immunity, kept</i>"]

    style RTH fill:#FFFFE0
    style WD fill:#90EE90
    style OBS fill:#FFE4B5
    style BLIND fill:#F0F0F0
```

Read this against one fact — at v0.6's scale the loud sense is `UnderPowered` on *every*
class by default — and the consequence falls out:

> **At v0.6's scale, the system literally cannot auto-forget anything.** Every class hits
> the blind-channel rule and routes to a human. That is the moral center working as
> designed, not an unfinished edge.

---

## The affinity height (why it's a 2-vector, not a number)

The score a class earns is `(recall, precision)` — and it deliberately has *no total
order*. The two axes trade off, so there is no single "better"; there is a frontier.

```mermaid
flowchart LR
    subgraph "Affinity = (recall, precision) — PartialOrd, NO Ord"
        A["A (.9, .6)"]
        B["B (.6, .9)"]
        C["C (.95, .8)"]
    end
    A -.->|incomparable<br/>partial_cmp = None| B
    C -->|dominates both| A
    C -->|dominates both| B
    style C fill:#90EE90
```

`A` and `B` are genuinely incomparable — one wins on recall, the other on precision — and
`partial_cmp` returns `None` to say so. `C` Pareto-dominates both. The "maturation ceiling"
is not a magic threshold; it's the frontier the draft can no longer Pareto-improve off of.
A single scalar would silently pick a point on that trade-off and hide the choice; the
2-vector exposes it. (The score is *not* a probability — calibrating it is later work, and
the anti-scalar shape is the honest placeholder.)

---

## The co-native inverse

Off the sense→act arc, on the boundary between machine and human: the serializer turns a
learned fingerprint back into DSL text — the exact inverse of the parser.

```mermaid
flowchart LR
    DSL["DSL text<br/>(what a human reads/edits)"] -->|parse| FP["Fingerprint (AST)"]
    FP -->|serialize| DSL
    FP -.->|the #antigen macro<br/>compiles the same text| MACRO["compiled matcher"]
    style FP fill:#E8F4F8
```

`parse ∘ serialize == identity`. The same text a human reads is the text the parser
consumes is the text the macro compiles — round-trip exactness *is* co-nativeness, with no
translation layer between the machine's proposal and the human's ratification. Completeness
is a compiler guarantee: the constraint alphabet is closed, the serializer's match is
exhaustive with no wildcard arm, so a new operator fails to compile until its case is
written.

---

## The two boundaries every reader should mark

```mermaid
flowchart LR
    subgraph LIB["library — tested antigen::learn::* organs"]
        O1[STOCK] --- O2[MATURE] --- O3[sense] --- O4[classify] --- O5[curate]
    end
    subgraph CLI["wired CLI"]
        P["cargo antigen propose<br/><i>the afferent half only</i>"]
    end
    LIB -.->|the sequenced<br/>sense→classify→act verb<br/>is the v0.7 frontier| CLI
    O5 -.->|RouteToHuman<br/>is a designed exit,<br/>not a failure| H(["human"])
    style CLI fill:#FFFFE0
    style H fill:#FFFFE0
```

1. **Library / CLI.** Every efferent organ is a tested, composable library API. What does
   not ship is a `cargo antigen` verb that drives the whole arc end-to-end — that's the next
   release. The one wired verb is `propose`.
2. **Human-in-the-loop.** `RouteToHuman` is the designed exit where the undecidable leaves
   the machine for a person — not a failure state. At v0.6's scale the JOIN routes most
   forget-eligible classes here, on purpose. The strange loop (antigen curating its own
   classes autonomously) is unfired; that's the v0.7 frontier.

---

## See also

- [the maturing organism](the-maturing-organism.md) — the prose walk of this anatomy
- [drift-detection and the moral center](drift-detection-and-the-moral-center.md) — the
  JOIN and the drift sense from first principles
- [diagrams](diagrams.md) — the scan / audit / witness-tier flow diagrams
- [the immune-system guide, Chapter 11](the-immune-system-a-programmers-guide.md) — the same
  anatomy as biology
