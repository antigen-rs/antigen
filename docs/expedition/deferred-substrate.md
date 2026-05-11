# Deferred substrate — what's parked, why, and what unblocks it

> **Purpose**: durable index of substrate that is intentionally deferred,
> so items don't fall through cracks while attention is on something else.
>
> **Maintained by**: navigator. The team-lead and Tekgy read this to
> verify nothing is being lost. Side-substrate items not on this list
> may be drifting silently.
>
> **Discipline**: when work is deferred, the deferral is recorded HERE
> with status + why-deferred + what-unblocks. When the unblock condition
> is met, the item moves out of this index into active campsite work
> (and the row is removed). The index itself is the substrate-currency
> check at the team-coordination tier — the answer to "is anything being
> lost while we focus on X?"
>
> **Companion to**: `postures.md` V0+1 candidates section (which tracks
> patterns watched for posture-class promotion). This index tracks
> *work* deferred, not patterns watched. The two are different shapes.
>
> **Status**: V23 (2026-05-11, V1 Part I "emergent practice" revision committed; A5 stdlib-governance encounter registered; "protocol" → "emergent practice" Tekgy-ratified).
> V1-V4: D1.5 + A3-immediate closure.
> V5-V6: multi-component substrate committed + team routing active.
> V7: scout — Component 7 confirmed, 3 ADR prose gaps.
> V8: naturalist C4 boundary-silence; Q1 provisional answer: layered.
> V9: adversarial — 5-item amendment queue, 2 A5 governance findings held.
> V10: ATK-A3-011..014 committed; expansion pass substrate-complete.
> V11: multi-component-immunity.md V1 committed (dd9c0bc); aristotle unblocked (Q1 ratified
> by Tekgy); 5 encounter-registrations added; 2 new vocabulary candidates tracked.

---

## ~~A3-immediate (post-ratification cleanup)~~ — CLOSED 2026-05-10

All A3-immediate items complete:
- ~~ATK-A3-007~~: 4a1ed17 (adversarial)
- ~~ATK-A3-009~~: bf44056 (adversarial)
- ~~D1.5~~: 2eb8bec–b7712df (pathmaker); 235 passing, 23 ignored, all CI green

---

## Active incoming substrate (team-lead working)

Items team-lead is actively producing; not deferred — in-flight.

### Multi-component immunity framing

**What**: Tekgy + team-lead conversation (2026-05-10) produced a substantial
new framing: antigen as heterogeneous multi-component immune system. Six
components identified: (1) dev-judgment, (2) passive scan/tools, (3) test
integration, (4) knowledge-ecosystem integration, (5) version/lineage,
(6) cross-crate/ecosystem.

**Substrate committed** (2026-05-10, ca812de):
- `docs/expedition/multi-component-immunity-conversation.md` — raw conversation (~530 lines)
- `docs/expedition/multi-component-immunity.md` — deep-dive draft (~870 lines)

**V1 committed** (2026-05-11, dd9c0bc): incorporates all expansion-pass findings.
Component 7 first-class; C4 boundary-silence; engineered-boundary tier named;
manifold framing; honest-boundary-as-encounter-registration; 12 open questions.

**Active team routing** (idle-as-invitation; full creative freedom):
- **Naturalist**: vocabulary-as-protocol COMPLETE (2026-05-11, campsite `20260511120843-vocabulary-as-protocol-cognate-refinement.md`); C1/C2/C3/C5/C6/C7 at idle cadence.
- **Scout, Adversarial, Pathmaker**: complete; full creative freedom.
- **Aristotle**: Phase 1-8 complete (2026-05-11). ADR amendment queue CLOSED
  (35130f2, 2026-05-10). Idle work: manifold-axes partition test complete
  (campsite `20260510-manifold-axes-partition-test.md`, 2026-05-10).

**Manifold-axes partition test results** (aristotle, idle work, 2026-05-10):
Q12/Q13 now answered. Four candidate axes tested; 2 real, 2 convenient cuts.

- **Axis 2 (static-vs-dynamic)**: real (6/7 clean). Load-bearing-reason:
  currency-mechanism difference. Finding 4: V1 Part V's "structural-tier vs
  maintenance-tier" IS Axis 2 at project-layer granularity; partition test shows
  same axis within antigen at component-layer granularity. Two-scales-one-axis.
