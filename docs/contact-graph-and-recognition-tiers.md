# Antigen — Contact Graph and Recognition Tiers

> Forward-substrate document capturing two related findings:
>
> 1. **Three-tier cross-reactivity framework**: cross-reactivity is not a
>    single primitive; it operates at structural-shape, behavioral-assumption,
>    and contextual-assumption tiers. Each tier has its own failure mode,
>    its own measurable precision/recall, and its own tooling implications.
>
> 2. **Multi-modal transmission framework**: contact tracing is not limited
>    to `#[descended_from]` chains. The full code-relationship graph has
>    seven+ transmission modes (call graph, module proximity, trait impl,
>    macro expansion, descended_from, shared types, assumption-graph), each
>    with its own epidemiological cognate and tooling shape.
>
> Together these findings elevate antigen's recognition surface from
> *site-typed* (a fingerprint match is at one site) to *graph-typed* (a
> fingerprint match implies exposure across the contact graph rooted at
> that site, evaluated at three recognition tiers).
>
> **This document is recognition substrate, not design.** Per ADR-006: the
> findings name what *will* eventually be needed; they do not commit antigen
> to building any specific primitive on any specific timeline. v0.1.0
> ships descended_from + structural-shape-tier matching only; the rest is
> forward substrate for A3+ adoption-driven recognition.
>
> **Why captured now**: the substrate that produced these findings is
> substrate-context-window-bound. The same understanding may not surface
> in the same form in future sessions. Per ADR-001's structural-memory
> discipline, the carrier-strength hierarchy is highest when typed-substrate
> persists across context-windows. This document IS the typed substrate.
>
> **Companion to**:
> - [`immune-system-primitive-map.md`](immune-system-primitive-map.md) V0
>   (cross-reactivity + contact-tracing entries reference this document)
> - [`cross-domain-architectural-map.md`](cross-domain-architectural-map.md)
>   §8 (cybersecurity cognate for supply-chain propagation)
> - [`scope.md`](scope.md) (forward-substrate primitive map)

---

## Background: how this finding surfaced

This finding surfaced while tracing through what contact tracing could be in
antigen beyond `#[descended_from]` chains. An initial claim — that
"assumption-violation has no biological cognate at the cellular level, only
at the system-fitness level" — turned out to be wrong.

The counter-question was substrate-grounded: *might assumptions BE cellular?*
Working through it:

- Cell-surface receptors encode structural assumptions about ligand shapes
- Enzymes encode structural assumptions about substrate shapes
- DNA repair complexes encode structural assumptions about base-pair
  complementarity
- MHC presentation encodes structural assumptions about self/non-self
  distinction
- Cell-cell signaling encodes structural assumptions about target receptors

Each is a *molecular-recognition primitive that fails when the assumption
is violated by shape-similar-but-behaviorally-different inputs*. Receptors
binding wrong shapes; enzymes catalyzing wrong substrates; repair machinery
mis-correcting non-canonical pairs; immune system attacking self via
molecular mimicry.

Assumptions are cellular at the molecular-recognition level. That correction
propagated into a deeper finding: **antigen's fingerprint matching
primitive is structurally identical to molecular recognition**, and
cross-reactivity (which V0 already named) is not a single primitive but
a tiered phenomenon operating at every level of recognition.

That correction surfaced the three-tier framework. From there, the
multi-modal transmission framework named earlier (call graph + trait impl +
module proximity + macro expansion + shared types + descended_from) gained a
seventh mode — assumption-graph
transmission — and crystallized as a structurally distinct finding from
cross-reactivity tiers but operationally compositional with them.

Both findings together constitute the **graph-typed recognition surface**
finding. This document captures it.

---

## Part 1: Three-tier cross-reactivity framework

### Statement

Cross-reactivity in immunology is when an antibody binds molecules sharing
structural features beyond its intended target. V0's earlier entry treated
this as one primitive at the structural-shape level (fingerprint matches
multiple related failure-classes). The deeper finding: **cross-reactivity
operates at three distinct recognition tiers**, each with its own failure
mode and tooling implications.

### Tier 1: Structural-shape tier (canonical cross-reactivity)

**Biology**: an antibody binds molecules sharing surface epitope shape. The
binding is structurally correct; the antibody simply has multiple targets
with similar shapes.

**Antigen**: a fingerprint matches code AST patterns. Sometimes the intended
target is one specific failure-class but the fingerprint also matches
structurally-similar non-instances. Current W6a operators all live at this
tier (`item:`, `name:matches()`, `has_method`, `attr_present`,
`doc_contains`, `body_contains_macro` deferred to A4-A5).

