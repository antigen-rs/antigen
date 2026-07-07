# DRAFT — ADR-071: The Loop is Edges, Not Nodes — Organ-Hood & the Afferent Organ Hierarchy

**Status:** CONVERGED — post-council ([020](020-adr-067-071-deconstruction-council.md))
+ human ratification rulings ([021](021-capability-expansion-and-the-afferent-organ-hierarchy.md)).
Ready for the `decisions.md` ratification ceremony. New ADR (next free number 071; confirm
at ratify-time). Derived from notebook [017](017-the-0.7-organ-council-boyd-ruling.md).
Relates to ADR-037 (the six-stage control loop) and ADR-067 (the stroma) + its Capability
Expansion Law amendment (draft [018](018-ADR-067-amendment-DRAFT-the-capability-expansion-law.md)).

---

## Context

The 0.7 organ layer (L2 — capabilities that sense on the stroma) was chartered as "5
organs + capstone." Two questions had to be settled before building: how is an organ
*individuated*, and does the layer decompose along ADR-037's six control-loop stages
(SENSE → COMPARE → ROUTE → ACT → FEEDBACK → SIZE)?

A deconstruct-council found a seductive trap — six of eight lenses mapped the organs onto
the loop and recommended re-cutting the layer into six stage-organs — and then a deeper
correction: the loop and the organs live in *different spaces*, and organs form an
*afferent hierarchy* (organs that sense from other organs), which the "5 organs + JOIN
capstone" framing entirely missed.

## Decision

### 1 · The loop is EDGES, not NODES *(the single governing principle)*

The ADR-037 control loop is the **connective flow-structure** — *edges*: how signal
moves, what feeds what, the coordinate a capability sits at. **Organs are the nodes** —
the built things, individuated by **build cost + blast-if-absent**, *never* by which stage
they occupy. Everything else derives from this one principle:

- **Don't cut organs along the loop.** An organ occupies a stage the way a point occupies
  a coordinate; two organs can share a stage (the three effector tiers), and a stage can
  host a thin organ now + a fat one later (ROUTE). The loop *indexes* organs; it does not
  *individuate* them.
- **The loop also indexes a set of non-organ self-regulation capabilities** (antigen
  instrumenting its own loop — see §6), *indexed-by, not constituted-by* the stages. This
  is folded in here as a section, **not a separate ADR** — it is the same principle applied
  to a second capability-set, so splitting it out would manufacture a false tension.
- **Shared-coordinate arbitration.** Where the organ track and the self-regulation track
  meet at a stage (ROUTE / ACT / FEEDBACK), the node there is **one build unit** consumed
  by both; sharing a coordinate is not being individuated by it. The **organ track owns the
  external interface contract; self-regulation hooks on top** (consumer, not co-owner).
- The **split-test** is the individuation rule's contrapositive at finer grain (split when
  a witness holds on a proper subset). "belongs-iff" is a *refinement*-rule axis only.
- **Warrant:** the loop can't be the build-WBS because the map between **build-space**
  (cost/blast) and **fault-space** (the loop's failure partition) is non-injective both
  ways — two organs share ROUTE; the capstone spans all six stages. *(This is the reason —
  not `decisions.md:9808`, which is about the user-code disturbance genus and is only a
  supporting analogy.)*

### 2 · Organ-hood, and the afferent hierarchy

A capability is an **organ** iff it has a **distinct BUILD** (distinct cost + distinct
blast-if-absent), witnessed by a **distinct beneficiary-SET** (who consumes it) **and** a
distinct afferent source. Its **afferent input may be:**
- a **raw stroma signal** (`reachable_from`, `field_at`, `provenance_of`, `blast_from`) —
  a **tier-0** organ (cochlea→sound, vestibular→motion), **or**
- **another organ's written-back output** — a **latent** organ ("sense from the senses":
  fuse sound + motion → spatial position; then act on the estimate → balance).

A latent organ is still a **node**; its inputs are just **edges from other nodes** (keeps
§1 clean). *(This generalizes the council's F9 finding: the four control-theory
self-regulation organs are latent organs aimed inward, sensing antigen's own outputs —
one tree, outward + inward.)*

### 3 · Beneficiary-SET, not "an existing consumer to re-point"

