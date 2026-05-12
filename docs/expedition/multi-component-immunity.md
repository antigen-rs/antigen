# Multi-Component Structural Immunity

> **Status**: Deep-dive draft, **V1** (2026-05-11). V0 authored by team-lead
> (Claude Opus 4.7) from the 2026-05-11 conversation substrate. V1 revised
> after team expansion pass: scout (vocabulary-as-protocol + Component 7),
> naturalist (C4 boundary-silence finding + tier-structure), adversarial
> (threat-model + amendment candidates + honest-boundary substrate).
> Tekgy ratified revising cleanly before aristotle's Phase 1-8.
>
> **Where this lives in the lifecycle**: expedition substrate. Not project-
> tier yet. Promoted to `docs/multi-component-immunity.md` after team Phase
> 1-8 + ratification per `process.md`.
>
> **Status of enumeration**: PROVISIONAL and OPEN at multiple axes. Seven
> components currently named; enumeration may extend along multiple axes;
> tier structure surfaced as one axis (biology-tier / engineered-boundary
> tier); other axes may surface as we recurse. The structure of the
> enumeration itself has structure.
>
> **Companion substrate**:
> - `multi-component-immunity-conversation.md` — raw conversation that
>   produced V0 framing.
> - Scout finding: `campsites/antigen-A3/.../scout/20260510-adr-017-018-empirical-verification-and-component-candidates.md`
> - Naturalist C4 finding: `campsites/antigen-A3/.../naturalist/20260510215459-c4-boundary-silence-finding.md`
> - Adversarial threat model: `campsites/antigen-A3/.../adversarial/20260510-multi-component-threat-model.md`

---

## Part I: The vocabulary as emergent practice

Antigen is centrally a **vocabulary**, and that vocabulary functions as an
**emergent practice** — the shared coordination layer of a community-of-
practice. Heterogeneous components (dev-judgment, passive scan/tools, test
integration, knowledge bridging, lineage tracking, cross-crate scanning,
real-time feedback) participate in the same emergent practice even though
they do wildly different things.

This is sharper than both "vocabulary as spine" and "vocabulary as
protocol" (the V0 and earlier-V1 framings). A spine supports; a protocol
*specifies*; an emergent practice *coordinates and grows*. Components don't
just attach to the vocabulary; they actively *use* it — and the vocabulary's
shape itself co-evolves with the practice as participants recognize new
failure-classes, new witness shapes, new tier distinctions. (Scout's
sharpening of the function 2026-05-11; naturalist's refinement of the
genesis 2026-05-11 — see naturalist campsite
`20260511120843-vocabulary-as-protocol-cognate-refinement.md` for the full
substrate-grep and layer-distinction reasoning.)

**Why this matters architecturally:**

A team can adopt the vocabulary without adopting any specific tooling. A
single declaration of `#[antigen(name = "X", summary = "...")]` in a Rust
file is antigen-the-vocabulary in operation — even with `cargo antigen scan`
never run. The team has *named a failure class*; they have *given it
structural memory*; they have made it *legible to future-readers including
LLM collaborators*. The tool would extend this; the discipline would extend
this; the ecosystem would extend this. But the emergent practice is the
floor.

The emergent practice carries:
- *What* is being recognized (antigen declarations)
- *Where* it's present (presentation marks)
- *How* it's protected (immunity claims with witnesses)
- *How* it inherits (descended_from edges)
- *How* it's tolerated (rationale-required tolerance marks)
- *Why* it matters (references + rationale fields)
- *When* it was attested (verified_at temporal field)
- *Across-version identity* (canonical_path at name@version)

Different components consume different parts of the practice. Components
don't need to know about each other; they need to know about the
vocabulary.

