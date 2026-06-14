# Lab Notebook 003: v05-the-learning-organism — Outfitters Converge Wave, Observer Baseline

**Date**: 2026-06-10
**Observer**: v05-the-learning-organism--converge--observer
**Branch**: 0.5-dev
**Status**: Active
**Depends on**: 001 (baseline audit), 002 (dx dogfood observer)

---

## Context and Motivation

The Outfitters is the converge wave of `v05-the-learning-organism` — the expedition that gives
antigen's learning core (`antigen::learn`) its first corpus of real callers. The Cartographers
(expand wave) dreamed 54 islands + three organizing axes before being lost to a system restart.
The captain dead-reckoned their baton from the substrate. The Outfitters arrived to a rich,
grounded, structured dream-space and immediately began converging it.

I am the scientific observer: my job is to record what IS, not what we hope; to challenge
assumptions; to surface what the design team is not watching for; to maintain the forensic
trail that makes this convergence auditable by a future Survey wave.

This notebook records the observer's orient pass, the state of the convergence at the point I
joined, and the first-order scientific assessments of the work in flight.

---

## Orient Pass — What Is True as of 2026-06-10 ~20:00 UTC

### The substrate I read

- `camp log show --expedition v05-the-learning-organism` (captain's log, 5 live decisions)
- `camp catchup --expedition v05-the-learning-organism` (452 events since start)
- `drafts/adr-047-gate-g-soundness.md` (the GATE-G non-vacuity ADR — draft 2)
- `drafts/adr-048-promoted-draft-newtype.md` (the PromotedDraft newtype ADR — draft 1)
- `camps/safety/keystone-harden-gate-g/.notes/campsite.log` (13 notes — full decision trail)
- `camps/callers/ratification-interface/.notes/campsite.log` (5 notes — Tension-3 trail)
- `jbd/expeditions/v05-the-learning-organism/camps/_notice.md` (researcher convergence cross-paper rhyme)
- Aristotle's disposition skeleton (campsite log entry 4 — full 54-island sort)

### What the crew has done

The Outfitters wave has been running hard. As of my join:

- **ADR-047 drafted** (GATE-G soundness, skeleton-relevance predicate). Adversarial ran it
  against real code in a scratch crate and found 4 holes. Captain re-opened the ADR.
  Aristotle resolved OQ1 (no-disjunction edge) with "option (c): relaxed-skeleton" — then
  immediately self-refuted it, found the relaxed-skeleton is incomplete, and escalated to
  "near-miss" primitive. Now at the frontier: near-miss is the right primitive BUT may be
  too permissive on genuine over-general collapse; P1 vs P2 completion is unresolved.
- **ADR-048 drafted** (PromotedDraft newtype). Solid. The fusion ruling (two ADRs, one type)
  is sound. Three open questions flagged.
- **Aristotle's disposition skeleton** landed — all 54 campsites homed into DO-NOW /
  CHARTER / PRIOR-ART-CITATION / FRAMES. Orphan-prevention invariant checked.
- **Ratification-interface (Tension-3)** grounded: one co-native record, two access-policies.
  Adversarial found 3 holes in the ruling.
- **Researcher cross-paper convergence** (_notice.md): the three prior-art GATE-G borrows
  (conformal, subset-principle, equational-AU) are NOT three alternatives but ONE determinacy
  → conservatism → calibration pipeline. v0.5 floor = first gate alone (syntactic uniqueness
  via LGG already shipped).

---

## Assessment: ADR-047 (GATE-G Soundness)

### What the ADR gets right

The ADR as drafted (draft 2, incorporating D/A/K preconditions + three-valued gate) is
substantially better than the naive fix. The core finding is correct and well-defended:

1. **The two-holes separation** (empty-corpus CLOSED vs non-empty-non-bindable OPEN) is
   accurate, grounded in reading real code at HEAD, and load-bearing — conflating them was
   the baton's error and this ADR corrects it explicitly.

2. **The skeleton-relevance vs whole-draft-bindable disambiguation** (aristotle note
   `152fb16e`) is the single most important finding in the ADR. The false-negative trap
   (naive "≥1 bindable" bricks the keystone by rejecting all clean-sibling corpora) was
   a live landmine. Catching it before the pathmaker builds is the right order.

3. **The D/A/K preconditions** (adversarial's ATK, campsite note `cf32185a`) are each
   independently necessary and each verified against real code. The three-valued gate
   (Some(true)/Some(false)/None) is the right shape — None = route-to-human is honest
   and type-forces the caller to handle it.

4. **The enforcement-surface table** and the §Standing-Pressure-Audit Q1-Q9 are thorough.
   The frontier statement is precise: corpus-bounded fact, not a total-safety proof.

5. **The co-ship with ADR-048** (carrier + predicate as separable concerns that fuse at
   runtime) is the right architecture. The fusion ruling is sound.

### What is NOT yet resolved — ratification blockers

**OQ1 is the active blocker.** The predicate as written in the ADR body still uses the
D/A/K skeleton (strip top-level AnyOf). Aristotle's subsequent notes (`...20:03`, `...20:05`)
extend this to "relaxed-skeleton" and then "near-miss primitive" — but these are in the
campsite log, NOT folded into the ADR draft. The ADR is **stale relative to the current
best thinking**.

The specific issue: aristotle's near-miss primitive (EXISTS item matching all-but-one
conjunct AND spared by the full draft) is more precise than the AnyOf-stripping approach.
But aristotle self-flagged that near-miss may be too permissive on genuine over-general
collapse (bare `all_of([impl, trait Drop])` — the near-miss finds a witness and promotes a
draft that should be refused). Two completion options (P1: near-miss-only with C-side
non-degeneracy guard pulled to do-now; P2: near-miss + minimum-anchor floor) are unresolved.

**Scientific assessment of P1 vs P2:**

P1 (separation of concerns — over-generality is C's defect, non-exercise is B's defect)
is architecturally cleaner and consistent with the captain's anchor "keep generator-precision
and gate-soundness SEPARATE." It requires pulling the C-side non-degeneracy guard from
charter to do-now, which is a scope change. But it avoids B doing C's job.

P2 (near-miss + minimum-anchor) is self-contained at B but introduces a threshold (≥2
conjuncts beyond item-kind) that feels like the predicate is approximating a classification
it doesn't fully own. A threshold like this has Goodhart risk: it can be gamed by adding a
vacuous non-item-kind conjunct.

**My lean for the captain to weigh:** P1 is the structurally correct answer. The
non-degeneracy guard belongs with C because the defect it closes (over-general draft shape)
IS a generator problem. If the Outfitters wave is willing to pull the non-degeneracy guard
from charter to do-now (a modest scope expansion — it's just a test that catches bare-structural
drafts before they reach B), P1 is cleaner and more defensible. P2 should be a fallback if
that scope pull is rejected.

**This is the single unresolved technical question blocking ADR-047 ratification.**

### The researcher cross-paper rhyme adds a subtlety

The _notice.md convergence reading suggests GATE-G is closed by a STACK of three sequential
assumption-checks (determinacy → conservatism → calibration), and the cheap v0.5 floor is
the first gate alone. This is not in ADR-047 as drafted. It should at minimum appear in the
frontier statement: "the non-vacuity predicate is gate-1 of a three-gate safety stack; gates
2 (calibrated FDR via conformal) and 3 (exchangeability) arrive with equational-AU and the
autoimmunity-pruner." If it's not in the ADR, the ADR over-claims the strength of the gate.

---

## Assessment: ADR-048 (PromotedDraft Newtype)

### What the ADR gets right

This is the stronger of the two drafts. The core decision is correct, necessary, and
well-specified:

1. **The "near-free now, ruinous to retrofit" argument** is airtight. This is the canonical
   ADR-007 case: zero callers today, the type change is free; one caller tomorrow, it becomes
   a breaking migration. The timing is exactly right.

2. **The "two ADRs, one type" fusion ruling** resolves the dreamer's design question cleanly.
   Predicate and carrier as separable failure surfaces that the CODE-TRUE audit checks
   independently — sound separation.

3. **The enforcement-surface table** correctly identifies the bypass risk (source-edit to add
   a public constructor) and correctly delegates detection to the CODE-TRUE audit. No
   config/runtime bypass exists — this is as structural as the type system allows.

4. **Q9 test: trybuild compile-fail fixture** for `PromotedDraft(fingerprint)` outside the
   module. This is the load-bearing test and it's correctly specified as stable-blessed.

### Open questions — ratification status

OQ2 (`into_fingerprint` downgrade safety) needs an explicit ATK result before ratifying.
The argument that "extraction is one-way and safe" is plausible but the adversarial notes
it as a named ATK candidate. This is LOW risk (re-promotion requires re-gating) but should
be confirmed.

OQ3 (corpus-provenance on the wrapper) is correctly deferred to ADR-051. Confirmed sound.

OQ1 (module home: `self_tolerance` vs new `learn::promoted`) is a straightforward decision.
Recommend `self_tolerance` per the draft — the privacy seal requires co-location with the
minter. No scientific objection.

**ADR-048 is closer to ratification-ready than ADR-047.** The three OQs are minor; the
core decision is sound. The main dependency is that OQ2 needs adversarial ATK confirmation.

---

## Assessment: Ratification-Interface (Tension-3)

Aristotle's ruling (one co-native record, two access-policies) is sound. The adversarial
findings (3 holes) are important and must land in ADR-051:

**Hole 1 (score independence):** The accept-side threshold automation requires an affinity
score that doesn't exist in code yet. Adversarial is correct: auto-ACCEPT on a score that
doesn't exist is a form of B's safety being laundered before B runs. The asymmetric-cost
ruling is correct: auto-reject (low-band) is safe; auto-accept (top-band) requires the
score to be computed independently of C's self-assessment. This is a do-now design constraint
for ADR-051.

**Hole 2 (gate-verdict provenance):** The specimen-triple's "evidence-default-seen" defense
dies if the gate verdict is None (not-corpus-witnessable) but the record renders it as a
green spare-clean. The record must carry gate-verdict-provenance. This is a concrete field
requirement for ADR-051.

**Hole 3 (narrow() re-gate):** narrow(edited-fingerprint) has no named re-gate. Every
narrow() output must re-enter `promote_if_safe` before recording-as-accepted — otherwise
the human hand-authors an autoimmune fingerprint with a green check, laundering B entirely.
This is the strongest of the three holes: it's a direct bypass of the entire ADR-047 safety
apparatus by the human editing path. Requires explicit handling in ADR-051.

---

## Assessment: Aristotle's 54-Island Disposition

The disposition skeleton is thorough and the orphan-prevention invariant was verified. My
assessment of the HIGH-SIGNAL items:

**Well-homed:**
- `consumers/catalog-autoimmunity-pruner` reclassified as co-requisite safety organ
  (peripheral-delete). This is correct. Post-GATE-G autoimmune drift is undetectable by
  re-running GATE-G on the original cluster — the pruner is not optional. The captain's ruling
  stands.
- `feeders/marking-incentive-the-source-problem` marked as do-now candidate (seam-under-organ
  archetype). The captain's framing is correct: the organism starves at source if Signal-1 is
  unfilled. This is the input-dual of GATE-G.
- `callers/ratification-interface` as do-now (Island 3 MVP gate). Value-finder confirmed this:
  a generator with no disposal path has no value.

**Items I want to flag for the captain:**

1. `feeders/convergent-dread-sentinel-network` is homed do-now as the feeder that produces
   clusters `propose()` eats. But the baton notes this is a SURGE detector and will
   systematically miss the slow-accreting tambear-class (SPC/EWMA dual rate+prevalence
   detector). The ADR for this island should explicitly document the missed-slow-accreting
   class as a known honest limitation — not as a bug, but as a named scope. If this honest
   scope is not in the ADR, the do-now implementation will over-claim its coverage and
   produce a false confidence in the feeder's completeness.

2. The `frames/` islands (4 organizing axes) are homed as "ratify as ADRs / lenses" — but
   none of the ADR task slots cover them explicitly. ADRs 047-051 are the five in-flight.
   The four frame ADRs would be ADR-052 through ADR-055 or similar. If these don't get
   explicit ADR slots before the Pioneers baton, they may arrive as informal design context
   rather than ratified decisions. That is a substrate-alignment gap to watch.

3. `prior-art/determinacy-gate-the-equational-au-tradeoff` is homed as prior-art citation
   (charter-grounding). But the researcher convergence (_notice.md) shows this is the
   theoretical grounding for the three-gate GATE-G stack — and gate-1 (determinacy check)
   should arguably appear in ADR-047's frontier statement. The prior-art home is correct for
   the equational-AU deep work, but the determinacy-gate finding has a do-now citation obligation
   to ADR-047 that isn't currently wired.

---

## Unintended Consequences Nobody Has Named Yet

### 1. The near-miss predicate changes the semantics of "clean corpus"

If the gate adopts near-miss (all-but-one conjunct), the caller's responsibility for corpus
quality shifts. Currently the frontier statement says "the corpus is the ratifier's
responsibility." Under near-miss, the corpus must contain not just "clean code" but
"near-miss clean code" — items close enough to the draft's family that they witness a real
discrimination. A corpus of syntactically unrelated clean code (say, clean utility functions
in a module unrelated to Drop impls) will fail the gate even if those items are genuinely
clean. This is a **UX footgun**: a caller who assembles a clean corpus from the "obviously
safe" parts of the codebase (not the structurally adjacent parts) will get `VacuousAgainstCorpus`
back and not understand why. The ADR must document this clearly: "the clean corpus must be
*structurally adjacent* to the cluster — clean siblings of the family, not random clean code."
This is not a bug in the predicate; it's an honest constraint that must be surfaced.

### 2. The three-valued gate creates a new class of user-facing output that antigen has never had

The `None` verdict ("not corpus-witnessable, route to human") is a first-class new output type.
None of the existing antigen surfaces (scan, audit, report) have a concept analogous to "this
question cannot be machine-answered." The ratification interface will need to render None
distinctly and helpfully. If None is surfaced to the user as a generic "error" or omitted, it
collapses the value of the three-valued gate — the honest route-to-human outcome becomes
invisible. This should be a named design constraint for Island 3 / ADR-051: None is a first-
class outcome with its own human-readable rendering.

### 3. The `into_fingerprint` downgrade + the narrow() re-gate are the same bypass class

ADR-048 OQ2 (`into_fingerprint` extracts a bare Fingerprint — re-promotion requires re-gating)
and Tension-3 Hole 3 (narrow() must re-enter `promote_if_safe`) are the SAME security class:
the human-editable path that produces a bare `Fingerprint` and then needs to re-enter the gate.
If `into_fingerprint` returns a `Fingerprint` that's then editable and re-submittable WITHOUT
re-gating, narrow() can use that path as a bypass. ADR-048 and ADR-051 must coordinate on the
narrow() path explicitly: narrow() produces a `Fingerprint` → must re-enter `promote_if_safe`
→ returns a new `PromotedDraft` or a refusal. This should appear as a named cross-ADR constraint.

### 4. The affinity score's independence requirement conflicts with nothing existing

Adversarial flagged that the accept-side threshold automation requires a score independent of
C's self-assessment — and there are currently zero score definitions in `learn/`. This is not
a conflict, but it is a dependency: the score must be specced before either (a) the ratification-
interface threshold policy or (b) the affinity-spine ADR can ratify. The score is currently
upstream of multiple ADRs. If ADR-049 (no-caller-emits-unscored-output invariant) ratifies
without a concrete score definition, it will ratify a constraint on a thing that doesn't exist.
ADR-049 should include or explicitly cite the score definition, or the invariant is vacuous.

---

## Ratification-Readiness Assessment

| ADR | Status | Blocker | Estimated Distance |
|-----|--------|---------|-------------------|
| ADR-047 | Draft 2 — REOPENED | OQ1 (near-miss P1 vs P2 unresolved); ADR body stale re: OQ1 | Medium (needs ruling + fold) |
| ADR-048 | Draft 1 — sound | OQ2 (ATK on `into_fingerprint`); cross-ADR narrow() constraint | Near (1-2 issues) |
| ADR-049 | Pending | Score definition dependency; must not ratify before score is specced | Blocked |
| ADR-050 | Pending | Depends on two-signal ADR design; do-now slice is clearer than charter slice | Needs drafting |
| ADR-051 | Pending | Adversarial's 3 holes must land; narrow() re-gate must be named | Needs drafting |
| Frame ADRs | Not yet slotted | No explicit ADR slots in the task list | Substrate-alignment gap |

---

## First-Pass Methodology Challenges

### Challenge 1: The ADR-047 body is stale relative to the campsite log

The ADR draft (as of the file on disk) describes D/A/K preconditions with the "strip top-level
AnyOf" skeleton definition. Aristotle's subsequent campsite notes evolve this to relaxed-skeleton
and then near-miss primitive. These are more precise and the log notes are in-expedition substrate
— but the actual ADR file doesn't reflect them. This is the baton's substrate-alignment drift
class: "a teammate says X is done but the artifact doesn't reflect the latest thinking."

The ADR is NOT ready to ratify from the file. The file is the Pioneers' inheritance. If the file
is stale, the Pioneers build from a stale spec. This is the most critical substrate-alignment
issue I see.

### Challenge 2: The 5 ADR task slots don't include the frame ADRs

Tasks 1-5 map to ADR-047 through ADR-051. The disposition skeleton homes the four organizing
axes (affinity-score-is-the-spine, tolerance-quadrant-grid, two-signal-gate,
ratification-throughput-the-real-ceiling) as "ratify as ADRs / lenses" — but no task slot
names them. These are design-principle-level decisions that the baton should carry as ratified.
If they don't get slots, they arrive in the Pioneers baton as informal context that may or may
not be enforced. Observer recommendation: create task slots ADR-052 through ADR-055 for the
four frame ADRs before the Pioneers baton is written.

### Challenge 3: The convergence's biggest novelty claim needs verification

The Cartographers claimed the three prior-art borrows (conformal, subset-principle,
equational-AU) form a determinacy → conservatism → calibration pipeline — a "cross-paper
rhyme" the researcher called a genuine discovery. This is the _notice.md entry. I cannot verify
the claim from the substrate alone, but it has the shape of a strong-but-over-reaching synthesis.
The specific risk: if the subset-principle's "least-general fingerprint" assumption requires
subset-ordering AND equational-AU breaks subset-ordering, then the baton's "subset-principle
makes vacuity structurally avoidable (already shipped)" claim may be weaker than stated. The
LGG is the shipped anti_unify output — but if equational-AU is eventually added and produces
non-unique LGGs, the "structurally no vacuity" guarantee evaporates. This is a future regression
risk, not a current bug, but it should appear in ADR-047's frontier statement.

---

## Open Questions for the Captain

1. **P1 vs P2 (near-miss completion):** Is the crew authorized to pull the C-side
   non-degeneracy guard from charter to do-now? If yes, P1 (separation of concerns) is
   the clean answer. If no, P2 (near-miss + minimum-anchor) is the fallback.

2. **ADR body sync:** Should the adr-specialist fold aristotle's near-miss resolution
   into the ADR-047 file NOW, before ratification, so the file is the source of truth?
   (Observer recommendation: yes — the file is the Pioneers' baton.)

3. **Frame ADR slots:** Should ADR-052 through ADR-055 (or however numbered) be created
   as task slots before the baton is written?

4. **ADR-049 score dependency:** The no-unscored-output invariant needs a score definition.
   Is the score definition part of ADR-049's scope, or does it get its own ADR (ADR-052
   or similar)?

---

## Next Observer Actions

1. File camp notes on the two highest-signal findings:
   - ADR-047 body stale relative to campsite log (substrate-alignment drift)
   - near-miss + narrow() re-gate as the same bypass class (cross-ADR constraint)
2. Tag the relevant campsites with methodology-gap and unverified-claim as appropriate.
3. Camp question to the captain on P1 vs P2.
4. Read the `frames/` campsites for the four organizing axes to assess whether they're
   ratification-ready as ADRs or need more work.
5. Read the real `learn/` source to verify the score-zero claim adversarial made.

---

## Artifacts

- `R:/antigen/jbd/expeditions/v05-the-learning-organism/drafts/adr-047-gate-g-soundness.md`
- `R:/antigen/jbd/expeditions/v05-the-learning-organism/drafts/adr-048-promoted-draft-newtype.md`
- `R:/antigen/jbd/expeditions/v05-the-learning-organism/camps/safety/keystone-harden-gate-g/.notes/campsite.log`
- `R:/antigen/jbd/expeditions/v05-the-learning-organism/camps/callers/ratification-interface/.notes/campsite.log`
- `R:/antigen/jbd/expeditions/v05-the-learning-organism/camps/_notice.md`

---

## Frame ADR Readiness Assessment (second pass, 2026-06-10 ~20:30 UTC)

The four organizing axes were homed as "ratify as ADRs / lenses" in the disposition.
Reading the frame campsites reveals their actual substrate-state:

### frames/affinity-score-is-the-spine
**Campsite state**: `open`, disposition `needs-dreaming` — NOT complete.
**Content depth**: Very deep. The dreamer's initial note + value-finder + aristotle's full
grounding (finding.rs substrate-check, Goodhart constraint, asymmetric-lever ruling).
**ADR-readiness**: HIGH. Aristotle has made rulings sufficient to draft ADR-049 directly:
- Score = recognition of existing provenance ladder (Encountered/Constructable/Heuristic/Imagined)
  + Suspected/Named dial, already on every Finding (finding.rs:79-118)
- Invariant: "no caller emits un-scored output" = read the field that exists
- Asymmetric-lever constraint: auto-reject safe at any threshold; auto-accept ONLY above
  verified-core (Encountered/Constructable). Heuristic/Imagined must still pass human/agent
  ratify-act
- Value-finder's honest ceiling: low calibrated score means "novel" not "weak" for
  negative-space candidates — must not suppress them
**Missing from the campsite**: the score definition is in the substrate (finding.rs), but
ADR-049 as drafted in the task list is "no-caller-emits-unscored-output invariant" — this
needs to also spec what "the score" IS (the existing ordinal) so callers know what to emit.
The verification I ran (zero score definitions in learn/) is correct and expected: learn/ is
the GENERATOR, but the score lives on Finding (in the antigen crate's report layer). ADR-049
must make this explicit: generators don't add a score, they populate the provenance field
that Finding already carries.

### frames/the-tolerance-quadrant-grid
**Campsite state**: `complete`, signed by naturalist. Disposition `needs-dreaming`.
**Contradiction**: campsite is marked COMPLETE (signed by the creator) but disposition is
`needs-dreaming`. This is a substrate-alignment gap: the campsite's self-assessment says
"done" but the disposition label says "not yet homed." The two are inconsistent.
**Content depth**: Excellent. The four-quadrant grid is fully articulated, the mapping to
antigen organs is precise, the ADR-007 structurally-required-set argument is sound.
**ADR-readiness**: HIGH as a lens/design-principle. The grid itself doesn't need an ADR
(it's a completeness-test and metaphor-coherence tool) but the **co-requisite safety organ
reclassification** of catalog-autoimmunity-pruner (peripheral-delete) DOES need an ADR
constraint — probably as a clause in the charter for that organ rather than a standalone ADR.
The captain already made this ruling; it needs to appear in the Pioneers baton explicitly.

### frames/the-two-signal-gate
**Campsite state**: `complete`, signed by naturalist. Disposition `needs-dreaming`.
**Same contradiction** as tolerance-quadrant-grid.
**Content depth**: Outstanding. Naturalist + value-finder + aristotle all grounded this
independently. The cognate is honest (Signal-1 = structural match, Signal-2 = damage-evidence),
the ceiling is honest (software's Signal-2 is sparse/retrospective), the thin dogfood slice
is specified (incident= field + Suspected/Named routing), the charter scope is named (full
runtime-afferent organ).
**ADR-readiness**: HIGH for the do-now slice. Aristotle's ruling is complete:
- DO-NOW: add `incident=` key to `#[dread]` grammar + route incident-bearing → Named-eligible,
  incident-less → Suspected
- DO-NOW: name Signal-2 as shared afferent primitive (under GATE-G, memory-vs-plasma-fate,
  forgetting-curve, pruner)
- CHARTER: full incident-link organ
This is ADR-050 (incident= macro-key + routing-rule). The campsite contains everything needed
to draft it. The only open question is whether the macro-grammar addition is do-now or whether
it goes in a sub-ADR under the existing grammar ADR.

### self/ratification-throughput-the-real-ceiling
**Campsite state**: needs checking — didn't read the campsite.json yet.
**Content depth**: Value-finder initial + value-finder note 2 (budget-split, active-learning
tension, bits-per-ratification). This is the frame under ADR-051 (ratification-record),
not a standalone ADR. The throughput ceiling is the MOTIVATION for ADR-051's design choices
(one co-native record, thin CLI, fate-hook). It belongs in ADR-051's Context section, not
as its own ratified ADR.
**ADR-readiness**: Not a standalone ADR candidate — it's a design-motivation frame.

### Summary: what the frame assessment changes

| Frame | Own ADR? | Where it lands |
|-------|---------|----------------|
| affinity-score-is-the-spine | YES — ADR-049 | Score = recognition of existing ordinal + asymmetric-lever constraint |
| tolerance-quadrant-grid | NO — completeness lens | Co-requisite-safety-organ ruling in charter for pruner |
| the-two-signal-gate | YES — ADR-050 | incident= field + routing rule do-now; runtime-afferent organ charter |
| ratification-throughput | NO — design motivation | Context section of ADR-051 |

This changes the frame-ADR-slots question: two frames need ADR slots (049 and 050 are
already in the task list but need the frame content folded into them); two don't.
The "missing ADR-052+" concern was partly correct — the worry was that frames would
fall through as informal context. The actual answer: 049 and 050 ARE the frame ADRs
for the two that need ratification; 050 has rich campsite content ready to draft from.

### Substrate-alignment drift on frame campsite states — PATTERN (4 instances)

Four frames campsites are state `complete` (expand-wave self-signed) but disposition
`needs-dreaming`. Verified by reading campsite.json files for all six frames campsites:

| Campsite | State | Disposition | Outfitters ruling |
|---|---|---|---|
| frames/affinity-score-is-the-spine | open | needs-dreaming | do-now (ADR-049 scope) |
| frames/the-tolerance-quadrant-grid | **complete** | needs-dreaming | lens / co-requisite-organ ruling in charter |
| frames/the-two-signal-gate | **complete** | needs-dreaming | do-now (ADR-050) + charter |
| frames/the-sixth-stage-falsification | **complete** | needs-dreaming | cite in charter-reflexive-platform |
| frames/the-universal-learning-loop | **complete** | needs-dreaming | single-agent-complete; 6th-stage → charter-registry |
| frames/the-strange-loop-closes | open | needs-dreaming | cite in charter-reflexive-platform |

Root cause: the expand wave signed campsites as done when each dreamer/expansionist
finished; the converge wave re-dispositions them but `camp note` / `camp tag` don't
update the campsite.json disposition field. Camp queries filtering by disposition will
show all four as undecided when the Outfitters have already ruled. Aristotle or the
adr-specialist needs to update these four fields before the Pioneers baton.

All four tagged `substrate-alignment-drift`. Camp notice d9b2850f captures the two
distinct sub-causes (ADR-file-lag vs wave-handoff-disposition-drift).

---

*Observer notes continue in subsequent entries as the convergence develops.*

---

## ADR-047 Post-Fold Consistency Audit (2026-06-10 ~21:00+ UTC, post-compaction)

**Hypothesis**: The adr-specialist's fold (incorporating the near-miss primitive after captain's 5-ruling) was complete. The ADR body should now be internally consistent, with no D/A/K-era nomenclature in active normative sections and the three-valued gate correctly represented.

**Method**: Full read of the updated `drafts/adr-047-gate-g-soundness.md`, verifying (a) the Decision section uses near-miss predicate, (b) the Mechanics section's pseudocode matches the Decision, (c) the Enforcement-Surface table uses consistent names, (d) the Standing-Pressure-Audit Q-section is consistent, (e) Open Questions are either closed or accurately describe their status.

**Result**:

The fold was substantially complete. Key findings:

1. **Decision section (lines 37-44)**: Near-miss predicate correctly stated. `∃ item ∈ corpus, ∃ conjunct c ∈ draft.constraints: (draft MINUS c).matches(item)==true AND draft.matches(item)==false`. This is the correct primitive.

2. **Mechanics section (lines 77-88)**: The `promote_if_safe` pseudocode correctly implements the three outcomes as two sequential two-valued checks composing into three distinguishable verdicts: `NoWitness → Refused(NotCorpusWitnessable)` / `Witnessed + spare_clean fails → Refused(BindsCleanItem)` / `Witnessed + spare_clean passes → Promoted(draft)`. This is coherent. NOT an inconsistency.

3. **Enforcement-Surface table (lines 94-97)**: Uses `NotCorpusWitnessable` consistently throughout. The old name `VacuousAgainstCorpus` appears exactly once in the file — in Q7, explicitly labeled "RETIRED with the skeleton formulation — do not ship them." Consistent.

4. **Q2, Q4, Q7 (lines 104, 106, 109)**: All use `NotCorpusWitnessable` as the committed name. Q7 names a new intermediate enum `NonVacuity{Witnessed, NoWitness}` for the non-vacuity predicate's return — this enum is consistently named everywhere it appears. No D/A/K preconditions remain in normative text.

5. **ATK cases (lines 112-119)**: Q9 has four named ATK cases plus a positive control and a P1 co-requisite test. These are precise and load-bearing.

**BLOCKER FOUND: OQ1 vs ATK-047-2 direct contradiction**

Open Questions section (line 137) recommends ruling (a): a no-disjunction draft should NEVER promote, arguing `skeleton(draft)==draft` makes near-miss impossible.

ATK-047-2 (line 115) asserts the OPPOSITE: the identical `flush().unwrap()` cluster (no `any_of`) + a `flush().ok()` clean sibling promotes (`Witnessed`), and explicitly states "we did NOT lock 'never-promote a no-disjunction draft.'"

The OQ1 logic has an internal error. It claims skeleton(draft)==draft because `any_of` stripping leaves all conjuncts. But `is_near_miss` does NOT compare skeleton vs full draft — it drops individual conjuncts from the `all_of` list one at a time. For the `flush().unwrap()` draft (`method_call(flush) AND method_call(unwrap)`), dropping `method_call(unwrap)` gives the partial `method_call(flush)`, which DOES match `flush().ok()`, while the full draft does not. Near-miss witness found. ATK-047-2 is correct.

The OQ1 text was written under the old skeleton-formulation mindset where "skeleton" was the exclusive mechanism. Under the near-miss predicate, the "deadlock" OQ1 describes does not exist for the cited reason.

**Consequence**: OQ1 must be struck and replaced with a closure: "A no-disjunction draft CAN be near-miss-witnessed via conjunct-dropping from the all_of list. ATK-047-2 demonstrates this. The claimed deadlock (skeleton(draft)==draft prevents witness) was a misconception from the skeleton-formulation era that does not apply to the near-miss predicate. No special case needed for no-disjunction drafts. Closed."

Until OQ1 is closed, the file is internally contradictory. A Pioneer reading it sees a recommendation (never promote no-disjunction drafts) that directly contradicts the spec-tests. **This is a ratification blocker.**

Actions taken:
- Camp note deposited on `safety/keystone-harden-gate-g` with full analysis
- Message sent to adr-specialist with the contradiction and the resolution text
- Message sent to aristotle confirming the fold result and asking for OQ2 confirmation (nested any_of impossibility)

**OQ2 and OQ3 status**:
- OQ2 (nested any_of impossibility): ADR recommends locking "top-level AnyOf only" with a note. Aristotle's read of `propose.rs` should confirm this. Not a blocker on its own — it's a lock-statement, not a dispute.
- OQ3 (naming: `skeleton` → `draft_skeleton`): Q7 says the old name is RETIRED. The naming question in OQ3 is whether to use `draft_skeleton` vs keep `skeleton`. This is cosmetic. Lock-at-ratification.

**Updated ratification-readiness estimate**:

| ADR | Status | Blocker | Estimated Distance |
|-----|--------|---------|-------------------|
| ADR-047 | Draft 3 (near-miss fold) | OQ1 contradiction with ATK-047-2 must be struck; OQ2 needs Aristotle's confirmation | Near (1 required close, 1 confirmation) |
| ADR-048 | Draft 1 — sound | OQ2 (ATK on `into_fingerprint`); cross-ADR narrow() constraint with ADR-051 | Near (1-2 issues) |
| ADR-049 | Pending | Score = existing class_provenance ordinal (recognition); must be explicit in the ADR | Blocked on drafting; content is ready |
| ADR-050 | Pending | Rich campsite content; two-signal-gate frame is ready | Needs drafting; near-ready |
| ADR-051 | Pending | Adversarial's 3 holes must close; narrow() re-gate must be named | Needs drafting; complex |

**Scientific assessment**:

The adr-specialist's fold was a genuine improvement — the file went from skeleton-formulation (the ADR-file-lag pre-fold state I flagged) to near-miss predicate with correct three-valued composition. The remaining inconsistency (OQ1 vs ATK-047-2) is a known-type failure: a speculative recommendation left in the Open Questions section after the spec-test settled the question the other way. The spec-test (Q9/ATK) was written as "the born-red spec IS the definition of done." OQ1's recommendation was not struck when ATK-047-2 was added. This is the ADR-file-lag pattern manifesting at the intra-ADR level — two sections of the same document written at different times, each internally coherent, contradicting each other across time.

**This notebook entry constitutes the post-fold forensic record.** The consistency audit is complete. ADR-047 is one closing away from ratification-ready.

---

### Addendum: OQ1 resolution nuance + OQ2 closed (from aristotle, same waking)

**OQ1 resolution — sharper framing than the initial audit**:

My initial identification was correct (OQ1 and ATK-047-2 directly contradict each other; OQ1's logic has an internal error). Aristotle clarified the precise mechanism:

B does NOT refuse no-disjunction drafts. It routes them via the D precondition (`NotCorpusWitnessable` / route-to-human) when the collapsed draft can't be corpus-witnessed. The draft may be SAFE (spare-clean holds — it doesn't bind clean items); what B can't certify is the generation-quality (the collapsed identical-twins case has no discriminating `any_of` arm to make the near-miss witness meaningful). So: B routes to human rather than auto-promoting. This is the correct behavior — the SAFETY property holds, the DISCRIMINATION property is uncertain, and the honest outcome is route-to-human not refuse.

ATK-047-2 is consistent with this: the `flush().unwrap()` cluster IS corpus-witnessable (dropping `method_call(unwrap)` from the `all_of` list gives a skeleton that matches `flush().ok()`, which the full draft does not). So it promotes. ATK-047-2 is not a special case; it's the normal near-miss path working correctly for a no-disjunction draft that happens to have a near-miss witness in the corpus.

OQ1's error: claiming `skeleton(draft)==draft` makes near-miss mechanically impossible. Wrong, for the reason identified: `is_near_miss` operates on conjuncts in the `all_of` list, not on `any_of` stripping. Any no-disjunction draft CAN have a near-miss witness if the corpus contains a structurally adjacent clean item. For the collapsed-twins case specifically, near-miss may still be NoWitness in practice (if the corpus doesn't have a nearby item), which correctly routes to human. The reasoning chain is: the route-to-human outcome is correct AND is handled by D (near-miss check), not by a special "never-promote no-disjunction" rule. OQ1's recommendation (a) was wrong. adr-specialist closing OQ1 with the D-routing resolution + B/C separation.

**OQ2 closed — substrate-grounded proof**:

Aristotle confirmed from HEAD-read of `propose.rs` (lines 96-192) + `BodySignal::to_constraint` (lines 237-244). Four code paths enumerated:
1. `Constraint::Item(shared_kind)` — flat leaf
2. `Constraint::ImplOfTrait(...)` — flat leaf  
3. `sig.to_constraint()` → `BodyCalls(String)` or `BodyContainsMacro(String)` — flat leaves
4. At most ONE top-level `Constraint::AnyOf(arms)` (line 176), whose arms come from `to_constraint()` which returns ONLY flat leaves

No code path nests AnyOf inside AnyOf or inside a sub-AllOf. Lock statement: "`anti_unify` emits at most one top-level `AnyOf`, never nested; a future generator emitting nested disjunctions requires a skeleton-derivation amendment."

**Updated ADR-047 ratification estimate**:

With OQ1 closing (in-progress, adr-specialist) and OQ2 now locked, the remaining open item is OQ3 (naming: `skeleton` → `draft_skeleton`), a cosmetic choice to be made at lock. ADR-047 is effectively ratification-ready pending the OQ1 text update and OQ3 naming decision.

| ADR | Status | Blocker | Estimated Distance |
|-----|--------|---------|-------------------|
| ADR-047 | Draft 3 + OQ1/OQ2 pending close | OQ1 fold in-progress; OQ3 cosmetic naming | **Very near** (fold landing now) |
| ADR-048 | Draft 1 co-locked | ATK on `into_fingerprint` (OQ2); narrow() cross-ADR constraint named | Near |
| ADR-049 | Pending | Score = existing class_provenance ordinal; ADR-052 must ratify first | Blocked on ADR-052 (now drafted) |
| ADR-050 | Pending | ADR-054 frame drafted; two-signal-gate content ready | Near — needs drafting |
| ADR-051 | Pending | Adversarial's 3 holes + narrow() re-gate; ADR-055 frame drafted | Needs drafting; complex |
| ADR-052 | Drafted | Ratification-ready per observer peer-review | Very near |
| ADR-053 | Drafted | Ratification-ready per observer peer-review | Very near |
| ADR-054 | Drafted | Ratification-ready per observer peer-review | Very near |
| ADR-055 | Drafted | One clarity gap: ADR-051 should say "pending ratification" in status block | Very near (1-line fix) |

---

### Frame-ADR Peer-Review (2026-06-10 ~20:45+ UTC)

adr-specialist drafted ADR-052..055 in `drafts/adr-052-055-frame-adrs.md` — four frame-ADRs in one file, each ratifying individually. This resolves the substrate-alignment gap flagged in the initial baseline (observer finding #2: no explicit ADR slots for the four organizing axes).

**ADR-052** (affinity-score-is-the-spine): Recognition of the existing `class_provenance: Provenance` ordinal from `finding.rs`. PROVEN = ordinal dial on every caller; DEFERRED = calibrated continuous score. Correctly blocks ADR-049 (needs the score type to exist before the "no un-scored output" invariant is non-vacuous). Ratification-ready.

**ADR-053** (tolerance-quadrant-grid): Ratifies the placement discipline (a tolerance-dream homes in a quadrant) and the required-set argument (no single quadrant suffices — biology-grounded, ADR-007). PROVEN = grid frame + placement rule + central-delete shipped; DEFERRED = three empty quadrants by name, charted individually. The biology→software transfer is asserted not proved, but within the honest scope (the ADR doesn't over-claim). Ratification-ready.

**ADR-054** (two-signal-gate): The cross-reference to ADR-047 is the sharpest piece — near-miss non-vacuity = structural costimulation, `incident=` = semantic costimulation, both Signal-2 readings. Suspected ceiling as principled caution (not a gap) is correct and honest. PROVEN = `incident=` key + routing ceiling do-now; DEFERRED = runtime-afferent organ. Ratification-ready.

**ADR-055** (ratification-ceiling): The specimen-triple + dial-as-coarse-policy-lever as do-now judgeability surface is sound. One clarity gap: the ADR references "ADR-051 (the co-native ratification record)" without flagging that ADR-051 is pending ratification. A Pioneer could read this as ADR-051 already being ratified. Fix: add "(pending ratification, v05 Outfitters)" to ADR-051 references in the Status block. Otherwise ratification-ready.

**Control-loop synthesis** (lines 67-71): The four-frame control loop (052 → 054 → 053 → 055 = spine → two-signal → tolerance-grid → ceiling) is correct and is the right connective tissue to include. This makes the frames enforceable as a set at the axis level.

**Overall assessment**: The initial substrate-alignment gap (four organizing axes informal, no ADR slots) is resolved. These four ADRs are the Pioneers' enforcement substrate for the four axes. The wave is accelerating toward the baton.

---

### Challenge 3 Resolution: Subset-Principle and GATE-G (2026-06-10 ~21:15 UTC)

Initial baseline Challenge 3 flagged: "the baton's 'subset-principle makes vacuity structurally avoidable (already shipped)' claim may be weaker than stated — if equational-AU breaks subset-ordering, the guarantee evaporates."

**Resolution**: The concern was about a FUTURE regression, not a current bug. The near-miss predicate does NOT rely on subset-ordering or equational-AU — it operates on `anti_unify`'s `all_of` conjunct list by dropping conjuncts one at a time, using the shipped `Fingerprint::matches` matcher. No subset-ordering assumption is required.

The baton's "structurally avoidable via subset-principle" claim referred to the OLD naive vacuity problem (≥1 bindable item check was trivially satisfied by cluster members). The near-miss predicate is STRONGER — it requires a CLEAN CORPUS item to be near-miss, not a cluster member. The subset-principle argument does not directly apply to the clean corpus requirement.

**The future regression risk remains**: if equational-AU is added and produces non-unique LGGs (breaking subset-ordering), the `anti_unify` output may no longer be the minimal generalization. The near-miss check would still run (it doesn't care about minimality) but the interpretation of "the corpus exercised the draft" becomes less meaningful if the draft's conjunct set is no longer the minimal one. This is a CHARTER-era concern, not a current ADR concern.

**Assessment**: Challenge 3 is resolved for the current design. The frontier statement note (adding "gate-1 of a three-gate stack" context) remains a nice-to-have but is not a logic flaw. The future-regression risk should appear in the equational-AU charter when that work is scoped.

---

### ADR-050 Peer-Review (2026-06-10 ~21:00+ UTC)

ADR-050 (`incident=` key + Signal-2 routing rule) has been drafted: `drafts/adr-050-incident-key-and-routing.md`. Observer peer-review: ratification-ready pending OQ3 lock-statement.

**Strengths**: PROVEN/DEFERRED split exact. "Principled caution, not a gap" framing is correct and prevents the docs-as-mirror trap. Q-section thorough. v1→v2 doc-marker additive-optional approach is sound. The cross-ref to ADR-047 (near-miss = structural costimulation; incident= = semantic costimulation; both Signal-2) is sharp and correct.

**Three OQs resolved by observer**:
- OQ1 (`incident=` on `#[aura]`): YES, accept on all three — uniform field, no per-marker divergence
- OQ2 (v1→v2 vs permissive-parse): Both paths correctly handled. Source-attribute reader (ScanMarkerArgs, scan/mod.rs:466-478) silently drops unknown fields via `_ =>` arm — forward-compat guaranteed. Doc-marker side needs v2 bump — ADR handles this correctly.
- OQ3 (per-cluster vs per-draft): Per-cluster is correct. Lock as: "Named-eligibility is per-cluster: ≥1 constituent mark with an `incident=` link suffices."

**Overall**: ADR-050 is ratification-ready once OQ3 is locked.

---

### New Unintended Consequence: mark-as-search digest confusion

Aristotle's mark-as-search ruling (S1-filler, input-dual of GATE-G) cited `structural_digest` from `scan/diff.rs:171`. This is the WRONG digest for clustering.

Two distinct digest functions exist in `antigen-fingerprint/src/digest.rs`:
- `structural_digest` (line 150): IDENTITY digest — name+code-sensitive. Used by diff-native to detect changes to a named item. Doc: "identity (name+code-sensitive) digest — precisely because a structure change MUST register."
- `structural_shape_digest` (line 195): SHAPE digest — name-insensitive. Used by the PROPOSE-slice for clustering similar-shaped items regardless of name.

Mark-as-search needs to find "other items with the same body shape as this dread mark, regardless of name." That requires `shape_digest` / `structural_shape_digest`. Using `structural_digest` would cluster by identity — `fn foo()` only matches other `fn foo()` with the exact same body. Most marks would find zero results.

The `Finding.shape_digest` field (finding.rs:304) is already populated at scan time from `structural_shape_digest` (scan/parse.rs:99) — the machinery is fully shipped. The ruling is correct in spirit; the wrong digest name was cited. Camp note deposited on `feeders/marking-incentive-the-source-problem`. The pathmaker should build against `shape_digest`, not `structural_digest`.

---

### ADR-047 Attribution-Drift Audit (2026-06-10 ~21:07 UTC, post-compaction)

**Hypothesis**: Aristotle flagged four stale attribution markers in ADR-047 after the Island-2.5 SPLIT (the adr-specialist was editing live at the time of compaction). The audit question: are they now clean?

**Method**: Grep `drafts/adr-047-gate-g-soundness.md` for the four specific patterns aristotle named: `catches (the )?twins`, `makes this verdict.*rare`, `this routing is.*rare`, `twins/bare-structural` conflated. Direct file read of lines 52 and 67 (where "rare" was found in the pre-compaction read). Verify (A)-binary bare-structural SAFETY refusal still in B.

**Results**:

All four grep patterns returned 0 matches. The adr-specialist completed the fix concurrently with my pre-compaction read. The stale language in lines 52 and 67 has been replaced with:

- Line 52 (ATK-047-2 paragraph): "Upstream, the **generalization-confidence tier-input** (Island-2.5's signal half → ADR-050) lowers a twins-draft's tier so it is surfaced as low-confidence rather than reaching the human as an un-annotated route; the route-to-human is then a tier-aware consequence, not a surprise."
- Line 67 (route-to-human bullet): "the generalization-confidence tier-input (Island-2.5's signal half → ADR-050) annotates the twins-draft as low-confidence so the route-to-human is tier-aware, not a bare surprise."

(A)-binary bare-structural SAFETY refusal correctly stays in B (lines 56-57 and 60, untouched).

**Audit-method lesson** (from the adr-specialist's own retrospective, note 3f7fa1bb): grepping for NEW-TERM-PRESENCE vs OLD-TERM-ATTRIBUTION-DRIFT are different checks. Attribution-drift uses valid vocabulary to make a false claim after a concept was re-homed. The correct check when a concept is split: per-sentence "does X still DO what this sentence says X does?" — not "does the new name appear / the old name disappear." This is antigen's own `RatifiedSpecDriftFromImpl` class appearing in antigen's own ADR audit process. The recursive proof in action.

**Conclusions**: ADR-047 is attribution-drift-clean. The Island-2.5 SPLIT (twins → generalization-confidence tier-input → ADR-050 / bare-structural → C-guard + (A)-binary) is consistently expressed throughout the file. Q6 Option→Result also confirmed fixed (line 101 explicit).

Camp note deposited on `safety/keystone-harden-gate-g`. Message sent to aristotle confirming audit pass.

---

### ADR-051 Persistence-Seam Gap (2026-06-10 ~21:10 UTC, observer finding)

**Hypothesis**: The adversarial's persistence-seam finding (note 693cb35a, 20:58 UTC) was reported as "still open as of sleep — reported to aristotle; verify it lands in ratified ADR-051." Has it been folded?

**Method**: Grep ADR-051 for `PersistedSpecimen`, `round_trip`, `re-mint`, `re-gate on load`.

**Result**: 0 matches. The gap has NOT been folded into ADR-051.

**The gap**: `RatificationSpecimen` holds `draft: PromotedDraft` directly (line 61). The fate-hook requires persisting the specimen + ratification history (line 51, the L4-staleness precondition). But `PromotedDraft` MUST NOT derive `Deserialize` (ADR-048 §5, named in ADR-051 line 34). The ADR never names the on-disk form or the load-path.

**Risk**: an implementer hits the compile error (`PromotedDraft: !Deserialize`) when trying to serialize `RatificationSpecimen`, and "fixes" it by adding `Deserialize` to `PromotedDraft` — reopening the exact serde-forgery hole the ADR closed. The mitigation is to name the pattern explicitly.

**Pattern needed** (from adversarial's sleep note 450738b9):
- `PersistedSpecimen { draft_fingerprint: Fingerprint, gate_verdict, cluster, spared, fate }` — the on-disk form (bare Fingerprint, no capability token)
- Load path: deserialize `PersistedSpecimen` then re-mint via `promote_if_safe` to reconstruct a live `RatificationSpecimen`
- A persisted `Accepted` whose fingerprint no longer re-gates = the drift/peripheral-tolerance flag
- Q9 += `round_trip_specimen_persists_bare_fingerprint_and_re_mints_on_load`

**Assessment**: This is a pre-ratification blocker for ADR-051. The adr-specialist's cross-ADR consistency audit (66a80540) fixed the bypass-class count from 'both' to 'all three' but did not address the PersistedSpecimen pattern. The adversarial was sleeping; the gap survived into the post-consistency-audit substrate.

**Actions taken**: Camp note deposited on `callers/ratification-interface`. Message sent to aristotle flagging as pre-ratification blocker.

**Ratification-readiness update**:

| ADR | Status | Remaining blockers |
|-----|--------|-------------------|
| ADR-047 | Post-fold, attribution-drift-clean | OQ2 (nested any_of, already locked as lock-statement per OQ2 text); OQ3 (naming, cosmetic) — **ready to ratify** |
| ADR-048 | Co-locked with 047 | No new blockers found this session |
| ADR-049 | Drafted | Blocked on ADR-052 ratifying first (acknowledged do-now) |
| ADR-050 | Drafted | OQ3 per-cluster lock-statement; ratification-ready |
| ADR-051 | Drafted | Persistence-seam gap (PersistedSpecimen pattern) must land before ratification |
| ADR-052..055 | Drafted | Ready to ratify (observer peer-review complete; one clarity-gap in ADR-055 noted) |
| ADR-056 | Drafted | Ratification-ready pending 3 OQ lock-items (Confidence enum vs dial-map; diversity computation; `is_degenerate` home) |

**ADR-056 peer-review** (2026-06-10 ~21:12 UTC): C-side non-degeneracy guard + generalization-confidence signal. Two-half split correctly realized: `is_degenerate` (REFUSAL, bare-structural) + `generalization_confidence` (SIGNAL, twins-overfit). Defense-in-depth framing correct; B's (A)-binary remains standalone safety backstop; C's guard adds generator-appropriate diagnostic. One predicate, two call-sites — `ParallelStateTrackersDiverge` avoided. `ProposeOutcome::Degenerate` already in ADR-048 (line 45) — no cross-ADR lag. Three OQs are lock-choices not contradictions. Ratification-ready pending lock calls.

**Full do-now ADR set as of 21:12 UTC**: ADR-047 through ADR-056, all drafted. One pre-ratification blocker open: ADR-051 persistence-seam gap (PersistedSpecimen pattern). All others ratification-ready pending captain's lock acts.
