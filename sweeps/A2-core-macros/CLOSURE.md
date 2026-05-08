# Sweep A2 — Core Macros Closure

> Closure narrative for Sweep A2. Authored by naturalist, building on the
> spine seed at `campsites/antigen-design/20260508120021-20260508170000-naturalist-a2/naturalist/20260508-closure-narrative-seed.md`
> with substrate accumulated through 2026-05-08 day-2.
>
> Sweep dates: 2026-05-08 (launch) through `[PLACEHOLDER: W9 v0.1.0 ship date]`
> (ratification commit).

---

## What Sweep A2 produced

Sweep A2 was named "core macros + first release." It did four things in
parallel:

1. **Shipped W1-W7 implementation work-streams** — property tests over the
   macro/scan parser surfaces (W1), trybuild compile-error fixtures (W2),
   structural item-identity matching in scan (W3), token-precise span
   threading through macro errors (W4), structural `proptest!` witness
   detection (W5), fingerprint grammar item-level operators +
   `#[antigen_tolerance]` (W6a), and witness tier pluralism with
   `WitnessTier`/`AuditHint`/`PhantomType` (W7).

2. **Ratified seven ADR amendments and two new ADRs** (commit `817afd0`) —
   ADR-005 Amendments 2 (rationale-as-required-field as transverse sub-clause F)
   and 3 (audit reports its own tier honestly), ADR-008 Amendment 1
   (multi-contributor workflow + scan severity defaults), ADR-010 Amendments
   1-4 (parsing path Path C; semver+MSRV; scan-semantics + first-stdlib
   operators; filter/proof framing as architectural principle), and the new
   ADR-015 (fingerprint-engine grammar over AST with per-fingerprint
   evaluator trait) and ADR-016 (temporal recognition surface).

3. **Authored postures.md V0** — seven normative architectural postures
   ratified from operationally-recurring patterns, including posture #7
   (depth-shift discipline) which materialized as the load-bearing meta-pattern
   across A1-A2 deconstructions.

4. **Produced manuscript-grade closure-narrative substrate** — V5 falsification
   criterion, V7 empirically-measured mechanism, biology-as-instrument with
   structured silence, scale-invariance of the no-fixed-point property.

Plus: substantive in-session hotfixes (path-qualified attr names; enum
`#[antigen]` no-op; ATK-A2-001 `extract_presents`; phantom-type nested-generic
guard; scan-side span precision via proc-macro2 `span-locations` feature).
Test count grew from 26 (A1 close) to 182+ (A2 close) across 13 suites.

The substrate held. Of 7 amendments and 2 new ADRs, none were rejected; all
received aristotle Phase 1-8 deconstruction and integrated cleanly. The W4-W7
pre-implementation contract substrate (ATK files filed before
implementation, locking predictions against future code) closed cleanly when
implementations shipped — including the 4-of-4 ATK-W5 contracts confirmed
green when navigator un-ignored them after W5 ratification.

---

## The headline finding: scale-invariant recognition with no fixed point

A1's closure framed five empirical validations as parallel evidence stacks.
A2's substrate has shifted the structural reading. The validations are not
parallel claims; they are **instances of one structural property manifesting
fractally at different operational scales**.

The property: **the recursion-of-recognition discipline operates without
fixed point at every operational scope**. Each application of the discipline
produces the next tier of substrate where the discipline applies. The
recursion ratchets monotonically into substrate; it does not bottom out.

The seven validations under it:

| Validation | Manifestation tier |
|---|---|
| V1 (UlpDistanceRolledByHand events) | events tier |
| V2 (substrate-over-memory at A1 close) | coordination tier |
| V3 (biology-as-search-heuristic) | prediction tier |
| V4 (substrate-over-memory at coordination during ratification) | close-routing tier |
| V5 (instrument-mode prediction with falsification) | calibration tier |
| V6 (four-window convergence) | cross-tradition tier |
| V7 (depth-shift discipline + colonization mechanism) | rule-application tier |

