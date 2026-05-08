# Antigen — Cross-Domain Architectural Map

> **V1 (2026-05-08)**. Forward-substrate map of academic fields beyond
> classical immunology, virology, and medicine where structural-recognition-
> with-memory-and-inheritance has been independently studied. Authored by
> academic-researcher during A2 day-2 evening as the cross-domain companion
> to [`immune-system-primitive-map.md`](immune-system-primitive-map.md) V0
> (which catalogs the biology spine). The two documents map together: V0
> deepens biology; this document deepens *the rest of the academic record*
> that converges on the same architectural questions biology has been
> answering for ~500 million years.
>
> **This document is recognition substrate, not design.** Per ADR-006: each
> cognate listed here is a tooling/framing primitive that lands in antigen
> *when adoption surfaces a real instance that needs it*. The point of the
> map is not to commit antigen to building anything; it is to name where the
> rest of the academic literature has already done structural work the
> project will eventually need to recognize.
>
> **The window count is a lower bound, not a census.** The map currently
> names fifteen fields plus one confirmed A2-day-2 addition (window 16:
> epistemic logic / common-knowledge proofs — passes full structural-identity
> test; scientist ruling 2026-05-08). This is structurally expected: a
> framework with genuine falsifiability structure will keep finding gaps when
> applied carefully. The no-fixed-point property (Finding 6 below) operates
> at the meta-level — the framework predicts its own expansion. Each time the
> structural-identity test is run against a new field, the count updates.
> "Fifteen+" means at-least-fifteen, not exactly-fifteen.
>
> **Companion to**:
> - [`immune-system-primitive-map.md`](immune-system-primitive-map.md) V0
>   (biology / virology / medicine / public health spine)
> - [`scope.md`](scope.md) (the multi-window convergence frame)
> - Sweep A1 closure narrative
>   ([`../sweeps/A1-design-ratification/CLOSURE.md`](../sweeps/A1-design-ratification/CLOSURE.md)),
>   four empirical validations, especially Validation 3 (three-window defense
>   of ADR-003 — biology + past-self gardening + academic CS lineage)

---

## Why this document exists

Sweep A1's closure named **three windows** that converge on antigen's
architecture (biology, past-self gardening, programming-language theory) and
scope.md added a **fourth** (2026 ML graph-memory research). The three- and
four-window convergence is itself an empirical defense of the architecture
(per ADR-003): when independent traditions arrive at the same primitive
without coordinating, the structure is real, not metaphor-dependent.

This document widens the convergence test. It asks: **across the academic
record more broadly — not just biology, not just programming languages —
where else does the same architecture appear?**

The thesis the map will substantiate: **structural-recognition-with-memory-
and-inheritance is a domain-general architecture studied in at least
fifteen academic fields**, each of which has discovered it independently
and given it different vocabulary. The biology spine in V0 is one
instantiation among many. Antigen is the first ergonomically-adoptable
instantiation in a programming-language ecosystem; it is not the first
instantiation of the architecture itself.

This is the strongest possible defense of recognition-not-design (posture
§2): if fifteen+ independent fields converge on the same architecture, the
project is not designing anything — it is recognizing what was already
there, distributed across the disciplines.

---

## Structural-identity criteria

Per math-researcher's structural-identity test in postures.md §7 (depth-
shift discipline), the criteria for "this field's framework is the same
pattern, not a rhyme":

1. **Same fail-mode without the architecture** (i.e., what fails when
   recognition + memory + inheritance is absent).
2. **Same recovery shape** (i.e., the architectural move that fixes the
   fail-mode).
3. **Same routing pattern** (i.e., how the recognition propagates from
   instance to declaration to inheritance to enforcement).

A field passes the structural-identity test when all three criteria are
"yes" — the field has independently arrived at antigen's architecture.

A field passes the **partial-cognate** test when one or two criteria match
— the field illuminates a piece of the architecture but not the whole.

A field is **silent** when zero criteria match — the field is genuinely a
different architecture, and the silence is informative (per naturalist's
biology-as-instrument framing: silence where it should be silent is
evidence of load-bearing-ness).

Each entry below records its identity-criteria status explicitly.

---

## Index

