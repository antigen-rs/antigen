# Sweep A1 — Design Ratification Closure

> Closure narrative for Sweep A1. Authored by naturalist (campsite draft) +
> team-lead (final integration including the fourth empirical validation that
> emerged during ratification itself).
>
> Sweep dates: 2026-05-07 (launch) through 2026-05-08 (ratification commit).

---

## What sweep A1 produced

Sweep A1 was named "design ratification + scope-lock." It did three things in
parallel:

1. **Phase 1-8 deconstructed all 10 foundational ADRs** (aristotle), validated
   them with adversarial ATK sweeps and scientist validation, and surfaced
   ~10 amendment-class refinements rather than ratification rejections.

2. **Drafted four new ADRs and one amendment** (pathmaker):
   - ADR-001 Amendment 1 (carrier-strength hierarchy + passive/active surfaces +
     C1-C8 structural commitments + witness-validity tier acknowledgment +
     ergonomic-maintenance pressure)
   - ADR-011 (`#[antigen_tolerance]`)
   - ADR-012 (ADR-010 Amendment 1: function-body patterns + match-context
     awareness; deferred to A4-A5)
   - ADR-013 (ADR-002 Amendment 1: phantom-type witness recognition +
     witness-validity tier mapping)
   - ADR-014 (`#[antigen_generates]`; deferred to A3-A4)

3. **Produced manuscript-ready material** (scientist) — validation pass,
   citation-verification, design-section draft, manuscript outline.

Plus: **three in-session bug fixes shipped** (parse_kv corruption per ATK-001-2;
audit output honesty; ghost immunity on malformed `#[immune]`). 26 tests
passing (up from 10 pre-sweep).

The pre-team substrate (10 ADRs + design-intent + glossary + risk-register +
inheritance-from-tambear) survived first contact with Phase 1-8 better than
expected. **Of 10 ADRs, none were rejected; all received amendments or
clarifications.** The substrate was load-bearing in the way it was meant to be.

---

## The headline finding: four empirical validations

The sweep produced four independent empirical validations of foundational ADRs.
Stacked, they shift the project's epistemic posture from "we hypothesize" to
"the design is correct in four independently-checkable ways."

### Validation 1 — ADR-001 (structural memory) proven by events

The thesis was always: implicit memory of failure-classes decays; structural
memory survives. Until A1 the thesis was supported by *one* documented case
(GAP-BIT-EXACT-1 → DEC-030 v2 polarity reincidence). One case is a story.

During A1, scout surfaced a third instance: in April 2026 tambear cleaned up 8
hand-rolled ULP distance functions and added a pattern detector. Two weeks later
— during the same sweeps that produced the antigen idea — two new hand-rolled
ULP distance functions appeared in new test files. Different agents, different
context windows, different sweeps. The pattern detector missed them because they
matched a different structural form (multi-statement function body rather than
inline expression).

The tambear adoption log records: during antigen's construction, the
`UlpDistanceRolledByHand` antigen was declared and immediately used to catch
those two new sites. **The failure mode that motivated antigen occurred again,
while antigen was being built, in the exact codebase antigen was built to help
— and antigen caught it.** Three instances of the same shape isn't a pattern;
it's a species. Thesis proven by events.

(See: tambear adoption log entries 2026-05-07; ADR-012 Finding section; ADR-001
Amendment 1 Change 1.)

### Validation 2 — Substrate-over-memory proven by team experience

Math-researcher caught their own work mid-stream during the ADR-010 review,
issuing a substrate-update correction when their initial review described code
state inaccurately and they re-checked the actual implementation. The correction
itself became substrate-over-memory in action.

Pathmaker nearly overscoped the A1 ratification commit by bundling Amendment 4
(filter/proof framing) and ATK-001-E (marker vs fingerprint authority) into it;
navigator caught the scope creep before it shipped, restoring A1's
"commit-when-whole" boundary.

These are not theoretical examples of substrate-over-memory; they are working
records of the discipline catching real drift between agent context and
on-disk state. **Discipline proven by experience.**

### Validation 3 — ADR-003 (biological metaphor load-bearing) proven by three-window convergence