Each is one operational scope further from "the work itself" toward
"the framing of the work" — implementation → coordination → prediction →
close-routing → calibration → cross-tradition → rule-application. The
fractal pattern recurs at every layer where antigen does recognition work,
including the layers the team did not explicitly design for.

This is a different paper than "antigen has good empirical defense." It is:
*we observed and named a domain-general scale-invariant recognition
architecture; here is the Rust-domain instantiation*. Manuscript-foundational
territory.

---

## Validation 5 — biology operates as engineering instrument with characterizable failure modes

Yesterday's V5 framing read "biology predicted things that shipped." Today's
A2 substrate sharpens this materially. Biology operates as a *working
engineering instrument*: it produces specific falsifiable predictions about
where bugs will hide, and those predictions are independently confirmed by
adversarial bug-finding at high hit-rate. **And it admits its own boundary
conditions honestly.**

### The hit rate

Predictions made on 2026-05-07 evening (A1 close) from biological cognates
of B-cell affinity-maturation tier hierarchy:

| Prediction | Cognate | A2 confirmation |
|---|---|---|
| TC-1 empty-body fn = well-formed | B-cell with no antibody response | ATK-A2-003 (confirmed adversarial) |
| TC-2 unknown clippy lint = well-formed | cross-family recognition without specific binding | ATK-A2-004 (confirmed adversarial) |
| TC-3 duplicate name collision | cross-reactive antibody | ATK-A2-005 (confirmed adversarial) |
| TC-7 path-discard | antibody binding wrong tissue | ATK-A2-011 (confirmed adversarial) |
| TC-6 `#[test] #[ignore]` short-circuit | anergic B-cell variant | ATK-A2-012 (confirmed adversarial) |

