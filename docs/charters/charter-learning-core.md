# Charter — the Learning Core (COMPARE / LEARN: the adaptive immune system)

*A named, dependency-placed future home for the cluster that makes antigen **learn**. This is the
keystone deferred cluster: the only organ-system that GENERATES immune knowledge rather than applying it.
A future Cartographers/dream wave launches its expedition from this charter.*

**Tier:** deep-future (a chartered expedition). **Roots the whole downstream dependency tree.**

---

## Organ-identity (the dreamer's voice — what this IS)

The adaptive immune system. antigen-the-spine ships *authored* failure-class memory; this cluster is the
machinery that lets antigen *author its own* — cluster the felt-but-unnamed (`#[dread]`/`#[aura]` marks)
into proposed fingerprints, test them, promote the ones that bind real failures, prune the ones that bind
clean code. It is the difference between a catalog and a learner; between "a tool a human curates" and "a
system that matures." When this ships, antigen stops being only the structural memory of failure-classes
*we wrote down* and becomes the structural memory of failure-classes *it discovered*.

## The dreams (campsites) in this cluster

- **`dream/affinity-maturation-engine`** — THE keystone. cluster → propose → test → promote/prune. The
  germinal-center loop. **Near-slice (the falsification gate for the whole charter): the PROPOSE step** —
  cluster marked-unknowns by `structural_digest` (now in the Finding schema, ADR-039 §C) → emit ONE draft
  fingerprint on our own code. If the PROPOSE-slice can't produce a real draft, the charter's worth is
  unproven; gate the rest on it.
- **`dream/self-tolerance-negative-selection-engine`** — the governor. **CO-REQUISITE with maturation:
  ship together or ship autoimmunity** (an engine that generates fingerprints without negative selection
  floods the codebase with false positives — the same false-positive-storm the legibility spine, ADR-042,
  exists to prevent, now self-inflicted). Negative selection needs a defined "self" (next item).
- **`dream/intent-substrate-spec-alignment`** — the AIRE-analog: **self = captured intent**. Negative
  selection tests candidate fingerprints against "what was MEANT" — the deepest silent failure (correct
  code, wrong thing). This is what gives self-tolerance a "self" to be tolerant of.
- **`dream/negative-space-the-classes-nobody-names`** — the frontier the engine explores: the failure
  classes that have no name yet because no one has been bitten loudly enough to name them. The maturation
  engine's reason to exist (the silent-first population, origin.md's lesson, mechanized).

## Dependencies (what unblocks this)

- **The ADR-039 emit-seam (SHIPPED, build-now).** The maturation engine subscribes to the typed Finding
  stream — specifically `structural_digest` + `cluster-key` (both added per adversarial finding #12,
  precisely so this engine clusters by a field-lookup instead of re-parsing). The seam was the near-free
  precondition; it shipped, so this charter starts from a real signal.
- **A PROPOSE-slice demo (the build-now gate).** The charter does not launch on faith — the worth is made
  falsifiable: the PROPOSE-slice must produce one real draft fingerprint on antigen's own dogfood marks.
- **Self-tolerance + intent-substrate are internal co-requisites** (the three ship as one expedition, not
  three — splitting them ships an ungoverned learner).

## Could-combine-with

- The **SIZE/ADAPT charter** (`adversarial-evasion-red-queen`) — maturation pointed at evasion is the same
  engine with a moving target; the red-queen arms race is *why* adaptive immunity exists. Natural sequel.
- The **registry/herd charter** — self-tolerance is the registry's *named precondition* (you cannot share
  fingerprints across codebases until you can tell self from non-self); so this charter must precede it.

## Buildability / effort scoping (pathmaker's lane — grounded in the real code)

- **PROPOSE-slice: MODEST.** The inputs exist (the Finding stream carries `structural_digest`; clustering
  by digest+class is a group-by). The hard part is the *diff-to-common-tell* (extract the shared
  fingerprint shape from a cluster) — that wants the matched-constraint-shape the synthesis pass already
  computes, not just the digest. Real work, but bounded; the substrate is there.
- **Negative selection: MODERATE-to-HARD.** Testing a candidate fingerprint against "binds the cluster,
  spares clean code" is the affinity-pair discipline (adversarial finding #13) — antigen already has the
  scanner to run a fingerprint against a corpus; the corpus-of-clean-code is the new substrate.
- **Intent-substrate: HARD / research-adjacent.** "What was meant" has no clean substrate today (it's the
  AIRE-analog); the cheapest real anchor is `attested=(who, why)` (ADR-020) — captured intent as
  attestation, not inferred. The deeper version is genuinely open.
- **Net:** the PROPOSE-slice is the cheap falsifiable first step; the full learning core is a real
  expedition. Sequence: PROPOSE-slice demo → negative-selection (affinity-pair) → intent-substrate anchor.

## Invitation to deepen (for the next dreamer)

The germinal center is the most beautiful loop in immunology — somatic hypermutation + selection produces
antibodies against pathogens evolution never anticipated. The open frontier: does the engine mutate
fingerprints (somatic hypermutation) or only cluster-and-propose? The genome's-intent-for-the-proteome
deepening of intent-substrate (the naturalist's AIRE lane). And the strange loop: when the maturation
engine proposes a fingerprint for a failure-class in *antigen's own code*, antigen has learned to immunize
itself — the reflexive charter's seed lives here.
