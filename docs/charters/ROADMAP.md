# Antigen — the disposition roadmap

*The organized, prioritized, dependency-ordered disposition of the entire beta.2 dream output.
Produced by the Outfitters (Boat 2 of the `v03-beta2-growth-masterclass` voyage) so that **no campsite
leaves unsorted**: every one of the ~85 campsites is either **build-now** (an ADR, this wave) or assigned
a **named, dependency-placed future home** (a charter, this folder). A future Cartographers/dream wave
launches FROM a charter, not by spelunking a gitignored expedition tree.*

**Why this lives in `docs/charters/` (tracked), not in the expedition (gitignored):** the expedition's
`dream/*` campsites are voyage-internal substrate — they will be buried when the voyage seals. This roadmap
+ the charters are the *durable carrier*: the cross-voyage comparison-surface the next dream wave reads to
see what was imagined, what shipped, and what's waiting with a designed home. The campsite notes are the
territory; this is the index that survives.

**Status:** living. The Outfitters drew the disposition; the harbor-masters (Tekgy + main) ratify it; each
tier promotes as the prior one ships. **Mutable — this is an ever-shifting priority order, not a static
list.** A charter jumps tiers when reality teaches (a v0.4 charter becomes v0.3 the moment a new dream makes
it cheap or necessary). This store doubles as the project's **future-expeditions to-do list**.

---

## How this store is consumed (the dual protocol — read this first)

This charter store is the project's **persistent cross-voyage backlog**, consumed by two kinds of crew, at
two different times. Both protocols depend on each charter having a clear, evocative **scope-identity** and
a **"could-combine-with"** edge — those are what make both reads cheap.

1. **DREAMERS (an expansion / Cartographers wave) consume-to-EXPAND — *before* logging new campsites.**
   Before a dreamer logs a new dream, they check this store: if the dream is already a charter here, they
   **deepen that charter** (add to its scope, sharpen its identity, extend its invitation) rather than
   re-dream it. Only genuinely-novel dreams become new campsites. This is the **primary-vs-secondary immune
   response**: a novel dream → a new campsite (primary); an already-chartered dream → deepen the charter
   (secondary). The store is the dreamers' **memory** — so dreams *mature across voyages* instead of
   restarting each time. (This is why every charter below is written dreamer-legible + expandable, with an
   open invitation-to-deepen, not as a closed locked spec.)
2. **CONVERGENCE crews (an Outfitters / lock wave) consume-to-RE-CONVERGE — compare new + deepened dreams
   against this store.** A convergence crew lays the new wave's campsites beside these charters and looks
   for **pairings / merges / priority-flips**: two dreams from two voyages may turn out to be one organ
   (merge into one charter); a new dream may make a deferred charter suddenly cheap (flip its tier); a
   "could-combine-with" edge may become a real combination. The charter is not a write-once deferral — it
   is a **comparison surface**.

So a charter is alive: it gets deepened (dreamers), re-tiered (reality), and merged/paired (convergence).
The clear scope-identity + the could-combine-with edges are the load-bearing structure that keeps both
consumers cheap — they are what prevent the dreamers re-inventing the same organs voyage after voyage.

---

## The organizing shape — one organism, the control loop, escalating scales

The dream cluster is not a feature-list. It is **one immune system at escalating scales**, organized along
the **SENSE → COMPARE/LEARN → ROUTE → ACT → FEEDBACK → SIZE control loop** that ADR-037 ratified as
antigen's regulator self-model — plus three outer rings (POPULATION, PREVENTION, REFLEXIVE) that zoom past
one codebase. The families/grammar/sensors/specs are the *machinery* that builds the loop's near rings.

The dependency backbone (what unblocks what):