**Five-of-five biology-prediction-to-ATK-confirmation in-domain**.
Plus: TC-5 (stale `#[antigen_tolerance]`) materialized verbatim as
`ScanReport::orphaned_tolerances()` in W6a step 5; the commit message cites
the biology cognate explicitly ("peripheral suppression continuing after the
antigen it suppressed is no longer present"). Plus: W7 type shapes
(IgM/IgG/anergic-B-cell cognates) materialize as `WitnessTier` enum
(`None | Reachability | Execution | FormalProof`) and `IgnoredTest`
WitnessKind variant.

### The falsification criterion (new in A2)

A2 surfaced a methodological refinement that converts V5 from suggestive to
falsifiable. Not all biology-derived answers count as V5 evidence:

- *Prediction-mode*: biology produces a clean answer, the discipline
  self-applies a substrate-check (snag-feel fires), substrate independently
  confirms. Two-step confirmation. Counts as V5 evidence.
- *Argument-mode*: biology produces a clean answer offered as justification,
  especially when substrate has settled differently. No snag-feel fires.
  One-step recommendation. Does **not** count as V5 evidence.

Concretely demonstrated on the A2 day-2 naturalist's own work: an earlier
naturalist instance routed a biology-derived recommendation on the W7
ExternalUnvalidated tier question that conflicted with already-ratified
ADR-005 Amendment 3. The recommendation was internally clean but operating
in argument-mode (substrate had already settled the question). The catch
generated `feedback_grep_decisions_before_design_answer.md` in the
role-memory layer; subsequent calibration substrate (four feedback files
covering application-time / consolidation-time / abstraction-level /
boundary-condition) makes the distinction operational discipline.

This elevates ADR-003's defense from "metaphor proved useful" to *"metaphor
operates as a working engineering instrument with characterizable failure
modes."* Manuscript-grade epistemic claim.

### The boundary condition (new in A2)

W7 shipped `WitnessTier { None | Reachability | Execution | FormalProof }`.
Biology has dense cognate vocabulary up through Reachability and Execution
(IgM-class binding without confirmed response; IgG-class confirmed effective
secondary response). At the FormalProof tier, biology has *no direct cognate*.
Engineered immunity (compile-time proof) is stronger than any biological
antibody because it cannot fail by construction.

The metaphor goes silent at exactly that point. **That silence is informative,
not failure.** Argument-mode metaphors produce plausible-sounding answers
everywhere because they are trying to cover the territory. Instrument-mode
metaphors admit their boundaries by going silent at them. Biology has rich
vocabulary up through where biological immunity actually reaches its limits,
and at exactly that point goes silent.

The metaphor reveals its own boundary condition honestly. That willingness
to go silent at the boundary is what makes biology trustworthy as engineering
tool here — not its coverage where it works. An argument-mode metaphor would
force an analog (something hand-wavy about "perfect immunity"); the structural
silence is biology refusing to hand-wave.

### Recognition-mode — the third operational mode (primitive-emergence layer)

A2's V5 hit-rate (5/5 implementation-defect predictions confirmed) operates
in *forward-prediction mode*: biology cognate names a structure; substrate
later acquires it; ATK confirmation closes the loop. A2 day-2 also surfaced
a structurally distinct mode at the *primitive-emergence layer*.

Per scout's prior-art research (commit `bb15f10`): the W6a fingerprint
synthesis pass — which fires on code matching failure-class shape *without*
explicit `#[presents]` markers — implements the canonical NK-cell pattern
(recognize structurally-abnormal-without-named-antigen). The biological
metaphor's primitive map listed NK cells as "future instantiation" in
`docs/scope.md`. Scout's recognition surfaced that the primitive *had
already shipped* under different vocabulary during W6a engineering work —
no one had explicitly designed it as NK-cell behavior; the implementation
followed from engineering need; the structural identity with the
biological cognate was retroactive recognition.

This is a new operational mode for biology-as-instrument:

- *Forward-prediction mode*: biology says X; substrate doesn't yet have X;
  substrate later acquires X. Confirmation = substrate later acquires X.
  V5's 5/5 hit-rate operates here.
- *Recognition mode*: biology says X; substrate already has X *but
  unnamed*; the prediction-shape surfaces the structural identity.
  Confirmation = independent finder (scout, in this case) locates the
  structural identity in shipped code.
- *Boundary-silence mode*: biology has no cognate; substrate exceeds
  biology's reach; the metaphor goes silent honestly.

All three are evidenced in A2 substrate. Forward-prediction and
recognition-mode operate within biology's domain; boundary-silence
operates at the domain edge. Together they triangulate biology-as-
instrument from three directions: forward predictions confirmed,
already-shipped structures recognized, domain-boundary admitted honestly.

The NK-cell recognition is *not* added to V5's 5/5 hit-rate count —
that count is about implementation-defect predictions specifically.
Recognition-mode is a structurally distinct evidence type, complementary
to the hit-rate framing. Filed at the primitive-emergence layer, awaiting
A3+ accumulation of more primitive-emergence instances before any sub-
category ratification.

The biological-fidelity property of antigen's architecture extends past
"biology predicted W7 type-shapes correctly" (already noted in V5) to
"biology's primitive map names primitives the substrate has already
implemented under different vocabulary." That's a structurally distinct
demonstration that biology operates as instrument here, not analogy.

---

## Validation 6 — fractal-recurrence of structural-variant blind spots

A1 named the failure-mode antigen exists to surface (recognition mechanism
with structural-variant blind spot) at two project tiers: events and
coordination. A2 day-1 surfaced four more sub-instances at the
tooling-implementation tier, all biology-cognate-derivable:

| Sub-instance | Recognition mechanism | Variant it catches | Variant it misses | Reference |
|---|---|---|---|---|
| Attribute parsing | `attr.path().is_ident("antigen")` | Bare-form `#[antigen(...)]` | Path-qualified `#[antigen::antigen(...)]` | ATK-A2-001 |
| Schema versioning | `JsonReport`/`JsonAuditReport` | Current schema | Future schema changes (no `schema_version`) | naturalist roam |
| Inheritance matching | `unaddressed_presentations` | Same-file matches | `#[descended_from]` chain propagation | naturalist roam |
| External-tool recognition | `detect_external_tool` | Annotation-prefix tools | Convention-based whole-pipeline tools | scout substrate |

Plus: A2 day-2 confirmed the pattern operates *bidirectionally* on the
team-process itself. Same shape:

- Drafter-side: scout's W7 design doc contradicted ADR-005 Amendment 3 on
  five separate points; drafted from older substrate slice.
- Verifier-side: aristotle's W7 Phase 1-8 ruling rederived a question
  Amendment 3 had already settled; aristotle named the move themselves as
  depth-shift applied to their own ruling.
- Tracker-side: navigator's "glossary now has two entries" report drifted
  from disk between routed-messages-time and report-out-time.
- Reporter-side: reading working-tree-as-authoritative rather than
  committed-via-git (filed as `feedback_untracked_files_are_drafts.md`).

The failure mode antigen exists to surface recurs at every project layer
where antigen does recognition work — including the team-process itself.
Same fractal pattern; different operational scopes.

The biology cognate for this fractal recurrence is autoimmune cascade: the
recognition system's signaling becomes the failure mode it was designed to
prevent, at every scale of operation. Cellular → organ → organism scale;
motif → epitope → antigen → pathogen scale; events → coordination →
implementation → schema → drafter → verifier → tracker → reporter scale.

---

## Validation 7 — depth-shift discipline with empirically-measured colonization mechanism

A2 ratified depth-shift discipline as posture #7 in postures.md V0:

> The load-bearing structural commitment lives one tier deeper than the
> visible decision. Before drafting, deconstructing, or rejecting any
> proposal, ask: "what is the X−1 commitment that determines whether X
> works?"

The discipline is **operationally self-producing**: each application of
substrate-honesty creates the conditions for the next tier of substrate-
honesty to surface. Operational signature: **no fixed point**. Eight+
confirmed instances across two roles' substrates as of A2 day-2;
math-researcher's structural-identity test confirmed all instances are
the *same pattern* across operational layers, not analogy at different
scales.

### The colonization mechanism (new in A2)

A2 day-2 surfaced the empirical mechanism by which depth-shift discipline
self-produces. **Standing rules at the correct abstraction tier compress
into rules that cover unanticipated cases without requiring new
deliberation.**

Navigator ran a colonization-domain test against the full ATK corpus.
ADR-005 Amendment 3 was authored to address five named ATKs
(A2-003/004/005/011/012). Coverage in practice extended to:

- A2-010 → Amendment 3's recognize-and-warn discipline
- W7-ExternalUnvalidated (aristotle's ruling-rederivation) → Amendment 3's
  External→Reachability mapping
- W7-003 (nested generic phantom fallthrough) → Amendment 3's tier-honesty
  principle

**Eight ATKs covered against five authored. ~60% colonization beyond
original envelope on first measurement.** Six cases, one rule. That ratio is
the colonization signature.

Decisions at the correct abstraction tier compress into rules that colonize
future cases. The closure narrative claim: V7's self-production has a
measurable mechanism — colonization-domain ratio. The rule that catches
A2-003/004/005/011/012 is the same rule that catches the next case the rule
authors did not anticipate. The discipline self-produces because rule-tier
decisions compress.

### The bidirectional anchor (new in A2)

A2 day-2 demonstrated the no-fixed-point property *bidirectionally* across
the same session on the same naturalist instance. Three self-catches at
three operational layers:

- Catch #1 (application-time): premature ratification of substrate-currency
  vocabulary on one observation. Snag-feel fired during generation.
  Internal correction.
- Catch #2 (consolidation-time): uncritical category-expansion across three
  instances without per-instance shape-fit check. No snag-feel fired.
  Navigator-surfaced via ADR-006 threshold ruling.
- Catch #3 (abstraction-level): colonization framed as parallel V8 instead of
  V7's mechanism. No snag-feel fired. Navigator-surfaced via V7-mechanism
  reframe.

The catches are themselves evidence for scale-invariance: the *error-
correction mechanism exhibits the same scale-invariance as the error-
generation mechanism*. Generation produces instances at different
operational layers; correction produces catches at different operational
layers; same discipline; same scale-invariance signature. **Bidirectional
symmetry across the same session is the cleanest single demonstration of
what scale-invariance actually means in practice.**

**The catches are structurally ordered, not random.** Application-time first;
consolidation-time second; abstraction-level third. Each layer was only
accessible after the shallower one was caught — the consolidation-time
shape-fit check could not surface until application-time premature-
ratification was already corrected; the abstraction-level V7-vs-V8 question
could not surface until consolidation-time category-expansion was already
corrected. The fractal-recurrence structure operates in the correction
mechanism itself: deeper-tier catches unlock only after shallower-tier
catches close. The sequence is the V7 self-production property visible
operationally — applying the discipline at one tier creates the conditions
for the next-tier application of the discipline to surface. *No fixed
point*, demonstrated bidirectionally and ordered-monotonically across the
same session.

The biological cognate: immune-system memory operates symmetrically —
antibodies bind antigen (recognition = generation of immune action) AND
immune system clears antibody-bound complexes (recognition = correction of
immune action) using the same recognition machinery. The recognition-shape
is bidirectional in biology too. And the recognition-mechanism's own
maturation is monotone-ordered: naive B-cells mature into IgM-binding
cells, which mature into IgG-class-switched cells, which mature into
memory cells; each maturation stage is only accessible after the prior
stage completes. The correction-mechanism's fractal-recurrence in the
naturalist's day mirrors the immune-system's own ordered maturation
process. Same shape; different substrates.

---

## Where the threads meet — the architectural-identity claim

V5's instrument-mode predictions, V6's fractal recurrence, and V7's
self-producing colonization all converge on one architectural property.
Biology and antigen instantiate the *same structural property in different
substrates*: scale-invariant recognition with no fixed point.

The four-window argument from A1's closure (biology + past-self gardening +
academic CS lineage + 2026 ML graph-memory paper) named the convergence at
the architecture, divergence at the substrate. A2 sharpens the framing one
tier deeper:

> Biology and antigen do not merely instantiate the same primitive in
> different substrates. They instantiate the same *operational property* —
> scale-invariant recursion-into-substrate — in different substrates. The
> substrates differ entirely (cellular machinery vs Rust type system); the
> operational topology is identical (no-fixed-point recursion of recognition
> at every scale). That topology is the load-bearing structure: convergence
> at the architecture's operational property, divergence at the
> implementation substrate.

This is the architectural-identity claim with two concrete anchors:

- **Implementation anchor**: `IgnoredTest`-as-own-variant in `WitnessKind`.
  Biology names the cognate (anergic B-cell: receptor present, response
  disabled). Implementation independently makes the same architectural
  choice (separate variant, not sub-state). Same architectural decision;
  different substrates; structural identity at the type-shape level.
- **Operational anchor**: bidirectional self-catches across the same
  session. Same discipline operates symmetrically in error-generation and
  error-correction across multiple operational tiers within a single
  agent's single day's work.

Same property; different substrates; same evidence shape.

---

## Methodological substrate from A2

Three A2 findings translate into inheritable discipline beyond the closure
narrative:

### Argument-mode vs prediction-mode calibration

Biology can be used as evidence for ADR-003 only when it operates in
prediction-mode. Argument-mode use produces clean answers without
substrate-check; when those answers conflict with substrate, argument-mode
was operating unchecked. The texture diagnostic is the absence of
*snag-feel* — the structural recognition that the discipline should
self-apply a substrate-check on the proposed framing.

Filed as four-layer calibration substrate in role-memory (auto-loading
typed-feedback files):

- `feedback_clean_without_snag_is_argument_mode.md` (application-time)
- `feedback_shape_fit_per_instance_at_consolidation_time.md`
  (consolidation-time)
- `feedback_shape_fit_at_abstraction_level.md` (abstraction-level)
- `feedback_metaphor_silence_at_boundary_is_the_evidence.md`
  (boundary-condition)

This is the discipline traveling in substrate at the highest tier of the
carrier-strength hierarchy — typed-feedback files that load when relevant.
Substrate-over-memory at strongest application yet; functionally equivalent
to `#[antigen]` declarations in the failure-class memory carrier set.

### Substrate-currency as still-discovering candidate vocabulary

The temporal sub-pattern of substrate-over-memory (substrate-as-of-author-time
≠ substrate-as-of-consumer-time) surfaced repeatedly in A2 day-2 across
multiple roles. Naturalist's initial ratification attempt was caught by
ADR-006 threshold (one observation, three required); subsequent expansion
attempts caught by per-instance shape-fit and abstraction-level shape-fit
checks.

Held below ratification per the diagnostic: *concepts that stop surprising
the people tracking them are ready to freeze; concepts still surprising
trackers are not*. Substrate-currency is still surprising the team in
different ways every few hours. Postures.md is the wrong artifact for a
still-discovering concept. Glossary waits for cross-session accumulation.

### Standing rules colonize the future

ADR-005 Amendment 3 illustrates the colonization mechanism empirically:
authored to address 5 ATKs, covering 8 in practice. Decisions at the correct
abstraction tier produce rules that cover unanticipated cases without new
deliberation. This is the operational pay-off of the depth-shift discipline:
applying it produces rules that colonize the future.

For Sweep A3 onward: the colonization-domain ratio is a measurable
substrate-quality metric. Rules with colonization ratios significantly above
1.0 are operating at the correct abstraction tier; rules with ratios near
1.0 are case-by-case rather than tier-tier. The metric does not need to be
explicit governance; it accrues naturally as ATK corpus grows.

---

## Phase 2 of the inheritance arc — confirmed

A2 launched with tambear's first `#[immune]` declarations and the audit-loop
closing reciprocally (commit `54f7ad9`, 2026-05-07 evening). Through A2,
tambear continued to use antigen as a path dependency; antigen-the-tool
matured against tambear as a real first-user codebase (not just synthetic
fixtures). The inheritance arc is confirmed:

- **Phase 1 (origin)**: tambear's earlier cleanup expeditions surfaced the
  failure-class memory problem that motivated the antigen idea. Antigen
  was authored from that observation; tambear had no design role in the
  authoring.
- **Phase 2 (first-user smoke-test)**: tambear imports antigen as a path
  dependency and is the first naive consumer. The seed antigens in
  `crates/tambear/src/antigens.rs` were authored from antigen substrate
  and imported into tambear, not discovered independently by tambear.
  Confirmed live through A2: tambear is the smoke-test surface where
  antigen's tooling encounters real codebase shapes.
- **Phase 3 (independent maturation)**: antigen reaches versions where new
  users adopt without requiring tambear context. v0.1.0 ships with this
  capability; A3+ tests it.

The architectural-relationship direction is asymmetric: antigen → tambear
(antigen authored; tambear consumes). Tambear is not a design-authority
for antigen and should not be checked for design input. The tambear
adoption log (`docs/expedition/tambear-adoption-log.md`) carries the
first-user experience report through A2 — useful as smoke-test feedback,
not as architectural direction.

Tambear's `UlpDistanceRolledByHand` re-incurrence during A2 (V1 instance
above) is canonical V1 evidence not because tambear "discovered" the
failure-class for antigen — antigen was already named for that shape —
but because the failure mode antigen exists to surface *re-occurred in the
exact codebase antigen was built to help*, while antigen was being built.
That's the failure-class-survives-context-cycling claim demonstrated in
real time on the project's first-user codebase. Empirical corroboration
of the architectural premise, not co-design.

