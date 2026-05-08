# Antigen — Postures

> Informational catalog of the normative architectural postures threaded through
> antigen's ratified ADRs. **This document does not ratify; it recognizes.**
> Each entry points AT a ratified ADR (or amendment) where the posture lives;
> the catalog surfaces them together so a future contributor can ask
> "is X-decision posture-class?" by checking against this list rather than
> rediscovering each posture from substrate.
>
> **Status**: V0 (2026-05-09). Authored by aristotle on Sweep A2 day 2;
> contributions from naturalist (depth-shift discipline) and pathmaker (the
> ratified ADR-005 Amendment 2 + Amendment 3 substrate the catalog draws on).
>
> **Lifecycle relation**: Per [`process.md`](process.md), ratification lives in
> `decisions.md`. Postures do not gate ratification; they are read alongside an
> ADR draft to ask "which ratified posture(s) does this ADR thread through?"
> ADR-006 (recognition-not-design) keeps the catalog honest: an entry lands here
> only after the posture is already operationally present in the substrate;
> postures.md does not introduce postures, it surfaces them.
>
> **Two grounding paths satisfy the entry threshold**:
> 1. **Direct ratification** — a specific ADR or amendment authoritatively
>    states the posture (postures §1–§6 each cite one). The "Where ratified"
>    section names the authoritative source.
> 2. **Operational presence with ADR-006 threshold** — the posture is not
>    separately ratified but is operationally present across three+ independent
>    instances in substrate, with the discipline satisfying ADR-006 itself.
>    The "Where ratified" section names the most load-bearing canonical
>    example (an ADR that exemplifies the posture in its Finding) and
>    enumerates the substrate instances. Posture §7 (depth-shift discipline)
>    is the V0 instance of this path.
>
> Both paths are legitimate; the asymmetry is intentional. Path 2 exists
> because some postures emerge as cross-cutting recognition before any single
> ADR is the right home for ratifying them as a normative rule. When a
> Path-2 posture eventually accrues a dedicated ADR, the entry's "Where
> ratified" section is updated to Path 1.
>
> **Vocabulary lock**: every term used here is anchored in
> [`glossary.md`](glossary.md). When a posture's wording drifts between this
> file and an ADR, the ADR is authoritative; this file is corrected.

---

## How to read an entry

Each posture is presented in the same five-part form:

1. **Posture** — the rule, stated normatively.
2. **Why** — the structural reason the rule is load-bearing.
3. **Where ratified** — the ADR (or amendment) where the rule lives. This is
   the authoritative source; if this catalog and the cited ADR diverge, the
   ADR wins.
4. **Recognition examples** — substrate-grounded instances showing the rule in
   operation. Future-readers can verify the catalog isn't inventing the
   posture by checking these references.
5. **How to apply** — the operational question to ask when drafting an ADR or
   reviewing a design decision. Each posture is generative: it instructs
   future work, not just describes past work.