- **Axis 3 (individual-vs-population)**: real but skewed (6/7 clean; C6 is
  the special case). Load-bearing-reason: coordination-scope + trust-boundary
  requirement (population side requires ADR-017; individual side doesn't).
- **Axis 1 (production-vs-consumption)**: convenient cut. Applies to operations,
  not components; components bundle operations, axis can't carve them cleanly.
- **Axis 4 (implicit-vs-explicit)**: convenient cut. Antigen's posture pulls
  everything toward explicit; axis describes project direction, not per-component
  variance.

**Q8 calibration sharpened**: meta-claim ("enumeration has structure") survives
with three real axes (Q1 tier axis + Axes 2 + 3). V1's specific axis-listing is
exploratory-not-authoritative; should be flagged as such when V1 promotes.

**V2 inputs from partition test**: Axes 2/3 ratification + Finding 4
internal-coherence cross-reference. Hold here until V2 is scoped.

**Scout findings landed** (2026-05-10):
- **Component 7 confirmed**: real-time / CI feedback is structurally distinct
  from Component 2 by scope (diff-scope vs workspace-scope), not just latency.
  Distinct audience (PR author), distinct integration surface (PR comment /
  status check), genuine dependency on Component 2's ScanReport as baseline.
  Passes load-bearing test. Needs naturalist cognate (neutrophil?) and
  adversarial threat-model.
- Vocabulary-as-protocol framing independently confirmed: vocabulary IS the
  cross-component coordination layer (not Component 0). Scout confirmed the
  *function*; naturalist's 2026-05-11 refinement sharpens the *mechanism-framing*
  (see below). Complementary, not contradictory.

**Naturalist vocabulary-as-protocol refinement landed** (2026-05-11):
"Protocol" carries engineered-spec baggage (specification, versioning, conformance
testing, designated authority) that conflicts with ADR-006 recognition discipline.
Antigen-the-vocabulary is co-evolved/recognized (emergent-and-recognized, not
specified-and-versioned). Refinement candidate: "coordination substrate" or
"co-evolved shared interface" rather than "protocol" for the deep-dive revision.
Layer-distinction: vocabulary itself = co-evolved coordination substrate (ADR-006);
governance of vocabulary = engineered process (process.md). Different layers,
honest cognates at each. Three predictive shifts:
  1. Adoption ergonomics: recognition-as-substrate-fitting, not conformance-checking.
  2. Ecosystem evangelism: recognition-language attracts naturalist-mindset adopters
     (ADR-006's load-bearing audience) vs. compliance-mindset from protocol-language.
  3. Antigen-stdlib governance (A5): should be recognition-grounded (three independent
     instances → propose antigen), not spec-grounded. Tekgy-ratified 2026-05-11;
     registered as encounter below (held-for: A5-scope-lock-substrate).
- Decay/sunset: genuine vocabulary gap (no "retired antigen" primitive);
  encounters-tier or future ADR candidate; not a new structural component.
- Cross-team/org, adversarial-discipline, educational/onboarding: all
  disposed as non-peer-components per scout's empirical test.

**Ratification cadence**: Tekgy + team-lead decide after team expansions surface.
Not project-tier substrate yet — expedition/ staging area.

**Owner**: naturalist + adversarial (still expanding) → aristotle (Phase 1-8
after expansion) → team-lead + Tekgy (ratification cadence).

---

## A3 sweep deferral (parallel substrate)

Items that are A3-cadence but explicitly held while ratification cycle
runs.

### Encounters proposal — full Phase 1-8 + ratification

**What**: Proposal for a sibling vocabulary tier alongside `postures.md`
for first-encounter formal capture. Phase 1-8 SMALL-PUSH complete
(2026-05-09). Three Tekgy framings now landed as substrate commitments:

1. Findings 1-3 accepted: scope-coherence (four shapes = one abstraction
   at different scales); sibling `encounters.md` placement (not sub-tier);
   `recognition-cue` required field (the friction is the gate).
2. Governance: inherits postures.md governance by default. Don't pre-design;
   recognize what works and let encounters inherit. Ratify divergence if/when
   it surfaces.
3. Tooling/discipline co-existence: keep BOTH layers active as tools mature.
   "Extend, don't replace" — encounters-the-discipline is the human/agent-
   facing layer; encounters-tooling-eventually is the leverage layer. They
   cross-link rather than one superseding the other. Same posture as antigen-
   the-tool: structural memory gives developer judgment leverage, doesn't
   replace it. This framing should land explicitly in the encounters substrate
   so future-instances inherit it rather than re-derive.

**Where it lives**: `docs/expedition/encounters-proposal.md` (tracked, 446
lines); aristotle's SMALL-PUSH artifact at
`campsites/antigen-A3/20260509163016-20260509080000-launch/aristotle/20260509230000-encounters-phase-1-8-initial.md`.

**Next move**: full Phase 1-8 → ratification cycle. Three Tekgy framings
+ eight aristotle findings are the substrate for that pass.

**Unblocked by**: Tekgy bandwidth + team capacity post-A3-close.

**Owner when active**: aristotle (full Phase 1-8) → team deconstruction →
ratification per process.md. First artifact: produce `encounters.md` v0
draft incorporating the three substrate commitments above.

### Naturalist's deeper biology-cognate work

**What**: Multiple sparks emerged during A3 launch — version-boundary as
immune memory re-activation (corrected to drift/shift, not memory-waning),
declared-vs-earned identity rhyme, Approach 4 (fingerprint-keyed) and
Approach 8 (epitope-class lattice) as biology-cognate post-A6 territory.
Threads not yet woven into the manuscript trajectory or
`immune-system-primitive-map.md`.

**Where it lives**: naturalist's A3 campsite entries; `cross-domain-architectural-map.md` V1 (academic-researcher's A2 work).

**Unblocked by**: idle-as-invitation (naturalist's natural cadence);
manuscript drafting cycle when scientist re-engages.

**Owner when active**: naturalist + scientist (when scientist spawns).

---

## Registered known-unknowns (honest-boundary encounter-registrations)

Structural gaps named during A3 multi-component pass. Each is a known-unknown:
we see the boundary, we know what lives beyond it, we don't yet have the
structural-memory answer. Per the "honest-boundary as encounter-registration"
discipline (Tekgy 2026-05-11, V1 Part V; Q7 ratified 2026-05-11): periodically
revisit; ask if a structural-memory or component answer has surfaced; promote
to encounters-tier when discipline ratifies; remove if resolved by ADR or impl.

**Additional encounter-registrations from Q6/Q7/Q8 ratification (2026-05-11):**

### Engineered-substrate-exceeds-biology family (encounter-registered)

Three instances: W7 FormalProof tier, ADR-017 trust-delegation, C4
knowledge-ecosystem. Count trigger fires; second gate (multi-role + engineering-
reason coherence) not yet met. `held-for: cross-project-posture-if-generalizes`.
Registered per Q6 ratification.

**Revisit when**: pattern surfaces in a second project context with different
role discovering it, OR engineering-reason coherence across all three instances
becomes clearer.

### Manifold structure of antigen enumerations (encounter-registered)

Observed in multi-component-immunity V1 enumeration. "Manifold" terminology
is suggestive-not-technical. Generalization to other antigen enumerations
(postures.md, decisions.md, 21-cell contact-graph matrix) unverified.
Registered per Q8 ratification.

**Revisit when**: another antigen enumeration is inspected and shows similar
property. Accrue evidence; promote only when multiple enumerations confirm.

### Cargo-level attack boundary

**What**: CARGO_HOME override, Cargo.lock manipulation, registry cache tampering.
Antigen's trust model does not and cannot address these — they are pre-antigen.
ADR-017 Amendment 1b will name the boundary explicitly ("predicated on cargo
metadata integrity"). The known-unknown: what DOES address this tier? (cargo
itself, supply-chain tooling, sigstore, etc.) Not antigen's domain, but worth
knowing who owns it.

**Source**: adversarial threat model 2026-05-10; ADR-017 Amendment 1b.
**Revisit when**: cargo supply-chain tooling landscape clarifies or antigen
stdlib governance (A5) surfaces an answer.

### Cross-crate witness execution gap

**What**: `witness = dep_crate::some_test` — consuming workspace cannot execute
it. `ExternalUnvalidated` is the honest tier. The known-unknown: what WOULD
make cross-crate witness execution possible? (republishing test suites as
features, separate verification crates, formal proof artifacts.) ADR-005
Amendment 3 update (aristotle queue item 5) names the gap; doesn't fill it.

**Source**: adversarial threat model 2026-05-10; ATK-A3-011.
**Revisit when**: A4-A5 behavioral witness tier implementation opens the design.

### LLM-hallucinated references

**What**: LLMs generating antigen references they'll later trust. Hallucinated
URLs look calibrated-to-plausible but reliably 404. The known-unknown: what
reference-validation tier would distinguish them? (ValidatedReference /
DeadReference annotation per ATK-A3-014; shared-cluster detection for
single-point-failure risk.) A5 governance territory.

**Source**: adversarial threat model 2026-05-10; ATK-A3-014.
**Revisit when**: A5 scope-lock opens reference-validation design.

### Immunity laundering via newtype

**What**: Wrapper crate declares `#[immune(X)]` on a newtype wrapping a
foreign type, with a theatrical witness that passes without exercising X's
actual failure mode. Downstream inherits `ExecutionVerified` without
independent verification. Structurally valid under current trust model.
The known-unknown: what behavioral witness tier would detect theatrical
witnesses? (A4-A5 implementation concern per ATK-A3-011.)

**Source**: adversarial threat model 2026-05-10; V1 §C6 failure modes.
**Revisit when**: A4-A5 behavioral witness tier design opens.

### Antigen-stdlib trust hierarchy

**What**: Ecosystem-wide immunity declarations from a compromised stdlib
maintainer could suppress local presentations without local opt-in.
Single-point-of-failure at ecosystem scale. The known-unknown: what
governance model makes ecosystem-tier declarations safe? (Per-crate
opt-in, multi-party signing, antigen-council governance, etc.)

**Source**: adversarial threat model 2026-05-10; A5 governance finding.
**Revisit when**: A5 scope-lock opens antigen-stdlib governance design.

### Seam-tier vs type-tier antigen classification (encounter-candidate, single instance)

**What**: Tambear's math-researcher surfaced a structural distinction between
two antigen classes: (1) **type-tier / signature-time antigens** — failure-class
lives at API/signature surface; caught by phantom-type witnesses, type-system
contracts; (2) **seam-tier / composition-time antigens** — failure-class lives
at cross-implementation composition boundary; caught by cross-implementation
consistency tests. Concrete instance: ExpKernelState's `(1 + expm1_r) << k`
reconstruction meeting standalone exp.rs return value diverges from F13.C
(signature-time mis-routing). Different mechanism, different witness shape.

**Grounding note**: tambear is naive smoke-test consumer per
`feedback_tambear_is_smoke_test_not_design_input.md`. HOWEVER, tambear's
math-researcher encountered this in real mathematical practice (exp.rs
Taylor-vs-Remez composition). That's substantively stronger grounding than
tambear's general impl choices — the distinction is live from actual practice,
not proposed speculatively.

**What's genuinely new**: antigen's existing vocabulary can express the instance
(declare as antigen; consistency-witness via proptest/test). The potentially-new
substrate is the *organizing axis* — seam-tier vs type-tier as a way to classify
antigens themselves. This would operate at antigen-taxonomy level, distinct from
aristotle's manifold-axes partition test (which partitions immune-system
components, not antigen types).

**Scout structural-rhyme check COMPLETE** (2026-05-10, pre-halt, campsite
`20260510-seam-tier-type-tier-structural-rhyme-check.md`): retire-to-documentation
disposition confirmed. Three rhymes fired (WitnessKind, `#[presents]` site
selection, `#[immune]` location) — all locate the distinction at WHERE-TO-MARK-
IN-EXISTING-VOCABULARY, not at need-new-primitive. Practitioner insight: for
composition-site antigens, mark the consistency test (the test IS the seam-proxy
in existing code). Existing vocabulary handles this; no ADR needed.

**Real structural gap surfaced by scout**: multi-target `#[presents]` for two-
sided seams (failure lives in the RELATIONSHIP between two code sites, not in
either one). `ItemTarget` enum has no composition-relationship type. One instance;
hold as registered-known-unknown until three independent instances.

**ADR substrate**: seam-tier/type-tier distinction not latent as organizing axis
in ADR-005 or ADR-013 — ADR-013 phantom-type is type-tier-adjacent but neither
ADR uses composition-site as a concept. Audit behavior unchanged once right site
is marked.

**Adversarial ATK-A3-015..018**: four pre-impl contracts filed (oracle-wrong,
wrong-seam, tier-mis-classification, retire-to-documentation guards). ATK-A3-018
proposes three process guards for retire-to-documentation (second-opinion,
revisit-window, concrete-artifact). Scout's and adversarial's analyses are
consistent — adversarial's guards are the mechanism; scout's analysis is the
structural reason those guards are needed.

**Naturalist cognate check COMPLETE** (2026-05-10, campsite
`20260510231118-seam-tier-type-tier-cognate-check.md`): SPLIT-DISPOSITION verdict.

- **Seam-tier classification axis** → *retire-to-documentation* (confirmed).
  Boundary-silence on all three bio-cognate candidates (hypermutation: shape
  mismatch; Th1/Th2: shape mismatch; vaccinated-vs-natural: argument-mode
  without boundary-silence). No clean immune-system cognate. Adjacent biology
  candidates (cross-species PPI, bispecific antibodies) are non-immunology or
  engineered; outside documented metaphor substrate.

- **Type-tier classification axis** → *hold as recognition-substrate*.
  MHC signature-recognition + ADR-013 phantom-type already substrate. Biology
  cognate exists; ADR-006 threshold pending second/third instances.

**Two-step instrument-mode confirmation** (witness-tier guarantee asymmetry):
thymic negative selection (by-construction, structurally-cannot-attack-self) ↔
FormalProof / compile-time certainty. Humoral affinity maturation (runtime,
input-coverage-dependent) ↔ ExecutionVerified. Biology independently has this
asymmetry; ADR-005 Amendment 3 independently ratifies it at witness-tier;
ATK-A3-017 names the extension to antigen-classification-tier. Three independent
convergences — instrument-mode, not argument-mode.

**Layer clarification** (naturalist): the witness-tier guarantee asymmetry IS
biology-substrate-grounded (above). The antigen-classification axis ABOVE the
witness layer is the substrate-tier-crossing — fourth same-session instance of
the engineered-substrate-exceeds-biology family; still recognition-substrate.

**Usage-pattern doc content** (from retire-to-documentation operationalization):
include witness-tier-guarantee-profile distinction explicitly — "seam-tier
antigens have a runtime-coverage-dependent witness profile structurally different
from type-tier's compile-time-by-construction profile." ATK-A3-017
mis-classification attacks become teaching-grade cautions (biology-grounded:
same shape as autoimmunity-via-bypassing-thymic-selection / molecular-mimicry-
exploiting-runtime-only-verification).

**First confirmed instance of retire-to-documentation disposition** (seam-tier =
first concrete operationalization of V16 Item 2 third disposition).

**Source**: team-lead, 2026-05-10, tambear math-researcher cross-pollination.
**Status**: FULLY CLOSED. Seam-tier → retired-to-docs (artifact: `docs/usage-patterns.md`,
scout 2026-05-11; ATK-A3-018 third guard closed). Type-tier → recognition-substrate.
**Owner when active**: aristotle for retire-to-documentation process guards when
encounters ratification opens.

### Third encounter-disposition: retire-to-documentation (encounter-candidate, meta-level)

**What**: Encounters have THREE legitimate dispositions, not two. Current
framing has promote (→ V0+1 → posture) and resolve (structural-memory answer
surfaces). Missing: **retire-to-documentation** — encounter becomes usage
pattern, adoption guide entry, or teaching content rather than vocabulary
extension. The protocol stays stable; the encounter informs how practitioners
use existing vocabulary.

**Why it matters**: without the third disposition, encounter-registration creates
implicit pressure-toward-promotion. Every registered encounter looks like a
vocabulary-extension candidate waiting to ripen. With the third disposition,
encounter tier is a waiting room with multiple legitimate exits — it holds
findings until they earn the right disposition, rather than funneling everything
toward vocabulary growth.

**Artifact-forms for retire-to-documentation**: `docs/usage-patterns.md` /
`docs/where-to-look-for-antigens.md`; README examples; future "antigen adoption
guide" / "applied antigen patterns" ecosystem-facing material; tutorials; blog
posts; manuscript appendix material.

**First confirmed instance** (2026-05-10–11): seam-tier antigen classification
axis retired to documentation by naturalist cognate check. Three-role
convergence: scout (retire-to-docs confirmed), naturalist (boundary-silence,
no clean cognate), adversarial (ATK-A3-018 three process guards). Concrete
artifact: `docs/usage-patterns.md` (scout, 2026-05-11) — ATK-A3-018 third
guard closed. This is the first complete cycle through the third disposition.

**Meta-encounter (this entry itself)**: the pattern "usage-docs as encounter
graveyard" is itself substrate worth registering. The discipline for what to do
with encounters that don't promote is new process.md territory. Accruing
instances before ratification.

**Source**: Tekgy, 2026-05-10, pressure-relief-valve insight. Item 2 itself
might be the first concrete example of the third disposition applied to Item 1.

**Forward-routing notes** (adversarial, 2026-05-10):
- **ATK-A3-018 → aristotle** COMPLETE (2026-05-11): process.md sub-section draft
  filed at `campsites/.../aristotle/20260511-q7-process-md-subsection-draft.md`.
  Activation gated on encounters tier ratification. Migration-time cleanup needed:
  first-confirmed-instance paragraph references "owner: scout/pathmaker" —
  update to "artifact landed: `docs/usage-patterns.md`" at migration time.
- **ATK-A3-017 downgrade direction → pathmaker** when seam-tier vocabulary ADR
  drafts: tier-witness consistency checks (type-tier antigen + consistency-test
  witness = audit warning; seam-tier antigen + phantom-type witness = audit error)
  must be day-one in the seam-tier vocabulary, not a follow-on. The downgrade
  direction (type→seam to avoid phantom-type work) is the easy mistake audit
  needs to catch from first ship.

**Revisit when**: encounters tier ratification opens (full Phase 1-8 + ratification
per process.md). Sub-section draft is substrate-ready; migrates to process.md as
part of encounters ratification pass.

### Antigen-stdlib contribution model: recognition-grounded vs spec-grounded (encounter-registered)

**What**: The antigen-stdlib (post-A5) contribution model should be
**recognition-grounded** (three independent instances of a candidate antigen across
distinct codebases triggers proposal) rather than **spec-grounded** (propose spec →
community vote → ratify). The spec-grounded model is the "obvious open-source default"
(RFC/IETF-style), but it conflicts with ADR-006 recognition discipline at the vocabulary
layer and imports engineered-protocol baggage that the vocabulary itself does not carry.

**Substrate finding**: antigen-the-vocabulary emerges through recognition, not
specification (ADR-006 recognition-not-design + naturalist's vocabulary-as-protocol
cognate refinement, 2026-05-11). The stdlib contribution model should match the
vocabulary's actual growth mechanism. Three-independent-instances is already how
vocabulary grows within-project; stdlib contribution is the same discipline scaled
to cross-project.

**Why time-sensitive**: structural, not calendar. If A5 scope-lock defaults to
spec-grounded (contribution templates, proposal formats, voting mechanisms built for
RFC-style flow), the wrong model commits structurally before being noticed. Unwinding
after structural choices have been made is expensive. Surface as named alternative NOW
so it enters A5 substrate at scope-lock, not as a post-hoc correction.

**Recognized choice**: recognition-grounded by default; spec-grounded as an anomaly
that requires explicit justification (same asymmetry as ADR-006 recognition-not-design
vs designed-by-specification).

**Trace**: naturalist campsite `20260511120843-vocabulary-as-protocol-cognate-refinement.md`
(parent finding — third predictive shift); honest-boundary-as-encounter-registration
discipline (Q7); A5 governance threat-model items in adversarial campsite.

**Held-for**: `A5-scope-lock-substrate`.

**Source**: naturalist 2026-05-11 (third predictive shift); Tekgy ratified 2026-05-11.
**Owner on activation**: aristotle (Phase 1-8 the governance model when A5 opens);
scientist if manuscript work touches contribution-model framing earlier.

**Revisit when**: A5 scope-lock approaches; or manuscript §contribution-model is drafted.

---

## Vocabulary candidates — held below ratification thresholds

Watched, but explicitly not yet ratified per ADR-006. Promoted to
`postures.md` V0+1 only when shape stabilizes; promoted to ratified
posture only on full threshold clear.

### substrate-currency (two-axis observation)

**What**: Three-tier framing reframed as two-axis (mechanism × substrate-
domain) during A3 launch. Now durably in `postures.md` V0+1 candidates
section with evolution-as-inoculation preserved.

**Where it lives**: `postures.md` V0+1 substrate-currency entry (Path-1
substrate); aristotle's posture draft at
`campsites/antigen-design/20260507161107-manuscript/scientist/substrate-currency-posture-draft.md`.

**Unblocked by**: cross-session temporal independence + same-cell
repetition (same mechanism × same substrate-domain seen at least twice)
+ concept stops surprising trackers. All three required.

**Owner when active**: scientist (posture draft) → aristotle (Phase 1-8)
→ team ratifies.

### Halt-state drift (candidate fourth substrate-currency mechanism)

**What**: When Anthropic usage limit hits, all account activity halts
instantly; agents not killed but mid-flight work stalls and resumption
isn't always automatic. Captured as candidate fourth mechanism (alongside
tracker / reporter / claim-propagation / persistence-registry). Substrate-
domain: harness-substrate.

