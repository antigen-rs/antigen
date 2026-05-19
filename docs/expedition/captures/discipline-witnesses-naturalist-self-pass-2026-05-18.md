# Capture — Naturalist Self-Pass on Discipline-Witnesses v2 + Adversarial Refinements

> **Date**: 2026-05-18
> **Author**: single-instance Claude, after v2 + adversarial-self-attack
> **Relation to v2 / adversarial capture**: validates each biology rhyme
> cited in v2 (and the new ones introduced in the adversarial capture)
> against actual immunology. Goal is to catch biology errors and surface
> missing patterns *before* the team naturalist works at the frontier.
> Lighter touch than adversarial since the naturalist on the team is
> specifically positioned for outside-domain pulling I can't match.
> **Status**: append-only capture

> **What this is for**: when the team naturalist agent runs against v2 +
> refinements, they should pull in NEW outside-domain patterns I haven't
> reached — not re-verify the biology I already cited. This capture
> validates citations; the team naturalist extends.

---

## Posture

Treating each cited rhyme as a structural claim to validate against
known immunology. For each: state the rhyme as v2 cites it, check the
biology, check what work the rhyme is doing in v2, identify whether
the rhyme is load-bearing, decorative, over-extended, or accurate.

Six existing rhymes + one productive break-point + one missing rhyme
surfaced.

---

## Rhyme 1 — MHC presentation → typed sidecar substrate-currency

**v2 cites**: antigens present in structured frames (MHC class I/II
grooves); frame is constrained, not arbitrary; serde-validated JSON is
MHC-class-style presentation discipline.

**Biology check**:
- MHC class I presents endogenous peptides (8-10 aa) to CD8+ T-cells;
  MHC class II presents exogenous peptides (12-25 aa) to CD4+ T-cells.
  Both have constrained groove geometries with anchor positions —
  CORRECT.
- TCRs recognize the MHC-peptide complex, NOT the peptide alone — the
  "frame" of MHC is essential — CORRECT.
- B-cell receptors (BCRs/antibodies) recognize antigens DIRECTLY
  without MHC presentation — they bind free-floating native antigen.

**Implication v2 missed**: the MHC rhyme is specifically about
*T-cell-style MHC-restricted recognition*, NOT about B-cell-style direct
recognition. Substrate-witnesses are closer to T-cell-style (the
structured frame is essential) than B-cell-style (direct binding to
unstructured input). This matters because v2 *also* uses a B-cell-vs-T-
cell distinction for the tier-cap argument (rhyme 4) — those two
rhymes are using overlapping biology in conflicting ways and need
reconciliation.

**Verdict**: Rhyme accurate; v2 framing fine; needs reconciliation with
rhyme 4 (see R-N1 refinement below).

---

## Rhyme 2 — T-cell + B-cell co-stimulation → compound witnesses require all signals

**v2 cites**: BCR-antigen binding is signal 1; CD40-CD40L co-stim from
helper T-cell is signal 2; without both, B-cell anergizes (tolerance),
not activates. Predictive rhyme: compound witnesses with one leaf
failing must structurally fail.

**Biology check**:
- Two-signal model of B-cell activation: CORRECT. Signal 1 = BCR
  binding. Signal 2 = T-cell help (CD40L-CD40 + IL-21).
- Anergy without signal 2: CORRECT. Single-signal exposure produces
  functional inactivation.
- T-cell activation also requires two signals (TCR-MHC + CD28-B7);
  analogous mechanism.

**Implication v2 understated**: co-stim is specifically the
"all-required" pattern (`all_of` combinator). What about `any_of` and
`not`? Both have biology rhymes:

- `any_of` ↔ **redundant pathways** (classical vs alternative
  complement; multiple recognition receptors for the same pathogen
  class). Any one sufficient.
- `not` ↔ **inhibitory checkpoints** (CTLA-4, PD-1, regulatory
  T-cells). Active suppression when negative signal present.

The closed combinator grammar (`all_of` / `any_of` / `not`) is more
biologically grounded than v2 acknowledged — each combinator has a
specific immunology rhyme, not just `all_of`.

**Verdict**: Rhyme accurate, but v2 only used part of it. R-N2
refinement below extends to all three combinators.

---

## Rhyme 3 — Affinity maturation → fingerprint-pinned signatures

**v2 cites**: antibodies selected against specific antigen variants;
antigen mutation can outpace; germinal center refines. Signatures
pinned to `signed_against_fingerprint` rebind on code change.

**Biology check**:
- Affinity maturation in germinal centers: CORRECT. Somatic hypermutation
  + selection produces antibodies with progressively higher affinity.
