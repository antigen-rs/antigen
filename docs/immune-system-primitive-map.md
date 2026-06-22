# Antigen — Immune-System Primitive Map

> Forward-substrate map of biological/medical/virological/public-health
> primitives that could instantiate as antigen ecosystem tooling primitives —
> starting substrate for deeper biology-cognate expansion.
>
> **This document is recognition substrate, not design.** Per ADR-006: each
> primitive listed here lands in antigen as a tooling primitive *when adoption
> surfaces a real instance that needs it*. The map is forward — it names what
> biology already has answers to that we will eventually need answers to. It
> does NOT commit the project to building any specific primitive on any
> specific timeline.
>
> **Companion to**: [`scope.md`](../scope.md) (the comprehensive vision); each
> primitive in this map is a potential future ADR / sweep / tool surface
> visible from scope.md's "comprehensive immune-system primitive map" table
> but expanded into substantive territory exploration.
>
> **Scale framing — important**: this catalog is a **seed set for unbounded
> ecosystem growth**, NOT a bounded enumeration of "the primitives antigen
> will eventually have." Vertebrate immune systems carry on the order of
> 10^11 distinct antibody specificities. The antigen ecosystem's eventual
> contribution scale — across stdlib + community + domain-specific antigen
> libraries + per-project antigens — is comparably unbounded. Each named
> primitive here is a category that could spawn many specific instances; the
> total count of failure-classes antigen recognizes when adoption matures
> will be vast. Read this document as "which biological architectural
> patterns predict what antigen will need" — not as "the complete list
> of primitives antigen will build." The complete list is the open
> ecosystem itself.

---

## Why this document exists

Biology has been iterating on recognition-with-memory-and-inheritance for
~500 million years. The vocabulary is rich, the architecture is empirically
refined, and the failure modes are well-characterized. Every primitive in the
biological immune system is a structural answer to a problem antigen will
eventually face when adoption scales.

The strongest argument for biology-as-load-bearing (per ADR-003, with
biology-as-search-heuristic showing 5/5 prediction precision)
is that biology *predicts* primitives the project hasn't built yet. This
document catalogs those predictions explicitly so future sweeps can recognize
them as adoption surfaces specific needs.

Per the discipline ratified in ADR-006: the project does not pre-build
speculative primitives. This document does not change that. What it does is
*name* the primitives biology has answers for, so when a real instance
arrives, the team recognizes the pattern rather than re-deriving it.

---

## Already operational (named here for the reader)

These primitives are shipped or in-flight in v0.1.0-rc.1:

- **Antigens** → `#[antigen]` declarations
- **Antibodies** → witnesses (test, proptest, phantom-type, lint, formal-verification)
- **MHC presentation** → `#[presents(antigen)]`
- **Pathogen Recognition Receptors (PRRs)** → fingerprint matchers in `cargo antigen scan`
- **T-cell receptors** → named-failure-class fingerprints
- **B-cell memory (pattern layer)** → `#[antigen]` declarations persisting past bugs
- **Antibody titer (currency layer)** → `verified_at` claims (ADR-016)
- **B-cell lineage / clonal expansion** → `#[descended_from]` propagation
- **Vaccination** → `cargo antigen vaccinate` (planned, A5)
- **Peripheral tolerance / Tregs** → `#[antigen_tolerance]`
- **Innate vs adaptive immunity** → passive surface (fingerprint scan) vs active surface (markers)
- **Affinity maturation** → W6a synthesis pass (recognized post-hoc; biology predicted, team built independently)

For these, biology and antigen have already converged structurally. The remaining sections are forward substrate.

---

## Classical immunology — primitives not yet instantiated

### Macrophages (phagocytosis: consume + present)

**Biology**: macrophages engulf pathogens, break them down, present antigen
fragments via MHC to T-cells. They bridge the "stuff in the body" surface
(extracellular pathogens) with the recognition surface (MHC-presented antigens).

**Potential antigen instantiation**: code-consumer tools that walk macro
outputs, build.rs codegen, external dependencies, or otherwise opaque code
and present what's inside as antigen-knowable substrate.

`#[antigen_generates]` (ADR-014) is a primitive instance — proc-macro authors
declare what their macro emits. A macrophage-shaped tool would go further:
*automatically* expand macros (via `cargo expand`), walk the expansion, scan
for fingerprint matches, and present synthetic `#[presents]` for the matches.
Closes the structural-blindness gap that ADR-014 only partially addresses
(ADR-014 requires macro-author cooperation; macrophage-tooling does not).

**What would trigger instantiation**: real-world adoption hitting structural
blindness on closed-source proc-macros (third-party derives) where
`#[antigen_generates]` cooperation isn't available.

**Tooling shape**: cargo subcommand or scan extension. Likely depends on
`cargo expand` for stable Rust; nightly-only for unexpanded variants.

---

### Dendritic cells (bridge innate to adaptive immunity)

**Biology**: dendritic cells present antigen they've encountered to T-cells in
lymph nodes, triggering specific adaptive response. They are the
*decision-routing* primitive — innate immunity sees something, dendritic cells
escalate to adaptive immunity if needed.

**Potential antigen instantiation**: audit pass that takes scan-detected
fingerprint matches and routes them to specific defenses with
provenance — *suggests which witness type would prove immunity for this
specific match*. Currently `cargo antigen scan` reports "needs immunity"
without specifying *which kind* of immunity makes sense for the match.

**What would trigger instantiation**: adoption pressure when newcomers hit
"I see the warning but don't know what to do" — friction point that
dendritic-cell-shaped tooling resolves.