```
  [2 near-free seams, BUILD-NOW]          [the keystone, charter]
  emit-typed-events (ADR-039) ───┐
  SCRAM out-of-band (ADR-036) ───┼──▶ affinity-maturation-engine ──co-requisite──▶ self-tolerance
                                 │     (the ONLY organ that GENERATES              (the governor; ship
  the dial/dread/Finding schema  │      immune knowledge)                          together or ship
  (ADR-039/041) = the event-bus  │            │                                    autoimmunity)
  every organ rides ─────────────┘            ▼                                          │
                                        cytokine-signaling                               ▼
                                        (= the emit-seam, already build-now)    shared-antigen-registry
                                                                                (herd immunity; needs
                                                                                 self-tolerance as a
                                                                                 named precondition)
```

**The rule the seams encode:** the two beta.2 seams are *near-free now, ruinous to retrofit* — they keep
the entire future learning-loop buildable for ~zero cost. Everything downstream of them charters, but
because the seams shipped, each future organ subscribes to an existing typed signal instead of re-plumbing.

---

## The tiers

| Tier | Meaning | Gate to promote |
|---|---|---|
| **BUILD-NOW** | locked as an ADR + a build-now slice this wave (Bushwhackers, Boat 3) | already ratified |
| **beta.2** | ships in `0.3.0-beta.2` (the near slices of the build-now ADRs) | the ADR build lands |
| **v0.3** | the next minor — recognition-heavy, cheap, depends only on beta.2 | beta.2 stable |
| **v0.4** | needs `ra_ap_syntax` / rust-analyzer-grade semantic analysis (MSRV-1.95-unblocked) | the syntactic slice ships + demand confirmed |
| **deep-future** | needs a real new subsystem (a store, a network, a runtime arm) — a chartered expedition | its named precondition ships |
| **research-undefined** | the substrate or the identity is genuinely open; needs a study, not a build | a harbor-master identity/scope ruling |

---

## TIER: BUILD-NOW — the 7 ratified ADRs (036–042) + their slices

These are locked (this voyage). The Bushwhackers build them; build order **decomposition FIRST, sweeps
LAST** (see `docs/expedition/beta2-boat-briefings.md` §Boat 3 + the file-by-file extraction order).

