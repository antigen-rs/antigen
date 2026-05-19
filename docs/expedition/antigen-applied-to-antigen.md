# Antigen Applied to Antigen — Recursive Self-Application

> **Status**: Substrate proposal (2026-05-12). Authored by team-lead in
> conversation with Tekgy after a day of substantive A3.5 sweep substrate
> surfaced a pattern that wants to be named.
>
> Not yet project-tier. Open for team expansion + Phase 1-8 + ratification.
> Companion to `multi-component-immunity.md` (V1, currently in expedition/)
> and `multi-component-immunity-conversation.md` (raw substrate).

---

## The observation

Antigen is not just building structural failure-class memory; it is
**recursively applying its own architecture to its own substrate**. The
project's development practice is itself an instance of the architectural
class antigen exists to instantiate.

This recursion is not decorative. It is structurally productive: each tier
of recursion produces substrate that the next tier operates on.

---

## Discriminator criterion (Q8 finding — added 2026-05-12)

Not every instance of "the discipline applied to itself" is antigen-
applied-to-antigen specifically. Generic engineering disciplines —
code review, post-mortem, refactoring rigor — are also recursively
self-applied without being instances of antigen's particular
architecture.

**Discriminator** (adversarial Q8 Finding 3): a genuine instance of
antigen-applied-to-antigen uses antigen's specific vocabulary at a
meta-tier — *named failure-class*, *structural fingerprint*, *witness*,
*provenance*, or *encounters/postures discipline*. Without one or more
of these vocabulary primitives operating at a meta-tier, the instance
may be antigen-style thinking but isn't antigen-on-antigen.

Applying this discriminator to the substrate instances surfaced in
A3.5 produces a tiered classification, not a uniform six-equivalent
list:

## Evidence from A3.5 substrate (2026-05-11 → 2026-05-12)

Six instances of *something antigen-style* surfaced in a single sweep.
Applying the discriminator: two STRONG, one MEDIUM, three antigen-style-
but-not-vocabulary (weaker fit). The aggregate is still substantive
evidence of recursive self-application, but the instances are
heterogeneous in mechanism. They are documented below with explicit
tier markers.

### 4. "Tool eating its own cooking" (Amendment 5 triage discipline) — STRONG

The triage discipline ratified in ADR-010 Amendment 5 Clause D — "can a
pure tokenizer (no semantic position analysis) close this gap? If yes,
engine-bridgeable; if no, semantic-distinction → docs" — is itself a
failure-class-recognition pattern. The triage IS antigen-style thinking
applied to the fingerprint grammar's own failure modes.

Antigen-applied-to-antigen at the engine layer: the project's vocabulary
for classifying code failure-classes generalizes to classify the project's
own engine failure-classes.

**Discriminator fit**: STRONG. Names a failure-class explicitly
("engine-bridgeable vs semantic-distinction"); uses failure-class-
recognition vocabulary at meta-tier.

### 6. Two-instance tokenization-asymmetry encounter — STRONG

The encounter "fingerprint silent-failure from tokenization-asymmetry"
now has two substrate-grounded instances (spacing + token-class). The
*meta-discipline of recognizing the encounter as a class* is itself
antigen-style failure-class memory operating at the project-substrate
tier.

Antigen-applied-to-antigen at the substrate tracking: the encounters
discipline catches its own multi-mechanism failure classes the same way
the tool catches multi-instance code failure classes.

**Discriminator fit**: STRONG. The encounters discipline IS antigen's
own substrate vocabulary; applied at project-substrate tier.

### 2. Recursive V8 (verifier-self-correction) — MEDIUM

Aristotle's Phase 1-8 of ADR-010 Amendment 5 caught an overclaim in
scout's draft (Mechanism B was listed in "Failure-class eliminated"
section, but proc_macro2 round-trip does NOT bridge token-class
distinctions — only whitespace asymmetry). This is V8 (verifier-self-
correction) at the amendment-proposal tier, one layer deeper than
A1-CLOSURE's verifier-self-correction surfaced at the verdict tier.

Antigen-applied-to-antigen at the ratification process: the discipline
catches its own proposal's overclaim before ratifying.