**Failure mode**: false positives. The fingerprint fires on code that has
the structural shape but isn't actually an instance of the intended
failure-class.

**Measurable precision/recall**: structural-fingerprint-precision metric
(how many fingerprint matches are true positives?). This is the Hamming-
distance-discriminability concern from cross-domain-map §4 (information
theory).

**Recovery shape**: fingerprint refinement (narrow the structural pattern
until precision is acceptable); tolerance markers (`#[antigen_tolerance]`)
for sites that match structurally but are legitimate by design.

### Tier 2: Behavioral-assumption tier

**Biology**: a receptor binds a ligand of the right *shape*, but the
ligand's downstream signaling differs from the receptor's encoded
assumption. Examples: agonist vs antagonist with similar binding affinities;
molecular mimicry causing autoimmunity (a pathogen peptide shares shape
with a self-peptide; antibodies cross-react and attack self-tissue).

**Antigen**: code that *looks like* a known failure-class structurally but
has different *behavioral semantics*. The fingerprint match is correct
structurally; the *behavioral assumption* embedded in the fingerprint's
intent is violated.

**Concrete example**: a fingerprint matches "enum with strongest-first
discriminants and meet method" (the polarity-inverted-class-meet pattern).
A code site matches structurally — looks exactly like the pattern. But the
specific code site implements `meet` correctly (uses `max` not `min`). The
fingerprint match at tier 1 (structural) is correct; at tier 2 (behavioral),
the assumption that "this code is vulnerable to polarity inversion" is
violated because the behavior is correct.

**Failure mode**: structurally-correct match against behaviorally-correct
code. Two failure-shapes:
- *False alarm*: tooling flags code that doesn't need attention (developer
  fatigue → real threats ignored).
- *False non-alarm*: tooling fails to flag code that DOES need attention
  because the structural shape doesn't quite match (the inverse —
  behaviorally vulnerable but structurally novel).

**Measurable precision/recall**: behavioral-witness-precision metric. The
witness IS the behavioral check (per ADR-001 Amendment 1's witness-tier
discipline). When the witness runs and confirms behavior, tier-2 precision
is satisfied for that site.

**Recovery shape**: pair every structural fingerprint with a behavioral
witness pattern; the fingerprint matches → witness verifies → tier-2
classification ratifies the match. Witness-tier gradient (ADR-005 Amendment
3) operationalizes this when `WitnessTier::Execution` or higher passes.

### Tier 3: Contextual-assumption tier

**Biology**: a molecule binds correctly and behaves correctly in isolation,
but violates assumptions made by *downstream cascades* about its operating
context. Examples: a hormone whose concentration is normal but whose
*timing* deviates from circadian assumptions; a signal whose magnitude is
correct but whose phase relative to other signals is wrong; a normally-
expressed protein in an aberrant cellular context (right molecule, wrong
neighborhood).

**Antigen**: code whose shape matches AND whose behavior is locally-correct,
but whose downstream *callers* depended on assumptions the code violates.
The local correctness is real; the contextual contract violation is real;
both are simultaneously true. This is the "assumption-graph" failure mode
that motivated this document's authoring.

**Concrete examples**:
- A function that doesn't actually panic, but whose downstream callers
  assumed panic-freedom and built load-bearing logic on that assumption.
  The function works locally; downstream invariants depend on the local
  correctness propagating through specific call paths the function doesn't
  guarantee.
- A function that's locally thread-safe but whose downstream callers
  assumed a stronger property (e.g., re-entrancy safety) the function
  doesn't actually provide.
- An enum variant whose discriminant ordering is correct in isolation, but
  whose downstream serialization code assumes a specific ordering that's
  one variant deeper than the local guarantee.

**Failure mode**: locally-correct code violates contextual contracts that
downstream code depends on. The local site passes all tests; the downstream
site fails for reasons that look unrelated.

**Measurable precision/recall**: contextual-assumption-precision metric.
This requires reasoning about *what downstream code assumes* — substantially
harder than local-behavioral verification. Requires call-graph + trait-impl-
graph traversal (per Part 2 of this document).

**Recovery shape**: contact-graph traversal to identify downstream
assumption-dependents; explicit assumption-declarations on functions that
make load-bearing contextual claims; tier-3-aware audit reporting that
distinguishes "locally verified" from "contextually verified."

### Why three tiers matter (architectural implication)

Antigen's recognition surface has three different ways matching can succeed
(at one tier) while still being *wrong* at deeper tiers. Tooling that only
addresses tier 1 (current W6a) catches structural false positives but
misses behavioral and contextual failures. Tooling that only adds tier 2
(witnesses verifying local behavior) misses contextual failures. Full
coverage requires graduated tooling at all three tiers.