| ADR | What it locks | Campsites it dispositions |
|---|---|---|
| **036** | scan/audit decomposition + out-of-band SCRAM host (purity = single-conductor) | `infra/decompose-audit-scan-orchestration` |
| **037** | antigen-is-a-closed-loop-regulator (the control-loop master-frame, ORGANIZING) | `synthesis/six-shapes-are-one-control-loop`, `synthesis/lifecycle-and-control-loop-converge`, `synthesis/loop-a-reflexive-face-completeness`, `process/requisite-variety-grounds-yawni` |
| **038** | stdlib taxonomy grid (3 divergence-genera + super-family parents) | the **21 `families/*`** (as members under the frame — see below) + `unifier/staleness-check-outlives-truth` |
| **039** | confidence dial + permissive build-gate + emit-seam (the Finding schema) | `spec/build-gate-and-confidence-dial`, `spec/graduation-as-evidence-updating` |
| **040** | grammar increment (`body_calls` + the syntactic absence family) | `grammar/frontier-dimension-extensions`, `grammar/absence-detection-missing-self`, `grammar/gate-collapse-universal-shadow-method`, `grammar/safe-sibling-bypassed-api-pair`, `sensors/absence-as-signal-nk-missing-self` (the sensor-half of the absence dimension — its *syntactic* slice rides ADR-040's absence family; the semantic slice waits at v0.4) |
| **041** | marked-unknown plane (`#[aura]`/`#[dread]`/`#[red_flag]`) | `primitive/marked-unknown-dread-aura` |
| **042** | usage-discipline (3 disciplines) + `#[autoimmune]` naming reconciliation | `spec/usage-discipline-liberal-vs-regulatory`, `dogfood/autoimmune-as-regulator` (the audit-mode screen) |

**The 21 families (ADR-038 members — build-now slices per the worth-map, NOT each a new ADR):** the
silent-flagship members ship in beta.2; the rest sort under the three genera + the super-family parents
(Information-Loss / Staleness / Drop-Lifecycle) and ship as members as the grammar they need lands.
**Worth-multiplier (naturalist):** each admitted class ships *with the specimen that admitted it* — the
`constructable`-tier demo IS the masterclass pathology specimen IS the dogfood (one build, three payoffs).
The full per-family disposition (all 21):

- **beta.2 silent-flagship members (build-now, the worth-map leaders):**
  `families/unsafe-soundness-boundary`, `families/deserialization-trust-boundary`, `families/crypto-misuse`,
  `families/drop-and-panic-discipline`, `families/resource-lifecycle-leak`, `families/numeric-truncation-overflow`,
  `families/error-info-loss`, `families/config-provenance-loss`, `families/panic-on-index`.
- **beta.2 / v0.3 members (sort under ADR-038 genera + parents; ship as their grammar lands —
  `body_calls`/absence are build-now ADR-040, so most are beta.2-eligible):**
  `families/async-soundness-family`, `families/semver-contract-violation`, `families/feature-unification-hazards`,
  `families/time-and-ordering-hazards`, `families/trait-coherence-consistency`,
  `families/iterator-laziness-and-side-effect-ordering`, `families/api-sequencing-and-unconsumed-builder`,
  `families/herd-immunity-cross-codebase-vaccination` (the in-codebase seed; the population-scale form is the
  registry charter), `families/stdlib-expansion-build` (the meta-campsite = the build-now family-shipping
  process itself, ADR-038).
- **Re-filed to LOOP-A (regulator-machinery, NOT disturbances — chartered, not stdlib):**
  `families/gradient-routing-chemokine-recruitment` (the ROUTE primitive → `charter-route-arm.md`) and
  `families/cascade-becomes-the-problem-sepsis-anaphylaxis` (the FEEDBACK governor → `charter-feedback-homeostasis.md`).
  `families/setpoint-corruption-goodhart-autoimmune` is the canonical BOTH case (disturbance =
  `FingerprintGamedNotDefended`, a stdlib family; regulator-fix = the `autoimmune-check` screen, ADR-042).

**Candidate primitive — `#[justified_bypass(reason=…)]` (disposition-placed, RULED into the pass by the
captain, NOT into the finish-line ADR batch).** The declared-disable-with-rationale **dual of the shipped
`#[antigen_tolerance]`** (tolerance says "this looks like failure-class X but it's intentional";
justified-bypass says "I'm switching off guard Y here, and here's why"). **Cluster: the COMPARE-stage /
comparator-divergence genus** (ADR-038) — it is the *deliberate-disable* branch of gate-collapse (a guard
present but switched off on purpose), the dual of the absent-guard case. **Scope: cheap** — a declaration
marker macro (same shape as the shipped `#[antigen_tolerance]`/`#[anergy]`, with a required `reason=`
rationale per the rationale-as-required-field discipline, ADR-005 Amd2). **Tier: beta.2-cheap-and-central
→ a fast ADR-043 OR a deliberate-disable MEMBER under ADR-038's comparator-divergence genus** (the choice
is made in this disposition pass, not rushed into the 036–042 batch). Discriminator from
`SafeSiblingBypassed` (which just needs the safe-sibling swap, no rationale): justified-bypass is the
*deliberate-disable-needs-a-why* case; the contract-doc-sensor witness (documented rationale) is its
witness. **Disposition verdict: PLACED as a beta.2-or-fast-followup candidate, homed in comparator-divergence;
the ADR-043-vs-ADR-038-member call is a small follow-up, not a blocker on the 036–042 commit batch.**

**The dogfood + specimen + research sweep (beta.2, codebase-wide, LAST):**
`dogfood/comprehensive-front-line-coverage`, `specimen/pathology-garden-code-silent-defects` (behind
`cfg`/example biosafety glass — rustc-enforced un-callability), `specimen/correction-memory-demonstration`,
`specimen/adr-drift-ratify-amend-reconcile`, `research/multimodal-failure-class-sweep`,
`process/roadmap-refresh-and-release-shape`.

---

## TIER: v0.3 — recognition-heavy, cheap, depends only on beta.2