1. [Cognitive science — schema theory + chunking + structure-mapping](#1-cognitive-science--schema-theory--chunking--structure-mapping)
2. [Evolutionary biology — convergent evolution + fitness landscapes + niche construction](#2-evolutionary-biology--convergent-evolution--fitness-landscapes--niche-construction)
3. [Ecology — niche partitioning + keystone species + ecosystem resilience](#3-ecology--niche-partitioning--keystone-species--ecosystem-resilience)
4. [Information theory — error-correcting codes + Hamming distance + redundancy](#4-information-theory--error-correcting-codes--hamming-distance--redundancy)
5. [Semiotics — Peirce's icon/index/symbol triad + sign-to-symbol elevation](#5-semiotics--peirces-iconindexsymbol-triad--sign-to-symbol-elevation)
6. [Knowledge management — Nonaka SECI tacit/explicit conversion](#6-knowledge-management--nonaka-seci-tacitexplicit-conversion)
7. [Complex adaptive systems — Holland's signals + boundaries + emergence](#7-complex-adaptive-systems--hollands-signals--boundaries--emergence)
8. [Cybersecurity — MITRE ATT&CK + CVE/NVD + threat-intelligence taxonomies](#8-cybersecurity--mitre-attck--cvenvd--threat-intelligence-taxonomies)
9. [Aviation safety — NTSB blameless post-mortem + structural recommendations](#9-aviation-safety--ntsb-blameless-post-mortem--structural-recommendations)
10. [Pattern languages — Christopher Alexander's architecture + GoF descent](#10-pattern-languages--christopher-alexanders-architecture--gof-descent)
11. [Cumulative culture — Tomasello's ratchet effect + transmission fidelity](#11-cumulative-culture--tomasellos-ratchet-effect--transmission-fidelity)
12. [Indigenous epistemologies — oral transmission + intergenerational pattern](#12-indigenous-epistemologies--oral-transmission--intergenerational-pattern)
13. [Stigmergy — environmental signaling + memory-in-substrate](#13-stigmergy--environmental-signaling--memory-in-substrate)
14. [Bayesian inference — prior/posterior + rare-event reasoning](#14-bayesian-inference--priorposterior--rare-event-reasoning)
15. [Philosophy of science — paradigm shift + normal-science accumulation](#15-philosophy-of-science--paradigm-shift--normal-science-accumulation)
16. [Epistemic logic — common-knowledge proofs + the muddy-children puzzle](#16-epistemic-logic--common-knowledge-proofs--the-muddy-children-puzzle)
17. [Where the cross-domain map goes silent (honest boundaries)](#where-the-cross-domain-map-goes-silent-honest-boundaries)
18. [Cross-domain convergence findings](#cross-domain-convergence-findings)

---

## 1. Cognitive science — schema theory + chunking + structure-mapping

### Field synopsis

Cognitive science has three independent threads that converge on antigen's
architecture from different angles:

- **Schema theory** (Bartlett 1932 → Rumelhart 1980 → contemporary 2024
  research): humans encode new experience by fitting it into pre-existing
  schemas — abstract structural patterns that organize knowledge. Schema
  activation drives comprehension, pattern recognition, and memory
  consolidation. Failure to recognize a schema produces lossy encoding;
  successful schema match produces durable, retrievable memory.

- **Chunking theory** (Chase & Simon 1973, de Groot's chess studies, Gobet
  & Simon's template theory): expertise is the result of accumulating
  thousands of named structural units (chunks/templates) in long-term
  memory. Experts perceive at the level of chunks, not raw features.
  Beginners see piece positions; chess masters see "Sicilian Defense
  middle-game with isolated d-pawn" — a structurally-named class. Chunks
  are inherited via template hierarchies; new chunks build on existing
  ones.

- **Structure-mapping theory** (Gentner 1983, Gentner & Holyoak 1997, the
  Structure-Mapping Engine): analogical reasoning works by mapping
  *systems of relations* — not surface features — from base domain to
  target domain. The mapping is constrained by *systematicity*: relations
  embedded in larger relational systems transfer; isolated features do not.

### Structural-identity test

| Criterion | Schema | Chunking | Structure-mapping |
|---|---|---|---|
| Same fail-mode without architecture | yes (schema-less learning is lossy) | yes (chunkless perception caps at ~7 items) | yes (surface-similarity retrieval misses deep structure) |
| Same recovery shape | yes (named schema persists past instances) | yes (named chunks persist past instances) | yes (mapped relational systems persist) |
| Same routing pattern | yes (schema propagation across instances) | yes (template inheritance hierarchies) | yes (systematicity-constrained relation transfer) |

**Verdict: full structural identity across all three threads.** Cognitive
science is one of the deepest cross-domain instantiations of antigen's
architecture, with three internal sub-instantiations.

### What this maps to in antigen

- `#[antigen]` declaration ≈ named schema / named chunk / named relational
  system. Each is an explicit, retrievable, propagatable structural class.
- Fingerprint matching ≈ schema activation / chunk recognition / structure
  mapping. The act of matching a code site to a fingerprint *is* the act of
  recognizing the pattern.
- `#[descended_from]` ≈ template inheritance hierarchy (Gobet & Simon) /
  schema hierarchy / structure-mapping systematicity. Inherited structure
  carries inherited recognition.
- The four-window convergence claim is itself a structure-mapping move —
  mapping the architecture across biology / programming-language theory /
  past-self gardening / ML graph-memory at the *relational* level, not the
  surface level.

### Where the cognitive-science framework adds new substrate beyond biology

- **Transmission fidelity is a measurable property, not just a desired one.**
  Schema theory and chunking theory both quantify the conditions under which
  pattern memory survives transmission across individuals (Tomasello's
  ratchet effect — see §11 — extends this). Antigen has not yet quantified
  its own transmission-fidelity properties; the cognitive-science literature
  predicts what to measure.

- **Surface-similarity retrieval is the dominant failure mode.** Gentner's
  finding that humans tend to retrieve surface-similar cases rather than
  relationally-similar ones is directly relevant to antigen's W6a synthesis
  pass and adversarial test substrate: developers confronted with a
  fingerprint match may pattern-match by surface (variable names, syntactic
  shape) rather than by structural relationship. The cognitive-science
  literature predicts adversarial test patterns we should specifically
  generate.

- **Expertise is structurally distinct from skill.** Chunking theory says
  expertise is *named-pattern accumulation*; antigen-stdlib + project-
  specific antigens combined IS the operationalization of expertise as a
  shared, structural artifact. The framing is recognition-grade for
  ecosystem-outreach.

### Recognition triggers in antigen

- When stdlib reaches the point where adversarial tests need to specifically
  generate "surface-similar but structurally-different" code that should NOT
  match (Gentner's failure-mode in test form), the cognitive-science
  framework makes the test pattern explicit.
- When antigen-stdlib documentation needs a framing for "why this catches
  things expert reviewers miss," the chunking framework predicts the answer:
  experts have chunks; antigen makes chunks structural and shareable.

### Recognition examples in cited prior art

- Schema theory: 2024 *Schemas play a causal role in forming lasting
  associative memory representations during one-trial learning, emphasizing
  their importance in memory consolidation* — direct cognate to ADR-001's
  "structural memory survives, implicit memory decays."
- Chunking: Chase & Simon's finding that expert chess players store
  thousands of named chunks in LTM is the empirical analog to antigen-stdlib
  as accumulated ecosystem expertise.
- Structure-mapping: SME (Forbus et al.) is computationally homologous to
  antigen's fingerprint-matching engine — both recognize structured
  relational patterns rather than feature bags.

---

## 2. Evolutionary biology — convergent evolution + fitness landscapes + niche construction

### Field synopsis

Beyond immunology-specific evolution (V0 §Virology mutation rates), broader
evolutionary biology has three threads relevant to antigen:

- **Convergent evolution**: independent species evolve similar traits under
  similar selective pressures. The shared environment selects for the same
  structural answer, even when the organisms have no common ancestor for
  that trait.

- **Fitness landscapes** (Wright 1932, Kauffman 1993): the genotype-phenotype-
  fitness relationship visualized as a topology with peaks and valleys.
  Selection pressure causes populations to climb toward local peaks;
  populations stuck on suboptimal peaks may need oscillating selection or
  drift to traverse fitness valleys to higher peaks.

- **Niche construction theory** (Lewontin 1983, Odling-Smee, Laland) +
  **extended phenotype** (Dawkins): organisms do not passively adapt to
  selection; they *construct* the selective environment. Beaver dams,
  termite mounds, human agricultural systems — built environment that
  becomes the selective context for subsequent generations. The constructed
  niche is itself heritable.

### Structural-identity test

| Criterion | Convergent evolution | Fitness landscape | Niche construction |
|---|---|---|---|
| Same fail-mode without architecture | yes (re-derivation across populations) | partial (suboptimal-peak lock-in) | yes (each generation re-builds environment) |
| Same recovery shape | yes (independent arrival at same structure) | partial (path-dependence is the architecture) | yes (constructed niche persists across generations) |
| Same routing pattern | yes (selection pressure as fingerprint) | partial (gradient-following in topology) | yes (extended phenotype as inherited environment) |

**Verdict: full structural identity for convergent evolution and niche
construction; partial for fitness landscapes.** Convergent evolution is
the deepest cognate; niche construction is the most actionable for
ecosystem framing.

### What this maps to in antigen

- **Convergent evolution → the four-window convergence as evolutionary
  evidence.** Biology, programming-language theory, past-self gardening,
  ML graph-memory all converged on structural-memory-with-recognition-and-
  inheritance under different selective pressures (different problem
  spaces). Independent arrival at the same architecture is empirical
  evidence for the architecture's load-bearing structure.

- **Fitness landscapes → fingerprint-precision/recall trade-off curves.**
  V0's "dose-response curves" entry already names this; the
  fitness-landscape framing makes the trade-off topology *explicit*. A
  fingerprint can be tuned to local maxima of precision; fingerprint
  refinement is gradient-following on the precision/recall topology.
  Antigen-stdlib's per-antigen precision metrics ARE the fitness-landscape
  coordinates.

- **Niche construction → antigen-stdlib as ecosystem-niche construction.**
  This is the single most important framing this document adds. *The
  antigen-stdlib is the constructed niche that subsequent Rust development
  inherits as its selective environment.* Each antigen declared in stdlib
  shifts the selection pressure on Rust code: code that presents the
  failure-class pattern is selected against by tooling; code that resists
  it is selected for. Over time, the constructed niche reshapes the
  ecosystem's fitness landscape.

### Where the evolutionary-biology framework adds new substrate beyond biology immunology

V0 covers immunology evolution. This section's contribution is that
**evolutionary biology MORE BROADLY frames antigen-stdlib as ecosystem-
niche construction** — antigen is not just a tool that helps codebases
adapt; it is a tool that *changes the selective environment* in which
Rust codebases evolve. This shifts the project's framing from "useful
tool" to "ecosystem niche-constructor" — closer to scope.md's "first
ergonomically-adoptable instantiation of a domain-general architecture."

The framing also predicts: as antigen-stdlib accumulates, the Rust
ecosystem's *fitness landscape* shifts. New code is evaluated against a
landscape that already includes the immunity peaks. Adoption is not
just lateral spread; it is environmental construction.

### Recognition triggers in antigen

- Vision-pitch material discussing antigen's ecosystem effects can use
  niche-construction framing to explain why adoption is non-linear: each
  adopting project shifts the selective environment for subsequent
  projects.
- Convergent-evolution framing is already operational in scope.md's
  four-window convergence; this section extends the framing to
  *fifteen-window convergence* via this whole document.

### Recognition examples

- Convergent evolution producing similar eye structures across cephalopods
  + vertebrates is the canonical biological case. The four-window
  convergence on structural memory is the same shape: same problem, same
  selection pressure, same architectural answer, independent origins.
- Niche construction: human agricultural environments selecting for
  amylase gene duplications. Antigen-stdlib selecting for code patterns
  that don't trigger fingerprint matches is the same shape, different
  substrate.

---

## 3. Ecology — niche partitioning + keystone species + ecosystem resilience

### Field synopsis

Ecology studies population-level + ecosystem-level dynamics:

- **Niche partitioning**: competing species divide resources to coexist.
  Different niches reduce direct competition; biodiversity is preserved
  by structural differentiation rather than by exclusion.

- **Keystone species**: species whose impact on ecosystem structure is
  disproportionately large relative to their abundance. Removal causes
  cascade effects across many other species. Examples: sea otters,
  beavers, predators that cap herbivore populations.

- **Ecosystem resilience**: the ability of an ecosystem to recover from
  disturbance. Diverse species pools absorb shocks better; functional
  redundancy means species loss is less catastrophic when other species
  can fill the niche.

### Structural-identity test

| Criterion | Niche partitioning | Keystone species | Ecosystem resilience |
|---|---|---|---|
| Same fail-mode without architecture | partial (monoculture vulnerability) | yes (disproportionate-impact removal cascades) | yes (low-diversity ecosystems collapse on shock) |
| Same recovery shape | yes (structural differentiation) | partial (disproportionate-impact retention) | yes (functional redundancy across species) |
| Same routing pattern | yes (niche coordinates as fingerprints) | partial (impact propagation graphs) | yes (cross-species functional substitutability) |

**Verdict: full structural identity for niche partitioning and ecosystem
resilience; partial for keystone species.**

### What this maps to in antigen

- **Niche partitioning → antigen-vs-clippy-vs-tests-vs-formal-verification
  ecosystem coexistence.** The eight first-principles failure-classes
  occupy distinct niches in the recognition ecosystem. Antigen does not
  compete with clippy (compose-don't-compete, posture §3); it occupies a
  distinct niche — *named-failure-class memory across time and
  inheritance*. Each verification tool occupies a structurally different
  niche; none replaces another.

- **Keystone species → load-bearing antigens.** Some antigens, once
  declared, produce disproportionate ecosystem effects. The hypothetical
  `PolarityInvertedClassMeet` antigen, if propagated through stdlib, would
  prevent a category of bugs that cascade across many codebases. The
  framing predicts: post-A5, antigen-stdlib will contain a small number of
  keystone antigens with disproportionate impact, and a long tail of
  niche-specific antigens with localized impact. Stdlib prioritization
  follows keystone identification.

- **Ecosystem resilience → witness pluralism as functional redundancy.**
  ADR-002's commitment to multiple witness types (test, proptest, phantom,
  formal verification, lint) is functional redundancy. If one witness
  category becomes unavailable (e.g., kani is deprecated), other witness
  types can absorb the load. The ecosystem is resilient because no single
  witness is load-bearing.

### Where the ecology framework adds new substrate beyond biology immunology

- **Population-level dynamics** are absent from immunology-as-individual-
  organism framing. Ecology operates at the population/ecosystem scale —
  precisely the scale relevant to "antigen-stdlib accumulating as
  ecosystem expertise." V0's herd-immunity entry is one instance;
  ecology generalizes.

- **Stability-vs-resilience distinction**: ecology distinguishes between
  *stability* (resistance to disturbance) and *resilience* (recovery
  from disturbance). Antigen's design is a resilience strategy, not a
  stability strategy: failure-classes will keep emerging; the architecture
  is about *recovering* through pattern-naming, not about *preventing*
  pattern emergence.

### Recognition triggers in antigen

- Stdlib prioritization should ask "which antigens are keystone (high
  impact, low abundance) vs. niche (low impact, high specificity)?" The
  ecology framework predicts the question.
- Adoption-pitch material can use ecosystem-resilience framing for
  enterprise audiences who think in terms of risk management.

### Recognition examples

- Sea otters keeping urchin populations in check, preserving kelp
  forests = `PolarityInvertedClassMeet` keeping a category of class-meet
  bugs in check, preserving lattice-correctness across a swath of code.
- Coral reef diversity providing resilience to bleaching events =
  witness-pluralism providing resilience to verification-tool deprecation.

---

## 4. Information theory — error-correcting codes + Hamming distance + redundancy

### Field synopsis

Shannon (1948), Hamming (1950), and successors developed formal theory of
reliable information transmission over noisy channels:

- **Error-correcting codes**: redundancy added to messages so that errors
  can be detected and corrected. Forward error correction encodes the
  message in a way that allows the receiver to recover the original even
  when some bits are corrupted.

- **Hamming distance**: the minimum number of bit-flips needed to
  transform one valid codeword into another. A code with minimum
  distance d can detect d-1 errors and correct ⌊(d-1)/2⌋ errors.

- **Signal-to-noise ratio**: the trade-off between message bandwidth and
  reliability. Stronger codes use more redundancy and reduce effective
  bit rate, but improve resilience.

### Structural-identity test

| Criterion | Error-correcting codes |
|---|---|
| Same fail-mode without architecture | yes (silent corruption indistinguishable from valid signal) |
| Same recovery shape | yes (redundancy lets receiver recover from local errors) |
| Same routing pattern | partial (decoding is single-stage; antigen has propagation) |

**Verdict: partial cognate.** Information theory illuminates the
*per-instance* recognition problem (is this message corrupted? is this
code an antigen instance?) but does not directly map to inheritance /
propagation. Still, the per-instance illumination is precise.

### What this maps to in antigen

- **Hamming distance → fingerprint discriminability.** Two failure-classes
  whose fingerprints have small "Hamming distance" (i.e., overlap heavily
  in structural features) will produce false positives — the recognition
  cannot distinguish them. V0's cross-reactivity entry names this
  biologically; information theory makes it formal: antigen-stdlib needs
  a *minimum-fingerprint-distance* discipline analogous to minimum Hamming
  distance, so that fingerprints don't ambiguously fire on related-but-
  distinct failure-classes.

- **Redundancy as resilience → witness pluralism + verification tier
  gradient.** Multiple witness types attached to a single antigen are
  redundancy. ADR-013's phantom-type witnesses + ADR-002's external-tool
  witnesses + ADR-016's `verified_at` re-attestation form an error-
  correcting code over the immunity-claim space. A single witness can
  fail (compile error, tooling regression, stale proof); multiple
  redundant witnesses make the immunity claim correctable rather than
  losable.

- **Signal-to-noise → fingerprint precision/recall + tolerance.** The
  fingerprint engine emits a signal (match) over a noisy channel
  (codebase). Tolerance (ADR-011) is the operational acknowledgment that
  the channel has noise: some matches are not real signal. The signal-to-
  noise framing predicts that tolerance count + rationale count per
  antigen are themselves measurable quality metrics.

### Where the information-theory framework adds new substrate

- **Discriminability is a formal property, not just a desideratum.** V0's
  cross-reactivity entry names the problem; information theory says how
  to *measure* it. Future stdlib quality work can adopt explicit
  fingerprint-distance metrics rather than relying on adversarial-test
  intuition.

- **Channel capacity bounds.** There is a theoretical upper bound on how
  much failure-class memory a fingerprint engine can carry per byte of
  declaration. As antigen-stdlib grows, fingerprints will need to encode
  more disambiguation — the framework predicts the architectural cost
  rather than letting it surprise the project.

### Recognition triggers in antigen

- When stdlib accumulates to the point where antigen-overlap becomes a
  real ergonomic issue, the Hamming-distance framing makes the
  disambiguation discipline formal.
- When witness-pluralism adoption produces real cases of "kani regression
  invalidated my proof but my proptest still holds," the
  error-correcting-code framing makes the resilience visible to users.

### Recognition examples

- ECC RAM uses Hamming codes to detect/correct single-bit errors silently
  in computer memory. Antigen with witness-pluralism makes single-witness
  loss recoverable silently in immunity claims.
- Reed-Solomon codes (used in CDs, DVDs, QR codes) handle burst errors via
  block-level redundancy. Witness-tier-gradient (ADR-005 Amendment 3) is
  block-level honesty about which redundancy layer is actually doing the
  work.

---

## 5. Semiotics — Peirce's icon/index/symbol triad + sign-to-symbol elevation

### Field synopsis

Charles Sanders Peirce (1839-1914) developed a triadic theory of signs
that distinguishes three modes by which a sign refers to its object:

- **Icon**: refers by *resemblance* (a portrait resembles its subject; a
  diagram resembles its referent's structural relationships). Iconicity
  is intrinsic — the icon shares qualities with its object.

- **Index**: refers by *factual connection* (smoke is index of fire;
  footprint is index of an animal's passage). Indexicality requires a
  causal/contiguous relation in the world.

- **Symbol**: refers by *interpretive habit or convention* (the word
  "tree" refers to trees by social convention, not by resemblance or
  causal connection). Symbolicity is purely conventional — there is no
  intrinsic or causal connection between signifier and signified.

The progression icon → index → symbol is a hierarchy of *abstraction*
and *generativity*: symbols permit categorical reasoning; indices permit
specific reference; icons permit similarity-based recognition.

### Structural-identity test

| Criterion | Peirce's triad |
|---|---|
| Same fail-mode without architecture | yes (un-symbolized patterns are non-categorically-shareable) |
| Same recovery shape | yes (symbol creation makes pattern categorically shareable) |
| Same routing pattern | partial (symbol-via-convention vs symbol-via-declaration) |

**Verdict: partial cognate at the per-sign level; full cognate at the
*sign-to-symbol elevation* level.** This is the deepest semiotic framing
for antigen.

### What this maps to in antigen

- **Fingerprint-as-pattern is iconic.** The fingerprint resembles the code
  pattern it matches — it is structurally similar to the failure-class
  shape. `name: matches('*Class')` resembles the named-class pattern by
  shape.

- **Fingerprint-match-on-actual-code is indexical.** The match is a
  factual-causal connection between the fingerprint and the code site —
  "this code site contains the structural feature the fingerprint is
  about." The match is not the pattern in the abstract; it is the
  pointer-to-instance relation.

- **`#[antigen(name = "...")]` is symbolic.** The named declaration is
  Peirce's symbol — a categorical referent established by convention
  (the project's substrate). Once named, `PolarityInvertedClassMeet`
  refers categorically to the failure-class regardless of which icon
  (which fingerprint variant), which index (which code site), or which
  witness (which verification tool).

- **The fingerprint → match → declaration → witness chain is a
  Peircean ladder**. From iconic structure (the fingerprint shape)
  through indexical pointing (the match) to symbolic naming (the
  antigen) to symbolic-conventional verification (the witness, which is
  a *named* tool with conventional semantics). Each tier is a Peirce-
  shaped abstraction.

### Where the semiotics framework adds new substrate

- **Implicit-to-explicit elevation (posture §5) is a sign-to-symbol move
  in Peirce's vocabulary.** Implicit memory (developer mind) is iconic at
  best — a fading internal resemblance. Antigen converts the implicit to
  symbolic: a name, a referent, a categorically-shareable artifact. The
  ADR-004 posture is structurally identical to Peirce's "thirdness"
  category — symbols-as-conventions are the substrate of categorical
  thought.

- **The four-window convergence is a meta-symbolic move.** When biology,
  programming-language theory, past-self gardening, and ML graph-memory
  converge on the same architecture, they are independently constructing
  the same *symbol* — the architecture itself becomes referent. Antigen-
  the-project is the act of making that symbol explicit-and-named in
  the Rust ecosystem.

### Recognition triggers in antigen

- When ecosystem-outreach material explains why "naming the failure-class
  matters," the Peirce framework gives the explanation: naming is the
  shift from icon/index to symbol; it is what makes the pattern
  categorically shareable across contexts.

- When ADR-001 Amendment 1's C1-C8 commitments need a meta-framing for
  manuscript material, Peirce's triad provides it: C1-C8 is the
  enumeration of antigen's symbolic surface, the categorical referents
  the project commits to making explicit.

### Recognition examples

- A photograph of a face is iconic; the security-camera footage tagged
  with timestamp + location is indexical; the police-report case-number
  referring to the suspect is symbolic. The case-number permits
  cross-jurisdictional categorical reference; the photograph alone does
  not. Antigen names are case-numbers for failure-classes.

---

## 6. Knowledge management — Nonaka SECI tacit/explicit conversion

### Field synopsis

Ikujiro Nonaka and Hirotaka Takeuchi developed the SECI model (1995)
based on studies of Japanese corporate innovation in the 1980s-90s. The
model formalizes how organizational knowledge cycles between two modes:

- **Tacit knowledge**: cannot be fully articulated; acquired through
  practice, shared experience, and apprenticeship. Lives in individual
  minds and shared workplace context.

- **Explicit knowledge**: expressible in words, formulas, documents.
  Communicable across distance and time without face-to-face
  transmission.

The SECI cycle has four conversion modes:

1. **Socialization** (tacit → tacit): shared experience transmits tacit
   knowledge between individuals.
2. **Externalization** (tacit → explicit): tacit knowledge is articulated
   into communicable form.
3. **Combination** (explicit → explicit): explicit knowledge is integrated,
   organized, restructured.
4. **Internalization** (explicit → tacit): explicit knowledge is absorbed
   back into individual practice and intuition.

Organizational learning depends on all four conversions; missing any one
breaks the cycle.

### Structural-identity test

| Criterion | SECI |
|---|---|
| Same fail-mode without architecture | yes (tacit-only knowledge dies with the carriers) |
| Same recovery shape | yes (externalization to explicit form is the architectural move) |
| Same routing pattern | yes (combination + internalization are propagation/inheritance) |

**Verdict: full structural identity.** SECI is one of the closest non-
biological cognates for antigen's whole architecture, because both
fields are explicitly about knowledge persisting past its original
carriers.

### What this maps to in antigen

- **Tacit knowledge ≈ implicit failure-class memory** (developer minds,
  AI session context, mentorship). Decays at carrier turnover (per
  scope.md "Carriers that drift" enumeration).

- **Externalization (tacit → explicit) ≈ writing an `#[antigen]`
  declaration.** This IS the SECI externalization move, structurally.
  The team has tacit knowledge of a failure-class (we keep hitting this);
  someone externalizes to `#[antigen(name, fingerprint, summary,
  references)]` form.

- **Combination (explicit → explicit) ≈ antigen-stdlib accumulation +
  cross-crate `#[descended_from]` propagation.** Explicit antigens are
  organized, refined, related, propagated through a dependency graph.

- **Internalization (explicit → tacit) ≈ developer intuition shaped by
  scan output and IDE annotations.** As `cargo antigen scan` runs over
  time, developers internalize the failure-classes; future code writing
  is informed by tacit awareness derived from explicit feedback.

- **Socialization** ≈ the original failure-class transmission via
  mentorship, code review, war stories. Antigen does not eliminate
  socialization; it ensures the cycle continues even when socialization
  fails (carrier turnover).

### Where the knowledge-management framework adds new substrate beyond biology

- **The full cycle vs the half-cycle.** Biology gives antigen the
  *externalization + combination* halves of SECI (tacit → explicit;
  explicit accumulation). Knowledge management adds the
  *internalization* half: how externalized knowledge becomes tacit
  practice. This predicts that A6's IDE integration is structurally
  load-bearing for the SECI cycle, not just an ergonomic adjuvant.
  Without internalization, antigen-stdlib would accumulate explicit
  knowledge that is never absorbed into tacit practice — knowledge
  management calls this *knowledge stagnation*.

- **The cycle is iterative, not unidirectional.** SECI is a spiral, not
  a one-way pipeline. Antigen-stdlib's evolution is a SECI spiral:
  tacit experience → explicit antigen → combined ecosystem → internalized
  intuition → new tacit experience that motivates the next antigen.
  V0's "vaccine modalities" entry hints at this; SECI makes the spiral
  explicit.

### Recognition triggers in antigen

- A6 IDE integration framing material can explicitly invoke
  internalization: "antigen turns the explicit failure-class declaration
  back into tacit developer awareness via inline IDE annotation."
- Methodology-paper material on JBD-team-with-substrate discipline maps
  cleanly onto SECI: the team operates the externalization + combination
  half explicitly and structurally; substrate-over-memory is the
  discipline that prevents the cycle from collapsing back to tacit-only.

### Recognition examples

- Toyota Production System's externalization of tacit machine-shop
  expertise into explicit standard-work documents is the historical
  prototype. Antigen-stdlib is the same shape for Rust ecosystem
  failure-class expertise.

---

## 7. Complex adaptive systems — Holland's signals + boundaries + emergence

### Field synopsis

John H. Holland's complex-adaptive-systems (CAS) framework, especially in
*Signals and Boundaries* (2012), formalizes systems characterized by:

- **Many adaptive agents** with local interactions
- **Signals** that propagate information across the system
- **Boundaries** that delineate semi-permeable subsystems (cells,
  organizations, ecosystems)
- **Emergence**: system-level behavior that is not reducible to component-
  level behavior
- **Coevolution**: agents and environment shape each other simultaneously

CAS produces hierarchical organization through nested signal/boundary
arrangements. Niches act as semi-permeable boundaries; smells, visual
patterns, and (in ecosystems) chemical signals drive coordination.

### Structural-identity test

| Criterion | Complex adaptive systems |
|---|---|
| Same fail-mode without architecture | yes (no signals → no coordination across boundaries) |
| Same recovery shape | yes (named signals + nested boundaries enable coordination) |
| Same routing pattern | yes (signals propagate hierarchically across boundary layers) |

**Verdict: full structural identity.** CAS is the most general framework
on this list; it subsumes much of biology, ecology, and economics under
one architectural lens.

### What this maps to in antigen

- **`#[antigen]` declarations are signals.** They propagate across crate
  boundaries (which are semi-permeable), across team boundaries (humans
  + AI agents reading the codebase), across time boundaries (sessions
  cycle but signals persist).

- **Crate / module / function boundaries are CAS boundaries.** Each is
  semi-permeable in different ways. `#[descended_from]` propagation is
  one specific signal-crossing-boundary pattern; cross-crate scan (A3)
  is another.

- **Antigen ecosystem is a CAS at the meta level.** Projects + their
  dependencies + their developers + the AI agents reading them = a
  complex adaptive system. Antigen provides the *signaling
  infrastructure* for that system. Without antigen, the system has
  signals (commits, PR reviews, mentorship) but they decay at
  boundary-crossings (carrier turnover); with antigen, the signals are
  *structural* and persist across boundaries.

- **Emergence**: The four-window convergence + the colonization-ratio
  finding (8/5 = 160% from scope.md) are emergent properties of the
  team-with-substrate operating over A1+A2 — properties that no single
  agent or document encodes, but that emerge from the coordination
  pattern.

### Where the CAS framework adds new substrate

- **The boundary/signal/emergence triad makes antigen's ecosystem effects
  predictable.** When v0.1 ships and adoption begins, the project should
  expect emergent properties not encoded in any single ADR — this is a
  CAS prediction. The framework predicts what *kind* of properties will
  emerge: hierarchical signal-propagation, boundary-spanning coalitions,
  novel signal forms.

- **Coevolution framing**: antigen and Rust idioms will coevolve. As
  antigen-stdlib accumulates, Rust idioms will shift to avoid fingerprint
  matches; antigen-stdlib will then need to refine fingerprints to
  capture new instances. CAS predicts the dynamics; recognition-not-
  design (posture §2) lets the project ride them rather than legislate
  them.

### Recognition triggers in antigen

- Manuscript material discussing antigen's ecosystem effects (especially
  the foundational paper post-v0.2.0) can use CAS framing to predict
  rather than just describe the dynamics.
- Stdlib release-cadence guidance (V0's "Mutation rates" entry) becomes
  formal under CAS: signal evolution rates are a measurable property
  with structural drivers.

### Recognition examples

- Stock markets, ant colonies, immune systems, and ecosystems all share
  CAS structure. Antigen ecosystem is the same shape — different
  substrate, same architecture.

---

## 8. Cybersecurity — MITRE ATT&CK + CVE/NVD + threat-intelligence taxonomies

### Field synopsis

Cybersecurity has two well-developed structural-memory artifacts:

- **CVE (Common Vulnerabilities and Exposures)** + **NVD (National
  Vulnerability Database)**: a globally-shared registry of named
  vulnerabilities. Each CVE has a unique identifier (e.g., CVE-2021-44228
  for log4shell), affected versions, severity scores (CVSS), and links to
  patches. CVE entries propagate through software supply chains: a CVE
  in a transitive dependency affects every dependent project.

- **MITRE ATT&CK**: a globally-accessible knowledge base of *adversary
  tactics and techniques* based on real-world observations. Each
  technique has a structured ID (e.g., T1059.001 for malicious
  PowerShell), tactical category (Initial Access, Execution, etc.),
  documented adversary usage, and detection guidance. ATT&CK is a
  taxonomy of attack-pattern *shapes*, distinct from CVE's instance-
  level vulnerability registry.

### Structural-identity test

| Criterion | CVE/NVD | MITRE ATT&CK |
|---|---|---|
| Same fail-mode without architecture | yes (re-exploitation of unnamed vulnerabilities) | yes (re-defense against unnamed attack patterns) |
| Same recovery shape | yes (named vulnerability + patches) | yes (named technique + detections) |
| Same routing pattern | yes (CVE propagation through dep graphs) | yes (technique propagation through threat reports) |

**Verdict: full structural identity for both.** Cybersecurity is **the
single most direct cross-domain instantiation of antigen's architecture
in non-biological substrate.**

### What this maps to in antigen

- **CVE ≈ instance-level antigen analog.** A CVE is a named instance of
  a vulnerability in a specific package version. `#[presents]` markers
  are the antigen analog of CVE entries — instance-level, vulnerable code.

- **NVD ≈ antigen-stdlib + project-specific antigen registry.** NVD
  accumulates CVEs across the ecosystem; antigen-stdlib + downstream
  declarations accumulate antigens across the Rust ecosystem.

- **MITRE ATT&CK ≈ `#[antigen]` taxonomy.** ATT&CK is structurally
  closest to antigen-the-project: a *taxonomy of named pattern-classes*
  (not instance-level), with structural fingerprints (in ATT&CK,
  procedural descriptions; in antigen, syn-AST patterns), with
  inheritance relationships (sub-techniques like T1059.001 descend from
  T1059), with witness-like artifacts (detection guidance).

- **CVSS scoring ≈ severity-weighted antigen prioritization.** V0's
  "triage" entry names the need; CVSS provides the prior art for how a
  named-failure-class registry handles severity at scale.

- **Supply-chain propagation ≈ A3 cross-crate scan.** A CVE in a
  dependency 5 levels deep affects the top-level project; the same
  shape is what A3 implements for antigen propagation.

### Where the cybersecurity framework adds new substrate

This is the section where this document earns its keep. Several
architectural questions antigen will face have direct prior-art answers:

- **How does antigen-stdlib release cadence work?** CVE assignment +
  NVD enrichment workflows are 25+ years of ecosystem operational
  experience. CNAs (CVE Numbering Authorities), embargo windows,
  responsible-disclosure timelines — antigen will eventually face
  analogs and can adopt patterns rather than re-derive them.

- **How are antigen IDs structured?** CVE format (`CVE-YYYY-NNNNN`) +
  ATT&CK format (`TNNNN.NNN` with sub-techniques) are two different
  successful patterns. Antigen IDs are currently named-string
  (`PolarityInvertedClassMeet`); a structured-ID layer may eventually
  be load-bearing for cross-ecosystem coordination.

- **How does fingerprint-vs-instance separation work at scale?** ATT&CK
  separates *technique* (the pattern class) from *procedure* (specific
  observed instances) from *detection* (the witness). Antigen's
  parallel structure is `#[antigen]` (technique) + `#[presents]`
  (procedure) + `witness` (detection). The cybersecurity framework
  predicts ergonomic friction points (e.g., when a single technique has
  many procedures with subtly different shapes) and how to handle them
  (sub-technique decomposition).

- **What is the social structure of stdlib contribution?** ATT&CK is
  curated by MITRE with community contribution; CVE has CNAs
  distributed across the ecosystem. Both have governance models that
  antigen-stdlib could adopt or adapt.

- **What is the framing for "this is a new category of structural
  verification"?** Cybersecurity is the field-level peer that has been
  doing exactly this for the security domain. Adoption-pitch material
  for antigen can frame as "what CVE/MITRE ATT&CK did for security
  knowledge, antigen does for failure-class knowledge more generally."

### Recognition triggers in antigen

- A5+ when stdlib release cadence becomes load-bearing, the CVE/CNA
  governance model is recognition substrate.
- Vision-pitch v2 + manuscript material can directly invoke the
  CVE/ATT&CK comparison as the most precise non-biological cognate.
- When a single antigen accumulates many distinct fingerprint variants
  (W6a synthesis pass surfacing different structural forms), ATT&CK's
  sub-technique decomposition is the architectural prior.

### Recognition examples

- log4shell (CVE-2021-44228) propagated through millions of projects
  via supply chain — exactly the propagation A3 cross-crate scan
  enables for antigens.
- T1059.001 PowerShell sub-technique was created when T1059 (Command
  and Scripting Interpreter) accumulated enough sub-pattern instances
  to warrant decomposition. A future antigen-stdlib evolution will
  likely produce analogous decompositions; ATT&CK shows what they
  look like.

---

## 9. Aviation safety — NTSB blameless post-mortem + structural recommendations

### Field synopsis

The National Transportation Safety Board (NTSB) and analogous bodies in
healthcare have developed blameless investigation practices since the
mid-20th century:

- **Blameless investigation**: incidents are investigated without
  assigning fault; the goal is *understanding* and *prevention*, not
  liability. The investigation finds *contributing factors*, not
  blameworthy individuals.

- **Structural recommendations**: investigation outputs are *structural
  changes* (procedures, training requirements, equipment modifications,
  regulations) — not individual sanctions. Each recommendation closes a
  specific failure pathway identified by the investigation.

- **Cumulative safety culture**: the ecosystem accumulates structural
  changes over decades. Aviation's exceptional safety record is
  attributed to this cumulative cultural artifact.

The discipline migrated to software via Google SRE's blameless-
postmortem culture (Beyer et al. 2016) and adjacent industry practice.

### Structural-identity test

| Criterion | NTSB-style post-mortem |
|---|---|
| Same fail-mode without architecture | yes (without structural recommendations, lessons stay tacit) |
| Same recovery shape | yes (structural changes prevent failure-class recurrence) |
| Same routing pattern | yes (recommendations propagate via regulation + training) |

**Verdict: full structural identity.** This is the closest professional-
practice cognate to antigen's discipline.

### What this maps to in antigen

- **Antigen declaration ≈ NTSB structural recommendation.** Each
  `#[antigen]` is a structural change to the codebase's recognition
  surface, derived from a specific failure instance, designed to prevent
  recurrence.

- **References field ≈ NTSB report linkage.** The `references = [...]`
  field on `#[antigen]` (issue numbers, ADRs, commits) is the antigen
  analog of NTSB report citations — provenance from instance to
  declaration.

- **Witness ≈ NTSB recommendation verification.** A safety
  recommendation is implemented + audited; an antigen has a witness
  that verifies immunity.

- **Antigen-stdlib ≈ aviation regulatory ecosystem.** The accumulated
  structural recommendations of decades of NTSB investigation IS the
  aviation safety culture. Antigen-stdlib is the analogous accumulated
  structural carrier for Rust ecosystem failure-classes.

### Where the aviation framework adds new substrate

- **Origin-incident discipline.** NTSB practice requires that every
  recommendation be tied to a specific incident (or pattern of
  incidents). Antigen's `references` field formalizes the same
  discipline. The aviation framework is the *historical proof* that
  this discipline scales to ecosystem-wide cumulative knowledge over
  decades.

- **Blameless culture is structurally load-bearing, not just ergonomic.**
  NTSB's effectiveness depends on the blameless framing — investigators
  get truth from witnesses because there are no liability consequences.
  Antigen's adoption depends on the same: developers must feel safe
  declaring antigens about their own past mistakes. The framing is *not*
  decorative; it shapes whether the architecture works at all.

- **The asymmetry between aviation/medical blamelessness and software
  blamelessness** (per the Google SRE source: "where human error is a
  factor, the FAA may revoke a pilot's license") is itself instructive:
  software's ability to be *fully* blameless is a structural advantage
  the architecture can lean into.

### Recognition triggers in antigen

- Vision-pitch material for enterprise/regulated-industry audiences can
  use NTSB cognates directly. "Antigen brings aviation-grade
  failure-class memory to the Rust ecosystem."
- Stdlib contribution guidelines should formalize the
  origin-incident-required discipline as NTSB practice does.

### Recognition examples

- Post-Tenerife (1977) collision: standardized phraseology + crew
  resource management. The recommendation was structural; it propagated
  globally; airspace coordination has not had a similar collision since.
  The pattern is structurally identical to "post-DeterminismClass-
  polarity-bug, declare `PolarityInvertedClassMeet` antigen, propagate
  through stdlib, similar bugs prevented across ecosystem."

---

## 10. Pattern languages — Christopher Alexander's architecture + GoF descent

### Field synopsis

Christopher Alexander's *A Pattern Language* (1977) introduced the idea of
*patterns* in physical architecture: each pattern is a problem + context
+ structural solution + relations to other patterns. 253 patterns in the
original book formed a *pattern language* — a network of patterns that
call upon one another.

The Gang of Four book *Design Patterns* (Gamma, Helm, Johnson, Vlissides
1994) imported this structure to software design (Singleton, Observer,
Factory, etc.). Ward Cunningham invented the wiki specifically as a
collaboration substrate for documenting software patterns.

### Structural-identity test

| Criterion | Pattern languages |
|---|---|
| Same fail-mode without architecture | yes (re-derivation of structural solutions across projects) |
| Same recovery shape | yes (named patterns capture structural answers) |
| Same routing pattern | partial (patterns are descriptive, not enforced; antigen has tooling enforcement) |

**Verdict: partial cognate.** Pattern languages are the architectural
predecessor to antigen-the-discipline — same recognition move (name
recurring structural answers), different enforcement model (descriptive
documentation vs. structural type-system carriers).

### What this maps to in antigen

- **GoF design patterns ≈ project-level antigens that don't ship as
  stdlib.** Each GoF pattern names a recurring structural answer; each
  antigen names a recurring failure-class. The recognition discipline is
  the same (name what recurs); the modality differs (GoF promotes good
  patterns; antigen warns against bad patterns).

- **Alexander's pattern relationships ≈ `#[descended_from]`
  inheritance.** Alexander explicitly designed patterns to call upon
  one another; antigen explicitly supports inheritance. The Alexander
  network structure is the prior art for what cross-antigen relations
  look like at scale.

- **Wiki + WikiWikiWeb origin ≈ collaborative-substrate publication for
  pattern collections.** Antigen-stdlib + the GitHub repo + future
  community contribution flows are the modern instantiation of the same
  collaborative pattern-curation move Cunningham invented for GoF.

### Where the pattern-language framework adds new substrate

- **Patterns as a *language*, not a list.** Alexander's original
  framing emphasized that patterns *combine* — the language is
  generative, not just descriptive. Antigen's commitment to
  composition (multiple antigens at one site, inheritance graphs,
  family relationships) inherits this generative-language framing.
  V0's "comorbidity" entry (multiple antigens manifesting together)
  is the antigen analog of pattern-language composition.

- **Pattern languages have a known failure mode: descriptive without
  enforcement.** GoF patterns famously *are not enforced* by tools; they
  are encouraged by documentation. The result: real-world adoption is
  inconsistent, and pattern-misuse persists. Antigen's structural
  enforcement (cargo subcommand + macros) is the architectural
  improvement over GoF — *named patterns + tooling* > *named patterns
  alone*.

### Recognition triggers in antigen

- Stdlib documentation framing can invoke GoF as the closest
  software-engineering precedent: "GoF for failure-classes, with
  cargo enforcement."
- When community contribution flows mature, Cunningham's WikiWikiWeb
  history is recognition substrate for governance.

---

## 11. Cumulative culture — Tomasello's ratchet effect + transmission fidelity

### Field synopsis

Michael Tomasello and collaborators (2009, 2012) developed the *cultural
ratchet effect* framework explaining why human culture is *cumulative*
in ways no other species' is:

- Cultural innovations *stay* in the population (ratchet) rather than
  drifting back to baseline.
- The mechanism: high *transmission fidelity* — language, teaching,
  imitation — that preserves innovations precisely enough to be improved
  upon by the next generation.
- Without high fidelity, innovations drift and are lost; with it, each
  generation builds on the prior.

Chimpanzees have culture (regional tool-use traditions); they don't have
*cumulative* culture (nothing builds; each generation re-invents at
baseline). The species-level difference is transmission fidelity.

### Structural-identity test

| Criterion | Cumulative culture |
|---|---|
| Same fail-mode without architecture | yes (innovations drift back to baseline without high-fidelity transmission) |
| Same recovery shape | yes (high-fidelity transmission preserves innovations across generations) |
| Same routing pattern | yes (teaching + language + imitation = inheritance mechanisms) |

**Verdict: full structural identity.** Cumulative-culture theory is
*about exactly what antigen is about* — preserving innovations
(failure-class recognitions) across generations (sessions, contributors,
AI agents) via high-fidelity carriers (structural macros).

### What this maps to in antigen

- **Implicit failure-class memory ≈ chimpanzee culture.** Real, but
  doesn't ratchet. Each session re-invents at baseline. The bug
  patterns of 2024 are still ship-able in 2026 because the carrier was
  low-fidelity.

- **Antigen as high-fidelity carrier ≈ language + teaching.** The
  `#[antigen]` declaration is the high-fidelity transmission mechanism.
  Once declared, the failure-class recognition propagates across
  context-window boundaries, across team-handoffs, across
  AI-agent-generations *without re-derivation*.

- **Antigen-stdlib growing over years ≈ cumulative cultural innovation.**
  Each declared antigen is a contribution to the cumulative cultural
  artifact; the artifact ratchets — antigens stay declared, future
  innovations build on them.

### Where the cumulative-culture framework adds new substrate

- **Transmission fidelity as a measurable property.** Tomasello's group
  developed experimental paradigms for measuring transmission fidelity.
  Antigen could in principle measure its own: declare an antigen, hand
  the codebase to a fresh-context agent, ask whether the agent
  understands the failure-class without being told. High fidelity if
  yes; low if not. This is recognition substrate for empirical defense.

- **Ratchet vs drift distinction makes adoption stakes precise.** Without
  antigen, the Rust ecosystem's failure-class knowledge is in
  drift-mode: innovations (lessons learned) keep being lost to carrier
  turnover. With antigen, the ecosystem becomes ratchet-mode:
  innovations stay. The framing is non-decorative — it names the
  qualitative shift in ecosystem dynamics.

- **The four-window convergence is itself a cumulative-culture artifact.**
  Biology accumulated immune-system architecture over 500M years.
  Programming-language theory accumulated structural-property
  verification over 50+ years. Past-self gardening accumulated
  pattern-recognition insights over months. ML graph-memory accumulated
  in 2026. Each window is a cumulative-cultural ratchet in its
  domain. Antigen is the artifact that *crosses the windows* — the
  meta-ratchet.

### Recognition triggers in antigen

- Foundational-paper material (post-v0.2.0, per scope.md trajectory) can
  use cumulative-culture framing as the deepest non-biological
  defense of why structural memory matters: *humans only have
  cumulative civilization because of high-fidelity transmission;
  software ecosystems will only have cumulative failure-class culture
  because of structural carriers.*
- AI-dev-tooling paper can use the cumulative-culture frame to
  contrast antigen with fine-tuning-based knowledge embedding:
  fine-tuning is low-fidelity (training-data residue, model-version
  drift); antigen is high-fidelity (substrate-grounded, model-
  independent).

### Recognition examples

- The accumulated stock of human technology over 200,000 years (stone
  tools → agriculture → metallurgy → electronics → AI) IS the
  cumulative-cultural ratchet. Antigen-stdlib growing over years +
  decades IS the same shape, restricted to one specific domain
  (Rust failure-class knowledge).

---

## 12. Indigenous epistemologies — oral transmission + intergenerational pattern

### Field synopsis

Indigenous knowledge systems across the world (Vhavenda, Tsimane,
Hawaiian, Coast Salish, Navajo, etc.) share certain structural features
in how they preserve and transmit knowledge across generations:

- **Multi-modal carriers**: knowledge is encoded in stories, ceremony,
  song, dance, ritual, place-naming, plant/animal ecology — distributed
  across redundant channels rather than localized in one document or
  practice.

- **Layered meaning**: stories carry multiple interpretive levels;
  different audiences (children, adults, elders, initiates) extract
  different content from the same surface. The layering is a feature,
  not noise.

- **Authoritative carriers**: respected individuals (elders, hereditary
  chiefs, medicine people) are responsible for accurate transmission and
  contextualization. They are not just knowledge-holders; they are
  *trust-boundary validators* — they decide which content is shared
  with which audience under what conditions.

- **Land-embedded knowledge**: substantial knowledge is encoded in
  place-relations rather than abstracted from them. The land itself is
  a carrier.

### Structural-identity test

| Criterion | Indigenous epistemology |
|---|---|
| Same fail-mode without architecture | yes (knowledge dies at carrier death without structural transmission) |
| Same recovery shape | yes (multi-modal redundancy + authoritative validation preserves) |
| Same routing pattern | partial (intergenerational propagation is structurally similar; substrate-embedding is novel relative to antigen's text-based substrate) |

**Verdict: full structural identity for the recognition + memory +
inheritance triad; indigenous frameworks ADD substrate-embedding as a
fourth principle that text-based antigen does not currently encode.**

### What this maps to in antigen

- **Multi-modal carriers ≈ multi-witness pluralism.** Indigenous systems
  preserve knowledge across stories, ceremony, song — multiple redundant
  channels. Antigen preserves immunity claims across test, proptest,
  phantom-type, formal-verification, lint — multiple redundant witness
  types. The structural move is the same (redundancy as resilience).

- **Layered meaning ≈ tier-aware audit honesty (ADR-005 Amendment 3).**
  Different audiences extract different content from the same surface.
  Antigen's audit reports the tier its verification work actually
  supports — not a stronger one. The audience determines the depth.

- **Authoritative carriers ≈ rationale-required field (posture §6) +
  substrate-over-memory.** Trust-boundary validators in indigenous
  systems are structurally analogous to the rationale field: someone is
  responsible for the justification, and the justification is checkable.

### Where the indigenous-epistemology framework adds new substrate

- **Substrate-embedding as a fourth principle.** Indigenous knowledge is
  not just *recorded in* the substrate; it is *embedded in* the
  substrate (place-names, plant-relations, ecosystem-knowledge). For
  antigen, the analogous direction is that fingerprints + immunity
  claims live IN the code (as macros), not in a separate registry.
  This is already operational (per ADR-001's structural-not-documentary
  posture), but the indigenous framework makes the principle more
  vivid: *the carrier and the carried must live in the same substrate*.

- **The danger of extraction-without-context.** Indigenous knowledge
  systems are damaged when content is extracted from cultural context
  (e.g., medicinal plant knowledge extracted into pharmacology
  literature without the practice context). The framework predicts:
  antigen-stdlib content extracted into manuscripts/papers/tutorials
  without the ecosystem-practice context is similarly degraded. This
  is a posture-class implication for how antigen is *talked about*
  outside the substrate.

- **Plurality of traditions, not singular framework.** Indigenous
  epistemologies are *plural* — Vhavenda is not Tsimane is not
  Coast Salish. The field-level lesson: antigen-stdlib should not
  presume a single "correct" failure-class framing for all Rust
  contexts. Domain-specific antigens (embedded systems, web servers,
  cryptography) may need framing that respects the plurality of
  domain practice.

### Recognition triggers in antigen

- Methodology-paper material can invoke indigenous-epistemology framing
  as the deepest non-Western prior art for the principle *the carrier
  must live in the substrate of practice*. The framing widens the
  paper's reach beyond Western academic traditions.

### Recognition examples

- Tsimane oral-tradition research (cited prior art): elders'
  storytelling expertise *increases* with age — knowledge transmission
  is bidirectional and lifecycle-distributed, not just downward.
  Antigen-ecosystem analog: senior contributors' value to
  antigen-stdlib comes from accumulated pattern-recognition, not just
  technical knowledge.

---

## 13. Stigmergy — environmental signaling + memory-in-substrate

### Field synopsis

*Stigmergy* (Grassé 1959; Theraulaz & Bonabeau 1999) is coordination
without direct communication — agents leave signals in the environment;
other agents detect and respond to the signals. Ant pheromone trails are
the canonical example:

- Ants searching for food deposit trail-pheromone-A; ants returning
  follow trail-pheromone-B.
- Trails strengthen with use, evaporate without it.
- The pheromone field is *external memory*: simple agents with no
  individual record of the colony's path-history nonetheless
  collectively converge on optimal routes via the substrate-encoded
  signal.

Stigmergy generalizes beyond ants: termite mounds, traffic flow, social-
media trending, distributed-system gossip protocols, version-control
systems all operate stigmergically.

### Structural-identity test

| Criterion | Stigmergy |
|---|---|
| Same fail-mode without architecture | yes (without environmental signal, agents repeat exploration) |
| Same recovery shape | yes (signal-in-substrate enables convergence without central control) |
| Same routing pattern | yes (signal strength + decay + reinforcement = recognition propagation) |

**Verdict: full structural identity.** Stigmergy is **the deepest
architectural cognate for substrate-over-memory**, the discipline that
makes antigen's project-level coordination work.

### What this maps to in antigen

- **Substrate-over-memory IS stigmergy.** The campsite logbook + git
  history + on-disk docs are the pheromone field. Each agent (Claude
  instance, human contributor) leaves signals in substrate; other
  agents detect and respond without direct coordination. Validation 4
  in the A1 closure narrative (the team passed three "ratification
  complete" signals through routing on the basis of agent context, not
  substrate-grounded check) is *exactly* the failure mode stigmergy
  warns against: when agents respond to other agents' signals rather
  than to substrate, coordination collapses.

- **`#[antigen]` declarations ≈ pheromone trails for recognition.**
  Each declaration is a persistent environmental signal. Future
  agents (developers, AI assistants, scan tooling) detect the signal
  and respond. The signal does not require synchronous communication
  between agents.

- **Stale tolerance + verified_at decay ≈ pheromone evaporation.**
  Without the decay term, stigmergy would saturate (every trail
  reinforced indefinitely). Antigen's `verified_at` (ADR-016) is the
  decay term — claims have a freshness. Without decay, witness
  pluralism would saturate (every claim "verified once" is equivalent
  to "verified eternally," which violates sub-clause F).

### Where the stigmergy framework adds new substrate

- **Coordination-tier substrate-over-memory mitigation has a name.**
  The A1 closure narrative names the discipline ("every 'X is complete'
  routing must include a substrate-grounded check name"); stigmergy
  gives it a 60-year theoretical grounding. The discipline is not ad
  hoc; it is a specific instance of a domain-general coordination
  architecture.

- **Stigmergy explains why the JBD-team-with-substrate works at scale.**
  Direct communication scales as O(N²) with team size; stigmergic
  coordination scales as O(N) because each agent only interacts with
  the substrate, not with all other agents. Antigen's coordination
  pattern (campsite logbook + ADRs + docs/expedition + memory.md)
  operates stigmergically by design — the team scales because the
  substrate carries the coordination load.

- **Signal evaporation is load-bearing, not just hygiene.**
  Pheromone-trail evaporation prevents lock-in to suboptimal paths.
  ADR-016's `verified_at` evaporation prevents lock-in to stale
  immunity claims. The framework predicts: *every persistent signal
  in antigen needs a decay term, or it will eventually mislead*.
  This is recognition substrate for future-ADR territory.

### Recognition triggers in antigen

- Methodology-paper material on JBD-team-with-substrate can directly
  invoke stigmergy as the theoretical grounding for substrate-over-
  memory. The discipline becomes a *named architectural practice*,
  not just a project-specific reflex.
- Future ADR territory: when antigen primitives are added that don't
  have decay terms (e.g., `references` field, `summary` field), the
  stigmergy framework asks "does this signal need evaporation?" The
  answer is sometimes no (provenance claims should not evaporate);
  sometimes yes (claims about freshness should).

### Recognition examples

- The Linux kernel coordination model (no central architect; coordination
  via patches, mailing lists, git history) is stigmergic at scale.
  Antigen-stdlib + cross-crate antigen propagation will be the same shape.
- Wikipedia editorial coordination is stigmergic. Antigen-stdlib
  contribution governance will likely converge on similar patterns.

---

## 14. Bayesian inference — prior/posterior + rare-event reasoning

### Field synopsis

Bayesian statistics formalizes belief-updating under uncertainty:

- **Prior distribution**: representation of belief before new evidence.
- **Posterior distribution**: representation of belief after incorporating
  new evidence.
- **Bayes' theorem**: prescribes how to combine prior + likelihood of
  evidence under hypotheses to produce posterior.

Bayesian methods are the dominant framework for engineering reliability
modeling under rare events: a component with one or zero observed failures
in its operational history still permits calibrated belief about future
failure rates, given an informative prior.

### Structural-identity test

| Criterion | Bayesian inference |
|---|---|
| Same fail-mode without architecture | partial (purely-empirical reasoning struggles with rare events) |
| Same recovery shape | partial (informative prior compensates for sparse data) |
| Same routing pattern | partial (prior → posterior → next prior is iterative) |

**Verdict: partial cognate, but illuminating.** Bayesian framework is
not the architecture itself; it is the *reasoning engine* used over the
architecture.

### What this maps to in antigen

- **Antigen-stdlib ≈ informative prior for codebase failure-class
  reasoning.** A codebase that imports antigen-stdlib starts with a
  *prior belief* that certain failure-classes exist in the world. As
  scan + audit produces evidence (matches, immunity claims), the
  codebase's posterior shifts. The mathematical framework for
  reasoning about partially-observed risks IS Bayesian.

- **Rare-event reasoning ≈ failure-class reasoning at scale.** Most
  individual codebases will never personally experience most failure-
  classes (e.g., a small data-pipeline crate may never have a
  Drop-impl panic). But the existence of antigen-stdlib gives them a
  *prior* — failure-classes the broader ecosystem has seen, even if
  this codebase has not. Bayesian framework predicts: codebases with
  good priors (from stdlib) handle rare events better than codebases
  with flat priors (no failure-class knowledge).

- **`verified_at` re-attestation ≈ posterior-becomes-next-prior.**
  Each round of audit + re-verification updates the system's belief
  state. The Bayesian iterative-update structure matches ADR-016's
  temporal substrate.

### Where the Bayesian framework adds new substrate

- **Confidence calibration as ecosystem-level metric.** Bayesian methods
  produce *calibrated* beliefs (probability claims that match
  empirical frequencies). Antigen-stdlib could in principle produce
  calibrated immunity claims: "codebases with witness W for antigen X
  have empirical failure rate Y." This is recognition substrate for
  future stdlib quality metrics.

- **Rare-event-with-informative-prior is the structurally hard case.**
  Most individual project codebases will see most antigen-stdlib
  failure-classes zero times; antigen-stdlib's value is precisely in
  bringing the prior. Bayesian framework formalizes why this is
  load-bearing rather than overhead.

### Recognition triggers in antigen

- Stdlib quality framework can adopt Bayesian-calibration metrics as
  precision/recall metrics mature.
- Adoption-pitch material for risk-conscious audiences (regulated
  industry, safety-critical software) can use Bayesian framing
  directly: antigen-stdlib is the prior; project-specific antigens
  are the likelihood; verified immunity is the posterior with low
  failure rate.

### Recognition examples

- Aerospace component reliability: a turbine blade's failure
  probability is estimated via Bayesian methods even when individual
  blades have no observed failures, by leveraging informative priors
  from broader fleet data. Antigen-stdlib brings the same architectural
  shape to Rust ecosystem reliability.

---

## 15. Philosophy of science — paradigm shift + normal-science accumulation

### Field synopsis

Thomas Kuhn's *The Structure of Scientific Revolutions* (1962) describes
science as alternating between:

- **Normal science**: cumulative puzzle-solving within an established
  paradigm. Structural assumptions are shared; investigators work on
  problems the paradigm makes tractable.

- **Crisis**: anomalies accumulate that the paradigm cannot accommodate.

- **Revolution / paradigm shift**: a new paradigm reorganizes the
  field's structural assumptions; old anomalies dissolve under new
  framing; new puzzles become tractable that were unaskable before.

Subsequent philosophy of science (Lakatos, Feyerabend) refined the
account but the cumulative-paradigm + paradigm-shift dichotomy persists.

### Structural-identity test

| Criterion | Philosophy of science |
|---|---|
| Same fail-mode without architecture | partial (paradigm-less science is incoherent, but this is more a meta-claim) |
| Same recovery shape | partial (paradigm articulation is a recognition move) |
| Same routing pattern | yes (normal-science-within-paradigm = inheritance + accumulation) |

**Verdict: partial cognate at the meta level; illuminating for
positioning antigen as paradigm-shift candidate.**

### What this maps to in antigen

- **Antigen as paradigm-shift candidate**: scope.md's "domain-knowledge
  memory safety as a fourth structural property of secure-by-default
  Rust development" is explicitly a paradigm-shift claim. Memory
  safety, type safety, thread safety are the existing paradigm;
  domain-knowledge memory safety is a fourth axis the field has not
  organized around.

- **Antigen-stdlib accumulation as normal science.** Each antigen
  declaration is a normal-science contribution within the antigen
  paradigm: cumulative puzzle-solving once the paradigm is
  established. The first 10-20 stdlib antigens are paradigm-
  establishing; the next thousands are normal-science accumulation.

### Where the philosophy-of-science framework adds new substrate

- **Paradigm-shift discipline.** Kuhn's account predicts: paradigm
  shifts are *resisted* until anomalies accumulate too obviously.
  Antigen will face this. The four-window convergence is the project's
  defense against premature paradigm-shift claim (the architecture is
  not arbitrary; it has independent multi-window grounding) AND the
  argument that the shift is overdue (the architecture has been
  emerging in four fields independently for decades).

- **Normal-science vs revolutionary-science cadence.** The project's
  own work cadence will shift: A1+A2 are paradigm-establishing
  (revolutionary mode); A3+A4+A5 are normal-science (puzzle-solving
  within established paradigm). The recognition-not-design (posture
  §2) discipline is suited for normal-science mode; the
  anti-YAGNI / structurally-guaranteed-need (posture §4) discipline
  is suited for revolutionary mode. The framework predicts: as
  antigen matures, posture-§2 dominance grows and posture-§4 fades.

### Recognition triggers in antigen

- Foundational-paper material (post-v0.2.0) can use the paradigm-shift
  framing directly. The contribution is a new structural property
  the field has not previously organized around.
- Post-A5 retrospective framing material: as antigen-stdlib stabilizes
  and contribution cadence shifts to normal-science, the framework
  predicts the team's experience of the work changing.

---

## 16. Epistemic logic — common-knowledge proofs + the muddy-children puzzle

### Field synopsis

Epistemic logic — the modal logic of knowledge — is the formal study of
*who knows what, and who knows what others know*. The thread relevant to
antigen runs through Halpern & Moses's "Knowledge and common knowledge in
a distributed environment" (1990), Halpern's *Reasoning About Knowledge*
(1995, with Fagin, Moses & Vardi), and the much older muddy-children
puzzle as the canonical demonstration.

The field formalizes a hierarchy of knowledge states for a group of
agents:

- **Each agent knows X individually**: every agent has X in their private
  belief state.
- **Everyone knows X (E¹X)**: each agent knows X, but they may not know
  that the others know.
- **Everyone knows that everyone knows X (E²X)**: each agent knows that
  each other agent knows.
- **Common knowledge (CX)**: the infinite hierarchy — E¹X ∧ E²X ∧ E³X ∧ …
  At common knowledge, each agent knows X, knows everyone else knows,
  knows everyone knows everyone knows, ad infinitum.

The field's central structural finding: **common knowledge cannot be
established by private bilateral exchange**. No finite number of pairwise
messages between agents can produce common knowledge. The structural
answer is *public announcement that all agents witness simultaneously* —
when the announcement happens in a way every agent observes (and observes
every other agent observing), common knowledge is established in one
step. The muddy-children puzzle is the canonical demonstration: three
children with mud on their foreheads cannot collectively conclude their
own muddiness from private observation alone, but a public announcement
("at least one of you has mud") triggers a chain of inductive updates
that resolves it.

The field has 35+ years of development across distributed systems
(consensus protocols, CRDTs), social epistemology (pluralistic ignorance,
common-ground theory), and game theory (coordination problems with
common-knowledge constraints).

### Structural-identity test

| Criterion | Epistemic logic / common-knowledge proofs |
|---|---|
| Same fail-mode without architecture | yes (agents hold correct private beliefs while common knowledge is absent — V5/V7 of the comprehension-drift family at the formal level) |
| Same recovery shape | yes (public announcement that all agents witness simultaneously establishes common knowledge in one inductive cascade) |
| Same routing pattern | yes (inductive propagation from the shared signal — each agent updates, others update based on the first update, etc.) |

**Verdict: full structural identity.** All three criteria pass. Window 16
joins the structural-identity-passing windows; the count moves from 15 to
at-least-16, and the framework's lower-bound property (per the document
header) is operationally demonstrated by this addition.

### What this maps to in antigen

- **`#[antigen]` declarations as public announcements**: the named
  failure-class declaration is a *public* artifact in the codebase —
  every agent (developer, AI assistant, scan tooling, audit pass) that
  reads the codebase observes the declaration, observes that other
  agents observe it, and the observation is itself observable. That is
  the muddy-children-puzzle structural answer instantiated at the
  recognition surface.

- **`#[descended_from]` propagation as inductive cascade**: when an
  antigen declaration propagates through inheritance, each descendant
  inherits the public knowledge AND inherits the fact that other
  descendants will inherit it. The cascade is structurally the
  E¹X → E²X → E³X → … inductive chain that establishes common
  knowledge across the dependency graph.

- **Substrate-over-memory discipline as common-knowledge protocol**:
  the team-coordination practice of *checking substrate before claiming
  state* IS the formal answer epistemic logic prescribes for
  distributed-knowledge problems. The "X is complete — `git grep
  ADR-NNN docs/decisions.md` returns matches" pattern from CLOSURE.md
  Validation 4 is the muddy-children move at team scale: don't rely on
  private bilateral exchange ("pathmaker said it's done" → "navigator
  relays" → "observer logs"); rely on public-substrate inspection
  every agent can witness.

### Connection to the comprehension-drift family

V5 (stakeholder mental-model divergence) and V7 (mutual-update
assumption coupling) of the comprehension-drift family are *instances*
of common-knowledge failure at the team-coordination scope. The
literature-grounding doc (campsite trail) cited Halpern & Moses 1990 +
Prentice & Miller 1993 for these variants; this window makes the
citation structurally load-bearing rather than incidental.

The unreachability claim for V5/V7 ("static analysis cannot establish
common knowledge; it is private bilateral inspection of substrate, not
multi-agent witnessed announcement") is now grounded in a direct
structural-identity-passing window. Static analysis tools inspect each
file in isolation; even when they aggregate findings, the aggregation
is a *single agent's* private inspection of the substrate. Common
knowledge requires *multi-agent simultaneous witnessing* — a property
no static-analysis architecture can supply, per the muddy-children
proof. The structural answer is what antigen already provides:
shared-substrate declarations that all agents read as public
announcements.

V8b (verification-rigor decay) is also adjacent: ritual-form-without-
substance preserves the surface of public announcement while the
substantive content erodes — the *appearance* of common knowledge
without the underlying CX state. Vaughan's normalization-of-deviance
predicted this; epistemic logic gives the formal account of why
appearance-without-substance fails (the inductive cascade requires
substantive observation, not just observation-of-the-form).

### Why this window matters for the foundational paper

This window makes the V5/V7 unreachability argument rest on a direct
structural-identity-passing window rather than indirect support. The
foundational paper's strongest positioning claim — that some failure
modes are *structurally* unreachable by static analysis and the
architectural answer (shared-substrate declarations) is the only
available response — now has 35+ years of formal-logic substrate
behind it, not just team-cognition or social-epistemology framing.

### Recognition triggers in antigen

- **Foundational paper drafting** (post-v0.2.0): Halpern & Moses 1990
  and the muddy-children proof become primary citations for the
  V5/V7 unreachability claim. The argument structure: *static
  analysis is structurally a private bilateral inspection mechanism;
  common knowledge cannot be established by private bilateral
  exchange; therefore static analysis cannot detect failures that
  require common-knowledge coordination; the structural answer is
  shared-substrate declarations as public announcement.*
- **Methodology paper**: substrate-over-memory's grounding in
  stigmergy (per scientist's routing) extends naturally into
  epistemic logic — both are formal accounts of how distributed
  agents coordinate via shared substrate rather than private
  exchange. Bridge-posture material.
- **AI dev tooling paper**: the muddy-children proof applied to
  AI-team coordination is the contrast with fine-tuning — fine-tuning
  is private bilateral state (model weights); it cannot establish
  common knowledge across model instances or human-AI teams. Antigen
  declarations are the public announcement that all agents witness.

### Recognition examples

- **The muddy-children puzzle itself**: three children, three
  forehead-mud states, no private resolution possible — but the
  father's public announcement triggers the inductive cascade. Direct
  structural cognate of antigen's substrate-over-memory practice.
- **Distributed consensus protocols** (Lamport, Paxos, Raft):
  three-phase commit + acknowledgment chains exist *because* common
  knowledge requires multi-agent witnessed announcement. Antigen's
  ADR-text-as-public-substrate is the same shape, instantiated at the
  team-coordination layer rather than the protocol layer.
- **CLOSURE.md Validation 4 recovery protocol**: navigator's "X is
  complete — `git grep ...`" pattern is the muddy-children move
  applied operationally. The substrate-grounded check is the public
  announcement; agent-to-agent relay is the private bilateral
  exchange that fails to establish common knowledge.

---

## Where the cross-domain map goes silent (honest boundaries)

Per naturalist's biology-as-instrument framing in V0: the silence is
informative. Fields where the structural-identity test fails *zero*
criteria — i.e., the field's framework genuinely doesn't map to
antigen's architecture — are themselves load-bearing evidence that the
architecture has shape.

### Fields that are silent

- **Pure mathematics** (set theory, category theory, topology in the
  abstract): mathematics provides the *substrate* in which formal
  verification operates (Liquid Haskell types, kani proofs are
  mathematical objects), but mathematics-itself does not have the
  *recognition + memory + inheritance over time* structure. A theorem
  is timeless; antigen claims have temporal substrate (`verified_at`).
  Mathematics is one tier deeper than antigen; antigen-the-architecture
  is not mathematics.

- **Physics** (thermodynamics, statistical mechanics, quantum field
  theory): physics has *laws*, not *recognized patterns*. A failure
  mode in physics is a violation of conservation; not a learnable
  recurrence. The framework genuinely doesn't apply.

- **Pure logic** (propositional, first-order): logic provides the
  *grammar* for reasoning about failure-classes (e.g., the witness
  type-system invariants), but logic does not have the *empirical-
  pattern-recognition* layer that antigen's fingerprint engine
  embodies.

- **Formal language theory** (Chomsky hierarchy, regular/context-free/
  context-sensitive): the AST grammar antigen operates over is
  formally a context-sensitive matter, but formal-language-theory
  itself is about the substrate, not the architecture-over-substrate.

These silences are evidence of antigen's architectural specificity:
*recognition-with-memory-and-inheritance over an empirically-evolving
substrate*. Pure mathematics, physics, logic, and formal-language theory
all lack one or more of those modifiers (empirical, evolving,
substrate-bound). The architecture lives in the intersection where
those modifiers all hold.

### Fields with weak partial-cognate that don't earn their own section

- **Game theory / mechanism design**: provides framing for the
  *strategic interactions* between antigen's contributors (why
  contribute to stdlib? why declare antigens about your own
  mistakes?), but the strategic-interaction layer is one tier above
  the architecture, not part of it.

- **Economics of information goods**: relevant to ecosystem-adoption
  dynamics (positive externalities, network effects), but not part of
  the architectural substrate.

- **Linguistics (syntax / phonology)**: linguistics is closer to
  formal-language theory than to recognition-with-memory; the
  structural-identity test fails most criteria.

- **Anthropology of science** (Latour, actor-network theory): provides
  framing for how antigen-as-artifact mediates relationships between
  human and AI agents and codebases. The framework is illuminating but
  is a *meta-account* of the architecture, not the architecture itself.

These are catalogued here so the catalog is honest: not every field
illuminates antigen; some fields illuminate the *context around*
antigen but not the architecture. The distinction matters for
manuscript trajectory (which framings appear in which papers).

---

## Cross-domain convergence findings

After reviewing fifteen fields and naming silences in four+ more, the
following structural findings emerge:

### Finding 1 — Fifteen-window convergence

The four-window convergence in scope.md (biology, programming-language
theory, past-self gardening, ML graph-memory) is a special case of a
broader fifteen-window convergence:

| Window | Vocabulary | Architectural cognate |
|---|---|---|
| 1. Biology | antibodies, MHC, B-cell memory | *V0's spine* |
| 2. Cognitive science (schema) | schema activation | named-pattern persistence |
| 3. Cognitive science (chunking) | template hierarchy | inheritance |
| 4. Cognitive science (structure-mapping) | systematicity | relational-fingerprint matching |
| 5. Evolutionary biology | convergent evolution | independent arrival at same structure |
| 6. Evolutionary biology | niche construction | ecosystem-niche shaping |
| 7. Ecology | niche partitioning + keystone | tool-coexistence + load-bearing antigens |
| 8. Information theory | error-correcting codes | witness pluralism as redundancy |
| 9. Semiotics | symbol-formation | named declaration as symbol |
| 10. Knowledge management (SECI) | externalization | implicit-to-explicit elevation |
| 11. Complex adaptive systems | signals + boundaries | antigen as signal infrastructure |
| 12. Cybersecurity (CVE/ATT&CK) | named vulnerability/technique | direct architectural twin |
| 13. Aviation safety (NTSB) | structural recommendation | discipline cognate |
| 14. Pattern languages (Alexander/GoF) | named pattern | recognition predecessor |
| 15. Cumulative culture (Tomasello) | ratchet effect | high-fidelity transmission |
| (16. Indigenous epistemologies) | multi-modal redundancy | substrate-embedding |
| (17. Stigmergy) | environmental signal | substrate-over-memory cognate |
| (18. Bayesian inference) | informative prior | reasoning engine over architecture |
| (19. Philosophy of science) | paradigm + normal science | meta-positioning |

That is **15 fields with full structural identity, 4 more with partial
cognates, 4+ silent fields**. The four-window claim in scope.md is a
*lower bound*. The actual convergence is wider.

The implication for ADR-003 (biological metaphor load-bearing): the
metaphor is one window onto an architecture so widely-distributed that
calling it "biological" undersells it. The architecture is **domain-
general**, present wherever recognition + memory + inheritance over
empirically-evolving substrate is needed.

This is an empirical defense of the project's foundational claim that
the architecture is not arbitrary. Fifteen+ fields could not all
independently have arrived at it by accident.

### Finding 2 — The closest non-biological cognate is cybersecurity

CVE/NVD + MITRE ATT&CK is the architectural twin of antigen, restricted
to the security domain. Almost every architectural question antigen
will face has prior-art answers in the cybersecurity ecosystem:

- ID structure (CVE-YYYY-NNNNN, T1059.001)
- Severity scoring (CVSS)
- Supply-chain propagation (transitive CVE dep tracking)
- Sub-pattern decomposition (ATT&CK sub-technique structure)
- Governance models (CNAs, MITRE curation)
- Disclosure timelines, embargo windows
- Detection guidance attached to technique declarations
- Cross-ecosystem coordination

This is recognition substrate for ecosystem governance work post-A5.
Antigen does not need to reinvent how named-failure-class registries
operate at scale; cybersecurity has 25+ years of operational experience.

The framing is also *load-bearing for adoption-pitch*: the closest
analog audiences already know is CVE/MITRE ATT&CK. "What CVE/ATT&CK
did for security knowledge, antigen does for failure-class knowledge
more generally" is a precise framing.

### Finding 3 — The deepest substrate-over-memory grounding is stigmergy

The A1 closure narrative's Validation 4 (substrate-over-memory caught
the team's own coordination failure) is given a 60-year theoretical
foundation by stigmergy theory. The discipline is not project-specific;
it is a specific instance of a domain-general coordination architecture.

This shifts methodology-paper framing: instead of presenting
substrate-over-memory as a JBD-team practice, present it as the project's
adoption of stigmergic coordination, with citations across biology
(Grassé), distributed systems (gossip protocols), and version control
(git as stigmergic substrate).

### Finding 4 — Implicit-to-explicit elevation is Peirce's sign-to-symbol move

Posture §5 (implicit-to-explicit elevation) gains a precise theoretical
articulation via semiotics: the elevation is a Peircean sign-to-symbol
move. Implicit memory is iconic at best (fading internal resemblance);
externalized antigens are symbolic (categorical, conventionally-
shareable referents). The ADR-004 posture is structurally identical to
Peirce's "thirdness" category.

This is recognition substrate for foundational-paper material: the
architectural posture is not idiosyncratic; it is a specific instance
of a Peirce-shaped abstraction with 100+ years of philosophical
articulation.

### Finding 5 — Cumulative culture is the deepest non-biological argument for why this matters

Tomasello's cumulative-culture / ratchet-effect framework is the most
load-bearing non-biological framing for *why* structural failure-class
memory matters at the ecosystem level. The framing is precise:

- **Without antigen**: Rust ecosystem failure-class knowledge is in
  drift mode (chimpanzee-culture-like). Innovations are lost across
  carrier turnover.
- **With antigen**: Rust ecosystem failure-class knowledge ratchets
  (human-culture-like). Innovations persist across generations.

The qualitative shift — from drift-mode to ratchet-mode — is the
ecosystem-level claim antigen embodies. The cognitive-science
literature documents this shift as the difference between human and
non-human cognition; antigen brings the shift to a software ecosystem.

This is the strongest possible argument for antigen's foundational
claim, and it is *not* metaphor. It is a structural-identity claim.

### Finding 6 — The architecture's signature is no fixed point

Across all fifteen+ fields, the architecture has the same operational
signature: *the discipline that catches the discipline's failure must
be structural too*. No fixed point. The recursion is generative, not
pathological.

- **Biology**: B-cell memory of past pathogens IS the system; immune
  failures cause autoimmunity which is itself a B-cell failure;
  Tregs are the structural answer to Treg failure-modes. Each tier
  recurses.
- **Cumulative culture**: high-fidelity transmission is what preserves
  innovations; failure of high-fidelity transmission is what loses
  innovations; *teaching how to teach* is the recursive structural
  answer. Each tier recurses.
- **Stigmergy**: signal-in-substrate enables coordination; stale
  signals mislead coordination; signal-decay is the structural answer;
  *decay-of-the-decay-mechanism* is the next-tier question. Each tier
  recurses.
- **Antigen**: structural memory of failure-classes IS the system;
  failure-of-structural-memory (Validation 4 substrate-currency
  failure) is the failure mode; substrate-over-memory is the
  structural answer; substrate-currency-of-substrate-over-memory is
  the next-tier question. Each tier recurses.

This is the depth-shift discipline (posture §7) generalized: across the
fifteen+ fields, every architecture that successfully implements
recognition-with-memory-and-inheritance has the no-fixed-point property.
The recursion is the discipline, not a bug in the discipline.

The implication: when antigen substrate inevitably surfaces failure
modes of antigen's own coordination, the response is *not* "the
architecture has a flaw"; the response is "the architecture is doing
what successful instances of this architecture in fifteen other fields
do — generating its own next tier." The recursion is evidence of
structural soundness, not weakness.

---

## What this map is for

Each cognate listed here is **forward substrate**, not commitment.
Per ADR-006: when a real adoption pressure surfaces a need that
matches a cognate listed here, the team recognizes the prior art rather
than re-deriving it.

This document is V1. Future deepening:

- **Naturalist** can deepen the biology-spine V1 (V0 deepening) in
  parallel; this document is the cross-domain sibling.
- **Scout** can find prior-art partial-instantiations within specific
  cognate fields (e.g., academic papers on schema-theory tooling that
  could be ergonomic prior art for IDE integration in A6).
- **Scientist** can integrate the fifteen-window convergence into
  manuscript trajectory: tool-paper uses cybersecurity + aviation
  cognates for adoption-pitch; foundational-paper uses cumulative-
  culture + Peirce + stigmergy cognates for the architectural-claim
  substrate; methodology-paper uses stigmergy + SECI cognates for the
  team-discipline framing.
- **Aristotle** can note candidate posture-class entries surfacing from
  cross-domain framing — particularly stigmergy as the theoretical
  grounding of substrate-over-memory.

V2 of this document, if needed, would deepen each cognate with
specific academic citations, prior-art instantiations of related
tooling within each field, and recognition triggers tied to specific
A3+ adoption pressures.

---

## Closing posture

Antigen is not a Rust tool that happens to use biological vocabulary.
Antigen is the first ergonomically-adoptable instantiation in a Rust
ecosystem of an architecture that fifteen+ academic fields have been
independently developing for decades to centuries.

The biological framing is one window. The deepest non-biological
windows are cybersecurity (architectural twin), cumulative culture
(why-this-matters argument), and stigmergy (substrate-over-memory
foundation).

The discipline (per ADR-006): no speculative instantiation. This map
is recognition substrate — when adoption surfaces a real instance, the
team finds the answer here rather than re-deriving it. The map's
existence does not commit the project to building any specific
primitive on any specific timeline.

Per the four empirical validations from A1 closure: the substrate has
earned its scope. Per this V1 deepening: the substrate's *theoretical
grounding* has fifteen-window depth. Both supports compound.

---

## Appendix A — Cybersecurity cognate deepening: ecosystem governance prior art

The cybersecurity cognate (§8) is the closest non-biological architectural
twin and the most actionable for ecosystem governance work. This appendix
enumerates specific operational patterns from CVE/NVD, MITRE ATT&CK, and
RustSec that antigen-stdlib will eventually face analogs of, so the team
can recognize prior art rather than re-derive when the time comes.

Per ADR-006: this is recognition substrate, not a commitment to adopt any
specific pattern. The point is to make the prior art surveyable.

### A.1 — RustSec is the directly-comparable Rust ecosystem prior

The single most directly-applicable prior art is **RustSec** (rustsec.org +
the rustsec/advisory-db GitHub repo + cargo-audit + the rustsec crate). It
is structurally what antigen-stdlib will resemble at scale, restricted to
the security-vulnerability subset of failure-classes:

- **Structured advisory format**: each advisory has package, dates, IDs,
  patched versions, references — precisely the shape of a future
  antigen-stdlib entry.
- **PR-based contribution**: vulnerabilities reported by opening pull
  requests against the rustsec/advisory-db GitHub repo. This is the
  community-curation pattern; antigen-stdlib can adopt directly.
- **OSV format export**: RustSec exports to OSV (osv.dev), and the GitHub
  Advisory Database imports RustSec advisories. This is **multi-format
  interoperability** as a known operational pattern. Antigen-stdlib should
  expect to need an interoperability format from inception, not retrofit.
- **Tool consumption** (cargo-audit, the rustsec crate): the database is
  consumed by tooling; the database format and the tool API co-evolve.
  Antigen-stdlib + cargo-antigen audit are the parallel structure.

**Recognition trigger**: post-A5, when antigen-stdlib reaches publication
maturity, RustSec's contribution governance is the closest Rust-ecosystem
operational analog. Likely the right invitation pattern is "if you've
contributed to RustSec, you'll recognize most of antigen-stdlib's
contribution flow."

### A.2 — CNA federation is the scaling-governance pattern

The CVE program scales to thousands of vulnerabilities per year via CNA
(CVE Numbering Authority) federation: a hierarchical structure where
higher-level CNAs assign blocks of CVE IDs to lower-level CNAs, who use
their block to issue IDs within their domain.

- **Program Root → Root → Sub-CNA hierarchy.** Each level has the same
  rules; higher levels can sanction lower levels for non-compliance.
- **Block-based ID assignment**: rather than central issuance per
  vulnerability, blocks are pre-allocated. This is *stigmergic
  governance* (per §13): each CNA operates on a substrate (their assigned
  block) without per-vulnerability central coordination.
- **De-centralized within rule-bound structure**: the goal is federation
  of governance ability while maintaining program-wide consistency.

**Recognition trigger**: if antigen-stdlib accumulates to multi-thousand
scale (post-A6+), federation will become structurally necessary. Domain-
specific extensions (antigen-stdlib-embedded, antigen-stdlib-web,
antigen-stdlib-crypto) are candidate federation boundaries. The CVE CNA
model predicts the architectural shape such federation would take.

### A.3 — Disclosure/embargo policies are not optional

Every CNA must publish a disclosure (embargo) policy. The policy describes
when CVE IDs are assigned, when publication happens, communication
guidelines, and timelines. The disclosure framework is *load-bearing for
contributor trust* — without it, reporters do not know what to expect.

**Recognition trigger**: antigen-stdlib does not currently have an
analogous concept (failure-classes are not under-embargo the way security
vulnerabilities are), but post-A5 may surface analogs:
- **Pre-publication antigens**: an antigen recognized internally before
  the underlying failure-class is widely understood. Embargo equivalent:
  hold the antigen until the broader pattern is understood enough that
  publication doesn't cause confusion.
- **Cross-project coordination**: an antigen that affects multiple
  unrelated projects may need coordination before publication so all
  affected projects can prepare.
- **Witness-not-yet-available antigens**: an antigen recognized but
  without an executable witness yet. Tier-honesty (ADR-005 Amendment 3)
  already handles this at the audit reporting layer; the disclosure-
  policy framing makes it ecosystem-coordinatable.

### A.4 — CVSS as severity rubric template

The Common Vulnerability Scoring System (CVSS) maps each vulnerability to
a vector of base/threat/environmental metrics, producing a 0-10 numeric
score that maps to None/Low/Medium/High/Critical tiers. The vector is
*structured* (e.g., `AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H` for log4shell)
so that score derivation is reproducible.

Two Rust crates exist: `cvssrust` and `cvss-rs` — prior art for how the
Rust ecosystem already encodes CVSS structurally.

**Recognition trigger**: V0's "triage" + "dose-response curves" entries
both predict that severity-weighting becomes load-bearing post-stdlib.
CVSS is the prior art for how a named-failure-class registry handles
severity *reproducibly* (vector-based derivation, not just tier
assignment). Antigen-stdlib could in principle adopt a CVSS-shaped
vector for failure-class severity — base metrics (e.g., manifestation
likelihood, blast radius, recoverability), environmental modifiers (e.g.,
relevance to specific project type), threat modifiers (e.g., exploit
maturity in the wild). The architectural shape is recognition substrate;
specific metric choice is future ADR territory.

### A.5 — Sub-technique decomposition criteria from ATT&CK

ATT&CK's sub-technique system (introduced 2020) is the prior art for how
a named-pattern-class taxonomy handles fingerprint variation. Specific
operational patterns:

- **One-to-one parent relationship**: each sub-technique has *exactly
  one* parent technique; no diamond inheritance. This is "complicated
  and difficult to maintain relationships" avoidance — ATT&CK explicitly
  rejected diamond inheritance to keep the model tractable.
- **Decomposition triggered by accumulation**: techniques are decomposed
  into sub-techniques when content accumulates that warrants
  finer-grained naming (Account Manipulation, Process Injection are
  cited examples). The decomposition is *recognition-driven*, not
  designed-ahead.
- **Realignment + deprecation as part of lifecycle**: techniques can be
  pruned, moved to different tactics, or deprecated. Deprecated objects
  are marked but retained; references are not invalidated retroactively.

**Recognition trigger**: when antigen-stdlib's W6a synthesis pass surfaces
sufficient structural variants of a single antigen that decomposition
becomes ergonomically necessary, ATT&CK's sub-technique pattern is the
prior. The single-parent constraint is especially load-bearing —
`#[descended_from]` already supports multi-parent inheritance, but
antigen-stdlib's *taxonomic* structure may benefit from ATT&CK-style
single-parent constraint at the stdlib-level even while
`#[descended_from]` permits richer relationships at code-level.

### A.6 — Contribution attribution and review patterns

ATT&CK's contribution governance combines:
- **Contact-before-writing**: contributors are asked to contact MITRE
  before spending time on a new technique. This prevents wasted effort
  and ensures alignment with the curation roadmap.
- **Final-product review**: the final product is run by the contributor
  who is credited as a contributor. Attribution is preserved while
  curation maintains quality.
- **Public threat-intelligence as primary source**: the data source is
  publicly-available threat intelligence + incident reporting, with
  research on new techniques included when they "closely align with
  what adversaries commonly do" — a recognition-not-design discipline.

**Recognition trigger**: antigen-stdlib contribution governance has not
been articulated yet. ATT&CK's pattern (contact-first, attributed-review,
recognition-driven curation) is recognition substrate. The pattern is
likely to be the right starting point because it already reflects
recognition-not-design discipline (posture §2) and rationale-as-
required-field (posture §6) at the social layer.

### A.7 — Multi-format interoperability is structural

Both NVD and RustSec interoperate with multiple consumer formats (OSV,
GitHub Advisories, etc.). Multi-format interoperability is *structural*
in the cybersecurity ecosystem — every consumer of vulnerability data
expects the data to be available in multiple machine-readable formats
because different tooling pipelines need different shapes.

**Recognition trigger**: antigen-stdlib's data shape will be consumed by
cargo-antigen scan/audit, future LSP integration (A6), future external
tooling. The multi-format-interoperability pattern is recognition
substrate for an *ecosystem-data-format* ADR (likely post-A5+).
Pre-emptive design for interoperability would violate ADR-006
(speculative); recognition of the pattern as forward substrate is
exactly what this appendix is for.

### A.8 — The framing arc for adoption-pitch material

The cybersecurity cognate provides a precise framing arc that adoption-
pitch material can reuse:

> *In the early 1990s, software security was tribal knowledge — every
> security team carried a personal mental catalog of attack patterns
> learned from incidents and word-of-mouth. CVE (1999) and later MITRE
> ATT&CK (2013) made that knowledge structural: named, shared, propagable
> across organizations. The Rust ecosystem currently has tribal knowledge
> for non-security failure-classes (panicking-in-Drop, polarity-inverted-
> class-meet, etc.). Antigen does for non-security failure-class
> knowledge what CVE+ATT&CK did for security knowledge: makes it
> structural, named, and propagable.*

This framing has two important properties:
1. **It uses prior art the audience already knows.** Anyone who has
   worked with CVEs or ATT&CK techniques recognizes the architectural
   move; the cognitive load to grasp antigen's framing is low.
2. **It positions antigen as a category extension, not a category
   reinvention.** The architecture is established; antigen is the
   first instantiation of the architecture for general failure-classes
   in a programming-language ecosystem. This is closer to scope.md's
   actual claim than the "another linter" framing the audience might
   default to.

**Recognition trigger**: the foundational paper (post-v0.2.0 per
scope.md trajectory) has positioning material that benefits from this
framing arc directly. The tool paper (post-v0.1.0) has a less-formal
version useful for blog posts and conference proposals.

---

## Appendix B — Field-by-field manuscript framing assignments

Different cognates serve different manuscript trajectories. This is the
academic-researcher's recommendation for which framing fits where, per
scope.md's multi-paper trajectory:

### Tool paper (post-v0.1.0)

Primary cognates: **cybersecurity (CVE+ATT&CK), aviation safety (NTSB),
RustSec**. These give the audience-familiar framing for "what category
of tool is antigen, and why does the category matter?" The empirical
defenses (biology-as-search-heuristic precision, colonization ratio, ATK
confirmation rate, tambear adoption signal) take primary load; the
cognates are the *category framing* the reader needs to understand the
empirical defenses.

### Foundational paper (post-v0.2.0 with antigen-stdlib + cross-crate)

Primary cognates: **cumulative culture (Tomasello), Peirce semiotics,
cognitive science (schema + chunking + structure-mapping + team-
cognition / shared mental models), evolutionary biology (niche
construction), philosophy of science (paradigm shift), social
epistemology (common-knowledge proofs)**.
These give the deep architectural-claim defenses that the foundational
paper's positioning requires:
- Tomasello's ratchet effect frames *why structural memory matters*
  beyond ergonomics (the qualitative shift from drift-mode to ratchet-
  mode).
- Peirce frames *why naming is load-bearing* beyond convenience (the
  iconic→indexical→symbolic elevation).
- Cognitive science frames *why fingerprint-matching works* beyond
  empirical observation (chunking + structure-mapping are the
  mechanisms). The team-cognition / shared-mental-models extension
  (Cannon-Bowers, Salas & Converse 1993; Klimoski & Mohammed 1994;
  Clark & Brennan 1991) frames *why the comprehension-drift family
  unifies as one fail-mode across ten surface forms* — the structural
  unifier finding from the comprehension-drift literature grounding.
- Social epistemology / common-knowledge proofs (Halpern & Moses 1990;
  Halpern *Reasoning About Knowledge* 1995; Prentice & Miller 1993)
  ground the *strongest positioning claim antigen has*: V5/V7 of the
  comprehension-drift family are *structurally unreachable* by static
  analysis per the muddy-children proof. Static analysis is itself
  private bilateral inspection that cannot establish common knowledge;
  the structural answer is shared-substrate declarations. The
  3-fundamental-gap → 5-fundamental-gap finding (per scout↔academic
  convergence) is foundational-paper material grounded in this
  literature.
- Niche construction frames *why ecosystem-stdlib matters* beyond
  adoption (the constructed niche reshapes the ecosystem's selective
  environment).
- Paradigm-shift framing positions the contribution at the right level
  (a new structural property the field has not previously organized
  around).

### Methodology paper (post-A2 closure)

Primary cognates: **stigmergy, knowledge management (SECI), complex
adaptive systems (Holland), organizational learning (Argyris),
substrate-of-practice (Polanyi + Lave & Wenger + Wenger-Trayner et al.
2015 + Pyrko et al. 2019)**. These give the team-discipline framings:
- Stigmergy provides theoretical grounding for substrate-over-memory.
- SECI provides the cycle framing (tacit-explicit-explicit-tacit) for
  why externalization-without-internalization fails.
- CAS provides the boundary/signal/emergence framing for why JBD-team
  scales — the team coordinates stigmergically, not through direct
  communication.
- Argyris's espoused-theory vs theory-in-use distinction (Argyris &
  Schön 1974; Argyris 1976) grounds the form-content gap cluster
  (variants 4, 8, 10 + impure 9 per scientist's three-cluster
  amendments) — the comprehension-drift sub-claim about *behavior-
  drifting-while-nominal-form-stays-fixed*. Vaughan's normalization of
  deviance (1996) extends Argyris into safety-science territory.
- Substrate-of-practice (per Appendix C citation chain) grounds the
  three-layer unification: stigmergy (coordination architecture) +
  SECI (knowledge-conversion cycle) + substrate-of-practice
  (recognition discipline) = one architecture at three operational
  layers, with communities-of-practice as connecting tissue. The
  substrate-currency posture and depth-shift discipline live across
  all three layers; bridge postures (rationale-as-required-field;
  candidate depth-shift per A3 prediction) are the connective tissue
  the manuscript should make explicit.

### AI dev tooling paper (after AI-industry comparison)

Primary cognates: **cumulative culture (high-fidelity transmission),
indigenous epistemologies (substrate-embedding), Peirce (icon-to-symbol
elevation), social epistemology (common-knowledge proofs at AI-team
scale)**. The contrast with fine-tuning-based approaches is sharpest
through these:
- Fine-tuning is low-fidelity transmission across model versions;
  antigen is high-fidelity (substrate-grounded, model-independent).
- Fine-tuning extracts knowledge from substrate of practice; antigen
  embeds knowledge IN the substrate.
- Fine-tuning operates at the iconic-resemblance level (vector-space
  similarity); antigen operates at the symbolic-categorical level
  (named declarations).
- The muddy-children unreachability claim from the comprehension-drift
  family applies directly: *fine-tuning is private bilateral state
  (the model's weights); it cannot establish common knowledge across
  model instances*. Antigen's structural declarations ARE the public
  announcement that all agents witness — Halpern & Moses 1990's
  structural answer applied to AI dev contexts. The 5-gap-variants
  → forced-externalization antibody finding (V4 loop-of-meaning drift;
  V5 stakeholder mental-model divergence; V7 mutual-update assumption
  coupling; V8b verification-rigor decay; V10 comprehension-vs-naming
  drift) is the AI-dev-tooling-paper-specific contribution: these
  variants are exactly where AI-team coordination fails without
  shared-substrate carriers. The five-variant set spans Clusters B and
  C in scientist's three-cluster structure, which is itself a finding
  — the antibody shape is determined by *substrate localizability*,
  not by which cluster the variant occupies (per scientist's cross-
  cluster antibody finding, methodology-paper-substrate.md, 2026-05-08).

### Comprehension-drift family material distribution

The comprehension-drift family was authored *after* the initial
Appendix B framing and its primary cognates (team cognition / shared
mental models, Argyris organizational learning, social epistemology /
common-knowledge proofs, systems thinking) initially had no slot
assignment. Per the recognition-not-design discipline (posture §2):
*the structure is prior; we recognize it rather than design new slots
speculatively*. The family's material distributes across the existing
four slots rather than warranting a fifth:

- **Foundational paper** absorbs the structural-unifier finding (one
  fail-mode across ten surface forms) and the 5-fundamental-gap
  cluster (V4, V5, V7, V8b, V10 structurally unreachable per
  literature prediction). The manuscript-grade positioning sentence
  (per scientist's substrate, 2026-05-08): *"There are five named
  comprehension-drift variants that static analysis fundamentally
  cannot reach: V4 (loop-of-meaning drift), V5 (stakeholder mental-
  model divergence), V7 (mutual-update assumption coupling), V8b
  (verification-rigor decay), V10 (comprehension-vs-naming drift).
  The literatures predict the unreachability — this is not contingent
  tooling absence. The structural answer in each case is shared-
  substrate declarations: the muddy-children-puzzle move generalized."*
  Cognate additions: team cognition / shared mental models, social
  epistemology / common-knowledge proofs.
- **Methodology paper** absorbs the three-cluster structure (Currency-
  lag A {1, 2, 3, 7}; Form-content gap B {4, 8a, 10 + impure 9};
  Social-tracking C {5, 6}), the V8 split (V8a iconic / V8b symbolic-
  tier per Peirce framing — V8b moves to substrate-not-localizable
  cluster), and the substrate-of-practice ↔ comprehension-drift
  unification finding. Worth noting: variant 5 is *Cluster C by
  symptom but Cluster A by mechanism* — the coordination failure is
  observable; the underlying mechanism is individual-comprehension
  failure at each agent. The cluster structure names the symptom; the
  structural unifier names the mechanism. Cognate additions: Argyris
  organizational learning + Vaughan normalization of deviance
  (form-content gap mechanism), substrate-of-practice citation chain
  (per Appendix C).
- **Tool paper** absorbs the V9 showcase variant (three Rust crates
  exist at three independent angles per scout's prior-art sweep;
  schema theory predicts the three-axis architecture per the
  literature-to-tooling-architecture-prediction finding). No new
  cognate additions; the existing tool-paper cognates suffice.
- **AI dev tooling paper** absorbs the V5/V7 unreachability claim and
  the cross-cluster forced-externalization antibody finding (V4, V5,
  V7, V8b, V10 — five variants spanning Clusters B and C; antibody
  shape is determined by *substrate localizability*, not cluster
  membership). Cognate additions: social epistemology / muddy-
  children-puzzle structural answer applied at AI-team scale.

The genuinely novel contribution scientist flagged ("gap variants with
forced-externalization antibody shape that doesn't map to any existing
slot") lives at the *architectural-claim layer*, not the manuscript-
shape layer. It is foundational-paper positioning material — the
structural argument that some failure modes are unreachable by static
analysis and the architecture's structural answer (shared-substrate
declarations) is the only available response. Distribution rather
than a fifth slot preserves recognition-not-design discipline while
locating the novelty correctly.

The cross-cluster span of the antibody finding is itself a methodology
paper finding distinct from the foundational paper's gap-existence
claim: *the antibody architecture for a failure family depends on the
structure of the drifting-thing (substrate localizability), not on the
surface-form cluster of the failure*. Naming this explicitly preempts
a reviewer asking why the same antibody applies to variants in
different clusters.

### Closing posture

This appendix is recommendation, not commitment. Scientist owns
manuscript-trajectory decisions; the appendix surfaces the cognate-to-
paper assignments that the academic-researcher has substrate confidence
in.

---

---

## Appendix C — Indigenous-epistemology cognate deepening: substrate-of-practice principle and extraction-degradation

The indigenous-epistemologies cognate (§12) was held lightly in the main
substrate because the structural-identity test came back partial-
substrate-embedding. That partial verdict deserves expansion, because
the partial is at the *substrate axis* — the very axis the project's
whole architecture is about.

This appendix grounds the §12 framing in specific academic literatures
(CARE Principles, Two-Eyed Seeing / Etuaptmumk, Haraway's situated
knowledges, Polanyi + Lave & Wenger on substrate-of-practice tacit
knowing) and develops the two §12 threads that the main pass left
sketched: **extraction-without-context as a named degradation mode**,
and **the substrate-of-practice principle** as the deepest non-Western
prior for ADR-001's structural-not-documentary commitment.

Per ADR-006: this is recognition substrate, not a commitment. The
indigenous-epistemology framework predicts specific failure patterns
in *how antigen is talked about* outside the codebase substrate; the
appendix names them so the team recognizes the patterns when they
surface.

### C.1 — Why this appendix is structurally necessary

The whole project commits to a posture: *the carrier and the carried
must live in the same substrate* (ADR-001 structural-not-documentary;
posture §5 implicit-to-explicit elevation; the substrate-over-memory
discipline; the no-fixed-point property at every operational layer).

That posture is the deepest non-decorative claim antigen makes. It is
also the claim with the *thinnest Western-academic literature
supporting it*. Western philosophy of science from Descartes onward has
generally separated knowledge (proposition) from substrate (practice);
the modern academic reflex is to extract knowledge into propositions
and then talk about the propositions. Antigen's posture inverts that
reflex.

The frameworks that support antigen's posture most cleanly come from
*outside* the Western academic mainline: indigenous epistemologies
(many traditions; substrate-embedded knowledge as the rule, not the
exception); Haraway's situated knowledges (Western feminist
epistemology making the same move from inside the mainline); Polanyi's
tacit knowledge (Western philosophy explicitly arguing that knowledge
cannot be separated from the knower's body and practice); Lave &
Wenger's communities-of-practice (organizational learning making the
same move at team scale).

The cluster of these frameworks is the substantive prior for antigen's
posture. The cybersecurity cognate (§8) is about *governance of named-
failure-class registries at scale*; this cognate is about *why the
substrate-embedding posture is load-bearing rather than aesthetic*.
Different operational scopes; same architecture.

### C.2 — Two-Eyed Seeing / Etuaptmumk as the integration framing

**Background**: Two-Eyed Seeing (Mi'kmaq: *Etuaptmumk*) was named by
Mi'kmaq Elders Albert and Murdena Marshall with Cheryl Bartlett at
Cape Breton University in the early 2000s. The principle: integrate
Indigenous and Western knowledge by *learning to see with one eye
through Indigenous knowledge and the other eye through Western
knowledge — and using both eyes together*. Not amalgamation (mashing
together); not assimilation (one swallowing the other); but
*coexistent perspective*, where each tradition's strengths inform the
other while neither is reduced.

Two-Eyed Seeing has expanded from its origin in fisheries/biology
education to applications across health research, neuroscience, and
ecological management. The framework has 25+ years of operational
experience integrating two epistemologies that the academic mainline
otherwise treats as incompatible.

**What this maps to in antigen**: the project's combination of
*biology-as-instrument* (naturalist's framing) plus *programming-
language-theory-as-instrument* (the academic CS lineage) plus *past-
self-gardening-as-instrument* plus *ML-graph-memory-as-instrument*
(scope.md's four-window convergence) is *structurally Two-Eyed Seeing
extended to four eyes*. Each window is an independent epistemological
tradition; the project doesn't reduce any to the others; it sees
*through* all of them simultaneously.

This is recognition substrate for the foundational paper's positioning.
The reflex Western framing would be: "antigen takes the biological
metaphor as inspiration and translates it into Rust." That's
assimilation-shaped; biology gets used and discarded. The Two-Eyed
Seeing framing is: "antigen sees through biology AND through
programming-language theory AND through past-self gardening AND
through ML graph-memory simultaneously, and the architecture is the
intersection-with-no-reduction." Different framing; different ethical
relationship to source traditions.

**Recognition trigger**: when foundational-paper material discusses
the biological framing's role, the Two-Eyed Seeing principle is the
most-load-bearing prior for "biology-as-instrument is not biology-as-
metaphor." The principle is older than the antigen-project's framing
and has 25+ years of operational practice supporting it.

### C.3 — CARE Principles as substrate-of-practice governance

**Background**: the CARE Principles for Indigenous Data Governance
(Collective Benefit, Authority to Control, Responsibility, Ethics)
were developed by the International Indigenous Data Sovereignty
Interest Group in 2020. They complement the more famous FAIR
Principles (Findable, Accessible, Interoperable, Reusable) for
research data — but where FAIR is *technical and people-neutral*,
CARE is *people- and purpose-oriented*.

Operational core of CARE:
- **Collective benefit**: data ecosystems should benefit Indigenous
  Peoples whose data are involved.
- **Authority to control**: Indigenous Peoples' rights to govern data
  about their territories, resources, and communities.
- **Responsibility**: data stewards have responsibilities for ethical
  use and re-use across the data lifecycle.
- **Ethics**: prioritizing Indigenous well-being in data decisions
  throughout the lifecycle.

The principles are operationalized through ICIP (Indigenous Cultural
and Intellectual Property) protocols + free-prior-informed-consent
(FPIC) requirements + explicit attribution + benefit-sharing
provisions.

**What this maps to in antigen**: the cybersecurity cognate (§8)
provides governance prior-art for *named-failure-class registries at
scale* (CVE/NVD, MITRE ATT&CK, RustSec). CARE provides governance
prior-art for **communities-of-practice authority over their own
substrate**. As antigen-stdlib accumulates community contributions,
the question of *who has authority over what* becomes load-bearing.

CARE predicts specific governance questions antigen-stdlib will face:
- Who has authority to decide what becomes a stdlib antigen vs. a
  project-specific antigen?
- How does a domain community (embedded systems, cryptography, web
  servers) preserve authority over its own failure-class framings?
- What is the equivalent of free-prior-informed-consent when an
  antigen declared in a small project gets adopted into stdlib?
  (Scope.md's "PRs to antigen-stdlib become contributions to
  collective Rust-ecosystem memory" raises this question.)
- What is the antigen analog of attribution + benefit-sharing? The
  `references` field is a partial answer; CARE predicts the question
  is broader.

The CARE framework is not a direct one-to-one fit for antigen — antigen
isn't governing personally-identifying or culturally-sensitive data —
but the *structural shape* of "communities-of-practice retain authority
over how their substrate is named and propagated" is exactly the
shape antigen-stdlib will need at A5+. Recognition substrate for that
work, not a proposal.

**Recognition trigger**: post-A5 when stdlib accumulation produces
real questions about domain-specific authority. Vision-pitch material
for community contributors can also invoke the CARE-shaped framing
as a precise alternative to "open-source means anyone can use anything
without context" — that framing is not the only available
relationship-to-substrate.

### C.4 — Haraway's situated knowledges: the Western-academic prior for substrate-bound objectivity

**Background**: Donna Haraway's "Situated Knowledges: The Science
Question in Feminism and the Privilege of Partial Perspective" (1988)
is a foundational text in feminist epistemology + science studies. The
core move: reject "the god trick" — the conceit that objectivity comes
from a "view from nowhere" — and replace it with **embodied
objectivity** that is "view from somewhere," partial, situated, and
accountable to its location.

Haraway's argument is structurally important: she does not argue for
relativism or against objectivity. She argues that *real* objectivity
requires acknowledging the specific embodied position from which a
claim is made, and that knowledge that pretends to be from-nowhere is
worse-objectivity, not better.

The literature on situated knowledges has 35+ years of development in
feminist epistemology, science studies, and adjacent fields. It is
the closest *Western-academic* prior to indigenous substrate-embedding
principles, and it specifically inoculates against the criticism that
substrate-bound knowledge is "merely subjective."

**What this maps to in antigen**: ADR-005 Amendment 3 (audit reports
its own tier honestly) is structurally a Haraway-shaped move.
Pre-amendment, the audit's `is_well_formed()` performed the god trick
— claimed verification at a tier the audit did not actually do the
work to support. Post-amendment, the audit reports the tier of
verification it actually performed, with its actual partiality made
explicit.

This is "view from somewhere" applied at the recognition surface.
The audit's objectivity is *strengthened* by the explicit
acknowledgment of partiality, not weakened. That structural property
is exactly Haraway's argument.

The implication for the foundational paper: *antigen's commitment to
tier-honesty is not an engineering compromise; it is a specific
instance of the situated-knowledges epistemological move that the
science-studies literature has been making since 1988*. The position
is principled, not pragmatic.

**Recognition trigger**: foundational-paper material defending the
tier-honesty discipline can invoke Haraway as the deepest Western-
academic prior. Manuscript framing material for AI-dev-tooling paper
can use the situated-knowledges frame to contrast antigen with
"objective AI assistants from nowhere" — a god-trick framing the AI
industry tends to default into.

### C.5 — Polanyi + Lave & Wenger: the tacit-knowing-as-substrate-bound prior

**Background**: Michael Polanyi (*The Tacit Dimension*, 1966) famously
formulated *we can know more than we can tell*. His central claim:
tacit knowledge is not "implicit knowledge waiting to be made
explicit"; it is knowledge that is *intrinsic to the knower's body,
background, beliefs, and practice* and cannot be separated from those
without degradation. *Indwelling* — Polanyi's term — is the specific
mode of knowing where the knower inhabits the substrate of practice.

Lave & Wenger (*Situated Learning*, 1991) extended Polanyi's
substrate-bound framing to organizational scale: **communities of
practice** are the structural units in which tacit knowledge actually
lives. Knowledge is shared *indirectly through socialization in shared
practice*, not transmitted as propositions.

Recent work (Hadjimichael, Ribeiro, Tsoukas 2024 — Polanyi via
Merleau-Ponty) develops the embodiment-cognition cognate further:
substrate-bound tacit knowledge requires embodied participation, not
just textual access.

**What this maps to in antigen**:

This is the most directly-load-bearing cognate for antigen's
substrate-of-practice principle, because Polanyi + Lave & Wenger
together provide a *Western academic substrate* for the principle that
the indigenous-epistemology framing surfaces from outside the academy.
The two literatures converge on the same structural claim from
different starting points; the convergence itself is recognition
substrate.

Specifically:

- **`#[antigen]` declarations as partially-explicit substrate-bound
  knowledge**: an antigen declaration is not full propositional capture
  of the failure-class; it is a substrate-bound trace that *requires
  the codebase practice to be meaningful*. A stdlib antigen extracted
  into a paper without the codebase context loses meaning the same way
  Polanyi predicts tacit knowledge loses meaning when extracted from
  practice.

- **Antigen-stdlib + per-project antigens as a community of practice**:
  the contributor community + the codebase + the tooling + the shared
  failure-class vocabulary together form a Lave-Wenger-shaped community
  of practice. The substrate is the community's collective indwelling
  in the codebase. Adoption is structurally a process of newcomers
  entering the community of practice, not a process of transmitting
  propositions.

- **The fine-tuning vs. structural-substrate contrast (scope.md AI dev
  tooling implications)** maps cleanly to Polanyi's distinction:
  fine-tuning extracts knowledge into model weights (loses substrate-
  embedding); antigen keeps knowledge in code (preserves substrate-
  embedding). The structural difference is *exactly Polanyi's
  distinction* between extracted-and-degraded vs. substrate-bound-and-
  preserved.

**Recognition trigger**: the AI-dev-tooling paper trajectory (post AI-
industry comparison data accumulates per scope.md) has its deepest
positioning material here. Polanyi + Lave & Wenger + Hadjimichael et al.
2024 together form the "why structural matters more than extracted"
argument the paper needs. The argument has 60 years of Western academic
substrate behind it and converges with indigenous-epistemology
substrate from a different starting point — convergent-evolution-
shaped evidence per §2 of the main map.

### C.6 — The named extraction-degradation modes

The §12 main pass named "extraction-without-context as a degradation
mode" but did not develop the modes. The literatures predict at least
**five distinct extraction-degradation patterns** that antigen-related
content can suffer when extracted from the substrate of codebase
practice. Naming them explicitly is recognition substrate for the
methodology paper + tutorial/blog-post material.

#### Mode 1 — Decoration extraction (using vocabulary without commitment)

The pattern: antigen vocabulary (named failure-classes, fingerprints,
witnesses) is used in writing or talking about *non-antigen-using*
projects, deployed as decoration rather than as commitment to the
underlying discipline.

Indigenous-epistemology cognate: ceremonial language used outside
ceremonial context degrades both. The vocabulary stops carrying weight.

Predicted antigen-context manifestation: blog posts that use "antigen-
shaped" framing to describe Rust patterns without actually adopting the
tooling. This dilutes the vocabulary's structural meaning. The
antibody: stdlib documentation should distinguish *adopting antigen*
(structural commitment) from *using antigen-flavored language*
(decoration); the former is the project's destination, the latter is
not.

#### Mode 2 — Reduction extraction (the core idea without the substrate)

The pattern: an academic paper or summary captures "the core idea" of
antigen — failure-class memory + recognition + inheritance — without
the substrate-bound implementation. The reduction reads as if the idea
could be implemented anywhere; the substrate-bound nature of why the
implementation matters is lost.

Polanyi-cognate: tacit knowledge presented as proposition. Loses
meaning.

Predicted antigen-context manifestation: papers about antigen that
discuss "the architecture" without engaging with the *Rust-specific
implementation choices* (proc-macro substrate, syn AST, cargo
extension surface, witness-tier honesty). The architecture-without-
substrate framing is structurally degraded — it cannot be reproduced
because the substrate-bound implementation choices are *load-bearing*,
not incidental.

The antibody: foundational-paper material should explicitly defend
substrate-bound implementation choices as part of the architecture,
not as engineering details. ADR-001 (structural-not-documentary) does
this in operational form; the foundational paper needs an analogous
section.

#### Mode 3 — Generalization extraction (claiming domain-applicability without domain-specific work)

The pattern: someone reads the cross-domain map V1 (15 windows) and
generalizes "antigen is the universal failure-class memory
architecture for any software" without engaging the specific
adoption-in-Rust substrate work that made antigen viable in Rust.

Two-Eyed Seeing cognate: assimilating one tradition's framework into
another without the integration work. Loses the strengths of both.

Predicted antigen-context manifestation: papers / startups / tooling
that try to instantiate antigen for Python / TypeScript / Go without
doing the substrate-specific work (proc-macro analog, AST analog,
witness-tier analog, IDE-integration analog). The generalization
claim is precisely the failure mode Two-Eyed Seeing inoculates against.

The antibody: scope.md's positioning ("first ergonomically-adoptable
instantiation for Rust") is structurally specific. The foundational
paper should preserve the specificity even as it claims domain-
generality of the architecture; one is not the other.

#### Mode 4 — Authority extraction (canonical framings from non-canonical sources)

The pattern: someone outside the project's substrate-of-practice
publishes "the canonical framing of antigen" without engaging the
ratified ADR substrate, the postures.md catalog, or the cross-
project history. Their framing is then cited downstream as
authoritative.

CARE-cognate: data governance failures where outside parties claim
authority over substrate they don't own.

Predicted antigen-context manifestation: external papers / blog posts
that paraphrase project documents and become the canonical citation
in downstream literature, even when their framing diverges from the
project's substrate. The substrate-grounded framing gets eclipsed by
the easier-to-cite secondary source.

The antibody: explicit project-authored canonical material (the
foundational paper, scope.md, the cross-domain map V1 itself) at
visibility levels that outpace secondary citations. ADR substrate as
the authoritative source. Citation discipline in project-authored
material modeling the relationship between substrate and citation.

#### Mode 5 — Authoritarian extraction (substrate becomes the authority)

The pattern: as antigen-stdlib stabilizes and adoption grows, the
*substrate itself* becomes authoritative in ways that close off
re-grounding. New contributors defer to "what stdlib says" rather
than re-engaging with first principles. The community-of-practice
calcifies into rule-application.

This is Polanyi + Lave & Wenger predicting their own degradation
mode: communities of practice can fossilize into communities of
rule-following, and tacit indwelling is replaced with explicit-but-
ritualized compliance. (See also: variant 8 of the comprehension-
drift family — verification-fatigue drift / normalization of
deviance — at the team-coordination scope.)

Predicted antigen-context manifestation: post-A5+, antigen-stdlib
becomes "what every Rust project should adopt" without engagement
with the underlying recognition discipline. The architecture
calcifies; the substrate-of-practice principle is replaced with
substrate-as-authority.

The antibody: continuous re-grounding discipline at the project
level (the team's substrate-over-memory practice generalized);
explicit reaffirmation of recognition-not-design (posture §2) and
anti-YAGNI (posture §4) as the project scales; the no-fixed-point
operational signature kept active rather than allowed to relax.

### C.7 — The substrate-of-practice principle: synthesis

Across CARE Principles, Two-Eyed Seeing, Haraway, Polanyi, Lave &
Wenger, and indigenous-epistemology traditions, a single structural
principle emerges that the §12 main pass named without developing:

> **The carrier and the carried must live in the same substrate of
> practice. Knowledge that pretends to be substrate-free is degraded
> knowledge.**

This is the deepest non-decorative claim antigen makes. It is also
the claim with the densest cross-tradition substrate behind it —
indigenous traditions plus feminist epistemology plus philosophy of
tacit knowing plus organizational learning theory plus
communities-of-practice all converge on it.

Operational implications for antigen:

- **For the architecture itself**: the substrate-of-practice principle
  predicts that any future feature pulling antigen-substrate *out of*
  the codebase (e.g., a separate registry, a documentation-only
  declaration form, a doc-comment fingerprint) will degrade
  proportionately to the distance from substrate. The recognition
  trigger is when such proposals surface; the framework predicts the
  failure mode.

- **For the team's coordination**: the comprehension-drift family
  literature grounding (this campsite thread) maps the substrate-of-
  practice principle to the team-coordination scope. Substrate-over-
  memory IS the team-coordination instance of the principle. The
  cross-domain map V1 catches the project-architecture instance; this
  appendix catches the substrate-of-practice principle as the deeper
  cross-scope unifier.

- **For manuscript trajectory**: the foundational paper (post-v0.2.0)
  needs the substrate-of-practice principle as one of its load-bearing
  positioning claims — *what makes antigen architecturally novel is
  not that it remembers failure-classes, but that it remembers them
  in the substrate of the practice that produces them*. That framing
  has Polanyi + Haraway + Lave & Wenger + indigenous traditions + Two-
  Eyed Seeing all behind it; it is not a project-specific idiosyncrasy.

- **For ecosystem outreach**: the Two-Eyed Seeing principle inoculates
  against the most-likely misframing — that the biological metaphor is
  decorative or assimilative. The framing is *biology-as-instrument
  alongside other instruments without reduction*, which is what Two-
  Eyed Seeing names.

### C.8 — Recognition triggers for antigen

Per ADR-006, no primitive proposals follow. Specific recognition
triggers from this appendix:

- **Foundational paper drafting** (post-v0.2.0): substrate-of-practice
  principle becomes a core positioning claim with the citation cluster
  named in C.5 + C.7.
- **AI-dev-tooling paper**: Polanyi distinction (extracted vs.
  substrate-bound) becomes the architectural argument against fine-
  tuning-based knowledge embedding.
- **Methodology paper** (post-A2): communities-of-practice (Lave &
  Wenger) becomes the team-coordination framing alongside stigmergy
  (which scientist already has).
- **Adoption pitch v2**: Two-Eyed Seeing as the framing for biology-as-
  instrument; CARE-shaped framing for community authority over
  domain-specific stdlib branches.
- **Stdlib governance** (post-A5+): CARE Principles as recognition
  substrate for community authority structure.
- **External-citation hygiene**: when external papers/blog posts
  surface that paraphrase antigen substrate, the named extraction-
  degradation modes (C.6) help diagnose which mode is operating and
  how to respond.

### C.9 — What this appendix does NOT do

To prevent scope creep:

- Does NOT propose new postures. The substrate-of-practice principle
  is operationally present at four-plus instances in current ratified
  substrate (ADR-001, posture §5, substrate-over-memory, Amendment 3
  tier-honesty); the appendix surfaces the underlying principle but
  does not advance it to formal posture-class until the team chooses
  to. Aristotle's call.

- Does NOT advocate for direct adoption of CARE Principles or
  Two-Eyed Seeing as governance frameworks. They are recognition
  substrate; specific adoption is future ADR territory if at all.

- Does NOT claim antigen "applies" indigenous knowledge frameworks.
  Two-Eyed Seeing explicitly inoculates against extractive use of
  Indigenous frameworks; the appendix follows that discipline by
  treating the indigenous traditions as substrate-of-practice prior
  art that the project respects rather than appropriates. The
  framework is *cited*, not *applied to make antigen indigenous-
  flavored*.

- Does NOT replace §12's main pass. The §12 main pass + this
  appendix together constitute the cognate; the appendix deepens
  rather than restates.

---

*V1 authored 2026-05-08 by academic-researcher during A2 day 2.
Companion to V0 immune-system-primitive-map.md. Maintained as ADR-006
threshold is met by additional fields surfacing in adoption.*

*Appendix A (cybersecurity governance deepening) + Appendix B (manuscript
framing assignments) added 2026-05-08 same authoring session, after the
main V1 substrate landed and idle-as-invitation pulled academic-researcher
toward the most actionable cognate.*

*Appendix C (indigenous-epistemology cognate deepening + substrate-of-
practice principle synthesis + named extraction-degradation modes) added
2026-05-08 same authoring session, after navigator endorsed continued
idle-as-invitation pursuit and academic-researcher followed the next
unmined adjacent vein per JBD discipline.*