This isn't a roadmap for v0.1.0; it's recognition substrate for the
architectural maturation arc. v0.1.0 ships tier 1 with structural
fingerprinting + tolerance markers. Witness pluralism (ADR-013) +
witness-tier gradient (ADR-005 Amendment 3) ship the foundation for tier 2.
Tier 3 awaits A3+ when contact-graph traversal is operational.

### Connection to biology's cross-reactivity literature

Biology recognizes cross-reactivity as a phenomenon that operates at
multiple levels — cellular receptor cross-reactivity, antibody-antigen
cross-reactivity, T-cell-receptor cross-reactivity. Each level has its
own characteristic failure modes (autoimmunity at cellular level;
allergies at antibody level; transplant rejection at T-cell level). The
tiered framework above maps cleanly: each biological cross-reactivity tier
has its antigen analog at the corresponding recognition tier.

This is an instance of the structural-identity claim in scope.md and
CLOSURE.md — biology and antigen instantiate the same operational property
in different substrates. Cross-reactivity-at-multiple-tiers is one of those
properties.

---

## Part 2: Multi-modal transmission framework

### Statement

Contact tracing in antigen is not limited to `#[descended_from]` chains.
The full code-relationship graph has multiple "transmission modes" —
different ways a "sick" code site (one that presents an antigen) can
affect related code without direct executed contact. Each transmission
mode has a different graph-type that antigen tooling could traverse, and
each maps to a different epidemiological cognate.

### The seven transmission modes

| Mode | Code relationship | Graph-type | Antigen exposure shape |
|---|---|---|---|
| **Direct contact** (skin-to-skin) | Call graph: A calls B | Call-graph | Function presenting antigen → caller is structurally exposed via the call relationship |
| **Droplet transmission** (proximity) | Module proximity: same file/mod | Module-graph | Local invariant assumptions in nearby code; shared imports; shared private state |
| **Airborne transmission** (long-range) | Trait implementation: `impl X for Y` | Trait-impl-graph | Trait Y's behavior is exposed everywhere X is used as a bound or trait object — long-range structural transmission |
| **Vector transmission** (mosquitoes) | Macro expansion: macro M used at N call sites | Macro-expansion-graph | Macros carry the pattern across many call sites without direct contact at any specific site |
| **Vertical transmission** (parent → child) | Descended_from inheritance | Descended-from-graph | Currently operational; the only graph antigen reasons over today |
| **Fomite transmission** (shared objects) | Shared types / traits / generics | Type-relation-graph | `Container<T>` as carrier; the type itself is the surface that propagates between callers |
| **Assumption-graph transmission** | Downstream contextual assumptions | Assumption-graph | Cross-reactivity at the contextual-assumption tier (per Part 1, Tier 3) — sick-but-locally-correct code violates downstream invariants |

### Operational implications per mode

Each transmission mode has its own:
- **Infectious-period bounds**: how long does an exposure persist? Direct
  contact (a single call) is bounded by call lifetime; trait-impl-graph
  transmission persists across the codebase until the impl is removed.
- **Intervention strategy**: what kind of mitigation is meaningful? Direct-
  contact exposure can be mitigated by adding a witness at the call site;
  trait-impl-graph exposure may require mitigation across all sites that
  use the trait.
- **Tooling integration**: what existing infrastructure provides the graph?
  Call graph and trait-impl-resolution come from rust-analyzer; macro-
  expansion-graph requires `cargo expand` or proc-macro2 introspection;
  descended_from comes from antigen's own scan.

### The architectural commitment this elevates

Currently antigen reasons over *one graph* (descended_from). The contact-
graph framing says antigen will eventually consume *multiple graph types*
as substrate.

That elevates antigen's recognition surface from **site-typed** (a
fingerprint match is at one site) to **graph-typed** (a fingerprint match
implies exposure across the contact graph rooted at that site, evaluated
across multiple transmission modes at three recognition tiers).

This is a substantial architectural maturation arc. v0.1.0 ships site-typed
+ tier-1-cross-reactivity. The full graph-typed surface across all three
tiers is A4+ work (likely several sweeps).

### Existing rust-analyzer infrastructure

Most of the graph substrate antigen would need is already produced by
rust-analyzer:

- **Call graph**: rust-analyzer maintains call resolution.
- **Trait-impl resolution**: rust-analyzer's trait solver resolves which
  impls apply to which types.
- **Type relations**: rust-analyzer tracks generic instantiations.
- **Macro expansion**: rust-analyzer expands proc-macros and macro-rules
  on demand.