- **`dream/metaphor-as-queryable-substrate-gaps`** (`cargo antigen gaps`) — best worth-to-cost; mostly
  recognition (the taxonomy grid ADR-038 + the loop ADR-037 make completeness a finite checklist —
  function × phase × scale × remedy-parent). Near-slice: the static immune-map (a masterclass showpiece).
  No separate charter — it's a v0.3 build-now CLI surface over the shipped taxonomy.
- **`sensors/substrate-reach-beyond-ast`** — the syntactic slices that don't need rust-analyzer
  (attribute/derive/trait-impl-path reads); pairs with ADR-040's leaf-matchers.
- **`sensors/absence-as-signal-nk-missing-self`** — the NK-missing-self sensor; the *syntactic* absence
  slice is build-now (ADR-040's absence family rides the shipped `check_not_placement` anchor rule); the
  *semantic* absence (X-impls-nowhere-in-the-resolved-program) waits at v0.4. Listed here as the
  recognition-tier seed of the absence dimension.
- **`grammar` G4 per-type correlation (syntactic slice)** — the per-type index machine (ADR-040 carves it
  to its own sibling ADR; the *syntactic* correlation is v0.3-eligible, pairs with the ADR-036 modular
  scanner).

---

## TIER: v0.4 — needs `ra_ap_syntax` (rust-analyzer-grade semantic analysis)

The MSRV-1.95 raise unblocks this. The confidence dial bridges the depths honestly: the syntactic slice
ships at the `heuristic`/`suspected` tier in beta.2; the semantic slice graduates it to `constructable`/`named`.

- **semantic grammar** — resolved-type identity (`LossyNumericCast` resolved-width), resolved-field-type
  (G4 semantic), body-liveness / control-flow (`LockHeldAcrossAwait` — typed-binding live across a
  suspension point). The families that need these wait here.
- **`sensors/coordination-transcript-substrate`** *(also feeds the SENSE/runtime-afferent charter)* — read
  camp's own `activity.jsonl`; the near-slice (detect an unratified multi-agent decision = antigen sensing
  its own agentic-coordination failure, the most thesis-true dogfood) is v0.3-cheap; the full
  reasoning-sensor (`dream/conversation-reasoning-sensor`) lives in `charter-runtime-afferent.md`.

---

## TIER: deep-future — needs a real new subsystem (a chartered expedition each)

Each of these has a **named charter file** in this folder (its scope, deps, organ-identity, and the
precondition that promotes it). They root at the keystone (maturation) + the governor (self-tolerance).

- **`charter-learning-core.md`** — the COMPARE/LEARN adaptive core: `affinity-maturation-engine` (the
  keystone — the only organ that generates immune knowledge) + `self-tolerance-negative-selection-engine`
  (co-requisite governor) + `intent-substrate-spec-alignment` (the AIRE-analog "self" = captured intent) +
  `dream/negative-space-the-classes-nobody-names`. **Precondition: the ADR-039 emit-seam (shipped) + a
  PROPOSE-slice demo** (the build-now falsification gate: it must produce one real draft fingerprint on our
  own code).
- **`charter-runtime-afferent.md`** — the SENSE/runtime arm: `runtime-sensor-deployed-organism`
  (identity-SETTLED-yes, Tekgy), `sensors/afferent-drainage-runtime-feedback-loop`,
  `sensors/tissue-resident-memory-runtime-guard`, `sensors/one-signal-four-scales-titer`,
  `sensors/efferent-arm-matured-response-export`, `dream/environment-threat-landscape-sensor`,
  `dream/human-attention-inspection-substrate`, `dream/conversation-reasoning-sensor`. **Bounding principle:
  SAMPLING ≠ ACTING** (antigen drains prod, the dev decides effector action). Precondition: the dial/Finding
  schema (shipped) leaves a `runtime-resident` tier slot; the `(tell, loc, ts, sev)` inward interface is the
  cheap stub.
