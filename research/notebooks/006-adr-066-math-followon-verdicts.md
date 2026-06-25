# ADR-066 §4 math-research follow-on — guard-applied verdicts

> **Research notebook, not canonical.** This is the output of the `math-feeder-066` voyage opened by ADR-066's
> deferral of §4 (the standing's math). It applies ADR-066's **guard** ("enforce a process's math only where the
> analog is faithful; a math-demanded-but-absent term is the falsifier") term-by-term to the four named
> candidates, and feeds the **successor ADR** (the §4 math-development / stroma-builder ADR). Captured
> 2026-06-24 from the math-feeder's report (its own synthesis lives in its garden;
> `v0.6.1-self-non-self` has no `team-config.json`, so this repo notebook is the durable substrate home).
>
> Source-fidelity note: the Eigen-1971 and Anderson-May-1991 originals were not machine-fetchable; those
> attributions are via authoritative secondary sources (Wilke 2005, Saakian-Hu 2006, Page-Nowak 2002, Newman
> 2002 — read verbatim, carry the load-bearing equations). All citations resolved against Crossref/PubMed.

## Headline — the four candidates are BUILD-SPECS, not transfers

The math-feeder read the **actual stroma** and found it supplies almost none of the substrate these maths
require as real data:

- **No graph-with-edges** — only a sparse `LineageEdge` antigen-inheritance DAG; no dependency/call/import
  graph, no degree distribution, no giant component.
- **No "loudness" field** — `grep=0`; only a 3-valued `Severity` enum + 2-valued `DialTier`, per-site,
  independent.
- **No population / mean-field pool** — maturation is *single-draft* gradient-climb.
- **No combine operator** — polyclonal/igg are macro markers the audit *counts*, not an algebra.
- **No membrane** — canonical-path tuple equality = *identity*, not permeability.

So the honest guard output is mostly **PARTIAL-or-METAPHOR-summoning-a-build**, with one genuinely
near-faithful analog. **ENFORCE-grade today: NONE** — *which is the guard working as designed, not a failure.*
This vindicates ADR-066's decision to ratify the *discipline + named candidates* and defer the math.

## Per-candidate verdicts (a native math · b guard table · c verdict · d build-spec)

### 1) Quasispecies / replicator-mutator — **PARTIAL (strongest of the four)**

- **(a)** Eigen 1971 / Page-Nowak 2002: `dx_i/dt = Σ_j f_j Q_ij x_j − x_i φ(t)`; `φ` = mean fitness (the
  mean-field normalization holding `Σx_i = 1`); error threshold `L < ln(σ)/(1−q)`, `σ` = master superiority,
  **requires a single-peak landscape + the mean-field pool.**
- **(b) Guard table** — required vs supplied vs absent:
  - fitness `f_i` → **SUPPLIED, richly**: the affinity 2-vector (recall, precision) in `learn/affinity.rs` —
    deliberately a Pareto 2-vector, not a scalar (anti-Goodhart). REAL.
  - mutation operator `Q` → **SUPPLIED**: drop-a-discriminator (CDR-not-framework) in `learn/maturation.rs`. REAL.
  - antigen-depletion → **SUPPLIED**: budget-decay-as-affinity-rises. REAL.
  - selection → **SUPPLIED**: keep-if-pareto-improves stopping rule. REAL.
  - mean-field competition `φ(t)` → **ABSENT**: maturation matures ONE draft; no pool of competing fingerprints
    sharing a normalization.
  - single-peak landscape → **ABSENT**: affinity is a rugged 2-objective Pareto surface (multi-peak).
- **(c)** The **individual-lineage dynamics are faithful**; the **population layer and the error-threshold are
  not.** The "computable complexity ceiling" ADR-066 hoped for requires single-peak + mean-field, both absent —
  so it is **metaphor until antigen runs a population.**
- **(d) Build-spec:** make maturation operate on **N drafts under one shared selection budget** (= the `φ`
  pool). *Then* Eigen's threshold gives a real ceiling on fingerprint conjunct-count vs mutation-rate. The
  fitness function is the **existing affinity 2-vector**, lifted to rank a population (Pareto-front selection).

### 2) Percolation / R₀ — **METAPHOR today / ENFORCE-grade the moment the dep-graph ships. HIGHEST VALUE.**

- **(a)** `f_c = 1 − 1/R₀`; on a network `R₀ = T(⟨k²⟩/⟨k⟩ − 1)`; targeted (hub) immunization ≪ random because
  it kills the `⟨k²⟩` divergence (Pastor-Satorras-Vespignani 2001; Cohen et al. 2000/2003; Newman 2002). All
  four faces (vanishing threshold / random-failure robustness / random-immunization futility / hub-immunization
  power) are **one fact: the second moment `⟨k²⟩`.**
- **(b) Guard:** needs a graph with a degree distribution `P(k)` [**ABSENT** — antigen has no dep/call/import
  graph], a transmission-along-edges process [**ABSENT**], locally-tree-like structure [unknown], heavy tail for
  threshold-vanishing [unknown]. Substrate absent across the board.
- **(c)** **Metaphor-only today** (no substrate) — but the cleanest math and highest payoff. It needs only **two
  moments `⟨k⟩, ⟨k²⟩`.**
- **(d) Build-spec:** ingest the crate/module/call graph (**cheap** — `cargo metadata` + the `syn` parse antigen
  already runs). Then percolation yields **blast-radius + a hub-immunization priority ranking** for free.
  `f_c = 1 − 1/(⟨k²⟩/⟨k⟩ − 1)` for random; hub-targeting for the cheap-coverage ranking.

### 3) Hill kinetics — **PARTIAL (closes ADR-024 cleanly; cooperativity-n is a build, and our mapping was over-eager)**

- **(a)** `θ = [L]ⁿ/(K_d + [L]ⁿ)`, bounded `[0,1]`, **saturating**. Hill 1910; Weiss 1997: `n` is an *effective*
  interaction parameter, `n ≤ #sites`, and **`n > 1` requires physical site-site coupling** (independent sites
  → `n = 1`, full stop).
- **(b/c)** **Saturation is FAITHFUL and resolves the superpose-vs-saturate conflict correctly:** mitigation
  "lowers, doesn't sum past 1" *is* saturation; evidence "summing" is the low-occupancy near-linear toe. **This
  closes the ADR-024 seam.** BUT the **clonal/igg/polyclonal → cooperativity-`n>1` mapping is UNFAITHFUL as
  drawn** (the expansionist's enthusiasm): antigen's multiple antibodies at a site are **independent** (no
  coupling), so forcing `n>1` is the guard's falsifier (a demanded coupling term with no correspondent).
- **(d) Honest mapping:** the qualifiers set `K_d` / per-evidence weight; **`n` stays ~1** (independent
  saturating evidence) *unless* antigen models a real interaction (corroborating evidence meaningful only
  jointly = genuine cooperativity → then `n>1` is earned). Build-spec: a per-site saturating combine `θ` over
  evidence, qualifier → `K_d`, **`n=1` default**, `n>1` only behind an explicit coupling declaration.

### 4) Reaction-diffusion — **PARTIAL (sound NEW model — not Fick; summons graph AND field). Lower priority.**

- **(a/b)** The **graph-Laplacian RD is sound on its own linear algebra** (NOT inheriting Fick's proof): `−cL`
  needs only nonnegative symmetric edge weights → PSD Laplacian; conserved quantity = `Σ node-values`
  (`1ᵀL = 0`); and the **key result** — the steady state `(cL + diag(d))u* = s` is **unique iff `diag(d) ≻ 0`**
  (a **sink at each node**). The sink/decay term replaces the flux boundary the continuum needed.
- **(c)** This **vindicates ADR-066's instinct**: reaction-diffusion (not pure diffusion) is sound *because* the
  **antibody-as-sink IS the `d>0` decay term** pure-diffusion lacked — and that is exactly what makes "residual
  loudness = steady state" **well-defined and unique.** So §5/§11's "quiescent under policy P" object is real
  *once* graph+field+sinks exist. But all of graph, field, sources, sinks are **absent.**
- **(d)** Lower priority than percolation: needs *more* absent substrate (graph + field + sources + sinks) for
  less immediate signal. Build it **after** the graph + a field representation land.

## The membrane primitive (three maths converge)

Quasispecies (federation = a second competition pool), percolation (host↔federation = a compartment boundary
reshaping the graph), and reaction-diffusion (the membrane *is* the Dirichlet/flux boundary) all demand the same
absent primitive. **Minimal primitive:** a typed **boundary** between compartments (host, federation) carrying a
**permeability predicate** = *"is this fingerprint re-verifiable in the destination?"* **Flux law:** a fingerprint
crosses iff it re-verifies destination-side (decidable → permeable); asserted / host-local-trust does **not**
cross (impermeable). **antigen already has the predicate** (re-verifiable-or-it-doesn't-count) — it just isn't a
first-class boundary object yet. This is the discrete flux-boundary all three continuous-analog maths demand.

## One-paragraph verdict + the keystone

**ENFORCE-grade today: NONE** (all four need absent substrate — the guard working). **PARTIAL:** quasispecies
(lineage-faithful; population + error-threshold are a build), reaction-diffusion (sound model; summons
graph+field), Hill (saturation faithful + closes ADR-024; cooperativity-`n` is a build, `n=1` default).
**METAPHOR-today / ENFORCE-on-build:** percolation (no graph yet, cleanest math, highest value).

**SINGLE HIGHEST-VALUE PIECE TO BUILD FIRST: the dependency/code graph.** It is the shared missing substrate
**three candidates need at once** (percolation directly, reaction-diffusion as its manifold, the membrane as its
boundary), it is **cheap** (`cargo metadata` + `syn` already parsed), and percolation extracts blast-radius + a
hub-immunization priority from just `⟨k⟩, ⟨k²⟩`. **Build the graph and three of four maths acquire their
substrate simultaneously.** The graph is the keystone.

**Convergence with ADR-066 §2:** that keystone graph *is* the **sovereign stroma-builder** ADR-066 §2 already
named as a required dependency. So the graph is the keystone of **both** halves of ADR-066 — binding-identity
soundness (§2) and the field-math (§4). One build unlocks both. → see the stroma-builder ADR (next).