- **Module structure**: rust-analyzer tracks the workspace's module hierarchy.

Antigen-tooling integration becomes "consume rust-analyzer's graphs as
substrate, walk them with antigen-aware queries." This is the same
architectural posture as ADR-002 (compose, don't compete) — antigen
doesn't reinvent the graphs; it leverages existing tooling.

The exception is the assumption-graph (Mode 7 / Tier 3) — there is no
existing infrastructure that captures contextual assumptions. This may
require new declarations on functions that make load-bearing contextual
claims (something like `#[invariant("panic-free")]` or
`#[assumes("ordering: lexicographic")]`). Future-ADR territory; out of
scope for now.

### The biological richness extends beyond just transmission modes

Public-health epidemiology has rich vocabulary that maps cleanly:

- **Reproductive number (R₀)**: how many secondary infections per primary?
  Antigen analog: how many code sites exposed per ancestor pattern?
  Stdlib antigens with high R₀ are keystone-class (per
  cross-domain-map.md §3 ecology cognate); low R₀ are niche-class.

- **Patient zero**: the original case. Already encoded in antigen via the
  `references = ["GAP-BIT-EXACT-1"]` field. Every antigen knows its index
  case.

- **Quarantine**: PR-gating — when a PR introduces new transmission-mode
  contacts (descended_from chains, call-graph contacts, trait-impl
  surfaces, etc.) from an antigen-marked ancestor, gate the PR until
  immunity is declared OR tolerance is justified. CI-time public-health
  intervention.

- **Cluster identification**: families of types sharing an ancestor that
  presents an antigen. The `family` field on `#[antigen]` partially
  encodes this; contact-graph traversal operationalizes it.

- **Index case investigation**: "where did this start, and what was the
  chain?" Forward trace from references field; backward trace through
  descended_from + other transmission modes.

### Cross-crate version (post-A3)

The full contact-graph framework becomes especially powerful at cross-crate
scope:

> A computational-mathematics project's `Container<T>` was just declared as
> presenting X. `cargo antigen trace --cross-crate` shows: every project
> depending on that crate that uses Container<T> is structurally exposed.

That's CVE-style supply-chain propagation but for *failure-classes
generally*, not just security. The cybersecurity precedent (CVE in
dep-graph affects every dependent project) maps directly. See
cross-domain-architectural-map.md §8 + Appendix A for governance precedent.

---

## Part 3: How the two findings compose

The three-tier cross-reactivity framework (Part 1) and the multi-modal
transmission framework (Part 2) are *compositional*, not redundant. Together
they produce a full matrix of antigen's recognition surface:

```
                    Tier 1                  Tier 2                  Tier 3
                 (structural)         (behavioral)         (contextual)
                 ___________          ___________          ___________
Mode 1 (call)   |  T1×M1     |       |  T2×M1     |       |  T3×M1     |
Mode 2 (mod)    |  T1×M2     |       |  T2×M2     |       |  T3×M2     |
Mode 3 (trait)  |  T1×M3     |       |  T2×M3     |       |  T3×M3     |
Mode 4 (macro)  |  T1×M4     |       |  T2×M4     |       |  T3×M4     |
Mode 5 (desc)   |  T1×M5  ✓  |       |  T2×M5     |       |  T3×M5     |
Mode 6 (type)   |  T1×M6     |       |  T2×M6     |       |  T3×M6     |
Mode 7 (assum)  |  T1×M7     |       |  T2×M7     |       |  T3×M7  *  |
                 ¯¯¯¯¯¯¯¯¯¯¯           ¯¯¯¯¯¯¯¯¯¯¯           ¯¯¯¯¯¯¯¯¯¯¯
✓ = currently operational in v0.1.0 (descended_from + tier-1 structural)
* = tier-3 + mode-7 is the same finding intersecting itself (assumption-violation
    at the contextual tier IS what assumption-graph transmission is)
```

Most cells (3×7 = 21) are forward substrate. Currently operational: T1×M5
(structural-tier matching against descended_from-graph traversal). Witness
pluralism (ADR-013) + witness-tier gradient (ADR-005 Amendment 3) lay
foundation for the T2 column. The full matrix is A4-A5+ work.

The matrix isn't a roadmap for what to build — it's recognition substrate
for what the architecture has space to grow into. Per ADR-006: build cells
when adoption surfaces real instances that need them.

---

## Part 4: Manuscript trajectory implications

This document's framings have implications for the multi-paper publication
trajectory (per cross-domain-architectural-map.md Appendix B):

### Tool paper (post-v0.1.0)

The graph-typed-recognition-surface framing is too forward-looking for the
tool paper. v0.1.0 ships site-typed + tier-1; the tool paper documents
what shipped. The contact-graph framework is mentioned briefly as future
work but not centered.

### Foundational paper (future)

This is where graph-typed recognition becomes load-bearing. The
architectural-identity claim ("antigen and biology instantiate the same
property in different substrates") gains precision when expressed as
"both use cross-reactivity-at-three-tiers + multi-modal-transmission."
Section material:

- A subsection on "the three tiers of recognition" — uses biology's
  receptor / enzyme-substrate / system-context examples to ground the
  three-tier framework
- A subsection on "the multi-modal transmission framework" — uses
  epidemiology's transmission-mode taxonomy to ground the seven graph
  types antigen reasons over

### Methodology paper

The graph-typed-recognition framing is less central but still relevant:
the team's recognition-of-its-own-recognition discipline (substrate-currency,
depth-shift) operates at multiple tiers and across multiple "transmission
modes" of agent communication (direct messages, observed substrate,
inferred state). The framework predicts methodology refinements at scale.