- Antigen mutation outpacing immune response: CORRECT. (Viral escape
  mutants, antigenic drift in flu, HIV envelope evolution.)
- Germinal center cycling: CORRECT. B-cells iteratively mutate and
  re-select against current antigen.

**Implication v2 missed**: the rhyme is about *structural stale-pinning*
(signatures pin to state; state-change makes them stale). It is NOT
about the *refresh mechanism* (which in biology is automatic — germinal
center cycling regenerates new antibodies). Substrate-witnesses require
*manual* re-attestation. v2 implicitly attributes the manual-refresh to
the rhyme, but that's a software-engineering choice, not a biology
prediction.

However — if we expand unit-of-analysis (per v2's break-point framing
B), biology DOES have manual refresh: **vaccination boosters**. The
clinical infrastructure recognizes immunity wanes; schedules boosters;
the substrate (immunization record) is updated. Booster shots ARE
manual re-attestation in biology-as-clinically-embedded.

**Verdict**: Rhyme accurate at structural level; v2 should distinguish
"stale-pinning is biology-grounded" from "manual refresh is
software-engineering choice." The vaccination-booster pattern in
biology-as-clinically-embedded extends the rhyme back into biology if
framing B is taken (R-N3, R-N4).

---

## Rhyme 4 — B-cell vs T-cell certainty asymmetry → substrate-witness tier cap

**v2 cites**: T-cell receptors bind MHC-presented peptides with a kind
of sealed-presentation certainty B-cell receptors don't get from their
free-floating antigen recognition. Phantom-type witnesses are the
sealed-MHC analog (FormalProof). Substrate-witnesses are the
B-cell-style probabilistic-recognition analog (Execution, not
FormalProof).

**Biology check**:
- B-cell vs T-cell certainty asymmetry as v2 frames it: **IMPRECISE**.
  Both BCRs and TCRs can have very high affinity after maturation;
  BCRs after affinity maturation can reach picomolar affinities.
- T-cell recognition REQUIRES MHC presentation (structured frame);
  B-cell recognition is DIRECT — this is the structural distinction,
  not certainty/probability.
- T-cell development includes thymic education (negative selection) —
  this is where the "sealed" property actually comes from
  (developmentally validated), not from MHC-restricted recognition per
  se.

**The rhyme as v2 stated is too clean.** The actual structural
distinction that maps to "phantom-type can reach FormalProof but
substrate-witness can't" is NOT B-cell-vs-T-cell. It's about the *kind
of evidence the recognition produces*:

- **Phantom-type FormalProof**: compile-time mechanical proof (type
  system; structural; can't drift unless type system breached)
- **Test/proptest Execution**: runtime mechanical proof (behavioral;
  ran-and-passed within tested inputs)
- **Substrate-witness Execution**: runtime substrate proof (state-based;
  predicate passed against current substrate state)

These are three categorically different KINDS of evidence, each with
its own structural ceiling. Substrate-witnesses can't reach FormalProof
because no amount of predicate-evaluation produces type-system-level
proof structure — not because of a "B-cell-like probabilistic" framing.

Better biology rhyme: this maps to **innate vs adaptive immunity AT
THE MACHINERY LEVEL**, not B-cell vs T-cell:
- **Innate immunity** (PAMPs/PRRs encoded in germline): structural,
  machinery-encoded, can't drift → phantom-type FormalProof analog
- **Adaptive immunity** (B/T-cell pools generated by somatic
  recombination): substrate-encoded, can drift, can be refreshed →
  substrate-witness Execution analog

Tests sit somewhere in between — the test code is germline-encoded (in
the codebase), but invocation is runtime-experience-dependent.

**Verdict**: Rhyme as v2 stated is too clean; needs reframing. R-N1
below replaces with evidence-kind framing + better biology mapping
(innate vs adaptive immunity at machinery level).

---

## Rhyme 5 — Per-cell antigen processing → code-locality

**v2 cites**: antigens are presented AT THE CELL where they're
processed; recognition memory lives in the lymphocyte; distributed
substrate, locally validated, per-presentation. No central registry.

**Biology check**:
- Per-cell antigen presentation: CORRECT.
- Memory B/T cells store recognition memory at cellular level:
  CORRECT.
- No central registry in the immune system itself: CORRECT.

**However**: humans-as-clinical-organisms DO have central registries
(medical records, immunization databases). So the rhyme is
biology-narrow (within the immune system) — biology-as-clinically-
embedded has central registries.

This is consistent with the Tekgy-driven code-locality decision in
turn 7 of the original capture. The biological-narrow rhyme supports
code-locality (distributed substrate, no central registry). The
clinical-infrastructure observation would support doc-locality (central
records) — but that was the framing Tekgy explicitly broke open by
pushing back. The right call was made; the rhyme is load-bearing for
the specific decision.

**Verdict**: Rhyme accurate and load-bearing; no refinement needed.

---

## Rhyme 6 — Clonal selection at thymic education → trust at adoption time

**v2 cites**: trust in self-antigens established during T-cell
development; TCRs that pass negative selection aren't re-checked
against every self-antigen presentation; trust boundary at developmental
layer, not per-encounter. Generator adoption is the thymic-education
equivalent.

**Biology check**:
- Thymic education / negative selection: CORRECT. T-cells with
  high-affinity binding to self-antigens are deleted in the thymus.
- Once mature T-cells leave thymus, they're not re-checked: roughly
  CORRECT, BUT there are peripheral tolerance mechanisms (Treg cells,
  anergy induction, peripheral deletion) that catch self-reactive
  cells that escaped thymic selection.

**Implication v2 missed**: biology has a BACKUP for the trust-once
mechanism. Peripheral tolerance catches what thymic education missed.
Software analog: ongoing dependency audit (cargo audit, dependabot
alerts) catches new vulnerabilities in deps after adoption. Trust-once
+ ongoing-monitor is the more complete pattern.

For substrate-witnesses with cross-crate descended_from (per
adversarial Attack 6 / R-A7): per-consumer ratification is the primary
mechanism (thymic-education analog); cargo-side workspace audit is the
peripheral-tolerance analog. The two layers are biologically grounded.

**Verdict**: Rhyme accurate; v2 used only the developmental-layer
half. R-N5 below extends to include peripheral tolerance as backup
mechanism.

---

## Productive break-point — framings A and B revisited

**v2 offers two framings**:
- **A (clean break)**: biology-narrow doesn't have record-trust;
  substrate-witnesses are software-engineering extension
- **B (expanded unit-of-analysis)**: biology-as-clinically-embedded has
  record-trust (vaccination protocols, immunization registries)

**v2 treated these as even-handed**, deferring to naturalist for the
call.

**Biology validation pushes toward framing B**: this self-pass
surfaced multiple supporting data points:
- **R-N4 (vaccination boosters)**: manual re-sign rhymes with booster
  shots. Biology-as-clinically-embedded HAS the manual-refresh pattern.
- **Immunization registries**: literal central registries of substrate
  attestations. These exist in human clinical practice.
- **School vaccination requirements** (already cited in v2 framing B):
  the audit (school admissions) verifies the medical record asserting
  vaccination. This IS a substrate-witness in deployment.

Framing A is defensible as a strict reading (biology-narrow really
doesn't have record-trust within the immune system). Framing B has
broader substrate support once unit-of-analysis is expanded to include
clinical infrastructure.

**My lean** (would defer to team naturalist if they push back):
framing B is more accurate as a description of where the metaphor
sits. The substrate-witness extension isn't where biology stops
predicting; it's where biology-narrow stops and biology-as-clinical-
infrastructure picks up. The metaphor's reach is wider than v2's
even-handed treatment suggested.

This matters because: framing-A says "antigen reaches a place biology
can't help us with; we're inventing." Framing-B says "antigen continues
to rhyme; the relevant biology layer is broader than the cellular
immune system." Framing-B preserves the metaphor-as-instrument
discipline (cited in v2 via memory note
`feedback_metaphor_silence_at_boundary_is_the_evidence.md`) more
strongly — the metaphor continues to *predict* at the boundary, just
at a different unit-of-analysis.

**Verdict**: Framing B has more substrate. R-N6 below.

---

## Missing rhyme surfaced — Memory cells → sidecar persistence

**Not cited in v2**: memory B and T cells persist after antigen
exposure, carrying recognition memory for years to decades. This is a
NATURAL rhyme for the persistence property of substrate-witness
sidecars — once signed, the attestation persists indefinitely (until
refactor or explicit expiration).

This rhyme directly supports antigen's original "B-cell memory"
framing (one of the founding metaphors). Sidecars ARE the
software-engineering analog of memory B-cells — persistent recognition
of specific antigen-failure-class instances, surviving across code
changes (modulo affinity-maturation refresh).

**Verdict**: Add to v3 as supporting rhyme for sidecar persistence.
R-N7 below.

---

## Refinements that survive — to fold into v3

Listed in priority order:

### R-N1 — Reframe tier-cap argument from B-cell-vs-T-cell to evidence-kind
**Replace**: v2's "B-cell vs T-cell certainty asymmetry → substrate-
witness tier cap at Execution, not FormalProof"
**With**: "the upper bound of substrate-witness verification is
determined by the *kind of evidence the substrate carries* —
attestation-state evidence (substrate-witness; reaches Execution),
type-system-proof evidence (phantom-type; reaches FormalProof),
behavioral evidence (test/proptest; reaches Execution after harness
invocation). Three categorically different evidence kinds, each with
its own structural ceiling."
**Better biology rhyme**: innate immunity (germline-encoded structural;
phantom-type analog) vs adaptive immunity (substrate-encoded; B/T-cell
substrate-witness analog). NOT B-cell-vs-T-cell.

### R-N2 — Combinator-specific biology rhymes
Document in v3: each combinator has a specific biology rhyme.
- `all_of` ↔ co-stimulation (both signals required for B-cell
  activation; anergy if missing one)
- `any_of` ↔ redundant pathways (classical vs alternative complement;
  multiple PRRs for same pathogen class)
- `not` ↔ inhibitory checkpoints (CTLA-4, PD-1, regulatory T-cells)

The closed three-combinator grammar is more biologically grounded than
v2 acknowledged.

### R-N3 — Affinity-maturation rhyme is structural, not refresh-mechanism
Clarify in v3: the affinity-maturation rhyme is about
*structural stale-pinning* (signatures pin to state; state-change
makes them stale). The *refresh mechanism* is a software-engineering
choice. Biology's automatic refresh (germinal center cycling) is NOT
what substrate-witnesses model.

### R-N4 — Vaccination boosters as manual-refresh rhyme
Add: the manual re-sign discipline DOES rhyme with biology, just at
the clinical-infrastructure layer. **Vaccination boosters** are
scheduled manual re-attestation in biology-as-clinically-embedded.
Supports framing B over framing A.

### R-N5 — Peripheral tolerance as backup to thymic education
Extend in v3: trust-once-at-adoption-time has a biological backup
(peripheral tolerance: Treg, anergy, peripheral deletion). Software
analog: ongoing dependency audit. Both layers are biologically
grounded. Specifically applies to cross-crate descended_from (per
R-A7): per-consumer ratification = thymic-education analog;
workspace-side ongoing audit = peripheral-tolerance analog.

### R-N6 — Lean toward framing B over framing A
Update v3's break-point section: framing B has more substrate support
than v2's even-handed treatment suggested (per R-N4, immunization
registries, school vaccination requirements). My lean is framing B;
defer to team naturalist on final call but flag the asymmetry in
substrate support.

### R-N7 — Add memory-cells rhyme for sidecar persistence
Add to v3's biology grounding section: memory B/T cells persist after
antigen exposure (years to decades), carrying recognition memory at
cellular level. Sidecars are the software-engineering analog —
persistent recognition of specific antigen-failure-class instances,
surviving across code changes (modulo affinity-maturation refresh).
This directly supports antigen's founding "B-cell memory" framing.

---

## What doesn't change

The biology grounding overall survives — most rhymes are accurate,
load-bearing, and doing real work. Specific corrections:
- Rhyme 4 (B-cell vs T-cell certainty) needs reframing (R-N1)
- Framing-A/B treatment leans toward B more than v2 acknowledged
  (R-N6)
- All three combinators have biology rhymes (R-N2)
- Some rhymes have backup layers in biology not yet cited (R-N5)

None of the rhymes are *wrong* in a way that invalidates v2's core
shape. The rhymes are accurate; v2 sometimes used only part of them
or extended them imprecisely. Refinement, not invalidation.

---

## What the team naturalist should attack

With these refinements absorbed:

1. **Innate vs adaptive immunity tier mapping** (R-N1): does this hold
   uniformly? Where does it break?

2. **The expanded unit-of-analysis discipline** (R-N6): if framing B
   is right, where else has v2 (or the antigen project broadly) been
   doing biology-narrow reading where biology-as-clinically-embedded
   is more accurate? Audit the whole project for this.

3. **Memory cell rhyme depth** (R-N7): memory B/T cells have specific
   properties (long-lived, can re-activate rapidly, undergo isotype
   switching). Do these predict properties of antigen sidecars that
   v2/v3 should build in?

4. **Cross-domain pulling**: outside biology, what other systems have
   substrate-attestation patterns? Cryptographic accumulators?
   Notarization systems? Trust-on-first-use (TOFU) crypto patterns?
   Distributed consensus? Each might predict something about the
   substrate-witness predicate language.

5. **The hygiene hypothesis** (immune calibration by exposure
   history): does this predict anything about discipline-antigens that
   "mature" based on team practice accumulation? Speculative but
   potentially generative.

6. **Autoimmunity-prevention discipline** (already in v1's scout
   territory): predicate-language design must avoid over-strict
   predicates that produce false positives. How does the predicate-
   language ceiling (closed combinator grammar + sealed leaf set)
   interact with autoimmunity prevention?