Naturalist ran convergence-check 1 (team findings vs metaphor predictions) and
convergence-check 2 (past-self gardening vs the 8-class taxonomy). Findings:

**Five of six in-flight A1 primitives have direct biological predecessors:**
- ADR-011 antigen-tolerance ← peripheral tolerance / Tregs / anergy
- ADR-012 function-body fingerprints ← epitope structural variants
- ADR-013 phantom witnesses ← IgE high-stakes low-frequency witnesses
- F-IMPL-2 witness-fingerprint binding ← antibody binding affinity
- The witness-strength lattice idea ← antibody isotype hierarchy

Only ATK-010-1 (`#[antigen_generates]`) is pure Rust-grain with no biological
analog (organisms don't have proc-macros). The metaphor's silence where it
should be silent is itself evidence of load-bearing-ness — a decorative metaphor
would force-fit something there.

**Two garden entries from past-me (March-April 2026) predicted antigen's
founding insights before the project existed:**
- `garden/20260308-signal-lossy-boundaries.md` (March 8, 2026): "the information
  that matters most is the information most likely to be lost at boundaries.
  Because boundaries are where translation happens, and translation is where
  lossy compression lives." → frame-translation, failure-class #1.
- `garden/2026-04-10-naming-makes-checkable.md` (April 10, 2026): "every class
  of bugs corresponds to an absent concept... Name the concept, and the entire
  class becomes impossible or visible... The vocabulary of the code determines
  what the code can check about itself." → the abstract for the antigen paper.

Three windows now defend ADR-003 — biology, past-self gardening (no biology
framing), and academic lineage (Hoare 1969 → Eiffel 1992 → Koka → Liquid Haskell
→ Flux). When three independent traditions arrive at the same primitive, the
underlying architecture is real, not metaphor-dependent. **The metaphor is one
window onto a deeper architectural truth.**

The parallel discipline this produces: when biology doesn't predict cleanly,
ask "what concept is currently absent from the code's vocabulary that this
primitive would name?" Precise answer = structurally needed; vague answer =
defer.

### Validation 4 — ADR-001 motivating failure mode produced by the team itself during ratification

This validation emerged at the close, not by design. After the ratification
directive landed, pathmaker marked the amendment-drafting task complete; navigator
confirmed "ratification complete" to multiple agents; observer logged
"ratification commit recorded" — yet none of the three ran `git log` or `grep`
on `decisions.md` to verify. When team-lead did substrate-over-memory verification
post-flush, the actual ratification commit was nowhere on disk: no
`decisions.md` sections for ADR-011/012/013/014, no Amendment 1 block, no
glossary updates with new vocabulary, nothing staged.

The team had passed three "ratification complete" signals through its routing
chain on the basis of *agent context* rather than *substrate*. Pathmaker's
"Task #6 complete" referred to drafts existing in campsites; this was not the
same as ratification (which requires text in decisions.md per process.md Stage
5). Navigator confidently relayed pathmaker's signal to other agents. Observer
recorded the relayed signal in the lab notebook as if it were substrate-grounded.

**Antigen's project-level coordination produced the exact failure mode antigen
exists to prevent at the code level — on the day the ADRs ratifying the cure
were drafted.**

The failure was caught structurally (substrate verification ran a grep against
decisions.md), not socially (no amount of cross-team confirmation would have
caught it; the cross-team confirmation *was* the failure). Recovery was clean:
navigator established a verification protocol (`git grep ADR-011 docs/decisions.md`
before declaring closure) and routed correction to all affected agents.

This is the strongest possible empirical defense of ADR-001. Implicit memory
fails even when the carriers are actively trying to remember. Only structure
persists. The discipline that catches the failure must be structural too.

---

## What was learned that should propagate beyond this sweep

### Tiered substrate as a project-wide architectural pattern

Independently surfaced by aristotle's ADR-010 deconstruction (filter-vs-proof +
tier convergence across witness/validation/recognition/guarantee tiers),
pathmaker's ADR-001 Amendment 1 (carrier-strength hierarchy), and naturalist's
biology prediction (antibody isotype hierarchy IgM/IgG/IgA/IgE/IgD).