**Discriminator fit**: MEDIUM. Uses Phase 1-8 ratification vocabulary
which is *project-internal verification discipline* but not antigen's
published vocabulary specifically. The V8/postures §7 framing IS antigen
vocabulary; the underlying mechanism is generic peer-review.

### 1. C1 folded structure — antigen-style (weaker fit)

Naturalist's C1 (dev-in-the-loop) cognate refinement found that the
developer is **both** the meta-agent shaping the substrate AND a
participant in the substrate being shaped. No other component has this
property. The developer writes antigen declarations (production source)
AND writes code that exhibits failure-classes (subject of presentation/
immunity).

This is antigen-applied-to-antigen at the developer level: the agent
applying the discipline is also subject to the discipline they apply.

**Discriminator fit**: WEAK. Antigen-style framing ("developer is both
agent and subject"); no named failure-class, no fingerprint, no witness
at the developer tier. It's structural observation about C1's character,
not antigen vocabulary operating at a meta-tier. Catalogued because the
folded property is real architectural substance even if it doesn't fit
the strict discriminator.

### 3. ADR-tier substrate-currency catch — antigen-style (weaker fit)

Scout's A3.5 tutorial cross-check found that ADR-010 ratified text had
contained `"(Self, Self) -> Self"` (malformed for the actual use case)
for the entire post-ratification period. The ratification process had
verified the ADR's substance but not its concrete fingerprint examples
against the matcher's actual behavior.

Antigen-applied-to-antigen at the ratified canonical substrate: the
discipline catches drift in its own ratified text retroactively. Surfaced
as candidate for postures §7 Instance 10; held below threshold (one
instance, name the catch, watch for recurrence).

**Discriminator fit**: WEAK. The catch is antigen-style thinking
applied to project's own substrate, but the *mechanism* (spec-vs-code
cross-check) is generic code-review discipline, not specifically antigen
vocabulary. The postures §7 framing IS antigen vocabulary; the underlying
catch-mechanism is not.

### 5. Scout's cross-check discipline — antigen-style (weaker fit)

Scout's spec-against-engine cross-check (verifying tutorial fingerprint
strings against actual matcher behavior by reading `render_inputs()` and
tracing what proc_macro2 produces) is a substrate-currency discipline at
a tier that adversarial discipline cannot reach. Two complementary
disciplines:

- **Adversarial**: probes the *named* attack surface; files ATK contracts;
  tests Phase 8 forced-rejection
- **Scout**: cross-checks *specification* against *actual engine behavior*
  for spec-invisible silent failures

Adversarial would not have caught either of today's two tokenization bugs
(`&mut self` whitespace; `Self`/`self` token-class). Both required
substrate-level check against engine behavior with specific function-form
awareness.

Antigen-applied-to-antigen at the verification methodology: the project
has multiple disciplines for catching its own failure modes, each catching
a different class of silent failure.

**Discriminator fit**: WEAK. Substrate-currency vocabulary is antigen-
adjacent but generic; "complementary disciplines catch different failure-
class shapes" is antigen-style reasoning about the discipline, not
antigen vocabulary at a meta-tier.

### 7. The Q8+Q2 amendments themselves — STRONG (added 2026-05-12)

Adversarial (Q8) and naturalist (Q2) caught team-lead's overclaim in this
document — the six-instance list treated heterogeneous fits as uniform;
the vaccination cognate was used as unified when three different cognates
were needed; the ending overclaimed "no fixed point." The discipline
applied to the very document about the discipline applied to itself.

**Discriminator fit**: STRONG. Uses antigen vocabulary (encounters
discipline, falsification trigger, recognition-not-design threshold,
posture-class candidacy) at meta-tier. The amendments themselves are
the operation the document describes.

The recursion deepens by one tier: the catch on the document about the
catches is itself a catch.

### 8. ADR-011 tolerance-without-attestation self-diagnosis and cure — STRONG (added 2026-05-19)

`#[antigen_tolerance(...)]` without attestation is exactly the failure class
`attestation-void-discipline-claim`: a discipline claimed but not attested
in verifiable substrate. No structured record of who approved the tolerance,
when, against what review — just a rationale string.