Conjunct-(b) means the organ has a **distinct set of consumers**, *not* "an existing
consumer to migrate" — the latter wrongly excludes greenfield organs (self/non-self
re-points nothing). "Who consumes it" is **individuation**; the ABSORB migration of
existing callers is a **separate lifecycle duty** ([018] §2). This also resolves the
cross-draft capstone collision (see §5): [018]'s expansion-law applies to "every capability
*that builds*," which the capstone's latent organs do.

### 4 · The OR-witnesses are real and named

The distinct-build witness is a genuine disjunction: **gate-only** (the three effector
tiers share the `blast_from` signal but differ by authorization gate) **or** **signal-only**
(self/non-self differs by afferent signal with no special gate) **or** **latent** (afferent
= another organ's output). No pairing required; the disjunction is what individuates.

### 5 · The capstone is the recursive LATENT-ORGAN TREE — not a JOIN

Base (tier-0) organs pre-wire during their own absorbs, so there is **no leftover join.**
The capstone is the **afferent tree above tier-0**: integrator organs + controller organs,
each **sensing from organs**, recursively — plus the orchestration and the meta-tooling
(macros, syntax, CLI, integrations) the full set makes possible. It is **open-ended /
multi-wave**, climbing tiers until the **fixpoint**: combining organ outputs yields nothing
new. Each latent organ builds + absorbs like any capability ([018]).

### 6 · Signal-algebra — the substrate the latent tree stands on

For a latent organ to sense from another organ, that organ's **output must be written back
into the stroma as a first-class fact/edge.** The dimensional climb is successive
**read → derive → write-back** rounds; dimensionality = tree depth. **Signal-algebra is
therefore load-bearing, not optional** — it gets its **own BUILD → ABSORB → PROVE
expedition, landing (built + absorbed) before the latent tree.**

### 7 · Known-completeness (honest, not over-claimed)

- The organ map is a **design decomposition, not a parallel build decomposition, until the
  stroma closure ships** — the closure gates the tier-0 organs (confirmed real + non-stale:
  `antigen-stroma/src/read/query.rs:25/35/47/61` + `scip.rs:36` are `todo!()`).
- "Known-complete" holds only **w.r.t. ADR-037's own *open* six-stage frame.** ADR-037 names
  four candidate-missing stages; by §2 + Ashby, each predicts a **structurally-guaranteed
  latent self-regulation organ** — name all four **on_hold** for the latent era:
  **observability** (reconstruct loop state from logs), **controllability** (move the
  setpoint, not just watch it drift), **delay/latency** (does feedback arrive before the
  cascade is irreversible), **stability-margin** (how close is the loop to oscillating).

## Layer stack & eras

`L0 stroma → L1 frame (intent-kernel + coordinate system) → L2 tier-0 organs (sense raw
stroma signal) → signal-algebra (write organ outputs back as facts) → L3 latent tree
(organs that sense from organs).`

- **`0.7` era** = stroma (done) + the tier-0 organs + signal-algebra (the bridge), each an
  additive `0.7.x` release (expand + deprecate).
- **`0.8` boundary** = contract the pre-stroma deprecations **+** open the latent theme.
- **`0.8` era** = the latent tree, climbing `0.8.x` to the fixpoint. *(Per [021] §C.)*

## Consequences

- The "5 organs + capstone" resolves to a **tier-0 base set + an open-ended latent tree**,
  not "~14 nodes + a JOIN." The effector splits into three authorization-tiers; several
  charter-concepts resolve to sub-slices or stroma-primitives.
- Build-order is gated by one unbuilt tier-0 node — the **stroma closure**.
- `antigen::learn` stays shipped-not-re-planned; the "germinal-center" residue is the
  reputation-update stroma primitive, homed in the SENSE organ.

## Relationship to existing ADRs

- **ADR-037** (the six-stage loop): unchanged as a *taxonomy of edges*; this ADR forbids
  using it as a build-WBS and *adds* its use as the index for a self-regulation
  capability-set.
- **ADR-067** + its Capability Expansion Law amendment ([018]): every organ — tier-0 or
  latent — is a capability subject to BUILD → ABSORB → PROVE.

## Open questions — resolved at convergence

- **Authorization-gate a sufficient sole axis?** — *Resolved (§4):* it is one of three
  OR-witnesses of distinct build (gate-only / signal-only / latent); no pairing required.
- **Control-plane its own ADR?** — *Resolved (§1):* a section here — it is the same
  "loop is edges" principle applied to the self-regulation capability-set.