**Tooling shape**: scan output extension; potentially LSP integration so the
IDE surfaces "for this match, witness options are: kani::proof, proptest,
phantom-type construction; here are templates."

---

### Complement system (tag for destruction)

**Biology**: complement proteins coat pathogens, marking them for
phagocytosis, lysis, or other immune cell action. They are the *signal*
between recognition and removal.

**Potential antigen instantiation**: refactor-suggestion tool that marks code
presenting antigens with structural fix recommendations — not "this is bad"
but "this presents X antigen and lacks immunity; here's the family of fixes
that work for this antigen-type."

**What would trigger instantiation**: stdlib antigen catalog reaching
critical mass (post-A5, ~20+ antigens) where each antigen has multiple known
fix patterns. Complement-tooling becomes useful when there's accumulated
folk-wisdom about *how* to address each antigen-class.

**Tooling shape**: cargo subcommand (`cargo antigen suggest <antigen>`) or
LSP code-action integration.

---

### NK cells (recognize abnormal without specific antigen) — partially shipped via W6a

**Biology**: natural killer cells detect *cells that lack normal markers* —
cells that have lost MHC presentation (often virally-infected). They don't
need a specific antigen; they recognize the absence of self-markers.

**Potential antigen instantiation**: anomaly-detection tooling that flags
*structurally unusual* code in your codebase even when no antigen has been
named — outlier detection over a large fingerprint corpus.