The failure class has documented real-world instances at the industry tier:
Boeing 737 MAX MCAS (software review discipline claims without structured
attestation of who reviewed which code against which requirements);
Heartbleed/OpenSSL (code in the critical path reviewed by volunteers; no
structured record of who reviewed which commits against which security
requirements); Log4Shell (no structured review process for security-sensitive
code paths despite critical-infrastructure status). ADR-006's
three-independent-instances threshold is met for stdlib promotion eligibility.

Antigen diagnosed `attestation-void-discipline-claim` in its own ADR-011
mechanism and discipline-witnesses v3 cures it: `#[antigen_tolerance(X,
sidecar = true)]` with the isomorphic Ratification schema, enforced by the
`tolerance-vibes-grade` audit hint that fires for unattested tolerances.
The structural fingerprint (tolerance attribute without sidecar), the witness
(audit hint emitted at EvidenceKind::None tier), and the cure (sidecar opt-in
with same schema as immunity sidecars) all use antigen vocabulary at a
meta-tier.

**Discriminator fit**: STRONG. Named failure-class (`attestation-void-discipline-
claim`); structural fingerprint (the `#[antigen_tolerance]`-without-sidecar
pattern); witness (the `tolerance-vibes-grade` audit hint). Antigen recognized
its own failure class and shipped the remedy. This is stronger than instances
4, 6, and 7 in one respect: those involve antigen-style thinking applied to
the project's development process; this one involves the project shipping a
tool feature that explicitly curates a failure class the project itself
presented.

**Long-arc note**: `attestation-void-discipline-claim` is ready to propose
as a seed antigen for antigen-stdlib. Instances are documented (Boeing MCAS,
Heartbleed, Log4Shell). Fingerprint: a codebase has a discipline-review claim
(team policy, README, ADR prose, comment) but no machine-verifiable substrate
backing the claim. This is the foundation-antigen for the discipline-witnesses
feature area — the failure class discipline-witnesses exist to address.

---

### Tiered summary

| Instance | Discriminator fit | Mechanism |
|---|---|---|
| 4. Amendment 5 triage | STRONG | Named failure-class vocabulary at engine layer |
| 6. Tokenization encounter | STRONG | Encounters discipline at project-substrate tier |
| 7. Q8+Q2 amendment catch | STRONG | Antigen vocabulary catching antigen-document overclaim |
| 8. ADR-011 self-diagnosis + discipline-witnesses cure | STRONG | Named failure-class + fingerprint + witness at meta-tier; tool cures own failure class |
| 2. Recursive V8 | MEDIUM | Phase 1-8 + postures §7 (mixed antigen + generic peer-review) |
| 1. C1 folded structure | WEAK | Structural observation; no failure-class vocabulary at meta-tier |
| 3. ADR-tier substrate-currency | WEAK | Generic spec-vs-code check; postures §7 framing is antigen |
| 5. Scout's cross-check | WEAK | Substrate-currency vocabulary is antigen-adjacent; generic discipline |

Four STRONG instances pass the discriminator clearly. Three WEAK
instances are antigen-style thinking applied to the project but don't
use antigen vocabulary at a meta-tier. The recursion-is-architectural
claim still holds — ADR-006's three-independent-instances threshold is
met by the STRONG instances alone — but the document is no longer
overclaiming "six uniform instances." Instance 8 (added 2026-05-19) is
the most structurally complete: it names a failure class with real-world
ecosystem instances AND demonstrates the full cure via discipline-witnesses.

---

## What antigen-on-antigen does NOT catch

Recognition-not-design at the discriminator layer: the recursive
discipline doesn't claim to catch everything. The scope is bounded by
what antigen's vocabulary names. Other catches happen via other
disciplines.

Concrete examples of failures *not* caught by antigen-on-antigen
specifically:

- **Backtick compilation bug** (caught by `cargo build`, not by
  recursive discipline). The Rust compiler is the right tool; antigen
  doesn't add a recursion layer on syntax errors.
- **39 parse failures in test fixtures** (visible from scan output, not
  from antigen-on-antigen reasoning). The scan reports parse failures
  directly; no recursion needed.
- **Generic spec-vs-code drift** (caught by scout's spec-against-engine
  cross-check, classified as WEAK fit). Engineering disciplines like
  code review catch this without needing antigen vocabulary.
- **CI/build pipeline misconfiguration** — entirely outside antigen's
  scope.