- **`charter-registry-herd.md`** — the POPULATION arm: `shared-antigen-registry-herd-immunity`,
  `phylogeny-population-genetics`, `dependency-boundary-inherited-immunity`, `microbiome-tolerated-symbiont`
  (the self/symbiont/pathogen ternary). **Precondition: self-tolerance (the learning-core charter) — the
  registry's named precondition; a stored directory would itself be `RoutingTableStale`, so the
  gradient-routing/content-addressed substrate collapses registry+phylogeny+forgetting into one
  distance-metric + evaporation rule.**
- **`charter-effector-arm.md`** — the ACT arm: `repair-tier` (slow effector; near-slice = SUGGEST-floor,
  emit diff, human ratifies), `neutralization-containment-effector` (fast effector; contain blast-radius
  without fixing). Two-speed effector. **Sub-clause-F gates AUTO-APPLY.**
- **`charter-route-arm.md`** — the ROUTE arm: `gradient-routing-chemokine-recruitment` (the regulator
  primitive, re-filed from families), `convergent-dread-sentinel-network` (N dread-marks auto-escalate —
  **design the HOOK into beta.2's dread/aura now, don't paint dread solitary**),
  `epidemiology-contact-tracing-outbreak-map`, `reviewer-side-immune-lens` (`cargo antigen review <diff>`).
- **`charter-feedback-homeostasis.md`** — the FEEDBACK arm: `cytokine-signaling-network` (the event-bus =
  the ADR-039 emit-seam, so its *substrate* is build-now; the inter-organ propagation is the charter),
  `cascade-becomes-the-problem-sepsis-anaphylaxis` (the SCRAM governor — host is build-now ADR-036, the
  governor logic charters), `forgetting-curve-memory-curation`, `tissue-locality-immune-privilege`.
- **`charter-size-adapt.md`** — the SIZE/ADAPT arm: `adversarial-evasion-red-queen` (the arms race;
  maturation pointed at evasion), `circadian-process-rhythm` (thinnest — maybe a recurrence refinement).
- **`charter-prevention-transmission.md`** — `teaching-instrument-onboarding-curriculum` +
  `immune-memory-as-onboarding` (converged — the marked codebase IS a curriculum), `felt-immune-system-ambient-sense`
  (the proprioceptive mode — beta.2 hint: store dread/aura magnitude as a continuous felt-WEIGHT now).
- **`charter-reflexive-platform.md`** — the outermost loops: `antigen-as-platform-immune-infrastructure`
  (the LSP-of-failure-class-memory; **keep the core callable now, treat the Finding schema as the
  wire-format** — already done via ADR-039 schema_version), `evolve-immune-system-of-the-immune-system`
  (`#[evolve]`; Jerne's idiotypic network).

---

## TIER: research-undefined — substrate or identity genuinely open

- **`charter-beyond-code.md`** — `beyond-code-any-generated-artifact` +
  `dream/document-immune-system-sidecar-markdown` + `dream/nearest-second-organism-data-and-research-code`.
  **IDENTITY-FORK (bigger than runtime; Tekgy ruled YES, charter-not-beta.2).** Honest gate: only where the
  domain has an EXACT substrate (sidecar-to-markdown with synced line-refs, ADR-in-markdown first); else
  it's RAG, which the vision rejects. This is a *designed* future expedition, not a graveyard — but its
  scope needs a study before a build.

---

## The invariant, checked

Every one of the ~85 campsites appears above exactly once: 1 infra + 4 synthesis/process (ADR-037) + 21
families + 1 unifier (ADR-038) + 2 spec + 1 dogfood (ADR-039/042) + 4 grammar (ADR-040) + 1 primitive
(ADR-041) + 2 spec/dogfood (ADR-042) + the v0.3/v0.4 tiers + the 9 deep-future charters covering the 31
dreams + 7 sensors + the research-undefined charter. **No campsite leaves unsorted.** Each deferred cluster
has a named charter home with a promotion-precondition — the harbor-masters' promise kept: *a fully-designed
future expedition, not a folder it rots in.*

*See the individual `charter-*.md` files in this folder for each deferred cluster's full scope, dreams,
dependencies, organ-identity, and invitation-to-deepen.*
