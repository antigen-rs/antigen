# Charter — the Registry / Herd Immunity (POPULATION: beyond one codebase)

*The cluster where antigen's memory becomes shared — failure-class immunity spreading across codebases like
herd immunity across a population. Gated on self-tolerance (the learning core) as its named precondition.*

**Tier:** deep-future (chartered expedition). **Precondition: self-tolerance must ship first.**

---

## Organ-identity (what this IS)

Herd immunity. One codebase's hard-won failure-class memory becomes the whole ecosystem's — a
crates.io-for-antigens where a fingerprint matured in one project vaccinates the next on `cargo add`. This
is where the "trained cross-codebase model" lives: not a hosted moonshot, but a **gradient field** —
content-addressed, no central server (a stored directory would itself be the `RoutingTableStale`
failure-class antigen names). The expansionist's unification: gradient-routing collapses
registry + phylogeny + forgetting into ONE distance-metric + ONE evaporation rule.

## The dreams in this cluster

- **`dream/shared-antigen-registry-herd-immunity`** — the registry; vaccination/herd. Where cross-codebase
  immunity lives. **Self-tolerance is its named precondition** (see deps).
- **`dream/phylogeny-population-genetics`** — conserved-core vs dialect; the registry made NAVIGABLE;
  antigenic-cartography for bugs (a literal 2D failure-map, the dreamer's frontier). Near-question: which
  families fire across the MOST diverse codebases = stdlib priority (feeds back into ADR-038 ranking).
- **`dream/dependency-boundary-inherited-immunity`** — the failure-classes a crate presents become YOURS
  on `cargo add`; the registry from the consumer side (the `#[descended_from]` inheritance, ADR-018,
  extended across the dependency boundary).
- **`dream/microbiome-tolerated-symbiont`** — tolerated load-bearing NON-SELF; forces the self-question
  **TERNARY: self / symbiont / pathogen** (the idiomatic-but-weird dependency that's neither ours nor a
  threat — the `#[antigen_tolerance]` corpus is a "self + symbiont" corpus, not just "self").

## Dependencies (what unblocks this)

- **SELF-TOLERANCE (the learning-core charter) is the NAMED precondition.** You cannot share fingerprints
  across codebases until you can distinguish self from non-self from symbiont — a registry without negative
  selection spreads false positives ecosystem-wide (herd *auto*immunity). This charter MUST follow the
  learning core.
- **The gradient-routing substrate** (the ROUTE charter's content-addressed/Kademlia mechanism) — the
  registry is its aggregation; sharing the substrate de-risks the registry from a hosted-service moonshot.

## Could-combine-with

- The **ROUTE charter** (gradient-routing) — the registry IS gradient-routing aggregated; phylogeny is the
  same substrate made navigable. The expansionist's collapse means these may be ONE expedition, not two.
- **`families/herd-immunity-cross-codebase-vaccination`** (a build-now-adjacent family) — the in-codebase
  seed of cross-codebase vaccination; the registry is its population-scale form.

## Buildability / effort scoping

- **HARD — a real new subsystem (a distributed substrate).** This is the deepest-future cluster: it needs a
  content-addressed store + a distance-metric + an evaporation rule. The expansionist's gradient-field
  framing keeps it from being a hosted server (good — a server would be `RoutingTableStale`), but it is
  still a genuine networked component.
- **The consumer-side slice is CHEAPER** — `dependency-boundary-inherited-immunity` extends the shipped
  `#[descended_from]` propagation (ADR-018) across the `cargo add` boundary; a real but bounded extension of
  existing machinery, buildable before the full registry.
- **CHEAPEST consumer-side slice — the BUNDLED-STDLIB-CATALOG scan mode (v0.4, beta.2 Bushwhackers
  finding).** Substrate-verified during the beta.2 dogfood sweep: `synthesis_pass` builds its
  fingerprint catalog from antigen DECLARATIONS *in the scanned tree* (no built-in catalog), so scanning a
  *consumer* crate standalone fires **zero** stdlib members — antigen self-catches only because antigen's
  own tree CONTAINS `stdlib/*.rs`. Cross-crate adoption today therefore requires the consumer to import +
  `#[presents]` the stdlib members (the active model — tambear declares 2 seed antigens). The cheap missing
  mechanism: a **bundled-stdlib-catalog scan mode** — `cargo antigen scan` loads `antigen::stdlib::*`
  fingerprints as a *built-in catalog* and matches them against any scanned tree, with no per-crate
  declaration. This is "scan MY crate against antigen's shipped stdlib," the first real product-grade
  adoption surface — strictly cheaper than the inheritance-boundary slice (no `#[descended_from]` graph; a
  bundled `&[(String, Fingerprint)]` passed into the existing `synthesis_pass`). The honest masterclass
  framing it resolves: *"antigen catches a class in ITSELF (proven, beta.2 — UnboundedDeser true-positive in
  antigen's own `multi_crate.rs`); catching it in YOUR crate needs adoption-presents (now) OR this
  bundled-catalog mode (v0.4)."* NOT gated on self-tolerance (it ships antigen's OWN curated stdlib, not a
  cross-codebase-shared catalog — the negative-selection precondition applies to the *shared registry*, not
  to shipping antigen's vetted families).
- **Net:** sequence — **bundled-stdlib-catalog mode (v0.4, cheapest)** → `dependency-boundary` (extends
  shipped inheritance) → the gradient substrate → the full registry/phylogeny. The first hop ships antigen's
  own stdlib (no self-tolerance gate); the registry hops are gated on self-tolerance throughout.

## Invitation to deepen

Antigenic cartography as a literal 2D failure-map (the dreamer's frontier): plot failure-classes by which
codebases they bind, and the map's topology tells you the conserved core (ship in stdlib) vs the dialect
(adopter-extension). The self/symbiont/pathogen ternary is biology's most sophisticated discrimination —
the gut spends its *most* tolerance-machinery on commensals — and the dev cognate (the load-bearing weird
dependency) is exactly where a naive "all non-self is pathogen" antigen would be ecosystem-autoimmune.