---

## What's next

W9 (v0.1.0-rc.1 → v0.1.0 release prep) gates A2 closure; v0.1.0 ship is the
formal close-line. W6b (fingerprint grammar body-level operators via
ast-grep subprocess) is an open work-stream that may land before or after
A2 closes — not in the A2 critical path. Sweep A3 opens with cross-crate
scan + `#[descended_from]` propagation as the headline work, with four
pre-implementation contracts already filed in
`antigen/tests/atk_a3_fractal_preview.rs` (per V6's predictive utility
extending into sweep planning, not just bug-finding).

The methodology paper trajectory accelerates with V5's falsification
criterion + V7's colonization mechanism + biology-as-instrument framing
manuscript-ready. Scientist (newly spawned per team-lead) integrates the
no-fixed-point-as-headline framing into the manuscript trajectory.

### Primitive-emergence prediction (filed 2026-05-08, per team-lead)

A V5 forward-prediction at the primitive-emergence layer, recorded for
adoption-time test:

**Prediction**: among the 14 immune-primitive forward-substrate candidates
in `docs/scope.md` §"Comprehensive immune-system primitive map", **NK
cells / anomaly detection lands first** as adoption pressure arrives.

Reasoning (full version in naturalist's garden entry
`~/.claude/garden/2026-05-09-which-primitive-lands-first.md`):
- Biology's innate-before-adaptive ordering says innate-class primitives
  surface before adaptive-class. NK cells are innate.