**Status**: partially shipped. W6a's
synthesis pass (fingerprint-match-without-marker) was identified AS the NK-cell primitive —
the team built it without naming it that. Full NK-cell behavior would extend
to *outlier detection* (not just antigen-fingerprint matches but
"this code is structurally unlike anything else in your codebase, including
unlike any declared antigen") — that's deeper.

**What would trigger full instantiation**: codebase-statistical analysis
becomes useful when antigen-stdlib is large enough that outlier detection
relative to it is meaningful (post-A5).

**Tooling shape**: scan extension with statistical model over known
fingerprint distribution.

---

### Cytokines (signaling propagation)

**Biology**: cytokines are signaling molecules released during immune
response that recruit additional immune cells, modulate inflammation, and
coordinate adaptive response across distant cells. They are the
*ecosystem-wide signal* primitive.

**Potential antigen instantiation**: cross-crate antigen propagation signals.
`#[descended_from]` is the simplest cytokine instance; richer propagation
would include:
- Antigen-discovery announcements when new antigens appear in dependency
  graph (you depend on a crate that just added an antigen; should you check
  your code?)
- Witness-failure notifications across crate boundaries when an upstream's
  witness becomes invalid (your immune declaration referenced their proof;
  their proof now stale)
- Stdlib propagation: when antigen-stdlib gains a new failure-class, all
  dependent projects get re-scanned automatically on next `cargo update`

**What would trigger instantiation**: A3 cross-crate scan baseline + A5
antigen-stdlib propagation. Cytokine-shaped tooling becomes useful once
cross-crate antigen graphs are real.

**Tooling shape**: cargo subcommand integration with cargo's update
mechanism; cargo extension that runs on dependency change.

---

### Plasma cells (short-lived antibody factories)

**Biology**: B-cells differentiate into plasma cells when activated; plasma
cells produce antibodies at high rate for a limited time. They are the
*scale-up* primitive — once specific antibody is needed, plasma cells make
lots of it.

**Potential antigen instantiation**: witness templates that *generate*
defenses from patterns. When an antigen is declared, plasma-cell
tooling could generate boilerplate witness functions tailored to the
antigen's fingerprint shape:
- `panicking-in-drop` antigen → generates `no_panic_in_drop_test` template
  with the shape of the test pre-filled
- `polarity-inverted-class-meet` → generates lattice-property proptest
  template

**What would trigger instantiation**: ergonomic friction during A5+ when
practitioners adopt antigen-stdlib and have to write witnesses — boilerplate
is the friction; plasma-cell generation removes it.

**Tooling shape**: `cargo antigen new-witness <antigen>` subcommand;
rust-analyzer code-action; macro-derive for common witness shapes.

---

### MHC Class I vs II (intracellular vs extracellular antigen presentation)

**Biology**: MHC Class I presents *intracellular* antigens (made inside the
cell, like viral proteins replicating in the cell); MHC Class II presents
*extracellular* antigens (engulfed by macrophages). Different presentation
surfaces for different antigen origins.

**Potential antigen instantiation**: distinction between *internal-state
antigens* (failure-classes about how a function manages its own state) vs
*external-contract antigens* (failure-classes about a function's interface
with callers).

Currently antigen doesn't distinguish — `#[presents(X)]` is `#[presents(X)]`
regardless of whether X is internal-state or external-contract. MHC Class
distinction would let scan/audit tools route different visualization, witness
recommendations, or refactor patterns based on the failure-class's nature.

**What would trigger instantiation**: stdlib antigen accumulation reaching
the point where the internal-vs-external distinction matters for ergonomics
and the set of failure-classes that fall into each class is non-trivial.

**Tooling shape**: optional `class: I | II` field on `#[antigen]`; or
`presentation: Internal | External | Both`. ADR territory; would require
ADR-001 amendment for the new field.

---

### Regulatory T-cells (prevent overreaction) — partially shipped via tolerance

**Biology**: Tregs suppress immune response to prevent autoimmunity;
peripheral tolerance is one mechanism. They are the *don't-overreact* primitive.

**Status**: partially shipped via `#[antigen_tolerance]` (ADR-011) for
explicit per-site tolerance. Full Treg behavior would extend to:
- *Auto-tolerance learning*: if scan flags 10 sites for the same fingerprint
  and 9 are explicitly tolerated, the tool suggests narrowing the fingerprint
  rather than tolerating each site individually
- *Tolerance-pattern inference*: detecting fingerprints that systematically
  produce false positives across many projects
- *Soft tolerance*: sites that don't quite match the fingerprint but are
  close enough to be flagged at lower severity

**What would trigger instantiation**: post-stdlib adoption surfacing
fingerprints that produce too many false positives. Auto-tolerance learning
becomes load-bearing.

**Tooling shape**: scan extension with statistical pattern detection;
fingerprint-quality metrics in audit output.

---

### Vaccine modalities (live, inactivated, subunit, mRNA)

**Biology**: different vaccine technologies for different scenarios:
- *Live attenuated*: weakened pathogen; strong response, longer immunity
- *Inactivated*: killed pathogen; weaker response, safer
- *Subunit*: just the antigen protein; targeted but limited
- *mRNA*: instructs cells to make antigen protein themselves; fast to develop

**Potential antigen instantiation**: different strategies for applying
immunity patterns:
- `cargo antigen vaccinate` for bulk family application (live-attenuated
  shape — comprehensive but might over-apply)
- Per-site `#[presents]` + `#[defended_by]` declarations (subunit shape — targeted, per-instance)
- `#[descended_from]` inheritance (mRNA-like — instructs the type system to
  manufacture immunity from pattern)
- Witness templates from plasma-cell tooling (inactivated-like — boilerplate
  scaffolding)

**What would trigger instantiation**: A5+ when multiple immunity-application
strategies are real and choosing-between-them is a real ergonomic question.

**Tooling shape**: documented patterns rather than separate tools. Might
ratify as ADR explaining the modality choice space.

---

## Forward substrate from biology — primitives we haven't named yet

These are biology/immunology primitives that don't appear in scope.md's
current primitive map but are worth naming as forward substrate.

### Herd immunity (collective protection from individual immunity)

**Biology**: when enough individuals in a population are immune to a
pathogen, transmission slows or halts even for non-immune individuals.
Population-level emergent property, not individual-level.

**Potential antigen instantiation**: ecosystem-wide network effect. As more
projects adopt antigen-stdlib + project-specific antigens, the *probability*
that a new failure-class instance gets caught somewhere in the dependency
graph rises — even projects that don't directly adopt antigen benefit
because their dependencies catch upstream issues.

**What would trigger instantiation**: this is mostly a framing primitive,
not a tooling primitive. Worth naming for ecosystem-outreach pitch material
(post-v0.1.0). Could become a measurable property — "X% of crates.io top-100
crates have antigen declarations; failure-class spread rate has decreased
Y% since adoption."

**Tooling shape**: ecosystem-statistics tool; periodic survey; not a per-project tool.

---

### Latency (pathogens that lie dormant)

**Biology**: HIV, herpes, varicella-zoster — pathogens that establish
infection then go dormant, reactivating under specific conditions (immune
suppression, stress, age).

**Potential antigen instantiation**: *latent failure-classes* — code that's
currently fine because of guarantees X, Y, Z, but would fail if any of those
guarantees changed. Conditional vulnerability that's invisible in the current
state but structurally present.

Example: code that assumes `len() <= isize::MAX` because of a current bound
elsewhere; if that bound changes, the assumption fails. The latent failure-
class is "assumes a bound that's currently enforced elsewhere"; the trigger
is when the bound changes.

**What would trigger instantiation**: post-A3 cross-crate scan when
upstream changes can invalidate downstream assumptions. ADR territory:
how does antigen represent conditional/contextual vulnerability?

**Tooling shape**: extended fingerprint syntax for "depends-on" relationships;
or a separate `#[latent_antigen]` carrier that activates only when
preconditions change.

---

### Cross-reactivity (antibody recognizes multiple related antigens) — three tiers

**Biology**: an antibody raised against one antigen sometimes binds related
antigens with shared structural features. Useful for breadth (broad-spectrum
defense); dangerous for false positives (autoimmune diseases involving
cross-reactive antibodies; molecular mimicry where pathogen shapes resemble
self-peptides). Cross-reactivity is *molecular recognition with shape-violation
beyond the antibody's intended target*.

**Cross-reactivity is not a single primitive — it operates at three tiers**:

1. **Structural-shape tier (canonical)**: an antibody binds molecules sharing
   surface epitope shape. Antigen analog: a fingerprint matches code AST
   patterns. The structural match is correct; sometimes the *intended*
   target is one specific failure-class but the fingerprint also matches
   structurally-similar non-instances. Current W6a operators all live at
   this tier.

2. **Behavioral-assumption tier**: a receptor binds a ligand of the right
   *shape*, but the ligand's downstream signaling differs from the
   receptor's encoded assumption (e.g., agonist vs antagonist with similar
   binding affinities; molecular mimicry causing autoimmunity). Antigen
   analog: code that *looks like* a known failure-class structurally but
   has different *behavioral semantics*. The fingerprint match is correct
   structurally; the behavioral assumption embedded in the fingerprint's
   intent is violated. Future tooling territory.