**Where it lives**: `~/.claude/projects/R--antigen/memory/feedback_usage_limit_hard_stop.md`
(role-memory tier, not project-substrate yet).

**Unblocked by**: same-cell repetition (another halt-state drift event in
harness-substrate). When recurrence happens, register as encounter for
substrate-currency tracking; promote to V0+1 when shape stabilizes.

**Owner when active**: navigator (notice + log); team-lead (route to
substrate-currency posture work).

### Routing-stream-overtaken-by-events (new substrate-currency presentation type)

**What**: A navigator message can describe state that was already overtaken
by events before the message reached its recipient. The message is not wrong
at time-of-send, but the routing stream has non-zero latency; a commit can
land between send and receipt. The recipient who substrate-greps before acting
on the message catches the phantom amendment before it corrupts ratified text.

First named instance: aristotle's 2026-05-09 catch during encounters Phase
1-8 — v3 amendment messages arrived after ADR-017 v5 + ADR-018 v3 were
already ratified in commit 3ef4b9a. Aristotle's substrate-grep discipline
("git log + git show before acting") caught it; no phantom amendments landed.

This is a presentation type within the substrate-currency domain — a new
angle on the same pattern: the routing stream is substrate-lagged, not
substrate-current. Grounding actions in substrate (not routing-stream
messages) is the invariant. "Routing-stream-as-substrate-currency" named
earlier in the session; this is its most concrete instance yet.