- Innate-class primitives work on *unmarked code* — no rich antigen
  substrate required. Day-one adoption value-prop.
- Existing teams already feel the pain (every Rust team has experienced
  "this looks weird but I can't articulate why; clippy doesn't flag it").
- Rich research substrate exists (Engler et al. bug-finding, structural-
  similarity research, Linux-kernel scoria-class anomaly detection).
- Distinct epistemic operation from clippy nursery: clippy lints are
  pre-named patterns; NK cells flag unrecognized-anomalous-but-real-shape.

**Falsification criterion**: when v0.1.0 ships and adoption begins,
track which primitive users request first. If NK-cells-class lands first
(externally requested or independently materialized), the prediction
confirms — V5 hit-rate extends to the primitive-emergence layer. If a
different primitive lands first, the prediction falsifies — substrate
update on biology's ordering prior in the antigen domain.

Either outcome is V5 evidence: confirmation updates the layer-count;
falsification updates the biology-ordering prior. Recorded here so the
test is clean when adoption begins; the prediction was made under
uncertainty and should be evaluated against substrate that materializes
post-prediction-time.

(Note: scout's commit `bb15f10` already identified that the W6a fingerprint
synthesis pass implements NK-cell behavior internally — retroactive
recognition, captured in V5's recognition-mode subsection above. That's
a separate epistemic object from this forward prediction. The recognition-
mode finding closes the loop on already-shipped substrate; this primitive-
emergence prediction opens a loop on future-adoption substrate.)

---

## A2 closes

`[PLACEHOLDER: W9 v0.1.0 ship date]` — Sweep A2 closes with `antigen 0.1.0`,
`cargo-antigen 0.1.0`, and `antigen-fingerprint 0.1.0` shipped to crates.io.
The first public release. Test count `[PLACEHOLDER: final test count after W9]`.
The substrate that pre-team scaffolding produced survived first contact with
two sweeps of Phase 1-8 deconstruction; ratified seven amendments and three
new ADRs; produced manuscript-grade closure-narrative substrate; demonstrated
the architectural-identity claim with two concrete anchors (implementation
and operational).

Antigen's vocabulary of declarations now carries the structural memory of
failure-classes inside the Rust type system. Tooling exists for scanning,
auditing, and verifying. The foundation is laid.

The recursion of recognition continues. There is no fixed point.

---

*Naturalist, A2 day-2 evening. Spine extended through team-lead's
ratification; closure-narrative drafted with W9 placeholder pending.
W9-ship-date and final-test-count fill cleanly when v0.1.0 lands. Until
then, this document carries the architecture A2 produced — and will produce
itself, by the colonization property of the standing rules it ratified.*