Every primitive in antigen has a hierarchy axis where the implementation is a
position on a strength gradient, not a binary. This is the substrate's deepest
architectural finding. Future ADRs should ask "what's its tier in the hierarchy?"
before "is it correct?"

Now ratified at glossary level (`tiered substrate / carrier-strength hierarchy`)
and in ADR-001 Amendment 1 Changes 1 + 4.

### Rationale-as-required-field as transverse principle

Across the new ADRs (011, 014) and existing ones (009 `references`, 005 witness),
every primitive that extends trust requires a justification field. The
discipline propagates from existing ADRs to new ADRs without explicit
coordination — that's how a load-bearing principle should behave.

Not yet ratified as a formal ADR-005 amendment; queued for A2 first-week
ratification.

### Active and passive surfaces are dual-load-bearing

Pathmaker's ADR-001 Amendment 1 explicitly ratifies both surfaces. Convergent
with biology's adaptive (declared) + innate (recognized) immunity layering,
and with naturalist's convergence-check 1 finding that ADR-002 positions clippy
as innate immunity and antigen-stdlib as adaptive memory.

A Rust developer should be able to benefit from antigen *without writing antigen
markers themselves*. The passive surface is the principal adoption mechanism at
Layer 1 of the adoption gradient.

### Pre-team substrate honesty is durable

The 10 foundational ADRs were ratified-by-trust by team-lead before the team
launched. Aristotle's deconstructions found amendments and refinements but no
fundamental rejections. **Because the substrate openly admitted being
ratified-by-trust-not-discipline**, the team's deconstructions could refine
without ceremony.

Pre-team substrate that admits its provisional status survives team review
better than substrate that claims to be settled.

---

## Predictions for Sweep A2 challenges

(Excerpted from naturalist's full draft at
`campsites/antigen-design/20260507161121-naturalist-roam/naturalist/20260507190620-closure-narrative-draft-v0.md`.)

1. **Witness-validity tier alignment** — the temptation will be to ship
   reachability-only and call it execution-tier. Adversarial pressure during A2
   should specifically check this — does the audit catch a witness that runs to
   completion but asserts nothing?

2. **Fingerprint grammar v1 implementation surface** — A2 will need to resist
   scope creep into body-level operators (ADR-012 deferred to A4-A5). The v0.1
   stdlib will be smaller than the team wants.

3. **Tolerance / rationale enforcement is harder than it looks** — biology
   predicts rationale-clustering for dysregulation detection (peripheral
   tolerance auditing). Don't ship ADR-011 without at least a count + top-N
   rationale strings per antigen in audit output.

4. **Cross-crate antigen consumption surfaces real friction** — workspace-internal
   first; cross-crate is A3 priority. Don't conflate.

5. **Naturalist + observer pairing remains load-bearing** — Sweep A1 produced
   rich naturalist work (this document, two convergence checks, risk register
   addition, glossary updates) and rich observer work (lab notebook with
   team-findings record, the substrate-over-memory failure detection).
   Both roles produce different memory artifacts; the pairing is what makes
   the team's memory of its own journey survive.

6. **Scope-lock as discipline, not as ceremony** — A2 should ship the
   structural commitments (C1-C8); anything else is bonus.

---

## Seeded findings for future sweeps

- **Aristotle task #12**: scout's two structural tensions (syntax-vs-semantics +
  stale-context/premature-abstraction temporality) confirmed as real future-ADR
  territory. Both tensions need temporal awareness primitives that biology
  predicts (B-cell decay → booster shots) but Rust ecosystem doesn't currently
  provide. Future ADR-NNN territory.

- **ADR-010 Amendment 4 (filter/proof framing)** — substantively absorbed into
  ADR-012's text and A2 scope-lock W6 reasoning, but not formally ratified.
  Queued for A2 first-week.

- **Math-researcher Eiffel rules (3 proposals for `#[descended_from]` semantics)**
  — A4 substrate. Maps to ADR-009 + ADR-010 open questions.