**The two layers** (naturalist's V1 layer-distinction):

There's an important asymmetry worth naming explicitly:

- **The vocabulary itself** is *emergent practice* — co-evolved through
  ADR-006's recognition discipline. New antigens enter through observed
  instances clearing the three-independent-instances threshold, not through
  specification. The vocabulary's growth is recognition-driven; new
  primitives recognize structure already in the substrate.

- **The vocabulary's governance** is *engineered process* — `process.md`,
  ADR lifecycle, amendment cycles, ratification commits. These are
  explicitly specified, versioned through commit history, have conformance
  criteria, and have a designated authority (the ratification process).
  Engineered substrate.

Both layers are real and serve different purposes. Recognition-discipline
at the vocabulary layer; specified process at the governance layer.
Conflating them — importing engineered-spec baggage to the vocabulary
itself, or claiming recognition-discipline at the governance layer where
explicit process is load-bearing — drifts the project's posture in
unhelpful directions.

**The biology cognate:**

The body's immune system shares MHC molecules + cytokine signaling as the
*co-evolved coordination substrate* that wildly different cell types use
to communicate. B cells, T cells, NK cells, dendritic cells, macrophages
don't all do the same thing — but they all *speak* the shared substrate.
The substrate is what makes them an immune *system* rather than
disconnected mechanisms.

Antigen-the-vocabulary is structurally analogous: the emergent practice
layer that components use to coordinate. **Different genesis, same
compositional function**: biology's substrate emerged through evolutionary
pressure across millions of years; antigen's emerges through community
recognition under ADR-006 discipline. Both are co-evolved-not-engineered at
the vocabulary layer. Heterogeneous components coordinate around shared
substrate without central authority specifying the coordination layer.

(The "protocol" framing — which earlier V1 used briefly and which scout
sharpened from V0's "spine" — was structurally accurate at the *function*
level but imported engineered-spec baggage at the *mechanism-framing*
level. Naturalist's substrate-grep confirmed antigen substrate doesn't use
"protocol" for this purpose elsewhere; the refinement to "emergent
practice" honors ADR-006's recognition character and Wenger's
community-of-practice framing without inheriting RFC-style spec-baggage.
The refinement is in V1 deep-dive; future references to the layer should
use "emergent practice" or "co-evolved coordination substrate" rather than
"protocol".)

---

## Part II: Tier structure of the enumeration

The enumeration is not flat. Naturalist's C4 finding surfaced this as a
substrate-grounded answer to V0's Q1.

**Two tiers observed (one axis among potentially many):**

**Biology-tier** (substrate where biology metaphor is operationally
load-bearing):
- C1: Dev-in-the-loop immunity
- C2: Passive scan/lint/tool immunity
- C3: Test-integration immunity
- C5: Cross-version / lineage immunity
- C6: Cross-crate / ecosystem immunity
- C7: Real-time / CI feedback immunity (promoted from candidate; scout)

**Engineered-boundary tier** (substrate where engineering extends beyond
biology's domain at honest boundaries):
- C4: Knowledge-ecosystem immunity (references to PRs, ADRs, CVEs, papers)

Why C4 lives in a different tier:

References (PR threads, ADR files, CVE entries, papers) are artifacts of
*human knowledge ecosystems*. In biology, organisms don't read their own
scientific literature. Knowledge-about-the-organism lives in
epidemiologists / public-health-institutions *outside* the organism. C4's
failure modes are categorically different from C1-2-3-5-6-7 — they inherit
from library science (link rot), academic integrity (citation
fabrication), and info-warfare (source contamination). Different taxonomy
entirely.

This is metaphor-as-instrument operating at its sharpest. Biology going
silent at C4 is *evidence*, not absence. The silence shows where
engineering has built capabilities biology hasn't evolved — and tells us
the relevant cognate-family lives elsewhere.

### The "engineered-substrate-exceeds-biology" family

Naturalist surfaced that C4 is the third member of a family with shared
shape: **engineered substrate gains capabilities biology hasn't evolved,
honestly bounded.**

- **W7 FormalProof tier** — machine-verified proof; biology has no
  formal-proof verification mechanism for immune-tier claims.
- **ADR-017 trust-delegation** — cargo's checksum verification chain;
  immunology has no analog at the dependency boundary.
- **C4 knowledge-ecosystem** — references to lived-context artifacts;
  organisms don't have knowledge-about-themselves outside themselves.

Three instances of the same architectural shape. By ADR-006, threshold
met for the pattern itself — though we hold ratification pending the
encounters discipline shape.

This family has a meta-discipline implication (see Part V).

### The manifold framing — enumeration may have multiple axes

Tier-structure (biology / engineered-boundary) is one axis. Other axes
may exist:

- *Production-vs-consumption* axis: which components produce vocabulary
  artifacts (C1) vs consume them (C2, C3)?
- *Static-vs-dynamic* axis: build-time vs runtime vs interactive?
- *Individual-vs-population* axis: per-codebase vs cross-codebase?
- *Implicit-vs-explicit* axis: what components encode implicit assumptions
  vs make them structural?

The enumeration's structure has structure. The structure of the
structure has structure. *Recognition-not-design at the meta-meta level:
hold the enumeration open in shape, not just open in count.*

This is consistent with the contact-graph framework already in antigen
substrate (`docs/contact-graph-and-recognition-tiers.md`) — a 3-tier ×
7-mode matrix that is itself a manifold of recognition relationships.
Multi-component immunity may also be a manifold; the tier-structure
finding is one slice.

Future-instances of the team finding new axes is expected, not a deviation.

---

## Part III: The seven components

### Component 1: Dev-in-the-loop immunity (biology-tier)

**What it does**

The developer writes antigen declarations into their Rust code based on
their judgment of what failure classes exist, where vulnerabilities sit,
what's protected and how, and how the failure classes inherit. *Production
of immunity through human cognition* — the team knows something is
dangerous, names it, and makes it structural.

**The five vocabulary primitives:**

- `#[antigen(name = "...", summary = "...", references = [...])]`
- `#[presents(antigen_name)]`
- `#[immune(antigen_name, witness = ...)]`
- `#[descended_from(parent)]`
- `#[antigen_tolerance(antigen_name, rationale = "...", until = ...)]`

**Discipline + tooling sides**

Primarily discipline. Macros parse, validate, make declarations
structurally present to other components. Tooling in service of the
discipline.

**Floor / ceiling**

- Floor: a single `#[antigen(...)]` declaration. Structural memory exists.
- Ceiling: comprehensive antigen taxonomy with rich rationale, full
  lineages, every site marked, every accepted-known-risk tolerance-marked.

**Biology cognate**

Closest to **deliberate vaccination + informed prior exposure**.

[NATURALIST: V0 asked between vaccination / memory-B-cell / T-helper
coordination. Open for refinement.]

**Failure modes / attack surface**

- Mis-named antigen (declaration shape doesn't match real failure-class)
- Speculative antigen (declared without instance grounding; ADR-006
  violation)
- Stale rationale on tolerance
- Lineage drift across descended_from chains
- Malicious antigen-declaration injection (proc-macro generation
  territory, ADR-014)

**Connection to other components**

Component 1 is the *production source* — most other components consume
what it produces. Floor-mode antigen is essentially C1 alone, with the
other components mostly inactive.

**Substrate locations**

`antigen-macros/`, `antigen-macros/src/parse.rs`, `docs/glossary.md`,
ADR-001, ADR-004, ADR-005 Amendment 2, ADR-011, ADR-014.

---

### Component 2: Passive scan/lint/tool immunity (biology-tier)

**What it does**

Automated walks find antigens, presentations, immunities, tolerances,
lineage edges. Audit verifies witness validity at each immunity site.
Fingerprint engine matches structural patterns against unmarked code.
Cycle detection guards lineage. *Recognition through structural analysis.*

**Concrete operations**: `cargo antigen scan`, `cargo antigen audit`,
fingerprint engine, cycle detection (ATK-A3-002), diamond inheritance
dedup (ADR-018).

**Discipline + tooling sides**

Primarily tooling. Discipline lives around the tool (read the audit
report; address unaddressed presentations; respond to fingerprint
matches).

**Floor / ceiling**

- Floor: `cargo antigen scan` once. Structural memory laid bare.
- Ceiling: scan in CI, audit-gated PRs, fingerprint matches inline,
  scan-failure rejects on commit.

**Biology cognate**

Closest to **PRR (pattern-recognition receptor) function + dendritic-cell
processing within innate immunity** (per naturalist Q-C2 verification,
2026-05-12 — Resolution B: surfaces both biology cognates already bound
in `immune-system-primitive-map.md` lines 64 + 110, rather than collapsing
to a single "innate immunity" framing that collides with clippy at the
system-level posture).

Two abstraction levels operate cleanly:
- **System level**: clippy occupies "innate-immunity-system" posture
- **Mechanism level (C2)**: PRR-pattern-recognition (fingerprint engine)
  + dendritic-cell processing (audit pipeline) within the system

**Boundary-silence note**: the PRR/dendritic-cell cognates go silent at
fingerprint *authoring* — writing new fingerprints is closer to
genome-editing-equivalent than innate-immunity-equivalent. Authoring
lives in C1 territory (where developmental/vaccination/surveillance
cognates apply); C1 and C2 are complementary, splitting authoring vs
runtime recognition.

**Failure modes / attack surface**

- Tool says clean when substrate is broken (sub-clause F violation)
- Fingerprint false-positive autoimmunity
- Audit tier over-claiming (ADR-005 Amendment 3 discipline)
- Cycle detection false-pass (ATK-A3-002 contract)
- Crash-resistance violation
- Trust-boundary bypass via alternative path-discovery (ATK-A3-007)

**Substrate locations**

`antigen/src/scan.rs`, `antigen/src/audit.rs`, `cargo-antigen/`, ADR-001,
ADR-002, ADR-005, ADR-010, ADR-015, ADR-017, ADR-018.

---

### Component 3: Test-integration immunity (biology-tier)

**What it does**

Witnesses link to actual tests. Test history becomes immune history. The
audit's verification grounds in real behavioral confirmation.
*Verification through behavioral observation.*

**Witness vocabulary**: test, proptest, clippy lint, kani / prusti / verus
/ creusot proof, phantom-type (ADR-013).

**WitnessTier gradient** (ADR-005 Amendment 3): FormalProof /
ExecutionVerified / Reachability / ExternalUnvalidated / Missing.

**Discipline + tooling sides**

Mixed. Team writes the tests / proofs; tool resolves witnesses, validates,
reports tier honestly.

**Floor / ceiling**

- Floor: `#[immune(X, witness = test::tx_test)]`; verification at
  ExecutionVerified tier.
- Ceiling: tier-appropriate witness for every immunity; FormalProof where
  possible; cross-version witness validity tracked through descended_from
  (C5 territory); witness identity presentation-keyed (ADR-018).

**Biology cognate**

Closest to **memory B-cell binding confirmation**.

[NATURALIST: V0 asked between memory-B-cell / affinity-maturation /
T-cell-receptor. Open for refinement.]

**Failure modes / attack surface**

- Witness drift (valid when written, no longer applies)
- Tier over-claiming
- Witness-as-placeholder (always-passing test)
- External-unvalidated proliferation
- Test pass-but-not-meaningful (ATK-A2-003/004/005/011/012)

**Substrate locations**

`antigen/src/audit.rs`, `audit.rs::detect_external_tool`, ADR-002, ADR-005,
ADR-007, ADR-013, ADR-018.

---

### Component 4: Knowledge-ecosystem immunity (engineered-boundary tier)

**What it does**

References attached to antigen declarations point to lived context: PR
threads, post-mortem blog posts, git issues, manual pages, ADR/DEC files,
internal tutorials, CVEs, RFCs, papers. *Contextual memory linking lived
history across heterogeneous knowledge substrates.*

**Mechanism**: `#[antigen(..., references = [...])]` per ADR-009 Layer 2;
rationale fields on tolerance/immunity; future bidirectional links.

**Discipline + tooling sides**

Currently primarily discipline. Vocabulary exists in v0.1; scan collects
references; audit doesn't yet validate they resolve. Future tooling
extensions (link-rot detection, bidirectional indexing, cluster-detection)
are encounter-tier substrate.

**Floor / ceiling**

- Floor: one antigen with one reference. Connection-point to lived context.
- Ceiling: comprehensive bidirectional references; antigens are knowledge-
  graph nodes; cross-team / cross-organization shared references.

**Why this is engineered-boundary tier**

References live in human knowledge ecosystems (CVE databases, RFC
processes, blog posts, ticket systems, ADR conventions). Biology doesn't
have this — organisms don't read their own scientific literature.
Knowledge-about-the-organism lives in epidemiology / public-health
institutions *outside* the organism. C4's failure-mode taxonomy comes from
library science, academic integrity, and info-warfare disciplines — not
from immunology.

**The boundary is honest, not a metaphor failure.** Biology going silent
here is *evidence* that engineering has extended into a domain biology
hasn't reached. C4 joins W7 (FormalProof tier) and ADR-017 (trust-
delegation) as third member of the "engineered-substrate-exceeds-biology
at honest boundaries" family.

[NATURALIST: V1 ratification of your C4 finding. Open: are there
within-engineered-boundary cognates we should use instead — library
science, citation networks, knowledge graphs? Is there an explicit
analog from another domain (academic publishing? legal precedent
citation?) that sharpens?]

**Failure modes / attack surface**

- Stale references (link rot)
- Fake references (plausibly-named but fabricated)
- Poisoned external references (CVE redacted/superseded)
- Reference noise (signal drowning)
- Cross-reference loops
- **LLM-hallucinated references** (co-native attack surface: LLMs both
  generate and consume references; hallucinated URLs look
  calibrated-to-plausible — adversarial finding, A5 governance)

**Connection to other components**

- Consumes C1's references field.
- C2 collects but doesn't yet validate; future tooling extension.
- C5 lineage may be version-specific.
- C6 may carry references across crate boundaries.

**Substrate locations**

`#[antigen(..., references = [...])]` per ADR-009 Layer 2; ADR-001,
ADR-004, ADR-005 Amendment 2, ADR-009.

---

### Component 5: Cross-version / lineage immunity (biology-tier)

**What it does**

`#[descended_from]` chains track inheritance, evolution, specialization
across antigens. Temporal recognition surface (ADR-016) tracks *when*
immunity was established, *what version* was verified, *how* immunity has
evolved. Version-boundary-as-feature (ADR-017) treats version transitions
as recognition opportunities. *Evolutionary memory across change.*

**Mechanism**: `#[descended_from(parent)]`, diamond inheritance dedup
(ADR-018), `verified_at` (ADR-016), `canonical_path` in `name@version`
form (ADR-017), version-boundary-orphans-as-feature, ProvenanceEntry
(ADR-018) on `inherited_from`.

**Discipline + tooling sides**

Mixed. Team maintains descended_from chains, re-validates inherited
witnesses, treats version transitions as recognition opportunities. Tool
collects, propagates, dedups, surfaces orphans.

**Floor / ceiling**

- Floor: a single `#[descended_from(Parent)]`. Taxonomy starts.
- Ceiling: rich inheritance trees, version-aware identity, temporal
  recognition surface, cross-version re-validation, isotype-switching
  equivalent at witness-evolution level.

**Biology cognate**

Multiple cognates converge:
- **Antibody class-switching** (isotype switching) — same lineage, different
  specialized roles
- **B-cell hypermutation** — small variations producing competing
  specificities
- **Memory vs plasma B-cell differentiation** — same lineage, different
  active roles

For version-boundary-as-feature specifically: **antigenic drift / shift**
(naturalist's prior correction). Antigenic drift = small version updates;
antigenic shift = major version boundary with genome reassortment producing
categorically new recognition surface. Orphan-lineage-edge IS body
recognizing previously-known antigen as no-longer-matching-memory because
drift has been large enough.

[NATURALIST: depth-check on drift/shift cognate — how deep does it go?
Are there other version-boundary phenomena beyond drift/shift?]

**Failure modes / attack surface**

- Lineage cycle (ATK-A3-002)
- Stale lineage
- Witness staleness across descent (ADR-005 Decision item 2)
- Diamond inheritance unintended-dedup (ADR-018 ProvenanceEntry)
- Version-boundary autoimmunity
- Orphan-on-version-change category error (ATK-A3-010 — drift, not waning)

**Substrate locations**

`antigen/src/scan.rs::LineageEdge`, synthesis pass, ProvenanceEntry per
ADR-018, canonical_path per ADR-017; ADR-001, ADR-005, ADR-007, ADR-008,
ADR-016, ADR-017, ADR-018.

---

### Component 6: Cross-crate / ecosystem immunity (biology-tier)

**What it does**

Antigen declarations propagate across crate boundaries. Cross-crate scan
via `.cargo/registry` source-walking reads antigens from dependencies.
`antigen-stdlib` (post-A5) provides shared failure-class memory.
Canonical-path identity (`name@version`) distinguishes same-named antigens.
Trust delegation to cargo's checksum chain. *Population-level immunity.*

**Mechanism**: `cargo antigen scan` walks workspace + deps,
`enumerate_dep_crate_roots()` is the ONLY trust-delegated path-discovery
mechanism (ADR-017), canonical_path on declarations, ProvenanceEntry
preserves cross-crate provenance, future antigen-stdlib.

**Discipline + tooling sides**

Mixed. Discipline of using deps' antigens vs re-declaring locally; of
contributing to antigen-stdlib; of treating cross-crate trust boundaries
properly. Tool walks, resolves, stamps canonical_path, detects cycles +
diamonds across crates.

**Floor / ceiling**

- Floor: `cargo antigen scan --include-deps`.
- Ceiling: antigen-stdlib widely adopted; per-organization registries;
  antigen declarations in CVE databases; ecosystem-wide failure-class
  memory.

**Biology cognate**: **epidemiological surveillance infrastructure**
(per naturalist Q-C6 verification, 2026-05-12). The WHO flu surveillance
network is the substrate-grounded cognate — five-axis structural match
confirmed; three independent bindings already present in
`contact-graph-and-recognition-tiers.md` (lines 16, 246, 317-318).

Why surveillance-infrastructure rather than the V0 candidates:

- **Herd immunity** — rejected. Predicts that population-level protection
  emerges from many individuals being independently immune; antigen's
  cross-crate propagation is *coordinated infrastructure*, not emergent
  from individual adoption.
- **MHC polymorphism** — rejected. Predicts the *opposite* architectural
  value (diversity-for-resilience); ADR-017's canonical convergence
  goes the other direction. Surfaces a real convergence-vs-diversity
  tension worth naming in V2 (the team's multi-discipline composition
  IS the project's MHC-polymorphism analog at the development-tier).
- **Microbiome / commensal organisms** — rejected. Predicts tolerance
  of beneficial organisms; antigen-stdlib is about shared *failure-class
  memory*, not tolerated beneficial patterns.

What surveillance-infrastructure predicts correctly:

- Coordinated centralized identification of failure-class patterns
- Cross-population sharing of recognized patterns (analogous to flu
  strain announcements driving global vaccine production)
- Recognition delays at boundaries (new strain → recognition → response
  cascade; analogous to novel-failure-class → registration → stdlib
  propagation)
- Governance structure tied to recognition authority (WHO's role in
  flu surveillance maps onto antigen-stdlib's contribution model;
  recognition-grounded per A5 governance encounter)

**Ecosystem-level exclusion cognate** (V0 placeholder question):
**negative thymic selection** — the body's mechanism for refusing
self-attacking T-cell clones before they enter circulation. Maps onto
`cargo antigen forbid <pattern>` — declaration-formation-time exclusion,
distinct from `#[antigen_tolerance]` (peripheral tolerance, applied
case-by-case after declaration). Encounter-registration candidate;
post-A5 territory (see deferred-substrate.md V54).

**Failure modes / attack surface**

- Trust-boundary bypass via alternative path-discovery (ATK-A3-007)
- Cross-crate name collision (ATK-A3-005, solved by canonical_path)
- Re-export false NotFound (ATK-A3-001, A4+ territory)
- Registry tampering (cargo verifies fetch, not post-fetch manipulation)
- Supply-chain antigen poisoning (malicious crate ships antigens that
  defend against wrong things)
- **Immunity laundering via newtype** (adversarial finding): wrapper crate
  declares `#[immune(X)]` on a newtype wrapping a foreign type, with
  theatrical witness; downstream consumers inherit ExecutionVerified
  without independent verification. Structurally valid under current
  trust model. Behavioral witness tier (A4-A5) is the right fix; possibly
  also a "WrappedTypeImmunity" tier or different trust-boundary handling.

**Substrate locations**

`antigen/src/scan.rs::enumerate_dep_crate_roots`, canonical_path types,
antigen-stdlib (post-A5); ADR-001 Amendment 1 (C7), ADR-002, ADR-005,
ADR-017, ADR-018.

---

### Component 7: Real-time / CI feedback immunity (biology-tier)

**Promoted from candidate to first-class component per scout's empirical
finding (V1 substrate).**

**What it does**

Surfacing antigen-surface diffs per PR; flagging when PR touches a
vulnerable site marked `#[presents]`; surfacing fingerprint matches
against unmarked changes; sub-acute feedback during code review rather
than build-time recognition. *Recognition at the moment of change, not
the moment of compilation.*

**Why this is structurally distinct from C2:**

- *Distinct audience*: PR author / reviewer vs workspace maintainer
- *Distinct integration surface*: PR comment / status check / inline
  review annotation vs `cargo antigen audit` in CI
- *Distinct scope*: diff-scope (what this PR changed) vs workspace-scope
  (everything currently visible)
- *Structural dependency on C2*: consumes ScanReport as baseline; doesn't
  re-scan
- *Distinct latency requirement*: sub-second / per-commit-push vs
  build-time

C7 is downstream of C2's substrate but operates as its own immune-system
mechanism with different protective role.

**Discipline + tooling sides**

Primarily tooling. Discipline: read the inline annotations; respond to
diff-scope fingerprint matches; integrate with existing PR review
practice.

**Floor / ceiling**

- Floor: simple bot that posts a comment when a PR touches `#[presents]`
  sites.
- Ceiling: rich integration with GitHub / GitLab / Bitbucket; per-file
  inline annotations; diff-scope fingerprint engine; PR-blocking when
  immunity would regress.

**Biology cognate**

Closest to **neutrophil response** — the body's rapid, sub-acute response
to acute injury or pattern detection. Different from innate immunity's
constitutive surveillance (C2 cognate); neutrophils are *recruited* in
response to acute events.

[NATURALIST: refine. Neutrophil cognate vs other rapid-response cells?
Eosinophil? Mast cell? What's the right cognate for "fast diff-scope
response triggered by acute change"?]

**Failure modes / attack surface**

- False-negative on subtle changes (fingerprint engine doesn't match
  but failure-class is real)
- False-positive autoimmunity at diff-scope (flagging legitimate changes
  as if they regressed immunity)
- Latency exceeds developer attention span (recognition arrives after
  PR is merged)
- Reviewer overload / alert fatigue
- Integration-surface breakage (GitHub API changes; webhook drops)

[ADVERSARIAL: deeper analysis wanted. C7 is the youngest component in the
enumeration; the attack surface is least mapped. What about adversarial
PR shaping to evade fingerprint detection? Time-of-check vs time-of-use?]

**Connection to other components**

- Consumes C2's ScanReport (baseline structural state).
- May surface C4 references inline (showing reviewer the lived context).
- May invoke C5 lineage propagation (showing reviewer what would
  propagate from the change).
- Cross-crate (C6) extension is open question.

**Substrate locations**

Not yet implemented. Substrate-seed: scout's empirical finding,
20260510 campsite entry. Future sweep territory.

---

## Part IV: Composition patterns

### Cross-component flow

Most teams will deploy *some* components and not others. Composition is
*plural at its core* — not a single "antigen deployment" but a fabric of
immune-system components the team selects from.

**Likely composition patterns:**

- **Linter-mode** (C2 alone): structural memory of declared antigens; zero
  buy-in.
- **Pragmatic dev** (C1 + C2): team-written antigens + scan; most common
  adoption shape.
- **Pragmatic dev + tested immunity** (C1 + C2 + C3): witnesses linked to
  tests; audit-tier-honesty.
- **Bridged-knowledge** (C1 + C2 + C3 + C4): references attached;
  knowledge-graph emerging.
- **Lineage-aware** (C1 + C2 + C3 + C4 + C5): failure-class taxonomy
  managed; version-boundary handling.
- **Ecosystem participant** (C1-6): cross-crate; antigen-stdlib;
  population-level coordination.
- **Reviewer-integrated** (any of above + C7): real-time PR-scope
  feedback; recognition at moment of change.

**Floor for each component is independent of all other components.** A
team can deploy C5 without C4; C4 without C3; C7 without C5; etc.
Composition is genuinely orthogonal in most cases.

### Cross-component dependencies (real but minimal)

- C5 + C6 tightly coupled through canonical_path identity. Cross-crate
  lineage edges require both.
- C3's witness re-validation across descent uses C5's lineage.
- C2's audit-tier-honesty depends on C3's witness declarations.
- C7 structurally depends on C2's ScanReport as baseline.

These dependencies are real but small. Architecture is primarily
compositional, with cases where one component's full value lands only
with another active.

### Extend-not-replace at the component level

Each component *extends* a baseline practice without *replacing* it:

- C1 extends developer judgment.
- C2 extends manual code review.
- C3 extends testing.
- C4 extends knowledge management.
- C5 extends version management.
- C6 extends ecosystem coordination.
- C7 extends PR review.

The compositional property *is* the extension at each level. Antigen
doesn't compete with any practice; it extends each practice with a
structural-memory layer specific to that practice's failure modes.

---

## Part V: Architectural properties

### Heterogeneous recursion

The compositional property recurses through all scales. The mechanisms at
each scale do not.

Helper T cell ≠ macrophage ≠ NK cell ≠ B cell ≠ complement system ≠ MHC
class I ≠ MHC class II. Each is wildly different in biology. Yet all
participate in one architectural class. The property
*structural-memory-without-going-stale* recurses through all of them. The
mechanisms don't.

Antigen-the-project mirrors this: heterogeneous components (C1-C7)
cooperating under the shared emergent practice (the vocabulary), unified
architectural class (structural failure-class memory).

### Structural-tier vs. maintenance-tier

Tests, documentation, ADRs, sprint planning, knowledge wikis, Slack — all
**maintenance-tier** practices. Their currency depends on ongoing team
effort. Stale as soon as code evolves.

Antigen operates at **structural-tier**. Currency enforced by the same
machinery that enforces type-checking. When fingerprints fail to match,
the antigen surface notices — not because someone updated a doc, but
because structural memory and structural reality diverged and the
compiler/scanner sees it.

The components vary in how much they extend this property:
- C1, C2, C5, C6, C7 are most structural-tier.
- C4 has maintenance-tier elements (references can go stale; tooling
  doesn't yet validate them); but the antigen they're attached to stays
  structurally current. Honest-boundary at the references-side.
- C3 inherits test-suite's maintenance-tier discipline but enforces
  audit-tier-honesty structurally.

### Co-native with human and LLM collaborators

The vocabulary is readable by both kinds of collaborators without
translation. Biology metaphor is universal lived experience for humans;
unambiguous semantic cognate for LLMs. Macro syntax follows existing Rust
attribute conventions. Audit report is structured data; humans read
human-readable version; LLMs consume JSON.

The co-native property is what makes encounters-the-discipline work in
mixed-collaboration teams. Future versions of any team-member (human or
LLM) inherit failure-class memory by reading what's already in the code.

**Co-native creates its own attack surface (C4 finding)**: LLMs both
generate and consume references; hallucinated URLs look calibrated-to-
plausible to other LLMs. This is registered as encounter-tier substrate
(adversarial A5 governance finding).

### Honest-boundary as encounter-registration (discipline)

Every "X is out of scope" statement is a first-encounter registration,
not a terminal declaration.

When we declare honest boundaries (cargo-level attacks out of scope per
ADR-017 trust-scope amendment; cross-crate witness execution gap per
ADR-005 Amendment 3 amendment; LLM-hallucinated-reference attack surface;
immunity-laundering-via-newtype; antigen-stdlib trust-hierarchy
single-point-of-failure), we are *noticing* something current antigen
can't handle. That noticing IS the first encounter.

The discipline:
1. Make the boundary statement explicit (ADR amendment or equivalent).
2. *Also* register the bounded-thing as an encounter.
3. Periodically revisit: is there a structural-memory or immune-system-
   component answer we haven't seen yet?
4. If yes, the encounter promotes to V0+1 candidate or to a sweep-scope
   work item.
5. If no, the encounter stays in the catalog as a registered known-unknown.

Today's substrate produced five such encounters:
- Cargo-level attacks (CARGO_HOME override, Cargo.lock manipulation,
  registry cache tampering)
- Cross-crate witness execution gap
- LLM-hallucinated references in co-native design
- Immunity laundering via newtype (downstream-trust-without-independent-
  verification)
- antigen-stdlib trust hierarchy single-point-of-failure

The pattern of "honest-boundary as encounter-registration" is itself a
candidate posture eventually (three or more recurrences of the pattern
across vocabulary). For now: registered as discipline, watched for shape
stability.

### The manifold property (V1 substrate)

The enumeration's structure has structure. Tier-structure (biology-tier
vs engineered-boundary tier) is one axis. Other axes may exist
(production-vs-consumption, static-vs-dynamic, individual-vs-population,
implicit-vs-explicit). Future-instances of the team finding new axes is
expected.

Recognition-not-design at the meta-meta level: hold the enumeration open
in *shape*, not just in *count*. The structure of the structure has
structure. Don't lock prematurely.

This is consistent with the contact-graph framework in antigen substrate
(3-tier × 7-mode matrix as manifold of recognition relationships).
Multi-component immunity may also be a manifold; tier-structure is one
slice.

---

## Part VI: Open enumeration — what we haven't named

Scout's rulings on V0 candidates:

- **Real-time / CI feedback** → CONFIRMED as Component 7 (load-bearing-
  not-decorative test passed).
- **Cross-team / organizational tier** → Component 6 at different
  registry backend; feeds ADR-017 OQ1; not a new structural component.
- **Adversarial discipline** → meta-component feeding C3; no direct
  vocabulary primitives; not peer.
- **Educational / onboarding** → vocabulary property (co-native
  readability), not component.
- **Decay / sunset** → vocabulary gap (no "retired antigen" primitive),
  not new component. Encounter-tier or future ADR territory.

Other candidates still worth watching (open per recognition-not-design):

- **Cross-language tier**: antigen-vocabulary extending beyond Rust.
  Engineered-boundary tier (other languages have different proc-macro /
  attribute mechanisms).
- **Cross-organism tier**: extending beyond software (hardware, control
  systems, financial systems).
- **Population-level governance**: distributed stdlib trust, multi-
  maintainer attestation, threshold signatures (A5 governance encounter).
- **Reference-validation tier**: separate component for validating
  references resolve / detecting fabrication / detecting LLM
  hallucination (C4 attack-surface encounter).

Decay/sunset as vocabulary gap — could be addressed by either:
- A new vocabulary primitive (`#[antigen_retired(rationale = ...)]`)
- An extension of `#[antigen_tolerance]` semantics
- A separate ADR

To be ratified per encounters discipline once instances accumulate.

---

## Part VII: What this changes

### Adoption framing

Not "how engaged is this team with antigen?" but "which immune-system
components has this team composed?" Marketing / vision-pitch:
- Lead with vocabulary (the emergent practice layer).
- Show floor concept first.
- Reveal components progressively.
- Don't insist on full-fabric adoption.

### Manuscript framing

Three coexisting framings at three abstraction levels (Tekgy's
"both-can-work"):
- **"Antigen catches failure-class memory"** — floor concept; v0.1.0
- **"Antigen composes multiple kinds of structural immunity"** — V0.2.0+
- **"Antigen is a vocabulary-as-emergent-practice with a fabric of immune-system
  components"** — paradigm-shift framing; post-A6

Manuscript trajectory is layered, not sequenced.

### scope.md / vision-pitch.md updates

Both should be extended (not replaced) with the multi-component framing
*after* this deep-dive ratifies. Suggested:
- scope.md vision: add components-fabric subsection after four-window
  convergence.
- scope.md adoption: reframe adoption-ergonomics in component-selection
  terms.
- vision-pitch.md: keep failure-class-memory framing first paragraph;
  add multi-component paragraph; add "where you start, where you can
  grow" paragraph.

### Project trajectory implications

If multi-component framing holds through Phase 1-8:
- `glossary.md`: add "component" + each component as vocabulary terms;
  add "tier-structure" + "engineered-boundary" terms.
- `README.md`: extend project description with components-fabric framing.
- Future sweep planning around component-tier capabilities (A5 = C6
  ecosystem-tier; future sweep for C7 real-time/CI; future sweep for
  cross-language tier).
- Encounters-tier substrate: each component is an encounter site;
  honest-boundary statements become encounter-registrations.

### Aristotle's pending amendment queue (post-ratification of THIS doc)

Five amendment items consolidated for aristotle's next pass:
1. ADR-018 prose: diamond dedup mechanism (pathmaker)
2. ADR-017 prose: workspace-internal exclusion in §Mechanics (scout)
3. ADR-017 prose: diamond dedup same-version case (scout)
4. ADR-017 NEW: trust-scope statement (cargo-level attacks out of
   scope) (adversarial)
5. ADR-018 / ADR-005 Amendment 3 NEW: cross-crate witness tier defaults
   to ExternalUnvalidated unless consuming workspace can execute
   (adversarial)

Items 4-5 are architectural amendments (not just prose). Both are
honest-boundary moves — making explicit what the project does NOT claim
to defend against. Should be paired with encounter-registration per
discipline above.

---

## Open questions for aristotle's Phase 1-8

(Updated from V0; Q1 has substrate-grounded answer.)

Q1. *RESOLVED (provisional)*: enumeration is layered (one axis among
potentially many). Biology-tier: C1, C2, C3, C5, C6, C7. Engineered-
boundary tier: C4. Manifold-framing: other axes may exist.

Q2. **Biology-cognate sharpness across components**: V0 listed loose
cognates for C1-3 and C6; naturalist's expansion pass is in flight at
idle cadence. What's the Phase 1-8 verdict on cognate-tightness across
biology-tier components? Should the deep-dive cite specific cognates as
load-bearing or hold them as substrate-watch?

Q3. **Component dependencies**: are C2→C7, C5→C6, C3→C5, C2→C3 the only
real dependencies? Or are there more? Should some dependencies be
classified differently (e.g., C5 as meta-component operating within
others)?

Q4. **C7 cognate**: neutrophil cognate is V1 proposal. Confirm or refine.

Q5. **Decay/sunset vocabulary gap**: separate primitive vs extension of
existing tolerance vs separate ADR? Phase 1-8 the options before
encounters-discipline ratifies the encounter.

Q6. **"Engineered-substrate-exceeds-biology at honest boundaries"
family**: three instances (W7, ADR-017, C4); ADR-006 threshold met. Is
this a candidate posture-class? Or encounters-tier first-recognition
pending more instances?

Q7. **"Honest-boundary as encounter-registration" discipline**: Tekgy's
2026-05-11 framing. Is this a candidate posture-class? Or part of the
encounters-discipline itself?

Q8. **Manifold structure of enumeration**: how should the deep-dive
treat the open-in-shape property? As an explicit V0+1 candidate? As a
posture? As a property of recognition-not-design at meta-meta level?

Q9. **Cross-language extension**: does multi-component framing break
when antigen extends beyond Rust? Some components are Rust-specific
(C1's proc-macros); others (C4 references, C6 trust delegation) may be
language-agnostic. Phase 1-8 the question.

Q10. **C4 within-engineered-boundary cognates**: V1 names library science,
academic integrity, info-warfare. Naturalist asked: are there sharper
cognates (academic publishing, legal precedent citation)? Worth Phase 1-8
treatment.

Q11. **Component 7 attack surface**: deep-dive flags as least-mapped.
Adversarial wants to extend the threat model; Phase 1-8 should surface
what's missing before A4+ scope-lock.

Q12. **Manifold axes worth surfacing**: production-vs-consumption,
static-vs-dynamic, individual-vs-population, implicit-vs-explicit. Are
these real axes or convenient analytic cuts? Phase 1-8 deconstructs.

---

## What this document is NOT

- Not a ratified framing. V1 draft for team Phase 1-8.
- Not a replacement for existing framings. Per "extend-not-replace at
  the framing level," failure-class-memory framing remains valid as
  floor.
- Not an implementation roadmap. Components are architectural framing,
  not development plan.
- Not authoritative on biology. Naturalist refinements supersede where
  they conflict.
- Not exhaustive on attack surface. Adversarial seams flagged; what's
  here is best-read, not comprehensive threat model.
- Not the final shape of the enumeration. New components, new axes,
  new tiers may surface.

---

## Acknowledgments

V1 processes substrate from:
- 2026-05-11 conversation between Tekgy and team-lead
  (`multi-component-immunity-conversation.md`)
- Scout's empirical pass: ADR-017/018 verification + Component 7
  promotion + vocabulary-as-protocol sharpening + candidate rulings +
  ADR-017 Amendment 1 candidates
- Naturalist's C4 boundary-silence finding + tier-structure framing +
  "engineered-substrate-exceeds-biology" family naming
- Adversarial's multi-component threat model: 8 findings + 5 amendment
  candidates + four A4+ contracts (committed 6b8c527) + A5 governance
  findings (held)
- Pathmaker's A3 implementation closure (commit 937fa0d, 235 tests
  passing) — the real-substrate grounding that this framing describes

Tekgy's V1-decisive framings:
- "Out-of-scope is first-encounter registration, not terminal
  declaration"
- "Could be flat and layered at the same time. Or multi-layered at
  each layer. Not a line but a manifold, or many manifolds. Who knows."

The recursion continues. There is no fixed point. We may find more
components as we keep recursing. The enumeration is open in shape, not
just in count.

*V1 authored 2026-05-11 by team-lead after team expansion pass and
Tekgy ratification of revise-cleanly approach. Open for aristotle Phase
1-8 + further naturalist refinements + adversarial-threat-model
extension as candidate components surface + manifold-axis exploration.
Subject to revision; not yet project-tier substrate.*
