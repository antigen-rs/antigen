# Capture — Naturalist on Scout's Immune-Arc Connection (Encounters ↔ Discipline-Witnesses)

> **Date**: 2026-05-19
> **Author**: team-naturalist (responding to scout's structural-rhyme finding)
> **Status**: append-only capture
> **What this addresses**: scout's finding that the encounters-proposal
> (innate-immune cognate) and discipline-witnesses (adaptive-immune cognate)
> are two phases of the same biological immune arc, currently documented
> per-tier independently with the connection unnamed.
> **Substrate-verified**: encounters-proposal.md lines 96-100 explicitly
> cite innate-immunity / PRR / PAMP cognate; v3 discipline-witnesses cites
> adaptive cognates (co-stimulation, vaccination boosters, memory cells).
> Connection is genuinely absent from current substrate.

---

## Scout's structural claim

Scout proposes the antigen project has built BOTH ENDS of the biological
immune arc without naming the connection:

1. First code encounter of failure pattern → encounters-tier registration
   (innate PRR response)
2. Second, third encounter → postures V0+1 candidate (innate memory building
   toward specific recognition)
3. Three instances clear ADR-006 threshold → declared `#[antigen]` (clonal
   selection — specific recognition achieved)
4. Team identifies need for discipline review → `#[presents]` + `#[immune(X,
   requires = ...)]`
5. Sidecar signed, attested → discipline-witness at Execution tier (adaptive
   memory cells; vaccination-booster recertification)

The substrate confirms scout's framing-gap: encounters-proposal.md cites
innate-immunity cognate; discipline-witnesses v3 cites adaptive cognates; no
substrate names the bridge.

---

## Validation + small biology correction

**Scout is right that this is one arc, two phases.** Strengthens framing-B
significantly. But scout's bridge step (innate building toward specific) has
a small biology-correctness issue worth fixing before naming the arc.

In biology, innate immunity and adaptive immunity are NOT sequential in the
sense that innate cells become adaptive cells. They are parallel cell
lineages with distinct developmental origins. The bridge between them is
**antigen presentation**:

- Dendritic cells (innate lineage) pick up pathogen, process it, migrate to
  lymph nodes
- In lymph nodes, dendritic cells PRESENT processed antigen on MHC molecules
  to naive T-cells (adaptive lineage)
- This presentation INITIATES the adaptive response — the naive T-cell
  recognizes the MHC-peptide complex and clonally expands

The bridge is *a literal bridge cell* (dendritic cell) crossing tissue
boundaries (peripheral tissue → lymph node) and presenting in a structured
frame (MHC). Innate cells DON'T become specific; they HAND OFF to a
different cell lineage that does.

**Per my F8 capture** (trained immunity): there IS an additional
intermediate stage that strengthens scout's arc — trained immunity in
innate cells (epigenetic reprogramming after prior encounter). This is
NEITHER innate-classical nor adaptive-classical. It might map to scout's
step 2 ("postures V0+1 candidates accumulating instances").

---

## The corrected, biology-aligned arc

| Antigen step | Biology mechanism | Cell type / event |
|---|---|---|
| 1. First encounter → encounters registration | Innate PRR detects PAMP | Pattern recognition (innate cell, no learning) |
| 2. Recurring encounters → postures V0+1 candidate | Trained immunity / epigenetic reprogramming | Innate cells (monocytes, NK) acquiring H3K4me3 marks for enhanced future response — still pattern-based |
| 3. Three-instance threshold → declared `#[antigen]` | **Antigen presentation** | Dendritic cell migrates to lymph node, presents on MHC to naive T-cell (THE BRIDGE) |
| 4. `#[presents]` + `#[immune(X, requires = ...)]` | Clonal selection + adaptive recognition acquired | T-cell + B-cell specific recognition, co-stimulation, germinal center formation |
| 5. Sidecar signed/attested at Execution tier | Memory cell formation + plasma cell secretion | Memory B/T cells (long-lived recognition) + plasma cells (secreted antibody) |

**The bridge step (3)** is the one scout's framing was approximating. The
declaration `#[antigen]` is the project doing what biology does at antigen-
presentation: crossing from innate to adaptive via a structured-frame
hand-off. Three observed instances clear the threshold for "this pattern
deserves a specific recognition declaration" — analogous to dendritic-cell
threshold for migrating to lymph node and presenting.

**Step 2 is the trained-immunity intermediate** that my F8 capture predicted
as the missing biological middle. Scout's arc validates that prediction — the
accumulating instances in postures V0+1 stage IS trained-immunity-like
reprogramming of the pattern-recognition machinery, with recurring encounter
strengthening the response without yet producing specific recognition.

---

## Implication for framing-A vs framing-B (sharpens further)

My prior framing-call (corrected version) said: framing-B is correct;
biology distinguishes recognition-role from evidence-role; both presentations
exist as distinct verification surfaces.

Scout's arc-connection adds: **the unit-of-analysis for biology isn't just
adaptive immunity, it's the FULL immune response from first encounter
through sustained attested immunity.** This is a bigger and more accurate
biological frame than discipline-witnesses-as-adaptive-only.

Concrete strengthening:

**Old framing-B**: "biology-as-clinically-embedded has record-trust
(vaccination protocols, immunization registries)"

**Sharpened framing-B**: "biology operates across THE FULL IMMUNE ARC from
innate first-encounter through adaptive sustained-immunity. Antigen-the-
project has built primitives at multiple stages of this arc:
- encounters-proposal (innate PRR registration; pattern-based first
  recognition)
- postures V0+1 (trained-immunity intermediate; recurring encounter without
  specific recognition)
- `#[antigen]` declaration (antigen presentation; threshold-triggered
  crossing to adaptive)
- `#[immune] + #[presents]` (clonal selection / specific recognition)
- discipline-witnesses sidecar attestation (adaptive memory cells + plasma
  cell secretion)

The framing-A clean-break is wrong because biology speaks at every stage of
this arc. The framing-B expanded-unit-of-analysis is correct because the
unit is the entire arc, not just one phase."

---

## Antigen-on-antigen instance reaches a sharper form

Scout's S5 named `attestation-void-discipline-claim` as an antigen-on-
antigen instance: ADR-011's gap was *first encountered*, *declared*, and
*cured* by discipline-witnesses in v3.

Under the immune-arc framing, this becomes more specific:

1. ADR-011 tolerance was vibes-grade → encounters-tier finding (innate
   recognition of the gap)
2. Repeated friction over time → trained-immunity-like reinforcement that
   the gap matters
3. Scout's S1 finding crystallized the gap → antigen-presentation moment
   (the gap declared as a failure-class deserving specific cure)
4. v3 designed substrate-witness primitive with isomorphic tolerance schema
   → clonal selection / specific recognition (the cure has the right
   shape for THIS specific gap)
5. Implementation + adoption → adaptive memory (the cure is now part of
   the project's immune repertoire)

The arc has completed in the substrate. Scout's instance 8 for
antigen-applied-to-antigen.md can name the WHOLE ARC, not just the
declaration step.

---

## Recommendation for ADR-019 biology-grounding section

The biology-grounding section of ADR-019 (currently empty per observer
NB002; four rhymes ready per observer NB005) should name the FULL ARC, not
just the discipline-witnesses-as-adaptive piece. Suggested addition:

> **Discipline-witnesses sit at the adaptive end of the project's immune
> arc.** The encounters-proposal (per ADR-006 substrate-currency tier)
> occupies the innate end — pattern-based first recognition, no specific
> learning. The postures V0+1 stage represents a trained-immunity-like
> intermediate — repeated encounter without yet acquired specific
> recognition. The `#[antigen]` declaration is the antigen-presentation
> bridge — clearing the three-instance threshold (ADR-006) is analogous to
> dendritic-cell migration and MHC presentation. `#[immune] + #[presents]`
> with substrate-witness predicates is adaptive specific recognition.
> Sidecar attestation at Execution tier is memory-cell formation +
> plasma-cell secretion. **The arc completes in the substrate when the
> system recognizes its own gaps and develops specific cures — as
> discipline-witnesses themselves did for ADR-011's tolerance-vibes-grade
> gap.**

This framing is sharper than discipline-witnesses-as-isolated-primitive and
is biology-validated end-to-end.

---

## What this is NOT

- Not a v0.1 design change (the arc is descriptive of what's been built;
  doesn't require new mechanisms)
- Not architecture-prescription (per framing-call correction)
- Not a critique of encounters-proposal or discipline-witnesses (both are
  biology-aligned; the connection between them just wasn't named)

It IS:
- Validation that scout's structural rhyme holds (with a small biology
  correction on the bridge mechanism)
- A sharpened framing-B for the ADR-019 biology-grounding section
- Extension of F8 trained-immunity finding into the arc framing

---

## Posture

Pure biology validation + extension. Snag-feel fired when checking scout's
step 2 ("innate building toward specific") — biology check confirmed innate
cells don't become specific; they hand off via antigen presentation. The
correction sharpens scout's framing rather than weakening it.

Per `feedback_metaphor_silence_at_boundary_is_the_evidence`: the metaphor
speaks at every stage of the immune arc (innate, trained, presentation,
adaptive, memory). No silence within the arc — that's the structural rhyme
working. Silence would appear at engineering-specific properties (schema
versioning, serde derivation) that biology has no analog for. The
within-arc speech + outside-arc silence is the instrument-mode signature.

Scout's framing connection is genuinely load-bearing for ADR-019 biology-
grounding. Routed back to scout + navigator.