### Code immunology / immunology journal paper

This is where the contact-graph + cross-reactivity-tiers framework lands
most cleanly. The biology audience already knows these primitives; the
contribution is the structural-identity claim — that antigen instantiates
the same primitives in code-relationship-graphs that immune systems
instantiate in molecular-recognition + cellular-signaling. Mapping table
becomes the headline result for this audience.

### Public health / epidemiology journal paper

The multi-modal transmission framework + contact tracing primitive maps
cleanly to public-health concerns. The contribution: applying public-
health epidemiological tooling concepts to software ecosystem structural
failures. R₀, patient zero, quarantine, cluster identification, index
case investigation — all translate. This is a venue-targeted paper that
extends the project's reach into a field that hasn't typically engaged
with software-engineering tooling.

---

## Part 5: Recognition triggers

This document is forward substrate. Recognition triggers for ratification
(per ADR-006):

- **Tier 2 (behavioral-assumption) gets operationalized** when stdlib
  reaches the point where adversarial test fixtures need to specifically
  generate "structurally-similar but behaviorally-different" code that
  should NOT match. The cog-sci framing in cross-domain-map §1 (Gentner's
  surface-similarity-vs-relational-similarity) predicts this need; this
  document's three-tier framework operationalizes it.

- **Tier 3 (contextual-assumption) gets operationalized** when adoption
  surfaces real cases of "code locally-correct but downstream-impacted."
  Path-dependent adoption by a downstream consumer crate in A3+ may surface
  this when stdlib antigens affect that crate's assumptions.

- **Multi-modal transmission gets operationalized** when post-A3 cross-crate
  scan reveals graph-effects of single antigen declarations. The "a
  downstream crate's Container<T> propagates exposure to its dependents"
  scenario is the trigger.

- **The matrix as governance metric** when antigen-stdlib reaches multi-
  thousand antigen scale. Per cross-domain-map.md Appendix A.5 (ATT&CK
  sub-technique decomposition criteria), decomposition is recognition-
  driven, not designed-ahead. The 21-cell matrix predicts where decomposition
  pressure will surface; specific decompositions wait for adoption to
  surface them.

---

## Closing posture

The graph-typed-recognition-surface finding is the deepest single
architectural finding from the A2 design phase. It
elevates antigen's recognition primitive from "fingerprint matches at a
site" to "fingerprint matches at a graph-typed surface across three
recognition tiers."

This is forward substrate, not commitment. v0.1.0 ships site-typed +
tier-1 + descended_from only. The full architecture is many sweeps away.
This document captures the framework so that when adoption surfaces real
instances of tier-2, tier-3, or non-descended-from transmission modes,
the team recognizes the prior art rather than re-deriving it.

The two findings (three-tier cross-reactivity + multi-modal transmission)
emerged together because they are compositional aspects of the same deeper
finding: antigen's recognition surface is graph-typed, evaluated at three
tiers. Capturing them together in this document preserves the compositional
structure that one-at-a-time entries in the primitive map V0 obscured.

Per the substrate-over-memory discipline (carrier-strength hierarchy,
ADR-001 Amendment 1): this document IS the highest-tier carrier for these
findings. The original session-context where the findings emerged is
ephemeral; this typed-substrate carries the findings forward.

---

*V0 authored 2026-05-08, capturing findings from a design discussion.
Companion to the immune-system primitive map V0 and the cross-domain
architectural map V1. Maintained as adoption
pressure surfaces tier-2 / tier-3 / non-descended-from-mode instances that
warrant deeper-tier matrix cell ratification.*