- **Performance regressions** — separate discipline (benchmarks,
  profiling); antigen does not address.
- **Security vulnerabilities at code level** — antigen catches the
  *failure-class memory* of vulnerabilities (e.g., "this pattern has
  been exploited before"); the live discovery of new vulnerabilities is
  outside scope.

This scope-clarity matters: antigen-on-antigen is a recursion
*within antigen's own vocabulary*, not a claim that antigen-style
thinking subsumes all engineering discipline. The recursion exists
because antigen's vocabulary IS general enough to classify some of its
own failure modes — but not all of them.

---

## The discipline-preceded-tool genesis

Antigen-on-antigen is so visible in our substrate because of a specific
historical accident: **we had the discipline before we had the tool**.

The originating insight came from tambear's 2026-05-06 cleanup expedition
— a structural failure-class was caught a second time using tribal
recognition. The project that grew from that insight is the
operationalization of recognition discipline into code substrate. The
discipline preceded the tool by design.

This means **we are the ideal user of antigen** in a way that won't be
replicable for future adopters who come to both tool and discipline
fresh. Our substrate's density of antigen-on-antigen instances reflects
that we're inside the architecture we're building.

But:

---

## Three adopter pathways with three different biology cognates

Per naturalist Q2 falsification check (2026-05-12): the "vaccination"
cognate doesn't hold uniformly across adopter pathways. Each pathway has
a different relationship between tool and existing discipline; biology
has different mechanisms for each.

**Vocabulary disambiguation note**: "vaccination" is already bound in
`immune-system-primitive-map.md` (line 69) to `cargo antigen vaccinate`
(the planned A5 tooling primitive). Using "vaccination" at the
adoption-pathway tier creates double-binding. Below: distinct cognates
per pathway; "vaccination" reserved for the tooling primitive.

### Junior adopters — *developmental immunology* cognate

Someone learning Rust+antigen together as one practice (like learning
Rust+cargo+tests together) develops both in parallel. The tool teaches
the discipline by demanding it: declaring an antigen forces the developer
to name the failure-class, name the witness, justify the rationale. The
structure produces the practice.

**Biology cognate**: *developmental immunology*. Vaccination doesn't
apply here — biology doesn't vaccinate a being with no immune system.
What biology does is *build the recognition machinery itself*: thymic
education of T-cells, B-cell repertoire formation, the developmental
process of becoming an organism with an immune system. Junior adopters
develop antigen-the-discipline through antigen-the-tool; the cognate is
maturation of the recognition apparatus, not training of an existing
one.

**Tool-as-discipline-scaffold**: TRUE for this pathway. The tool
produces discipline through use.

### Senior adopters with partial discipline — *vaccination* cognate

Developers who already have some sense of failure-class awareness
(tribal knowledge, post-mortem discipline, code-review judgment) but
lack the structural-memory piece get the missing tier from the tool.
They don't rebuild antigen-the-discipline from scratch; they extend
their existing practice with antigen's structural surface.

**Biology cognate**: *vaccination*. This is the one pathway where the
original unified cognate is precisely right. Existing partial-recognition
machinery (memory cells from prior exposures, generic pattern-recognition
receptors) meets new structural targets via the tool. The vaccine
introduces the recognition pattern; the existing immune machinery
operates on it.

**Tool-as-discipline-amplifier**: more accurate framing for this
pathway than tool-as-scaffold. The tool extends existing partial
discipline; doesn't produce it from nothing.

### Mature organizations with explicit discipline — *immune surveillance / checkable immunity* cognate

Teams that already have ADR culture, post-mortem rigor, refactoring
discipline, etc. already have antigen-like practices in narrative form.
The tool gives them structural enforcement: ADR claims about failure-
classes become checkable by `cargo antigen scan`.

**Biology cognate**: *immune surveillance* + *checkable immunity*.
Vaccination is REDUNDANT here — these organizations already recognize
the relevant patterns and are already immune in narrative form. What
the tool provides is making their existing recognition *externally
verifiable*: structural enforcement of claims they already make in
prose.

**Tool-as-discipline-formalizer**: more accurate framing for this
pathway. The tool makes existing discipline *checkable*; it doesn't
*produce* the discipline.

### The framing across all three

The "non-replicable" framing of the project's own ideal-user property
is wrong, but for asymmetric reasons across pathways:

- Junior: replicable through developmental learning (scaffold)
- Senior: replicable through partial-discipline extension (amplifier)
- Mature: not "replicated" — recognized and formalized (formalizer)

The unified "tool-as-discipline-scaffold" framing only captures the
junior pathway precisely. The other two pathways need different verbs:
amplifier, formalizer. All three are real adoption pathways; the
"non-replicable" worry dissolves once each pathway's specific mechanism
is named.

### Alternative interpretation (Q8 Finding 4 — added 2026-05-12)

There's an alternative reading of the project's own dense
antigen-on-antigen substrate that deserves honest acknowledgment: the
density may reflect *observational priming* rather than *structural
property*. The team has been actively looking for instances of
recursion-of-recognition because the framing pulls; once a frame is
operating, the substrate it predicts becomes more visible.

Both readings can be partially true: there is a real structural property
(antigen's vocabulary IS general enough to classify some of its own
failure modes); AND there is some observational priming (we're more
likely to notice and catalogue antigen-style instances because we're
inside the project that named the discipline).

The discriminator criterion above is the principal defense against
observational-priming inflation — applying a falsification test before
counting an instance. The three WEAK instances in the catalogue might
exist partly through priming; the three STRONG instances pass the
discriminator regardless of priming.

---

## Multi-language extension

Antigen-the-vocabulary is language-agnostic in principle. The five
primitives (declare/present/immune/descended_from/tolerance) describe a
structural architecture of failure-class memory that doesn't depend on
Rust.

Per-language implementations are components in the multi-component
framing:

- **Rust** (current; antigen v0.1.0): proc_macro2/syn-based fingerprint
  engine; cargo subcommand
- **Python** (future): ast-module-based or tree-sitter-based fingerprint
  engine; pip-installable tool with `python -m antigen scan` invocation
- **JavaScript / TypeScript** (future): Babel-based or tree-sitter-based
  fingerprint engine; npm-installable tool
- **Framework-specific** (future): React-tier antigens, Django-tier
  antigens, Rails-tier antigens — each operating on the framework's
  metaprogramming surface
- **Other languages**: any language with sufficient metaprogramming or
  AST tooling can host an antigen implementation

The fingerprint engines specialize per language; the vocabulary stays
constant. A failure-class declared in one language's antigen-stdlib
can be ported to another language by translating the fingerprint while
preserving the architectural shape.

**Failure-class generalization across languages**: surface tokens differ
but architectural failure-classes recur. Examples:

- "Drop impl must not panic" (Rust) ≈ "context manager `__exit__` must
  not raise during cleanup" (Python) ≈ "destructor function in resource
  handler must not throw" (JavaScript) ≈ "destructor must not throw"
  (C++).
- "Polarity-inverted lattice meet" — abstract enough to apply anywhere
  a lattice structure is implemented with discriminant ordering.
- "Frame-translation drift" — abstract enough to apply to any
  type-system feature that converts between representations.

The taxonomy operates at the structural-shape level above any specific
language. Adding a fail-class can inform all languages.

---

## Cross-tier manifold (organization → code)

The recursion extends across abstraction tiers, not just within one
codebase or one language.

Observed (some today, some across A1-A3.5 substrate):

| Tier | Failure-class examples | Components/mechanism |
|---|---|---|
| **Organization** | Decision-failure-classes (sub-clause F violations at chartering; rationale-free policies; spec-then-ratify when recognition-grounded is correct) | ADR discipline; postures.md catalog |
| **Team coordination** | Substrate-currency drift across multi-agent routing; tier-honesty drift; outbox-state-as-substrate-state | A1-CLOSURE substrate-currency discipline; verification-protocol every "X complete" claim |
| **Solo developer** | Judgment-failure-classes (premature closure; framing-without-substrate; over-claim on cognate-fit) | Naturalist disciplines (snag-feel diagnostic, boundary-silence as evidence, structural-rhyme as falsification trigger) |
| **AI agent (LLM)** | Context-failure-classes (memory-based hallucination; pre-compaction summary as current state; spec-invisible-silent-failure overlooked) | Substrate-over-memory discipline; feels-familiar verification |
| **Tooling** | Engine-bridgeable vs semantic-distinction confusion; tokenization-asymmetry silent-failure; reranker-confidence-without-substrate | Cross-check discipline; Amendment 5 triage |
| **Language** | Tokenization asymmetries (Rust's `Self`/`self` distinction); semantic position drift; receiver-rendering categorical-token-difference | Per-language fingerprint engines |
| **Code** | Drop-impl panic; polarity-inverted-class-meet; commutativity-class panic-during-coercion; etc. | `#[antigen]` declarations + scan/audit |

At each tier the mechanism differs; the compositional property (structural
failure-class memory) recurses. This is heterogeneous recursion (per V1
deep-dive Part V) operating across tiers, not just within one tier.

The recursion-from-highest-to-lowest framing extends multi-component-
immunity beyond the seven components currently named. The manifold has
more axes than V1 captured — the tier axis is one we hadn't explicitly
named.

---

## What this changes

### Adoption framing

The "value gradient is continuous, no cliff" property of multi-component
immunity (V1 Part IV) extends to **tier of engagement**, not just choice
of components:

- A solo dev can adopt antigen for judgment-failure-classes alone
- A team can adopt for coordination-failure-classes + judgment + code
- An organization can adopt for decision-failure-classes + everything
  downstream
- An AI agent can adopt for context-failure-classes + tooling +
  judgment + code

Each tier of adoption produces structural failure-class memory at that
tier. The architecture is multi-tier as well as multi-component.

### Ecosystem evangelism

We don't sell antigen as "a Rust tool for catching bugs." We sell it as
**the practice of structural failure-class memory**, with antigen-the-
Rust-tool as one instantiation. The pitch generalizes:

- "Your team's tribal knowledge can be structural"
- "Your post-mortems can produce immunity, not just lessons"
- "Your discipline can be checkable by tooling at the same time it stays
  the discipline you already had"
- "Your codebase can carry the lessons that produced it, not just the
  code that resulted from them"

Each pitch lands at a different tier of adopter.

### Manuscript trajectory

The paradigm-shift paper (post-A6) now has a much richer claim than "we
built failure-class memory for Rust." The claim is:

> Antigen instantiates a multi-tier recursive architecture for structural
> failure-class memory that generalizes across languages, across team
> scales, across abstraction tiers, and across human/AI collaboration
> patterns. The Rust implementation is one instance; the architectural
> class is universal.

That's a much bigger paper. It's also more honest to what we've actually
been building.

### Roadmap territory

`docs/roadmap.md` should reflect:

- **v0.2+**: deeper Rust tooling (body-level fingerprints, IDE integration,
  CI surface)
- **v0.3-v0.5**: Rust ecosystem maturity (antigen-stdlib, cross-org
  registries)
- **v1.0+**: multi-language extension (Python, JS, framework-specific)
- **Cross-tier extensions**: not version-scoped; develop alongside per-
  language work as substrate accrues

The roadmap should make the multi-language and cross-tier intent visible
to adopters from v0.1.0 so they understand antigen is not "a Rust thing"
but "a structural-discipline thing whose first ergonomic instantiation
happens to be Rust."

### Showcase by building

The project's own development substrate IS the proof of value. We don't
ship v0.1.0 with promises and claims about what antigen will do. We ship
with the substrate that demonstrates what antigen has already done:

- Five ADR amendments today (Amendment 5 the latest), each ratified
  through the discipline antigen formalizes
- Multiple substrate-currency catches across the day, each surfaced by
  the disciplines antigen names
- Tutorial that walks through real failure-classes (PanickingInDrop,
  PolarityInvertedClassMeet, KernelReconstructionDivergence) caught in
  real codebases
- Substrate that demonstrates the multi-tier recursion is operational,
  not aspirational

The README and vision-pitch.md should foreground this. Not "trust us
this is valuable" — but "look at the substrate; the substrate's quality
IS the value claim."

---

## Open questions for team expansion

Q1. **Is "antigen-applied-to-antigen" itself a posture-class candidate?**
Six instances surfaced in A3.5 today. ADR-006 threshold met for count;
load-bearing-reason and shape-stability still TBD. Aristotle Phase 1-8
when bandwidth opens.

Q2. **Is "tool-as-discipline-scaffold" structurally accurate for future
adopters?** Predicts something about how adoption succeeds. Naturalist's
biology-cognate lane (vaccination as cognate). Scout's structural-rhyme
discipline (does the prediction hold under falsification?).

Q3. **Multi-language extension as A6+ scope candidate**: at what point
does Rust-only become limiting? When does Python or JavaScript become a
real next sweep? Currently held as roadmap aspiration; substrate-grounded
trigger TBD.

Q4. **Cross-tier manifold — how should it land in V2 deep-dive?**
Multi-component-immunity V1 names seven components within Rust code-tier
adoption. The cross-tier manifold extends the framing. Aristotle Phase
1-8 of V2 when V2 substrate accrues.

Q5. **Failure-class taxonomy at the structural-shape level**: does the
project want a separate taxonomy document at the abstract-failure-class
level (independent of Rust)? Possibly `docs/expedition/cross-language-
failure-class-taxonomy.md` when substrate accrues.

Q6. **Showcase-by-building as marketing posture**: does this change
vision-pitch.md? The current framing is value-prop-then-claim; the
showcase-by-building framing is substrate-as-claim. Tekgy + team-lead
adjudicate when manuscript work resumes.

Q7. **The "ideal user" property and v0.1.0 release narrative**: do we
make the discipline-preceded-tool genesis explicit in v0.1.0 release
material? It's an unusual claim ("the developers used the tool to build
the tool while building it") that could be a feature or a confounder.

Q8. **Adversarial surface on the recursive claim**: where does antigen-
applied-to-antigen fail or mislead? Self-referential systems have known
pathologies; is recursion-as-discipline subject to runaway recursion
(every meta-tier needs its own meta-tier; no fixed point)?

Adversarial check filed 2026-05-12 (campsite
`adversarial/20260512-antigen-on-antigen-q8.md`). Four findings: (1) MEDIUM:
"no fixed point" ambiguity — recursion terminates locally at substrate
artifacts but document doesn't name the stopping property; (2) HIGH:
circular validation — all six instances are catches, no analysis of blind
spots; (3) HIGH: scope collapse / no falsification criterion — resolved by
the discriminator section above; (4) LOW: discipline-preceded-tool is
unfalsifiable (observational priming vs structural property).

Findings 1, 2, and 4 open for aristotle Phase 1-8 integration. Finding 3
resolved above.

---

## What this document is NOT

- Not a ratified framing. Substrate proposal for team expansion + Phase
  1-8 + ratification.
- Not a replacement for V1 deep-dive. Companion substrate that extends
  V1's multi-component framing into multi-tier territory.
- Not authoritative on biology (anti-idiotype antibody cognate, T-cell
  regulation cognate). Naturalist refinements supersede.
- Not a roadmap commitment. Multi-language extension is structural
  ambition; per-version commitment lands when substrate accrues.
- Not a marketing pitch. Substrate-grounded observation that may
  inform vision-pitch.md eventually.
- Not exhaustive. The six instances catalogued from today are
  illustrative, not complete. More instances will surface as substrate
  accrues.

---

## Acknowledgment

Authored 2026-05-12 by team-lead in conversation with Tekgy after the
2026-05-11 A3.5 substrate produced six instances of antigen-applied-to-
antigen in a single sweep. The framing emerged in dialogue; this
document is one step of processing it into substrate.

Tekgy's sharpening landed three load-bearing corrections to my initial
framing:

1. The "non-replicable for fresh adopters" framing was wrong — tool-as-
   discipline-scaffold is the successful adoption pathway, replicable
   through onboarding.
2. Senior adopters with partial discipline are a real adoption tier; the
   tool fills the structural-memory piece without requiring rebuild.
3. Failure-class abstraction operates above any specific language —
   "Drop impl must not panic" generalizes structurally across Python,
   JavaScript, C++, etc.

Without those sharpenings, the framing would have been smaller and less
honest to what we're actually building.

*The recursion terminates locally at substrate artifacts — corrected
ADRs, passing tests, amended declarations, ratified amendments — even
though there is no global tier above which recursion stops. Local
termination is what matters; the absence of a global fixed point is not
a problem. We may find more instances as we keep recursing; each will
terminate locally in substrate.*