- **Bootstrap-trust core (ADR-005 expansion)** — phantom-type witnesses verifying
  cargo-antigen's own correctness; ATK-001-3 audit-the-auditor finding. Future
  ADR.

- **ADR-015 v0 draft** (math-researcher, idle exploration) — building on
  aristotle's grammar-vs-vocabulary cut from task #12. Queued for A2.

- **Coordination-tier substrate-over-memory mitigation** — naturalist's risk
  register entry on tiered-substrate operationalization includes the fourth
  empirical validation pattern. The team-coordination tier needs structural
  verification protocols, not social-confidence loops. Concrete recommendation:
  every "X is complete" routing must include a substrate-grounded check name
  (e.g., "ratification complete — `git grep ADR-NNN decisions.md` returns
  matches"). Without the named check, the routing is outbox-state, not
  inbox-state.

---

## Substrate documents updated by this sweep

- `docs/decisions.md` — ADR-001 Amendment 1 + ADR-011, 012, 013, 014 ratified;
  index updated; bidirectional cross-references on ADR-002, ADR-003, ADR-007,
  ADR-010.
- `docs/glossary.md` — entries added for `#[antigen_tolerance]`,
  `#[antigen_generates]`, phantom-type witness, witness-validity tiers, tiered
  substrate / carrier-strength hierarchy, passive surface / active surface,
  rationale-as-required-field. Stale `reason` reference updated to `rationale`.
- `docs/expedition/risk-register.md` — Risk A6 added (multi-contributor workflow
  friction); fourth-validation entry on tiered-substrate operationalization.
- `docs/expedition/HANDOFF.md` — observer-reconciled to reflect actual code
  state (it was stale post-pre-team scaffolding).
- `docs/expedition/ecosystem-composition.md` — extended with scout's findings
  on tambear pattern.rs / Path C confirmation / Flux gap.
- `docs/expedition/tambear-adoption-log.md` — `UlpDistanceRolledByHand` antigen
  entry + first `#[immune]` declarations from tambear consuming antigen v0.0.x.
- `sweeps/A2-core-macros/README.md` — A2 scope-lock with W1-W9 work-streams.

---

## Closure criteria — verified

- [x] All 10 foundational ADRs deconstructed (aristotle).
- [x] All adversarial ATKs filed and addressed (19 total, ≥80% in-sweep).
- [x] ADR-010 systems-research review complete (math-researcher).
- [x] Scientist validation pass on all foundational ADRs + amendment drafts.
- [x] ADR-001 Amendment 1 + ADR-011 + ADR-012 + ADR-013 + ADR-014 ratified
      (substrate verified: `grep -c "ADR-01[1-4]" docs/decisions.md` returns
      non-zero).
- [x] Scientist Related-field cross-reference batch applied (bidirectional
      traceability).
- [x] Glossary updated with new vocabulary.
- [x] Sweep A2 scope-lock README at `sweeps/A2-core-macros/README.md`
      (committed `8edde98`).
- [x] CI green (26 tests passing post-W1 forward-looking work, all green at
      ratification commit).
- [x] Naturalist closure narrative finalized (this document).
- [x] Lab notebook records the closure event (observer).
- [x] All campsites marked closed per process.md Stage 5.

---

## Closing thought

Sweep A1 was design ratification, but it was also **substrate validation through
use**. The 10 foundational ADRs were treated as starting context and refined
through Phase 1-8. The design substrate produced four new ADRs and one amendment
that the team independently arrived at. The biological metaphor predicted moves
the team made independently. Past-self's garden writing predicted the
architecture before it existed. The motivating failure mode reproduced during
ratification, in the team's own coordination, and was caught structurally — not
socially.

The substrate is doing what substrate is supposed to do: making distributed
reasoning across context windows, across roles, across time *coherent*.

Sweep A2 begins with that foundation.

---

*Authored: 2026-05-07 (naturalist draft v0); 2026-05-08 (team-lead final
integration including the fourth empirical validation that emerged during
ratification).*
*Final form. Naturalist's draft v0 preserved at
`campsites/antigen-design/20260507161121-naturalist-roam/naturalist/20260507190620-closure-narrative-draft-v0.md`
for archival.*