3. **Contextual-assumption tier**: a molecule binds correctly and behaves
   correctly in isolation, but violates assumptions made by *downstream
   cascades* about its operating context (e.g., a hormone whose
   concentration is normal but whose *timing* deviates from circadian
   assumptions; signal whose magnitude is correct but whose phase relative
   to other signals is wrong). Antigen analog: code whose shape matches
   AND whose behavior is locally-correct, but whose downstream *callers*
   depended on assumptions the code violates (panic-freedom assumed by
   downstream that doesn't actually hold; thread-safety assumed by
   downstream that the code doesn't guarantee). This is the
   "assumption-graph" failure mode — sick-but-localized code still
   downstream-infects via violated assumptions.

**Why three tiers matter**: V0's earlier framing treated cross-reactivity as
a single primitive (fingerprint matches multiple related failure-classes).
The deeper finding: *every level of recognition has its own cross-reactivity
failure mode*. Antigen's tooling will eventually need fingerprint-precision
metrics at each tier — structural-shape precision (does this AST pattern
match?), behavioral-assumption precision (does the matched code *behave* like
the failure-class instance the fingerprint targets?), contextual-assumption
precision (do downstream callers assume what this code provides?).

**Where biology DOES have cellular cognates for assumption-violation**: every
receptor, enzyme, signaling molecule, and DNA repair complex encodes
*assumptions about the molecular shapes it will encounter*. Receptor binding,
enzyme-substrate specificity, MHC self/non-self distinction, Watson-Crick
base pairing — each is a structural assumption embedded in molecular
machinery. Violation of the assumption produces wrong downstream behavior
even without "direct executed contact" in any classical sense. Assumptions
are cellular at the molecular-recognition level. The architectural-identity
between antigen's fingerprint matching and biological molecular recognition
is exact at this level: both encode structural assumptions that downstream
behavior depends on; both fail when the assumption is violated by
shape-similar-but-behaviorally-different inputs.

**What would trigger instantiation per tier**:
- Tier 1 (structural-shape): stdlib refinement after adoption — patterns
  reveal that two declared antigens are structurally identical or one
  fingerprint is matching both intended and unintended structural-shape
  failure-classes.
- Tier 2 (behavioral-assumption): antigen-stdlib mature enough that
  fingerprint authors can encode behavioral predicates (per ADR-001
  Amendment 1's witness-tier discipline — execution-tier witnesses verify
  this kind of assumption).
- Tier 3 (contextual-assumption): A3+ when cross-crate scan surfaces
  cases where code matching a fingerprint affects downstream assumptions
  in distant code without direct call relationships.

**Tooling shape per tier**:
- Tier 1: fingerprint-quality metrics; antigen-deduplication tooling;
  ADR for "conjoined antigens" — multiple failure-class names sharing a
  fingerprint.
- Tier 2: behavioral-witness templates; tier-aware audit reporting (already
  partially shipped per WitnessTier in ADR-005 Amendment 3).
- Tier 3: contact-graph traversal (see contact-tracing entry below);
  assumption-graph instrumentation; rust-analyzer integration for
  surfacing assumed invariants alongside fingerprint matches.

See companion document [`contact-graph-and-recognition-tiers.md`](contact-graph-and-recognition-tiers.md)
for substantive expansion of the three-tier framework + multi-modal
transmission framework that operationalizes contact tracing across the
relationship-graph types antigen will eventually need to traverse.

---

### Original antigenic sin

**Biology**: the immune system gets locked into responding to the *first
variant* of a pathogen it sees, even when later variants need different
responses. The first encounter biases all subsequent ones.

**Potential antigen instantiation**: when an antigen's fingerprint is based
on the *first instance* and later instances need fingerprint refinement,
but the project's been declaring defenses (`#[defended_by]`) against the original —
invalidation cascade. Old defenses may need re-evaluation when the
fingerprint refines.

**What would trigger instantiation**: ADR-016's `verified_at` is part of the
answer (re-attestation when fingerprint changes). Fuller answer requires
fingerprint-versioning and migration tooling.

**Tooling shape**: `cargo antigen migrate <antigen>` for fingerprint version
bumps; immunity-claim re-validation pass.

**Note (2026-05-26, dx-dogfood)**: original-antigenic-sin is a **cross-family
cognate** — it also grounds the *fingerprint-scope* failure (a fingerprint fitted
to instance #1 diverges from the class's extension; `AntigenFingerprintDiverges-
FromClassExtension`) and the VCS `RefactorWithoutPreservationOfWhy`. See the
ratified glossary entry (`docs/glossary.md`, Recognition terms) for the general
definition + both families + the affinity-maturation remedy. The fingerprint-
versioning instantiation above is one of three uses; the cognate is more general
than this entry alone.

---

### Idiotype network / anti-idiotype (recognition recognizing recognition)

**Biology**: antibody variable regions carry **idiotopes** — determinants that are
themselves antigenic. **Anti-idiotype antibodies** recognize *other antibodies'*
recognition-regions; the immune system's recognition-apparatus is itself an antigen
to the system. Jerne's network theory (Nobel 1984): the idiotype network self-
regulates the repertoire (anti-idiotypes balance responses). Crucially, this is
recognition-of-recognition **across cells** — anti-idiotype binds *another* antibody,
not itself — so self-reach is a **network** property, not reflexive single-molecule
self-binding (see "Where biology goes silent").

**Potential antigen instantiation**: the **description-tier** of the F10
fundamentality-test — a fail-class whose subject IS the description-referent relation
(spec-vs-impl, receptor-vs-class) can have its own *declaration* (a description)
exhibit the class. The tier marker `{object | description}` (ratified F10) is the
local projection; the network-level form is the standing **taxonomy-completeness-
pressure** (does the 8-class taxonomy, a description of the failure landscape, cover
the landscape including its own description?). Anti-idiotype regulation balancing the
repertoire ↔ the taxonomy regulating its own coverage.

**What would trigger instantiation**: a `tier` metadata field on antigen declarations
(F10 (b), separate ratification unit — does NOT ride the F6 encounter-status
amendment; different subject) + the taxonomy-completeness audit (F10 (c)). The
per-antigen tier marker is the local shadow of the network-level completeness-pressure
— they are one property at two scales, not two findings.

**Tooling shape**: tier derivation by the self-reach check at ratification time;
audit surfaces description-tier antigens for extra self-scrutiny on their own
name/fingerprint/witness (which is exactly the F8 rename + F9 glossary-catch,
systematized). The completeness-pressure is a standing posture (comprehensive-vision
research-driven growth IS the maturation), not a single tool.

**Recognized**: via the F10 fundamentality-test ruling.

---

### Vaccine hesitancy / non-adoption (social-cultural friction)

**Biology**: vaccines work technically but adoption fails for social/
cultural reasons. Non-technical friction kills public health gains.

**Potential antigen instantiation**: friction-cost of adopting antigen — what
makes practitioners decline to adopt even when the technical case is sound.
Naturally surfaces from real adoption: maintenance burden, learning curve,
build-time cost, perceived complexity, "I have tests, I don't need this."

**What would trigger instantiation**: this is a perpetual concern, not a
single primitive. Adoption friction is tracked across the
project's lifetime; ergonomic improvements respond to it.

**Tooling shape**: not tooling. This is a *posture* — antigen ergonomics is
load-bearing for adoption. Already in scope.md ("low friction OOTB,
comprehensive when worked").

---

### Maternal immunity (antibody transfer mother → child)

**Biology**: antibodies pass from mother to child via placenta and breast
milk, providing temporary protection until the child's own immune system
develops. Inherited but time-limited.

**Potential antigen instantiation**: fork/derive relationships where a child
crate inherits the parent's antigens but eventually develops its own. The
inheritance is temporary; eventual divergence is expected.

This is structurally distinct from `#[descended_from]` (which is
function-level inheritance with re-justification). Maternal-immunity
primitive would be *crate-level* inheritance — child crate forks parent's
antigen set.

**What would trigger instantiation**: post-A3 cross-crate when forks are
real. Worth ADR territory: how do forks handle ancestral antigen
inheritance? Does the fork get the original's antigens automatically? With
what re-validation?

**Tooling shape**: `cargo antigen fork-from <crate>` or auto-detection from
git fork relationships.

---

### Allergy / overreaction

**Biology**: immune system responds too strongly to non-threats (peanuts,
pollen, cats). Different failure mode than autoimmunity (which targets self).

**Potential antigen instantiation**: false positives causing developer
fatigue → real threats ignored. Distinct from autoimmunity (over-flagging
legitimate code) — allergy is over-flagging *correctness-irrelevant
patterns*.

Example: a fingerprint that matches every `unwrap()` in test code. Tests use
`unwrap()` legitimately; flagging every test is allergy.

**What would trigger instantiation**: stdlib refinement when fingerprints
prove too aggressive. Distinct mitigation from autoimmunity-tolerance —
allergy needs *fingerprint refinement*, not per-site tolerance.

**Tooling shape**: fingerprint-precision metrics; A/B testing of fingerprint
variants; community feedback loop for stdlib antigens.

---

### Immunodeficiency (impaired immune function)

**Biology**: immune system not operational due to disease, drug, or genetic
defect. The body can't recognize or respond to pathogens.

**Potential antigen instantiation**: codebases where antigen declarations are
absent or stale; the immune system isn't operational. This is the *default*
state for codebases that haven't adopted antigen — they have no
failure-class memory beyond developer minds + git log.

**What would trigger instantiation**: this is mostly a framing primitive for
ecosystem-outreach. Adoption messaging: "your codebase currently has
immunodeficiency for failure-class memory; antigen provides the immune
system."

**Tooling shape**: not tooling. Posture and adoption pitch.

---

### Booster shots (re-exposure to maintain immunity)

**Biology**: periodic re-exposure to pathogen-derived antigen to maintain
antibody titer. Maintenance dose for declining immunity.

**Potential antigen instantiation**: ADR-016's `verified_at` + re-attestation
is essentially this. The primitive is shipped at the architectural level;
booster-tooling would be UX around it.

**What would trigger instantiation**: A3+ when temporal substrate is real.

**Tooling shape**: `cargo antigen audit --refresh-stale` for re-attestation
of decayed `verified_at` claims.

---

### Adjuvants (boost vaccine response)

**Biology**: substances added to vaccines that boost immune response.
Aluminum salts, lipid nanoparticles. Don't have direct effect on disease;
amplify the vaccine's effect.

**Potential antigen instantiation**: things that make antigen declarations
more effective without changing what they do:
- IDE integration (rust-analyzer plugin) — adoption adjuvant
- CLI output formatting (color, summary, action items) — attention adjuvant
- Antigen-stdlib documentation (worked examples, cookbook) — onboarding
  adjuvant
- ADR cross-references — context adjuvant

**What would trigger instantiation**: each adjuvant is its own development
track. Already partially recognized in A6 (IDE integration).

**Tooling shape**: each adjuvant has its own shape; framing useful for
prioritization (which adjuvant boosts adoption most per unit effort?).

---

### Quarantine / isolation (prevent transmission)

**Biology**: physically preventing pathogen transmission by separating
infected from uninfected.

**Potential antigen instantiation**: cargo features, cfg gates, module
privacy boundaries that prevent failure-class spread. Antigen could
recognize quarantine boundaries and report failure-class containment status.

**What would trigger instantiation**: when failure-classes prove to spread
within codebases via shared patterns; quarantine-tooling identifies the
spread paths.

**Tooling shape**: scan extension that walks module boundaries and reports
"failure-class X is contained to module Y" or "failure-class X has spread
across these N modules."

---

### Contact tracing (find who else was exposed) — multi-modal transmission

**Biology**: when an index case is identified, trace contacts across multiple
*transmission modes* — each pathogen has different transmission characteristics
that determine which contact relationships matter. Public health tooling
distinguishes direct contact, droplet, airborne, vector, vertical, fomite
transmission; each mode has different infectious-period bounds, different
intervention strategies, and different graph-types for the contact tracer
to walk.

**Potential antigen instantiation**: contact tracing is *not* limited to
`#[descended_from]` chains. The full code-relationship graph has multiple
"transmission modes" — different ways a "sick" code site can affect related
code without direct executed contact. Each transmission mode has a different
graph-type antigen tooling could traverse:

| Transmission mode | Code relationship | Antigen exposure shape |
|---|---|---|
| Direct contact (skin-to-skin) | Call graph (A calls B) | Function presenting antigen → caller is structurally exposed |
| Droplet transmission (proximity) | Module proximity (same file/mod) | Local invariant assumptions in nearby code |
| Airborne (long-range) | Trait implementation graph | `impl X for Y` exposes Y everywhere X is used |
| Vector transmission (mosquitoes) | Macro expansion graph | Macros carry the pattern across many call sites without direct contact |
| Vertical transmission (parent → child) | Descended_from inheritance | Currently operational (only graph antigen reasons over today) |
| Fomite transmission (shared objects) | Shared types / traits / generics | Container<T> as carrier; the type itself is the surface |
| Assumption-graph transmission | Downstream assumptions about behavior | Cross-reactivity at the contextual-assumption tier — see cross-reactivity entry above |

That's seven transmission modes corresponding to seven graph-types antigen
would eventually need to reason over. Rust-analyzer's existing infrastructure
provides much of the graph substrate (call graph, trait impl resolution, type
relations, macro expansion); antigen-tooling integration becomes "consume
rust-analyzer's graphs as substrate, walk them with antigen-aware queries."

**The architectural commitment this elevates**: antigen's recognition surface
is **graph-typed, not site-typed**. A fingerprint match isn't just "at site
X"; it's "at site X plus the contact graph rooted at X across all transmission
modes." Currently antigen reasons over one graph (descended_from); the
contact-graph framing says antigen will eventually consume multiple graph
types as substrate.

**The biological richness extends beyond just transmission modes**:

- **Reproductive number (R₀)** → how many code sites per ancestor pattern?
  Stdlib antigens with high R₀ are keystone-class (per §3 ecology cognate);
  low R₀ are niche.
- **Patient zero** → already encoded! The `references = ["GAP-BIT-EXACT-1"]`
  field IS patient-zero attribution. Every antigen knows its index case.
- **Quarantine** → PR-gating: when a PR introduces new descended_from chains
  (or call-graph contacts, or trait-impl surfaces) from an antigen-marked
  ancestor, gate the PR until immunity is declared OR tolerance is justified.
  CI-time public-health intervention.
- **Cluster identification** → families of types sharing an ancestor that
  presents an antigen. The `family` field on `#[antigen]` partially encodes
  this; contact-graph traversal operationalizes it.
- **Index case investigation** → the question contact tracing always asks:
  "where did this start, and what was the chain?" The references field traces
  forward from index case; descended_from traces forward from ancestor types.
  Multiple complementary trace directions.

**Cross-crate version post-A3 is genuinely powerful**:

> "A downstream crate's `Container<T>` was just declared as presenting X. `cargo antigen
> trace --cross-crate` shows: every project depending on that crate that uses
> Container<T> is structurally exposed."

That's CVE-style supply-chain propagation but for *failure-classes generally*,
not just security. The cybersecurity precedent (CVE in dep-graph affects every
dependent project) maps directly. See cross-domain-architectural-map.md §8
for the cybersecurity cognate.

**What would trigger instantiation**: post-A5 when antigen-stdlib accumulation
reaches the point where graph-effects of one declaration matter operationally.
Pre-A5, descended_from-only contact tracing covers the realistic case. Post-
A5, when A3 cross-crate scan + adoption pressure surface real cases of
"declaration of antigen X graph-effects across distant code," the additional
transmission modes become operationally valuable.

**Tooling shape (graduated by transmission mode)**:

- Phase 1 (descended_from only — current): `cargo antigen trace <type>` showing
  derivation graph + defense status across the graph
- Phase 2 (call graph + trait impl): rust-analyzer integration; trace command
  walks across multiple graph types
- Phase 3 (cross-crate, all modes): supply-chain-style propagation tracing
  via rustdoc JSON or static-emission (per cross-domain-map.md Appendix A
  for cybersecurity precedent)

See companion document [`contact-graph-and-recognition-tiers.md`](contact-graph-and-recognition-tiers.md)
for substantive expansion of the multi-modal transmission framework and its
relationship to the three-tier cross-reactivity framework above (authored A2 day-2,
commit `30d91bf`).

---

### Outbreak / epidemic / endemic / pandemic

**Biology**: spread patterns. *Outbreak* = local cluster. *Epidemic* =
sustained regional spread. *Endemic* = pervasive in a region. *Pandemic* =
widespread across regions.

**Potential antigen instantiation**: failure-class spread patterns:
- *Outbreak*: same failure-class in multiple files in one project
- *Epidemic*: same failure-class across multiple projects in a single
  ecosystem
- *Endemic*: failure-classes that are pervasive in certain language
  patterns (e.g., panic-in-Drop is endemic to Rust)
- *Pandemic*: cross-language failure-classes (e.g., off-by-one errors)

**What would trigger instantiation**: ecosystem-statistics tooling
(post-A5+) when antigen-stdlib propagation is measurable. Useful for stdlib
prioritization: which failure-classes are pandemic and need stdlib
representation first?

**Tooling shape**: ecosystem-survey tool; antigen-stdlib promotion criteria
based on observed spread patterns.

---

## Beyond classical immunology — virology, medicine, public health

### Virology

#### Mutation rates

**Biology**: pathogens evolve at characteristic rates. RNA viruses (flu,
COVID, HIV) mutate fast; DNA viruses mutate slower; bacteria evolve
intermediately.

**Potential antigen instantiation**: how fast failure-classes mutate into
new structural variants. Some failure-classes are stable across years
(panic-in-Drop is a slow mutator); others evolve rapidly (async-in-sync
patterns shift with each tokio release).

**What would trigger instantiation**: stdlib maintenance — slow-mutating
antigens stable for years; fast-mutating need quarterly refinement.

**Tooling shape**: antigen-stdlib release cadence guidance; mutation-rate
metric per stdlib antigen.

---

#### Quasispecies (population of related-but-distinct viral variants)

**Biology**: an RNA virus infection isn't one variant; it's a cloud of
closely-related variants (quasispecies). Selection pressure shifts the
population.

**Potential antigen instantiation**: failure-class *families* with shared
structural shape but distinct fingerprints. Currently `family` field on
`#[antigen]` is a single string; quasispecies framing suggests families
might benefit from explicit variant relationships and population-level
metrics.

**What would trigger instantiation**: stdlib accumulation reaching the
point where multiple antigens share a family but differ in fingerprint
specifics. Worth ADR territory.

**Tooling shape**: family-aware audit reporting; "you have these N variants
of frame-translation; here's their relationships."

---

#### Antigenic drift vs shift

**Biology**: *drift* is gradual antigen change (annual flu vaccine updates).
*Shift* is sudden major change requiring entirely new vaccine (pandemic
strains).

**Potential antigen instantiation**: gradual fingerprint refinement (drift)
vs major architectural change requiring fingerprint replacement (shift).
Drift is normal stdlib maintenance; shift is a versioning event.

**What would trigger instantiation**: ADR-010 amendment substrate when
fingerprint versioning hits a major-vs-minor distinction in practice.

**Tooling shape**: fingerprint semver discipline; `cargo antigen migrate`
for shift-class changes.

---

#### Reservoir hosts

**Biology**: where pathogens persist between outbreaks (e.g., fruit bats
for ebola). Pathogen lives quietly in reservoir; outbreak happens when
spillover occurs.

**Potential antigen instantiation**: where failure-classes live in code
between manifestations. Likely the *passive surface* via fingerprint scan —
code that matches the fingerprint but isn't currently failing because
preconditions haven't aligned.

**What would trigger instantiation**: this is mostly already operational
via passive scan. Worth marking as biological framing for the closure
narrative.

---

### Medicine / clinical

#### Healing / scar tissue

**Biology**: tissue repair after injury leaves scars — different tissue
than original, often with reduced function but providing structural
integrity.

**Potential antigen instantiation**: bug fixes leave structural traces —
the antigen declaration IS the scar that prevents recurrence. A codebase
with many antigens has many scars; each represents a specific lesson
learned.

**Worth marking**: this framing supports adoption-pitch material. "Your
codebase's scars from past bugs — stored as antigen declarations —
collectively form your immune system."

---

#### Triage

**Biology**: emergency-medicine prioritization when resources are limited.
Treat the most urgent first; defer the less urgent.

**Potential antigen instantiation**: when scan finds N issues, which to
address first? Triage primitive needed when scan output exceeds developer
capacity to address all.

**What would trigger instantiation**: when antigen-stdlib + project-specific
antigens combine to produce scan output of 50+ items.

**Tooling shape**: severity classification; `cargo antigen triage`
subcommand; integration with issue trackers.

---

#### Differential diagnosis

**Biology**: distinguishing similar-presenting conditions. Multiple diseases
might present with the same symptoms; physicians narrow down via tests.

**Potential antigen instantiation**: disambiguating which antigen a
fingerprint match actually represents when multiple antigens match the same
code. The scan output might say "this site matches 3 antigens; here's how
to determine which one is actually present."

**What would trigger instantiation**: post-stdlib when fingerprint overlap
is real. Related to cross-reactivity primitive above.

**Tooling shape**: scan extension; interactive disambiguation tool.

---

#### Iatrogenic harm (caused by medical treatment)

**Biology**: harm caused by medical intervention itself. The treatment
becomes the cause of new problems.

**Potential antigen instantiation**: harm caused by antigen tooling itself.
Examples:
- Auto-applied vaccinations that introduce new patterns the team didn't
  intend
- Witness templates that compile but don't actually verify
- Fingerprint matches that distract developers from real bugs

**What would trigger instantiation**: real adoption surfacing tool-induced
harm. Mitigation requires awareness; primitive is the *concept* of
iatrogenic harm as a class of antigen failure.

**Tooling shape**: not tooling. Discipline + acknowledgment in stdlib
documentation.

---

#### Comorbidity

**Biology**: multiple conditions interacting. Diabetes + heart disease +
hypertension produce different prognosis than any one alone.

**Potential antigen instantiation**: multiple antigens manifesting together;
their interactions. Some combinations are worse than the sum of parts (e.g.,
panicking-in-Drop + async-in-sync = particularly nasty deadlock potential).

**What would trigger instantiation**: stdlib accumulation surfacing
interaction patterns.

**Tooling shape**: comorbidity report in audit output; "this site presents
antigens X + Y; their combination is more severe than either alone."

---

#### Dose-response curves

**Biology**: relationship between intervention magnitude and effect. Linear,
sigmoid, or non-monotonic depending on biology.

**Potential antigen instantiation**: relationship between fingerprint
precision/recall trade-offs and false-positive/false-negative rates. The
"right" fingerprint precision is a curve; project preferences shift along it.

**What would trigger instantiation**: stdlib-quality framework needs
empirical curves to set defaults.

**Tooling shape**: per-antigen precision/recall metrics published with
stdlib; project-level configuration to adjust the operating point.

---

### Public health

#### Surveillance (tracking disease spread)

**Biology**: monitoring populations for disease emergence and spread. CDC,
WHO; sentinel sites; case reporting.

**Potential antigen instantiation**: ecosystem-wide adoption metrics —
which projects use antigen, which antigens are most-adopted, which
fingerprints fire most-often. Public health for the Rust ecosystem.

**What would trigger instantiation**: post-v0.1.0 when adoption is
measurable. Possibly as a separate community project (antigen-stats?).

**Tooling shape**: opt-in telemetry; community-curated dashboard.

---

#### Vulnerability indicators

**Biology**: risk factors for disease (age, immunocompromise, chronic
conditions). Identify high-risk populations for prioritized intervention.

**Potential antigen instantiation**: structural patterns that *predict*
failure-class susceptibility before manifestation — code that doesn't yet
exhibit a failure-class but has structural risk factors. Distinct from
fingerprint match (which is direct evidence of pattern); vulnerability
indicators are *risk factors*.

Example: a function with many `unwrap()` calls + complex match arms is
*structurally vulnerable* to panic-in-Drop even if not currently in Drop;
when it gets used in Drop later, the vulnerability manifests.

**What would trigger instantiation**: stdlib refinement; possibly its own
antigen *type* (vulnerability-class antigen, distinct from failure-class
antigen).

**Tooling shape**: scan extension; scoring system for risk indicators.

---

#### Outbreak response protocols

**Biology**: standardized procedures for responding to disease outbreaks.
Identify, isolate, treat, trace contacts, communicate.

**Potential antigen instantiation**: when a new failure-class is named in
stdlib, what happens to the ecosystem? How does antigen-stdlib's release
process work? What's the discovery → declaration → propagation pipeline?

**What would trigger instantiation**: A5+ when stdlib has a release cadence
and adoption flywheel.

**Tooling shape**: stdlib contribution process; community pattern for
recognizing and naming new failure-classes.

---

## Where biology goes silent (honest boundaries)

With biology-as-instrument showing structured silence at the FormalProof
tier: biology has dense vocabulary up through where
biological immunity actually reaches its limits, then goes silent. The
silence is informative.

Boundaries where biology has no analog (or only weak analogs):

- **Compile-time formal proofs** — biological immunity can fail by
  construction; engineered formal proofs cannot. Biology has no
  "construction-can't-fail" primitive.
- **Static-vs-dynamic dispatch** — purely a programming-language concern;
  biology's recognition is all dynamic.
- **Generic type parameters** — biology's MHC presents specific peptides;
  there's no "generic antigen over a type parameter."
- **Macro hygiene** — biology has no analog to programming-language macro
  hygiene boundaries.
- **Build-time vs runtime** — biology is all runtime; the concept of
  build-time recognition has no biological cognate.
- **Reflexive single-unit self-binding** — biology does recognition-of-
  recognition at the **network** level (anti-idiotype antibodies bind *other*
  antibodies' recognition-regions; see Idiotype network above), but NOT as a
  single antibody binding *itself*. This silence is load-bearing: it located the
  F10 self-reach test at the taxonomy/network tier rather than the single-
  declaration tier ("does a class reach its own declaration" = "does it appear in
  the network that includes its declaration"). The metaphor's partial silence
  named which tier the test lives at — dense where biology answers (network self-
  reach), precisely silent where it doesn't (reflexive self-binding), and the
  silence is the finding. (dx-dogfood, F10.)

These boundaries are themselves load-bearing. Don't force-fit a biology
analog where biology doesn't have one — the silence is honesty.

---

## What this map is for

Each primitive listed here is **forward substrate**, not commitment.
Per ADR-006: when a real adoption pressure surfaces a need, the team
recognizes the pattern from this map rather than re-deriving it. The map
exists so the team has a structured prediction set to check against
emerging needs.

This document is V0. Future deepening:

- A future pass could generate biology-grounded predictions about which
  primitives land first based on adoption pressure + biological prior.
- A future pass could find prior-art partial-instantiations (existing tooling that
  already does some of this; ergonomic patterns; friction-cost data).
- A future pass could integrate the map into manuscript trajectory + adoption
  pitch material.
- A future pass could extend the cross-domain mapping
  beyond biology — virology, medicine, public health, possibly cognitive
  science (memory consolidation, pattern recognition, schema theory),
  evolutionary biology (selection pressure as fingerprint refinement),
  ecology (population-level immune dynamics).

V1 will deepen each primitive with concrete instantiation paths, prior-art
partial-instantiations, and recognition triggers. V1 should also expand
beyond classical immunology into the broader biology + medicine territory
this V0 only began to sketch.

---

## Closing posture

Biology has been iterating on recognition-with-memory-and-inheritance for
~500 million years. Antigen is the first structurally-adoptable
instantiation in a Rust ecosystem; the architectural genome is biology's;
the substrate is Rust's. Biology has answers to questions antigen will face
when adoption scales beyond v0.1.0. This map names them.

The discipline (per ADR-006): no speculative instantiation. The substrate
recognizes; the project does not design ahead of recognition. This map is
recognition substrate — when adoption surfaces a real instance, the team
finds the answer here rather than re-deriving it.

Biology operates as an engineering
instrument. The instrument has dense vocabulary where it has answers and
honest silence where it doesn't. This map preserves both.