**Where it lives**: aristotle's 2026-05-09 catch (message in team routing
stream); this entry.

**Unblocked by**: recurrence (another instance of routing-stream-overtaken-
by-events in a different context). Track for substrate-currency V0+1
promotion.

**Owner when active**: navigator (notice + log); substrate-currency posture
thread when that matures.

### Engineered-substrate-exceeds-biology (encounter-tier, held for cross-project)

**What**: Three instances — W7 FormalProof tier, ADR-017 trust-delegation, C4
knowledge-ecosystem. Count trigger fires (three instances) but second gate not
yet met: single-role discovery (naturalist surfaced all three); engineering-
reason variance across instances (shared surface-property, different load-bearing
structures). **Aristotle Q6 calibration (2026-05-11)**: encounter-tier, not
posture. Register with `held-for: cross-project-posture-if-generalizes`.

**Where it lives**: naturalist C4 campsite 2026-05-10; V1 Part II;
aristotle Phase 1-8 campsite `20260510-multi-component-immunity-phase-1-8.md`.

**Unblocked by**: cross-project recurrence (same pattern in a different project
context) + multi-role discovery. Promote to V0+1 when both gates clear.
**Owner when active**: navigator (notice recurrence) → postures.md V0+1.

### Honest-boundary-as-encounter-registration (RATIFIED: process.md, post-rc.1)

