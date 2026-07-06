# DRAFT — ADR-071: Organ-Hood, and the Control-Loop as a Diagnostic Taxonomy

**Status:** DRAFT (for the ratification ceremony). New ADR (next free number is 071; confirm at ratify-time).
Derived from notebook [017](017-the-0.7-organ-council-boyd-ruling.md) — the 8-lens council + Boyd ruling.
Relates to ADR-037 (the six-stage control loop) and ADR-067 (the stroma).

---

## Context

The 0.7.x organ layer (L2 — capabilities that sense on the stroma) was chartered as "5 organs + capstone." Two
questions had to be settled before building: how is an organ *individuated* (when does one split into
sub-organs, when is a candidate an organ at all?), and does the layer decompose along ADR-037's six control-loop
stages (SENSE→COMPARE→ROUTE→ACT→FEEDBACK→SIZE)?

A deconstruct-council found a seductive trap: six of eight lenses mapped the organs onto the loop, found ROUTE
and FEEDBACK "empty," and recommended re-cutting the layer into six stage-organs. That commits the exact
category-error `decisions.md:9808` warns against — ROUTE and FEEDBACK "have no user-code disturbance-genus (they
are antigen-response-only stages)."

## Decision

**1 · The organ-hood law.** A 0.7.x organ is a capability with (a) a distinct BUILD, (b) a distinct
BENEFICIARY-ABSORB, and (c) a live STROMA-SIGNAL *or* a distinct AUTHORIZATION-GATE in the same horizon. A
candidate SPLITS into sub-organs if it fails any of: **authorization-gate** (different validation gates → split
into tiers), **stroma-signal** (different query/closure-tier → split by build-order), **disjoint-absorb** (disjoint
consumer-sets → split), **belongs-iff** (needs an "A or B" disjunction → the *or* is the split-line).

**2 · The control-loop is a DIAGNOSTIC TAXONOMY, not the build-WBS.** ADR-037's six stages are a failure-class
sort-key for antigen's *own machinery* — where an organ *lives*, not what *individuates* it. An organ occupies
a loop-stage the way a point occupies a coordinate: two capabilities may share a stage (the three effector
authorization-tiers), and one capability may be denied organ-hood despite owning a stage (ROUTE, this horizon,
is *correctly thin* — not missing). **The layer is not re-cut along the loop.**

**3 · The loop IS a legitimate control-plane build-track — non-organ.** Distinct from (2): the six stages, made
concrete, are antigen's own **self-regulation** — antigen instrumenting its own control loop to catch when *its*
machinery misfires (a ROUTE failure, a FEEDBACK failure *in antigen*). These are build-items, not organs (they
regulate antigen; they do not sense user code). Per stage: SENSE/COMPARE self-integrity, ROUTE self-attention
(the dread-escalation hook), ACT self-provenance (marker-sovereignty), FEEDBACK self-legibility
(SCRAM/emit-bus), SIZE self-adaptation. The control-plane track and the organ track *share substrate* at ROUTE,
ACT, and FEEDBACK (build once, both consume); the rest are candidate build-outs, not organ prerequisites.

**4 · The layer stack.** L0 stroma (substrate) → L1 frame (intent-kernel + coordinate system) → L2 organs
(sense user code) ∥ control-plane (regulate antigen) → capstone (a JOIN / VALIDATE-event, builds nothing).

**5 · Known-completeness.** Any capability that fails the horizon gates is `on_hold`, **but named in the
loop-map** — the loop is *known*-complete, never silently truncated.

## Consequences

- The "5 organs + capstone" resolves to ~14 nodes: the effector splits into three authorization-tiers,
  FEEDBACK splits substrate/logic, several charter-concepts resolve to sub-slices or stroma-primitives rather
  than peer organs, and the capstone is confirmed a JOIN. (Full map: notebook 017.)
- Build-order is gated by one unbuilt node — the stroma closure (`antigen-stroma/src/read/query.rs`, `todo!()`).
- `antigen::learn` stays shipped-not-re-planned; the "germinal-center" residue is the reputation-update stroma
  primitive, homed in the SENSE organ.

## Relationship to existing ADRs

- **ADR-037** (the six-stage loop): unchanged as a *taxonomy*; this ADR forbids using it as a build-WBS and
  *adds* its use as a control-plane build-track.
- **ADR-067** (the stroma) + its Capability Expansion Law amendment (draft 018): organs and control-plane items
  are both capabilities subject to build→absorb→validate.

## Open questions for the ceremony

- Is "authorization-gate" a sufficient primary split-axis, or does it need pairing with the stroma-signal axis
  to avoid over-splitting cheap capabilities?
- Should the control-plane track (decision 3) be its own ADR, or a section here?