When all five parts hold, the posture is doing the work this catalog claims it
does. When any part decays (for example: the cited ADR is amended in a way that
breaks the posture's wording), the catalog is corrected.

---

## Index

1. [Sub-clause F at every trust boundary](#1-sub-clause-f-at-every-trust-boundary)
2. [Recognition, not design](#2-recognition-not-design)
3. [Compose, don't compete](#3-compose-dont-compete)
4. [Anti-YAGNI: structurally-guaranteed need](#4-anti-yagni-structurally-guaranteed-need)
5. [Implicit-to-explicit elevation](#5-implicit-to-explicit-elevation)
6. [Rationale-as-required-field](#6-rationale-as-required-field)
7. [Depth-shift discipline](#7-depth-shift-discipline)

---

## 1. Sub-clause F at every trust boundary

### Posture

> Every trust boundary requires a validation check before trust is extended.
> An asserted claim must be canonicalized and validated by the receiving
> system before it is acted upon. Where the validation cannot be performed
> at the claimed strength, the receiving system reports the strength of
> verification it actually performed — never a stronger one.

### Why

When the claim "this immunity is well-formed" or "this witness exists" is
acted on without validation, the immune system is poisoned: a `#[immune(X,
witness = Y)]` whose `Y` is missing, mis-resolved, or under-verified becomes
the new "trust me" comment — exactly the implicit-memory failure mode antigen
exists to surface (per ADR-001). The discipline is foundational: every
recognition surface in the project is a trust boundary, so the posture
threads transversally through nearly every ADR that introduces a primitive.

### Where ratified

[ADR-005 — Sub-clause F at every trust boundary](decisions.md#adr-005--sub-clause-f-at-every-trust-boundary)
(foundational; pre-team).

Two amendments extend the discipline to specific surfaces:
- [ADR-005 Amendment 2 — Rationale-as-required-field as transverse sub-clause F discipline](decisions.md#adr-005-amendment-2--rationale-as-required-field-as-transverse-sub-clause-f-discipline)
  applies the posture at the API surface (every trust-extending primitive).
  See posture §6.
- [ADR-005 Amendment 3 — Audit reports its own tier honestly](decisions.md#adr-005-amendment-3--audit-reports-its-own-tier-honestly)
  applies the posture at the audit reporting surface.

### Recognition examples

- **Immunity-claim trust boundary** (ADR-005 Decision item 1): `cargo antigen
  scan` validates that `witness = Y` resolves to a real test/proptest/proof/
  lint and that the function is in scope.
- **Inheritance-propagation trust boundary** (ADR-005 Decision item 2):
  `cargo antigen scan` walks `#[descended_from]` chains and re-checks that
  inherited witnesses still apply to descendants. (A3-prospective; contracts
  filed at `antigen/tests/atk_a3_fractal_preview.rs`.)
- **Audit-reporting trust boundary** (ADR-005 Amendment 3): the audit's
  status output (`is_well_formed()`, `WitnessStatus`, `witness_tier` in
  JSON) reports the tier its verification work actually supports — not a
  stronger one. Five confirmed adversarial-grounded violations
  (ATK-A2-003/004/005/011/012) motivate the amendment.
- **Crash-resistance at recognition surfaces** (ADR-005 Amendment 3
  Mechanics §3): `cargo antigen audit` MUST not crash on
  legitimate-but-pathological in-domain input. A crashing recognition
  mechanism extends trust *unconditionally* — a strictly stronger sub-clause
  F violation than silent over-claiming.

### How to apply

When drafting an ADR that introduces a primitive: ask **where is the trust
boundary, and what validates it?** If the primitive causes downstream
tooling/auditors/consumers to act differently because the primitive is
present, it extends trust. The ADR must name the validation check (or
explicitly waive with documented reasoning, per Amendment 2). When the
validation cannot be performed at full strength, the ADR specifies how the
surface reports the actual strength — not the maximal one.

---

## 2. Recognition, not design

### Posture

> When uncertain whether to design something or recognize something, lean
> toward recognition. New antigens, new witness types, new composition rules
> are added when they recognize existing structure in the substrate — not
> when they extend the design speculatively. The threshold for ratifying a
> recognition: three independent instances in substrate.

### Why

Antigen is fundamentally a recognition project. It does not invent
failure-classes; it names patterns that already exist in real-world Rust
codebases. The discipline guards against speculative-feature drift and
top-down design where features get added without empirical grounding. It
sets the right epistemic posture: when a proposed antigen feels speculative,
the question is "is there a real structural pattern this recognizes?" not
"should we add this to the design?"

### Where ratified

[ADR-006 — Recognition, not design](decisions.md#adr-006--recognition-not-design)
(foundational; pre-team).

ADR-007 (anti-YAGNI) is recognition's structurally-guaranteed counterweight
(see posture §4): some features must be built before instances surface
because the structure of the design commits to them.

### Recognition examples

- **Stdlib antigens require multiple in-the-wild instances**: per ADR-006
  Mechanics, every antigen in `antigen-stdlib` must have its source pattern
  documented. The two A5-prospective stdlib candidates
  (`PolarityInvertedClassMeet`, `PanickingInDrop`) carry three+ documented
  instances each — the pattern is recognized, not designed.
- **Three-window pattern as recognition shape**: ADR-003's defense rests on
  three independent windows (biology, past-self gardening, academic CS
  lineage) converging on the same architecture. The "three independent
  instances" threshold is the operational form of the discipline.
- **ADR-005 Amendment 3 cleared the threshold structurally**: five
  adversarial-confirmed instances + five `TODO(team)` markers in
  `antigen/src/` already enumerating the same gaps + biology cognate
  (B-cell affinity-maturation tier hierarchy) = three windows, threshold
  satisfied.
- **Team-lead's ADR-006 invocation as scope-limiter**: per Amendment 3
  Open Question 1, the broader generalization to all recognition mechanisms
  outside audit stays below threshold until three independent instances
  surface outside audit. The discipline is enforced *bidirectionally* — it
  prevents premature broadening as well as premature inclusion.

### How to apply

When drafting an ADR that proposes a new primitive or stdlib antigen, ask
**what is being recognized? show the instances**. Speculative-feature drift
is caught at this question. If the answer is "we might need it," the
proposal is YAGNI unless ADR-007 (posture §4) explicitly grants the
structural-guarantee. If the answer is "three independent instances exist
in substrate," the threshold is satisfied.

---

## 3. Compose, don't compete

### Posture

> Antigen composes existing Rust ecosystem tools rather than competing with
> them. Witness types DELEGATE to existing tools wherever possible. Every
> API decision filters through "are we composing or competing?"

### Why

Reinventing existing tools (clippy, proptest, kani, prusti, verus, creusot)
is wasteful and strategically wrong: it would fragment the ecosystem,
duplicate engineering, and miss the ecosystem-of-mature-tools advantage Rust
already has. The composition posture is what makes antigen viable as an
ecosystem-spanning vocabulary rather than a yet-another-lint.

### Where ratified

[ADR-002 — Compose, don't compete](decisions.md#adr-002--compose-dont-compete)
(foundational; pre-team).

The discipline extends through:
- [ADR-013 — phantom-type witness recognition + witness-validity tier mapping](decisions.md#adr-013--adr-002-amendment-1-phantom-type-witness-recognition--witness-validity-tier-mapping)
  formally amends ADR-002 to recognize phantom-type witnesses (witness-axis).
- [ADR-015 — Fingerprint engine: grammar-over-AST with per-fingerprint evaluator trait](decisions.md#adr-015--fingerprint-engine-grammar-over-ast-with-per-fingerprint-evaluator-trait)
  operationalises compose-not-compete at the *engine* axis (it amends
  ADR-010, not ADR-002 directly): each fingerprint engine (syn, ast-grep,
  future MIR/HIR/runtime) is a delegation-boundary; the evaluator trait
  abstracts.

### Recognition examples

- **Witness recognition delegates** (ADR-002 Decision; ADR-013): the
  `witness` parameter on `#[immune(...)]` accepts test, proptest, clippy
  lint, kani/prusti/verus/creusot proof, phantom-type construction, or
  antigen-native witness. `audit.rs::detect_external_tool` recognizes
  prefix-based external tools (`clippy::`, `kani::`, `prusti::`, etc.) by
  delegation — antigen does not validate the tool's correctness; it
  delegates the validation to the tool.
- **Fingerprint-engine delegation** (ADR-015): the fingerprint grammar is
  evaluator-trait-abstracted so syn (item-level operators) and ast-grep
  (body-level operators) compose under one fingerprint without antigen
  reinventing either. v0.1 ships syn natively + ast-grep via subprocess.
- **Naturalist's silence-where-silent test** (ADR-003 defense in A1
  closure): of the six in-flight A1 primitives, five had direct biological
  predecessors and one (`#[antigen_generates]`) was pure Rust-grain with no
  biological analog. The metaphor's *silence where it should be silent* is
  the same shape as compose-not-compete's silence: where a tool already
  occupies the surface, antigen stays silent.

### How to apply

When proposing a witness type, fingerprint operator, or recognition
mechanism: ask **does an existing tool already do this work?** If yes, the
new primitive must justify why it's not a thin delegation. If a tool exists
but its surface doesn't fit, propose an evaluator-trait abstraction that
admits the existing tool and the new one (per ADR-015's pattern). The
default: composition. The exception: documented reasoning per ADR-002
Enforcement.

---

## 4. Anti-YAGNI: structurally-guaranteed need

### Posture

> When the project's structural commitments guarantee a feature will be
> needed, build it now. Features that the design's principles guarantee
> will be needed are built upfront; features that are merely "might be
> useful" are deferred. The discipline is the counterweight to
> recognition-not-design (posture §2): some features must precede instances
> because the structure forces them.

### Why

Mainstream YAGNI is correct in many contexts, but it has a load-bearing
inversion: when the project's structural commitments guarantee a feature
will be needed, building it later (when the structure forces the issue)
incurs retrofit cost. Shipping without structurally-guaranteed features
fragments the design and creates incoherence between commitments and
implementation.

### Where ratified

[ADR-007 — Anti-YAGNI: structurally-guaranteed need](decisions.md#adr-007--anti-yagni-structurally-guaranteed-need)
(foundational; pre-team).

### Recognition examples

- **All four witness families ship in v0.1, not a subset** (ADR-007
  Sweep-level consequences): test, proptest, formal-verification, lint —
  no version that ships only some. ADR-013 completes the witness-type
  commitment by adding phantom-type recognition.
- **All 8 first-principles failure-classes are committed** (ADR-007
  Sweep-level consequences): not all 8 will have immediate stdlib
  instances, but all 8 are guaranteed-needed by the taxonomy. A2-day-1's
  recognition that `PolarityInvertedClassMeet` and `PanickingInDrop` are
  *prospective* stdlib members rather than V0.1 mandatory exemplifies the
  discipline's interaction with recognition-not-design (§2): structure
  commits us to all 8; recognition holds the threshold for which 8 ship
  first.
- **`#[descended_from]` ships with full propagation logic, not stubbed**
  (ADR-007 Sweep-level consequences). A3-prospective ATK contracts at
  `antigen/tests/atk_a3_fractal_preview.rs` lock the predictions against
  the implementation before A3 opens — the structurally-guaranteed need
  surfaces *as predicted bugs* before the work-stream opens.
- **`cargo antigen vaccinate` ships in v1, not v2** (ADR-007 Sweep-level
  consequences): committed in A5 because the fingerprint-grammar v1
  surface (W6) and the witness-type completeness (W7) make vaccination
  mechanically composable from the parts; deferring to v2 would force
  retrofit when the substrate already supports it.

### How to apply

When proposing to defer a feature, ask **what other ADR commits us to
this?** If a ratified ADR commits us, the feature is structurally
guaranteed and deferring incurs retrofit cost. If no ratified ADR commits
us and the feature is "might be useful," it is YAGNI. The aristotle role
owns the structurally-guaranteed-need analysis at sweep-planning time
(ADR-007 Mechanics).

---

## 5. Implicit-to-explicit elevation

### Posture

> Every design decision is evaluated against: does this make implicit
> structure explicit, or does it preserve implicit-mode obscurity? When
> the design forces work to flow through explicit declarations, it is
> doing the elevation correctly. When the design accepts implicit
> conventions ("everyone knows this is fragile"), it is falling back to
> implicit-mode.

### Why

Antigen is one specific application of the deepest fold operation a project
can perform: making structural what is implicit. Each elevation
(sequential→parallel, value→reference, concrete→symbolic, implicit→explicit)
makes new work possible while elevating the boundary that was preventing it.
Failure-class memory has been implicit in human/agent memory; antigen makes
it structural and explicit in the type system.

The cost of explicit-mode is forced pacing, more typing, slower velocity
per-line. The benefit is legibility — to future agents (Claude or human),
to fresh-context teams, to cross-project consumers.

### Where ratified

[ADR-004 — Implicit-to-explicit elevation as architectural posture](decisions.md#adr-004--implicit-to-explicit-elevation-as-architectural-posture)
(foundational; pre-team).

### Recognition examples

- **The four core macros enact elevation at the carrier surface**:
  `#[antigen]` makes failure-class names explicit; `#[presents]` makes
  vulnerable code sites explicit; `#[immune]` makes immunity claims with
  their witnesses explicit; `#[descended_from]` makes inheritance
  propagation explicit. Each replaces an implicit convention (implicit
  awareness, implicit "everyone knows," implicit verbal mentorship).
- **ADR-001 Amendment 1 Change 3 enumerates implicit commitments
  C1–C8 explicitly**: the amendment is *structural-forcing* — the project
  was already committed to all C1-C8 by other ADRs (007, 005, 010); ADR-001
  needed to enumerate them rather than leaving them implicit. The
  amendment itself is an elevation of implicit commitments to explicit
  structural ones.
- **ADR-005 Amendment 3 elevates implicit tier-encoding to explicit**:
  the audit's status word ("Resolved", "well-formed") implicitly encoded a
  verification-tier claim. The amendment makes the encoding *explicit* via
  the `witness_tier` + `audit_hint` fields. The implicit pattern becomes a
  named API surface.
- **ADR-template "implicit pattern being elevated" is a required section**
  (ADR-004 Enforcement): every ADR explicitly names the implicit pattern
  it replaces. Recursion-through: the discipline applies to its own
  ratification process.

### How to apply

When drafting any ADR or API surface, ask **what implicit pattern does this
elevate?** If the answer is "none — it's a new feature with no implicit
predecessor," consider whether the feature is YAGNI (posture §4) or
recognition-not-design (posture §2). If the answer names an implicit
convention being made structural, the elevation is doing real work. The
ADR-template's required "implicit pattern being elevated" section makes
this question structural at draft time.

---

## 6. Rationale-as-required-field

### Posture

> When an ADR ratifies a primitive (attribute macro, configuration field,
> declaration form) that extends trust, the primitive MUST carry a
> justification field (named `rationale`, `summary`, `references`,
> `witness`, or an ADR-specific equivalent) by default. Waivers require
> explicit ADR-level reasoning.

### Why

Sub-clause F at every trust boundary (posture §1) governs the validation
*check*; this posture governs the *rationale for extending trust in the
first place*. The asymmetry without this posture: ADR-005 names per-boundary
validation but does not name per-primitive justification. The substrate
exhibits the principle across nearly every primitive — the carrier set has
consistently grown justification fields as trust-extending power
accumulated. The posture promotes the observation from a property-of-the-
carriers (per ADR-001 Amendment 1 Change 7) to an operational-discipline-
of-trust-boundaries.

### Where ratified

[ADR-005 Amendment 2 — Rationale-as-required-field as transverse sub-clause F discipline](decisions.md#adr-005-amendment-2--rationale-as-required-field-as-transverse-sub-clause-f-discipline)
(2026-05-09).

Originating observation: ADR-001 Amendment 1 Change 7. The amendment
elevates the observation from property-of-carriers to
operational-discipline-of-boundaries — same fact, different lens.

### Recognition examples

Five+ manifestations across the carrier set, each a justification field
attached to a trust-extending primitive (per ADR-005 Amendment 2 Finding
table):

- `#[antigen(name = "...", summary = "...")]` — `summary` is the Layer 1
  human-readable description (ADR-009).
- `#[antigen(..., references = [...])]` — `references` is the Layer 2
  open-vocabulary list of CVE/RFC/ADR/URL pointers (ADR-009).
- `#[immune(X, witness = Y)]` — `witness` is the executable rationale
  (ADR-001 / ADR-002 / ADR-005).
- `#[immune(X, witness = Y, rationale = "...")]` — `rationale` is the
  narrative justification supplementing the executable witness (ADR-001
  Amendment 1 Change 7).
- `#[antigen_tolerance(X, rationale = "...")]` — `rationale` is required
  at parse time; tolerance without rationale is rejected (ADR-011).
- `#[antigen_generates(X, rationale = "...")]` — `rationale` is required;
  the macro author justifies the generation pattern (ADR-014).
- `#[antigen(..., evidence = [...])]` (ADR-016) — `evidence` is the
  justification field for temporal trust extensions.

The discipline propagates from existing ADRs to new ADRs (ADR-011, ADR-014,
ADR-016) without explicit coordination — that is how a load-bearing
principle should behave.

### How to apply

When drafting an ADR that introduces an attribute, config field, or
declaration form: ask **does this primitive extend trust? if yes, name the
justification field, OR document why one is not required**. Ratification
cannot proceed without an answer. The default flips with this posture in
place: new trust-extending primitives without justification fields require
active argument; primitives with justification fields are the unmarked
default. Adversarial seeds against new primitives include
"justification-field-missing" as a default attack pattern (per Amendment 2
Enforcement).

---

## 7. Depth-shift discipline

### Posture

> The load-bearing structural commitment lives one tier deeper than the
> visible decision. Before drafting, deconstructing, or rejecting any
> proposal, ask: "what is the X−1 commitment that determines whether X
> works?" Apply the question to the answer; the discipline is operationally
> self-producing — there is no fixed point.

### Why

Multiple A1–A2 deconstructions surfaced the same shape: the visible decision
(grammar operators, engine choice, "should `#[immune]` claims propagate
across `#[descended_from]`?", "should ExternalUnvalidated be a fifth tier?",
"should `is_well_formed` return true for empty bodies?") dissolved or
restructured once the load-bearing X−1 commitment was named (the
grammar/vocabulary cut, the evaluator-trait architecture, "substantive claims
require explicit re-attestation at each trust boundary," "what does
WitnessTier measure," "the audit's reporting surface IS a trust boundary").

This finding is structurally distinct from
recognition-not-design (posture §2) and biology-as-search-heuristic
(closure-narrative material): those are descriptive — they explain why a
move was correct in retrospect. Depth-shift discipline is *generative* — it
instructs future decisions before deconstruction surfaces them. Eight+
independent instances across two roles' substrates as of A2 day 2.

The headline property: the discipline is *operationally self-producing*.
Each application of substrate-honesty creates the conditions for the next
tier of substrate-honesty to surface; each tier is the verifier-of-the-
previous-tier; each tier is one Phase 8 deeper than the prior. The
operational signature is **no fixed point**.

The rhyme to biology-as-search-heuristic: both are instances of the same
structural move — *probe one tier deeper than the visible surface; the
load-bearing structure lives there*. Biology-as-heuristic does this for
failure-mode discovery (audit reports tier-N+1 while doing tier-N work).
Depth-shift does this for design-decision identification (ADR claims to be
about X; ask "what's the X−1 commitment that determines whether X works?").
Different operational layers, structurally identical move.

### Where ratified

The most load-bearing canonical example is
[ADR-005 Amendment 3 — Audit reports its own tier honestly](decisions.md#adr-005-amendment-3--audit-reports-its-own-tier-honestly)
(2026-05-09), Finding section: *"the visible question was 'should
`is_well_formed` return true for empty bodies?' The load-bearing commitment
was 'the audit's reporting surface IS a trust boundary' — once that
commitment is named, the empty-body case becomes one of five instances of
the underlying violation."*

Two further amendments are themselves depth-shift instances:
[ADR-015](decisions.md#adr-015--fingerprint-engine-grammar-over-ast-with-per-fingerprint-evaluator-trait)
(visible: ast-grep vs syn engine choice; load-bearing: the evaluator-trait
architecture) and
[ADR-016](decisions.md#adr-016--temporal-recognition-surface-provenance--freshness-primitives-for-stale-context-and-premature-abstraction)
(visible: temporal field set; load-bearing: `verified_at` granularity as
trust-boundary commitment).

The posture is not separately ratified — per the catalog's contract
(ADR-006), it surfaces what is already operationally present. The
ratification-by-trust threshold (three independent instances) is overshot
by the eight+ confirmed instances; ADR-006 is satisfied.

### Recognition examples

Substrate-grounded instances, each pairing a visible decision with the X−1
commitment that determined whether the visible decision worked:

1. **ADR-010 reciprocal Phase 1-8** (aristotle, A1): visible decision —
   *which fingerprint grammar operators to ship in v1*. Load-bearing
   commitment — *the grammar/vocabulary cut itself*.
2. **ADR-015 v0 self-deconstruction** (math-researcher, A2): visible
   decision — *engine choice (ast-grep vs syn)*. Load-bearing commitment
   — *the evaluator-trait architecture abstracting both engines*.
3. **`#[immune]` × `#[descended_from]` convergence-check** (math-researcher,
   A2): visible decision — *do `#[immune]` claims propagate across
   `#[descended_from]`?* Load-bearing commitment — *substantive claims
   require explicit re-attestation at each trust boundary* (already
   expressed in Eiffel D1/D2/D4 + ADR-005 sub-clause F + ADR-011's
   rationale-required field).
4. **ADR-015 external addendum** (aristotle, A2): visible decision —
   *backend-choice details*. Load-bearing commitment — *trait-architecture
   as the design surface, not the backend*.
5. **W7 tier-design Phase 1-8** (aristotle, A2): visible decision —
   *should ExternalUnvalidated be a fifth tier?* Load-bearing commitment
   — *what does `WitnessTier` measure (confirmed-current vs
   potential-maximum)?*
6. **ADR-005 Amendment 3 motivating Finding** (naturalist, A2): visible
   decision — *should `is_well_formed` return true for empty bodies?*
   Load-bearing commitment — *the audit's reporting surface IS a trust
   boundary*.
7. **ADR-016 `verified_at` granularity** (scout, A2): visible decision —
   *what fields go on the temporal surface?* Load-bearing commitment —
   *`verified_at` granularity as trust-boundary commitment*.
8. **Verification-of-verification** (aristotle, A2 day-2): visible decision
   — *is the v1 ratification verdict ready?* Load-bearing commitment —
   *is the verification rubber-stamping?* Phase 8 self-applied to the
   verifier's own verdict surfaced three findings; one warranted edit.

The structural-identity test (math-researcher, 2026-05-09) confirmed all
seven tiers as the *same pattern* across all instances, not just rhyming
patterns at different scales:

| Identity criterion | T1 code | T2 team-coord | T3 researcher | T4 self-application | T5 substrate-correction | T6 Phase-8-self-applied | T7 verifier-self-correction |
|---|---|---|---|---|---|---|---|
| Same fail-mode | yes | yes | yes | yes | yes | yes | yes |
| Same recovery shape | yes | yes | yes | yes | yes | yes | yes |
| Same routing pattern | yes | yes | yes | yes | yes | yes | yes |

Seven instances; three identity criteria; all yes. Structural identity, not
analogy at a different scale.

### How to apply

When drafting any ADR, deconstructing a proposal, or running Phase 8:
ask **"what is the X−1 commitment that determines whether X works?"** Apply
the question to the answer. Apply *to your own application of the
discipline* — there is no fixed point; that is the operational signature.

The catch is at three operational layers:

- **Drafting** — before ratifying X, name the X−1 commitment; if the X−1
  commitment is not yet ratified, the X-draft may be stating a consequence
  rather than the load-bearing decision.
- **Deconstruction** — Phase 1's "assumption autopsy" should include the
  visible-decision-vs-load-bearing-commitment cut explicitly, not as a
  catch-all "list every assumption."
- **Forced rejection (Phase 8)** — apply Phase 8 to the verification verdict
  itself, not just to the proposal. Verification-of-verification is the
  tier where rubber-stamping is caught.

The discipline's structural sibling is biology-as-search-heuristic:
biology recurses into substrate one tier below the visible question for
failure-mode discovery; depth-shift recurses into substrate one tier below
the visible question for design-decision identification. Both are
instances of the same recursion-into-substrate move at different
operational layers.

---

## What postures.md is NOT

To prevent scope creep:

- **Not a ratification surface**. ADRs ratify; postures.md surfaces what is
  already ratified. New posture entries land here only after the cited ADR
  (or amendment) is itself ratified per [`process.md`](process.md). Per
  ADR-006 (recognition-not-design), the catalog does not introduce postures
  speculatively.
- **Not authoritative when ADR text and posture text diverge**. The cited
  ADR is the source of truth; this catalog is corrected when the divergence
  is detected.
- **Not a working-agreement document**. Working agreements (campsite
  conventions, role definitions, navigator-routing topology) live in
  [`process.md`](process.md) and the team-briefing. Postures are
  *architectural* normative orientations, not procedural ones.
- **Not exhaustive**. A V0+1 entry is queued: the antigen-grammar /
  antigen-engine architectural cut (per glossary entry; ADR-015 + ADR-013
  + ADR-016 convergence). It lands here when the substrate-confirmation
  threshold (per ADR-006) is satisfied. Other postures may surface from
  A2/A3 substrate.

---

## V0+1 candidates

Surfaced from substrate during A2; awaiting threshold confirmation per
ADR-006:

- **antigen-grammar / antigen-engine architectural cut** — the boundary
  between antigen-grammar and external substrate as the design surface;
  the implementation across that boundary is delegation, not reinvention
  (per glossary entry; convergence of ADR-015 evaluator-trait + ADR-013
  phantom-witness recognition + ADR-016 temporal-axis).
- **filter / proof split** — fingerprints filter (recall-tuned candidate
  filters); witnesses prove (precision lives in the witness layer). False
  positives from the filter are EXPECTED and not failure states.
  Operationally ratified at ADR-010 Amendment 4; candidate for posture-
  class promotion when a third-axis instance surfaces outside the
  fingerprint/witness pair.
- **accept-and-note discipline** — when a macro arg-parser receives a field
  whose audit-side check is not yet implemented, the parser MUST accept
  the field with an explicit known-limitation note rather than silently
  accept (sub-clause F violation) or reject (forward-compat block). One
  ratified ADR-016 instance + one A2 verified_at instance; awaiting third.

- **substrate-currency** — the temporal sub-pattern of substrate-over-memory:
  substrate-as-of-author-time ≠ substrate-as-of-consumer-time; a claim or
  finding is only authoritative if verified against the substrate at the
  time of consumption, not the time of authorship. Three distinct instances
  at three operational layers (tracker-tier, reporter-tier, claim-propagation
  tier) confirmed in A2 day-2. Held below ratification: three instances at
  three *different* layers signals concept still extending into new territory,
  not repeating at known layers. Ratification trigger: same-layer repetition.
  Currently travels at role-memory tier (typed feedback files in agent roles)
  — the right carrier for an active team-coordination discipline. Glossary
  sub-bullet under substrate-over-memory when cross-session accumulation
  confirms shape stability.
- **settling-time diagnostic** — the criterion "concepts that stop surprising
  the people tracking them are ready to freeze; concepts still surprising
  trackers are not ready." Surfaced as the operative ratification test for
  substrate-currency (CLOSURE.md:357-370, A2 day-2). Plausibly posture-class
  in its own right as a vocabulary-maturation discipline generative beyond any
  single concept. One clean in-session instance (substrate-currency itself);
  awaiting cross-session confirmation that the diagnostic generalizes.

These are not posture-class today. They are catalogued so future-readers
know what is being watched and what threshold is outstanding.

---

*V0 authored 2026-05-09. Maintained by aristotle (catalog drafting +
ratification-citation discipline) with substrate contributions from the
A2 team. Subject to revision when cited ADRs are amended or when V0+1
candidates clear ADR-006 threshold.*