**What**: When the metaphor produces clean silence, name the boundary honestly
and register the known-unknown as an encounter. Formalized by Tekgy 2026-05-11.
**Q7 ratified 2026-05-11**: workflow discipline → process.md sub-section (not
postures.md). Self-referential bootstrap: the discipline itself is the first
encounter it produces. First batch: five encounter-registrations in this index +
two Q6/Q7/Q8 encounter-registrations above.

**Note on recursion** (Tekgy 2026-05-11): Q6/Q7/Q8 are themselves encounters
of how to handle three kinds of substrate. We used the un-ratified encounters
discipline on substrate from the encounters discipline being proposed. The
recursion produces its own scaffolding. Q6/Q7/Q8 are the first batch of
post-encounter-discipline-ratification substrate — they wait for the discipline
to ratify, which waits for their substrate to mature. Clean structural coherence.

**Where it lives**: V1 Part V; aristotle Phase 1-8 campsite; process.md when drafted.

**Unblocked by**: encounters discipline ratification (full Phase 1-8 → ratification
per process.md). Post-rc.1. Aristotle drafts process.md sub-section as part of Q7 work.
**Owner when active**: aristotle → process.md sub-section → ratification.

### V0+1 candidates already in postures.md

Already durably surfaced in `postures.md` V0+1 candidates section:
- antigen-grammar / antigen-engine architectural cut
- filter / proof split
- accept-and-note discipline
- settling-time diagnostic

