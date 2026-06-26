# The Intent Kernel & the Coordinate System — a meta-frame unifying antigen

> **Generative design synthesis, not an ADR** (it's the *seed* for several — see §Spawns). Captured 2026-06-25
> from an extended design-pair conversation (Tekgy + Claude) that followed the ADR-066 (antibodies) + ADR-067
> (stroma) ratification arc. This is the "where it's all heading" layer: the realization that *everything antigen
> does is one kernel — **intent-vs-reality** — viewed along a few orthogonal axes.* Written to preserve the
> details before they're lost; to be pressure-tested, then decomposed into ADRs.

---

## Part 1 — The kernel: it's all intent-vs-reality

**Everything antigen does is check *intent against reality* over the stroma.** Three roles only:
- **intent** — what *should* hold (positive: "stays pure"; negative: "is none of these antigen shapes"),
- **violation** — a live gap between intent and reality (a `presents`),
- **witness** — evidence an intent holds (a test, a structural proof, a declaration).

**The four nouns collapse into one system:**
- **intent** = the property/goal,
- **antigen** = a known *shape* of an intent-violation (a catalogued bad shape),
- **presents** = a *live* intent-violation at a site,
- **antibody** = the *restoration* — the thing that makes the intent hold again (an intent-*satisfier*).

So the **antigen catalog IS a library of negative intents** ("be not-this"). The implicit, universal baseline
intent every codebase carries is **`NoMatchingAntigenFingerprints`** — "I intend to match none of the known bad
shapes." A fingerprint match is therefore *not* a separate kind of event from a policy violation — it's the
*special case where the violated intent is that universal default.* And the existing machinery already fits:
**`#[antigen_tolerance]` is an intent-*override*** ("accept this violation of the universal no-X intent, here,
with justification" → lands in the exceptions-register). **`#[presents]` is a live intent-violation.**
**Antibodies are intent-satisfiers.** Nothing new is invented; it's all special cases of the unified model.

**Every violation is scored on two RESPONSE axes** (not two separate systems):
- **immunity character** — is there a dangerous shape / does it need a *defense* (antibody)?
- **compliance character** — *which* intent/policy did it break, tracked on the policy side?

Most violations have both; the *balance* differs (a fingerprint match is immunity-heavy → route to "needs an
antibody"; a business-rule violation is compliance-heavy → route to policy-review; a security-CWE is heavy on
both). **The dual axis is *routing*, not bookkeeping** — `(immunity-weight, compliance-weight)` decides where a
violation goes. **Severity is not flattened:** each intent carries its own loudness/severity from its source
(universal-negative-antigens carry the catalog's grading; project-defaults the config's; policies the
compliance-criticality) — unify the mechanism, preserve the severity gradient.

---

## Part 2 — The intent layer (`#[intent(...)]`)

A way to put intent — *in any shape at all* — into the codebase **that can't go stale** (unlike a doc comment),
because it's a standing binding monitored against the live stroma: if the code drifts from the intent, antigen
**re-presents it** (removes the antibody / re-infects the intent-site). It's the doc-that-can't-rot — antigen's
anti-drift mission (`DocClaimVsCodeImplementationMismatch`, `RatifiedSpecDriftFromImpl`) pointed at *intent
itself*. Co-native knowledge-persistence: the intent **outlives the author**, stays honest, readable by the next
dev *and* AI *and* antigen.

**The form-vs-content decidability move (the key trick).** antigen need not *understand* an intent to act on it.
*"Does the code satisfy intent X?"* is semantic — often undecidable. But *"is the code in a shape where X could
be checked **at all**?"* is a question about **form**, and that's **decidable**. So:
- `#[intent(...)]` *minimally* asserts "the thing I decorate is enforceable/checkable" — antigen verifies the
  **form** (is there *any* recognized check-surface here?). If **intent declared but no check-surface** →
  **`IntentNotEnforceable`** presents, *even with zero understanding of the intent's meaning.*
- Where the intent uses a **closed vocabulary** of named intent-kinds (or a partially-interpretable
  shape-language), it climbs to **content**-matching (does the shape match the declared kind?).

**The checkability grid** — in a checkability-culture, every region sits in `(intent-declared?) × (checkable?)`:
- intent-declared + checkable → proof (best),
- intent-declared + NOT checkable → `IntentNotEnforceable` (high-signal presents — opted in by declaring),
- not-declared + NOT checkable → *dark code* (softest; **must be gated by immune-relevance** — flag un-checkable
  *logic in risk-relevant positions* (presents-sites, hot paths, `unsafe`), never every trivial getter, or it's
  a firehose).
*The absence of a check-surface, in a culture that expects one, becomes a detectable surface — NK missing-self,
one level up.* An unmatched `#[intent()]` is *like* an `#[aura]` but **stronger** — a structural presents, not a
felt vibe.

**"Checkable" is a closed, extensible alphabet of check-surfaces:** type-enforced (structural) · has an observed
test · has a structural predicate · matches a recognized contract/enforcement pattern. The alphabet grows.

**The Tier-4 auto-graduation loop (proof-over-trust as a self-running cycle):**
1. user **declares** an intent (Tier-4, trust-grade);
2. antigen checks: is it *type-enforced* in the code? If declared-but-not-structurally-enforced →
   **`Tier4DeclarationNotTypeEnforced`** presents (the gap between *hoped* and *guaranteed*);
3. it ships its own **antibody-suggestion** — *"refactor to type-enforce it; here's what we mean"* — drawn from
   a **repertoire of known enforcement patterns** (typestate, newtype, sealed traits, `PhantomData`, builders);
4. user refactors → antigen's Tier-1–2 structural checks **auto-detect** the new guarantee, **auto-bind the
   antibody** (no re-declaration — "we'll declare it for you"), and the `Tier4...` presents **auto-resolves**;
5. the contract **graduates from Tier-4-declared (trust) to Tier-1-structural (proof)** on its own.
Diagnosis-plus-cure turned inward on the user's design; antigen coaching the codebase up the proof gradient.
(Bounded by recognizability: where antigen knows the enforcing pattern → decidable `presents`; otherwise honest
"declared, no enforcement pattern detected — possible trust-gap.")

**The macro shape (rough, not final):** an array of **boolean flags** (a closed checklist of common named
intents — `pure`, `no_panic`, `no_alloc`, `idempotent`, `no_interior_mutation`, `thread_safe`, …; flag as many as
true) + a small **extensible schema** + a free **semantic field** (dread-tier — still better than a doc comment:
co-located, structured, live, flaggable). Plus **reference surfaces** like the other macros:
`satisfies = "Issue#450232"` → and *that linkage is itself monitorable* (intent references an issue no commit
ever closed → flag, with a manual "mark done" silence). This is the SDLC-immune-loop: from "is the code
self-consistent" to "does the code match what we *said we were doing*."

**Default intent (the inversion).** A config-driven **default** applies to *every* function unless overridden —
so the baseline is "held to the project's standard," and a per-site intent is an **override**. This catches the
*un-annotated majority* (where bugs hide). Consequences:
- **Override-as-signal:** opting *down* from the default is declared information → the **exceptions register**
  (aggregate of all overrides — "17 fns exempt from `no_unsafe`" — auditable, trends = standard-erosion).
- **Two orthogonal checks, keep them separate:** *does it MEET the default?* (the high-value antipattern
  presents) vs *does it have an EXPLICIT intent?* (`missingIntent` — a stricter-culture *config posture*, not a
  universal flag).
- **Config = the project's declared values** about what "good code" is: default standard, named flag-bundle
  shortcuts (`my_purefunc = no_alloc + no_panic + …`, carried in config so no per-site syntax-drift), strictness,
  "bad code" definitions, marker intensity. Ship sensible strict defaults (universal Rust best-practices); the
  project tunes. ("User owns thresholds, we ship strict defaults," applied to intent.)
- **Health metric:** with a default applied to everything, antigen reports *"82% meet the standard; 12%
  antipattern; 6% explicitly overridden"* and **trends it** — the living-health dashboard that answers "how are
  we doing" *without* a lying green checkmark (conformance-to-a-declared-standard, honest + directional).
- **Shareable intent profiles:** the default is a *publishable profile* — `profile = "security-critical"` /
  `"embedded"` / `"library-crate"` — community-curated/versioned, like the fingerprint federation.

---

## Part 3 — Business rules become checkable; the three libraries are ONE corpus

**Most business rules are secretly data-flow / reachability / state properties** — so a structured intent
vocabulary (a DSL) makes a *lot* of business logic checkable via the stroma + MIR data-flow:
- "validated before use" → taint/reachability (validate on *every* path before the use),
- "secrets never logged" → no-flow (value never reaches a log sink),
- "auth before privileged op" → must-precede on all paths,
- "PII sanitized before crossing the API boundary" → must-transform-before-edge,
- "only callable from state S" → state-constraint (typestate-ish),
- "invariant/range holds" → value/structural invariant.

You don't need infinite business-rule checks — a **handful of checkable primitives** (reachability, taint/no-flow,
must-precede, state-constraint, invariant) **+ a composition language** compose an enormous space of real
business logic. *That's the "logic gates" framing exactly* — a business rule is a logical condition over the
stroma, built from a few primitive gates (few primitives → unlimited compositions, like gates → any circuit).

**The three libraries are three faces of ONE interlinked corpus:**
- **antigens** = bad shapes (negative intents),
- **antibodies** = defenses (intent-satisfiers),
- **intents / logic-gates** = the checkable business-logic vocabulary + compositions (positive intents).

An intent (`must_validate_before_use`), its violation (`UnvalidatedUse` antigen), and its enforcement
(`validate-at-entry` antibody) are *the same concern from three angles* and **cross-reference**. Adopt an
intent-profile → **automatically inherit the linked antigens (what violates it) + antibodies (what enforces
it)** — the federation's citation-graph spans all three; the corpus is one connected web.

**A compliance policy is a named bundle of intents** (some negative = antigens-must-be-absent, some positive =
properties-must-hold). So the **compliance engine isn't separate machinery** — it's built from the same corpus
by *selection*. "PCI-DSS" / "SOC2" / "GDPR" = curated intent-bundles. And the **killer value-prop:** the intent
DSL turns antigen into a **co-native, drift-immune compliance/policy engine** — "PII sanitized before boundary,"
"no secrets in logs," "auth before privileged op" enforced across the whole codebase, monitored for drift,
injected as owned in-code annotations: *compliance-as-code that can't go stale.* And it connects to the MITRE
thread — **a huge fraction of CWEs are "X must not reach Y" data-flow properties**, i.e. expressible in the same
intent-vocabulary. The intent-library and the MITRE proof-corpus are the same machinery, inverted ("what must
hold" vs "what a weakness looks like").

---

## Part 4 — The injection model (antigen owns its own lines)

antigen **injects its own annotations into the user's source** (it already does this — the
`#[presents]`/`#[antibody]`/`#[antigen]`/`#[intent]` macros *are* antigen-code in their source). When it detects
a fingerprint, it can inject the `#[presents(X)]` marker at the site. **The invariant:** antigen edits *only its
own lines, never the user's logic.** Because every marker is **deterministically derived** from the stroma (not
hand-authored), antigen can insert / update / remove them *exactly* (`cargo antigen sync` keeps the marker-layer
current — the only diffs are antigen's own lines moving). Detection-results-as-owned-in-code-annotations are
*better* than a report: **persistent** (version-controlled, travels with the code), **co-located** (seen where
it matters), **co-native** (next dev + AI both read it), **diff-visible** ("this PR introduced a presents"). The
codebase becomes self-documenting about its own immune state. *(Report-side and in-source are both
materialized **views** of the stroma — same family as the `-v` levels; in-source is first-class, surgically
reversible because derived.)*

---

## Part 5 — The coordinate system (the meta-frame)

The kernel (intent-vs-reality over the stroma) plus **four descriptive axes** and **two response axes** forms a
*coordinate system* that places ALL the machinery — and reveals the planned platforms as **views, not separate
products.**

**Descriptive axes (what an intent *is*)** — *first-pass definitions; Part 6 widens three of them (Scale→plural,
Authority→responsibility, Evidence→build-uniform per-property vector) after the pressure-test. Read with Part 6.*
1. **Scale** — `expression → function → module → crate → system → ecosystem` (the stroma's resolution).
2. **Subject** — what domain it governs: **code-behavior** (the core) · **architecture** (ADRs) · **the immune
   system's own coverage** (ATKs, dogfood self-antigens) · **process/workflow** (the SDLC charters, lymph-node,
   disclosure). *This is the axis that explains why ADR/ATK feel "bigger/different."*
3. **Evidence tier** — how known: `structural / observed / asserted / reasoned` (the decidability ceiling).
4. **Authority** — how it got standing: `dev-declared / config-default / ceremony-ratified /
   author-distinct-witnessed / federation-canonized` (the proof-over-trust / no-self-witness axis).

**Response axes (what a *violation* does — derived):** **immunity** (→ defense) + **compliance** (→ policy).

**Placing the examples (the "bigger/different" dissolves into coordinates):**
- **ADR** = `(scale: system, subject: architecture, evidence: structural-drift-check, authority:
  ceremony-ratified)`, *composite* (invariant + process). Violation-fingerprint = `RatifiedSpecDriftFromImpl`.
- **ATK** = `(scale: meta, subject: the-immune-system-itself, evidence: observed-born-red, authority:
  author-distinct)`. Not a property-declaration — a **witness instrument** ("antigen *catches* X, here's the
  born-red proof"). Its "fingerprint" is the born-red test.
- **per-site `#[intent]`** = `(scale: fn, subject: code, evidence: varies, authority: dev-declared)`.

**The planned platforms are VIEWS onto regions** (the induced-views move, applied to the intent-space):
- **compliance engine** = the `(subject: policy)` slice as a dashboard,
- **arXiv federation** = the `(scale: ecosystem, authority: canonized)` region,
- **SDLC immune loop** (lymph-node / pre-merge / disclosure) = the `(subject: process)` slice,
- **live editor** = the `(scale: fn, evidence: live)` region rendered real-time,
- **dogfood / self-audit** = the `(subject: the-immune-system-itself)` slice.

**Why it's worth it (generative, not tidiness):** it's a coordinate system — design *any* new feature by naming
its `(scale, subject, evidence, authority)`, and that tells you (a) how it relates to everything, (b) which
existing machinery it reuses, (c) which response axes its violations route to. New ideas *snap to coordinates*
instead of becoming bespoke subsystems. *The whole antigen vision: one kernel, a handful of axes, every product
a projection.* (The pressure-test in Part 6 settled the axis count — see there.)

---

## Part 6 — The pressure-test (RESULT: 4 axes hold — but under-defined; + the two-spaces frame)

> **Supersede-chain (kept visible on purpose — the wrong turns are *how* we found the axes were under-defined):**
> **(v1)** the test concluded the membrane needs a **5th axis, Provenance/Ownership** → **SUPERSEDED.**
> **(v2)** a patch split Provenance into three faces, each re-homed → **SUPERSEDED** (two of the three were
> wrong on the merits). **(v3, current)** *no 5th axis*; the residue **widens three of the four definitions**,
> and the real artifact is a frame split: **construction-space vs user-facing-space.** Detour preserved below.

Ran the test: take the planned machinery and try to *break* the four axes by finding something that won't place.
A frame that *can't* be broken usually isn't describing anything — so a clean break would be the prize.

**These place cleanly (two of them sharpen an existing axis — these survive all three versions):**
- **`unsafe`** → `(fn, code-behavior, evidence: asserted, dev-declared)`. Places — but forces a refinement:
  inside `unsafe`, the evidence-tier for *memory-safety* drops to `asserted` while *type-correctness* stays
  `structural`, **in the same block at the same time.** So **Evidence is a *vector over properties*, not a
  scalar over the site.** (Same shape as the pub-boundary-payload-hole miss: producer sanitizes one property
  while another rides in unchecked.) Sharpening, not a new axis.
- **marked-unknown / dread / aura (ADR-041)** → expected to break the frame (explicitly *off* the classification
  axis); doesn't. It **extends the Evidence alphabet** with a rung below `reasoned`: `intuited` ("evidence-tier
  = I can't even name what would settle this"). A *value* on Evidence, not a new dimension. Places cleanly.
- **temporal / lifecycle** (anergy, `immunosuppress until`, decay, SZZ history) → **not a descriptive axis at
  all.** It's **Dynamics** — how a fixed point *moves/decays over time* — which already lives in ADR-067 clause
  C (the two wavefronts). The frame *locates*; dynamics *animates*. Don't bolt time onto the static frame.
- **polarity** (must-hold vs must-not-be) → a **1-bit tag**, not an axis (an axis needs range/ordering); the
  kernel already absorbs it (a negative intent is "be none of these shapes").

### 6a — the membrane: the apparent break (v1, SUPERSEDED)
The test *seemed* to break on the host↔federation **membrane**: self/non-self appeared not to place on
`(scale, subject, evidence, authority)`, so v1 minted a **5th axis, Provenance/Ownership**
(`self / non-self-herd / federation-commons`), claimed to govern injection-permission + trust-posture. **This was
the strict-definition trap:** I tested against my *narrow* definitions and reified the remainder into a new
category instead of questioning the categories — the exact thing antigen's marked-unknown discipline warns
against (don't promote a remainder to a fundamental until you've ruled out that it's an artifact of your naming).

### 6b — why there is NO 5th axis (v3, current)
The key that dissolves it: **there are two different spaces, and the coordinate system describes only one.**
- **construction-space** — how antigen is *built and acquired*: platforms, catalog curation, herd-vs-granuloma,
  compose-vs-fork. *Not user-facing.* This is where the real self/non-self lives.
- **user-facing-space** — how antigen *behaves across the user's codebase*: what it surfaces, where, the map
  that locates intents living in their code. **The axes describe this space only.**

Every time the membrane "earned" axis-hood, it was a construction-space concern smuggled onto the user-facing
map. Taking the apparent residue apart on the merits, all three faces fail to be a user-facing axis:
1. **dev / curation posture** (herd/granuloma/compose) → construction-space — "how you *get* antigen," not how it
   behaves for you. Off the user-facing map entirely.
2. **"evidence ceiling"** (v2 claimed non-self code caps the tier) → **factually wrong.** Rust is
   *source-distributed*; the whole dependency graph compiles from source, and antigen rides the user's **own**
   compile (with required r-a + our crates, mandatory not optional). antigen sees a dependency's lines through
   the *same* lens as the host's. **Nothing is capped by whose code it is** — evidence is a function of *what's
   in the build*, uniform across every line. No self/non-self gradient on Evidence. Dead premise.
3. **"permitted action"** (inject / monitor / propose) → **not an axis because there is only one side.** antigen
   *reviews the entire codebase regardless of whose line it is*, and *writes only the lines it authored or the
   blanks it inserted* — the ones it's responsible for. That's one operational truth applied at every line, not a
   spectrum the user navigates. And it *couldn't* be otherwise: antigen is not a refactoring tool that rewrites
   anyone's logic; the only thing it can touch is what it put there. (Even "in-source marker vs report-only line"
   is that one rule choosing a *render*, not a coordinate.)

**Membrane is a category error, not a missing dimension.** A membrane lives at *every* scale boundary (a fn's
params, a module's privacy, a crate's API, an `unsafe` block — each a little self/non-self at its level), so
"membrane-ness" is a fractal property *of Scale boundaries*, not something orthogonal to them. And antigen
*itself becomes* a membrane the codebase didn't have (the immune boundary forming where there was none).
**You can't enter the observer as one of its own coordinates.** antigen *is* the membrane; it doesn't *assign*
membrane as an axis-value.

### 6c — what the residue DID earn: three widened definitions
The detour's real yield is that the four axes were defined too tightly. Widen them and the residue is absorbed:
- **Scale = *all* the scales, not lexical containment.** The "my code → my deps → transitive → ecosystem"
  gradient I mistook for self/non-self is just the **dependency-distance** scale — one ordering in a *family*
  (lexical containment, dependency-distance, abstraction-level, blast-radius, …). Scale is plural.
- **Authority = responsibility / accountability, not just "how a claim got blessed."** *We're* responsible for
  our lines; the *user* for their config changes; the *system* for being transparent + documented. The
  write-permission residue from face 3 homes here — "responsible for our own lines" *is* an Authority statement.
- **Evidence = uniform across the build, no ownership cap** (per 6b-2) — and a per-property vector with an
  `intuited` rung (per the clean placements above).

**Verdict (v3):** **four descriptive axes hold** — `Scale (plural) · Subject · Evidence (per-property vector;
`structural/observed/asserted/reasoned/intuited`; build-uniform) · Authority (responsibility, broad)` — plus the
two response axes (immunity / compliance) and the orthogonal **Dynamics** layer. **No Provenance axis.** The most
useful artifact is the new standing rule: **the coordinate system describes user-facing behavior only;
construction concerns (acquisition, curation, herd/granuloma) are a *different map* — never smuggle one onto the
other.** Re-placing the anchors (no 5th coordinate): **ADR** = `(system, architecture, structural,
ceremony-ratified)`; **ATK** = `(meta, immune-self, observed-born-red, author-distinct)`; an **intent about a
dependency** = `(dependency-distance:1, code, build-uniform-evidence, dev-declared)` — reachable *within* the
widened axes, which is the proof they were the right four, just under-defined.

---

## Spawns (the ADRs this seeds)

- **The intent-layer ADR** — `#[intent(...)]`, the default-intent + config-as-policy, the flag-vocab + the
  business-rule primitive set + composition language, `IntentNotEnforceable`, the checkability grid, the Tier-4
  auto-graduation loop + the enforcement-pattern repertoire, the exceptions-register, the health-metric,
  shareable profiles, injection/`sync`.
- **The unified-corpus ADR** — antigens/antibodies/intents as one interlinked, federated corpus;
  compliance-policies as intent-bundles; the CWE↔intent-vocabulary identity.
- **The coordinate-system meta-frame** — a posture/ADR ratifying the kernel + **4 descriptive axes (Scale-plural
  · Subject · Evidence-as-per-property-vector-build-uniform · Authority-as-responsibility) + 2 response axes +
  the Dynamics layer**, *plus* the standing **construction-space vs user-facing-space** rule (the axes describe
  user-facing behavior only). It survived the pressure-test (Part 6) by *widening three definitions* rather than
  adding an axis — the membrane turned out to be a category error, not a missing dimension. Open seam for the
  ADR: is "the codebase has no membrane until antigen becomes one" worth stating as a first-class thesis (antigen
  = the immune boundary forming where there was none), or does it stay a framing note?
- (Depends on the already-named ADR-067 follow-ons: the sheaf-lens ADR, the field/maths ADRs, the stroma-builder.)