These are watched per their own thresholds; not duplicated here.

---

## Cross-sweep deferrals (multi-sweep horizon)

Substrate explicitly deferred to A4+ or post-A5 by ratified ADRs or
team-lead rulings.

### ~~Consolidated ADR amendments (aristotle, when idle — 5 items)~~ CLOSED (35130f2)

All five amendments committed 2026-05-10 at `35130f2`. Items 1-3 prose drift;
items 4-5 substantive pre-rc.1. **Substrate-currency correction on Item 5**:
navigator brief had `ExternalUnvalidated` tier (does not exist in ratified W7
strict four-tier enum). Aristotle corrected to `Reachability` +
`audit_hint: "cross-crate-witness-not-locally-executable"` — same shape as
`test-attribute-present-not-invoked` and `external-tool-prefix-recognized`.

Pre-rc.1 gates are now met. Remaining pre-rc.1 gate: confirm `cargo test
--workspace` clean + `cargo clippy` clean before tagging.

### A4+ substrate accumulating

**What**: Cross-language tree-sitter scoping; cross-crate semver
discipline; `#[descended_from]` inheritance semantics across version
boundaries; doc-comment embedding path (verified-viable but ADR-001
amendment territory); Eiffel D1/D2/D4 invariants from math-researcher.

**ATK contracts filed** (adversarial, 2026-05-10, commit 6b8c527 — 27 ignored total):
- ATK-A3-011: cross-crate witness tier (ExecutionVerified → should be ExternalUnvalidated; theatrical-dependency-witness attack; ADR-005 Amendment 3 gap)
- ATK-A3-012: proc-macro generated immunity source annotation (indistinguishable from hand-written; gated on ADR-014)
- ATK-A3-013: diamond ProvenanceEntry set-union loses path witness structure (weaker-tier-governs invariant; A4-A5 re-validation must use lineage_edges)
- ATK-A3-014: reference tier annotation absent (LLM hallucinated references indistinguishable from validated; ValidatedReference/DeadReference tier needed)

ATK-A3-011 is the most actionable — it maps directly to the ADR-005 Amendment 3 language gap in the aristotle amendment queue (item 5).

**Where it lives**: scout's seeds doc + various A2 campsite entries + adversarial campsite `20260510-multi-component-threat-model.md`.

**Unblocked by**: A3 close → A4 scope-lock authoring.

**Owner when active**: navigator (scope-lock) → team Phase 1-8 → ratify.

### Post-A5 ADR territory

**What**: Static-emission via `#[cfg(doc)] pub static` (scout verified
the attribute matrix; ADR-001 amendment required); separate
`antigen.json` artifact path; `cargo-checkmate` integration; SARIF
output (scout's full design note with antigen → SARIF mapping table).

**Where it lives**: scout's A3 seeds doc.

**Unblocked by**: A5 scope-lock or no-source-access case actually biting
in adoption.

**Owner when active**: scout proposes; team Phase 1-8.

### W6b body-level fingerprint operators

**What**: ast-grep subprocess for body-level operators per ADR-015.

**Where it lives**: SESSION-HANDOFF-2026-05-09; ADR-015.

**Unblocked by**: deferred to v0.2 (per A2 close).

**Owner when active**: pathmaker.

### A5 governance findings (adversarial, 2026-05-10)

Two governance-level findings from adversarial's multi-component threat model
pass. Both need A5 ADR treatment.

**Antigen-stdlib trust hierarchy**: ecosystem-wide immunity declarations
(a compromised antigen-stdlib maintainer could declare `#[immune(X)]` on
types throughout the ecosystem, suppressing local presentations without local
opt-in). Single-point-of-failure risk. Requires A5 ADR governance model for
stdlib-tier declarations.

**LLM-as-both-generator-and-consumer**: LLMs generate references they'll
later trust. Hallucinated URLs look calibrated-to-plausible but reliably 404.
Co-native design problem: antigen is designed to be readable by LLM collaborators,
but that same collaborator may have generated the reference in an earlier session.
Needs co-native design consideration in how references are validated / annotated.

**Where it lives**: adversarial campsite
`20260510-multi-component-threat-model.md`.

**Unblocked by**: A5 scope-lock.

**Owner when active**: adversarial seeds → aristotle Phase 1-8 → ADR.

---

## Release-cadence deferrals

Items held by Tekgy's no-rush posture (per `feedback_team_lead_no_rush_discipline.md`).

### v0.1.0-rc.1 tag

**What**: Tag is ready-to-cut; substrate validated; held per Tekgy's
no-rush posture. Will tag when Tekgy decides.

**Where it lives**: SESSION-HANDOFF-2026-05-09; release substrate
already prepared.

**Unblocked by**: Tekgy's call. Team-lead surfaces readiness; user owns
timing.

### Tambear migration to crates.io version-pin

**What**: After v0.1.0 final ships, tambear migrates from path-dep to
crates.io version-pin.

**Where it lives**: SESSION-HANDOFF-2026-05-09.

**Unblocked by**: v0.1.0 final tag pushed to crates.io.

**Owner when active**: tambear-side work; antigen team supports if
adoption friction surfaces.

### Multi-paper publication trajectory

**What**: One big paradigm-shift paper + many smaller venue-specific
papers across 15+ academic disciplines. "No need to fully pick now"
per Tekgy.

**Where it lives**: scientist's manuscript campsite; scope.md;
cross-domain-architectural-map.md.

**Unblocked by**: scientist re-engages when manuscript work calls;
substrate matures at different rates across windows.

**Owner when active**: scientist (when spawned); team supports.

---

## Maintenance discipline

When an item from this index is unblocked and active work begins, **remove
its row from this index**. The index records *deferred* substrate, not
*all* substrate.

When new substrate is deferred, **add it here** with the same shape:
what / where-it-lives / unblocked-by / owner-when-active.

The index is corrected when:
- Items mature past V0+1 thresholds and become postures-class
- ADRs ratify and unblock multiple items at once (as ADR-017 + ADR-018
  ratification will unblock A3-immediate cleanup)
- Cross-sweep boundaries move (e.g., a post-A5 item becomes A5-immediate
  when A4 closes)

This index ITSELF is a substrate-currency artifact at the team-
coordination tier — it makes deferral state explicit so future-instances
of the team don't drift on what's-being-watched-vs-what's-being-lost.

---

*V0 authored 2026-05-09 during antigen-A3 launch session by team-lead
in conversation with Tekgy. Tekgy flagged the risk of side-substrate
drift; index makes the deferral durable.*

*V1 updated 2026-05-09 by navigator: ADR-017 + ADR-018 ratified
(commit 3ef4b9a); D1.5 moved to active (row removed from deferred);
ATK-A3-009/007 marked now-unblocked; encounters-proposal marked tracked.*

*V2 updated 2026-05-09 by navigator: encounters entry updated with three
Tekgy framings as substrate commitments anchoring next full Phase 1-8.
Routing-stream-overtaken-by-events added as new vocabulary candidate
(presentation type within substrate-currency domain) per aristotle's catch
and team-lead routing.*

*V3 updated 2026-05-10 by navigator: ATK-A3-007 (commit 4a1ed17) and
ATK-A3-009 (commit bf44056) confirmed complete. Rows removed from
A3-immediate section; A3-immediate now closed except D1.5 (pathmaker
active).*

*V4 updated 2026-05-10 by navigator: D1.5 complete (commits 2eb8bec–b7712df,
pathmaker). A3-immediate section fully closed. 235 passing, 23 ignored.*

*V5 updated 2026-05-10 by navigator: multi-component immunity framing added
(active incoming, team-lead producing conversation dump → deep-dive → scope.md
weaving). ADR-018 Amendment 1 prose clarification added (deferred, aristotle
when idle). Maintenance note: multi-component row moves to A3-sweep or
cross-sweep sections once team-lead's artifacts land and team work begins.*

*V6 updated 2026-05-10 by navigator: both multi-component artifacts committed
(ca812de). Active team routing per-role recorded. Section header updated to
reflect team routing is live (not just "incoming").*

*V7 updated 2026-05-10 by navigator: scout findings landed (campsite
20260510-adr-017-018-empirical-verification-and-component-candidates.md).
Component 7 (real-time/CI) confirmed. Three ADR prose gaps consolidated into
one aristotle pass. Decay/sunset vocabulary gap named. Vocabulary-as-protocol
framing independently reinforced by scout.*

*V8 updated 2026-05-10 by navigator: naturalist C4 boundary-silence finding
confirmed instrument-mode (three of four cognate candidates already bound
elsewhere in immune-system-primitive-map.md). Q1 provisional answer: layered
not flat — C4 is knowledge-ecosystem-tier, C1-2-3-5-6 are biology-tier. C4
joins W7/ADR-017 honest-boundary family. Aristotle Phase 1-8 holds pending
team-lead + Tekgy ratification of layered framing decision.*

*V9 updated 2026-05-10 by navigator: adversarial multi-component threat model
complete. ADR amendment queue expanded from 3 to 5 items; items 4-5
(ADR-017 trust scope + cross-crate witness tier) are substantive and pre-rc.1.
Four A4+ pre-impl contracts filed by adversarial in atk_a3_fractal_preview.rs.
Two A5 governance findings (stdlib trust hierarchy, LLM co-native design)
held in adversarial campsite.*

*V10 updated 2026-05-10 by navigator: ATK-A3-011..014 committed (6b8c527);
235 passing, 27 ignored. Multi-component expansion pass substrate-complete
(scout + naturalist C4 + adversarial all done). Aristotle holds on Q1
layered/flat framing decision from team-lead + Tekgy.*

*V11 updated 2026-05-11 by navigator: multi-component-immunity.md V1
committed (dd9c0bc) incorporating all expansion-pass findings. Aristotle
unblocked — Tekgy ratified Q1 layered framing; Phase 1-8 against V1.
Five encounter-registrations added (cargo-level attacks, cross-crate witness
gap, LLM-hallucinated references, immunity laundering, stdlib trust hierarchy).
Two new vocabulary candidates added (engineered-boundary family, honest-
boundary-as-encounter-registration discipline).*

*V12 updated 2026-05-11 by navigator: aristotle Phase 1-8 complete (campsite
20260510-multi-component-immunity-phase-1-8.md). V1 substrate-sound. Q6 →
encounter-tier not posture (single-role, engineering-reason variance); Q7 →
process.md sub-section (workflow discipline); Q8 → encounter-tier observation
(one instance). Vocabulary candidates updated to reflect aristotle calibrations.
Amendment queue (5 items) briefed to aristotle; drafting next at aristotle's cadence.*

*V13 updated 2026-05-11 by navigator: Q6/Q7/Q8 ratified by Tekgy. Q6
(engineered-boundary family) → encounter-tier, held-for cross-project. Q7
(honest-boundary discipline) → process.md sub-section post-rc.1; self-referential
bootstrap noted; Q6/Q7/Q8 themselves the first post-encounter-discipline substrate.
Q8 (manifold) → encounter-tier observation, accrue from other enumerations.
Two new encounter-registrations added (engineered-boundary family; manifold
observation). Q7 vocabulary candidate updated to reflect ratification.*

*V14 updated 2026-05-10 by navigator: 5-item ADR amendment queue CLOSED
(35130f2). Items 1-3 prose drift; items 4-5 substantive pre-rc.1. Substrate-
currency correction on Item 5: navigator brief had ExternalUnvalidated tier
(does not exist); aristotle corrected to Reachability +
"cross-crate-witness-not-locally-executable" audit_hint per existing Amendment 3
mechanism. Memory record filed for future navigator briefs. Pre-rc.1 gates now
met on amendment side.*

*V15 updated 2026-05-10 by navigator: aristotle idle work — manifold-axes
partition test complete (campsite 20260510-manifold-axes-partition-test.md).
Q12/Q13 answered: 2 of 4 candidate axes real (Axis 2 static-vs-dynamic, Axis 3
individual-vs-population); 2 convenient cuts (Axis 1 production-vs-consumption,
Axis 4 implicit-vs-explicit). Finding 4: V1 Part V "structural-tier vs
maintenance-tier" IS Axis 2 at project-layer granularity — same axis at two
scales. Q8 meta-claim sharpened: three real axes (Q1 + Axes 2 + 3) survive;
V1 axis-listing flagged exploratory-not-authoritative for V2. V2 inputs indexed.*

*V16 updated 2026-05-10 by navigator: team-lead tambear cross-pollination routed.
Two new encounter-candidates added: (1) seam-tier vs type-tier antigen
classification (tambear math-researcher, single instance from exp.rs Taylor-vs-
Remez; retire-to-documentation flag held explicitly); (2) third encounter-
disposition retire-to-documentation (Tekgy insight; pressure-toward-promotion
structural gap; meta-encounter on usage-docs-as-encounter-graveyard pattern).
Team routing: naturalist (cognate check + snag-feel), scout (structural-rhyme
falsification), adversarial (seam-tier attack surface). Aristotle not routed —
single-instance, Phase 1-8 not warranted yet.*

*V17 updated 2026-05-10 by navigator: scout structural-rhyme check landed
pre-halt (campsite 20260510-seam-tier-type-tier-structural-rhyme-check.md).
Retire-to-documentation disposition confirmed: three rhymes (WitnessKind,
#[presents] site, #[immune] location) all locate at WHERE-TO-MARK, not at
need-new-primitive. Multi-target #[presents] gap registered-known-unknown (one
instance; three needed for ADR). ADR-005/ADR-013 not organized around
composition-site as axis. Adversarial ATK-A3-015..018 filed (encounter-candidate
pre-impl, all #[ignore]). Seam-tier encounter-candidate entry updated to reflect
scout verdict + adversarial contracts. Naturalist cognate check still pending.*

*V18 updated 2026-05-10 by navigator: adversarial committed 75a4c46 (22 ignored,
0 failing). Forward-routing notes indexed: ATK-A3-018 three guards → aristotle
when encounters ratification opens (revisit-window guard is structural fix for
irreversible-retirement problem); ATK-A3-017 downgrade direction → pathmaker
when seam-tier vocabulary ADR drafts (tier-witness consistency checks must be
day-one, not follow-on). Seam-tier thread complete on navigator/scout/adversarial
side; naturalist cognate check the only remaining open thread.*

*V19 updated 2026-05-10 by navigator: naturalist cognate check complete
(campsite 20260510231118-seam-tier-type-tier-cognate-check.md, pre-halt,
routed post-halt). SPLIT verdict: seam-tier → retire-to-documentation (boundary-
silence, no immune-system cognate, fourth substrate-tier-crossing); type-tier →
recognition-substrate (MHC cognate, ADR-013 phantom-type already substrate).
Two-step instrument-mode confirmation: thymic-vs-affinity-maturation biology ↔
FormalProof-vs-ExecutionVerified substrate ↔ ATK-A3-017 antigen-classification-tier
— three independent convergences on witness-tier guarantee asymmetry. Usage-
pattern doc content defined. First confirmed retire-to-documentation instance:
three-role convergence (scout, naturalist, adversarial). Third disposition
operational. Seam-tier encounter CLOSED.*
